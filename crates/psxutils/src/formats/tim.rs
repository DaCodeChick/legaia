//! TIM (Texture Image) format parser
//!
//! TIM is the standard PlayStation 1 texture format. It supports:
//! - 4-bit indexed color (16 colors with CLUT)
//! - 8-bit indexed color (256 colors with CLUT)
//! - 16-bit direct color (RGB555)
//! - 24-bit direct color (RGB888)
//!
//! ## Format Specification
//!
//! ```text
//! TIM Header:
//!   u32 magic    = 0x10       // TIM magic number
//!   u32 flags    = bit field  // Bits 0-2: pixel mode, bit 3: has CLUT
//!
//! If has CLUT:
//!   u32 clut_size            // Size of CLUT block in bytes
//!   u16 clut_x, clut_y       // CLUT position in VRAM
//!   u16 clut_w, clut_h       // CLUT dimensions
//!   [u16; clut_w * clut_h]   // CLUT data
//!
//! Pixel Data:
//!   u32 pixel_size           // Size of pixel block in bytes
//!   u16 pixel_x, pixel_y     // Pixel position in VRAM
//!   u16 pixel_w, pixel_h     // Pixel dimensions
//!   [u8; pixel_size - 12]    // Pixel data
//! ```

use crate::{PsxError, Result};
use bytemuck::{Pod, Zeroable};

/// TIM magic number (0x10)
pub const TIM_MAGIC: u32 = 0x10;

/// Maximum reasonable TIM dimensions (from jPSXdec)
const MAX_TIM_WORD_WIDTH: u16 = 16384;
const MAX_TIM_HEIGHT: u16 = 8192;

/// TIM pixel modes
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PixelMode {
    /// 4-bit indexed color (16 colors, requires CLUT)
    Clut4Bit = 0,
    /// 8-bit indexed color (256 colors, requires CLUT)
    Clut8Bit = 1,
    /// 16-bit direct color (RGB555, 1 bit unused)
    Direct16Bit = 2,
    /// 24-bit direct color (RGB888)
    Direct24Bit = 3,
    /// Mixed mode (rare, used for special effects)
    Mixed = 4,
}

impl PixelMode {
    fn from_u32(value: u32) -> Result<Self> {
        match value & 0x7 {
            0 => Ok(Self::Clut4Bit),
            1 => Ok(Self::Clut8Bit),
            2 => Ok(Self::Direct16Bit),
            3 => Ok(Self::Direct24Bit),
            4 => Ok(Self::Mixed),
            _ => Err(PsxError::InvalidFormat(format!(
                "Invalid TIM pixel mode: {}",
                value
            ))),
        }
    }

    /// Returns the bits per pixel for this mode
    pub fn bits_per_pixel(&self) -> u8 {
        match self {
            Self::Clut4Bit => 4,
            Self::Clut8Bit => 8,
            Self::Direct16Bit => 16,
            Self::Direct24Bit => 24,
            Self::Mixed => 16,
        }
    }
}

/// TIM file header
#[repr(C)]
#[derive(Debug, Clone, Copy, Pod, Zeroable)]
struct TimHeader {
    magic: u32,
    flags: u32,
}

/// Color Lookup Table (CLUT) header
#[repr(C)]
#[derive(Debug, Clone, Copy, Pod, Zeroable)]
struct ClutHeader {
    size: u32,   // Total size of CLUT block including header
    vram_x: u16, // X position in VRAM
    vram_y: u16, // Y position in VRAM
    width: u16,  // Width in colors
    height: u16, // Height in palette rows
}

/// Pixel data header
#[repr(C)]
#[derive(Debug, Clone, Copy, Pod, Zeroable)]
struct PixelHeader {
    size: u32,   // Total size of pixel block including header
    vram_x: u16, // X position in VRAM
    vram_y: u16, // Y position in VRAM
    width: u16,  // Width in pixels (or half-pixels for 4-bit)
    height: u16, // Height in pixels
}

