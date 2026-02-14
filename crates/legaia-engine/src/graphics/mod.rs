//! Graphics rendering system
//!
//! Provides PSX-style rendering:
//! - Model rendering
//! - Texture management
//! - Animation playback
//! - Camera control
//! - Debug text rendering

pub mod debug;

use bevy::prelude::*;
pub use debug::DebugRenderer;

pub struct GraphicsPlugin;

impl Plugin for GraphicsPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<DebugRenderer>()
            .add_systems(Startup, setup_graphics)
            .add_systems(Update, update_graphics)
            .add_systems(Update, debug::render_debug_text)
            .add_systems(Update, debug::handle_debug_input);
    }
}

fn setup_graphics(debug_renderer: Res<DebugRenderer>) {
    tracing::info!("Graphics system initialized");
    tracing::info!("Debug renderer enabled: {}", debug_renderer.enabled);
    // TODO: Setup rendering pipeline
}

fn update_graphics() {
    // TODO: Update graphics
}
