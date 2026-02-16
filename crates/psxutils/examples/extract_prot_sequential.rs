//! Sequential PROT.DAT scanner - discovers all assets in order
//!
//! This tool scans PROT.DAT byte-by-byte to discover all embedded assets,
//! numbering them by discovery order (not by type).
//!
//! Detected formats:
//! - TIM textures (magic: 0x00000010)
//! - VAG audio samples (magic: "VAGp")
//! - Custom 3D models (signature: 0x80000002 at offset +4)
//! - LZSS compressed data (magic: "sszl")
//! - MIPS overlays (validated as MIPS machine code)
//! - Unknown/raw binary data (fallback)

use anyhow::{Context, Result};
use psxutils::{formats::Tim, CdRom};
use std::fs;
use std::path::Path;

#[cfg(feature = "extraction")]
use {
    indicatif::{ProgressBar, ProgressStyle},
    serde::Serialize,
};

/// Detected asset format
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "extraction", derive(Serialize))]
enum AssetFormat {
    Tim,
    Vag,
    CustomModel,
    Lzss,
    MipsOverlay,
    Unknown,
}

impl AssetFormat {
    fn extension(&self) -> &'static str {
        match self {
            AssetFormat::Tim => "tim",
            AssetFormat::Vag => "vag",
            AssetFormat::CustomModel => "model",
            AssetFormat::Lzss => "lzss",
            AssetFormat::MipsOverlay => "mips",
            AssetFormat::Unknown => "bin",
        }
    }

    fn name(&self) -> &'static str {
        match self {
            AssetFormat::Tim => "TIM texture",
            AssetFormat::Vag => "VAG audio",
            AssetFormat::CustomModel => "Custom 3D model",
            AssetFormat::Lzss => "LZSS compressed",
            AssetFormat::MipsOverlay => "MIPS overlay",
            AssetFormat::Unknown => "Unknown",
        }
    }
}

/// Discovered asset metadata
#[derive(Debug, Clone)]
#[cfg_attr(feature = "extraction", derive(Serialize))]
struct Asset {
    /// Sequential discovery index
    index: usize,
    /// Offset in PROT.DAT
    offset: usize,
    /// Size in bytes
    size: usize,
    /// Detected format
    format: AssetFormat,
    /// Output filename
    filename: String,
    /// Format-specific metadata (optional)
    #[cfg_attr(feature = "extraction", serde(skip_serializing_if = "Option::is_none"))]
    metadata: Option<String>,
}

fn main() -> Result<()> {
    #[cfg(not(feature = "extraction"))]
    {
        eprintln!("Error: This example requires the 'extraction' feature.");
        eprintln!("Run with: cargo run --example extract_prot_sequential --features extraction");
        std::process::exit(1);
    }

    #[cfg(feature = "extraction")]
    {
        run_extraction()
    }
}

#[cfg(feature = "extraction")]
fn run_extraction() -> Result<()> {
    let disc_path = std::env::args()
        .nth(1)
        .unwrap_or_else(|| "/home/admin/Downloads/Legend of Legaia.bin".to_string());

    let output_dir_str = std::env::args()
        .nth(2)
        .unwrap_or_else(|| "/home/admin/Documents/legaia_extracted/prot_sequential".to_string());
    let output_dir = Path::new(&output_dir_str);

    println!("=== PROT.DAT Sequential Scanner ===");
    println!("Disc: {}", disc_path);
    println!("Output: {}\n", output_dir.display());

    // Create output directory
    fs::create_dir_all(output_dir)?;

    println!("📀 Opening disc...");
    let disc = CdRom::open(disc_path)?;

    // Get PROT.DAT info
    let entries = disc.read_dir("/")?;
    let prot_entry = entries
        .iter()
        .find(|e| e.name.eq_ignore_ascii_case("PROT.DAT"))
        .context("PROT.DAT not found")?;

    println!("📂 PROT.DAT: {} MB", prot_entry.size / 1024 / 1024);
    println!("   Starting at LBA: {}\n", prot_entry.lba);

    // Read entire PROT.DAT (chunked internally)
    println!("💾 Reading PROT.DAT...");
    let prot_data = read_prot_dat_chunked(&disc, prot_entry)?;
    println!("✅ Loaded {} MB\n", prot_data.len() / 1024 / 1024);

    // Scan for all assets sequentially
    println!("🔍 Scanning for assets...");
    let assets = scan_sequential(&prot_data)?;

    // Print statistics
    print_statistics(&assets);

    // Extract all assets
    println!("\n📦 Extracting assets...");
    extract_assets(&prot_data, &assets, output_dir)?;

    // Write manifest
    let manifest_path = output_dir.join("manifest.json");
    let manifest_json = serde_json::to_string_pretty(&assets)?;
    fs::write(&manifest_path, manifest_json)?;

    println!("\n✅ Extraction complete!");
    println!("   Assets:  {}", output_dir.display());
    println!("   Manifest: {}", manifest_path.display());

    Ok(())
}