/// Parsed TIM texture
#[derive(Debug, Clone)]
pub struct Tim {
    /// Pixel mode (color depth)
    pub pixel_mode: PixelMode,
    /// Whether this TIM has a CLUT (color lookup table)
    pub has_clut: bool,
    /// CLUT data (for indexed color modes)
    pub clut: Option<ClutData>,
    /// Pixel data
    pub pixels: PixelData,
}

/// Color Lookup Table data
#[derive(Debug, Clone)]
pub struct ClutData {
    /// Position in VRAM
    pub vram_pos: (u16, u16),
    /// Dimensions (width x height)
    pub dimensions: (u16, u16),
    /// Raw CLUT data (RGB555 format)
    pub data: Vec<u16>,
}

/// Pixel data
#[derive(Debug, Clone)]
pub struct PixelData {
    /// Position in VRAM
    pub vram_pos: (u16, u16),
    /// Dimensions (width x height in pixels)
    pub dimensions: (u16, u16),
    /// Raw pixel data
    pub data: Vec<u8>,
}

impl Tim {
    /// Parse a TIM file from bytes
    pub fn parse(data: &[u8]) -> Result<Self> {
        if data.len() < 8 {
            return Err(PsxError::InvalidFormat("TIM file too small".to_string()));
        }

        // Parse header
        let header: &TimHeader = bytemuck::try_from_bytes(&data[0..8])
            .map_err(|e| PsxError::ParseError(format!("Failed to parse TIM header: {}", e)))?;

        if header.magic != TIM_MAGIC {
            return Err(PsxError::InvalidFormat(format!(
                "Invalid TIM magic: 0x{:08X}, expected 0x{:08X}",
                header.magic, TIM_MAGIC
            )));
        }

        let pixel_mode = PixelMode::from_u32(header.flags)?;
        let has_clut = (header.flags & 0x8) != 0;

        let mut offset = 8;

        // Parse CLUT if present
        let clut = if has_clut {
            if data.len() < offset + 12 {
                return Err(PsxError::InvalidFormat(
                    "TIM file truncated (CLUT header)".to_string(),
                ));
            }

            let clut_header: &ClutHeader = bytemuck::try_from_bytes(&data[offset..offset + 12])
                .map_err(|e| PsxError::ParseError(format!("Failed to parse CLUT header: {}", e)))?;

            offset += 12;

            let clut_data_size = (clut_header.size as usize).saturating_sub(12);

            // Sanity check for CLUT size
            const MAX_CLUT_SIZE: usize = 256 * 2; // 256 colors * 2 bytes max
            if clut_data_size > MAX_CLUT_SIZE {
                return Err(PsxError::InvalidFormat(format!(
                    "TIM CLUT data size too large: {} bytes (max {} bytes)",
                    clut_data_size, MAX_CLUT_SIZE
                )));
            }

            if data.len() < offset + clut_data_size {
                return Err(PsxError::InvalidFormat(
                    "TIM file truncated (CLUT data)".to_string(),
                ));
            }

            let clut_data = data[offset..offset + clut_data_size]
                .chunks_exact(2)
                .map(|chunk| u16::from_le_bytes([chunk[0], chunk[1]]))
                .collect();

            offset += clut_data_size;

            Some(ClutData {
                vram_pos: (clut_header.vram_x, clut_header.vram_y),
                dimensions: (clut_header.width, clut_header.height),
                data: clut_data,
            })
        } else {
            None
        };

        // Parse pixel data
        if data.len() < offset + 12 {
            return Err(PsxError::InvalidFormat(
                "TIM file truncated (pixel header)".to_string(),
            ));
        }

        let pixel_header: &PixelHeader = bytemuck::try_from_bytes(&data[offset..offset + 12])
            .map_err(|e| PsxError::ParseError(format!("Failed to parse pixel header: {}", e)))?;

        offset += 12;

        let pixel_data_size = (pixel_header.size as usize).saturating_sub(12);

        // Sanity check: PSX VRAM is only 1MB, so texture data should be reasonable
        // Reject absurdly large sizes that indicate corrupt data
        const MAX_REASONABLE_SIZE: usize = 10 * 1024 * 1024; // 10MB max
        if pixel_data_size > MAX_REASONABLE_SIZE {
            return Err(PsxError::InvalidFormat(format!(
                "TIM pixel data size too large: {} bytes (max {} bytes)",
                pixel_data_size, MAX_REASONABLE_SIZE
            )));
        }

        if data.len() < offset + pixel_data_size {
            return Err(PsxError::InvalidFormat(
                "TIM file truncated (pixel data)".to_string(),
            ));
        }

        let pixel_data = data[offset..offset + pixel_data_size].to_vec();

        Ok(Tim {
            pixel_mode,
            has_clut,
            clut,
            pixels: PixelData {
                vram_pos: (pixel_header.vram_x, pixel_header.vram_y),
                dimensions: (pixel_header.width, pixel_header.height),
                data: pixel_data,
            },
        })
    }

