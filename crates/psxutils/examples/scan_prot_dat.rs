/// Test asset scanner on PROT.DAT
use anyhow::Result;
use psxutils::{AssetScanner, AssetType, CdRom};

fn main() -> Result<()> {
    let disc_path = "/home/admin/Downloads/Legend of Legaia.bin";

    println!("Opening disc: {}", disc_path);
    let disc = CdRom::open(disc_path)?;

    println!("Reading PROT.DAT...");
    let prot_data = disc.read_file("/PROT.DAT")?;
    println!(
        "PROT.DAT size: {} bytes ({} MB)",
        prot_data.len(),
        prot_data.len() / 1024 / 1024
    );

    println!("\nScanning for embedded assets...");
    let scanner = AssetScanner::new(&prot_data).with_min_size(64);
    let assets = scanner.scan();

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
