//! Core game state resources
//!
//! Global state management for the Legend of Legaia game engine using Bevy ECS.

use bevy::prelude::*;

/// Display settings for screen effects
///
/// Controls brightness, fade transitions, and color grading.
#[derive(Resource, Debug, Clone)]
pub struct DisplaySettings {
    /// Current screen brightness (0.0 = black, 1.0 = full brightness)
    pub brightness: f32,
    /// Target brightness for fade transitions
    pub target_brightness: f32,
    /// Fade transition speed (brightness units per second)
    pub fade_speed: f32,
    /// Color grading tint (multiplied with rendered colors)
    pub color_tint: Color,
}

impl Default for DisplaySettings {
    fn default() -> Self {
        Self {
            brightness: 1.0,
            target_brightness: 1.0,
            fade_speed: 2.0, // Fade completes in 0.5 seconds
            color_tint: Color::WHITE,
        }
    }
}

/// Game debug configuration
///
/// Development and testing options.
#[derive(Resource, Debug, Clone)]
pub struct DebugConfig {
    /// Enable debug overlays (FPS, memory, etc.)
    pub show_debug_info: bool,
    /// Enable collision visualization
    pub show_colliders: bool,
    /// Enable camera gizmos
    pub show_camera_gizmos: bool,
    /// Enable no-clip movement
    pub noclip_enabled: bool,
    /// Debug starting state (None = normal game flow)
    pub override_initial_state: Option<String>,
}

impl Default for DebugConfig {
    fn default() -> Self {
        Self {
            show_debug_info: cfg!(debug_assertions),
            show_colliders: false,
            show_camera_gizmos: false,
            noclip_enabled: false,
            override_initial_state: None,
        }
    }
}

/// Plugin to register all core state resources
pub struct CoreStatePlugin;

impl Plugin for CoreStatePlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<DisplaySettings>()
            .init_resource::<DebugConfig>();
    }
}
