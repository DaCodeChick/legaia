# Legend of Legaia Scripting System

General-purpose Lua scripting system for entities, NPCs, combat, and environments. Based on reverse-engineered entity callback system from the original PSX game.

## Why Lua?

- **Industry standard** - Nearly every moddable game uses Lua
- **Massive ecosystem** - Tons of existing tools, libraries, documentation  
- **Modder familiarity** - Most modders already know Lua
- **Proven track record** - Used successfully in thousands of games

## Architecture

The scripting system is built around the **entity callback pattern** discovered in the PSX code:

1. Each entity (combat, NPC, environment) has a `ScriptCallback` component
2. Every frame, the `update_entity_callbacks` system calls the entity's Lua function
3. Scripts can query entity state, make decisions, and trigger actions
4. Color interpolation, timers, and rendering happen automatically

This matches the PSX execution flow:
```
update_entity_list_logic()
├─ For each entity: entity->update_func(entity)  // Script callbacks
├─ update_color_interpolation()                   // RGB transitions
├─ update_animation_timers()                      // Timer decrements
└─ Rendering systems                              // Visual output
```

## Script API Reference

All scripts receive an `entity` table containing the entity's current state.

### Entity State (Read-only)

```lua
entity.hp           -- Current HP
entity.max_hp       -- Maximum HP
entity.mp           -- Current MP
entity.max_mp       -- Maximum MP
entity.attack       -- Attack stat
entity.defense      -- Defense stat
entity.speed        -- Speed stat
entity.level        -- Entity level

-- Battle context (combat entities only)
entity.alive_enemies  -- Number of alive enemies
entity.alive_allies   -- Number of alive allies
entity.turn_number    -- Current turn number
```

### Entity Modification Functions

```lua
damage(entity, amount)     -- Deal damage to entity
heal(entity, amount)       -- Heal entity HP
set_hp(entity, hp)         -- Set HP directly
```

### Color Functions (PSX RGB System)

The PSX used a 0-0x3fc0 range for RGB values:

```lua
set_color_target(entity, r, g, b)   -- Set target color (0-0x3fc0)
-- Color interpolation happens automatically via velocity
-- entity.color.r, entity.color.g, entity.color.b contain current values
```

### Timer Functions

Three timers are available (matches PSX entity structure):

```lua
set_timer(entity, timer_id, value)   -- Set timer (1, 2, or 3)
get_timer(entity, timer_id)          -- Get timer value
-- Timer 1: General purpose
-- Timer 2: Auto-despawn when reaches 0
-- Timer 3: General purpose
```

### Random Functions

```lua
random()                    -- Random 0.0-1.0
random_range(min, max)      -- Random integer in range (inclusive)
```

### Damage Calculation Functions

```lua
calculate_physical_damage(atk, def, level)      -- Physical damage formula
calculate_art_damage(atk, power, def, level)    -- Art/special damage
apply_defense(damage, defense)                  -- Apply defense reduction
apply_random_variance(damage)                   -- Apply ±5% variance
```

## Example: Simple Enemy AI

```lua
-- scripts/entities/enemy_slime.lua

function on_update(entity)
    local hp_percent = entity.hp / entity.max_hp
    
    -- Low HP behavior - defend
    if hp_percent < 0.3 then
        if random() < 0.5 then
            return "defend"
        end
    end
    
    -- Normal behavior - attack
    return "attack"
end

function on_damage_taken(entity, damage_amount)
    -- Flash red when damaged
    set_color_target(entity, 0x3fc0, 0, 0)  -- Full red
    set_timer(entity, 1, 15)                 -- Flash for 15 frames
end

function on_animation_update(entity)
    local timer = get_timer(entity, 1)
    
    if timer == 0 then
        -- Return to normal color
        set_color_target(entity, 0x3fc0, 0x3fc0, 0x3fc0)  -- White
    end
end
```

## Example: Multi-Phase Boss

