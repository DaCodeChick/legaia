//! Script API and Engine Integration
//!
//! Provides Rhai script bindings for combat AI and formulas

use bevy::prelude::*;
use rhai::{Dynamic, Engine, EvalAltResult, Scope, AST};
use std::collections::HashMap;
use std::sync::Arc;

use crate::components::*;

/// Script engine resource
#[derive(Resource)]
pub struct ScriptEngine {
    engine: Engine,
    scripts: HashMap<String, Arc<AST>>,
}

impl Default for ScriptEngine {
    fn default() -> Self {
        Self::new()
    }
}

impl ScriptEngine {
    pub fn new() -> Self {
        let mut engine = Engine::new();

        // Register combat API functions
        Self::register_api(&mut engine);

        Self {
            engine,
            scripts: HashMap::new(),
        }
    }

    /// Load a script from file and compile it
    pub fn load_script(&mut self, path: &str) -> Result<(), Box<EvalAltResult>> {
        let ast = self.engine.compile_file(path.into())?;
        self.scripts.insert(path.to_string(), Arc::new(ast));
        Ok(())
    }

    /// Call a script function with entity context
    pub fn call_entity_callback(
        &self,
        script_path: &str,
        function: &str,
        entity_data: EntityScriptContext,
    ) -> Result<Dynamic, Box<EvalAltResult>> {
        let ast = self
            .scripts
            .get(script_path)
            .ok_or_else(|| format!("Script not loaded: {}", script_path))?;

        let mut scope = Scope::new();
        scope.push("entity", entity_data);

        self.engine.call_fn(&mut scope, ast, function, ())
    }

    /// Register all script API functions
    fn register_api(engine: &mut Engine) {
        // Entity query functions
        engine.register_fn("get_hp", |ctx: &mut EntityScriptContext| ctx.stats.hp);
        engine.register_fn("get_max_hp", |ctx: &mut EntityScriptContext| {
            ctx.stats.max_hp
        });
        engine.register_fn("get_mp", |ctx: &mut EntityScriptContext| ctx.stats.mp);
        engine.register_fn("get_attack", |ctx: &mut EntityScriptContext| {
            ctx.stats.attack
        });
        engine.register_fn("get_defense", |ctx: &mut EntityScriptContext| {
            ctx.stats.defense
        });
        engine.register_fn("get_speed", |ctx: &mut EntityScriptContext| ctx.stats.speed);

        // Entity modification functions
        engine.register_fn("set_hp", |ctx: &mut EntityScriptContext, hp: i64| {
            ctx.stats.hp = hp.max(0) as u32;
        });

        engine.register_fn("damage", |ctx: &mut EntityScriptContext, amount: i64| {
            ctx.stats.hp = ctx.stats.hp.saturating_sub(amount as u32);
        });

        engine.register_fn("heal", |ctx: &mut EntityScriptContext, amount: i64| {
            ctx.stats.hp = (ctx.stats.hp + amount as u32).min(ctx.stats.max_hp);
        });

        // Color interpolation (matching PSX RGB system)
        engine.register_fn(
            "set_color_target",
            |ctx: &mut EntityScriptContext, r: i64, g: i64, b: i64| {
                ctx.target_color = [r as u16, g as u16, b as u16];
            },
        );

        engine.register_fn("get_current_color", |ctx: &EntityScriptContext| {
            rhai::Dynamic::from(ctx.current_color.to_vec())
        });

        // Timer control
        engine.register_fn(
            "set_timer",
            |ctx: &mut EntityScriptContext, timer_id: i64, value: i64| match timer_id {
                1 => ctx.timers.0 = value as i16,
                2 => ctx.timers.1 = value as i16,
                3 => ctx.timers.2 = value as i16,
                _ => {}
            },
        );

        engine.register_fn(
            "get_timer",
            |ctx: &EntityScriptContext, timer_id: i64| match timer_id {
                1 => ctx.timers.0 as i64,
                2 => ctx.timers.1 as i64,
                3 => ctx.timers.2 as i64,
                _ => 0,
            },
        );

        // Random functions for AI
        engine.register_fn("random", || {
            use rand::Rng;
            rand::thread_rng().gen::<f64>()
        });

        engine.register_fn("random_range", |min: i64, max: i64| {
            use rand::Rng;
            rand::thread_rng().gen_range(min..=max)
        });

        // Battle state queries
        engine.register_fn("count_alive_enemies", |ctx: &EntityScriptContext| {
            ctx.alive_enemies as i64
        });

        engine.register_fn("count_alive_allies", |ctx: &EntityScriptContext| {
            ctx.alive_allies as i64
        });
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
