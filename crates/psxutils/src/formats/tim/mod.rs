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

mod convert;
mod parse;
mod types;

// Re-export public API
pub use types::{ClutData, PixelData, PixelMode, Tim, TIM_MAGIC};

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
