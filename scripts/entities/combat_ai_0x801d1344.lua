-- Passive Entity AI
--
-- This AI does minimal decision-making. It simply tracks internal state
-- and delegates actual behavior to the engine's update callback system.
-- This pattern is used for scripted events, passive entities, or
-- enemies that rely entirely on animation/timing rather than strategic AI.

-- State tracking
local state_a = 0
local state_b = 0
local match_flag = false

-- Called when entity spawns in battle
function on_spawn(entity, param_a, param_b)
    state_a = param_a or 0
    state_b = param_b or 0
    match_flag = false
end

-- Called every frame during battle
function on_update(entity, turn_param)
    -- Track whether current turn matches our stored state
    match_flag = (state_a == turn_param)
    
    -- This AI doesn't make decisions - it's purely passive
    -- The engine handles actual behavior via standard callbacks
end

-- Called when it's this entity's turn to act
function on_turn(entity)
    if match_flag then
        -- State matches - might trigger special animation or event
        return "wait"
    else
        -- Default behavior - do nothing or basic idle
        return "wait"
    end
end
