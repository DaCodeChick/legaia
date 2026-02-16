//! Extract VAB sound banks from disc
//!
//! Scans for VAB files and extracts them with metadata.
//! VAB files contain multiple VAG samples organized into programs.

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
        .unwrap_or_else(|| "./extracted_vabs".to_string());

    println!("Opening disc: {}", disc_path);
    let cdrom = CdRom::open(&disc_path).context("Failed to open disc image")?;

    println!("Scanning disc for VAB files...");
    let mut vabs = Vec::new();

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

        // Scan for VAB files in this data
        let file_vabs = scan_vab_files(&data, &file.name)?;
        if !file_vabs.is_empty() {
            println!("  Found {} VAB(s) in {}", file_vabs.len(), file.name);
            vabs.extend(file_vabs);
        }
    }

    println!("\nFound {} total VAB sound banks", vabs.len());

    if vabs.is_empty() {
        println!("No VAB files found.");
        return Ok(());
    }

    // Create output directory
    fs::create_dir_all(&output_dir).context("Failed to create output directory")?;

    // Extract each VAB
    for (index, vab_info) in vabs.iter().enumerate() {
        let filename = if vab_info.source_file.is_empty() {
            format!("soundbank_{:04}.vab", index)
        } else {
            format!("{}_{:04}.vab", vab_info.source_file, index)
        };
        let output_path = PathBuf::from(&output_dir).join(&filename);

        println!(
            "[{}/{}] Extracting {} (size: {} bytes, programs: {}, vags: {})",
            index + 1,
            vabs.len(),
            filename,
            vab_info.size,
            vab_info.num_programs,
            vab_info.num_vags
        );

        // Write VAB file
        fs::write(&output_path, &vab_info.data).context(format!("Failed to write {}", filename))?;
    }

    println!("\nExtraction complete!");
    println!("Output directory: {}", output_dir);

    Ok(())
}

/// Information about a discovered VAB file
struct VabInfo {
    data: Vec<u8>,
    source_file: String,
    size: u32,
    num_programs: u16,
    num_vags: u16,
}

/// Scan for VAB files in data
fn scan_vab_files(data: &[u8], source_name: &str) -> Result<Vec<VabInfo>> {
    const VAB_MAGIC: [u8; 4] = *b"VABp";
    let mut vabs = Vec::new();
    let mut offset = 0;

    while offset + 2048 <= data.len() {
        // Check for VAB magic number
        if data.get(offset..offset + 4) == Some(&VAB_MAGIC) {
            // Read VAB header
            if let Some(header_data) = data.get(offset..offset + 32) {
                // Parse header fields
                let version = u32::from_le_bytes([
                    header_data[4],
                    header_data[5],
                    header_data[6],
                    header_data[7],
                ]);

                // Check version (should be 0x00000007)
                if version == 0x00000007 {
                    let size = u32::from_le_bytes([
                        header_data[12],
                        header_data[13],
                        header_data[14],
                        header_data[15],
                    ]);

                    let num_programs = u16::from_le_bytes([header_data[18], header_data[19]]);
                    let num_vags = u16::from_le_bytes([header_data[22], header_data[23]]);

                    // Validate size
                    if size > 0 && size < 10_000_000 && offset + size as usize <= data.len() {
                        // Extract VAB data
                        let vab_data = data[offset..offset + size as usize].to_vec();

                        vabs.push(VabInfo {
                            data: vab_data,
                            source_file: source_name.to_string(),
                            size,
                            num_programs,
                            num_vags,
                        });

                        // Skip past this VAB
                        offset += size as usize;
                        continue;
                    }
                }
            }
        }

        offset += 2048; // VABs are aligned to 2KB boundaries
    }

    Ok(vabs)
}
