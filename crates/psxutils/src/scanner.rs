//! Asset scanner for PSX disc containers
//!
//! Scans through binary data containers looking for embedded assets by their
//! magic numbers and signatures. Similar to forensic tools like binwalk or foremost.

use crate::formats::tmd::TMD_MAGIC;
use crate::formats::{Tim, Tmd};

/// Magic number for TIM texture format (0x00000010)
const TIM_MAGIC: u32 = 0x00000010;

/// Magic number for VAG audio format ("VAGp")
const VAG_MAGIC: u32 = 0x70474156; // "VAGp" in little-endian

/// Discovered asset in a container file
#[derive(Debug, Clone)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct DiscoveredAsset {
    /// Offset in the container file where asset starts
    pub offset: usize,
    /// Size of the asset in bytes
    pub size: usize,
    /// Type of asset discovered
    pub asset_type: AssetType,
}

/// Type of discovered asset
#[derive(Debug, Clone, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub enum AssetType {
    /// TIM texture with dimensions
    Tim { width: u16, height: u16 },
    /// TMD 3D model with object count
    Tmd { object_count: u32 },
    /// VAG audio sample
    Vag,
}

/// Asset scanner for binary data
pub struct AssetScanner<'a> {
    data: &'a [u8],
    min_size: usize,
}

impl<'a> AssetScanner<'a> {
    /// Create a new asset scanner for the given data
    ///
    /// # Arguments
    /// * `data` - Binary data to scan
    /// * `min_size` - Minimum asset size to consider (default: 64 bytes)
    pub fn new(data: &'a [u8]) -> Self {
        Self { data, min_size: 64 }
    }

    /// Set minimum asset size filter
    pub fn with_min_size(mut self, min_size: usize) -> Self {
        self.min_size = min_size;
        self
    }

    /// Scan for all embedded assets
    ///
    /// Returns a vector of discovered assets sorted by offset.
    pub fn scan(&self) -> Vec<DiscoveredAsset> {
        let mut assets = Vec::new();

        // Scan for TIM textures
        assets.extend(self.scan_tim());

        // Scan for TMD models - DISABLED: causes OOM
        // assets.extend(self.scan_tmd());

        // Scan for VAG audio - DISABLED: causes OOM
        // assets.extend(self.scan_vag());

        // Sort by offset
        assets.sort_by_key(|a| a.offset);

        assets
    }

    /// Scan for TIM textures
    fn scan_tim(&self) -> Vec<DiscoveredAsset> {
        let mut assets = Vec::new();
        let mut offset = 0;

        while offset + 12 <= self.data.len() {
            // Check for TIM magic at this offset
            if let Some(magic_bytes) = self.data.get(offset..offset + 4) {
                let magic = u32::from_le_bytes([
                    magic_bytes[0],
                    magic_bytes[1],
                    magic_bytes[2],
                    magic_bytes[3],
                ]);

                if magic == TIM_MAGIC {
                    // Try to validate TIM without allocating memory
                    match Tim::validate(&self.data[offset..]) {
                        Ok((width, height, size)) => {
                            if size >= self.min_size {
                                assets.push(DiscoveredAsset {
                                    offset,
                                    size,
                                    asset_type: AssetType::Tim { width, height },
                                });
                                // Skip past this TIM
                                offset += size;
                                continue;
                            }
                        }
                        Err(_e) => {
                            // TIM magic but invalid format - just skip
                        }
                    }
                }
            }

            offset += 1;
        }

        assets
    }

    /// Scan for TMD models
    fn scan_tmd(&self) -> Vec<DiscoveredAsset> {
        let mut assets = Vec::new();
        let mut offset = 0;

        while offset + 12 <= self.data.len() {
            // Check for TMD magic at this offset
            if let Some(magic_bytes) = self.data.get(offset..offset + 4) {
                let magic = u32::from_le_bytes([
                    magic_bytes[0],
                    magic_bytes[1],
                    magic_bytes[2],
                    magic_bytes[3],
                ]);

                if magic == TMD_MAGIC {
                    // Try to parse and validate TMD
                    if let Ok(tmd) = Tmd::parse(&self.data[offset..]) {
                        // Estimate TMD size based on object count
                        // This is approximate - TMD files don't have explicit size field
                        let object_count = tmd.object_count() as u32;
                        let estimated_size = 12 + (object_count as usize * 1024); // Rough estimate

                        if estimated_size >= self.min_size {
                            assets.push(DiscoveredAsset {
                                offset,
                                size: estimated_size,
                                asset_type: AssetType::Tmd { object_count },
                            });
                            // Skip past this TMD
                            offset += estimated_size;
                            continue;
                        }
                    }
                }
            }

            offset += 1;
        }

        assets
    }

    /// Scan for VAG audio samples
    fn scan_vag(&self) -> Vec<DiscoveredAsset> {
        let mut assets = Vec::new();
        let mut offset = 0;

        while offset + 48 <= self.data.len() {
            // Check for VAG magic at this offset
            if let Some(magic_bytes) = self.data.get(offset..offset + 4) {
                let magic = u32::from_le_bytes([
                    magic_bytes[0],
                    magic_bytes[1],
                    magic_bytes[2],
                    magic_bytes[3],
                ]);

                if magic == VAG_MAGIC {
                    // VAG header is 48 bytes, followed by data
                    // Size is stored at offset 12 (4 bytes, big-endian)
                    if let Some(size_bytes) = self.data.get(offset + 12..offset + 16) {
                        let size = u32::from_be_bytes([
                            size_bytes[0],
                            size_bytes[1],
                            size_bytes[2],
                            size_bytes[3],
                        ]) as usize
                            + 48; // Add header size

                        if size >= self.min_size && offset + size <= self.data.len() {
                            assets.push(DiscoveredAsset {
                                offset,
                                size,
                                asset_type: AssetType::Vag,
                            });
                            // Skip past this VAG
                            offset += size;
                            continue;
                        }
                    }
                }
            }

            offset += 1;
        }

        assets
    }

    /// Extract a discovered asset as bytes
    pub fn extract(&self, asset: &DiscoveredAsset) -> Option<&[u8]> {
        if asset.offset + asset.size <= self.data.len() {
            Some(&self.data[asset.offset..asset.offset + asset.size])
        } else {
            None
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_scanner_empty() {
        let data = vec![0; 1024];
        let scanner = AssetScanner::new(&data);
        let assets = scanner.scan();
        assert_eq!(assets.len(), 0);
    }

    #[test]
    fn test_scanner_tim_magic() {
        let mut data = vec![0; 1024];
        // Insert TIM magic at offset 100
        data[100..104].copy_from_slice(&TIM_MAGIC.to_le_bytes());
        // This won't be detected as valid TIM without proper header
        let scanner = AssetScanner::new(&data);
        let assets = scanner.scan();
        // Should not detect invalid TIM
        assert_eq!(assets.len(), 0);
    }
}
