-- Damage Formula Scripts
-- These define how damage is calculated for different attack types

-- Basic physical attack formula
function physical_attack(atk, def, atk_level)
    local base = calculate_physical_damage(atk, def, atk_level)
    return apply_random_variance(base)
end

-- Critical hit (double damage)
function critical_hit(atk, def, atk_level)
    local base = calculate_physical_damage(atk, def, atk_level)
    return apply_random_variance(base * 2)
end

-- Art attack (special move)
function art_attack(atk, power, def, atk_level)
    local base = calculate_art_damage(atk, power, def, atk_level)
    return apply_random_variance(base)
end

-- Miracle Art (ultimate attack, ignores defense)
function miracle_art(atk, power, atk_level)
    local base = math.floor((atk * power * atk_level) / 50)
    return apply_random_variance(base)
end

-- Fire elemental attack (1.5x vs weak elements)
function fire_attack(atk, power, def, atk_level, target_element)
    local base = calculate_art_damage(atk, power, def, atk_level)
    
    -- Apply elemental multiplier
    if target_element == "water" then
        base = math.floor(base / 2) -- Weak against water
    elseif target_element == "earth" then
        base = math.floor(base * 3 / 2) -- Strong against earth
    end
    
    return apply_random_variance(base)
end

-- Healing formula
function heal_spell(caster_level, spell_power, target_max_hp)
    local base_heal = math.floor((spell_power * caster_level) / 5)
    local capped_heal = math.min(base_heal, target_max_hp)
    return capped_heal
end

-- Status effect chance formula
function status_effect_chance(caster_level, target_level, base_chance)
    local level_diff = caster_level - target_level
    local chance = base_chance + (level_diff * 5)
    -- Clamp to 5-95% range
    return math.max(5, math.min(95, chance))
end

-- Counter attack (defensive ability)
function counter_damage(defender_atk, defender_level, incoming_damage)
    -- Counter does 50% of incoming damage back to attacker
    local counter = math.floor(incoming_damage / 2)
    local max_counter = math.floor(defender_atk * defender_level / 10)
    return math.min(counter, max_counter)
end
