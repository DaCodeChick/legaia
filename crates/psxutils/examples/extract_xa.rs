use psxutils::cdrom::CdRom;
use psxutils::formats::xa::{CodingInfo, XaSubHeader};
use psxutils::formats::xa_adpcm::XaAdpcmDecoder;
use std::collections::HashMap;
use std::error::Error;
use std::fs::File;
use std::io::Write;
use std::path::Path;

/// Represents one XA audio stream
struct XaStream {
    file_number: u8,
    channel: u8,
    coding_info: CodingInfo,
    sectors: Vec<u32>,   // LBA addresses of sectors belonging to this stream
    source_file: String, // Name of the .XA file
}

fn main() -> Result<(), Box<dyn Error>> {
    let disc_path = "/home/admin/Downloads/Legend of Legaia.bin";
    let output_dir = "/tmp/extracted_xa";

    println!("Opening disc: {}", disc_path);
    let cdrom = CdRom::open(disc_path)?;

    // Create output directory
    std::fs::create_dir_all(output_dir)?;
    println!("Output directory: {}", output_dir);

    // Get list of XA files
    println!("\nScanning XA directory...");
    let xa_entries = cdrom.read_dir("/XA")?;
    println!("Found {} .XA files", xa_entries.len());

    // Scan all XA files
    let mut all_streams: Vec<XaStream> = Vec::new();

    for entry in &xa_entries {
        if !entry.name.ends_with(".XA") {
            continue;
        }

        print!("  Scanning {}... ", entry.name);
        let streams = scan_xa_file(&cdrom, entry.lba, entry.size, &entry.name)?;
        println!("{} streams", streams.len());

        all_streams.extend(streams);
    }

    println!("\nTotal audio streams found: {}", all_streams.len());

    // Extract all streams
    println!("\nExtracting streams...");
    for (idx, stream) in all_streams.iter().enumerate() {
        let duration_secs = estimate_duration(stream);
        print!(
            "  [{}/{}] {} File={} Ch={}: {:.1}s... ",
            idx + 1,
            all_streams.len(),
            stream.source_file,
            stream.file_number,
            stream.channel,
            duration_secs
        );

        match extract_stream(&cdrom, stream, output_dir) {
            Ok(path) => println!("✓ {}", path.file_name().unwrap().to_string_lossy()),
            Err(e) => println!("✗ Error: {}", e),
        }
    }

    println!("\n✓ Extraction complete!");
    println!("  Output: {}", output_dir);
    println!("  Files: {}", all_streams.len());

    Ok(())
}

/// Estimate duration in seconds for a stream
fn estimate_duration(stream: &XaStream) -> f64 {
    let samples_per_sector = match stream.coding_info.bits_per_sample() {
        4 => 28 * 8, // 224 samples per sector
        8 => 28 * 4, // 112 samples per sector
        _ => 0,
    };

    let total_samples = stream.sectors.len() * samples_per_sector;
    total_samples as f64 / stream.coding_info.sample_rate() as f64
}

/// Scan an XA file and group sectors into streams by file/channel
fn scan_xa_file(
    cdrom: &CdRom,
    start_lba: u32,
    size: u32,
    filename: &str,
) -> Result<Vec<XaStream>, Box<dyn Error>> {
    let mut streams: HashMap<(u8, u8), XaStream> = HashMap::new();

    // Calculate number of sectors (ISO sectors are 2048 bytes)
    let sector_count = (size + 2047) / 2048;

    const XA_SUBHEADER_OFFSET: usize = 16; // After 12-byte sync + 4-byte header

    for i in 0..sector_count {
        let lba = start_lba + i;
        let raw_sector = cdrom.read_raw_sector(lba)?;

        if raw_sector.len() < XA_SUBHEADER_OFFSET + 8 {
            continue;
        }

        let subheader_data = &raw_sector[XA_SUBHEADER_OFFSET..XA_SUBHEADER_OFFSET + 8];

        if let Some(header) = XaSubHeader::parse(subheader_data) {
            if !header.is_audio() {
                continue; // Not an audio sector
            }

            let key = (header.file_number, header.channel);

            streams
                .entry(key)
                .or_insert_with(|| XaStream {
                    file_number: header.file_number,
                    channel: header.channel,
                    coding_info: header.coding_info,
                    sectors: Vec::new(),
                    source_file: filename.to_string(),
                })
                .sectors
                .push(lba);
        }
    }

    Ok(streams.into_values().collect())
}

