//! Test Legend of Legaia custom TMD parser
//!
//! Usage: cargo run --example test_legaia_tmd <tmd_file>

use psxutils::formats::tmd::Tmd;
use std::env;
use std::fs;
use std::process;

fn main() {
    let args: Vec<String> = env::args().collect();
    if args.len() != 2 {
        eprintln!("Usage: {} <tmd_file>", args[0]);
        eprintln!("\nExample:");
        eprintln!(
            "  {} ~/.local/share/legaia/assets/PROT/file_0005.bin",
            args[0]
        );
        process::exit(1);
    }

    let input_path = &args[1];

    // Read TMD file
    let data = match fs::read(input_path) {
        Ok(d) => d,
        Err(e) => {
            eprintln!("Error reading {}: {}", input_path, e);
            process::exit(1);
        }
    };

    println!("Parsing TMD file: {}", input_path);
    println!("File size: {} bytes\n", data.len());

    // Parse TMD
    let tmd = match Tmd::parse(&data) {
        Ok(t) => t,
        Err(e) => {
            eprintln!("Error parsing TMD: {}", e);
            process::exit(1);
        }
    };

    println!("✓ Successfully parsed TMD!");
    println!("  Flags: {}", tmd.flags);
    println!("  Objects: {}", tmd.objects.len());
    println!();

    // Print stats for each object
    for (i, obj) in tmd.objects.iter().enumerate() {
        println!("Object {}:", i);
        println!("  Vertices: {}", obj.vertices.len());
        println!("  Normals: {}", obj.normals.len());
        println!("  Primitives: {}", obj.primitives.len());
        println!("  Scale: {}", obj.scale);

        // Show first few vertices
        if !obj.vertices.is_empty() {
            println!("\n  First 5 vertices:");
            for (vi, v) in obj.vertices.iter().take(5).enumerate() {
                println!("    V{}: ({}, {}, {})", vi, v.x, v.y, v.z);
            }
        }

        // Show first few normals
        if !obj.normals.is_empty() {
            println!("\n  First 5 normals:");
            for (ni, n) in obj.normals.iter().take(5).enumerate() {
                println!("    N{}: ({}, {}, {})", ni, n.nx, n.ny, n.nz);
            }
        }

        // Analyze primitives
        let mut tri_count = 0;
        let mut quad_count = 0;
        let mut textured_count = 0;

        for prim in &obj.primitives {
            match prim {
                psxutils::formats::tmd::TmdPrimitive::Triangle { texture_info, .. } => {
                    tri_count += 1;
                    if texture_info.is_some() {
                        textured_count += 1;
                    }
                }
                psxutils::formats::tmd::TmdPrimitive::Quad { texture_info, .. } => {
                    quad_count += 1;
                    if texture_info.is_some() {
                        textured_count += 1;
                    }
                }
            }
        }

        println!("\n  Primitive breakdown:");
        println!("    Triangles: {}", tri_count);
        println!("    Quads: {}", quad_count);
        println!("    Textured: {}", textured_count);
        println!();
    }

    println!("✓ Parsing complete!");
}
