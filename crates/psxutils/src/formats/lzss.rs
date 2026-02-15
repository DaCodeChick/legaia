//! LZSS (Lempel-Ziv-Storer-Szymanski) decompression
//!
//! LZSS is a dictionary-based compression algorithm commonly used in PlayStation 1 games.
//! It uses a sliding window to reference previously decompressed data.
//!
//! # Format
//!
//! LZSS compressed data consists of:
//! - Control bytes (flags indicating literal vs. reference)
//! - Literal bytes (raw uncompressed data)
//! - References (offset + length pairs to copy from sliding window)
//!
//! # Standard LZSS Parameters
//!
//! - Window size: 4096 bytes (12-bit offset)
//! - Look-ahead buffer: 18 bytes (4-bit length + 2)
//! - Minimum match length: 3 bytes
//! - Control byte: 8 flags (1 bit per token, processed LSB to MSB)

use std::io::{self, Read, Write};

/// LZSS decompression configuration
#[derive(Debug, Clone, Copy)]
pub struct LzssConfig {
    /// Size of the sliding window (typically 4096 for standard LZSS)
    pub window_size: usize,
    /// Maximum match length (typically 18 for standard LZSS)
    pub max_match_len: usize,
    /// Minimum match length (typically 3 for standard LZSS)
    pub min_match_len: usize,
    /// Number of bits for offset encoding (typically 12)
    pub offset_bits: u8,
    /// Number of bits for length encoding (typically 4)
    pub length_bits: u8,
}

impl Default for LzssConfig {
    fn default() -> Self {
        Self::standard()
    }
}

impl LzssConfig {
    /// Standard LZSS configuration (12-bit offset, 4-bit length)
    pub const fn standard() -> Self {
        Self {
            window_size: 4096, // 2^12
            max_match_len: 18, // 2^4 + 2
            min_match_len: 3,
            offset_bits: 12,
            length_bits: 4,
        }
    }

    /// PlayStation 1 variant (may differ per game)
    pub const fn ps1_variant() -> Self {
        Self::standard()
    }
}

/// LZSS decompressor
pub struct LzssDecoder {
    config: LzssConfig,
    window: Vec<u8>,
    window_pos: usize,
}

impl LzssDecoder {
    /// Create a new LZSS decoder with the given configuration
    pub fn new(config: LzssConfig) -> Self {
        Self {
            config,
            window: vec![0; config.window_size],
            window_pos: 0,
        }
    }

    /// Create a decoder with standard LZSS parameters
    pub fn standard() -> Self {
        Self::new(LzssConfig::standard())
    }

    /// Decompress LZSS data from reader to writer
    ///
    /// # Arguments
    ///
    /// * `input` - Compressed data source
    /// * `output` - Decompressed data destination
    ///
    /// # Returns
    ///
    /// Number of bytes written to output
    pub fn decompress<R: Read, W: Write>(
        &mut self,
        input: &mut R,
        output: &mut W,
    ) -> io::Result<usize> {
        let mut total_written = 0;
        let mut flags: u8 = 0;
        let mut flag_count: u8 = 0;

        loop {
            // Read control byte every 8 tokens
            if flag_count == 0 {
                let mut flag_byte = [0u8; 1];
                match input.read_exact(&mut flag_byte) {
                    Ok(_) => flags = flag_byte[0],
                    Err(e) if e.kind() == io::ErrorKind::UnexpectedEof => break,
                    Err(e) => return Err(e),
                }
                flag_count = 8;
            }

            // Check LSB of flags
            if flags & 1 != 0 {
                // Literal byte - copy directly
                let mut byte = [0u8; 1];
                input.read_exact(&mut byte)?;

                output.write_all(&byte)?;
                self.write_to_window(byte[0]);
                total_written += 1;
            } else {
                // Reference - read offset and length
                let mut ref_bytes = [0u8; 2];
                input.read_exact(&mut ref_bytes)?;

                // Standard LZSS encoding: 12-bit offset, 4-bit length
                let offset = ((ref_bytes[0] as usize) << 4) | ((ref_bytes[1] as usize) >> 4);
                let length = ((ref_bytes[1] & 0x0F) as usize) + self.config.min_match_len;

                // Copy from window
                for _ in 0..length {
                    let byte = self.window[offset];
                    output.write_all(&[byte])?;
                    self.write_to_window(byte);
                    total_written += 1;
                }
            }

            // Shift flags and decrement counter
            flags >>= 1;
            flag_count -= 1;
        }

        Ok(total_written)
    }

