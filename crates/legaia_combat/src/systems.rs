//! Combat system integration for Bevy
//!
//! Provides systems, plugins, and scheduling

use crate::components::*;
use crate::entity::*;
use crate::script::*;
use bevy::prelude::*;

/// Combat system plugin
pub struct CombatPlugin;

impl Plugin for CombatPlugin {
    fn build(&self, app: &mut App) {
        app
            // Resources
            .insert_resource(BattleState {
                mode: BattleMode::Normal,
                turn_order: Vec::new(),
                current_turn_index: 0,
            })
            .insert_resource(ScriptEngine::new())
            // Systems - matches PSX execution order:
            // 1. Update entity callbacks (game logic)
            // 2. Update animations/interpolations
            // 3. Rendering (handled by Bevy's built-in systems)
            .add_systems(
                Update,
                (
                    update_entity_callbacks,
                    update_color_interpolation,
                    update_animation_timers,
                    turn_system,
                ),
            );
    }
}

/// Turn-based combat system
pub fn turn_system(
    mut battle_state: ResMut<BattleState>,
    query: Query<(Entity, &CombatStats, &TurnState)>,
) {
    // Skip if no entities
    if query.is_empty() {
        return;
    }

    // Build turn order if empty
    if battle_state.turn_order.is_empty() {
        let mut entities: Vec<_> = query
            .iter()
            .filter(|(_, stats, state)| **state != TurnState::Dead && stats.hp > 0)
            .collect();

        // Sort by speed (descending)
        entities.sort_by(|a, b| b.1.speed.cmp(&a.1.speed));

        battle_state.turn_order = entities.into_iter().map(|(e, _, _)| e).collect();
        battle_state.current_turn_index = 0;
    }

    // TODO: Process current turn
}
