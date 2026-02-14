//! Example: Testing PSX format parsers
//!
//! This example demonstrates parsing PSX format files.
//! Since we don't have real game assets yet, we create minimal test files.

use psxutils::{Tim, Vag};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("PSX Utilities Test\n");

    // Test 1: TIM parser
    println!("=== TIM Texture Format ===");
    let test_tim = create_test_tim();
    match Tim::parse(&test_tim) {
        Ok(tim) => {
            println!("✓ TIM parsed successfully");
            println!("  Pixel mode: {:?}", tim.pixel_mode);
            println!("  Has CLUT: {}", tim.has_clut);
            println!("  Dimensions: {}x{}", tim.width(), tim.height());
        }
        Err(e) => println!("✗ TIM parse failed: {}", e),
    }

    // Test 2: VAG parser
    println!("\n=== VAG Audio Format ===");
    let test_vag = create_test_vag();
    match Vag::parse(&test_vag) {
        Ok(vag) => {
            println!("✓ VAG parsed successfully");
            println!("  Name: {}", vag.name);
            println!("  Sample rate: {} Hz", vag.sample_rate);
            println!("  Duration: {:.2}s", vag.duration_secs());
            println!("  Blocks: {}", vag.data.len() / 16);
        }
        Err(e) => println!("✗ VAG parse failed: {}", e),
    }

    println!("\n✓ All format parsers working!");

    Ok(())
}

/// Create a minimal valid TIM file (16-bit direct color, 2x2 pixels)
fn create_test_tim() -> Vec<u8> {
    let mut tim = Vec::new();

    // Header
    tim.extend_from_slice(&0x00000010u32.to_le_bytes()); // Magic
    tim.extend_from_slice(&0x00000002u32.to_le_bytes()); // Flags (16-bit, no CLUT)

    // Pixel data block size = header (12 bytes) + data (4 pixels * 2 bytes = 8 bytes) = 20 bytes
    tim.extend_from_slice(&0x00000014u32.to_le_bytes()); // Size (20 bytes total)
    tim.extend_from_slice(&0u16.to_le_bytes()); // VRAM X
    tim.extend_from_slice(&0u16.to_le_bytes()); // VRAM Y
    tim.extend_from_slice(&2u16.to_le_bytes()); // Width (in 16-bit units)
    tim.extend_from_slice(&2u16.to_le_bytes()); // Height

    // Pixel data (4 pixels, RGB555)
    tim.extend_from_slice(&0x8000u16.to_le_bytes()); // Red
    tim.extend_from_slice(&0x83E0u16.to_le_bytes()); // Green
    tim.extend_from_slice(&0x801Fu16.to_le_bytes()); // Blue
    tim.extend_from_slice(&0xFFFFu16.to_le_bytes()); // White

    tim
}

/// Create a minimal valid VAG file
fn create_test_vag() -> Vec<u8> {
    let mut vag = Vec::new();

    // Header
    vag.extend_from_slice(b"VAGp"); // Magic
    vag.extend_from_slice(&0x00000020u32.to_be_bytes()); // Version
    vag.extend_from_slice(&0x00000000u32.to_be_bytes()); // Reserved
    vag.extend_from_slice(&0x00000020u32.to_be_bytes()); // Size (32 bytes = 2 blocks)
    vag.extend_from_slice(&0x0000AC44u32.to_be_bytes()); // Sample rate (44100 Hz)
    vag.extend_from_slice(&[0u8; 12]); // Padding
    vag.extend_from_slice(b"test\0\0\0\0\0\0\0\0\0\0\0\0"); // Name

    // Data (2 blocks of silence)
    for _ in 0..2 {
        vag.push(0x00); // predict_nr + shift_factor
        vag.push(0x00); // flags
        vag.extend_from_slice(&[0u8; 14]); // data
    }

    vag
}
