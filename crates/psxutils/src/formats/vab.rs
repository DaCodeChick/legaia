//! VAB (Sound Bank) format parser
//!
//! VAB is the PlayStation 1 sound bank format that contains multiple VAG samples
//! organized into programs (instruments) with parameters like ADSR, pitch, and volume.
//!
//! ## Format Specification
//!
//! ```text
//! VAB Header (2048 bytes):
//!   char[4] magic      = "VABp"
//!   u32 version        = 0x00000007
//!   u32 vab_id         
//!   u32 size           // Total size
//!   u16 reserved
//!   u16 num_programs   // Number of programs (max 128)
//!   u16 num_tones      // Number of tones (max 256)  
//!   u16 num_vags       // Number of VAG samples (max 256)
//!   u8  master_volume  // Master volume (0-127)
//!   u8  master_pan     // Master pan (0-127, 64=center)
//!   u8  bank_attr1
//!   u8  bank_attr2
//!   u32 reserved2
//!
//! Programs (128 * 16 bytes):
//!   u8 num_tones       // Number of tones in this program
//!   u8 volume          // Program volume
//!   u8 priority        // Program priority
//!   u8 mode            // Program mode
//!   u8 pan             // Pan
//!   u8 reserved
//!   i16 pitch_bend     // Pitch bend
//!   u8[8] reserved
//!
//! Tones (256 * 32 bytes):
//!   u8 priority
//!   u8 mode
//!   u8 volume
//!   u8 pan
//!   u8 center_note     // Center note (60 = Middle C)
//!   u8 center_tune     // Fine tuning
//!   u8 min_note        // Minimum note
//!   u8 max_note        // Maximum note
//!   u8 vibrato_width
//!   u8 vibrato_time
//!   u8 portamento_width
//!   u8 portamento_time
//!   u8 pitch_bend_min
//!   u8 pitch_bend_max
//!   u8 reserved1
//!   u8 reserved2
//!   u16 adsr1          // ADSR envelope part 1
//!   u16 adsr2          // ADSR envelope part 2
//!   i16 prog           // Parent program number
//!   i16 vag            // VAG sample index
//!   i16 reserved3
//!   i16 reserved4
//!
//! VAG Offsets (256 * 2 bytes):
//!   u16[256] offsets   // Offset table to VAG data (in 2KB units)
//!
//! VAG Sizes (256 * 2 bytes):
//!   u16[256] sizes     // Size table for VAG data (in 2KB units)
//!
//! VAG Data:
//!   Raw VAG sample data concatenated
//! ```

use crate::{PsxError, Result};
use bytemuck::{Pod, Zeroable};

/// VAB magic number "VABp"
pub const VAB_MAGIC: [u8; 4] = *b"VABp";

/// VAB version
pub const VAB_VERSION: u32 = 0x00000007;

/// Maximum number of programs in a VAB
pub const MAX_PROGRAMS: usize = 128;

/// Maximum number of tones in a VAB
pub const MAX_TONES: usize = 256;

/// Maximum number of VAG samples in a VAB
pub const MAX_VAGS: usize = 256;

/// VAB file header (32 bytes, rest of 2048 is padding/reserved)
#[repr(C)]
#[derive(Debug, Clone, Copy, Pod, Zeroable)]
struct VabHeader {
    magic: [u8; 4],    // "VABp"
    version: u32,      // 0x00000007
    vab_id: u32,       // VAB ID
    size: u32,         // Total file size
    reserved1: u16,    // Reserved
    num_programs: u16, // Number of programs
    num_tones: u16,    // Number of tones
    num_vags: u16,     // Number of VAG samples
    master_volume: u8, // Master volume (0-127)
    master_pan: u8,    // Master pan (0-127)
    bank_attr1: u8,    // Bank attribute 1
    bank_attr2: u8,    // Bank attribute 2
    reserved2: u32,    // Reserved
}

