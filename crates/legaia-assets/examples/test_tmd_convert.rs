use legaia_assets::converter::tmd_to_gltf;
use psxutils::formats::Tmd;
use std::fs;
use std::path::Path;

fn main() -> anyhow::Result<()> {
    // Test with one of the known TMD files
    let tmd_path = dirs::data_local_dir()
        .unwrap()
        .join("legaia/assets/PROT/file_0005.bin");

    println!("Reading TMD file: {}", tmd_path.display());
    let tmd_data = fs::read(&tmd_path)?;

    let tmd = Tmd::parse(&tmd_data)?;
    println!("Parsed TMD with {} objects", tmd.object_count());

    for (i, obj) in tmd.objects.iter().enumerate() {
        println!(
            "  Object {}: {} vertices, {} normals, {} primitives",
            i,
            obj.vertices.len(),
            obj.normals.len(),
            obj.primitives.len()
        );
    }

    // Convert to glTF
    let output_path = Path::new("/tmp/test_model.gltf");
    println!("\nConverting to glTF: {}", output_path.display());
    tmd_to_gltf(&tmd, output_path)?;

    println!("Success! Check output at:");
    println!("  {}", output_path.display());
    println!("  {}", output_path.with_extension("bin").display());

    Ok(())
}
