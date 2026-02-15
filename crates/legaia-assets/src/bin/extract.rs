#!/usr/bin/env rust
//! Legend of Legaia asset extraction CLI tool
//!
//! Extracts and converts assets from PSX disc images to modern formats.

use anyhow::{Context, Result};
use clap::{Parser, Subcommand};
use legaia_assets::converter::tmd_to_gltf;
use psxutils::cdrom::CdRom;
use psxutils::formats::{Tim, Tmd, Vag};
use std::fs;
use std::path::PathBuf;
use tracing::{info, warn};

#[derive(Parser)]
#[command(name = "legaia-extract")]
#[command(about = "Extract and convert Legend of Legaia assets from PSX disc")]
#[command(version)]
struct Cli {
    #[command(subcommand)]
    command: Commands,

    /// Enable verbose logging
    #[arg(short, long, global = true)]
    verbose: bool,
}

#[derive(Subcommand)]
enum Commands {
    /// List all files on the disc
    List {
        /// Path to PSX disc image (.bin file)
        #[arg(short, long)]
        disc: PathBuf,
    },

    /// Extract a specific file from the disc
    Extract {
        /// Path to PSX disc image (.bin file)
        #[arg(short, long)]
        disc: PathBuf,

        /// File path on disc (e.g., "SYSTEM.CNF")
        #[arg(short, long)]
        file: String,

        /// Output file path
        #[arg(short, long)]
        output: PathBuf,
    },

    /// Convert TIM texture to PNG
    ConvertTim {
        /// Input TIM file
        input: PathBuf,

        /// Output PNG file
        output: PathBuf,
    },

    /// Convert VAG audio to WAV
    ConvertVag {
        /// Input VAG file
        input: PathBuf,

        /// Output WAV file
        output: PathBuf,
    },

    /// Show TMD model info
    InfoTmd {
        /// Input TMD file
        input: PathBuf,
    },

    /// Convert TMD model to glTF
    ConvertTmd {
        /// Input TMD file
        input: PathBuf,

        /// Output glTF file (.gltf)
        output: PathBuf,
    },

    /// Extract all assets from disc
    ExtractAll {
        /// Path to PSX disc image (.bin file)
        #[arg(short, long)]
        disc: PathBuf,

        /// Output directory for extracted assets
        #[arg(short, long)]
        output: PathBuf,

        /// Asset type to extract (textures, audio, models, all)
        #[arg(short, long, default_value = "all")]
        r#type: String,
    },
}

