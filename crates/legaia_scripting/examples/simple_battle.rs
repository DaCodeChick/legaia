//! Example combat scenario
//! Demonstrates the script-based Lua combat system

use bevy::prelude::*;
use legaia_scripting::*;

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_plugins(CombatPlugin)
        .add_systems(Startup, setup_battle)
        .run();
}

fn setup_battle(mut commands: Commands, mut script_engine: ResMut<ScriptEngine>) {
    // Load AI scripts (Lua)
    script_engine
        .load_script("scripts/entities/enemy_slime.lua")
        .expect("Failed to load slime AI");

    // Spawn player character
    commands.spawn((
        Name::new("Vahn"),
        CombatEntity {
            flags: 0,
            anim_flags: 0,
            render_priority: 100,
        },
        CombatStats {
            hp: 120,
            max_hp: 120,
            mp: 60,
            max_mp: 60,
            attack: 35,
            defense: 25,
            speed: 30,
            level: 8,
        },
        ColorInterpolation {
            current: Vec3::new(1.0, 1.0, 1.0),
            target: Vec3::new(1.0, 1.0, 1.0),
            velocity: Vec3::ZERO,
        },
        AnimationTimers {
            timer_1: 0,
            timer_2: -1,
            timer_3: 0,
        },
        TurnState::Waiting,
        ActionQueue { actions: vec![] },
    ));

    // Spawn enemy (slime)
    commands.spawn((
        Name::new("Slime"),
        CombatEntity {
            flags: 0,
            anim_flags: 0,
            render_priority: 90,
        },
        CombatStats {
            hp: 50,
            max_hp: 50,
            mp: 0,
            max_mp: 0,
            attack: 15,
            defense: 10,
            speed: 12,
            level: 3,
        },
        ScriptCallback {
            script_path: "scripts/entities/enemy_slime.lua".to_string(),
            function: "on_update".to_string(),
        },
        ColorInterpolation {
            current: Vec3::new(1.0, 1.0, 1.0),
            target: Vec3::new(1.0, 1.0, 1.0),
            velocity: Vec3::ZERO,
        },
        AnimationTimers {
            timer_1: 0,
            timer_2: -1,
            timer_3: 0,
        },
        TurnState::Waiting,
        ActionQueue { actions: vec![] },
    ));

    info!("Battle initialized: Vahn vs Slime");
}
