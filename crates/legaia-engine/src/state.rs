//! Game state management
//!
//! Based on decompilation of main() (0x80015e90):
//! - Original uses integer-indexed state machine with function pointer table
//! - 6 function handlers per state (likely: init, update, draw, cleanup, + 2 unknown)
//! - State transitions reset 4 counters
//! - Negative state value triggers exit to PSX.EXE

use bevy::prelude::{ResMut, Resource};
use bevy::state::state::{NextState, States};

/// Main game states
///
/// Corresponds to indices in the original g_state_handler_table (0x8007079c).
/// Each state has 6 function pointers in the original game.
#[derive(States, Debug, Clone, Copy, Default, Eq, PartialEq, Hash)]
pub enum GameState {
    /// Loading screen - initial state
    #[default]
    Loading,
    /// Main menu
    MainMenu,
    /// Field/world exploration
    Field,
    /// Battle system
    Battle,
    /// In-game menu system
    Menu,
    /// Cutscene playback
    Cutscene,
    /// Exit state (corresponds to negative state value in original)
    Exit,
}

/// State machine manager
///
/// Mirrors the original game's state management system:
/// - g_current_game_state (0x8007b83c)
/// - g_previous_game_state (0x8007b7ac)
/// - g_state_backup (0x8007b87c)
/// - g_state_counter_1 through g_state_counter_4
#[derive(Resource, Debug)]
pub struct StateManager {
    /// Current active state
    pub current_state: GameState,
    /// Previous state (for transition detection)
    pub previous_state: GameState,
    /// Backup state
    pub backup_state: GameState,
    /// State-specific counters (reset on state transition)
    pub counter_1: u32,
    pub counter_2: u32,
    pub counter_3: u32,
    pub counter_4: u32,
    /// Frame counter (g_frame_counter at 0x8007b6f4)
    pub frame_counter: u32,
}

impl Default for StateManager {
    fn default() -> Self {
        Self {
            current_state: GameState::Loading,
            previous_state: GameState::Loading,
            backup_state: GameState::Loading,
            counter_1: 0,
            counter_2: 0,
            counter_3: 0,
            counter_4: 0,
            frame_counter: 0,
        }
    }
}

impl StateManager {
    /// Create a new state manager
    pub fn new() -> Self {
        Self::default()
    }

    /// Check if state has changed (for transition detection)
    pub fn state_changed(&self) -> bool {
        self.current_state != self.previous_state
    }

    /// Reset counters on state transition (from cleanup_and_transition_state)
    pub fn reset_counters(&mut self) {
        self.counter_1 = 0;
        self.counter_2 = 0;
        self.counter_3 = 0;
        self.counter_4 = 0;
    }

    /// Transition to a new state
    pub fn transition_to(&mut self, new_state: GameState) {
        if self.current_state != new_state {
            self.previous_state = self.current_state;
            self.current_state = new_state;
            self.backup_state = new_state;
            self.reset_counters();
        }
    }

    /// Update frame counter (called once per frame)
    pub fn tick_frame(&mut self) {
        self.frame_counter = self.frame_counter.wrapping_add(1);
    }
}

/// System to update frame counter
pub fn update_frame_counter(mut state_mgr: ResMut<StateManager>) {
    state_mgr.tick_frame();
}

/// System to detect and handle state transitions
pub fn handle_state_transitions(
    mut state_mgr: ResMut<StateManager>,
    mut next_state: ResMut<NextState<GameState>>,
) {
    if state_mgr.state_changed() {
        // Update previous state tracker
        state_mgr.previous_state = state_mgr.current_state;

        // Trigger Bevy state transition
        next_state.set(state_mgr.current_state);

        tracing::info!(
            "State transition: {:?} -> {:?}",
            state_mgr.previous_state,
            state_mgr.current_state
        );
    }
}
