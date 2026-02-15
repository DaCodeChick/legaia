//! Asset extraction and management for Legend of Legaia
//!
//! This crate provides tools for:
//! - Extracting assets from PSX disc images
//! - Converting PSX formats to modern equivalents
//! - Managing asset manifests and metadata
//! - Organizing assets for the game engine

pub mod converter;
pub mod extraction;
pub mod extractor;
pub mod formats;
pub mod manifest;

pub use extraction::{AssetExtractionService, ExtractionProgress, ExtractionStats};
pub use extractor::AssetExtractor;
pub use manifest::{AssetEntry, AssetManifest};

use thiserror::Error;

#[derive(Debug, Error)]
pub enum AssetError {
    #[error("Failed to read disc image: {0}")]
    DiscReadError(#[from] std::io::Error),

    #[error("Invalid asset format: {0}")]
    InvalidFormat(String),

    #[error("Asset not found at address {0:#X}")]
    AssetNotFound(u32),

    #[error("Conversion failed: {0}")]
    ConversionError(String),

    #[error("Manifest error: {0}")]
    ManifestError(String),
}

pub type Result<T> = std::result::Result<T, AssetError>;
