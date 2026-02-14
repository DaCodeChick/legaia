//! Graphics rendering system
//!
//! Provides PSX-style rendering:
//! - Model rendering
//! - Texture management
//! - Animation playback
//! - Camera control

use bevy::prelude::*;

pub struct GraphicsPlugin;

impl Plugin for GraphicsPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Startup, setup_graphics)
            .add_systems(Update, update_graphics);
    }
}

fn setup_graphics() {
    tracing::info!("Graphics system initialized");
    // TODO: Setup rendering pipeline
}

fn update_graphics() {
    // TODO: Update graphics
}
