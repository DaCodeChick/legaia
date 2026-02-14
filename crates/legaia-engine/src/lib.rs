//! Legend of Legaia game engine
//!
//! This crate provides the game engine built on Bevy for the Legend of Legaia rewrite.
//! It includes all major game systems:
//! - Battle system
//! - Field/world system
//! - Menu system
//! - Graphics rendering
//! - Audio playback
//! - Input handling

pub mod audio;
pub mod battle;
pub mod field;
pub mod graphics;
pub mod input;
pub mod menu;
pub mod state;

pub use state::GameState;

use bevy::prelude::*;

/// Main engine plugin that sets up all systems
pub struct LegaiaEnginePlugin;

impl Plugin for LegaiaEnginePlugin {
    fn build(&self, app: &mut App) {
        app
            // TODO: Add game state management
            // .insert_state(GameState::Loading)
            // Add systems
            .add_systems(Startup, setup)
            // Battle system
            .add_plugins(battle::BattlePlugin)
            // Field system
            .add_plugins(field::FieldPlugin)
            // Menu system
            .add_plugins(menu::MenuPlugin)
            // Graphics
            .add_plugins(graphics::GraphicsPlugin)
            // Audio
            .add_plugins(audio::AudioPlugin)
            // Input
            .add_plugins(input::InputPlugin);
    }
}

fn setup(mut commands: Commands) {
    // Camera setup
    commands.spawn(Camera3dBundle {
        transform: Transform::from_xyz(0.0, 5.0, 10.0).looking_at(Vec3::ZERO, Vec3::Y),
        ..default()
    });

    // Lighting
    commands.spawn(DirectionalLightBundle {
        directional_light: DirectionalLight {
            illuminance: 10000.0,
            ..default()
        },
        transform: Transform::from_rotation(Quat::from_euler(EulerRot::XYZ, -0.5, -0.5, 0.0)),
        ..default()
    });

    tracing::info!("Engine initialized");
}
