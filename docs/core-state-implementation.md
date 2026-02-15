# Core State Implementation

## Overview

This document describes the implementation of core global state structures based on decompilation analysis of `init_game_state()` at address 0x8001d424 in SCUS_942.54.

## Source Material

The implementation is based on DICK (Decompile It Correctly, Knucklehead) methodology analysis from Session #3 (2026-02-14), where we analyzed `init_game_state()` and discovered 52+ global variables organized into distinct subsystems.

## Module: `core_state.rs`

### Resources Implemented

#### 1. `GpuConfig` - GPU Hardware Configuration
Maps PSX GPU control registers (0x1f800000 address range):
- `color_mask`: RGB channel masking (0xffffff = no mask)
- `primitive_color`: Default primitive rendering color (0x2c808080)
- `x_offset`, `y_offset`: Screen space drawing offsets (12, 12)
- `register_1/2/3`: Additional GPU configuration registers
- `scratch_config`: Scratch buffer configuration

**Original Globals:**
- g_gpu_color_mask @ 0x1f8003fc
- g_gpu_primitive_color @ 0x1f800398
- g_gpu_x/y_offset @ 0x1f8003f8/fa
- g_gpu_register_1/2/3
- g_gpu_scratch_config @ 0x1f8003a5

#### 2. `CameraState` - 3D Camera System
Camera positioning and perspective:
- `x/y/z_offset`: Camera position offsets (-100, -20, -1024)
- `distance`: Distance from camera target (0)
- `zoom_level`: Zoom/FOV control (5000)
- `param_1-5`: Additional camera parameters (purpose TBD)

**Original Globals:**
- g_camera_x/y/z_offset @ 0x80084008/a/c
- g_camera_distance @ 0x8008401c
- g_camera_zoom_level @ 0x8008401e
- g_camera_param_1 through _5

#### 3. `DisplayConfig` - Display & Rendering
Screen display configuration:
- `brightness`: Screen brightness level (0xf0 ≈ 94%)
- `fade_speed`: Fade transition speed in frames (8)
- `fade_speed_backup`: Backup fade speed value (8)
- `default_color`: Default rendering color (0x808080 = gray)
- `display_enabled`: Display enable flag

**Original Globals:**
- g_screen_brightness @ 0x8007b818
- g_fade_speed @ 0x8007b7e4
- g_fade_speed_backup @ 0x8007b7be
- g_default_color_value @ 0x8007bb48
- _g_display_enabled_flag

#### 4. `InputState` - Controller Input
PSX controller state tracking:
- `controller_state`: Button bitfield (0xffff = no buttons)
- `input_state`: Input processing state (0xffff)

**Original Globals:**
- g_controller_state @ 0x8007b768
- g_input_state @ 0x8007b860

#### 5. `GraphicsBuffers` - Frame Buffer Management
Double-buffered graphics memory:
- `buffer_1/2_address`: Primary/secondary frame buffer addresses
- `allocated_size`: Total allocated buffer size
- `active_buffer_index`: Current active buffer (0 or 1)
- `expansion_ram_enabled`: PSX expansion pak flag
- `special_state_buffer`: Optional buffer for state 0x14

**Original Globals:**
- g_graphics_buffer_1/2 @ 0x8007b728/2c
- g_buffer_allocated_size @ 0x8007b9cc
- g_graphics_buffer_index @ 0x8007ba3c
- g_expansion_ram_enabled @ 0x8007ba1c
- g_special_state_buffer @ 0x8007b814

#### 6. `CdromState` - CD-ROM System
CD-ROM operation tracking:
- `cached_state`: Cached CD-ROM state (0xffffffff = no cache)
- `load_flag`: Load operation flag (0xffffffff = idle)
- `protection_state`: Copy protection state (3 = active)
- `protection_flag_1/2`: Protection verification flags

**Original Globals:**
- g_cdrom_cached_state @ 0x8007b6e4
- g_cdrom_load_flag @ 0x8007b6e8
- g_cdrom_protection_state @ 0x8007bbb8
- g_cdrom_protection_flag_1/2

#### 7. `DebugFlags` - Debug & Development
Development mode controls:
- `debug_mode`: Debug mode enabled (from debug.txt)
- `initial_state_id`: Initial game state (2=normal, 0xe=system)
- `unknown_flag_1-6`: Flags needing further investigation

**Original Globals:**
- g_debug_mode @ 0x8007b9b4
- g_initial_state_id @ 0x8007b6e0
- g_unknown_flag_1 through _6

#### 8. `TimingState` - Frame Timing
VSync and frame timing:
- `vsync_frames_target`: Frames per VSync (1)
- `timing_samples_count`: Sample count (0x10)

**Original Globals:**
- g_vsync_frames_target @ 0x8007b6fc
- _g_timing_samples_count @ 0x8007b8e8

## Integration

The `CoreStatePlugin` registers all resources with Bevy's ECS system. Add it to your app:

```rust
use legaia_engine::LegaiaEnginePlugin;

app.add_plugins(LegaiaEnginePlugin);
```

This automatically initializes all core state resources with their default PSX values.

## Usage Examples

### Accessing Camera State
```rust
fn my_system(camera: Res<CameraState>) {
    println!("Camera zoom: {}", camera.zoom_level);
    println!("Camera pos: ({}, {}, {})", 
        camera.x_offset, camera.y_offset, camera.z_offset);
}
```

### Modifying Display Settings
```rust
fn fade_screen(mut display: ResMut<DisplayConfig>) {
    if display.brightness > 0 {
        display.brightness = display.brightness.saturating_sub(display.fade_speed);
    }
}
```

### Checking Debug Mode
```rust
fn debug_system(debug: Res<DebugFlags>) {
    if debug.debug_mode {
        // Render debug information
    }
}
```

## Future Work

### Sprite System (Not Yet Implemented)
The original game has a complex sprite management system with:
- 16 sprite table entries (g_sprite_table_entry_0 through _14)
- 6 sprite data buffers (g_sprite_data_buffer_0 through _5)
- Hardware register table (g_hardware_register_table)

These need to be implemented as a separate sprite management system once we understand the sprite data structures better.

### Unknown Flags
Several flags discovered during decompilation need further investigation:
- `g_unknown_flag_1` through `_6`: Purpose unknown
- `g_camera_param_1` through `_5`: Camera parameters (need to observe in-game)
- `g_gpu_register_1/2/3`: GPU configuration (need PSX documentation)

### Next Steps
1. Implement sprite management system
2. Connect input system to PSX controller mapping
3. Implement display fade effects using DisplayConfig
4. Add camera movement/interpolation systems
5. Investigate unknown flags through gameplay analysis

## References

- Decompilation: `.opencode/AGENTS.md` - Session #3 progress
- Original function: `init_game_state()` @ 0x8001d424 (SCUS_942.54)
- Analysis methodology: DICK (Decompile It Correctly, Knucklehead)
- Source file: `crates/legaia-engine/src/core_state.rs`
