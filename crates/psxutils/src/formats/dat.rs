//! DAT archive format parser for Legend of Legaia
//!
//! PROT.DAT and DMY.DAT are archive files containing game assets.
//!
//! ## Format
//!
//! ```text
//! [File Table]
//! Entry 0: [sector_offset: u32, sector_count: u32]  // 8 bytes
//! Entry 1: [sector_offset: u32, sector_count: u32]
//! ...
//! Entry N: [sector_offset: u32, sector_count: u32]
//!
//! [File Data]
//! File data starts at first non-zero sector offset (typically sector 3)
//! ```
//!
//! - Sector size: 2048 bytes
//! - Table size: Determined by first entry with sector_offset > table size
//! - Entry 0 typically has sector_offset=0, representing the table itself
//!
//! ## Example
//!
//! ```no_run
//! use psxutils::formats::DatArchive;
//!
//! let data = std::fs::read("PROT.DAT")?;
//! let archive = DatArchive::parse(&data)?;
//!
//! println!("Archive has {} files", archive.entry_count());
//!
//! // Extract file by index
//! let file_data = archive.extract_file(1)?;
//! # Ok::<(), Box<dyn std::error::Error>>(())
//! ```

use crate::{PsxError, Result};

/// CD-ROM sector size used in DAT archives
pub const SECTOR_SIZE: usize = 2048;

/// DAT archive file table entry
#[derive(Debug, Clone, Copy)]
pub struct DatEntry {
    /// Sector offset (multiply by 2048 for byte offset)
    pub sector_offset: u32,
    /// Number of sectors
    pub sector_count: u32,
}

impl DatEntry {
    /// Get byte offset in archive
    pub fn byte_offset(&self) -> usize {
        self.sector_offset as usize * SECTOR_SIZE
    }

    /// Get file size in bytes
    pub fn byte_size(&self) -> usize {
        self.sector_count as usize * SECTOR_SIZE
    }

    /// Get byte range (offset, size)
    pub fn byte_range(&self) -> (usize, usize) {
        (self.byte_offset(), self.byte_size())
    }
}

/// DAT archive parser
pub struct DatArchive<'a> {
    data: &'a [u8],
    entries: Vec<DatEntry>,
    table_size: usize,
}

impl<'a> DatArchive<'a> {
    /// Parse a DAT archive
    pub fn parse(data: &'a [u8]) -> Result<Self> {
        if data.len() < 16 {
            return Err(PsxError::ParseError(
                "File too small to be a DAT archive".to_string(),
            ));
        }

        // Parse file table
        let mut entries = Vec::new();
        let mut offset = 0;
        let mut table_size = 0;

        loop {
            if offset + 8 > data.len() {
                break;
            }

            let sector_offset = u32::from_le_bytes([
                data[offset],
                data[offset + 1],
                data[offset + 2],
                data[offset + 3],
            ]);
            let sector_count = u32::from_le_bytes([
                data[offset + 4],
                data[offset + 5],
                data[offset + 6],
                data[offset + 7],
            ]);

            let entry = DatEntry {
                sector_offset,
                sector_count,
            };

            // Check if this entry's byte end is beyond file size (invalid)
            let byte_end = entry.byte_offset() + entry.byte_size();
            if byte_end > data.len() * 2 {
                // Allow some overflow for last entry, but stop if too large
                table_size = offset;
                break;
            }

            // Stop if we've hit padding (all zeros in both fields after first entry)
            if entries.len() > 0 && sector_offset == 0 && sector_count == 0 {
                table_size = offset;
                break;
            }

            entries.push(entry);
            offset += 8;

            // Safety limit
            if entries.len() > 10000 {
                return Err(PsxError::ParseError(
                    "Too many entries in archive (> 10000)".to_string(),
                ));
            }
        }

        if entries.is_empty() {
            return Err(PsxError::ParseError(
                "No entries found in archive".to_string(),
            ));
        }

        // If table_size wasn't set, use current offset
        if table_size == 0 {
            table_size = offset;
        }

        Ok(Self {
            data,
            entries,
            table_size,
        })
    }

    /// Get number of files in archive
    pub fn entry_count(&self) -> usize {
        self.entries.len()
    }

    /// Get entry by index
    pub fn get_entry(&self, index: usize) -> Option<&DatEntry> {
        self.entries.get(index)
    }

    /// Get all entries
    pub fn entries(&self) -> &[DatEntry] {
        &self.entries
    }

    /// Get file table size in bytes
    pub fn table_size(&self) -> usize {
        self.table_size
    }

    /// Extract file data by index
    pub fn extract_file(&self, index: usize) -> Result<&'a [u8]> {
        let entry = self
            .entries
            .get(index)
            .ok_or_else(|| PsxError::ParseError(format!("File index {} out of range", index)))?;

        let offset = entry.byte_offset();
        let size = entry.byte_size();
        let end = offset + size;

        if end > self.data.len() {
            return Err(PsxError::ParseError(format!(
                "File {} extends beyond archive (offset: 0x{:x}, size: 0x{:x}, archive size: 0x{:x})",
                index,
                offset,
                size,
                self.data.len()
            )));
        }

        Ok(&self.data[offset..end])
    }

    /// Extract file data by index (owned copy)
    pub fn extract_file_owned(&self, index: usize) -> Result<Vec<u8>> {
        self.extract_file(index).map(|slice| slice.to_vec())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_sector_size() {
        assert_eq!(SECTOR_SIZE, 2048);
    }

    #[test]
    fn test_dat_entry_calculations() {
        let entry = DatEntry {
            sector_offset: 3,
            sector_count: 5,
        };

        assert_eq!(entry.byte_offset(), 3 * 2048);
        assert_eq!(entry.byte_size(), 5 * 2048);
        assert_eq!(entry.byte_range(), (3 * 2048, 5 * 2048));
    }
}
