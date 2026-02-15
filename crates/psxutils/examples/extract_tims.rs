//! Extract all TIM textures from PROT.DAT with full features:
//! - Multi-threaded extraction using rayon
//! - Progress bar with indicatif
//! - Metadata output (JSON)
//! - Thumbnail generation
//! - Error handling and statistics

use anyhow::{Context, Result};
use psxutils::{formats::Tim, AssetScanner, AssetType, CdRom};
use std::fs;
use std::path::Path;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::Arc;

#[cfg(feature = "extraction")]
use {
    indicatif::{ProgressBar, ProgressStyle},
    rayon::prelude::*,
    serde::Serialize,
};

/// Metadata for an extracted TIM texture
#[derive(Debug, Clone)]
#[cfg_attr(feature = "extraction", derive(Serialize))]
struct TimMetadata {
    /// Index number (0-based)
    index: usize,
    /// Offset in PROT.DAT file
    offset: usize,
    /// Size of TIM data in bytes
    size: usize,
    /// Width in pixels
    width: u16,
    /// Height in pixels
    height: u16,
    /// Pixel format
    pixel_mode: String,
    /// Whether it has a color lookup table
    has_clut: bool,
    /// Output filename
    filename: String,
    /// Whether extraction succeeded
    success: bool,
    /// Error message if extraction failed
    error: Option<String>,
}

/// Statistics for the extraction process
#[derive(Debug, Default)]
struct ExtractionStats {
    total: usize,
    successful: usize,
    failed: usize,
    total_bytes: usize,
}

fn main() -> Result<()> {
    #[cfg(not(feature = "extraction"))]
    {
        eprintln!("Error: This example requires the 'extraction' feature.");
        eprintln!("Run with: cargo run --example extract_tims --features extraction");
        std::process::exit(1);
    }

    #[cfg(feature = "extraction")]
    {
        run_extraction()
    }
}

#[cfg(feature = "extraction")]
fn run_extraction() -> Result<()> {
    let disc_path = "/home/admin/Downloads/Legend of Legaia.bin";
    let output_dir = Path::new("/tmp/extracted_tims");

    println!("=== TIM Texture Extractor ===");
    println!("Disc: {}", disc_path);
    println!("Output: {}\n", output_dir.display());

    // Create output directories
    fs::create_dir_all(output_dir)?;
    let full_dir = output_dir.join("full");
    let thumb_dir = output_dir.join("thumbnails");
    fs::create_dir_all(&full_dir)?;
    fs::create_dir_all(&thumb_dir)?;

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

    // Scan for TIM textures in chunks
    println!("🔍 Scanning for TIM textures...");
    let tim_assets = scan_prot_dat(&disc, prot_entry)?;
    println!("✅ Found {} TIM textures\n", tim_assets.len());

    // Extract all TIMs with progress bar
    println!("🎨 Extracting textures...");
    let metadata = extract_all_tims(&disc, prot_entry, &tim_assets, &full_dir, &thumb_dir)?;

    // Calculate statistics
    let stats = ExtractionStats {
        total: metadata.len(),
        successful: metadata.iter().filter(|m| m.success).count(),
        failed: metadata.iter().filter(|m| !m.success).count(),
        total_bytes: metadata.iter().map(|m| m.size).sum(),
    };

    // Write metadata JSON
    let metadata_path = output_dir.join("metadata.json");
    let metadata_json = serde_json::to_string_pretty(&metadata)?;
    fs::write(&metadata_path, metadata_json)?;

    // Print statistics
    println!("\n📊 Extraction Statistics:");
    println!("   Total TIMs found:    {}", stats.total);
    println!(
        "   Successfully extracted: {} ({:.1}%)",
        stats.successful,
        (stats.successful as f32 / stats.total as f32) * 100.0
    );
    println!("   Failed:              {}", stats.failed);
    println!(
        "   Total data size:     {:.2} MB",
        stats.total_bytes as f32 / 1024.0 / 1024.0
    );
    println!("\n✅ Extraction complete!");
    println!("   Full images:      {}", full_dir.display());
    println!("   Thumbnails:       {}", thumb_dir.display());
    println!("   Metadata:         {}", metadata_path.display());

    if stats.failed > 0 {
        println!(
            "\n⚠️  {} TIMs failed to extract (see metadata.json for details)",
            stats.failed
        );
    }

    Ok(())
}

#[cfg(feature = "extraction")]
fn scan_prot_dat(
    disc: &CdRom,
    prot_entry: &psxutils::cdrom::DirectoryEntry,
) -> Result<Vec<psxutils::DiscoveredAsset>> {
    const CHUNK_SIZE: usize = 5 * 1024 * 1024; // 5 MB chunks
    let file_size = prot_entry.size as usize;
    let file_lba = prot_entry.lba;
    let mut all_assets = Vec::new();

    // Progress bar for scanning
    let pb = ProgressBar::new(file_size as u64);
    pb.set_style(
        ProgressStyle::default_bar()
            .template("   [{bar:40}] {bytes}/{total_bytes} ({percent}%) {msg}")
            .unwrap()
            .progress_chars("=>-"),
    );

    let mut file_offset = 0;
    while file_offset < file_size {
        let chunk_size = CHUNK_SIZE.min(file_size - file_offset);
        let sectors_to_skip = file_offset / 2048;
        let chunk_lba = file_lba + sectors_to_skip as u32;

        let chunk_data = disc.read_data(chunk_lba, chunk_size)?;
        let scanner = AssetScanner::new(&chunk_data).with_min_size(64);
        let chunk_assets = scanner.scan();

        // Adjust offsets to be relative to file start
        for mut asset in chunk_assets {
            if matches!(asset.asset_type, AssetType::Tim { .. }) {
                asset.offset += file_offset;
                all_assets.push(asset);
            }
        }

        file_offset += chunk_size;
        pb.set_position(file_offset as u64);
    }

    pb.finish_with_message("Scan complete");
    Ok(all_assets)
}

