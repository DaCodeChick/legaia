//! CD-ROM ISO 9660 / CD-XA parser and streaming system
//!
//! This module provides functionality to read PlayStation disc images
//! in ISO 9660 format with CD-XA extensions, plus CD-ROM streaming types.
//!
//! ## Example
//!
//! ```no_run
//! use psxutils::cdrom::CdRom;
//!
//! let disc = CdRom::open("game.bin")?;
//! let files = disc.read_dir("/")?;
//! for file in files {
//!     println!("{}: {} bytes", file.name, file.size);
//! }
//! # Ok::<(), Box<dyn std::error::Error>>(())
//! ```

pub mod streaming;

pub use streaming::{
    timeouts, CdromAsyncMode, CdromPosition, CdromState, CdromStreamParams, CdromSyncStatus,
};

use crate::{PsxError, Result};
use memmap2::Mmap;
use std::fs::File;
use std::path::Path;

/// CD-ROM sector size (Mode 2 Form 1)
pub const SECTOR_SIZE: usize = 2352;

/// CD-ROM data size per sector (Mode 2)
pub const DATA_SIZE: usize = 2048;

/// Primary Volume Descriptor is at sector 16
const PVD_SECTOR: u32 = 16;

/// Volume descriptor type codes
const VD_PRIMARY: u8 = 1;

/// ISO 9660 directory record flags
const FLAG_DIRECTORY: u8 = 0x02;

/// PlayStation CD-ROM disc image
pub struct CdRom {
    _file: File,
    mmap: Mmap,
    root_dir_lba: u32,
    root_dir_size: u32,
}

/// Directory entry in ISO 9660 filesystem
#[derive(Debug, Clone)]
pub struct DirectoryEntry {
    /// File name
    pub name: String,
    /// File size in bytes
    pub size: u32,
    /// Starting sector (LBA)
    pub lba: u32,
    /// Is this a directory?
    pub is_dir: bool,
}

impl CdRom {
    /// Open a PlayStation disc image
    ///
    /// Supports BIN files (raw CD image format)
    pub fn open(path: impl AsRef<Path>) -> Result<Self> {
        let file = File::open(path)?;
        let mmap = unsafe { Mmap::map(&file)? };

        let mut cdrom = Self {
            _file: file,
            mmap,
            root_dir_lba: 0,
            root_dir_size: 0,
        };

        // Parse the Primary Volume Descriptor to find the root directory
        cdrom.parse_pvd()?;

        Ok(cdrom)
    }

    /// Parse the Primary Volume Descriptor
    fn parse_pvd(&mut self) -> Result<()> {
        let pvd = self.read_sector(PVD_SECTOR)?.to_vec();

        // Check for CD001 identifier at offset 1
        if &pvd[1..6] != b"CD001" {
            return Err(PsxError::ParseError(
                "Invalid ISO 9660 signature".to_string(),
            ));
        }

        // Check volume descriptor type (should be 1 for primary)
        if pvd[0] != VD_PRIMARY {
            return Err(PsxError::ParseError(format!(
                "Expected Primary Volume Descriptor, got type {}",
                pvd[0]
            )));
        }

        // Root directory record starts at offset 156 in the PVD
        let root_record = &pvd[156..];

        // Parse root directory LBA (LSB order at offset 2, 4 bytes)
        self.root_dir_lba = u32::from_le_bytes([
            root_record[2],
            root_record[3],
            root_record[4],
            root_record[5],
        ]);

        // Parse root directory size (LSB order at offset 10, 4 bytes)
        self.root_dir_size = u32::from_le_bytes([
            root_record[10],
            root_record[11],
            root_record[12],
            root_record[13],
        ]);

        Ok(())
    }

    /// Read a sector at the given LBA (Logical Block Address)
    pub fn read_sector(&self, lba: u32) -> Result<&[u8]> {
        let offset = lba as usize * SECTOR_SIZE;

        if offset + SECTOR_SIZE > self.mmap.len() {
            return Err(PsxError::ParseError(format!(
                "Sector {} out of bounds",
                lba
            )));
        }

        // For Mode 2 Form 1, data starts at offset 24 in the sector
        let data_offset = offset + 24;
        let data_end = data_offset + DATA_SIZE;

        if data_end > self.mmap.len() {
            // Fallback: return what we can
            Ok(&self.mmap[offset..offset + SECTOR_SIZE.min(self.mmap.len() - offset)])
        } else {
            Ok(&self.mmap[data_offset..data_end])
        }
    }