#[cfg(feature = "extraction")]
fn read_prot_dat_chunked(
    disc: &CdRom,
    prot_entry: &psxutils::cdrom::DirectoryEntry,
) -> Result<Vec<u8>> {
    const CHUNK_SIZE: usize = 50 * 1024 * 1024; // 50 MB chunks
    let file_size = prot_entry.size as usize;
    let file_lba = prot_entry.lba;
    let mut all_data = Vec::with_capacity(file_size);

    let pb = ProgressBar::new(file_size as u64);
    pb.set_style(
        ProgressStyle::default_bar()
            .template("   [{bar:40}] {bytes}/{total_bytes} ({percent}%)")
            .unwrap()
            .progress_chars("=>-"),
    );

    let mut file_offset = 0;
    while file_offset < file_size {
        let chunk_size = CHUNK_SIZE.min(file_size - file_offset);
        let sectors_to_skip = file_offset / 2048;
        let chunk_lba = file_lba + sectors_to_skip as u32;

        let chunk_data = disc.read_data(chunk_lba, chunk_size)?;
        all_data.extend_from_slice(&chunk_data);

        file_offset += chunk_size;
        pb.set_position(file_offset as u64);
    }

    pb.finish_with_message("Done");
    Ok(all_data)
}

#[cfg(feature = "extraction")]
fn scan_sequential(data: &[u8]) -> Result<Vec<Asset>> {
    let mut assets = Vec::new();
    let mut offset = 0;
    let mut asset_index = 0;

    let pb = ProgressBar::new(data.len() as u64);
    pb.set_style(
        ProgressStyle::default_bar()
            .template("   [{bar:40}] {bytes}/{total_bytes} ({percent}%) - {msg}")
            .unwrap()
            .progress_chars("=>-"),
    );

    while offset < data.len() {
        // Try to detect asset at current offset
        if let Some((format, size, metadata)) = detect_asset_at(&data[offset..]) {
            let filename = format!("asset_{:04}.{}", asset_index, format.extension());

            assets.push(Asset {
                index: asset_index,
                offset,
                size,
                format,
                filename,
                metadata,
            });

            pb.set_message(format!(
                "{} {} at 0x{:08X}",
                assets.len(),
                format.name(),
                offset
            ));

            offset += size;
            asset_index += 1;
        } else {
            // Move forward 1 byte and try again
            offset += 1;
        }

        // Update progress every 64KB
        if offset % (64 * 1024) == 0 {
            pb.set_position(offset as u64);
        }
    }

    pb.finish_with_message(format!("Found {} assets", assets.len()));
    Ok(assets)
}

/// Detect asset type and size at given offset
///
/// Returns (format, size, metadata) if an asset is detected, None otherwise
#[cfg(feature = "extraction")]
fn detect_asset_at(data: &[u8]) -> Option<(AssetFormat, usize, Option<String>)> {
    if data.len() < 16 {
        return None;
    }

    // Check signatures in priority order (most reliable first)

    // Check for TIM texture (magic: 0x00000010) - HIGHEST PRIORITY
    if let Some((size, metadata)) = check_tim(data) {
        return Some((AssetFormat::Tim, size, Some(metadata)));
    }

    // Check for VAG audio (magic: "VAGp")
    if let Some((size, metadata)) = check_vag(data) {
        return Some((AssetFormat::Vag, size, Some(metadata)));
    }

    // Check for LZSS compressed (magic: "sszl")
    if let Some((size, metadata)) = check_lzss(data) {
        return Some((AssetFormat::Lzss, size, Some(metadata)));
    }

    // Check for Custom 3D model (signature: 0x80000002 at offset +4)
    if let Some((size, metadata)) = check_custom_model(data) {
        return Some((AssetFormat::CustomModel, size, Some(metadata)));
    }

    // SKIP MIPS overlay detection - too unreliable with random data
    // It was catching 97k false positives. Manual identification needed.

    None
}

