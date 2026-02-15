//! Entity Components for Combat System
//!
//! Based on PSX entity structure (24+ offsets documented in entity-structure.md)

use bevy::prelude::*;
use serde::{Deserialize, Serialize};

/// Core combat entity component
/// Maps to PSX entity_t structure
#[derive(Component, Debug, Clone, Serialize, Deserialize)]
pub struct CombatEntity {
    /// Entity flags (PSX offset +0x10)
    pub flags: u32,

    /// Animation flags (PSX offset +0x62)
    pub anim_flags: u16,

    /// Ordering table depth for rendering (PSX offset +0x9e)
    pub render_priority: i16,
}

/// RGB color interpolation component
/// Handles smooth color transitions (PSX offsets +0x7c to +0x90)
#[derive(Component, Debug, Clone)]
pub struct ColorInterpolation {
    /// Current RGB values (0.0-1.0 range)
    pub current: Vec3,

    /// Target RGB values
    pub target: Vec3,

    /// Velocity for interpolation (signed deltas)
    pub velocity: Vec3,
}

/// Animation timer component
/// Maps to PSX timers at offsets +0x98, +0x9a, +0x9c
#[derive(Component, Debug, Clone)]
pub struct AnimationTimers {
    /// General animation timer
    pub timer_1: i16,

    /// Auto-destruction timer (entity despawns when reaches 0)
    pub timer_2: i16,

    /// Animation completion timer
    pub timer_3: i16,
}

/// Script callback component
/// Replaces PSX function pointer at offset +0xc
#[derive(Component, Debug, Clone)]
pub struct ScriptCallback {
    /// Script file path (e.g., "scripts/combat/enemy_slime.rhai")
    pub script_path: String,

    /// Callback function name (e.g., "on_update", "on_damage", "choose_action")
    pub function: String,
}

/// Combat stats for entities (player or enemy)
#[derive(Component, Debug, Clone, Serialize, Deserialize)]
pub struct CombatStats {
    pub hp: u32,
    pub max_hp: u32,
    pub mp: u32,
    pub max_mp: u32,
    pub attack: u32,
    pub defense: u32,
    pub speed: u32,
    pub level: u32,
}

/// Turn state for turn-based combat
#[derive(Component, Debug, Clone, PartialEq, Eq)]
pub enum TurnState {
    Waiting,
    ChoosingAction,
    ExecutingAction,
    TakingDamage,
    Dead,
}

/// Action queue for entity
#[derive(Component, Debug, Clone)]
pub struct ActionQueue {
    pub actions: Vec<CombatAction>,
}

/// Represents a combat action (attack, item, art, etc.)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CombatAction {
    pub action_type: ActionType,
    pub target: Option<Entity>,
    pub power: u32,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ActionType {
    Attack,
    Art { art_id: u32 },
    Item { item_id: u32 },
    Defend,
    Escape,
}

/// Battle mode state
/// Maps to g_battle_mode in PSX code
#[derive(Resource, Debug, Clone, Copy, PartialEq, Eq)]
pub enum BattleMode {
    Normal,
    Tutorial,
    Intro,
    Exit,
}

/// Battle state resource
#[derive(Resource, Debug, Clone)]
pub struct BattleState {
    pub mode: BattleMode,
    pub turn_order: Vec<Entity>,
    pub current_turn_index: usize,
}
