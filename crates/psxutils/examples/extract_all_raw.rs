//! Extract all raw files from the disc image
//!
//! This tool copies every file from the disc's ISO 9660 filesystem
//! to the output directory, preserving filenames.

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
        .unwrap_or_else(|| "./extracted_raw".to_string());

    println!("=== Raw Disc File Extractor ===");
    println!("Disc: {}", disc_path);
    println!("Output: {}\n", output_dir);

    let cdrom = CdRom::open(&disc_path).context("Failed to open disc image")?;

    // Create output directory
    fs::create_dir_all(&output_dir).context("Failed to create output directory")?;

    // Read root directory
    println!("📀 Listing disc contents...\n");
    let entries = cdrom.read_dir("/")?;

    // Display file list
    println!("Files on disc:");
    println!("{:<20} {:>12} {:>8} {}", "Name", "Size", "LBA", "Type");
    println!("{}", "-".repeat(60));

    let mut total_size: u64 = 0;
    let mut file_count = 0;
    let mut dir_count = 0;

    for entry in &entries {
        let type_str = if entry.is_dir { "DIR" } else { "FILE" };
        println!(
            "{:<20} {:>12} {:>8} {}",
            entry.name, entry.size, entry.lba, type_str
        );

        if entry.is_dir {
            dir_count += 1;
        } else {
            file_count += 1;
            total_size += entry.size as u64;
        }
    }

    println!("{}", "-".repeat(60));
    println!(
        "Total: {} files, {} directories, {:.2} MB\n",
        file_count,
        dir_count,
        total_size as f64 / 1024.0 / 1024.0
    );

    // Extract all files
    println!("💾 Extracting files...\n");

    extract_directory(&cdrom, "/", &output_dir)?;

    println!("\n✅ Extraction complete!");
    println!("Output directory: {}", output_dir);

    Ok(())
}

/// Recursively extract a directory and all its contents
fn extract_directory(cdrom: &CdRom, path: &str, output_base: &str) -> Result<()> {
    let entries = cdrom.read_dir(path)?;

    for entry in &entries {
        let full_path = if path == "/" {
            format!("/{}", entry.name)
        } else {
            format!("{}/{}", path, entry.name)
        };

        let output_path = PathBuf::from(output_base).join(entry.name.as_str());

        if entry.is_dir {
            // Create subdirectory and recurse
            fs::create_dir_all(&output_path)?;
            println!("📁 Entering directory: {}", full_path);
            extract_directory(cdrom, &full_path, output_path.to_str().unwrap())?;
        } else {
            // Extract file
            print!("Extracting {}... ", full_path);
            std::io::Write::flush(&mut std::io::stdout()).ok();

            // Handle large files (> 100MB) by reading in chunks
            const MAX_CHUNK_SIZE: usize = 50 * 1024 * 1024; // 50 MB chunks

            if entry.size as usize > MAX_CHUNK_SIZE {
                // Read in chunks
                let mut all_data = Vec::with_capacity(entry.size as usize);
                let mut remaining = entry.size as usize;
                let mut offset = 0;

                while remaining > 0 {
                    let chunk_size = remaining.min(MAX_CHUNK_SIZE);
                    let sectors_to_skip = offset / 2048;
                    let chunk_lba = entry.lba + sectors_to_skip as u32;

                    match cdrom.read_data(chunk_lba, chunk_size) {
                        Ok(chunk) => {
                            all_data.extend_from_slice(&chunk);
                            offset += chunk_size;
                            remaining -= chunk_size;
                        }
                        Err(e) => {
                            println!("✗ Error: {}", e);
                            break;
                        }
                    }
                }

                if remaining == 0 {
                    fs::write(&output_path, all_data)
                        .context(format!("Failed to write {}", entry.name))?;
                    println!("✓ ({} bytes, chunked)", entry.size);
                }
            } else {
                // Read entire file at once
                match cdrom.read_data(entry.lba, entry.size as usize) {
                    Ok(data) => {
                        fs::write(&output_path, data)
                            .context(format!("Failed to write {}", entry.name))?;
                        println!("✓ ({} bytes)", entry.size);
                    }
                    Err(e) => {
                        println!("✗ Error: {}", e);
                    }
                }
            }
        }
    }

    Ok(())
}
