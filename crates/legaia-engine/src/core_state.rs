//! Core global game state
//!
//! This module defines the global state variables discovered through decompilation.
//! Based on analysis of init_game_state() at 0x8001d424 (SCUS_942.54).
//!
//! The original PSX game initializes 50+ global variables during startup,
//! covering GPU configuration, camera systems, sprite management, input handling,
//! and display settings.

use bevy::prelude::*;

/// GPU hardware configuration (PSX GPU registers 0x1f800000 range)
///
/// These correspond to PSX GPU control registers that configure
/// color processing, drawing offsets, and primitive rendering.
#[derive(Resource, Debug, Clone)]
pub struct GpuConfig {
    /// Color mask for RGB channels (0xffffff = no masking)
    /// Original: g_gpu_color_mask @ 0x1f8003fc
    pub color_mask: u32,

    /// Default primitive color (format: 0xAABBGGRR)
    /// Original: g_gpu_primitive_color @ 0x1f800398 = 0x2c808080
    pub primitive_color: u32,

    /// Drawing offset X (screen space offset for primitives)
    /// Original: g_gpu_x_offset @ 0x1f8003f8 = 0x0c
    pub x_offset: u16,

    /// Drawing offset Y (screen space offset for primitives)
    /// Original: g_gpu_y_offset @ 0x1f8003fa = 0x0c
    pub y_offset: u16,

    /// GPU configuration registers (purpose TBD)
    /// Original: g_gpu_register_1/2/3
    pub register_1: u16,
    pub register_2: u8,
    pub register_3: u16,

    /// Scratch buffer configuration
    /// Original: g_gpu_scratch_config @ 0x1f8003a5
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
/// The original game uses signed 16-bit fixed-point values for positions.
#[derive(Resource, Debug, Clone)]
pub struct CameraState {
    /// Camera X position offset
    /// Original: g_camera_x_offset @ 0x80084008 = 0xff9c (-100 signed)
    pub x_offset: i16,

    /// Camera Y position offset
    /// Original: g_camera_y_offset @ 0x8008400a = 0xffec (-20 signed)
    pub y_offset: i16,

    /// Camera Z position offset
    /// Original: g_camera_z_offset @ 0x8008400c = 0xfc00 (-1024 signed)
    pub z_offset: i16,

    /// Camera distance from target
    /// Original: g_camera_distance @ 0x8008401c = 0
    pub distance: i16,

    /// Zoom level (larger = closer)
    /// Original: g_camera_zoom_level @ 0x8008401e = 5000
    pub zoom_level: u16,

    /// Additional camera parameters (purpose TBD)
    /// Original: g_camera_param_1 through _5
    pub param_1: u16,
    pub param_2: u16,
    pub param_3: u16,
    pub param_4: u16,
    pub param_5: u16,
}

impl Default for CameraState {
    fn default() -> Self {
        Self {
            x_offset: -100_i16,  // 0xff9c
            y_offset: -20_i16,   // 0xffec
            z_offset: -1024_i16, // 0xfc00
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
    /// Screen brightness level (0x00-0xff, 0xf0 = near full brightness)
    /// Original: g_screen_brightness @ 0x8007b818 = 0xf0
    pub brightness: u8,

    /// Fade transition speed (frames to complete fade)
    /// Original: g_fade_speed @ 0x8007b7e4 = 8
    pub fade_speed: u8,

    /// Backup fade speed value
    /// Original: g_fade_speed_backup @ 0x8007b7be = 8
    pub fade_speed_backup: u8,

    /// Default color value for rendering (RGB)
    /// Original: g_default_color_value @ 0x8007bb48 = 0x808080 (medium gray)
    pub default_color: u32,

    /// Display enabled flag (underscore prefix indicates overlapped symbol)
    /// Original: _g_display_enabled_flag = 1
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
/// PSX controllers return button states as bitfields (0xffff = no buttons pressed).
#[derive(Resource, Debug, Clone)]
pub struct InputState {
    /// Controller button state (bitfield, 0xffff = nothing pressed)
    /// Original: g_controller_state @ 0x8007b768 = 0xffff
    pub controller_state: u16,

    /// Input processing state (bitfield)
    /// Original: g_input_state @ 0x8007b860 = 0xffff
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
/// The PSX uses two frame buffers to enable double-buffering.
#[derive(Resource, Debug, Clone)]
pub struct GraphicsBuffers {
    /// Primary graphics buffer (initialized from g_default_buffer_address)
    /// Original: g_graphics_buffer_1 @ 0x8007b728
    pub buffer_1_address: usize,

    /// Secondary graphics buffer (offset from buffer_1)
    /// Original: g_graphics_buffer_2 @ 0x8007b72c
    pub buffer_2_address: usize,

    /// Allocated buffer size in bytes
    /// Original: g_buffer_allocated_size @ 0x8007b9cc
    pub allocated_size: usize,

    /// Current active buffer index (0 or 1)
    /// Original: g_graphics_buffer_index @ 0x8007ba3c = 0
    pub active_buffer_index: u8,

    /// Expansion RAM enabled flag (PSX expansion pak)
    /// Original: g_expansion_ram_enabled @ 0x8007ba1c
    pub expansion_ram_enabled: bool,

    /// Special state buffer (allocated for state 0x14)
    /// Original: g_special_state_buffer @ 0x8007b814
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
    /// Cached CD-ROM state (0xffffffff = no cache)
    /// Original: g_cdrom_cached_state @ 0x8007b6e4 = 0xffffffff
    pub cached_state: u32,

    /// CD-ROM load operation flag (0xffffffff = no load in progress)
    /// Original: g_cdrom_load_flag @ 0x8007b6e8 = 0xffffffff
    pub load_flag: u32,

    /// Copy protection state (3 = active protection check)
    /// Original: g_cdrom_protection_state @ 0x8007bbb8
    pub protection_state: u8,

    /// Protection verification flags
    /// Original: g_cdrom_protection_flag_1/2 @ 0x8007bc9c, 0x8007bbd4
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
    /// Debug mode enabled (loaded from debug.txt, '1' = enabled)
    /// Original: g_debug_mode @ 0x8007b9b4
    pub debug_mode: bool,

    /// Initial game state ID (2 = normal, 0xe = system mode)
    /// Original: g_initial_state_id @ 0x8007b6e0
    pub initial_state_id: u8,

    /// Various unknown state flags discovered during analysis
    /// These need further investigation to determine their purpose
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
    /// Original: g_vsync_frames_target @ 0x8007b6fc = 1
    pub vsync_frames_target: u32,

    /// Timing sample count (underscore indicates overlapped symbol)
    /// Original: _g_timing_samples_count @ 0x8007b8e8 = 0x10
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
