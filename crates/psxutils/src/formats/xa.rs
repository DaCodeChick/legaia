//! CD-ROM XA (eXtended Architecture) audio format parser.
//!
//! XA audio is part of the "Green Book" CD standard used by PlayStation 1 games.
//! Audio data is stored in CD sectors using ADPCM compression.
//!
//! # Format Structure
//!
//! XA audio sectors are MODE2FORM2 sectors with:
//! - 8-byte XA sub-header (duplicated for error detection)
//! - 2304 bytes of audio data (18 sound groups × 128 bytes)
//! - Last 20 bytes unused
//!
//! # References
//!
//! Based on jPSXdec implementation:
//! - `jpsxdec/src/jpsxdec/cdreaders/XaAnalysis.java`
//! - `jpsxdec/src/jpsxdec/cdreaders/CdSectorXaSubHeader.java`
//! - `jpsxdec/src/jpsxdec/modules/xa/SectorXaAudio.java`

use std::fmt;

/// Size of XA sub-header in bytes
pub const XA_SUBHEADER_SIZE: usize = 8;

/// Number of sound groups per XA audio sector
pub const SOUND_GROUPS_PER_SECTOR: usize = 18;

/// Size of each sound group in bytes
pub const SOUND_GROUP_SIZE: usize = 128;

/// Total XA audio data size (excluding unused 20 bytes at end)
pub const XA_AUDIO_DATA_SIZE: usize = SOUND_GROUPS_PER_SECTOR * SOUND_GROUP_SIZE;

/// CD-ROM XA sub-header
///
/// The sub-header is duplicated twice (bytes 0-3 and 4-7) for error detection.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct XaSubHeader {
    /// File number (0-255)
    pub file_number: u8,
    /// Channel number (0-31 for audio)
    pub channel: u8,
    /// Sub-mode flags
    pub sub_mode: SubMode,
    /// Coding information
    pub coding_info: CodingInfo,
}

impl XaSubHeader {
    /// Parse XA sub-header from bytes
    ///
    /// The sub-header format is:
    /// - Byte 0: File number
    /// - Byte 1: Channel
    /// - Byte 2: Sub-mode flags
    /// - Byte 3: Coding info
    /// - Bytes 4-7: Duplicate of bytes 0-3 for error detection
    pub fn parse(data: &[u8]) -> Option<Self> {
        if data.len() < XA_SUBHEADER_SIZE {
            return None;
        }

        // Read both copies
        let file_number1 = data[0];
        let file_number2 = data[4];
        let channel1 = data[1];
        let channel2 = data[5];
        let sub_mode1 = SubMode::from_byte(data[2]);
        let sub_mode2 = SubMode::from_byte(data[6]);
        let coding_info1 = CodingInfo::from_byte(data[3]);
        let coding_info2 = CodingInfo::from_byte(data[7]);

        // Validate duplication (basic error detection)
        // Note: jPSXdec has sophisticated error correction, but for scanning
        // we just require the copies to match
        if file_number1 != file_number2
            || channel1 != channel2
            || sub_mode1.bits != sub_mode2.bits
            || coding_info1.bits != coding_info2.bits
        {
            return None;
        }

        // Validate sub-mode and coding info
        if !sub_mode1.is_valid() || !coding_info1.is_valid() {
            return None;
        }

        Some(Self {
            file_number: file_number1,
            channel: channel1,
            sub_mode: sub_mode1,
            coding_info: coding_info1,
        })
    }

    /// Check if this is an XA audio sector
    pub fn is_audio(&self) -> bool {
        // XA audio requires: Form2 + Audio + RealTime flags
        // Must NOT have Data or Video flags
        self.sub_mode.is_form2()
            && self.sub_mode.is_audio()
            && self.sub_mode.is_real_time()
            && !self.sub_mode.is_data()
            && !self.sub_mode.is_video()
    }
}

/// Sub-mode flags
///
/// 8-bit flag field with the following bits:
/// - Bit 7: End of File (EOF)
/// - Bit 6: Real-Time (RT)
/// - Bit 5: Form (0=Form1, 1=Form2)
/// - Bit 4: Trigger
/// - Bit 3: Data
/// - Bit 2: Audio
/// - Bit 1: Video
/// - Bit 0: End of Record (EOR)
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct SubMode {
    bits: u8,
}

