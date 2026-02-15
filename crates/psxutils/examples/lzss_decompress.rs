//! LZSS Decompression Tool
//!
//! Decompresses LZSS-compressed files (commonly .lzs extension).
//!
//! Usage:
//!   cargo run --example lzss_decompress <input.lzs> [output]
//!
//! If output is not specified, writes to <input> with .lzs extension removed.

use psxutils::formats::lzss;
use std::env;
use std::fs;
use std::io;
use std::path::{Path, PathBuf};
use std::process;

fn main() -> io::Result<()> {
    let args: Vec<String> = env::args().collect();

    if args.len() < 2 {
        eprintln!("Usage: {} <input.lzs> [output]", args[0]);
        eprintln!();
        eprintln!("Decompresses LZSS-compressed files.");
        eprintln!("If output is not specified, removes .lzs extension from input filename.");
        eprintln!();
        eprintln!("Examples:");
        eprintln!("  {} player.lzs", args[0]);
        eprintln!("  {} player.lzs player.bin", args[0]);
        process::exit(1);
    }

    let input_path = Path::new(&args[1]);
    let output_path = if args.len() >= 3 {
        PathBuf::from(&args[2])
    } else {
        // Remove .lzs extension or add .out if no extension
        match input_path.file_stem() {
            Some(stem) => {
                let mut path = input_path.with_file_name(stem);
                // If the stem has an extension (e.g., player.bin.lzs -> player.bin), keep it
                // Otherwise just use the stem (e.g., player.lzs -> player)
                if let Some(parent) = input_path.parent() {
                    path = parent.join(stem);
                }
                path
            }
            None => input_path.with_extension("out"),
        }
    };

    println!("Decompressing: {}", input_path.display());
    println!("Output: {}", output_path.display());

    // Read compressed data
    let compressed = fs::read(input_path)?;
    println!("Input size: {} bytes", compressed.len());

    // Decompress
    let decompressed = lzss::decompress(&compressed)?;
    println!("Output size: {} bytes", decompressed.len());
    println!(
        "Compression ratio: {:.2}%",
        (decompressed.len() as f64 / compressed.len() as f64) * 100.0
    );

    // Write output
    fs::write(&output_path, &decompressed)?;
    println!("✓ Successfully decompressed to {}", output_path.display());

    Ok(())
}