#[cfg(feature = "extraction")]
fn check_tim(data: &[u8]) -> Option<(usize, String)> {
    const TIM_MAGIC: u32 = 0x00000010;

    if data.len() < 8 {
        return None;
    }

    let magic = u32::from_le_bytes([data[0], data[1], data[2], data[3]]);
    if magic != TIM_MAGIC {
        return None;
    }

    // Use TIM validator to get size
    match Tim::validate(data) {
        Ok((width, height, size)) => {
            let metadata = format!("{}x{}", width, height);
            Some((size, metadata))
        }
        Err(_) => None,
    }
}

#[cfg(feature = "extraction")]
fn check_vag(data: &[u8]) -> Option<(usize, String)> {
    if data.len() < 48 {
        return None;
    }

    // Check for "VAGp" magic
    if &data[0..4] != b"VAGp" {
        return None;
    }

    // VAG header is 48 bytes, followed by audio data
    // Size is at offset 0x0C (4 bytes, big-endian)
    let size = u32::from_be_bytes([data[12], data[13], data[14], data[15]]) as usize;

    // Sanity check: size should be reasonable (< 10 MB)
    if size > 10 * 1024 * 1024 || size < 48 {
        return None;
    }

    let total_size = 48 + size;
    if total_size > data.len() {
        return None;
    }

    let metadata = format!("{} bytes", size);
    Some((total_size, metadata))
}

#[cfg(feature = "extraction")]
fn check_custom_model(data: &[u8]) -> Option<(usize, String)> {
    if data.len() < 12 {
        return None;
    }

    // Check for 0x80000002 at offset +4
    let signature = u32::from_le_bytes([data[4], data[5], data[6], data[7]]);
    if signature != 0x80000002 {
        return None;
    }

    // First 4 bytes might be size or offset
    let possible_size = u32::from_le_bytes([data[0], data[1], data[2], data[3]]) as usize;

    // Sanity check: size should be reasonable (100 bytes to 1 MB)
    if possible_size < 100 || possible_size > 1024 * 1024 {
        return None;
    }

    if possible_size > data.len() {
        return None;
    }

    let metadata = format!("{} bytes", possible_size);
    Some((possible_size, metadata))
}

#[cfg(feature = "extraction")]
fn check_lzss(data: &[u8]) -> Option<(usize, String)> {
    if data.len() < 8 {
        return None;
    }

    // Check for "sszl" magic
    if &data[0..4] != b"sszl" {
        return None;
    }

    // LZSS files don't have explicit size in header
    // We need to decompress or estimate size
    // For now, look for next known signature or end of data

    // Simple heuristic: scan forward looking for next signature
    let max_search = 1024 * 1024; // Don't search more than 1MB
    let search_len = max_search.min(data.len());

    for i in 4..search_len {
        // Check for start of next asset (TIM, VAG, etc.)
        if i + 4 <= data.len() {
            let next_magic = u32::from_le_bytes([data[i], data[i + 1], data[i + 2], data[i + 3]]);

            // TIM magic
            if next_magic == 0x00000010 {
                return Some((i, format!("{} bytes", i - 4)));
            }

            // VAG magic ("VAGp")
            if &data[i..i + 4] == b"VAGp" {
                return Some((i, format!("{} bytes", i - 4)));
            }

            // Another LZSS
            if &data[i..i + 4] == b"sszl" {
                return Some((i, format!("{} bytes", i - 4)));
            }
        }
    }

    // If no next signature found, estimate as small chunk
    let estimated_size = 16 * 1024; // 16 KB default
    Some((
        estimated_size.min(data.len()),
        format!("~{} bytes", estimated_size),
    ))
}

#[cfg(feature = "extraction")]
fn check_mips_overlay(data: &[u8]) -> Option<(usize, String)> {
    if data.len() < 256 {
        return None;
    }

    // MIPS code validation heuristics:
    // 1. Instructions should be 4-byte aligned
    // 2. Most instructions should have valid MIPS opcode patterns
    // 3. Should have reasonable instruction mix

    let mut valid_instructions = 0;
    let mut total_checked = 0;
    const SAMPLE_SIZE: usize = 64; // Check first 64 instructions

    for i in (0..SAMPLE_SIZE * 4).step_by(4) {
        if i + 4 > data.len() {
            break;
        }

        let instr = u32::from_le_bytes([data[i], data[i + 1], data[i + 2], data[i + 3]]);

        if is_likely_mips_instruction(instr) {
            valid_instructions += 1;
        }
        total_checked += 1;
    }

    // If >70% of instructions look valid, consider it MIPS code
    if total_checked > 0 && (valid_instructions * 100 / total_checked) > 70 {
        // Estimate size by scanning for section end or next asset
        let estimated_size = estimate_mips_overlay_size(data);
        let metadata = format!(
            "{} bytes, ~{}% valid MIPS",
            estimated_size,
            valid_instructions * 100 / total_checked
        );
        return Some((estimated_size, metadata));
    }

    None
}

