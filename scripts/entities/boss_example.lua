-- Enemy AI: Boss (Advanced enemy with multiple phases)
-- Demonstrates complex AI based on PSX callback patterns

-- Global state (persisted between calls)
local phase = 1
local turn_count = 0
local used_special = false

-- Called every frame to update boss behavior
function on_update(entity)
    turn_count = turn_count + 1
    
    local hp_percent = entity.hp / entity.max_hp
    local alive_enemies = entity.alive_enemies
    
    -- Phase transitions based on HP
    if hp_percent < 0.25 and phase < 3 then
        phase = 3
        set_color_target(entity, 0x3fc0, 0, 0x3fc0) -- Purple (enraged)
        return "special_ultimate"
    elseif hp_percent < 0.5 and phase < 2 then
        phase = 2
        set_color_target(entity, 0x3fc0, 0x2000, 0) -- Orange (phase 2)
        return "special_heal"
    end
    
    -- Phase 3 - Aggressive (below 25% HP)
    if phase == 3 then
        if turn_count % 3 == 0 then
            return "special_multi_attack"
        end
        return "attack_strong"
    end
    
    -- Phase 2 - Balanced (25-50% HP)
    if phase == 2 then
        if alive_enemies <= 1 and random() < 0.3 then
            return "special_summon"
        end
        
        if turn_count % 4 == 0 then
            return "special_attack"
        end
        
        return (random() < 0.6) and "attack" or "defend"
    end
    
    -- Phase 1 - Normal (above 50% HP)
    if turn_count % 5 == 0 and not used_special then
        used_special = true
        return "special_attack"
    end
    
    return "attack"
end

-- Choose target with priority logic
function choose_target(entity)
    local alive_allies = entity.alive_allies
    
    if phase == 3 then
        -- In phase 3, target lowest HP ally
        return find_weakest_target(entity)
    end
    
    -- Otherwise random
    return random_range(0, alive_allies - 1)
end

-- Helper function to find weakest target
function find_weakest_target(entity)
    -- TODO: This would need engine support to query all targets
    -- For now, just random
    return random_range(0, entity.alive_allies - 1)
end

-- Visual feedback when taking damage
function on_damage_taken(entity, damage_amount)
    -- More dramatic flash in later phases
    if phase == 3 then
        set_color_target(entity, 0x3fc0, 0, 0) -- Bright red
        set_timer(entity, 1, 30) -- Longer flash
    else
        set_color_target(entity, 0x3000, 0, 0) -- Normal red
        set_timer(entity, 1, 15)
    end
end

-- Animation updates
function on_animation_update(entity)
    local timer = get_timer(entity, 1)
    
    if timer > 0 then
        -- Flashing from damage
    else
        -- Restore phase color
        if phase == 3 then
            set_color_target(entity, 0x3fc0, 0, 0x3fc0) -- Purple
        elseif phase == 2 then
            set_color_target(entity, 0x3fc0, 0x2000, 0) -- Orange
        else
            set_color_target(entity, 0x3fc0, 0x3fc0, 0x3fc0) -- White
        end
    end
end
