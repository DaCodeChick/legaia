//! TIM color conversion functions

use super::types::*;
use crate::{PsxError, Result};

/// Convert RGB555 color to RGBA8 format
///
/// PSX RGB555 format: 0BBBBBGGGGGRRRRR (15-bit color + 1 STP bit)
///
/// STP (semi-transparent) bit behavior:
/// - Black (RGB=0,0,0) with STP=0: Fully transparent (alpha=0) - transparency key
/// - Black (RGB=0,0,0) with STP=1: Fully opaque (alpha=255) - solid black
/// - Color with STP=0: Fully opaque (alpha=255) - normal rendering
/// - Color with STP=1: Semi-transparent (alpha=254) - blending enabled
#[inline]
fn rgb555_to_rgba(color: u16) -> [u8; 4] {
    let r = ((color & 0x1F) << 3) as u8;
    let g = (((color >> 5) & 0x1F) << 3) as u8;
    let b = (((color >> 10) & 0x1F) << 3) as u8;
    
    let a = if r == 0 && g == 0 && b == 0 {
        // Black pixels: bit 15 determines transparency
        // STP=0 (bit clear) → transparent (used as transparency key)
        // STP=1 (bit set) → opaque black
        if color & 0x8000 == 0 { 0 } else { 255 }
    } else {
        // Non-black pixels: bit 15 determines blending mode
        // STP=0 (bit clear) → fully opaque (normal rendering)
        // STP=1 (bit set) → semi-transparent (PSX blending)
        if color & 0x8000 == 0 { 255 } else { 254 }
    };
    
    [r, g, b, a]
}
    } else {
        // Non-black pixels: bit 15 determines blending mode
        // STP=0 (bit clear) → fully opaque (normal rendering)
        // STP=1 (bit set) → semi-transparent (PSX blending)
        if color & 0x8000 == 0 {
            255
        } else {
            254
        }
    };

    [r, g, b, a]
}

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
            rgba.extend_from_slice(&rgb555_to_rgba(color));
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
                    rgba.extend_from_slice(&rgb555_to_rgba(color));
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
                rgba.extend_from_slice(&rgb555_to_rgba(color));
            }
        }

        Ok(rgba)
    }
}
