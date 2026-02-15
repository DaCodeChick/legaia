//! Legend of Legaia Combat System
//!
//! Data-driven combat system using Rhai scripts for AI, formulas, and behaviors.
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
