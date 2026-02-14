//! Menu system
//!
//! Implements the in-game menu including:
//! - Character status
//! - Inventory management
//! - Equipment
//! - Save/load
//! - Options

use bevy::prelude::*;

pub struct MenuPlugin;

impl Plugin for MenuPlugin {
    fn build(&self, app: &mut App) {
        // TODO: Add state-based systems when state management is configured
        app.add_systems(Update, update_menu);
    }
}

fn update_menu() {
    // TODO: Update menu logic
}
