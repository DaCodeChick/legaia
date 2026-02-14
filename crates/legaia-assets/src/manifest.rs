//! Asset manifest management

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::{Path, PathBuf};

/// Manifest describing all assets extracted from the game
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AssetManifest {
    /// Version of the manifest format
    pub version: String,

    /// Source disc image information
    pub source: SourceInfo,

    /// All assets indexed by ID
    pub assets: HashMap<String, AssetEntry>,
}

/// Information about the source disc
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SourceInfo {
    /// Game title
    pub game: String,

    /// Region (NTSC-U, PAL, NTSC-J)
    pub region: String,

    /// Serial number (e.g., SCUS-94254)
    pub serial: String,

    /// Disc image path (relative or absolute)
    pub path: PathBuf,
}

/// Individual asset entry in the manifest
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AssetEntry {
    /// Asset type (texture, audio, model, etc.)
    pub asset_type: AssetType,

    /// Original address in PSX memory/disc
    pub source_address: u32,

    /// Original format
    pub source_format: String,

    /// Converted file path (relative to manifest)
    pub file_path: PathBuf,

    /// Converted format
    pub target_format: String,

    /// Additional metadata
    #[serde(default)]
    pub metadata: HashMap<String, String>,
}

/// Types of assets
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum AssetType {
    Texture,
    Audio,
    Model,
    Animation,
    Text,
    Script,
    Map,
    Other,
}

impl AssetManifest {
    /// Create a new empty manifest
    pub fn new(source: SourceInfo) -> Self {
        Self {
            version: "1.0.0".to_string(),
            source,
            assets: HashMap::new(),
        }
    }

    /// Add an asset to the manifest
    pub fn add_asset(&mut self, id: impl Into<String>, entry: AssetEntry) {
        self.assets.insert(id.into(), entry);
    }

    /// Load manifest from JSON file
    pub fn from_json(path: impl AsRef<Path>) -> crate::Result<Self> {
        let contents = std::fs::read_to_string(path)?;
        serde_json::from_str(&contents).map_err(|e| crate::AssetError::ManifestError(e.to_string()))
    }

    /// Save manifest to JSON file
    pub fn to_json(&self, path: impl AsRef<Path>) -> crate::Result<()> {
        let contents = serde_json::to_string_pretty(self)
            .map_err(|e| crate::AssetError::ManifestError(e.to_string()))?;
        std::fs::write(path, contents)?;
        Ok(())
    }

    /// Load manifest from TOML file
    pub fn from_toml(path: impl AsRef<Path>) -> crate::Result<Self> {
        let contents = std::fs::read_to_string(path)?;
        toml::from_str(&contents).map_err(|e| crate::AssetError::ManifestError(e.to_string()))
    }

    /// Save manifest to TOML file
    pub fn to_toml(&self, path: impl AsRef<Path>) -> crate::Result<()> {
        let contents = toml::to_string_pretty(self)
            .map_err(|e| crate::AssetError::ManifestError(e.to_string()))?;
        std::fs::write(path, contents)?;
        Ok(())
    }
}
