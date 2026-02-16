//! Extract TMD 3D models from PROT.DAT
//!
//! Scans for TMD files and extracts them with metadata.
//! TMD files are PSX 3D model format containing vertices, normals, and primitives.

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
        .unwrap_or_else(|| "./extracted_tmds".to_string());

    println!("Opening disc: {}", disc_path);
    let cdrom = CdRom::open(&disc_path).context("Failed to open disc image")?;

    println!("Finding PROT.DAT...");
    let entries = cdrom.read_dir("/")?;
    let prot_entry = entries
        .iter()
        .find(|f| f.name.eq_ignore_ascii_case("PROT.DAT"))
        .context("PROT.DAT not found on disc")?;

    println!(
        "Scanning PROT.DAT ({} MB at LBA {}) for TMD models...",
        prot_entry.size / 1024 / 1024,
        prot_entry.lba
    );
    let tmds = scan_prot_dat_for_tmds(&cdrom, prot_entry)?;

    println!("Found {} TMD models", tmds.len());

    if tmds.is_empty() {
        println!("No TMD models found.");
        return Ok(());
    }

    // Create output directory
    fs::create_dir_all(&output_dir).context("Failed to create output directory")?;

    // Extract each TMD
    for (index, tmd_info) in tmds.iter().enumerate() {
        let filename = format!("model_{:04}.tmd", index);
        let output_path = PathBuf::from(&output_dir).join(&filename);

        println!(
            "[{}/{}] Extracting {} (offset: {:#010x}, size: {} bytes, objects: {})",
            index + 1,
            tmds.len(),
            filename,
            tmd_info.offset,
            tmd_info.size,
            tmd_info.object_count
        );

        // Read TMD data from disc
        // Calculate which sector contains the TMD and the offset within that sector
        let sectors_to_skip = tmd_info.offset / 2048;
        let offset_in_sector = tmd_info.offset % 2048;
        let tmd_lba = prot_entry.lba + sectors_to_skip as u32;

        // Read enough data to include the TMD plus the offset within the sector
        let total_bytes_to_read = offset_in_sector + tmd_info.size;
        let tmd_data_with_offset =
            cdrom
                .read_data(tmd_lba, total_bytes_to_read)
                .context(format!(
                    "Failed to read TMD at offset {:#x}",
                    tmd_info.offset
                ))?;

        // Skip the bytes before the TMD start
        let tmd_data = &tmd_data_with_offset[offset_in_sector..];

        // Write TMD file
        fs::write(&output_path, tmd_data).context(format!("Failed to write {}", filename))?;
    }

    println!("\nExtraction complete!");
    println!("Output directory: {}", output_dir);

    Ok(())
}

/// Information about a discovered TMD model
struct TmdInfo {
    offset: usize,
    size: usize,
    object_count: u32,
}

/// Scan PROT.DAT for TMD models in chunks (since file is >100MB)
fn scan_prot_dat_for_tmds(
    cdrom: &CdRom,
    prot_entry: &psxutils::cdrom::DirectoryEntry,
) -> Result<Vec<TmdInfo>> {
    const CHUNK_SIZE: usize = 10 * 1024 * 1024; // 10 MB chunks
    const OVERLAP: usize = 4096; // Overlap to catch TMDs spanning chunk boundaries

    let file_size = prot_entry.size as usize;
    let file_lba = prot_entry.lba;
    let mut all_tmds = Vec::new();

    println!("  Scanning in {} MB chunks...", CHUNK_SIZE / 1024 / 1024);

    let mut file_offset = 0;
    while file_offset < file_size {
        let chunk_size = CHUNK_SIZE.min(file_size - file_offset);
        let sectors_to_skip = file_offset / 2048;
        let chunk_lba = file_lba + sectors_to_skip as u32;

        let chunk_data = cdrom.read_data(chunk_lba, chunk_size)?;
        let chunk_tmds = scan_tmd_models(&chunk_data)?;

        // Adjust offsets to be relative to file start
        for mut tmd in chunk_tmds {
            tmd.offset += file_offset;
            all_tmds.push(tmd);
        }

        print!(
            "  Progress: {}/{} MB\r",
            file_offset / 1024 / 1024,
            file_size / 1024 / 1024
        );

        // Move forward, but overlap to catch boundary cases
        if file_offset + chunk_size < file_size {
            file_offset += chunk_size - OVERLAP;
        } else {
            break;
        }
    }

    println!(
        "  Progress: {}/{} MB - Complete!",
        file_size / 1024 / 1024,
        file_size / 1024 / 1024
    );
    Ok(all_tmds)
}

