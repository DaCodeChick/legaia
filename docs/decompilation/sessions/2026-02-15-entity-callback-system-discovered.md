# Entity Callback System Discovery - 2026-02-15

## Summary

**MAJOR BREAKTHROUGH**: Discovered the entity update callback system that executes combat logic per-frame. The combat AI, damage calculations, and turn management are NOT in monolithic functions but are distributed across entity-specific update callbacks stored as function pointers.

## Key Discoveries

### 1. Entity Update Callback Mechanism

**Function**: `update_entity_list_logic` @ 0x8002519c (327 bytes)

**How It Works**:
```c
void update_entity_list_logic(entity_t *entity_list_head) {
    entity_t *current = entity_list_head;
    while (current != NULL) {
        // Call the entity's update callback at offset +0xc
        void (*update_func)(entity_t*) = current->update_func; // offset +0xc
        if (update_func != NULL) {
            update_func(current);
        }
        current = current->next; // offset +0x0
    }
}
```

### 2. Per-Frame Execution Flow

```
main @ 0x80016194
└─ state_handler_1_main_game_loop @ 0x80025eec (state 0x15 = battle)
   ├─ prepare_frame_render @ 0x8001698c
   │  └─ update_controller_input @ 0x8001822c
   ├─ debug_frame_diagnostics @ 0x80016444
   │  ├─ update_entity_list_logic(g_battle_data_ptr)     ← PRIMARY GAME LOGIC
   │  │  └─ entity->update_func(entity)                  ← COMBAT AI/DAMAGE HERE
   │  ├─ update_entity_list_logic(DAT_8007c350)          ← Called 5x per frame
   │  ├─ update_entity_list_logic(DAT_8007c354/c35c)     ← Different entity lists
   │  ├─ update_entity_list_logic(DAT_8007c36c)
   │  ├─ check_battle_timer_and_render()                 ← Rendering
   │  ├─ render_entity_list(g_battle_data_ptr)           ← Called 5x per frame
   │  ├─ render_entity_list(DAT_8007c350)
   │  └─ render_entity_list(DAT_8007c36c)
   └─ render_and_display_frame @ 0x80016b6c
```

**Critical Insight**: Update logic runs BEFORE rendering in the same frame.

### 3. Entity Structure Layout

```c
typedef struct entity_t {
    struct entity_t *next;           // +0x00: Linked list pointer
    // ... unknown fields ...
    void (*update_func)(entity_t*);  // +0x0c: Update callback (entity[3])
    uint32_t flags;                  // +0x10: Entity flags
    // +0x14-0x43: State/position data
    void *model_data;                // +0x44: Model data pointer
    // +0x5a: render type (1-11)
    // +0x62: entity flags
    // +0x6c: animation frame count
    // ... more fields ...
} entity_t;
```

### 4. Entity Type Descriptor Table

**Location**: 0x8007062c - 0x80070700 (multiple 24-byte structures)

**Structure Format**:
```c
typedef struct entity_type_descriptor_t {
    uint16_t type_id;          // +0x00: Entity type (0x0015, 0x0000, etc.)
    uint16_t unk_02;           // +0x02
    uint16_t unk_04;           // +0x04
    uint16_t marker;           // +0x06: Always 0xffff
    void (*callback)(void*);   // +0x08: Update callback function pointer
    uint32_t flags;            // +0x0c: Entity flags/type
    uint32_t unk_10;           // +0x10
    uint32_t count_or_index;   // +0x14
} entity_type_descriptor_t;
```

**Discovered Entity Type Descriptors**:

