//! Battle system
//!
//! Implements the turn-based battle system including:
//! - Art system (combo input)
//! - Damage calculation
//! - Enemy AI
//! - Battle animations

use bevy::prelude::*;

pub struct BattlePlugin;

impl Plugin for BattlePlugin {
    fn build(&self, app: &mut App) {
        // TODO: Add state-based systems when state management is configured
        app.add_systems(Update, update_battle);
    }
}

fn update_battle() {
    // TODO: Update battle logic
}
