//! Audio system
//!
//! Handles:
//! - Music playback
//! - Sound effects
//! - Audio mixing

use bevy::prelude::*;

pub struct AudioPlugin;

impl Plugin for AudioPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Startup, setup_audio)
            .add_systems(Update, update_audio);
    }
}

fn setup_audio() {
    tracing::info!("Audio system initialized");
    // TODO: Initialize audio system
}

fn update_audio() {
    // TODO: Update audio
}
