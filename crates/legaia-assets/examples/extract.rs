//! Asset extraction CLI tool

use anyhow::Result;
use clap::Parser;
use legaia_assets::{manifest::SourceInfo, AssetExtractor, AssetManifest};
use std::path::PathBuf;
use tracing_subscriber;

#[derive(Parser)]
#[command(name = "extract")]
#[command(about = "Extract assets from Legend of Legaia disc image")]
struct Args {
    /// Path to the disc image (BIN/CUE or ISO)
    #[arg(short, long)]
    input: PathBuf,

    /// Output directory for extracted assets
    #[arg(short, long, default_value = "./extracted_assets")]
    output: PathBuf,

    /// Generate manifest file
    #[arg(short, long, default_value = "true")]
    manifest: bool,

    /// Manifest format (json or toml)
    #[arg(long, default_value = "json")]
    manifest_format: String,
}

fn main() -> Result<()> {
    // Initialize logging
    tracing_subscriber::fmt::init();

    let args = Args::parse();

    tracing::info!("Legend of Legaia Asset Extractor");
    tracing::info!("Input: {}", args.input.display());
    tracing::info!("Output: {}", args.output.display());

    // Create extractor
    let extractor = AssetExtractor::new(&args.input)?;

    // Extract assets
    tracing::info!("Extracting assets...");
    extractor.extract_all(&args.output)?;

    // Generate manifest if requested
    if args.manifest {
        tracing::info!("Generating asset manifest...");

        let source = SourceInfo {
            game: "Legend of Legaia".to_string(),
            region: "NTSC-U".to_string(),     // TODO: Detect from disc
            serial: "SCUS-94254".to_string(), // TODO: Detect from disc
            path: args.input.clone(),
        };

        let manifest = AssetManifest::new(source);

        let manifest_path = args
            .output
            .join(format!("manifest.{}", args.manifest_format));

        match args.manifest_format.as_str() {
            "json" => manifest.to_json(&manifest_path)?,
            "toml" => manifest.to_toml(&manifest_path)?,
            _ => anyhow::bail!("Unsupported manifest format: {}", args.manifest_format),
        }

        tracing::info!("Manifest saved to: {}", manifest_path.display());
    }

    tracing::info!("Extraction complete!");

    Ok(())
}
