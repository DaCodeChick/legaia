/// Test asset scanner on PROT.DAT using chunked scanning
use anyhow::Result;
use psxutils::{AssetScanner, AssetType, CdRom};

fn main() -> Result<()> {
    let disc_path = "/home/admin/Downloads/Legend of Legaia.bin";

    println!("Opening disc: {}", disc_path);
    let disc = CdRom::open(disc_path)?;

    // Get file info without reading entire file
    println!("Finding PROT.DAT...");
    let entries = disc.read_dir("/")?;
    let prot_entry = entries
        .iter()
        .find(|e| e.name.eq_ignore_ascii_case("PROT.DAT"))
        .expect("PROT.DAT not found");

    println!(
        "PROT.DAT size: {} bytes ({} MB)",
        prot_entry.size,
        prot_entry.size / 1024 / 1024
    );
    println!("  Starting at LBA: {}", prot_entry.lba);

    // Scan in 5MB chunks to avoid OOM
    const CHUNK_SIZE: usize = 5 * 1024 * 1024; // 5 MB chunks
    let file_size = prot_entry.size as usize;
    let file_lba = prot_entry.lba;
    let mut all_assets = Vec::new();
    let mut file_offset = 0;

    println!("\nScanning for embedded assets in chunks...");
    while file_offset < file_size {
        let chunk_size = CHUNK_SIZE.min(file_size - file_offset);

        // Calculate which sectors to read
        let sectors_to_skip = file_offset / 2048;
        let chunk_lba = file_lba + sectors_to_skip as u32;

        print!(
            "  Scanning offset {}-{} MB... ",
            file_offset / 1024 / 1024,
            (file_offset + chunk_size) / 1024 / 1024
        );

        let chunk_data = disc.read_data(chunk_lba, chunk_size)?;

        let scanner = AssetScanner::new(&chunk_data).with_min_size(64);
        let chunk_assets = scanner.scan();
        let asset_count = chunk_assets.len();

        // Adjust offsets to be relative to file start
        for mut asset in chunk_assets {
            asset.offset += file_offset;
            all_assets.push(asset);
        }

        println!("found {} new assets", asset_count);

        file_offset += chunk_size;
    }

    let assets = all_assets;

    // Count by type
    let mut tim_count = 0;
    let mut tmd_count = 0;
    let mut vag_count = 0;

    for asset in &assets {
        match asset.asset_type {
            AssetType::Tim { .. } => tim_count += 1,
            AssetType::Tmd { .. } => tmd_count += 1,
            AssetType::Vag => vag_count += 1,
        }
    }

    println!("\nDiscovered {} assets:", assets.len());
    println!("  - {} TIM textures", tim_count);
    println!("  - {} TMD models", tmd_count);
    println!("  - {} VAG audio samples", vag_count);

    // Show first 10 TIM textures
    println!("\nFirst 10 TIM textures:");
    let mut shown = 0;
    for asset in &assets {
        if let AssetType::Tim { width, height } = asset.asset_type {
            println!(
                "  TIM at offset {:#08x}: {}x{} pixels ({} bytes)",
                asset.offset, width, height, asset.size
            );
            shown += 1;
            if shown >= 10 {
                break;
            }
        }
    }

    println!("\nExpected (from jPSXdec): 1132 TIM textures");
    println!(
        "Comparison: {} / 1132 = {:.1}%",
        tim_count,
        (tim_count as f32 / 1132.0) * 100.0
    );

    Ok(())
}