| Address | Type ID | Callback Address | Callback Name | Flags | Location |
|---------|---------|------------------|---------------|-------|----------|
| 0x8007062c | 0x0015 | 0x80021df4 | (not yet analyzed) | 0x00040082 | CODE |
| 0x80070644 | 0x0015 | 0x801d820c | (dynamically loaded) | 0x00000080 | RAM |
| 0x8007065c | 0x0000 | 0x801f159c | (dynamically loaded) | 0x00000000 | RAM |
| 0x80070674 | 0x0000 | 0x80025000 | entity_callback_render_update | 0x00000000 | CODE |
| 0x8007068c | 0x0000 | 0x8002174c | (not yet analyzed) | 0x00000000 | CODE |
| 0x800706a4 | 0x0000 | 0x80025000 | entity_callback_render_update | 0x00000002 | CODE |
| 0x800706bc | 0x0000 | 0x80024190 | (not yet analyzed) | 0x00000002 | CODE |
| 0x800706d4 | 0x0000 | 0x801e36a0 | (dynamically loaded) | 0x00000002 | RAM |
| 0x800706ec | 0x0000 | 0x80025044 | entity_callback_noop | 0x00001080 | CODE |

Additional callbacks found:
- 0x80070604: callback=0x801d1344 (RAM - dynamically loaded)
- 0x8007061c: callback=0x80025054 → entity_callback_sprite_update (CODE)

### 5. Entity Callback Function Assignment

**Flow**:
1. `create_battle_effect_entity` @ 0x80021b04 is called with entity type parameters
2. It calls `init_battle_graphics((undefined2 *)&DAT_8007062c, ...)` with descriptor table
3. `init_battle_graphics` @ 0x80020de0 allocates entity via `allocate_entity_from_pool`
4. **Critical line**: `piVar3[3] = iVar4;` where `iVar4 = *(int *)(param_1 + 4);`
   - This assigns the callback function pointer from descriptor table offset +8 to entity offset +0xc
5. Entity is added to linked list
6. Per-frame, `update_entity_list_logic` calls `entity->update_func(entity)`

### 6. Renamed Functions (DICK Methodology Applied)

| Old Name | New Name | Address | Purpose |
|----------|----------|---------|---------|
| FUN_80020c14 | update_entity_animation_interpolation | 0x80020c14 | Interpolates entity animation over time using vsync frames |
| FUN_80025000 | entity_callback_render_update | 0x80025000 | Entity callback: updates animation and triggers rendering |
| FUN_80025044 | entity_callback_noop | 0x80025044 | Empty callback (just returns) |
| FUN_80025054 | entity_callback_sprite_update | 0x80025054 | Entity callback: updates sprite animations |
| FUN_80021b04 | create_battle_effect_entity | 0x80021b04 | Creates a new battle effect entity with callback |
| FUN_80020454 | allocate_entity_from_pool | 0x80020454 | Allocates entity from memory pool |

### 7. Key Helper Functions

**update_entity_animation_interpolation** @ 0x80020c14:
- Interpolates entity position/animation using frame timing
- Uses `g_vsync_frames_target` for frame-rate-independent timing
- Returns -1 if waiting, or computed color value (RGB packed in bytes)
- Updates entity offsets: +0x7c-0x82 (current values), +0x84-0x8a (target values), +0x8c-0x92 (velocities)
- Checks timer at +0x98, +0x9c, +0x9a
- Sets flags at +0x62 and +0x10

**entity_callback_render_update** @ 0x80025000:
```c
void entity_callback_render_update(entity_t *entity) {
    uint result = update_entity_animation_interpolation(entity);
    if (result != 0xffffffff) {
        FUN_80024ee4(entity->field_0x9e, entity->field_0x94, result);
    }
}
```

**entity_callback_sprite_update** @ 0x80025054:
```c
void entity_callback_sprite_update(entity_t *entity, ...) {
    FUN_80025344(entity);  // Update sprite state
    int opacity = FUN_80019278(entity);  // Get opacity/alpha
    entity->field_0x16 = (short)opacity;
    FUN_800204f8(entity, param_2, param_3, param_4);  // Render/update
}
```

## Next Steps

### Immediate Priorities

1. **Analyze Static Callback Functions** ⭐⭐⭐
   - 0x80021df4 (type 0x0015 - might be player/enemy character)
   - 0x8002174c (type 0x0000 - battle effect)
   - 0x80024190 (type 0x0000 - battle effect)

2. **Apply Full DICK to Callback Functions** ⭐⭐⭐
   - Rename all parameters, local variables, and referenced symbols
   - Document entity structure offsets discovered
   - Map out what each callback does

