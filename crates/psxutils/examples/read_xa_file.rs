use psxutils::cdrom::CdRom;
use psxutils::formats::xa::XaSubHeader;
use std::error::Error;

fn main() -> Result<(), Box<dyn Error>> {
    let disc_path = "/home/admin/Downloads/Legend of Legaia.bin";

    println!("Opening disc: {}", disc_path);
    let cdrom = CdRom::open(disc_path)?;

    // Find XA file location
    println!("\nFinding XA/XA1.XA...");
    let xa_entries = cdrom.read_dir("/XA")?;
    let xa1 = xa_entries
        .iter()
        .find(|e| e.name == "XA1.XA")
        .expect("XA1.XA not found");

    println!("XA1.XA: {} bytes at LBA {}", xa1.size, xa1.lba);

    // Calculate number of sectors
    const SECTOR_SIZE: usize = 2352;
    let sector_count = (xa1.size as usize + 2047) / 2048; // ISO sectors are 2048 bytes
    println!("ISO sectors: {}", sector_count);

    // XA sub-header is at offset 16 in raw sector (after 12-byte sync + 4-byte header)
    const XA_SUBHEADER_OFFSET: usize = 16;
    // XA audio data starts at offset 24 (after sync + header + subheader)
    const XA_DATA_OFFSET: usize = 24;

    println!("\nFirst 10 raw sectors:");
    for i in 0..10.min(sector_count) {
        let lba = xa1.lba + i as u32;
        let raw_sector = cdrom.read_raw_sector(lba)?;

        if raw_sector.len() < XA_SUBHEADER_OFFSET + 8 {
            println!("  Sector {} at LBA {}: Too short", i, lba);
            continue;
        }

        let subheader_data = &raw_sector[XA_SUBHEADER_OFFSET..XA_SUBHEADER_OFFSET + 8];

        if let Some(header) = XaSubHeader::parse(subheader_data) {
            println!(
                "  Sector {} at LBA {}: File={}, Channel={}, SubMode={}, Coding={}",
                i, lba, header.file_number, header.channel, header.sub_mode, header.coding_info
            );
            if header.is_audio() {
                println!("    -> XA Audio sector detected");
            }
        } else {
            // Print raw bytes for debugging
            println!(
                "  Sector {} at LBA {}: Invalid header: {:02x} {:02x} {:02x} {:02x} | {:02x} {:02x} {:02x} {:02x}",
                i, lba,
                subheader_data[0],
                subheader_data[1],
                subheader_data[2],
                subheader_data[3],
                subheader_data[4],
                subheader_data[5],
                subheader_data[6],
                subheader_data[7]
            );
        }
    }

    Ok(())
}
