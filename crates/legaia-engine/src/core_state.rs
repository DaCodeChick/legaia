//! Core global game state
//!
//! This module defines global state resources for the game engine,
//! covering GPU configuration, camera systems, sprite management,
//! input handling, and display settings.

use bevy::prelude::*;

/// GPU hardware configuration
///
/// Configures color processing, drawing offsets, and primitive rendering.
#[derive(Resource, Debug, Clone)]
pub struct GpuConfig {
    /// Color mask for RGB channels (0xffffff = no masking)
    pub color_mask: u32,
    /// Default primitive color (format: 0xAABBGGRR)
    pub primitive_color: u32,
    /// Drawing offset X (screen space offset for primitives)
    pub x_offset: u16,
    /// Drawing offset Y (screen space offset for primitives)
    pub y_offset: u16,
    /// GPU configuration registers
    pub register_1: u16,
    pub register_2: u8,
    pub register_3: u16,
    /// Scratch buffer configuration
    pub scratch_config: u8,
}

impl Default for GpuConfig {
    fn default() -> Self {
        Self {
            color_mask: 0xffffff,
            primitive_color: 0x2c808080,
            x_offset: 0x0c,
            y_offset: 0x0c,
            register_1: 0x10,
            register_2: 8,
            register_3: 0x10,
            scratch_config: 0,
        }
    }
}

/// Camera system state
///
/// Manages 3D camera positioning, zoom, and perspective parameters.
#[derive(Resource, Debug, Clone)]
pub struct CameraState {
    /// Camera X position offset
    pub x_offset: i16,
    /// Camera Y position offset
    pub y_offset: i16,
    /// Camera Z position offset
    pub z_offset: i16,
    /// Camera distance from target
    pub distance: i16,
    /// Zoom level (larger = closer)
    pub zoom_level: u16,
    /// Additional camera parameters
    pub param_1: u16,
    pub param_2: u16,
    pub param_3: u16,
    pub param_4: u16,
    pub param_5: u16,
}

impl Default for CameraState {
    fn default() -> Self {
        Self {
            x_offset: -100,
            y_offset: -20,
            z_offset: -1024,
            distance: 0,
            zoom_level: 5000,
            param_1: 0,
            param_2: 0,
            param_3: 0,
            param_4: 0,
            param_5: 0,
        }
    }
}

/// Display rendering configuration
///
/// Controls screen brightness, fade effects, and color rendering.
#[derive(Resource, Debug, Clone)]
pub struct DisplayConfig {
    /// Screen brightness level (0x00-0xff)
    pub brightness: u8,
    /// Fade transition speed (frames to complete fade)
    pub fade_speed: u8,
    /// Backup fade speed value
    pub fade_speed_backup: u8,
    /// Default color value for rendering (RGB)
    pub default_color: u32,
    /// Display enabled flag
    pub display_enabled: bool,
}

impl Default for DisplayConfig {
    fn default() -> Self {
        Self {
            brightness: 0xf0,
            fade_speed: 8,
            fade_speed_backup: 8,
            default_color: 0x808080,
            display_enabled: true,
        }
    }
}

/// Input and controller state
///
/// Tracks controller input and button states.
#[derive(Resource, Debug, Clone)]
pub struct InputState {
    /// Controller button state (bitfield)
    pub controller_state: u16,
    /// Input processing state (bitfield)
    pub input_state: u16,
}

impl Default for InputState {
    fn default() -> Self {
        Self {
            controller_state: 0xffff,
            input_state: 0xffff,
        }
    }
}

/// Graphics buffer management
///
/// Manages double-buffered graphics memory regions used for rendering.
#[derive(Resource, Debug, Clone)]
pub struct GraphicsBuffers {
    /// Primary graphics buffer address
    pub buffer_1_address: usize,
    /// Secondary graphics buffer address
    pub buffer_2_address: usize,
    /// Allocated buffer size in bytes
    pub allocated_size: usize,
    /// Current active buffer index (0 or 1)
    pub active_buffer_index: u8,
    /// Expansion RAM enabled flag
    pub expansion_ram_enabled: bool,
    /// Special state buffer
    pub special_state_buffer: Option<usize>,
}

impl Default for GraphicsBuffers {
    fn default() -> Self {
        Self {
            buffer_1_address: 0,
            buffer_2_address: 0,
            allocated_size: 0,
            active_buffer_index: 0,
            expansion_ram_enabled: false,
            special_state_buffer: None,
        }
    }
}

/// CD-ROM system state
///
/// Tracks CD-ROM operations, caching, and error states.
#[derive(Resource, Debug, Clone)]
pub struct CdromState {
    /// Cached CD-ROM state
    pub cached_state: u32,
    /// CD-ROM load operation flag
    pub load_flag: u32,
    /// Copy protection state
    pub protection_state: u8,
    /// Protection verification flags
    pub protection_flag_1: u32,
    pub protection_flag_2: u32,
}

impl Default for CdromState {
    fn default() -> Self {
        Self {
            cached_state: 0xffffffff,
            load_flag: 0xffffffff,
            protection_state: 0,
            protection_flag_1: 0,
            protection_flag_2: 0,
        }
    }
}

/// Debug and development flags
///
/// Controls debug features and development mode settings.
#[derive(Resource, Debug, Clone)]
pub struct DebugFlags {
    /// Debug mode enabled
    pub debug_mode: bool,
    /// Initial game state ID
    pub initial_state_id: u8,
    /// Additional state flags
    pub unknown_flag_1: u32,
    pub unknown_flag_2: u32,
    pub unknown_flag_3: u16,
    pub unknown_flag_4: u32,
    pub unknown_flag_5: u32,
    pub unknown_flag_6: u32,
}

impl Default for DebugFlags {
    fn default() -> Self {
        Self {
            debug_mode: false,
            initial_state_id: 2,
            unknown_flag_1: 0,
            unknown_flag_2: 0,
            unknown_flag_3: 0,
            unknown_flag_4: 0,
            unknown_flag_5: 0,
            unknown_flag_6: 0,
        }
    }
}

/// Timing and synchronization state
///
/// Manages frame timing and VSync-related counters.
#[derive(Resource, Debug, Clone)]
pub struct TimingState {
    /// VSync frame target (number of frames per sync)
    pub vsync_frames_target: u32,
    /// Timing sample count
    pub timing_samples_count: u16,
}

impl Default for TimingState {
    fn default() -> Self {
        Self {
            vsync_frames_target: 1,
            timing_samples_count: 0x10,
        }
    }
}

/// Plugin to register all core state resources
pub struct CoreStatePlugin;

impl Plugin for CoreStatePlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<GpuConfig>()
            .init_resource::<CameraState>()
            .init_resource::<DisplayConfig>()
            .init_resource::<InputState>()
            .init_resource::<GraphicsBuffers>()
            .init_resource::<CdromState>()
            .init_resource::<DebugFlags>()
            .init_resource::<TimingState>();
    }
}