    /// Validate TIM format without allocating memory for pixel data
    ///
    /// This is much faster than `parse()` for scanning, as it only validates
    /// headers and doesn't allocate Vec for pixel/CLUT data.
    ///
    /// Returns `Ok((width, height, total_size))` if valid, where total_size is
    /// the size of the complete TIM file in bytes.
    pub fn validate(data: &[u8]) -> Result<(u16, u16, usize)> {
        if data.len() < 8 {
            return Err(PsxError::InvalidFormat("TIM file too small".to_string()));
        }

        // Parse header
        let header: &TimHeader = bytemuck::try_from_bytes(&data[0..8])
            .map_err(|e| PsxError::ParseError(format!("Failed to parse TIM header: {}", e)))?;

        if header.magic != TIM_MAGIC {
            return Err(PsxError::InvalidFormat(format!(
                "Invalid TIM magic: 0x{:08X}, expected 0x{:08X}",
                header.magic, TIM_MAGIC
            )));
        }

        let pixel_mode = PixelMode::from_u32(header.flags)?;
        let has_clut = (header.flags & 0x8) != 0;

        let mut offset = 8;
        let mut total_size = 8; // Header size

        // Validate CLUT if present (without reading data)
        if has_clut {
            if data.len() < offset + 12 {
                return Err(PsxError::InvalidFormat(
                    "TIM file truncated (CLUT header)".to_string(),
                ));
            }

            let clut_header: &ClutHeader = bytemuck::try_from_bytes(&data[offset..offset + 12])
                .map_err(|e| PsxError::ParseError(format!("Failed to parse CLUT header: {}", e)))?;

            offset += 12;

            let clut_data_size = (clut_header.size as usize).saturating_sub(12);

            // Sanity check for CLUT size
            const MAX_CLUT_SIZE: usize = 256 * 2; // 256 colors * 2 bytes max
            if clut_data_size > MAX_CLUT_SIZE {
                return Err(PsxError::InvalidFormat(format!(
                    "TIM CLUT data size too large: {} bytes (max {} bytes)",
                    clut_data_size, MAX_CLUT_SIZE
                )));
            }

            // Validate CLUT dimensions
            if clut_header.width > MAX_TIM_WORD_WIDTH || clut_header.height > MAX_TIM_HEIGHT {
                return Err(PsxError::InvalidFormat(format!(
                    "TIM CLUT dimensions too large: {}x{} (max {}x{})",
                    clut_header.width, clut_header.height, MAX_TIM_WORD_WIDTH, MAX_TIM_HEIGHT
                )));
            }

            if data.len() < offset + clut_data_size {
                return Err(PsxError::InvalidFormat(
                    "TIM file truncated (CLUT data)".to_string(),
                ));
            }

            offset += clut_data_size;
            total_size += 12 + clut_data_size;
        }

        // Validate pixel data (without reading data)
        if data.len() < offset + 12 {
            return Err(PsxError::InvalidFormat(
                "TIM file truncated (pixel header)".to_string(),
            ));
        }

        let pixel_header: &PixelHeader = bytemuck::try_from_bytes(&data[offset..offset + 12])
            .map_err(|e| PsxError::ParseError(format!("Failed to parse pixel header: {}", e)))?;

        offset += 12;

        let pixel_data_size = (pixel_header.size as usize).saturating_sub(12);

        // Validate pixel dimensions
        if pixel_header.width > MAX_TIM_WORD_WIDTH || pixel_header.height > MAX_TIM_HEIGHT {
            return Err(PsxError::InvalidFormat(format!(
                "TIM pixel dimensions too large: {}x{} (max {}x{})",
                pixel_header.width, pixel_header.height, MAX_TIM_WORD_WIDTH, MAX_TIM_HEIGHT
            )));
        }

        // Sanity check: calculate maximum possible size based on dimensions
        let max_possible_size = (MAX_TIM_WORD_WIDTH as usize * 2 * MAX_TIM_HEIGHT as usize) + 12;
        if pixel_data_size > max_possible_size {
            return Err(PsxError::InvalidFormat(format!(
                "TIM pixel data size too large: {} bytes (max {} bytes)",
                pixel_data_size, max_possible_size
            )));
        }

        if data.len() < offset + pixel_data_size {
            return Err(PsxError::InvalidFormat(
                "TIM file truncated (pixel data)".to_string(),
            ));
        }

        total_size += 12 + pixel_data_size;

        // Calculate actual pixel dimensions
        let width = match pixel_mode {
            PixelMode::Clut4Bit => pixel_header.width * 4,
            PixelMode::Clut8Bit => pixel_header.width * 2,
            PixelMode::Direct16Bit => pixel_header.width,
            PixelMode::Direct24Bit => pixel_header.width * 2 / 3,
            PixelMode::Mixed => pixel_header.width,
        };

        Ok((width, pixel_header.height, total_size))
    }

