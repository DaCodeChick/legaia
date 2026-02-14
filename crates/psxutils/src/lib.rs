//! # PSX Utilities
//!
//! A collection of parsers and utilities for PlayStation 1 file formats.
//!
//! ## Supported Formats
//!
//! - **CD-ROM**: ISO 9660 with CD-XA extensions
//! - **TIM**: Texture Image format (4/8/16/24-bit)
//! - **VAB**: Sound bank format
//! - **VAG**: Sound sample format (ADPCM)
//! - **TMD**: 3D model format (planned)
//! - **STR**: Movie/video format (planned)
//!
//! ## Example
//!
//! ```no_run
//! use psxutils::cdrom::CdRom;
//!
//! // Open a PSX disc image
//! let disc = CdRom::open("game.bin")?;
//!
//! // List files
//! for entry in disc.read_dir("/")? {
//!     println!("{}", entry.name);
//! }
//!
//! // Read a file
//! let data = disc.read_file("/SLUS_123.45")?;
//! # Ok::<(), Box<dyn std::error::Error>>(())
//! ```

pub mod cdrom;
pub mod formats;

// Re-export commonly used types
pub use cdrom::CdRom;
pub use formats::{tim::Tim, vab::Vab, vag::Vag};

/// Common error type for psxutils
#[derive(Debug, thiserror::Error)]
pub enum PsxError {
    #[error("I/O error: {0}")]
    Io(#[from] std::io::Error),

    #[error("Invalid format: {0}")]
    InvalidFormat(String),

    #[error("Parse error: {0}")]
    ParseError(String),

    #[error("File not found: {0}")]
    FileNotFound(String),

    #[error("Unsupported format version: {0}")]
    UnsupportedVersion(u32),
}

/// Common result type for psxutils
pub type Result<T> = std::result::Result<T, PsxError>;
