//! Debug text rendering system
//!
//! PSX-style on-screen debug text rendering:
//! - Character-by-character rendering
//! - Cursor position tracking
//! - 8-color palette support
//! - Format specifiers: %d, %x, %s, %c, %0Nd, %1-9d
//! - Newline handling with automatic cursor reset

use bevy::gizmos::prelude::*;
use bevy::input::ButtonInput;
use bevy::prelude::{Color, ColorToPacked, Component, KeyCode, Res, ResMut, Resource, Vec2};

/// Default debug text color (gray)
pub const DEFAULT_TEXT_COLOR: u32 = 0x808080;

/// Newline X position reset value
pub const NEWLINE_X_RESET: i32 = 0x10;

/// Line height in pixels
pub const LINE_HEIGHT: i32 = 8;

/// Character width in pixels (approximate)
pub const CHAR_WIDTH: i32 = 8;

/// Debug color palette (8 colors, indexed by %c format specifier)
pub const DEBUG_COLOR_PALETTE: [Color; 8] = [
    Color::srgb(1.0, 1.0, 1.0), // 0: White
    Color::srgb(1.0, 0.0, 0.0), // 1: Red
    Color::srgb(0.0, 1.0, 0.0), // 2: Green
    Color::srgb(0.0, 0.0, 1.0), // 3: Blue
    Color::srgb(1.0, 1.0, 0.0), // 4: Yellow
    Color::srgb(1.0, 0.0, 1.0), // 5: Magenta
    Color::srgb(0.0, 1.0, 1.0), // 6: Cyan
    Color::srgb(0.5, 0.5, 0.5), // 7: Gray
];

/// Debug text renderer state
#[derive(Resource, Debug)]
pub struct DebugRenderer {
    /// Current cursor X position
    pub cursor_x: i32,

    /// Current cursor Y position
    pub cursor_y: i32,

    /// Current text color (RGB packed as u32)
    pub text_color: u32,

    /// Color palette for %c format specifier
    pub color_palette: [Color; 8],

    /// Enabled flag
    pub enabled: bool,
}

impl Default for DebugRenderer {
    fn default() -> Self {
        Self {
            cursor_x: NEWLINE_X_RESET,
            cursor_y: 0,
            text_color: DEFAULT_TEXT_COLOR,
            color_palette: DEBUG_COLOR_PALETTE,
            enabled: true,
        }
    }
}

impl DebugRenderer {
    /// Create a new debug renderer
    pub fn new() -> Self {
        Self::default()
    }

    /// Reset cursor to start of screen
    pub fn reset_cursor(&mut self) {
        self.cursor_x = NEWLINE_X_RESET;
        self.cursor_y = 0;
    }

    /// Handle newline - reset X, advance Y
    pub fn newline(&mut self) {
        self.cursor_x = NEWLINE_X_RESET;
        self.cursor_y += LINE_HEIGHT;
    }

    /// Advance cursor by character width
    pub fn advance_cursor(&mut self) {
        self.cursor_x += CHAR_WIDTH;
    }

    /// Set color by palette index (0-7)
    pub fn set_color_by_index(&mut self, index: u8) {
        if (index as usize) < self.color_palette.len() {
            let color = self.color_palette[index as usize];
            // Convert Color to packed RGB
            self.text_color = color_to_rgb_u32(color);
        }
    }

    /// Set text color directly
    pub fn set_color(&mut self, color: u32) {
        self.text_color = color;
    }

    /// Get current color as Bevy Color
    pub fn get_color(&self) -> Color {
        rgb_u32_to_color(self.text_color)
    }

    /// Enable debug rendering
    pub fn enable(&mut self) {
        self.enabled = true;
    }

    /// Disable debug rendering
    pub fn disable(&mut self) {
        self.enabled = false;
    }
}

/// Convert packed RGB u32 to Bevy Color
fn rgb_u32_to_color(rgb: u32) -> Color {
    let r = ((rgb >> 16) & 0xFF) as f32 / 255.0;
    let g = ((rgb >> 8) & 0xFF) as f32 / 255.0;
    let b = (rgb & 0xFF) as f32 / 255.0;
    Color::srgb(r, g, b)
}

/// Convert Bevy Color to packed RGB u32
fn color_to_rgb_u32(color: Color) -> u32 {
    let [r, g, b, _] = color.to_srgba().to_u8_array();
    ((r as u32) << 16) | ((g as u32) << 8) | (b as u32)
}

/// Marker component for debug text entities
#[derive(Component)]
pub struct DebugText;

/// System to render debug text
///
/// In the future, this will implement the full printf-style formatting
pub fn render_debug_text(debug_renderer: Res<DebugRenderer>, mut gizmos: Gizmos) {
    if !debug_renderer.enabled {
        return;
    }

    // TODO: Implement actual text rendering
    // For now, just draw a cursor indicator
    let pos = Vec2::new(
        debug_renderer.cursor_x as f32,
        debug_renderer.cursor_y as f32,
    );

    gizmos.circle_2d(pos, 2.0, debug_renderer.get_color());
}

/// System to handle debug input
pub fn handle_debug_input(
    keyboard: Res<ButtonInput<KeyCode>>,
    mut debug_renderer: ResMut<DebugRenderer>,
) {
    // Toggle debug rendering with F1
    if keyboard.just_pressed(KeyCode::F1) {
        if debug_renderer.enabled {
            debug_renderer.disable();
            tracing::info!("Debug rendering disabled");
        } else {
            debug_renderer.enable();
            tracing::info!("Debug rendering enabled");
        }
    }

    // Reset cursor with F2
    if keyboard.just_pressed(KeyCode::F2) {
        debug_renderer.reset_cursor();
        tracing::info!("Debug cursor reset");
    }
}
