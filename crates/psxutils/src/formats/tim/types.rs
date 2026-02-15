//! TIM format type definitions

use crate::{PsxError, Result};
use bitflags::bitflags;
use bytemuck::{Pod, Zeroable};

/// TIM magic number (0x10)
pub const TIM_MAGIC: u32 = 0x10;

/// Maximum reasonable TIM dimensions (from jPSXdec)
pub(super) const MAX_TIM_WORD_WIDTH: u16 = 16384;
pub(super) const MAX_TIM_HEIGHT: u16 = 8192;

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
    pub(super) fn from_u32(value: u32) -> Result<Self> {
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
    pub const fn bits_per_pixel(&self) -> u8 {
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
pub(super) struct TimHeader {
    pub magic: u32,
    pub flags: u32,
}

bitflags! {
    /// TIM header flag bits
    ///
    /// The TIM flags field (u32) contains:
    /// - Bits 0-2: Pixel mode (0-4)
    /// - Bit 3: Has CLUT flag
    /// - Bits 4-15: Reserved (must be 0)
    #[derive(Debug, Clone, Copy, PartialEq, Eq)]
    pub(super) struct TimHeaderFlags: u32 {
        /// Has Color Lookup Table (CLUT)
        const HAS_CLUT = 0x08;
        /// Mask for pixel mode bits (0-2)
        const PIXEL_MODE_MASK = 0x07;
        /// Mask for valid bits (mode + CLUT flag)
        const VALID_BITS = 0x0F;
    }
}

impl TimHeader {
    /// Extract pixel mode from flags
    pub(super) fn pixel_mode(&self) -> Result<PixelMode> {
        PixelMode::from_u32(self.flags & TimHeaderFlags::PIXEL_MODE_MASK.bits())
    }

    /// Check if TIM has CLUT
    pub(super) const fn has_clut(&self) -> bool {
        (self.flags & TimHeaderFlags::HAS_CLUT.bits()) != 0
    }

    /// Validate reserved bits are not set
    pub(super) const fn validate_reserved_bits(&self) -> bool {
        (self.flags & !TimHeaderFlags::VALID_BITS.bits()) == 0
    }
}

/// Color Lookup Table (CLUT) header
#[repr(C)]
#[derive(Debug, Clone, Copy, Pod, Zeroable)]
pub(super) struct ClutHeader {
    pub size: u32,   // Total size of CLUT block including header
    pub vram_x: u16, // X position in VRAM
    pub vram_y: u16, // Y position in VRAM
    pub width: u16,  // Width in colors
    pub height: u16, // Height in palette rows
}

/// Pixel data header
#[repr(C)]
#[derive(Debug, Clone, Copy, Pod, Zeroable)]
pub(super) struct PixelHeader {
    pub size: u32,   // Total size of pixel block including header
    pub vram_x: u16, // X position in VRAM
    pub vram_y: u16, // Y position in VRAM
    pub width: u16,  // Width in pixels (or half-pixels for 4-bit)
    pub height: u16, // Height in pixels
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
