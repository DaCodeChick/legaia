# Legend of Legaia Combat System

Data-driven combat system using Rhai scripts for AI, formulas, and behaviors. Based on reverse-engineered entity callback system from the original PSX game.

## Architecture

The combat system is built around the **entity callback pattern** discovered in the PSX code:

1. Each combat entity (player, enemy, effect) has a `ScriptCallback` component
2. Every frame, the `update_entity_callbacks` system calls the entity's script function
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

### Entity Query Functions

```rust
get_hp() -> i64               // Current HP
get_max_hp() -> i64           // Maximum HP
get_mp() -> i64               // Current MP
get_attack() -> i64           // Attack stat
get_defense() -> i64          // Defense stat
get_speed() -> i64            // Speed stat
```

### Entity Modification Functions

```rust
set_hp(hp: i64)                    // Set HP directly
damage(amount: i64)                // Deal damage
heal(amount: i64)                  // Heal HP
```

### Color Functions (PSX RGB System)

The PSX used a 0-0x3fc0 range for RGB values:

```rust
set_color_target(r: i64, g: i64, b: i64)   // Set target color (0-0x3fc0)
get_current_color() -> [i64; 3]             // Get current RGB
```

Colors interpolate automatically based on velocity. Use for:
- Damage flash (red)
- Status effects (purple for poison, blue for frozen, etc.)
- Phase transitions (boss color changes)

### Timer Functions

Three timers available (matches PSX offsets +0x98, +0x9a, +0x9c):

```rust
set_timer(timer_id: i64, value: i64)   // Set timer (1, 2, or 3)
get_timer(timer_id: i64) -> i64        // Get timer value
```

**Timer Behaviors:**
- **Timer 1**: General purpose animation timer
- **Timer 2**: Auto-destruction timer (entity despawns when reaches 0)
- **Timer 3**: Animation completion timer

### Battle State Queries

```rust
count_alive_enemies() -> i64     // Number of alive enemies
count_alive_allies() -> i64      // Number of alive player characters
```

### Random Functions

```rust
random() -> f64                        // Random 0.0-1.0
random_range(min: i64, max: i64) -> i64  // Random integer in range
```

### Damage Calculation Functions

```rust
calculate_physical_damage(atk: i64, def: i64, level: i64) -> i64
calculate_art_damage(atk: i64, power: i64, def: i64, level: i64) -> i64
apply_defense(damage: i64, defense: i64) -> i64
apply_random_variance(damage: i64) -> i64  // ±5% variance
```

## Example: Simple Enemy AI

```rust
// enemy_slime.rhai
fn on_update() {
    let hp_percent = get_hp() / get_max_hp();
    
    if hp_percent < 0.3 {
        // Low HP - defend
        return "defend";
    }
    
    // Normal - attack
    return "attack";
}

fn on_damage_taken(damage_amount) {
    set_color_target(0x3fc0, 0, 0);  // Flash red
    set_timer(1, 15);                 // For 15 frames
}
```

## Example: Boss AI with Phases

See `scripts/combat/boss_example.rhai` for a complete example showing:
- HP-based phase transitions
- Different AI per phase
- Special attack patterns
- Visual feedback via color changes

## Damage Formulas

Damage calculations are scriptable. See `scripts/combat/damage_formulas.rhai` for examples:

```rust
// Custom damage formula
fn my_attack(atk, power, def, level) {
    let base = (atk * power * level) / 100;
    let after_defense = apply_defense(base, def);
    return apply_random_variance(after_defense);
}
```

## Integration with Bevy

Add the combat plugin to your Bevy app:

```rust
use legaia_combat::CombatPlugin;

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_plugins(CombatPlugin)
        .run();
}
```

Spawn a combat entity:

```rust
commands.spawn((
    CombatEntity {
        flags: 0,
        anim_flags: 0,
        render_priority: 100,
    },
    CombatStats {
        hp: 100,
        max_hp: 100,
        mp: 50,
        max_mp: 50,
        attack: 25,
        defense: 15,
        speed: 20,
        level: 5,
    },
    ScriptCallback {
        script_path: "scripts/combat/enemy_slime.rhai".to_string(),
        function: "on_update".to_string(),
    },
    ColorInterpolation {
        current: Color::WHITE,
        target: Color::WHITE,
        velocity: Vec3::ZERO,
    },
    AnimationTimers {
        timer_1: 0,
        timer_2: -1,  // -1 = never despawn
        timer_3: 0,
    },
    TurnState::Waiting,
    ActionQueue { actions: vec![] },
));
```

## PSX Compatibility Notes

The system closely mirrors the PSX implementation:

| PSX System | Bevy Equivalent |
|------------|-----------------|
| `entity_t` structure | `CombatEntity` + components |
| Function pointer at +0xc | `ScriptCallback` component |
| RGB values (0-0x3fc0) | `ColorInterpolation` with scaling |
| Timers (+0x98, +0x9a, +0x9c) | `AnimationTimers` component |
| `update_entity_list_logic()` | `update_entity_callbacks()` system |
| `update_entity_animation_interpolation()` | `update_color_interpolation()` system |

## Performance Considerations

- Scripts are compiled once and cached
- Entity callbacks run every frame (60fps target)
- Keep AI logic simple in `on_update()` to maintain 60fps
- Use timers to avoid per-frame calculations
- Expensive operations (pathfinding, etc.) should be async

## Future Enhancements

- [ ] Hot-reload scripts during development
- [ ] Script debugging tools
- [ ] Visual script editor
- [ ] More battle state queries (target selection helpers)
- [ ] Animation event triggers
- [ ] Particle effect spawning from scripts
