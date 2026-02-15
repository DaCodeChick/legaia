-- Enemy AI: Slime (Basic enemy)
-- Based on PSX entity callback system

-- Called every frame to update enemy behavior
function on_update(entity)
    -- Check if it's our turn
    local hp_percent = entity.hp / entity.max_hp
    
    -- Low HP behavior - try to escape or defend
    if hp_percent < 0.3 then
        if random() < 0.5 then
            return "defend"
        end
    end
    
    -- Normal behavior - attack
    return "attack"
end

-- Called when choosing a target
function choose_target(entity)
    local alive_allies = entity.alive_allies
    
    if alive_allies == 1 then
        -- Only one target
        return 0
    end
    
    -- Random target selection
    return random_range(0, alive_allies - 1)
end

-- Called when taking damage
function on_damage_taken(entity, damage_amount)
    -- Flash red when damaged
    set_color_target(entity, 0x3fc0, 0, 0) -- Full red
    set_timer(entity, 1, 15) -- Flash for 15 frames
end

-- Called every frame for visual updates
function on_animation_update(entity)
    local timer = get_timer(entity, 1)
    
    if timer > 0 then
        -- Still flashing red from damage
        -- Color interpolation happens automatically
    else
        -- Return to normal color
        set_color_target(entity, 0x3fc0, 0x3fc0, 0x3fc0) -- White
    end
end