impl SubMode {
    pub const MASK_END_OF_FILE: u8 = 0x80;
    pub const MASK_REAL_TIME: u8 = 0x40;
    pub const MASK_FORM: u8 = 0x20;
    pub const MASK_TRIGGER: u8 = 0x10;
    pub const MASK_DATA: u8 = 0x08;
    pub const MASK_AUDIO: u8 = 0x04;
    pub const MASK_VIDEO: u8 = 0x02;
    pub const MASK_END_OF_RECORD: u8 = 0x01;

    pub const fn from_byte(byte: u8) -> Self {
        Self { bits: byte }
    }

    pub const fn is_end_of_file(&self) -> bool {
        self.bits & Self::MASK_END_OF_FILE != 0
    }

    pub const fn is_real_time(&self) -> bool {
        self.bits & Self::MASK_REAL_TIME != 0
    }

    pub const fn is_form2(&self) -> bool {
        self.bits & Self::MASK_FORM != 0
    }

    pub const fn is_trigger(&self) -> bool {
        self.bits & Self::MASK_TRIGGER != 0
    }

    pub const fn is_data(&self) -> bool {
        self.bits & Self::MASK_DATA != 0
    }

    pub const fn is_audio(&self) -> bool {
        self.bits & Self::MASK_AUDIO != 0
    }

    pub const fn is_video(&self) -> bool {
        self.bits & Self::MASK_VIDEO != 0
    }

    pub const fn is_end_of_record(&self) -> bool {
        self.bits & Self::MASK_END_OF_RECORD != 0
    }

    /// Validate sub-mode flags
    ///
    /// Data, Audio, and Video bits should be mutually exclusive
    /// (only one of these three flags should be set)
    pub const fn is_valid(&self) -> bool {
        let dav_bits = (self.bits >> 1) & 0x07; // Extract bits 3,2,1 (Data, Audio, Video)
        let count = dav_bits.count_ones();
        count <= 1 // At most one of Data/Audio/Video should be set
    }
}

impl fmt::Display for SubMode {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "{}{}{}{}{}{}{}{}",
            if self.is_end_of_file() { 'F' } else { '-' },
            if self.is_real_time() { 'R' } else { '-' },
            if self.is_form2() { '2' } else { '1' },
            if self.is_trigger() { 'T' } else { '-' },
            if self.is_data() { 'D' } else { '-' },
            if self.is_audio() { 'A' } else { '-' },
            if self.is_video() { 'V' } else { '-' },
            if self.is_end_of_record() { 'E' } else { '-' },
        )
    }
}

/// Coding information for XA ADPCM audio
///
/// 8-bit field encoding audio parameters:
/// - Bits 7,6: Reserved (unused, bit 5,3,1 must be 0)
/// - Bits 5,4: Bits per sample (00=4-bit, 01=8-bit)
/// - Bits 3,2: Sample rate (00=37800Hz, 01=18900Hz)
/// - Bits 1,0: Stereo (00=mono, 01=stereo)
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct CodingInfo {
    bits: u8,
}

impl CodingInfo {
    const MASK_BITS_PER_SAMPLE: u8 = 0x10; // Bit 4
    const MASK_SAMPLE_RATE: u8 = 0x04; // Bit 2
    const MASK_STEREO: u8 = 0x01; // Bit 0
    const MASK_EMPHASIS: u8 = 0x40; // Bit 6
    const MASK_RESERVED: u8 = 0x2A; // Bits 5, 3, 1 (reserved, must be 0)

    pub const fn from_byte(byte: u8) -> Self {
        Self { bits: byte }
    }

    /// Get bits per sample (4 or 8)
    pub const fn bits_per_sample(&self) -> u8 {
        if self.bits & Self::MASK_BITS_PER_SAMPLE == 0 {
            4
        } else {
            8
        }
    }

    /// Get sample rate in Hz (37800 or 18900)
    pub const fn sample_rate(&self) -> u32 {
        if self.bits & Self::MASK_SAMPLE_RATE == 0 {
            37800
        } else {
            18900
        }
    }

    /// Check if stereo (true) or mono (false)
    pub const fn is_stereo(&self) -> bool {
        self.bits & Self::MASK_STEREO != 0
    }

    /// Check if emphasis flag is set
    pub const fn has_emphasis(&self) -> bool {
        self.bits & Self::MASK_EMPHASIS != 0
    }