    /// Read the root directory
    ///
    /// Reads and parses ISO 9660 directory entries
    pub fn read_dir(&self, path: &str) -> Result<Vec<DirectoryEntry>> {
        // For now, only support root directory
        if path != "/" {
            return Err(PsxError::FileNotFound(format!(
                "Only root directory '/' is currently supported, got '{}'",
                path
            )));
        }

        let mut entries = Vec::new();
        let mut offset = 0;

        // Read directory data
        let dir_data = self.read_data(self.root_dir_lba, self.root_dir_size)?;

        while offset < dir_data.len() {
            // First byte is the record length
            let record_len = dir_data[offset] as usize;

            // Record length of 0 means end of directory or padding to next sector
            if record_len == 0 {
                // Move to next sector boundary
                offset = ((offset / DATA_SIZE) + 1) * DATA_SIZE;
                if offset >= dir_data.len() {
                    break;
                }
                continue;
            }

            // Parse directory record
            if offset + record_len > dir_data.len() {
                break; // Incomplete record
            }

            let record = &dir_data[offset..offset + record_len];

            // Parse entry (skip '.' and '..' entries)
            if let Some(entry) = self.parse_directory_record(record)? {
                entries.push(entry);
            }

            offset += record_len;
        }

        Ok(entries)
    }

    /// Parse a single directory record
    fn parse_directory_record(&self, record: &[u8]) -> Result<Option<DirectoryEntry>> {
        if record.len() < 33 {
            return Ok(None); // Invalid record
        }

        // LBA at offset 2 (LSB), 4 bytes
        let lba = u32::from_le_bytes([record[2], record[3], record[4], record[5]]);

        // File size at offset 10 (LSB), 4 bytes
        let size = u32::from_le_bytes([record[10], record[11], record[12], record[13]]);

        // Flags at offset 25
        let flags = record[25];
        let is_dir = (flags & FLAG_DIRECTORY) != 0;

        // File identifier length at offset 32
        let name_len = record[32] as usize;

        // File identifier starts at offset 33
        if record.len() < 33 + name_len {
            return Ok(None); // Invalid record
        }

        let name_bytes = &record[33..33 + name_len];

        // Convert to string, removing version suffix (;1)
        let name = String::from_utf8_lossy(name_bytes)
            .split(';')
            .next()
            .unwrap_or("")
            .to_string();

        // Skip '.' and '..' entries
        if name == "\0" || name == "\u{1}" || name.is_empty() {
            return Ok(None);
        }

        Ok(Some(DirectoryEntry {
            name,
            size,
            lba,
            is_dir,
        }))
    }

    /// Read data from consecutive sectors
    fn read_data(&self, start_lba: u32, size: u32) -> Result<Vec<u8>> {
        let sector_count = ((size as usize + DATA_SIZE - 1) / DATA_SIZE) as u32;
        let mut data = Vec::with_capacity(size as usize);

        for i in 0..sector_count {
            let sector = self.read_sector(start_lba + i)?;
            let to_copy = std::cmp::min(DATA_SIZE, size as usize - data.len());
            data.extend_from_slice(&sector[..to_copy]);
        }

        Ok(data)
    }

    /// Read a file by path
    ///
    /// Reads a file from the ISO 9660 filesystem
    pub fn read_file(&self, path: &str) -> Result<Vec<u8>> {
        // Parse the path - for now only support root-level files
        let filename = if let Some(stripped) = path.strip_prefix('/') {
            stripped
        } else {
            path
        };

        if filename.contains('/') {
            return Err(PsxError::FileNotFound(
                "Subdirectories not yet supported".to_string(),
            ));
        }

        // Read root directory
        let entries = self.read_dir("/")?;

        // Find the file
        let entry = entries
            .iter()
            .find(|e| e.name.eq_ignore_ascii_case(filename))
            .ok_or_else(|| PsxError::FileNotFound(format!("File '{}' not found", filename)))?;

        if entry.is_dir {
            return Err(PsxError::ParseError(format!(
                "'{}' is a directory, not a file",
                filename
            )));
        }

        // Read the file data
        self.read_data(entry.lba, entry.size)
    }

    /// Get the total number of sectors
    pub fn sector_count(&self) -> usize {
        self.mmap.len() / SECTOR_SIZE
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_sector_size() {
        assert_eq!(SECTOR_SIZE, 2352);
        assert_eq!(DATA_SIZE, 2048);
    }
}
