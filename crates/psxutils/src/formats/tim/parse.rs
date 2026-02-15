//! TIM format parsing logic

use super::types::*;
use crate::{PsxError, Result};

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

        let pixel_mode = header.pixel_mode()?;
        let has_clut = header.has_clut();

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

        // Validate flags field more strictly (from jPSXdec TimValidator line 84)
        // Only bits 0-3 are used: bits 0-2 for pixel mode, bit 3 for hasClut
        if !header.validate_reserved_bits() {
            return Err(PsxError::InvalidFormat(format!(
                "Invalid TIM flags: 0x{:08X} (reserved bits set)",
                header.flags
            )));
        }

        let pixel_mode = header.pixel_mode()?;
        let has_clut = header.has_clut();

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

            // Use jPSXdec's CLUT size limit (TimValidator line 298)
            const CLUT_MAX_BYTE_SIZE: usize =
                (MAX_TIM_WORD_WIDTH as usize * 2 * MAX_TIM_HEIGHT as usize) + 12;
            if clut_data_size > CLUT_MAX_BYTE_SIZE || (clut_header.size % 2 != 0) {
                return Err(PsxError::InvalidFormat(format!(
                    "TIM CLUT data size invalid: {} bytes (max {} bytes)",
                    clut_data_size, CLUT_MAX_BYTE_SIZE
                )));
            }

            // Validate CLUT dimensions (TimValidator lines 319, 335, 350, 367)
            if clut_header.width == 0 || clut_header.width > MAX_TIM_WORD_WIDTH {
                return Err(PsxError::InvalidFormat(format!(
                    "TIM CLUT width out of range: {} (must be 1-{})",
                    clut_header.width, MAX_TIM_WORD_WIDTH
                )));
            }
            if clut_header.height == 0 || clut_header.height > MAX_TIM_HEIGHT {
                return Err(PsxError::InvalidFormat(format!(
                    "TIM CLUT height out of range: {} (must be 1-{})",
                    clut_header.height, MAX_TIM_HEIGHT
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

        // Validate pixel dimensions (TimValidator lines 157, 172, 187, 226)
        if pixel_header.width == 0 || pixel_header.width > MAX_TIM_WORD_WIDTH {
            return Err(PsxError::InvalidFormat(format!(
                "TIM pixel width out of range: {} (must be 1-{})",
                pixel_header.width, MAX_TIM_WORD_WIDTH
            )));
        }
        if pixel_header.height == 0 || pixel_header.height > MAX_TIM_HEIGHT {
            return Err(PsxError::InvalidFormat(format!(
                "TIM pixel height out of range: {} (must be 1-{})",
                pixel_header.height, MAX_TIM_HEIGHT
            )));
        }

        // Use jPSXdec's max size calculation (TimValidator line 137)
        const MAX_POSSIBLE_TIM_DATA_SIZE: usize =
            (MAX_TIM_WORD_WIDTH as usize * 2 * MAX_TIM_HEIGHT as usize) + 12;
        if pixel_data_size > MAX_POSSIBLE_TIM_DATA_SIZE {
            return Err(PsxError::InvalidFormat(format!(
                "TIM pixel data size too large: {} bytes (max {} bytes)",
                pixel_data_size, MAX_POSSIBLE_TIM_DATA_SIZE
            )));
        }

        if data.len() < offset + pixel_data_size {
            return Err(PsxError::InvalidFormat(
                "TIM file truncated (pixel data)".to_string(),
            ));
        }

        // Check consistency (jPSXdec TimValidator lines 242-252)
        // Allow +2 bytes tolerance for weird TIMs
        let expected_pixel_size =
            (pixel_header.width as usize * 2 * pixel_header.height as usize) + 12;
        if pixel_header.size < expected_pixel_size as u32
            || pixel_header.size > (expected_pixel_size + 2) as u32
        {
            return Err(PsxError::InvalidFormat(format!(
                "TIM pixel size inconsistent: header says {} bytes, but dimensions require {} bytes",
                pixel_header.size, expected_pixel_size
            )));
        }

        total_size += 12 + pixel_data_size;

        // Calculate actual pixel dimensions using jPSXdec's formula (Tim.java line 241)
        let width = match pixel_mode {
            PixelMode::Clut4Bit => pixel_header.width * 4, // iWordWidth * 2 * 2
            PixelMode::Clut8Bit => pixel_header.width * 2, // iWordWidth * 2
            PixelMode::Direct16Bit => pixel_header.width,  // iWordWidth
            PixelMode::Direct24Bit => pixel_header.width * 2 / 3, // iWordWidth * 2 / 3
            PixelMode::Mixed => pixel_header.width,
        };

        Ok((width, pixel_header.height, total_size))
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