    /// Validate coding info
    ///
    /// Bits 5, 3, and 1 must be 0 (reserved/unused)
    pub const fn is_valid(&self) -> bool {
        self.bits & Self::MASK_RESERVED == 0
    }
}

impl fmt::Display for CodingInfo {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "{} {}-bit {}Hz",
            if self.is_stereo() { "Stereo" } else { "Mono" },
            self.bits_per_sample(),
            self.sample_rate()
        )
    }
}

/// XA audio stream metadata
#[derive(Debug, Clone)]
pub struct XaAudioStream {
    /// Start sector number
    pub start_sector: u32,
    /// End sector number (inclusive)
    pub end_sector: u32,
    /// File number
    pub file_number: u8,
    /// Channel number
    pub channel: u8,
    /// Audio format
    pub coding_info: CodingInfo,
    /// Number of sectors in stream
    pub sector_count: u32,
}

impl XaAudioStream {
    /// Calculate total duration in seconds
    pub fn duration_seconds(&self) -> f64 {
        // Each sector generates a fixed number of PCM samples
        let samples_per_sector = match self.coding_info.bits_per_sample() {
            4 => 28 * 8, // 8 sound units × 28 samples each
            8 => 28 * 4, // 4 sound units × 28 samples each
            _ => 0,
        };

        let total_samples = self.sector_count * samples_per_sector;
        let sample_rate = self.coding_info.sample_rate();

        total_samples as f64 / sample_rate as f64
    }

    /// Calculate total size in bytes
    pub fn total_size(&self) -> usize {
        self.sector_count as usize * XA_AUDIO_DATA_SIZE
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_submode_validation() {
        // Valid: Only Audio flag set
        let submode = SubMode::from_byte(0x04);
        assert!(submode.is_valid());
        assert!(submode.is_audio());

        // Invalid: Both Audio and Video flags set
        let submode = SubMode::from_byte(0x06);
        assert!(!submode.is_valid());

        // Valid: Form2 + Audio + RealTime (typical XA audio)
        let submode = SubMode::from_byte(0x64); // 0b01100100
        assert!(submode.is_valid());
        assert!(submode.is_form2());
        assert!(submode.is_audio());
        assert!(submode.is_real_time());
    }

    #[test]
    fn test_coding_info() {
        // 4-bit, 37.8kHz, Mono
        let coding = CodingInfo::from_byte(0x00);
        assert!(coding.is_valid());
        assert_eq!(coding.bits_per_sample(), 4);
        assert_eq!(coding.sample_rate(), 37800);
        assert!(!coding.is_stereo());

        // 8-bit, 18.9kHz, Stereo
        let coding = CodingInfo::from_byte(0x15);
        assert!(coding.is_valid());
        assert_eq!(coding.bits_per_sample(), 8);
        assert_eq!(coding.sample_rate(), 18900);
        assert!(coding.is_stereo());

        // Invalid: Has reserved bit set
        let coding = CodingInfo::from_byte(0x02);
        assert!(!coding.is_valid());
    }

    #[test]
    fn test_xa_subheader_parse() {
        // Valid XA sub-header with duplication
        let data = [
            1, 2, 0x64, 0x00, // File 1, Channel 2, SubMode, Coding
            1, 2, 0x64, 0x00, // Duplicate
        ];

        let header = XaSubHeader::parse(&data).unwrap();
        assert_eq!(header.file_number, 1);
        assert_eq!(header.channel, 2);
        assert!(header.is_audio());

        // Invalid: Mismatched duplication
        let data = [
            1, 2, 0x64, 0x00, // File 1
            2, 2, 0x64, 0x00, // File 2 (mismatch!)
        ];

        assert!(XaSubHeader::parse(&data).is_none());
    }

    #[test]
    fn test_xa_audio_stream_duration() {
        let stream = XaAudioStream {
            start_sector: 0,
            end_sector: 74,
            file_number: 1,
            channel: 0,
            coding_info: CodingInfo::from_byte(0x00), // 4-bit, 37.8kHz, Mono
            sector_count: 75,
        };

        // 75 sectors × 224 samples/sector ÷ 37800 Hz ≈ 0.444 seconds
        let duration = stream.duration_seconds();
        assert!((duration - 0.444).abs() < 0.001);
    }
}
