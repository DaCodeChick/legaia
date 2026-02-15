//! Damage calculation system
//!
//! Scriptable damage formulas for combat

use crate::components::*;
use rhai::{Engine, EvalAltResult, Scope};

/// Damage formula engine
pub struct DamageEngine {
    engine: Engine,
}

impl DamageEngine {
    pub fn new() -> Self {
        let mut engine = Engine::new();

        // Register damage calculation helpers
        engine.register_fn("calculate_physical_damage", Self::calculate_physical_damage);
        engine.register_fn("calculate_art_damage", Self::calculate_art_damage);
        engine.register_fn("apply_defense", Self::apply_defense);
        engine.register_fn("apply_random_variance", Self::apply_random_variance);

        Self { engine }
    }

    /// Calculate physical attack damage
    /// Formula approximation based on typical JRPG patterns
    pub fn calculate_physical_damage(
        attacker_atk: i64,
        defender_def: i64,
        attacker_level: i64,
    ) -> i64 {
        let base_damage = (attacker_atk * attacker_level) / 10;
        let defense_reduction = defender_def / 2;
        (base_damage - defense_reduction).max(1)
    }

    /// Calculate art (special move) damage
    pub fn calculate_art_damage(
        attacker_atk: i64,
        art_power: i64,
        defender_def: i64,
        attacker_level: i64,
    ) -> i64 {
        let base_damage = (attacker_atk * art_power * attacker_level) / 100;
        let defense_reduction = (defender_def * 7) / 10; // Arts ignore 30% defense
        (base_damage - defense_reduction).max(1)
    }

    /// Apply defense reduction
    pub fn apply_defense(damage: i64, defense: i64) -> i64 {
        let reduction = defense / 2;
        (damage - reduction).max(1)
    }

    /// Apply random variance (typically ±5%)
    pub fn apply_random_variance(damage: i64) -> i64 {
        use rand::Rng;
        let variance = rand::thread_rng().gen_range(-5..=5);
        let variance_amount = (damage * variance) / 100;
        (damage + variance_amount).max(1)
    }

    /// Execute a custom damage formula from script
    pub fn eval_damage_formula(
        &self,
        formula: &str,
        attacker: &CombatStats,
        defender: &CombatStats,
        power: u32,
    ) -> Result<i64, Box<EvalAltResult>> {
        let mut scope = Scope::new();

        // Push attacker stats
        scope.push("atk", attacker.attack as i64);
        scope.push("atk_level", attacker.level as i64);
        scope.push("atk_hp", attacker.hp as i64);
        scope.push("atk_mp", attacker.mp as i64);

        // Push defender stats
        scope.push("def", defender.defense as i64);
        scope.push("def_level", defender.level as i64);
        scope.push("def_hp", defender.hp as i64);
        scope.push("def_max_hp", defender.max_hp as i64);

        // Push power
        scope.push("power", power as i64);

        // Evaluate formula
        self.engine.eval_with_scope::<i64>(&mut scope, formula)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_physical_damage() {
        let damage = DamageEngine::calculate_physical_damage(50, 20, 10);
        assert_eq!(damage, 40); // (50 * 10 / 10) - (20 / 2) = 50 - 10 = 40
    }

    #[test]
    fn test_art_damage() {
        let damage = DamageEngine::calculate_art_damage(50, 150, 20, 10);
        assert_eq!(damage, 736); // (50 * 150 * 10 / 100) - (20 * 7 / 10) = 750 - 14 = 736
    }

    #[test]
    fn test_damage_formula_script() {
        let engine = DamageEngine::new();

        let attacker = CombatStats {
            hp: 100,
            max_hp: 100,
            mp: 50,
            max_mp: 50,
            attack: 50,
            defense: 30,
            speed: 40,
            level: 10,
        };

        let defender = CombatStats {
            hp: 80,
            max_hp: 100,
            mp: 30,
            max_mp: 50,
            attack: 40,
            defense: 20,
            speed: 35,
            level: 8,
        };

        // Test custom formula
        let formula = "calculate_physical_damage(atk, def, atk_level)";
        let result = engine.eval_damage_formula(formula, &attacker, &defender, 100);
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), 40);
    }
}
