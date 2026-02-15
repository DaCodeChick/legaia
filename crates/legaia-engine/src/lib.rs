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
pub mod core_state;
pub mod field;
pub mod graphics;
pub mod input;
pub mod menu;
pub mod state;

pub use core_state::*;
pub use state::{GameState, StateManager};

use bevy::prelude::*;

/// Main engine plugin that sets up all systems
pub struct LegaiaEnginePlugin;

impl Plugin for LegaiaEnginePlugin {
    fn build(&self, app: &mut App) {
        app
            // Core state resources (from decompilation analysis)
            .add_plugins(CoreStatePlugin)
            // State management
            .init_state::<GameState>()
            .init_resource::<StateManager>()
            // Add state management systems
            .add_systems(Update, state::update_frame_counter)
            .add_systems(Update, state::handle_state_transitions)
            // Add core systems
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
    // Camera setup (Bevy 0.18+ uses required components instead of bundles)
    commands.spawn((
        Camera3d::default(),
        Transform::from_xyz(0.0, 5.0, 10.0).looking_at(Vec3::ZERO, Vec3::Y),
    ));

    // Lighting (Bevy 0.18+ uses required components)
    commands.spawn((
        DirectionalLight {
            illuminance: 10000.0,
            ..default()
        },
        Transform::from_rotation(Quat::from_euler(EulerRot::XYZ, -0.5, -0.5, 0.0)),
    ));

    tracing::info!("Engine initialized");
}