fn main() -> Result<()> {
    let cli = Cli::parse();

    // Initialize tracing
    let subscriber = tracing_subscriber::fmt()
        .with_max_level(if cli.verbose {
            tracing::Level::DEBUG
        } else {
            tracing::Level::INFO
        })
        .finish();
    tracing::subscriber::set_global_default(subscriber)?;

    match cli.command {
        Commands::List { disc } => list_files(&disc)?,
        Commands::Extract { disc, file, output } => extract_file(&disc, &file, &output)?,
        Commands::ConvertTim { input, output } => convert_tim(&input, &output)?,
        Commands::ConvertVag { input, output } => convert_vag(&input, &output)?,
        Commands::InfoTmd { input } => info_tmd(&input)?,
        Commands::ConvertTmd { input, output } => convert_tmd(&input, &output)?,
        Commands::ExtractAll {
            disc,
            output,
            r#type,
        } => extract_all(&disc, &output, &r#type)?,
    }

    Ok(())
}

fn list_files(disc_path: &PathBuf) -> Result<()> {
    info!("Opening disc: {}", disc_path.display());
    let cdrom = CdRom::open(disc_path)
        .with_context(|| format!("Failed to open disc: {}", disc_path.display()))?;

    info!("Reading root directory...");
    let entries = cdrom.read_dir("/")?;

    println!("\nFiles on disc:");
    println!("{:<40} {:>12} {:>10}", "Name", "Size (bytes)", "LBA");
    println!("{}", "-".repeat(64));

    for entry in &entries {
        let type_str = if entry.is_dir { "[DIR]" } else { "" };
        println!(
            "{:<40} {:>12} {:>10} {}",
            entry.name, entry.size, entry.lba, type_str
        );
    }

    println!("\nTotal: {} entries", entries.len());
    Ok(())
}

fn extract_file(disc_path: &PathBuf, file_path: &str, output_path: &PathBuf) -> Result<()> {
    info!("Opening disc: {}", disc_path.display());
    let cdrom = CdRom::open(disc_path)?;

    info!("Reading file: {}", file_path);
    let data = cdrom.read_file(file_path)?;

    info!("Writing to: {}", output_path.display());
    fs::write(output_path, &data)?;

    info!("Extracted {} bytes", data.len());
    Ok(())
}

fn convert_tim(input: &PathBuf, output: &PathBuf) -> Result<()> {
    info!("Reading TIM: {}", input.display());
    let data = fs::read(input)?;

    info!("Parsing TIM...");
    let tim = Tim::parse(&data)?;

    info!("Converting to PNG ({}x{})...", tim.width(), tim.height());
    let rgba_data = tim.to_rgba8()?;

    info!("Saving to: {}", output.display());
    image::save_buffer(
        output,
        &rgba_data,
        tim.width() as u32,
        tim.height() as u32,
        image::ColorType::Rgba8,
    )?;

    info!("Conversion complete!");
    Ok(())
}

fn convert_vag(input: &PathBuf, output: &PathBuf) -> Result<()> {
    info!("Reading VAG: {}", input.display());
    let data = fs::read(input)?;

    info!("Parsing VAG...");
    let vag = Vag::parse(&data)?;

    info!(
        "Decoding ADPCM: {} Hz, {:.2}s duration",
        vag.sample_rate,
        vag.duration_secs()
    );
    let pcm_samples = vag.decode_to_pcm();

    info!("Writing WAV: {}", output.display());
    let spec = hound::WavSpec {
        channels: 1,
        sample_rate: vag.sample_rate,
        bits_per_sample: 16,
        sample_format: hound::SampleFormat::Int,
    };

    let mut writer = hound::WavWriter::create(output, spec)?;
    for sample in pcm_samples {
        writer.write_sample(sample)?;
    }
    writer.finalize()?;

    info!("Conversion complete!");
    Ok(())
}

fn info_tmd(input: &PathBuf) -> Result<()> {
    info!("Reading TMD: {}", input.display());
    let data = fs::read(input)?;

    info!("Parsing TMD...");
    let tmd = Tmd::parse(&data)?;

    println!("\nTMD Model Information:");
    println!("  Flags: {:#010x}", tmd.flags);
    println!("  Objects: {}", tmd.object_count());

    for (i, obj) in tmd.objects.iter().enumerate() {
        println!("\n  Object {}:", i);
        println!("    Vertices: {}", obj.vertices.len());
        println!("    Normals: {}", obj.normals.len());
        println!("    Primitives: {}", obj.primitives.len());
        println!("    Scale: {}", obj.scale);
    }

    Ok(())
}

fn convert_tmd(input: &PathBuf, output: &PathBuf) -> Result<()> {
    info!("Reading TMD: {}", input.display());
    let data = fs::read(input)?;

    info!("Parsing TMD...");
    let tmd = Tmd::parse(&data)?;

    info!("Converting to glTF ({} objects)...", tmd.object_count());
    tmd_to_gltf(&tmd, output)?;

    info!("Saved glTF to: {}", output.display());
    info!("Binary buffer: {}", output.with_extension("bin").display());
    info!("Conversion complete!");
    Ok(())
}

fn extract_all(disc_path: &PathBuf, output_dir: &PathBuf, asset_type: &str) -> Result<()> {
    info!("Opening disc: {}", disc_path.display());
    let cdrom = CdRom::open(disc_path)?;

    fs::create_dir_all(output_dir)?;

    info!("Reading root directory...");
    let entries = cdrom.read_dir("/")?;

    let mut extracted_count = 0;
    let mut converted_count = 0;

    for entry in &entries {
        if entry.is_dir {
            continue;
        }

        let should_extract = match asset_type {
            "textures" => entry.name.ends_with(".TIM"),
            "audio" => entry.name.ends_with(".VAG") || entry.name.ends_with(".VAB"),
            "models" => entry.name.ends_with(".TMD"),
            "all" => true,
            _ => {
                warn!("Unknown asset type: {}", asset_type);
                false
            }
        };

        if !should_extract {
            continue;
        }

        info!("Extracting: {}", entry.name);

        match cdrom.read_file(&entry.name) {
            Ok(data) => {
                let output_path = output_dir.join(&entry.name);

                // Try to convert if it's a known format
                let converted = if entry.name.ends_with(".TIM") {
                    convert_tim_data(&data, &output_path.with_extension("png"))
                } else if entry.name.ends_with(".VAG") {
                    convert_vag_data(&data, &output_path.with_extension("wav"))
                } else if entry.name.ends_with(".TMD") {
                    convert_tmd_data(&data, &output_path.with_extension("gltf"))
                } else {
                    false
                };

                if converted {
                    converted_count += 1;
                } else {
                    // Just extract raw data
                    fs::write(&output_path, &data)?;
                }

                extracted_count += 1;
            }
            Err(e) => {
                warn!("Failed to extract {}: {}", entry.name, e);
            }
        }
    }

    info!(
        "Extraction complete! {} files extracted, {} converted",
        extracted_count, converted_count
    );

    Ok(())
}

fn convert_tim_data(data: &[u8], output_path: &PathBuf) -> bool {
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
                    warn!("Failed to convert TIM to PNG: {}", e);
                    false
                } else {
                    info!("  -> Converted to PNG: {}", output_path.display());
                    true
                }
            }
            Err(e) => {
                warn!("Failed to convert TIM to RGBA8: {}", e);
                false
            }
        },
        Err(e) => {
            warn!("Failed to parse TIM: {}", e);
            false
        }
    }
}

fn convert_vag_data(data: &[u8], output_path: &PathBuf) -> bool {
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
                            warn!("Failed to write WAV sample: {}", e);
                            return false;
                        }
                    }
                    if let Err(e) = writer.finalize() {
                        warn!("Failed to finalize WAV: {}", e);
                        return false;
                    }
                    info!("  -> Converted to WAV: {}", output_path.display());
                    true
                }
                Err(e) => {
                    warn!("Failed to create WAV writer: {}", e);
                    false
                }
            }
        }
        Err(e) => {
            warn!("Failed to parse VAG: {}", e);
            false
        }
    }
}

fn convert_tmd_data(data: &[u8], output_path: &PathBuf) -> bool {
    match Tmd::parse(data) {
        Ok(tmd) => {
            if let Err(e) = tmd_to_gltf(&tmd, output_path) {
                warn!("Failed to convert TMD to glTF: {}", e);
                false
            } else {
                info!("  -> Converted to glTF: {}", output_path.display());
                true
            }
        }
        Err(e) => {
            warn!("Failed to parse TMD: {}", e);
            false
        }
    }
}