#[cfg(feature = "extraction")]
fn is_likely_mips_instruction(instr: u32) -> bool {
    // Check for common MIPS R3000 instruction patterns
    let opcode = (instr >> 26) & 0x3F;

    // Common opcodes for MIPS R3000:
    // 0x00 = SPECIAL (add, sub, jr, etc.)
    // 0x02-0x03 = J, JAL (jumps)
    // 0x04-0x07 = BEQ, BNE, BLEZ, BGTZ (branches)
    // 0x08-0x0F = ADDI, ADDIU, SLTI, SLTIU, ANDI, ORI, XORI, LUI
    // 0x20-0x2B = LB, LH, LW, LBU, LHU, SB, SH, SW

    matches!(
        opcode,
        0x00 | 0x02
            | 0x03
            | 0x04
            | 0x05
            | 0x06
            | 0x07
            | 0x08
            | 0x09
            | 0x0A
            | 0x0B
            | 0x0C
            | 0x0D
            | 0x0E
            | 0x0F
            | 0x20
            | 0x21
            | 0x22
            | 0x23
            | 0x24
            | 0x25
            | 0x28
            | 0x29
            | 0x2A
            | 0x2B
    )
}

#[cfg(feature = "extraction")]
fn estimate_mips_overlay_size(data: &[u8]) -> usize {
    // Scan forward looking for section boundary or next known asset
    const MAX_OVERLAY_SIZE: usize = 256 * 1024; // 256 KB max
    let max_scan = MAX_OVERLAY_SIZE.min(data.len());

    for i in (256..max_scan).step_by(4) {
        if i + 4 > data.len() {
            break;
        }

        // Look for alignment padding (nulls) or next asset signature
        let word = u32::from_le_bytes([data[i], data[i + 1], data[i + 2], data[i + 3]]);

        // Check for TIM/VAG/LZSS magic
        if word == 0x00000010 || word == 0x70474156 || word == 0x6C7A7373 {
            return i;
        }

        // Check for long run of zeros (section padding)
        if word == 0 && i + 16 <= data.len() {
            let all_zero = data[i..i + 16].iter().all(|&b| b == 0);
            if all_zero {
                return i;
            }
        }
    }

    // Default to 64 KB if no boundary found
    (64 * 1024).min(data.len())
}

#[cfg(feature = "extraction")]
fn print_statistics(assets: &[Asset]) {
    let mut counts = std::collections::HashMap::new();
    let mut total_sizes = std::collections::HashMap::new();

    for asset in assets {
        *counts.entry(asset.format).or_insert(0) += 1;
        *total_sizes.entry(asset.format).or_insert(0) += asset.size;
    }

    println!("\n📊 Asset Statistics:");
    println!("   Total assets: {}", assets.len());

    for format in [
        AssetFormat::Tim,
        AssetFormat::Vag,
        AssetFormat::CustomModel,
        AssetFormat::Lzss,
        AssetFormat::MipsOverlay,
        AssetFormat::Unknown,
    ] {
        let count = counts.get(&format).unwrap_or(&0);
        let size = total_sizes.get(&format).unwrap_or(&0);
        if *count > 0 {
            println!(
                "   {}: {} assets ({:.2} MB)",
                format.name(),
                count,
                *size as f64 / 1024.0 / 1024.0
            );
        }
    }
}

#[cfg(feature = "extraction")]
fn extract_assets(data: &[u8], assets: &[Asset], output_dir: &Path) -> Result<()> {
    use std::io::Write;

    let pb = ProgressBar::new(assets.len() as u64);
    pb.set_style(
        ProgressStyle::default_bar()
            .template("   [{bar:40}] {pos}/{len} ({percent}%) {msg}")
            .unwrap()
            .progress_chars("=>-"),
    );

    for asset in assets {
        let asset_path = output_dir.join(&asset.filename);
        let end_offset = (asset.offset + asset.size).min(data.len());
        let asset_data = &data[asset.offset..end_offset];

        let mut file = fs::File::create(&asset_path)
            .with_context(|| format!("Failed to create {}", asset_path.display()))?;
        file.write_all(asset_data)
            .with_context(|| format!("Failed to write {}", asset_path.display()))?;

        pb.inc(1);
    }

    pb.finish_with_message("Done");
    Ok(())
}