```lua
-- scripts/entities/boss_example.lua

local phase = 1
local turn_count = 0

function on_update(entity)
    turn_count = turn_count + 1
    local hp_percent = entity.hp / entity.max_hp
    
    -- Phase transitions
    if hp_percent < 0.25 and phase < 3 then
        phase = 3
        set_color_target(entity, 0x3fc0, 0, 0x3fc0)  -- Purple (enraged)
        return "special_ultimate"
    elseif hp_percent < 0.5 and phase < 2 then
        phase = 2
        set_color_target(entity, 0x3fc0, 0x2000, 0)  -- Orange
        return "special_heal"
    end
    
    -- Phase 3 - Aggressive
    if phase == 3 then
        if turn_count % 3 == 0 then
            return "special_multi_attack"
        end
        return "attack_strong"
    end
    
    -- Phase 2 - Balanced
    if phase == 2 then
        if turn_count % 4 == 0 then
            return "special_attack"
        end
        return (random() < 0.6) and "attack" or "defend"
    end
    
    -- Phase 1 - Normal
    return "attack"
end
```

## Example: Custom Damage Formula

```lua
-- scripts/entities/damage_formulas.lua

function fire_attack(atk, power, def, atk_level, target_element)
    local base = calculate_art_damage(atk, power, def, atk_level)
    
    -- Apply elemental multiplier
    if target_element == "water" then
        base = math.floor(base / 2)       -- Weak against water
    elseif target_element == "earth" then
        base = math.floor(base * 3 / 2)   -- Strong against earth
    end
    
    return apply_random_variance(base)
end
```

## Using in Rust/Bevy Code

```rust
use bevy::prelude::*;
use legaia_scripting::*;

fn setup_entity(mut commands: Commands, mut script_engine: ResMut<ScriptEngine>) {
    // Load Lua scripts
    script_engine
        .load_script("scripts/entities/enemy_slime.lua")
        .expect("Failed to load script");

    // Spawn entity with script callback
    commands.spawn((
        Name::new("Slime"),
        CombatEntity {
            flags: 0,
            anim_flags: 0,
            render_priority: 90,
        },
        CombatStats {
            hp: 50,
            max_hp: 50,
            mp: 0,
            max_mp: 0,
            attack: 15,
            defense: 10,
            speed: 12,
            level: 3,
        },
        ScriptCallback {
            script_path: "scripts/entities/enemy_slime.lua".to_string(),
            function: "on_update".to_string(),
        },
        ColorInterpolation {
            current: Vec3::new(1.0, 1.0, 1.0),
            target: Vec3::new(1.0, 1.0, 1.0),
            velocity: Vec3::ZERO,
        },
        AnimationTimers {
            timer_1: 0,
            timer_2: -1,  // Auto-despawn disabled
            timer_3: 0,
        },
    ));
}
```

## PSX Entity Structure Mapping

The ECS components map directly to the PSX entity structure:

| PSX Offset | Field | ECS Component |
|------------|-------|---------------|
| +0x0c | `update_func` | `ScriptCallback.function` |
| +0x10 | `flags` | `CombatEntity.flags` |
| +0x7c-0x80 | `current_rgb` | `ColorInterpolation.current` |
| +0x84-0x88 | `target_rgb` | `ColorInterpolation.target` |
| +0x8c-0x90 | `velocity_rgb` | `ColorInterpolation.velocity` |
| +0x98 | `timer_1` | `AnimationTimers.timer_1` |
| +0x9a | `timer_2` | `AnimationTimers.timer_2` |
| +0x9c | `timer_3` | `AnimationTimers.timer_3` |
| +0x9e | `ot_priority` | `CombatEntity.render_priority` |

## Use Cases

This scripting system supports:

- **Combat AI** - Enemy behavior, boss phases, attack patterns
- **NPC Behavior** - Dialogue triggers, patrol routes, reactions
- **Environment** - Doors, chests, switches, triggers
- **Cutscenes** - Scripted sequences, camera control
- **Custom Game Modes** - Tutorial, special battles, minigames

## Running Examples

```bash
# Run the simple battle example
cargo run --example simple_battle
```