    /// Convert to RGBA8 format
    ///
    /// Returns a Vec<u8> with RGBA data (4 bytes per pixel)
    pub fn to_rgba8(&self) -> Result<Vec<u8>> {
        match self.pixel_mode {
            PixelMode::Direct16Bit => self.convert_16bit_to_rgba8(),
            PixelMode::Direct24Bit => self.convert_24bit_to_rgba8(),
            PixelMode::Clut4Bit => self.convert_4bit_to_rgba8(),
            PixelMode::Clut8Bit => self.convert_8bit_to_rgba8(),
            PixelMode::Mixed => Err(PsxError::InvalidFormat(
                "Mixed mode TIM conversion not yet supported".to_string(),
            )),
        }
    }

    fn convert_16bit_to_rgba8(&self) -> Result<Vec<u8>> {
        let mut rgba = Vec::with_capacity(
            self.pixels.dimensions.0 as usize * self.pixels.dimensions.1 as usize * 4,
        );

        for chunk in self.pixels.data.chunks_exact(2) {
            let color = u16::from_le_bytes([chunk[0], chunk[1]]);
            // RGB555 format: 0BBBBBGGGGGRRRRR
            let r = ((color & 0x1F) << 3) as u8;
            let g = (((color >> 5) & 0x1F) << 3) as u8;
            let b = (((color >> 10) & 0x1F) << 3) as u8;
            let a = if color & 0x8000 == 0 { 0 } else { 255 }; // Semi-transparency bit

            rgba.extend_from_slice(&[r, g, b, a]);
        }

        Ok(rgba)
    }

    fn convert_24bit_to_rgba8(&self) -> Result<Vec<u8>> {
        let mut rgba = Vec::with_capacity(
            self.pixels.dimensions.0 as usize * self.pixels.dimensions.1 as usize * 4,
        );

        for chunk in self.pixels.data.chunks_exact(3) {
            rgba.extend_from_slice(&[chunk[0], chunk[1], chunk[2], 255]);
        }

        Ok(rgba)
    }