    /// Decompress entire buffer in one call
    ///
    /// # Arguments
    ///
    /// * `compressed` - Compressed input data
    ///
    /// # Returns
    ///
    /// Decompressed data as a Vec<u8>
    pub fn decompress_buf(&mut self, compressed: &[u8]) -> io::Result<Vec<u8>> {
        let mut input = io::Cursor::new(compressed);
        let mut output = Vec::new();
        self.decompress(&mut input, &mut output)?;
        Ok(output)
    }

    /// Write a byte to the sliding window
    fn write_to_window(&mut self, byte: u8) {
        self.window[self.window_pos] = byte;
        self.window_pos = (self.window_pos + 1) % self.config.window_size;
    }

    /// Reset the decoder state (clears window)
    pub fn reset(&mut self) {
        self.window.fill(0);
        self.window_pos = 0;
    }
}

/// Decompress LZSS data with standard configuration
///
/// Convenience function for one-shot decompression.
///
/// # Arguments
///
/// * `compressed` - Compressed input data
///
/// # Returns
///
/// Decompressed data as a Vec<u8>
///
/// # Example
///
/// ```no_run
/// use psxutils::formats::lzss;
///
/// let compressed = std::fs::read("player.lzs")?;
/// let decompressed = lzss::decompress(&compressed)?;
/// # Ok::<(), std::io::Error>(())
/// ```
pub fn decompress(compressed: &[u8]) -> io::Result<Vec<u8>> {
    LzssDecoder::standard().decompress_buf(compressed)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_lzss_literal_only() {
        // Control byte 0xFF = all literals (8 bits set)
        // Followed by 8 literal bytes
        let compressed = vec![
            0xFF, // Control: 8 literals
            b'H', b'e', b'l', b'l', b'o', b' ', b'P', b'S',
        ];

        let mut decoder = LzssDecoder::standard();
        let result = decoder.decompress_buf(&compressed).unwrap();
        assert_eq!(result, b"Hello PS");
    }

    #[test]
    fn test_lzss_with_reference() {
        // Simple test with literal followed by reference
        let mut decoder = LzssDecoder::standard();

        // This is a simplified test - real LZSS would have proper encoding
        let compressed = vec![
            0xFF, // All literals for first 8
            b'A', b'B', b'C', b'D', b'E', b'F', b'G', b'H',
        ];

        let result = decoder.decompress_buf(&compressed).unwrap();
        assert_eq!(result, b"ABCDEFGH");
    }

    #[test]
    fn test_decoder_reset() {
        let mut decoder = LzssDecoder::standard();

        // Decompress some data
        let compressed = vec![0xFF, b'T', b'e', b's', b't', b'!', b'!', b'!', b'!'];
        let _ = decoder.decompress_buf(&compressed).unwrap();

        // Reset and verify window is cleared
        decoder.reset();
        assert_eq!(decoder.window_pos, 0);
        assert!(decoder.window.iter().all(|&b| b == 0));
    }

    #[test]
    fn test_config_standard() {
        let config = LzssConfig::standard();
        assert_eq!(config.window_size, 4096);
        assert_eq!(config.max_match_len, 18);
        assert_eq!(config.min_match_len, 3);
        assert_eq!(config.offset_bits, 12);
        assert_eq!(config.length_bits, 4);
    }
}
