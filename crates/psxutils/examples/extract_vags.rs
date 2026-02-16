//! Extract VAG audio samples from disc
//!
//! Scans for standalone VAG files and extracts them with metadata.
//! VAG files are ADPCM compressed audio samples used by PSX games.

use anyhow::{Context, Result};
use psxutils::CdRom;
use std::fs;
use std::path::PathBuf;

fn main() -> Result<()> {
    let disc_path = std::env::args()
        .nth(1)
        .unwrap_or_else(|| "/home/admin/Downloads/Legend of Legaia.bin".to_string());

    let output_dir = std::env::args()
        .nth(2)
        .unwrap_or_else(|| "./extracted_vags".to_string());

    println!("Opening disc: {}", disc_path);
    let cdrom = CdRom::open(&disc_path).context("Failed to open disc image")?;

    println!("Scanning disc for VAG files...");
    let mut vags = Vec::new();

    // Scan all files on disc
    let entries = cdrom.read_dir("/")?;
    for file in &entries {
        if file.is_dir {
            continue; // Skip directories for now
        }

        println!("Reading {}...", file.name);
        let data = match cdrom.read_data(file.lba, file.size as usize) {
            Ok(d) => d,
            Err(e) => {
                eprintln!("  Warning: Failed to read {}: {}", file.name, e);
                continue;
            }
        };

        // Scan for VAG files in this data
        let file_vags = scan_vag_files(&data, &file.name)?;
        if !file_vags.is_empty() {
            println!("  Found {} VAG(s) in {}", file_vags.len(), file.name);
            vags.extend(file_vags);
        }
    }

    println!("\nFound {} total VAG audio samples", vags.len());

    if vags.is_empty() {
        println!("No VAG files found.");
        return Ok(());
    }

    // Create output directory
    fs::create_dir_all(&output_dir).context("Failed to create output directory")?;

    // Extract each VAG
    for (index, vag_info) in vags.iter().enumerate() {
        let name = if vag_info.name.is_empty() {
            format!("sample_{:04}", index)
        } else {
            sanitize_filename(&vag_info.name)
        };
        let filename = format!("{}_{:04}.vag", name, index);
        let output_path = PathBuf::from(&output_dir).join(&filename);

        println!(
            "[{}/{}] Extracting {} (size: {} bytes, rate: {} Hz)",
            index + 1,
            vags.len(),
            filename,
            vag_info.size,
            vag_info.sample_rate
        );

        // Write VAG file
        fs::write(&output_path, &vag_info.data).context(format!("Failed to write {}", filename))?;
    }

    println!("\nExtraction complete!");
    println!("Output directory: {}", output_dir);

    Ok(())
}

/// Information about a discovered VAG file
struct VagInfo {
    data: Vec<u8>,
    name: String,
    size: u32,
    sample_rate: u32,
}

/// Scan for VAG files in data
fn scan_vag_files(data: &[u8], source_name: &str) -> Result<Vec<VagInfo>> {
    const VAG_MAGIC: [u8; 4] = *b"VAGp";
    let mut vags = Vec::new();
    let mut offset = 0;

    while offset + 48 <= data.len() {
        // Check for VAG magic number
        if data.get(offset..offset + 4) == Some(&VAG_MAGIC) {
            // Read VAG header (48 bytes)
            if let Some(header_data) = data.get(offset..offset + 48) {
                // Parse header fields (big-endian!)
                let version = u32::from_be_bytes([
                    header_data[4],
                    header_data[5],
                    header_data[6],
                    header_data[7],
                ]);

                // Check version (should be 0x00000020)
                if version == 0x00000020 {
                    let size = u32::from_be_bytes([
                        header_data[12],
                        header_data[13],
                        header_data[14],
                        header_data[15],
                    ]);

                    let sample_rate = u32::from_be_bytes([
                        header_data[16],
                        header_data[17],
                        header_data[18],
                        header_data[19],
                    ]);

                    // Extract name (16 bytes starting at offset 32)
                    let name_bytes = &header_data[32..48];
                    let name = String::from_utf8_lossy(name_bytes)
                        .trim_end_matches('\0')
                        .to_string();

                    // Validate size (should be reasonable)
                    let total_size = size as usize + 48; // Add header size
                    if size > 0 && size < 10_000_000 && offset + total_size <= data.len() {
                        // Extract VAG data (header + audio)
                        let vag_data = data[offset..offset + total_size].to_vec();

                        vags.push(VagInfo {
                            data: vag_data,
                            name,
                            size,
                            sample_rate,
                        });

                        // Skip past this VAG
                        offset += total_size;
                        continue;
                    }
                }
            }
        }

        offset += 16; // VAG blocks are 16 bytes
    }

    Ok(vags)
}

/// Sanitize filename by removing invalid characters
fn sanitize_filename(name: &str) -> String {
    name.chars()
        .map(|c| {
            if c.is_alphanumeric() || c == '_' || c == '-' {
                c
            } else {
                '_'
            }
        })
        .collect()
}