#[cfg(feature = "extraction")]
fn extract_all_tims(
    disc: &CdRom,
    _prot_entry: &psxutils::cdrom::DirectoryEntry,
    tim_assets: &[psxutils::DiscoveredAsset],
    full_dir: &Path,
    thumb_dir: &Path,
) -> Result<Vec<TimMetadata>> {
    // Progress tracking
    let pb = ProgressBar::new(tim_assets.len() as u64);
    pb.set_style(
        ProgressStyle::default_bar()
            .template("   [{bar:40}] {pos}/{len} ({percent}%) {msg}")
            .unwrap()
            .progress_chars("=>-"),
    );

    let successful_count = Arc::new(AtomicUsize::new(0));
    let failed_count = Arc::new(AtomicUsize::new(0));

    // Read entire PROT.DAT once (it's only 121MB)
    let prot_data = disc.read_file("PROT.DAT")?;

    // Parallel extraction with rayon
    let metadata: Vec<TimMetadata> = tim_assets
        .par_iter()
        .enumerate()
        .map(|(index, asset)| {
            let result = extract_single_tim(&prot_data, index, asset, full_dir, thumb_dir);

            match &result {
                Ok(meta) if meta.success => {
                    successful_count.fetch_add(1, Ordering::Relaxed);
                }
                _ => {
                    failed_count.fetch_add(1, Ordering::Relaxed);
                }
            }

            pb.inc(1);
            result.unwrap_or_else(|e| {
                let (width, height) = if let AssetType::Tim { width, height } = asset.asset_type {
                    (width, height)
                } else {
                    (0, 0)
                };

                TimMetadata {
                    index,
                    offset: asset.offset,
                    size: asset.size,
                    width,
                    height,
                    pixel_mode: "unknown".to_string(),
                    has_clut: false,
                    filename: format!("tim_{:04}.png", index),
                    success: false,
                    error: Some(e.to_string()),
                }
            })
        })
        .collect();

    pb.finish_with_message(format!(
        "Extracted {} / {} TIMs",
        successful_count.load(Ordering::Relaxed),
        tim_assets.len()
    ));

    Ok(metadata)
}

#[cfg(feature = "extraction")]
fn extract_single_tim(
    prot_data: &[u8],
    index: usize,
    asset: &psxutils::DiscoveredAsset,
    full_dir: &Path,
    thumb_dir: &Path,
) -> Result<TimMetadata> {
    let (width, height) = if let AssetType::Tim { width, height } = asset.asset_type {
        (width, height)
    } else {
        anyhow::bail!("Not a TIM asset");
    };

    // Extract TIM data
    let end_offset = (asset.offset + asset.size).min(prot_data.len());
    let tim_data = &prot_data[asset.offset..end_offset];

    // Parse TIM
    let tim = Tim::parse(tim_data).context("Failed to parse TIM")?;

    // Convert to RGBA
    let rgba_data = tim.to_rgba8().context("Failed to convert TIM to RGBA")?;

    // Save full-size PNG
    let filename = format!("tim_{:04}.png", index);
    let output_path = full_dir.join(&filename);
    image::save_buffer(
        &output_path,
        &rgba_data,
        width as u32,
        height as u32,
        image::ColorType::Rgba8,
    )
    .context("Failed to save PNG")?;

    // Generate thumbnail (max 256x256)
    let thumb_path = thumb_dir.join(&filename);
    generate_thumbnail(&rgba_data, width, height, &thumb_path)?;

    Ok(TimMetadata {
        index,
        offset: asset.offset,
        size: asset.size,
        width,
        height,
        pixel_mode: format!("{:?}", tim.pixel_mode),
        has_clut: tim.clut.is_some(),
        filename,
        success: true,
        error: None,
    })
}

#[cfg(feature = "extraction")]
fn generate_thumbnail(rgba_data: &[u8], width: u16, height: u16, output_path: &Path) -> Result<()> {
    const MAX_THUMB_SIZE: u32 = 256;

    let (thumb_w, thumb_h) = if width > MAX_THUMB_SIZE as u16 || height > MAX_THUMB_SIZE as u16 {
        let scale = (MAX_THUMB_SIZE as f32 / width.max(height) as f32).min(1.0);
        (
            (width as f32 * scale) as u32,
            (height as f32 * scale) as u32,
        )
    } else {
        (width as u32, height as u32)
    };

    // Use image crate for resizing
    let img = image::RgbaImage::from_raw(width as u32, height as u32, rgba_data.to_vec())
        .context("Failed to create image")?;

    let thumb = image::imageops::resize(
        &img,
        thumb_w,
        thumb_h,
        image::imageops::FilterType::Lanczos3,
    );

    thumb
        .save(output_path)
        .context("Failed to save thumbnail")?;

    Ok(())
}
