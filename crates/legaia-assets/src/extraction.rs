//! Asset extraction service
//!
//! Provides high-level API for extracting and converting assets from PSX disc.

use crate::converter::tmd_to_gltf;
use anyhow::{Context, Result};
use psxutils::cdrom::CdRom;
use psxutils::formats::{Tim, Tmd, Vag};
use std::fs;
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::Arc;

/// Progress callback for extraction
pub type ProgressCallback = Arc<dyn Fn(ExtractionProgress) + Send + Sync>;

/// Extraction progress information
#[derive(Debug, Clone)]
pub struct ExtractionProgress {
    /// Current file being processed
    pub current_file: String,
    /// Total files to process
    pub total_files: usize,
    /// Files processed so far
    pub processed_files: usize,
    /// Files successfully converted
    pub converted_files: usize,
    /// Current step description
    pub step: String,
}

impl ExtractionProgress {
    /// Calculate progress as 0.0 to 1.0
    pub fn fraction(&self) -> f32 {
        if self.total_files == 0 {
            0.0
        } else {
            self.processed_files as f32 / self.total_files as f32
        }
    }
}

/// Asset extraction service
pub struct AssetExtractionService {
    disc_path: PathBuf,
    output_dir: PathBuf,
    progress_callback: Option<ProgressCallback>,
}

impl AssetExtractionService {
    /// Create a new extraction service
    pub fn new(disc_path: PathBuf, output_dir: PathBuf) -> Self {
        Self {
            disc_path,
            output_dir,
            progress_callback: None,
        }
    }

    /// Set progress callback
    pub fn with_progress_callback(mut self, callback: ProgressCallback) -> Self {
        self.progress_callback = Some(callback);
        self
    }

    /// Extract all assets from disc
    pub fn extract_all(&self) -> Result<ExtractionStats> {
        // Open disc
        self.report_progress(ExtractionProgress {
            current_file: String::new(),
            total_files: 0,
            processed_files: 0,
            converted_files: 0,
            step: "Opening disc...".to_string(),
        });

        let cdrom = CdRom::open(&self.disc_path)
            .with_context(|| format!("Failed to open disc: {}", self.disc_path.display()))?;

        // Create output directory
        fs::create_dir_all(&self.output_dir).with_context(|| {
            format!(
                "Failed to create output directory: {}",
                self.output_dir.display()
            )
        })?;

        // Read directory entries
        self.report_progress(ExtractionProgress {
            current_file: String::new(),
            total_files: 0,
            processed_files: 0,
            converted_files: 0,
            step: "Reading directory...".to_string(),
        });

        let entries = cdrom.read_dir("/")?;

        // Filter to only files (not directories)
        let files: Vec<_> = entries.iter().filter(|e| !e.is_dir).collect();

        let total_files = files.len();
        let processed = AtomicUsize::new(0);
        let converted = AtomicUsize::new(0);

        // Extract each file
        for entry in &files {
            let current = processed.fetch_add(1, Ordering::SeqCst);

            self.report_progress(ExtractionProgress {
                current_file: entry.name.clone(),
                total_files,
                processed_files: current,
                converted_files: converted.load(Ordering::SeqCst),
                step: format!("Extracting {}", entry.name),
            });

            // Read file data
            match cdrom.read_file(&entry.name) {
                Ok(data) => {
                    let output_path = self.output_dir.join(&entry.name);

                    // Try to convert based on extension
                    let was_converted = if entry.name.ends_with(".TIM") {
                        self.convert_tim(&data, &output_path.with_extension("png"))
                    } else if entry.name.ends_with(".VAG") {
                        self.convert_vag(&data, &output_path.with_extension("wav"))
                    } else if entry.name.ends_with(".TMD") {
                        self.convert_tmd(&data, &output_path.with_extension("gltf"))
                    } else {
                        // Unknown format, just save raw data
                        if let Err(e) = fs::write(&output_path, &data) {
                            tracing::warn!("Failed to write {}: {}", entry.name, e);
                            false
                        } else {
                            true
                        }
                    };

                    if was_converted {
                        converted.fetch_add(1, Ordering::SeqCst);
                    }
                }
                Err(e) => {
                    tracing::warn!("Failed to read {}: {}", entry.name, e);
                }
            }
        }

        let final_processed = processed.load(Ordering::SeqCst);
        let final_converted = converted.load(Ordering::SeqCst);

        self.report_progress(ExtractionProgress {
            current_file: String::new(),
            total_files,
            processed_files: final_processed,
            converted_files: final_converted,
            step: "Complete!".to_string(),
        });

        Ok(ExtractionStats {
            total_files,
            extracted_files: final_processed,
            converted_files: final_converted,
        })
    }