/// Program entry (16 bytes)
#[repr(C)]
#[derive(Debug, Clone, Copy, Pod, Zeroable)]
struct ProgramEntry {
    num_tones: u8,
    volume: u8,
    priority: u8,
    mode: u8,
    pan: u8,
    reserved1: u8,
    pitch_bend: i16,
    reserved2: [u8; 8],
}

/// Tone entry (32 bytes)
#[repr(C, packed)]
#[derive(Debug, Clone, Copy, Pod, Zeroable)]
struct ToneEntry {
    priority: u8,
    mode: u8,
    volume: u8,
    pan: u8,
    center_note: u8,
    center_tune: u8,
    min_note: u8,
    max_note: u8,
    vibrato_width: u8,
    vibrato_time: u8,
    portamento_width: u8,
    portamento_time: u8,
    pitch_bend_min: u8,
    pitch_bend_max: u8,
    reserved1: u8,
    reserved2: u8,
    adsr1: u16,
    adsr2: u16,
    prog: i16,
    vag: i16,
    reserved3: i16,
    reserved4: i16,
    _padding: [u8; 4], // Padding to make 32 bytes
}

/// Parsed VAB sound bank
#[derive(Debug, Clone)]
pub struct Vab {
    /// VAB ID
    pub vab_id: u32,
    /// Master volume (0-127)
    pub master_volume: u8,
    /// Master pan (0-127, 64=center)
    pub master_pan: u8,
    /// Programs (instruments)
    pub programs: Vec<Program>,
    /// Tones (individual sounds)
    pub tones: Vec<Tone>,
    /// VAG samples (raw audio data)
    pub vag_samples: Vec<VagSample>,
}

/// Program (instrument) in a VAB
#[derive(Debug, Clone)]
pub struct Program {
    pub num_tones: u8,
    pub volume: u8,
    pub priority: u8,
    pub mode: u8,
    pub pan: u8,
    pub pitch_bend: i16,
}

/// Tone (individual sound) in a VAB
#[derive(Debug, Clone)]
pub struct Tone {
    pub priority: u8,
    pub mode: u8,
    pub volume: u8,
    pub pan: u8,
    pub center_note: u8,
    pub center_tune: u8,
    pub min_note: u8,
    pub max_note: u8,
    pub vibrato_width: u8,
    pub vibrato_time: u8,
    pub portamento_width: u8,
    pub portamento_time: u8,
    pub pitch_bend_min: u8,
    pub pitch_bend_max: u8,
    pub adsr1: u16,
    pub adsr2: u16,
    pub program_index: i16,
    pub vag_index: i16,
}

/// VAG sample within a VAB
#[derive(Debug, Clone)]
pub struct VagSample {
    /// Raw VAG data
    pub data: Vec<u8>,
}

