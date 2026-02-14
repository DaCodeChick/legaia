//! VAG (Sound Sample) format parser
//!
//! VAG is the PlayStation 1 ADPCM (Adaptive Differential Pulse Code Modulation)
//! compressed audio format for individual sound samples.
//!
//! ## Format Specification
//!
//! ```text
//! VAG Header:
//!   char[4] magic    = "VAGp"    // Magic number
//!   u32     version  = 0x20      // Version
//!   u32     reserved = 0         // Reserved
//!   u32     size     = data size // Size of audio data in bytes
//!   u32     rate     = 44100     // Sample rate
//!   u16     channels = 1         // Number of channels (usually mono)
//!   u16     reserved2 = 0        // Reserved
//!   char[16] name                // Sample name (null-terminated)
//!
//! VAG Data (per 16-byte block):
//!   u8 predict_nr   // Prediction filter number (0-4)
//!   u8 shift_factor // Shift factor (0-12)
//!   u8 flags        // Loop flags (0x00, 0x02, 0x03, 0x07)
//!   u8 reserved     // Reserved (usually 0)
//!   u8[12] data     // 28 nibbles of ADPCM data
//! ```

use crate::{PsxError, Result};
use bytemuck::{Pod, Zeroable};

/// VAG magic number "VAGp"
pub const VAG_MAGIC: [u8; 4] = *b"VAGp";

/// VAG version
pub const VAG_VERSION: u32 = 0x00000020;

/// Loop flags
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LoopFlag {
    /// Continue playing (default)
    Continue = 0x00,
    /// End marker (last block)
    End = 0x01,
    /// Loop region start
    LoopStart = 0x02,
    /// Loop region end
    LoopEnd = 0x03,
    /// Loop region start + end (single block loop)
    LoopStartEnd = 0x06,
    /// End with mute
    EndMute = 0x07,
}

/// VAG file header (48 bytes)
#[repr(C)]
#[derive(Debug, Clone, Copy, Pod, Zeroable)]
struct VagHeader {
    magic: [u8; 4], // "VAGp"
    version: u32,   // 0x00000020
    reserved1: u32, // 0x00000000
    size: u32,      // Data size in bytes
    rate: u32,      // Sample rate (Hz)
    _pad: [u8; 12], // Reserved + padding
    name: [u8; 16], // Sample name
}

/// Parsed VAG audio sample
#[derive(Debug, Clone)]
pub struct Vag {
    /// Sample name
    pub name: String,
    /// Sample rate in Hz
    pub sample_rate: u32,
    /// Raw ADPCM data (16-byte blocks)
    pub data: Vec<u8>,
    /// Loop start position (in samples)
    pub loop_start: Option<usize>,
    /// Loop end position (in samples)
    pub loop_end: Option<usize>,
}

impl Vag {
    /// Parse a VAG file from bytes
    pub fn parse(data: &[u8]) -> Result<Self> {
        if data.len() < 48 {
            return Err(PsxError::InvalidFormat("VAG file too small".to_string()));
        }

        // Parse header
        let header: &VagHeader = bytemuck::try_from_bytes(&data[0..48])
            .map_err(|e| PsxError::ParseError(format!("Failed to parse VAG header: {}", e)))?;

        if header.magic != VAG_MAGIC {
            return Err(PsxError::InvalidFormat(format!(
                "Invalid VAG magic: {:?}, expected {:?}",
                header.magic, VAG_MAGIC
            )));
        }

        let version = u32::from_be(header.version);
        if version != VAG_VERSION {
            tracing::warn!("Unexpected VAG version: 0x{:08X}", version);
        }

        // Parse name (null-terminated)
        let name = header
            .name
            .iter()
            .take_while(|&&b| b != 0)
            .map(|&b| b as char)
            .collect();

        let size = u32::from_be(header.size) as usize;
        let sample_rate = u32::from_be(header.rate);

        // Extract audio data
        let data_start = 48;
        let data_end = data_start + size.min(data.len() - data_start);
        let audio_data = data[data_start..data_end].to_vec();

        // Scan for loop markers
        let (loop_start, loop_end) = Self::find_loop_points(&audio_data);

        Ok(Vag {
            name,
            sample_rate,
            data: audio_data,
            loop_start,
            loop_end,
        })
    }

    /// Find loop start and end points by scanning block flags
    fn find_loop_points(data: &[u8]) -> (Option<usize>, Option<usize>) {
        let mut loop_start = None;
        let mut loop_end = None;

        for (block_idx, block) in data.chunks(16).enumerate() {
            if block.len() < 2 {
                break;
            }

            let flags = block[1];
            let sample_pos = block_idx * 28; // 28 samples per block

            match flags {
                0x02 => loop_start = Some(sample_pos), // Loop start
                0x03 => loop_end = Some(sample_pos),   // Loop end
                0x06 => {
                    // Loop start + end
                    loop_start = Some(sample_pos);
                    loop_end = Some(sample_pos + 28);
                }
                _ => {}
            }
        }

        (loop_start, loop_end)
    }

    /// Decode ADPCM data to 16-bit PCM
    ///
    /// Returns a Vec<i16> with decoded PCM samples
    pub fn decode_to_pcm(&self) -> Vec<i16> {
        let mut output = Vec::with_capacity(self.data.len() / 16 * 28);
        let mut hist1: i32 = 0;
        let mut hist2: i32 = 0;

        // ADPCM filter coefficients
        const FILTERS: [[i32; 2]; 5] = [[0, 0], [60, 0], [115, -52], [98, -55], [122, -60]];

        for block in self.data.chunks(16) {
            if block.len() < 16 {
                break;
            }

            let predict_nr = (block[0] & 0x0F) as usize;
            let shift_factor = (block[0] >> 4) as u32;

            if predict_nr >= FILTERS.len() {
                tracing::warn!("Invalid VAG predict_nr: {}", predict_nr);
                continue;
            }

            let filter = FILTERS[predict_nr];

            // Decode 28 samples (14 bytes * 2 nibbles per byte)
            for i in 0..14 {
                let byte = block[2 + i];

                // Process low nibble
                let mut sample = ((byte & 0x0F) as i8) << 4;
                sample >>= 4; // Sign extend
                let mut decoded = (sample as i32) << (12 - shift_factor);
                decoded += (hist1 * filter[0] + hist2 * filter[1] + 32) / 64;
                decoded = decoded.clamp(-32768, 32767);
                output.push(decoded as i16);
                hist2 = hist1;
                hist1 = decoded;

                // Process high nibble
                let mut sample = (byte & 0xF0) as i8;
                sample >>= 4; // Sign extend
                let mut decoded = (sample as i32) << (12 - shift_factor);
                decoded += (hist1 * filter[0] + hist2 * filter[1] + 32) / 64;
                decoded = decoded.clamp(-32768, 32767);
                output.push(decoded as i16);
                hist2 = hist1;
                hist1 = decoded;
            }
        }

        output
    }

    /// Get the duration in seconds
    pub fn duration_secs(&self) -> f64 {
        let num_samples = (self.data.len() / 16) * 28;
        num_samples as f64 / self.sample_rate as f64
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_vag_header_size() {
        assert_eq!(std::mem::size_of::<VagHeader>(), 48);
    }

    #[test]
    fn test_invalid_vag() {
        let data = vec![0u8; 10];
        assert!(Vag::parse(&data).is_err());
    }
}
