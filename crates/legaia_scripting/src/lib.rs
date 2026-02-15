//! Legend of Legaia Scripting System
//!
//! General-purpose scripting system using Lua for entities, NPCs, combat, and environments.
//! Based on reverse-engineered entity callback system from PSX original.

pub mod components;
pub mod damage;
pub mod entity;
pub mod script;
pub mod systems;

pub use components::*;
pub use entity::*;
pub use script::*;
pub use systems::*;