    fn convert_4bit_to_rgba8(&self) -> Result<Vec<u8>> {
        let clut = self
            .clut
            .as_ref()
            .ok_or_else(|| PsxError::InvalidFormat("4-bit TIM requires CLUT".to_string()))?;

        let mut rgba = Vec::with_capacity(
            self.pixels.dimensions.0 as usize * 2 * self.pixels.dimensions.1 as usize * 4,
        );

        for byte in &self.pixels.data {
            // Each byte contains 2 pixels (4 bits each)
            let idx1 = (byte & 0x0F) as usize;
            let idx2 = ((byte >> 4) & 0x0F) as usize;

            for idx in [idx1, idx2] {
                if idx < clut.data.len() {
                    let color = clut.data[idx];
                    let r = ((color & 0x1F) << 3) as u8;
                    let g = (((color >> 5) & 0x1F) << 3) as u8;
                    let b = (((color >> 10) & 0x1F) << 3) as u8;
                    let a = if color & 0x8000 == 0 { 0 } else { 255 };
                    rgba.extend_from_slice(&[r, g, b, a]);
                }
            }
        }

        Ok(rgba)
    }

    fn convert_8bit_to_rgba8(&self) -> Result<Vec<u8>> {
        let clut = self
            .clut
            .as_ref()
            .ok_or_else(|| PsxError::InvalidFormat("8-bit TIM requires CLUT".to_string()))?;

        let mut rgba = Vec::with_capacity(
            self.pixels.dimensions.0 as usize * self.pixels.dimensions.1 as usize * 4,
        );

        for &idx in &self.pixels.data {
            let idx = idx as usize;
            if idx < clut.data.len() {
                let color = clut.data[idx];
                let r = ((color & 0x1F) << 3) as u8;
                let g = (((color >> 5) & 0x1F) << 3) as u8;
                let b = (((color >> 10) & 0x1F) << 3) as u8;
                let a = if color & 0x8000 == 0 { 0 } else { 255 };
                rgba.extend_from_slice(&[r, g, b, a]);
            }
        }

        Ok(rgba)
    }

    /// Get the width in pixels
    pub fn width(&self) -> u16 {
        match self.pixel_mode {
            PixelMode::Clut4Bit => self.pixels.dimensions.0 * 4, // 4 pixels per 16-bit word
            PixelMode::Clut8Bit => self.pixels.dimensions.0 * 2, // 2 pixels per 16-bit word
            _ => self.pixels.dimensions.0,
        }
    }

    /// Get the height in pixels
    pub fn height(&self) -> u16 {
        self.pixels.dimensions.1
    }

    /// Get the total size of the TIM file in bytes
    pub fn data_size(&self) -> usize {
        let mut size = 8; // Header (magic + flags)

        // Add CLUT block size if present
        if let Some(clut) = &self.clut {
            size += 12; // CLUT header
            size += clut.data.len() * 2; // CLUT data (u16 entries)
        }

        // Add pixel block size
        size += 12; // Pixel header
        size += self.pixels.data.len(); // Pixel data

        size
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_pixel_mode_from_u32() {
        assert_eq!(PixelMode::from_u32(0).unwrap(), PixelMode::Clut4Bit);
        assert_eq!(PixelMode::from_u32(1).unwrap(), PixelMode::Clut8Bit);
        assert_eq!(PixelMode::from_u32(2).unwrap(), PixelMode::Direct16Bit);
        assert_eq!(PixelMode::from_u32(3).unwrap(), PixelMode::Direct24Bit);
        assert!(PixelMode::from_u32(7).is_err());
    }

    #[test]
    fn test_bits_per_pixel() {
        assert_eq!(PixelMode::Clut4Bit.bits_per_pixel(), 4);
        assert_eq!(PixelMode::Clut8Bit.bits_per_pixel(), 8);
        assert_eq!(PixelMode::Direct16Bit.bits_per_pixel(), 16);
        assert_eq!(PixelMode::Direct24Bit.bits_per_pixel(), 24);
    }
}
