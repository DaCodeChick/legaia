//! Input handling system
//!
//! Manages:
//! - Controller input
//! - Keyboard input
//! - Input buffering (for Art system)
//! - Menu navigation

use bevy::prelude::*;

pub struct InputPlugin;

impl Plugin for InputPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Startup, setup_input)
            .add_systems(PreUpdate, handle_input);
    }
}

fn setup_input() {
    tracing::info!("Input system initialized");
    // TODO: Setup input mappings
}

fn handle_input() {
    // TODO: Process input
}
