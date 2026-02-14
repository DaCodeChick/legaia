//! Legend of Legaia - Game Entry Point

use bevy::prelude::*;
use legaia_engine::LegaiaEnginePlugin;

fn main() {
    // Initialize logging
    tracing_subscriber::fmt()
        .with_target(false)
        .with_level(true)
        .init();

    tracing::info!("Starting Legend of Legaia");

    App::new()
        // Bevy default plugins
        .add_plugins(DefaultPlugins.set(WindowPlugin {
            primary_window: Some(Window {
                title: "Legend of Legaia".into(),
                resolution: (1280, 720).into(), // Bevy 0.18+ requires integers
                present_mode: bevy::window::PresentMode::Fifo,
                ..default()
            }),
            ..default()
        }))
        // Game engine
        .add_plugins(LegaiaEnginePlugin)
        // Run the game
        .run();
}
