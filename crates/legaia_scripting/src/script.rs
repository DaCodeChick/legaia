//! Script API and Engine Integration
//!
//! Provides Lua script bindings for entity callbacks, combat AI, NPCs, and environments

use bevy::prelude::*;
use mlua::prelude::*;
use std::collections::HashMap;
use std::sync::{Arc, Mutex};

use crate::components::*;

/// Script engine resource
#[derive(Resource, Clone)]
pub struct ScriptEngine {
    lua: Arc<Mutex<Lua>>,
    loaded_scripts: Arc<Mutex<HashMap<String, ()>>>,
}

impl Default for ScriptEngine {
    fn default() -> Self {
        Self::new()
    }
}

impl ScriptEngine {
    pub fn new() -> Self {
        let lua = Lua::new();

        // Register the entity API
        Self::register_api(&lua).expect("Failed to register Lua API");

        Self {
            lua: Arc::new(Mutex::new(lua)),
            loaded_scripts: Arc::new(Mutex::new(HashMap::new())),
        }
    }

    /// Load a script from file
    pub fn load_script(&mut self, path: &str) -> Result<(), Box<dyn std::error::Error>> {
        let lua = self.lua.lock().unwrap();
        let code = std::fs::read_to_string(path)?;
        lua.load(&code).set_name(path).exec()?;

        let mut scripts = self.loaded_scripts.lock().unwrap();
        scripts.insert(path.to_string(), ());

        Ok(())
    }

    /// Call a script function with entity context
    pub fn call_entity_callback(
        &self,
        function: &str,
        entity_data: EntityScriptContext,
    ) -> Result<(), Box<dyn std::error::Error>> {
        let lua = self.lua.lock().unwrap();

        // Create entity table
        let entity = lua.create_table()?;

        // Stats
        entity.set("hp", entity_data.stats.hp)?;
        entity.set("max_hp", entity_data.stats.max_hp)?;
        entity.set("mp", entity_data.stats.mp)?;
        entity.set("max_mp", entity_data.stats.max_mp)?;
        entity.set("attack", entity_data.stats.attack)?;
        entity.set("defense", entity_data.stats.defense)?;
        entity.set("speed", entity_data.stats.speed)?;
        entity.set("level", entity_data.stats.level)?;

        // Colors
        let color_table = lua.create_table()?;
        color_table.set("r", entity_data.current_color[0])?;
        color_table.set("g", entity_data.current_color[1])?;
        color_table.set("b", entity_data.current_color[2])?;
        entity.set("color", color_table)?;

        // Timers
        let timers = lua.create_table()?;
        timers.set(1, entity_data.timers.0)?;
        timers.set(2, entity_data.timers.1)?;
        timers.set(3, entity_data.timers.2)?;
        entity.set("timers", timers)?;

        // Battle context
        entity.set("alive_enemies", entity_data.alive_enemies)?;
        entity.set("alive_allies", entity_data.alive_allies)?;
        entity.set("turn_number", entity_data.turn_number)?;

        // Call the function
        let func: LuaFunction = lua.globals().get(function)?;
        func.call::<()>(entity)?;

        Ok(())
    }

    /// Register all script API functions
    fn register_api(lua: &Lua) -> LuaResult<()> {
        let globals = lua.globals();

        // Entity modification functions
        globals.set(
            "damage",
            lua.create_function(|_, (entity, amount): (LuaTable, u32)| {
                let hp: u32 = entity.get("hp")?;
                entity.set("hp", hp.saturating_sub(amount))?;
                Ok(())
            })?,
        )?;

        globals.set(
            "heal",
            lua.create_function(|_, (entity, amount): (LuaTable, u32)| {
                let hp: u32 = entity.get("hp")?;
                let max_hp: u32 = entity.get("max_hp")?;
                entity.set("hp", (hp + amount).min(max_hp))?;
                Ok(())
            })?,
        )?;

        globals.set(
            "set_hp",
            lua.create_function(|_, (entity, hp): (LuaTable, u32)| {
                entity.set("hp", hp)?;
                Ok(())
            })?,
        )?;

        // Color control (PSX RGB system: 0-0x3fc0)
        globals.set(
            "set_color_target",
            lua.create_function(|_, (entity, r, g, b): (LuaTable, u16, u16, u16)| {
                let color: LuaTable = entity.get("color")?;
                color.set("target_r", r)?;
                color.set("target_g", g)?;
                color.set("target_b", b)?;
                Ok(())
            })?,
        )?;

        // Timer control
        globals.set(
            "set_timer",
            lua.create_function(|_, (entity, timer_id, value): (LuaTable, i32, i16)| {
                let timers: LuaTable = entity.get("timers")?;
                timers.set(timer_id, value)?;
                Ok(())
            })?,
        )?;

        globals.set(
            "get_timer",
            lua.create_function(|_, (entity, timer_id): (LuaTable, i32)| {
                let timers: LuaTable = entity.get("timers")?;
                let value: i16 = timers.get(timer_id)?;
                Ok(value)
            })?,
        )?;

        // Random functions for AI
        globals.set(
            "random",
            lua.create_function(|_, ()| {
                use rand::Rng;
                Ok(rand::thread_rng().gen::<f64>())
            })?,
        )?;

        globals.set(
            "random_range",
            lua.create_function(|_, (min, max): (i64, i64)| {
                use rand::Rng;
                Ok(rand::thread_rng().gen_range(min..=max))
            })?,
        )?;

        // Damage calculation helpers
        globals.set(
            "calculate_physical_damage",
            lua.create_function(|_, (atk, def, level): (i64, i64, i64)| {
                let base_damage = (atk * level) / 10;
                let defense_reduction = def / 2;
                Ok((base_damage - defense_reduction).max(1))
            })?,
        )?;

        globals.set(
            "calculate_art_damage",
            lua.create_function(|_, (atk, power, def, level): (i64, i64, i64, i64)| {
                let base_damage = (atk * power * level) / 100;
                let defense_reduction = (def * 7) / 10; // Arts ignore 30% defense
                Ok((base_damage - defense_reduction).max(1))
            })?,
        )?;

        Ok(())
    }
}

/// Script context passed to entity callbacks
/// Contains all data the script needs to make decisions
#[derive(Debug, Clone)]
pub struct EntityScriptContext {
    /// Entity's combat stats
    pub stats: ScriptStats,

    /// Current RGB color (PSX format: 0-0x3fc0)
    pub current_color: [u16; 3],

    /// Target RGB color
    pub target_color: [u16; 3],

    /// Animation timers (timer_1, timer_2, timer_3)
    pub timers: (i16, i16, i16),

    /// Battle context
    pub alive_enemies: usize,
    pub alive_allies: usize,
    pub turn_number: u32,
}

#[derive(Debug, Clone)]
pub struct ScriptStats {
    pub hp: u32,
    pub max_hp: u32,
    pub mp: u32,
    pub max_mp: u32,
    pub attack: u32,
    pub defense: u32,
    pub speed: u32,
    pub level: u32,
}

impl From<&CombatStats> for ScriptStats {
    fn from(stats: &CombatStats) -> Self {
        Self {
            hp: stats.hp,
            max_hp: stats.max_hp,
            mp: stats.mp,
            max_mp: stats.max_mp,
            attack: stats.attack,
            defense: stats.defense,
            speed: stats.speed,
            level: stats.level,
        }
    }
}
