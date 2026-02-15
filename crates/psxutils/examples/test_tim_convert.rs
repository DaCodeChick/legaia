use psxutils::formats::Tim;
use std::fs;
use std::path::Path;

fn main() -> anyhow::Result<()> {
    let tim_path = Path::new("/tmp/extracted_tim.tim");

    println!("Loading TIM file: {}", tim_path.display());
    let tim_data = fs::read(tim_path)?;

    let tim = Tim::parse(&tim_data)?;
    println!("Parsed TIM successfully!");
    println!("  Pixel mode: {:?}", tim.pixel_mode);
    println!("  Dimensions: {}x{}", tim.width(), tim.height());
    println!("  Has CLUT: {}", tim.clut.is_some());

    if let Some(ref clut) = tim.clut {
        println!("  CLUT size: {} colors", clut.data.len());
    }

    // Convert to RGBA8
    println!("\nConverting to RGBA8...");
    let rgba_data = tim.to_rgba8()?;
    println!("  Output size: {} bytes", rgba_data.len());

    // Save as PNG using image crate
    println!("\nSaving as PNG...");
    let output_path = Path::new("/tmp/extracted_texture.png");

    image::save_buffer(
        output_path,
        &rgba_data,
        tim.width() as u32,
        tim.height() as u32,
        image::ColorType::Rgba8,
    )?;

    println!("✅ Success! Saved to: {}", output_path.display());

    Ok(())
}
