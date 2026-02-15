//! Entity callback system
//!
//! Bevy implementation of PSX entity callback architecture

use crate::components::*;
use crate::script::*;
use bevy::prelude::*;

/// System that updates entity callbacks (matches PSX update_entity_list_logic)
pub fn update_entity_callbacks(
    mut query: Query<(
        Entity,
        &ScriptCallback,
        &CombatStats,
        &ColorInterpolation,
        &AnimationTimers,
    )>,
    script_engine: Res<ScriptEngine>,
    _battle_state: Res<BattleState>,
) {
    for (entity, callback, stats, color, timers) in query.iter_mut() {
        // Build script context
        let context = EntityScriptContext {
            stats: stats.into(),
            current_color: [
                (color.current.x * 0x3fc0 as f32) as u16,
                (color.current.y * 0x3fc0 as f32) as u16,
                (color.current.z * 0x3fc0 as f32) as u16,
            ],
            target_color: [
                (color.target.x * 0x3fc0 as f32) as u16,
                (color.target.y * 0x3fc0 as f32) as u16,
                (color.target.z * 0x3fc0 as f32) as u16,
            ],
            timers: (timers.timer_1, timers.timer_2, timers.timer_3),
            alive_enemies: 0, // TODO: count from query
            alive_allies: 0,  // TODO: count from query
            turn_number: 0,   // TODO: get from battle state
        };

        // Call script callback
        if let Err(e) = script_engine.call_entity_callback(&callback.function, context) {
            error!("Script callback failed for entity {:?}: {}", entity, e);
        }
    }
}

/// System that updates RGB color interpolation (matches PSX update_entity_animation_interpolation)
pub fn update_color_interpolation(mut query: Query<&mut ColorInterpolation>, time: Res<Time>) {
    let delta = time.delta_seconds();

    for mut color in query.iter_mut() {
        // Apply velocity to move current toward target
        let velocity = color.velocity * delta;
        color.current += velocity;

        // Clamp each component to target if we've reached it
        for i in 0..3 {
            if (color.current[i] - color.target[i]).abs() < 0.01 {
                color.current[i] = color.target[i];
                color.velocity[i] = 0.0;
            }
        }
    }
}

/// System that updates animation timers
pub fn update_animation_timers(
    mut commands: Commands,
    mut query: Query<(Entity, &mut AnimationTimers)>,
    time: Res<Time>,
) {
    let delta_frames = (time.delta_seconds() * 60.0) as i16; // Assuming 60fps target

    for (entity, mut timers) in query.iter_mut() {
        // Decrement all timers
        timers.timer_1 = timers.timer_1.saturating_sub(delta_frames);
        timers.timer_2 = timers.timer_2.saturating_sub(delta_frames);
        timers.timer_3 = timers.timer_3.saturating_sub(delta_frames);

        // Auto-destroy if timer_2 reaches 0 (PSX behavior)
        if timers.timer_2 == 0 {
            commands.entity(entity).despawn();
        }
    }
}