3. **Find Combat Logic Callbacks** ⭐⭐⭐
   - The dynamically loaded callbacks (0x801d820c, 0x801f159c, etc.) are likely in PROT.DAT
   - Need to extract and disassemble overlays from PROT.DAT
   - These overlays probably contain:
     - Player character combat AI (input handling, art selection)
     - Enemy AI decision-making
     - Damage calculation formulas
     - Turn management

4. **Document Entity Structure** ⭐⭐
   - Create complete struct definition based on discovered offsets
   - Map all entity types and their purposes
   - Document entity flags and their meanings

5. **Extract Battle Overlays from PROT.DAT** ⭐⭐
   - Search for code overlays that contain combat callback functions
   - Addresses in RAM segment (0x8007b800+) are loaded at runtime
   - These contain the ACTUAL combat logic we're looking for

### Strategy for Finding Combat Logic

The actual combat AI and damage formulas are likely in the **dynamically loaded callbacks**:
- 0x801d1344, 0x801d820c (in RAM @ 0x801d0000-0x801dffff range)
- 0x801f159c, 0x801e36a0 (in RAM @ 0x801e0000-0x801fffff range)

These code sections are loaded from PROT.DAT during battle initialization. We need to:
1. Find which files in PROT.DAT contain executable code
2. Determine the load addresses for each overlay
3. Import them into Ghidra at the correct addresses
4. Analyze the combat callback functions

## Architecture Insights for Bevy Rewrite

### Entity-Component-System Mapping

The PSX uses an OOP-style entity system with function pointers. In Bevy, we should convert this to:

**Entity Types → Bevy Components**:
```rust
#[derive(Component)]
enum EntityType {
    PlayerCharacter,
    Enemy,
    BattleEffect,
    DamageNumber,
    // etc.
}

#[derive(Component)]
struct EntityAnimation {
    current: Vec3,
    target: Vec3,
    velocity: Vec3,
    timer: u16,
}
```

**Update Callbacks → Bevy Systems**:
```rust
// Instead of function pointers, use Bevy systems with queries
fn update_character_combat_ai(
    mut query: Query<(&mut Character, &EntityAnimation), With<CombatActive>>,
    input: Res<Input<KeyCode>>,
) {
    for (mut character, animation) in query.iter_mut() {
        // Player/enemy AI logic here
    }
}

fn update_entity_animation(
    mut query: Query<&mut EntityAnimation>,
    time: Res<Time>,
) {
    for mut anim in query.iter_mut() {
        // Interpolation logic (from update_entity_animation_interpolation)
        anim.current += anim.velocity * time.delta_seconds();
    }
}
```

**System Ordering** (matches PSX execution order):
```rust
app.add_systems(Update, (
    update_input,
    update_combat_ai,        // PSX: update_entity_list_logic
    update_animations,       // PSX: update_entity_animation_interpolation
    update_sprite_state,     // PSX: entity_callback_sprite_update
    render_entities,         // PSX: render_entity_list
).chain());
```

### Key Architectural Differences

**PSX Approach**:
- Single `entity_t` struct with function pointer for polymorphism
- Manual memory management with entity pools
- Explicit linked lists for iteration
- Update callbacks called per-frame via function pointers

**Bevy Approach** (recommended):
- Components for entity data (no inheritance)
- Systems for behavior (no callbacks)
- Automatic memory management
- ECS queries for iteration
- Schedule-based system ordering

This discovery confirms that **combat logic is scriptable** - the callbacks are just function pointers that could easily be replaced with Lua/Rhai script references!

## Files Modified

- None yet (only Ghidra database changes)

## Ghidra Symbols Renamed

6 functions renamed:
1. update_entity_animation_interpolation
2. entity_callback_render_update
3. entity_callback_noop
4. entity_callback_sprite_update
5. create_battle_effect_entity
6. allocate_entity_from_pool

## References

- `update_entity_list_logic` @ 0x8002519c (docs/decompilation/sessions/2026-02-15-combat-logic-search.md)
- Entity type descriptor table @ 0x8007062c
- `init_battle_graphics` @ 0x80020de0
- `debug_frame_diagnostics` @ 0x80016444