/// Extract and decode one XA stream to WAV
fn extract_stream(
    cdrom: &CdRom,
    stream: &XaStream,
    output_dir: &str,
) -> Result<std::path::PathBuf, Box<dyn Error>> {
    // Create decoder
    let mut decoder = XaAdpcmDecoder::new(
        stream.coding_info.bits_per_sample(),
        stream.coding_info.is_stereo(),
        1.0, // Volume = 1.0 (normal)
    );

    // Decode all sectors
    let mut pcm_data = Vec::new();

    const XA_DATA_OFFSET: usize = 24; // Sync(12) + Header(4) + SubHeader(8)
    const XA_DATA_SIZE: usize = 2324; // MODE2FORM2 payload size

    for &lba in &stream.sectors {
        let raw_sector = cdrom.read_raw_sector(lba)?;

        if raw_sector.len() < XA_DATA_OFFSET + XA_DATA_SIZE {
            continue;
        }

        let audio_data = &raw_sector[XA_DATA_OFFSET..XA_DATA_OFFSET + XA_DATA_SIZE];
        let samples = decoder.decode_sector(audio_data);
        pcm_data.extend_from_slice(&samples);
    }

    // Create filename: xa1_file1_ch0.wav
    let base_name = stream.source_file.trim_end_matches(".XA").to_lowercase();
    let wav_filename = format!(
        "{}_file{}_ch{}.wav",
        base_name, stream.file_number, stream.channel
    );

    let wav_path = Path::new(output_dir).join(wav_filename);

    write_wav(
        &wav_path,
        &pcm_data,
        stream.coding_info.sample_rate(),
        stream.coding_info.is_stereo(),
    )?;

    Ok(wav_path)
}

/// Write PCM data to WAV file
fn write_wav(
    path: &Path,
    pcm_data: &[i16],
    sample_rate: u32,
    stereo: bool,
) -> Result<(), Box<dyn Error>> {
    let mut file = File::create(path)?;

    let num_channels: u16 = if stereo { 2 } else { 1 };
    let bits_per_sample: u16 = 16;
    let byte_rate = sample_rate * num_channels as u32 * bits_per_sample as u32 / 8;
    let block_align: u16 = num_channels * bits_per_sample / 8;
    let data_size = (pcm_data.len() * 2) as u32; // 2 bytes per i16

    // Write RIFF header
    file.write_all(b"RIFF")?;
    file.write_all(&(36 + data_size).to_le_bytes())?; // File size - 8
    file.write_all(b"WAVE")?;

    // Write fmt chunk
    file.write_all(b"fmt ")?;
    file.write_all(&16u32.to_le_bytes())?; // fmt chunk size
    file.write_all(&1u16.to_le_bytes())?; // Audio format (1 = PCM)
    file.write_all(&num_channels.to_le_bytes())?;
    file.write_all(&sample_rate.to_le_bytes())?;
    file.write_all(&byte_rate.to_le_bytes())?;
    file.write_all(&block_align.to_le_bytes())?;
    file.write_all(&bits_per_sample.to_le_bytes())?;

    // Write data chunk
    file.write_all(b"data")?;
    file.write_all(&data_size.to_le_bytes())?;

    // Write PCM samples (little-endian)
    for &sample in pcm_data {
        file.write_all(&sample.to_le_bytes())?;
    }

    Ok(())
}