impl Vab {
    /// Parse a VAB file from bytes
    pub fn parse(data: &[u8]) -> Result<Self> {
        if data.len() < 2048 {
            return Err(PsxError::InvalidFormat("VAB file too small".to_string()));
        }

        // Parse header
        let header: &VabHeader = bytemuck::try_from_bytes(&data[0..32])
            .map_err(|e| PsxError::ParseError(format!("Failed to parse VAB header: {}", e)))?;

        if header.magic != VAB_MAGIC {
            return Err(PsxError::InvalidFormat(format!(
                "Invalid VAB magic: {:?}, expected {:?}",
                header.magic, VAB_MAGIC
            )));
        }

        let version = u32::from_le(header.version);
        if version != VAB_VERSION {
            tracing::warn!("Unexpected VAB version: 0x{:08X}", version);
        }

        let num_programs = u16::from_le(header.num_programs) as usize;
        let num_tones = u16::from_le(header.num_tones) as usize;
        let num_vags = u16::from_le(header.num_vags) as usize;

        // Parse programs (starts at offset 2048)
        let mut programs = Vec::with_capacity(num_programs);
        for i in 0..num_programs.min(MAX_PROGRAMS) {
            let offset = 2048 + i * 16;
            if data.len() < offset + 16 {
                break;
            }

            let prog: &ProgramEntry = bytemuck::try_from_bytes(&data[offset..offset + 16])
                .map_err(|e| {
                    PsxError::ParseError(format!("Failed to parse program {}: {}", i, e))
                })?;

            programs.push(Program {
                num_tones: prog.num_tones,
                volume: prog.volume,
                priority: prog.priority,
                mode: prog.mode,
                pan: prog.pan,
                pitch_bend: i16::from_le(prog.pitch_bend),
            });
        }

        // Parse tones (starts at offset 2048 + 128*16 = 4096)
        let mut tones = Vec::with_capacity(num_tones);
        for i in 0..num_tones.min(MAX_TONES) {
            let offset = 4096 + i * 32;
            if data.len() < offset + 32 {
                break;
            }

            let tone: &ToneEntry = bytemuck::try_from_bytes(&data[offset..offset + 32])
                .map_err(|e| PsxError::ParseError(format!("Failed to parse tone {}: {}", i, e)))?;

            tones.push(Tone {
                priority: tone.priority,
                mode: tone.mode,
                volume: tone.volume,
                pan: tone.pan,
                center_note: tone.center_note,
                center_tune: tone.center_tune,
                min_note: tone.min_note,
                max_note: tone.max_note,
                vibrato_width: tone.vibrato_width,
                vibrato_time: tone.vibrato_time,
                portamento_width: tone.portamento_width,
                portamento_time: tone.portamento_time,
                pitch_bend_min: tone.pitch_bend_min,
                pitch_bend_max: tone.pitch_bend_max,
                adsr1: u16::from_le(tone.adsr1),
                adsr2: u16::from_le(tone.adsr2),
                program_index: i16::from_le(tone.prog),
                vag_index: i16::from_le(tone.vag),
            });
        }

        // Parse VAG offset table (starts at 4096 + 256*32 = 12288)
        let vag_offset_table_start = 12288;
        let vag_size_table_start = vag_offset_table_start + 512; // 256 * 2

        let mut vag_samples = Vec::with_capacity(num_vags);
        for i in 0..num_vags.min(MAX_VAGS) {
            let offset_idx = vag_offset_table_start + i * 2;
            let size_idx = vag_size_table_start + i * 2;

            if data.len() < size_idx + 2 {
                break;
            }

            // Read offset and size (in 2KB units)
            let offset_units =
                u16::from_le_bytes([data[offset_idx], data[offset_idx + 1]]) as usize;
            let size_units = u16::from_le_bytes([data[size_idx], data[size_idx + 1]]) as usize;

            let vag_offset = offset_units * 2048;
            let vag_size = size_units * 2048;

            if vag_size > 0 && data.len() >= vag_offset + vag_size {
                let vag_data = data[vag_offset..vag_offset + vag_size].to_vec();
                vag_samples.push(VagSample { data: vag_data });
            } else {
                vag_samples.push(VagSample { data: Vec::new() });
            }
        }

        Ok(Vab {
            vab_id: u32::from_le(header.vab_id),
            master_volume: header.master_volume,
            master_pan: header.master_pan,
            programs,
            tones,
            vag_samples,
        })
    }

    /// Get a specific VAG sample by index
    pub fn get_vag(&self, index: usize) -> Option<&VagSample> {
        self.vag_samples.get(index)
    }

    /// Get a specific program by index
    pub fn get_program(&self, index: usize) -> Option<&Program> {
        self.programs.get(index)
    }

    /// Get a specific tone by index
    pub fn get_tone(&self, index: usize) -> Option<&Tone> {
        self.tones.get(index)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_vab_header_size() {
        assert_eq!(std::mem::size_of::<VabHeader>(), 32);
    }

    #[test]
    fn test_program_entry_size() {
        assert_eq!(std::mem::size_of::<ProgramEntry>(), 16);
    }

    #[test]
    fn test_tone_entry_size() {
        assert_eq!(std::mem::size_of::<ToneEntry>(), 32);
    }

    #[test]
    fn test_invalid_vab() {
        let data = vec![0u8; 10];
        assert!(Vab::parse(&data).is_err());
    }
}