/// Scan for TMD models in data
fn scan_tmd_models(data: &[u8]) -> Result<Vec<TmdInfo>> {
    const TMD_MAGIC: u32 = 0x00000041;
    let mut tmds = Vec::new();
    let mut offset = 0;

    while offset + 12 <= data.len() {
        // Check for TMD magic number
        if let Some(magic_bytes) = data.get(offset..offset + 4) {
            let magic = u32::from_le_bytes([
                magic_bytes[0],
                magic_bytes[1],
                magic_bytes[2],
                magic_bytes[3],
            ]);

            if magic == TMD_MAGIC {
                // Read object count from header
                if let Some(count_bytes) = data.get(offset + 8..offset + 12) {
                    let object_count = u32::from_le_bytes([
                        count_bytes[0],
                        count_bytes[1],
                        count_bytes[2],
                        count_bytes[3],
                    ]);

                    // Validate object count (sanity check)
                    if object_count > 0 && object_count <= 100 {
                        // Try to estimate TMD size
                        if let Some(size) = estimate_tmd_size(data, offset, object_count) {
                            tmds.push(TmdInfo {
                                offset,
                                size,
                                object_count,
                            });

                            // Skip past this TMD
                            offset += size;
                            continue;
                        }
                    }
                }
            }
        }

        offset += 4; // Align to 4-byte boundaries for faster scanning
    }

    Ok(tmds)
}

/// Estimate the size of a TMD file
///
/// TMD format doesn't have an explicit size field, so we calculate it
/// from the object table entries.
fn estimate_tmd_size(data: &[u8], offset: usize, object_count: u32) -> Option<usize> {
    // TMD structure:
    // - Header: 12 bytes
    // - Object table: object_count * 28 bytes
    // - Data (vertices, normals, primitives): variable

    let header_size = 12;
    let obj_table_size = object_count as usize * 28;
    let obj_table_offset = offset + header_size;

    if obj_table_offset + obj_table_size > data.len() {
        return None; // Object table doesn't fit
    }

    // Find the maximum extent by reading object table entries
    // Start with end of object table as minimum size
    let mut max_extent = offset + header_size + obj_table_size;

    for i in 0..object_count as usize {
        let entry_offset = obj_table_offset + (i * 28);
        let entry = &data[entry_offset..entry_offset + 28];

        // Read offsets and counts from object entry
        // Offsets are relative to TMD start, not absolute
        let vert_offset = u32::from_le_bytes([entry[0], entry[1], entry[2], entry[3]]) as usize;
        let vert_count = u32::from_le_bytes([entry[4], entry[5], entry[6], entry[7]]) as usize;

        let normal_offset = u32::from_le_bytes([entry[8], entry[9], entry[10], entry[11]]) as usize;
        let normal_count =
            u32::from_le_bytes([entry[12], entry[13], entry[14], entry[15]]) as usize;

        let prim_offset = u32::from_le_bytes([entry[16], entry[17], entry[18], entry[19]]) as usize;
        let prim_count = u32::from_le_bytes([entry[20], entry[21], entry[22], entry[23]]) as usize;

        // Sanity check: offsets and counts should be reasonable
        if vert_count > 10000 || normal_count > 10000 || prim_count > 10000 {
            return None; // Probably not a valid TMD
        }

        // Calculate extents (add offset to convert to absolute)
        let vert_end = offset + vert_offset + (vert_count * 8);
        let normal_end = offset + normal_offset + (normal_count * 8);
        // Primitives are variable size, estimate conservatively
        let prim_end = offset + prim_offset + (prim_count * 32); // 32 bytes per prim is upper bound

        max_extent = max_extent.max(vert_end).max(normal_end).max(prim_end);
    }

    // Return size relative to TMD start
    Some(max_extent - offset)
}
