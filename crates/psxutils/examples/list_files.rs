use psxutils::cdrom::CdRom;
use std::error::Error;

fn main() -> Result<(), Box<dyn Error>> {
    let disc_path = "/home/admin/Downloads/Legend of Legaia.bin";

    println!("Opening disc: {}", disc_path);
    let cdrom = CdRom::open(disc_path)?;

    println!("\nRoot directory contents:");
    let root_entries = cdrom.read_dir("/")?;
    for entry in &root_entries {
        if entry.is_dir {
            println!("  DIR:  {}", entry.name);
        } else {
            println!("  FILE: {} ({} bytes)", entry.name, entry.size);
        }
    }

    // Check for XA directory
    if root_entries.iter().any(|e| e.name == "XA" && e.is_dir) {
        println!("\nXA directory contents:");
        let xa_entries = cdrom.read_dir("/XA")?;
        println!("  Found {} files in XA/", xa_entries.len());
        for entry in xa_entries.iter().take(10) {
            println!("    {}: {} bytes", entry.name, entry.size);
        }
        if xa_entries.len() > 10 {
            println!("    ... and {} more files", xa_entries.len() - 10);
        }
    }

    Ok(())
}