    /// Convert TIM texture to PNG
    fn convert_tim(&self, data: &[u8], output_path: &Path) -> bool {
        match Tim::parse(data) {
            Ok(tim) => match tim.to_rgba8() {
                Ok(rgba_data) => {
                    if let Err(e) = image::save_buffer(
                        output_path,
                        &rgba_data,
                        tim.width() as u32,
                        tim.height() as u32,
                        image::ColorType::Rgba8,
                    ) {
                        tracing::warn!("Failed to save PNG: {}", e);
                        false
                    } else {
                        tracing::debug!("Converted TIM → PNG: {}", output_path.display());
                        true
                    }
                }
                Err(e) => {
                    tracing::warn!("Failed to convert TIM to RGBA: {}", e);
                    false
                }
            },
            Err(e) => {
                tracing::warn!("Failed to parse TIM: {}", e);
                false
            }
        }
    }

    /// Convert VAG audio to WAV
    fn convert_vag(&self, data: &[u8], output_path: &Path) -> bool {
        match Vag::parse(data) {
            Ok(vag) => {
                let pcm_samples = vag.decode_to_pcm();

                let spec = hound::WavSpec {
                    channels: 1,
                    sample_rate: vag.sample_rate,
                    bits_per_sample: 16,
                    sample_format: hound::SampleFormat::Int,
                };

                match hound::WavWriter::create(output_path, spec) {
                    Ok(mut writer) => {
                        for sample in pcm_samples {
                            if let Err(e) = writer.write_sample(sample) {
                                tracing::warn!("Failed to write WAV sample: {}", e);
                                return false;
                            }
                        }
                        if let Err(e) = writer.finalize() {
                            tracing::warn!("Failed to finalize WAV: {}", e);
                            return false;
                        }
                        tracing::debug!("Converted VAG → WAV: {}", output_path.display());
                        true
                    }
                    Err(e) => {
                        tracing::warn!("Failed to create WAV writer: {}", e);
                        false
                    }
                }
            }
            Err(e) => {
                tracing::warn!("Failed to parse VAG: {}", e);
                false
            }
        }
    }

    /// Convert TMD model to glTF
    fn convert_tmd(&self, data: &[u8], output_path: &Path) -> bool {
        match Tmd::parse(data) {
            Ok(tmd) => {
                if let Err(e) = tmd_to_gltf(&tmd, output_path) {
                    tracing::warn!("Failed to convert TMD to glTF: {}", e);
                    false
                } else {
                    tracing::debug!("Converted TMD → glTF: {}", output_path.display());
                    true
                }
            }
            Err(e) => {
                tracing::warn!("Failed to parse TMD: {}", e);
                false
            }
        }
    }

    /// Report progress via callback
    fn report_progress(&self, progress: ExtractionProgress) {
        if let Some(callback) = &self.progress_callback {
            callback(progress);
        }
    }
}

/// Statistics about extraction
#[derive(Debug, Clone)]
pub struct ExtractionStats {
    /// Total files found on disc
    pub total_files: usize,
    /// Files successfully extracted
    pub extracted_files: usize,
    /// Files successfully converted to modern formats
    pub converted_files: usize,
}
