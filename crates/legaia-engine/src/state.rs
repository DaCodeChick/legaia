//! Game state management

use bevy::prelude::*;

/// Main game states
// TODO: Fix States derive macro - currently disabled until we configure Bevy correctly
#[derive(Debug, Clone, Copy, Default, Eq, PartialEq, Hash)]
pub enum GameState {
    #[default]
    Loading,
    MainMenu,
    Field,
    Battle,
    Menu,
    Cutscene,
}
