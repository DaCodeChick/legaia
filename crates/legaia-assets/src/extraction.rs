//! Asset extraction service
//!
//! Provides high-level API for extracting and converting assets from PSX disc.

use crate::converter::tmd_to_gltf;
use anyhow::{Context, Result};
use psxutils::cdrom::CdRom;
use psxutils::formats::{DatArchive, Tim, Tmd, Vag};
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

        // Collect all files recursively
        self.report_progress(ExtractionProgress {
            current_file: String::new(),
            total_files: 0,
            processed_files: 0,
            converted_files: 0,
            step: "Scanning directories...".to_string(),
        });

        let all_files = self.collect_files_recursive(&cdrom, "/", &self.output_dir)?;
        let total_files = all_files.len();
        let processed = AtomicUsize::new(0);
        let converted = AtomicUsize::new(0);

        // Extract each file
        for (disc_path, output_path) in &all_files {
            let current = processed.fetch_add(1, Ordering::SeqCst);

            self.report_progress(ExtractionProgress {
                current_file: disc_path.clone(),
                total_files,
                processed_files: current,
                converted_files: converted.load(Ordering::SeqCst),
                step: format!("Extracting {}", disc_path),
            });

            // Read file data
            match cdrom.read_file(disc_path) {
                Ok(data) => {
                    // Create parent directory if needed
                    if let Some(parent) = output_path.parent() {
                        let _ = fs::create_dir_all(parent);
                    }

                    // Try to convert based on extension
                    let was_converted = if disc_path.ends_with(".TIM") {
                        self.convert_tim(&data, &output_path.with_extension("png"))
                    } else if disc_path.ends_with(".VAG") {
                        self.convert_vag(&data, &output_path.with_extension("wav"))
                    } else if disc_path.ends_with(".TMD") {
                        self.convert_tmd(&data, &output_path.with_extension("gltf"))
                    } else if disc_path.ends_with(".DAT") {
                        // Try to extract DAT archive
                        self.extract_dat_archive(&data, output_path, disc_path)
                    } else {
                        // Unknown format, just save raw data
                        if let Err(e) = fs::write(output_path, &data) {
                            tracing::warn!("Failed to write {}: {}", disc_path, e);
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
                    tracing::warn!("Failed to read {}: {}", disc_path, e);
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

    /// Recursively collect all files from disc
    fn collect_files_recursive(
        &self,
        cdrom: &CdRom,
        dir_path: &str,
        output_base: &Path,
    ) -> Result<Vec<(String, PathBuf)>> {
        let mut files = Vec::new();
        let entries = cdrom.read_dir(dir_path)?;

        for entry in entries {
            let full_path = if dir_path == "/" {
                format!("/{}", entry.name)
            } else {
                format!("{}/{}", dir_path, entry.name)
            };

            let output_path = output_base.join(&entry.name);

            if entry.is_dir {
                // Recursively scan subdirectory
                let subdir_files = self.collect_files_recursive(cdrom, &full_path, &output_path)?;
                files.extend(subdir_files);
            } else {
                // Add file to list
                files.push((full_path, output_path));
            }
        }

        Ok(files)
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

    /// Extract DAT archive and save files
    fn extract_dat_archive(&self, data: &[u8], output_path: &Path, disc_path: &str) -> bool {
        match DatArchive::parse(data) {
            Ok(archive) => {
                tracing::info!(
                    "Extracting DAT archive: {} ({} files)",
                    disc_path,
                    archive.entry_count()
                );

                // Create archive directory
                let archive_dir = output_path.with_extension("");
                if let Err(e) = fs::create_dir_all(&archive_dir) {
                    tracing::warn!(
                        "Failed to create archive directory {}: {}",
                        archive_dir.display(),
                        e
                    );
                    return false;
                }

                let mut extracted_count = 0;

                // Extract each file from archive
                for index in 0..archive.entry_count() {
                    match archive.extract_file(index) {
                        Ok(file_data) => {
                            // Generate filename
                            let filename = format!("file_{:04}.bin", index);
                            let file_path = archive_dir.join(&filename);

                            // Try to convert if recognized format
                            let converted = if file_data.len() >= 4 {
                                match &file_data[0..4] {
                                    [0x10, 0x00, 0x00, 0x00] => {
                                        // TIM texture
                                        self.convert_tim(
                                            file_data,
                                            &file_path.with_extension("png"),
                                        )
                                    }
                                    [b'V', b'A', b'G', b'p'] => {
                                        // VAG audio
                                        self.convert_vag(
                                            file_data,
                                            &file_path.with_extension("wav"),
                                        )
                                    }
                                    _ => {
                                        // Unknown format, save as bin
                                        if let Err(e) = fs::write(&file_path, file_data) {
                                            tracing::warn!(
                                                "Failed to write {}: {}",
                                                file_path.display(),
                                                e
                                            );
                                            false
                                        } else {
                                            true
                                        }
                                    }
                                }
                            } else {
                                // File too small, save as-is
                                if let Err(e) = fs::write(&file_path, file_data) {
                                    tracing::warn!(
                                        "Failed to write {}: {}",
                                        file_path.display(),
                                        e
                                    );
                                    false
                                } else {
                                    true
                                }
                            };

                            if converted {
                                extracted_count += 1;
                            }
                        }
                        Err(e) => {
                            tracing::warn!(
                                "Failed to extract file {} from {}: {}",
                                index,
                                disc_path,
                                e
                            );
                        }
                    }
                }

                tracing::info!(
                    "Extracted {}/{} files from {}",
                    extracted_count,
                    archive.entry_count(),
                    disc_path
                );
                extracted_count > 0
            }
            Err(e) => {
                tracing::warn!(
                    "Failed to parse {} as DAT archive: {}, saving as raw file",
                    disc_path,
                    e
                );
                // Not a DAT archive, save as raw file
                if let Err(e) = fs::write(output_path, data) {
                    tracing::warn!("Failed to write {}: {}", output_path.display(), e);
                    false
                } else {
                    true
                }
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
