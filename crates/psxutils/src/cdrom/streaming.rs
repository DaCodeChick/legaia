//! CD-ROM streaming system types
//!
//! Types and constants for CD-ROM streaming operations used in PSX games.

/// CD-ROM system state
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
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CdromAsyncMode {
    /// Synchronous/blocking mode
    Sync = 0,
    /// Asynchronous mode
    Async = 1,
}

/// CD-ROM streaming parameters
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
/// Represents a position on the CD in minutes:seconds:sectors format (MSF)
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
/// Standard timeout values for CD-ROM operations:
/// - Wait counter: 120 frames (~2 seconds at 60fps)
/// - Timeout counter: 180 frames (~3 seconds)
pub mod timeouts {
    /// Wait counter timeout (120 frames = ~2 seconds at 60fps)
    pub const WAIT_COUNTER: u32 = 0x78;

    /// Timeout counter (180 frames = ~3 seconds at 60fps)
    pub const TIMEOUT_COUNTER: u32 = 0xb4;
}

/// CD-ROM sync status codes
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
