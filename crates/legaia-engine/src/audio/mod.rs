//! Audio system
//!
//! PSX-style audio with 24 sound channels:
//! - 24 simultaneous sound channels
//! - Each channel is 27 bytes
//! - 17 sound function handlers
//! - Sound sequences with active flags
//! - Reverb support via SPU

use bevy::prelude::*;

/// Maximum number of sound channels
pub const MAX_SOUND_CHANNELS: usize = 24;

/// Size of each sound channel structure in bytes
pub const CHANNEL_SIZE_BYTES: usize = 0x1b; // 27 bytes

/// Sound channel state
#[derive(Debug, Clone, Copy)]
pub struct SoundChannel {
    /// Channel priority (default: 0x18 = 24)
    pub priority: u8,

    /// Channel status flags
    pub status: u8,

    /// Volume level (default: 0xff = 255, max volume)
    pub volume: u8,

    /// Pan position (default: 0)
    pub pan: u8,

    /// Additional channel data (remaining 23 bytes)
    /// TODO: Decode remaining fields as we analyze more functions
    pub _reserved: [u8; 23],
}

impl Default for SoundChannel {
    fn default() -> Self {
        Self {
            priority: 0x18, // Default priority 24
            status: 0,
            volume: 0xff, // Max volume
            pan: 0,
            _reserved: [0; 23],
        }
    }
}

/// Audio system state
#[derive(Resource, Debug)]
pub struct AudioSystem {
    /// Array of 24 sound channels
    pub channels: [SoundChannel; MAX_SOUND_CHANNELS],

    /// Currently active channel index
    pub current_channel: usize,

    /// Sound sequence active flag
    pub sequence_active: bool,

    /// Sound sequence status
    pub sequence_status: u8,

    /// SPU reverb enabled
    pub reverb_enabled: bool,

    /// System initialized flag
    pub initialized: bool,
}

impl Default for AudioSystem {
    fn default() -> Self {
        Self {
            channels: [SoundChannel::default(); MAX_SOUND_CHANNELS],
            current_channel: 0,
            sequence_active: false,
            sequence_status: 0,
            reverb_enabled: false,
            initialized: false,
        }
    }
}

impl AudioSystem {
    /// Create a new audio system
    pub fn new() -> Self {
        Self::default()
    }

    /// Reset all channels to default state
    ///
    /// Based on reset_sound_channels (0x80064bd0)
    pub fn reset_channels(&mut self) {
        for channel in &mut self.channels {
            *channel = SoundChannel::default();
        }
        tracing::info!("Reset {} audio channels", MAX_SOUND_CHANNELS);
    }

    /// Enable SPU reverb effect
    ///
    /// Based on spu_enable_reverb (0x800655ac)
    pub fn enable_reverb(&mut self) {
        self.reverb_enabled = true;
        tracing::info!("SPU reverb enabled");
    }

    /// Disable SPU reverb effect
    pub fn disable_reverb(&mut self) {
        self.reverb_enabled = false;
        tracing::info!("SPU reverb disabled");
    }

    /// Get a channel by index
    pub fn get_channel(&self, index: usize) -> Option<&SoundChannel> {
        self.channels.get(index)
    }

    /// Get a mutable channel by index
    pub fn get_channel_mut(&mut self, index: usize) -> Option<&mut SoundChannel> {
        self.channels.get_mut(index)
    }

    /// Cleanup sound sequence (variant 1)
    ///
    /// Based on cleanup_sound_sequence_1 (0x800266e0)
    pub fn cleanup_sequence_1(&mut self) {
        self.sequence_active = false;
        self.sequence_status = 0;
        tracing::debug!("Sound sequence cleanup (variant 1)");
    }

    /// Cleanup sound sequence (variant 2)
    ///
    /// Based on cleanup_sound_sequence_2 (0x80026520)
    pub fn cleanup_sequence_2(&mut self) {
        // In the original, this waits for VSync and sets NCK
        self.sequence_active = false;
        tracing::debug!("Sound sequence cleanup (variant 2)");
    }
}

pub struct AudioPlugin;

impl Plugin for AudioPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<AudioSystem>()
            .add_systems(Startup, setup_audio)
            .add_systems(Update, update_audio);
    }
}

fn setup_audio(mut audio_system: ResMut<AudioSystem>) {
    tracing::info!("Initializing audio system");

    // Reset all channels to default state
    audio_system.reset_channels();

    // Mark as initialized
    audio_system.initialized = true;

    tracing::info!(
        "Audio system initialized with {} channels",
        MAX_SOUND_CHANNELS
    );
}

fn update_audio() {
    // TODO: Update audio playback
    // This will handle:
    // - Sound sequence updates
    // - Channel state updates
    // - Music streaming
}
