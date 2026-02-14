//! CD-ROM streaming system types
//!
//! Based on decompilation of CD-ROM functions:
//! - prepare_cdrom_data_load (0x8003ebe4)
//! - wait_for_cdrom_read (0x8003de7c)
//! - prepare_cdrom_stream (0x8003e800)
//! - start_cdrom_async_read (0x8003f128)
//! - poll_cdrom_sync_status (0x8003f2b8)

/// CD-ROM system state
///
/// Based on decompilation globals:
/// - g_cdrom_cached_state (0x8007bc3c)
/// - g_cdrom_load_flag (0x8007bc4c)
/// - g_cdrom_active_flag (0x8007ba70)
/// - g_cdrom_busy_flag (0x8007bc40)
/// - g_cdrom_status_code (0x8007bc98)
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CdromState {
    /// CD-ROM is idle
    Idle,
    /// CD-ROM is preparing to read
    Preparing,
    /// CD-ROM is actively reading
    Reading,
    /// CD-ROM read is complete
    Complete,
    /// CD-ROM operation failed
    Error,
}

/// CD-ROM async mode flags
///
/// Based on g_cdrom_async_mode (0x8007bca0)
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CdromAsyncMode {
    /// Synchronous/blocking mode
    Sync = 0,
    /// Asynchronous mode
    Async = 1,
}

/// CD-ROM streaming parameters
///
/// Based on prepare_cdrom_stream (0x8003e800)
#[derive(Debug, Clone, Copy)]
pub struct CdromStreamParams {
    /// Sector count to read
    pub sector_count: u32,

    /// Mode flags:
    /// - Bit 0 (0x01): Start async read
    /// - Bit 1 (0x02): Wait for completion
    pub mode_flags: u32,
}

impl CdromStreamParams {
    /// Create new stream parameters
    pub fn new(sector_count: u32, start_async: bool, wait_complete: bool) -> Self {
        let mut flags = 0u32;
        if start_async {
            flags |= 0x01;
        }
        if wait_complete {
            flags |= 0x02;
        }

        Self {
            sector_count,
            mode_flags: flags,
        }
    }

    /// Check if async read should start
    pub fn should_start_async(&self) -> bool {
        (self.mode_flags & 0x01) != 0
    }

    /// Check if should wait for completion
    pub fn should_wait_complete(&self) -> bool {
        (self.mode_flags & 0x02) != 0
    }
}

/// CD-ROM sector position
///
/// Represents a position on the CD in minutes:seconds:sectors format
/// Based on CdlLOC structure used in g_cdrom_current_position (0x8007bc5c)
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct CdromPosition {
    /// Minutes (0-99)
    pub minute: u8,
    /// Seconds (0-59)
    pub second: u8,
    /// Sectors (0-74)
    pub sector: u8,
}

impl CdromPosition {
    /// Create a new CD-ROM position
    pub fn new(minute: u8, second: u8, sector: u8) -> Self {
        Self {
            minute,
            second,
            sector,
        }
    }

    /// Convert position to absolute sector number
    ///
    /// Based on CdPosToInt function used in start_cdrom_async_read
    pub fn to_sector_number(&self) -> u32 {
        let minutes = self.minute as u32;
        let seconds = self.second as u32;
        let sectors = self.sector as u32;

        // PSX CD: 75 sectors per second, 60 seconds per minute
        (minutes * 60 * 75) + (seconds * 75) + sectors
    }

    /// Create position from absolute sector number
    pub fn from_sector_number(sector: u32) -> Self {
        let minute = (sector / (60 * 75)) as u8;
        let second = ((sector / 75) % 60) as u8;
        let sector = (sector % 75) as u8;

        Self {
            minute,
            second,
            sector,
        }
    }
}

/// CD-ROM operation timeouts
///
/// Based on decompilation findings:
/// - g_cdrom_wait_counter: 0x78 = 120 frames (~2 seconds at 60fps)
/// - g_cdrom_timeout_counter: 0xb4 = 180 frames (~3 seconds)
pub mod timeouts {
    /// Wait counter timeout (120 frames = ~2 seconds at 60fps)
    pub const WAIT_COUNTER: u32 = 0x78;

    /// Timeout counter (180 frames = ~3 seconds at 60fps)
    pub const TIMEOUT_COUNTER: u32 = 0xb4;
}

/// CD-ROM sync status codes
///
/// Based on poll_cdrom_sync_status (0x8003f2b8) and start_cdrom_async_read
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u32)]
pub enum CdromSyncStatus {
    /// Operation in progress
    InProgress = 0,
    /// Operation complete
    Complete = 2,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_cdrom_position_conversion() {
        // Test position 00:02:00 (2 seconds = 150 sectors)
        let pos = CdromPosition::new(0, 2, 0);
        assert_eq!(pos.to_sector_number(), 150);

        // Test round-trip conversion
        let sector_num = 12345u32;
        let pos = CdromPosition::from_sector_number(sector_num);
        assert_eq!(pos.to_sector_number(), sector_num);
    }

    #[test]
    fn test_stream_params_flags() {
        let params = CdromStreamParams::new(100, true, false);
        assert!(params.should_start_async());
        assert!(!params.should_wait_complete());
        assert_eq!(params.mode_flags, 0x01);

        let params = CdromStreamParams::new(100, true, true);
        assert!(params.should_start_async());
        assert!(params.should_wait_complete());
        assert_eq!(params.mode_flags, 0x03);
    }
}
