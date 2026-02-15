//! TIM color conversion functions

use super::types::*;
use crate::{PsxError, Result};

impl Tim {
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
}
