// Damage Formula Scripts
// These define how damage is calculated for different attack types

// Basic physical attack formula
fn physical_attack(atk, def, atk_level) {
    let base = calculate_physical_damage(atk, def, atk_level);
    return apply_random_variance(base);
}

// Critical hit (double damage)
fn critical_hit(atk, def, atk_level) {
    let base = calculate_physical_damage(atk, def, atk_level);
    return apply_random_variance(base * 2);
}

// Art attack (special move)
fn art_attack(atk, power, def, atk_level) {
    let base = calculate_art_damage(atk, power, def, atk_level);
    return apply_random_variance(base);
}

// Miracle Art (ultimate attack, ignores defense)
fn miracle_art(atk, power, atk_level) {
    let base = (atk * power * atk_level) / 50;
    return apply_random_variance(base);
}

// Fire elemental attack (1.5x vs weak elements)
fn fire_attack(atk, power, def, atk_level, target_element) {
    let base = calculate_art_damage(atk, power, def, atk_level);
    
    // Apply elemental multiplier
    if target_element == "water" {
        base = base / 2; // Weak against water
    } else if target_element == "earth" {
        base = base * 3 / 2; // Strong against earth
    }
    
    return apply_random_variance(base);
}

// Healing formula
fn heal_spell(caster_level, spell_power, target_max_hp) {
    let base_heal = (spell_power * caster_level) / 5;
    let capped_heal = base_heal.min(target_max_hp);
    return capped_heal;
}

// Status effect chance formula
fn status_effect_chance(caster_level, target_level, base_chance) {
    let level_diff = caster_level - target_level;
    let chance = base_chance + (level_diff * 5);
    return chance.clamp(5, 95); // 5-95% range
}

// Counter attack (defensive ability)
fn counter_damage(defender_atk, defender_level, incoming_damage) {
    // Counter does 50% of incoming damage back to attacker
    let counter = incoming_damage / 2;
    return counter.min(defender_atk * defender_level / 10);
}
