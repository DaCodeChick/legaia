# Entity Structure Documentation - 2026-02-15

## Entity Structure (Partial - Discovered Offsets)

Based on analysis of entity callback functions and entity creation code.

```c
typedef struct entity_t {
    // === Linked List (0x00-0x0b) ===
    struct entity_t *next;           // +0x00: Pointer to next entity in list
    uint32_t unk_04;                 // +0x04: Unknown
    uint32_t unk_08;                 // +0x08: Unknown
    
    // === Update Callback (0x0c-0x0f) ===
    void (*update_func)(entity_t*);  // +0x0c: Per-frame update callback function pointer
    
    // === Flags & State (0x10-0x5f) ===
    uint32_t flags;                  // +0x10: Entity flags (bit 3 = 0x8)
    // ... many unknown fields ...
    uint16_t anim_flags;             // +0x62: Animation flags (bit 8 = 0x100)
    // ... unknown fields ...
    
    // === Animation/Color Interpolation (0x7c-0x9f) ===
    // Current RGB color values (interpolated per-frame)
    uint16_t current_r;              // +0x7c: Current red component (0-0x3fc0)
    uint16_t current_g;              // +0x7e: Current green component (0-0x3fc0)
    uint16_t current_b;              // +0x80: Current blue component (0-0x3fc0)
    uint16_t unk_82;                 // +0x82: Alignment/padding
    
    // Target RGB color values (destination for interpolation)
    uint16_t target_r;               // +0x84: Target red component
    uint16_t target_g;               // +0x86: Target green component
    uint16_t target_b;               // +0x88: Target blue component
    uint16_t unk_8a;                 // +0x8a: Alignment/padding
    
    // RGB velocity/delta values (interpolation speed)
    int16_t velocity_r;              // +0x8c: Red velocity (signed)
    int16_t velocity_g;              // +0x8e: Green velocity (signed)
    int16_t velocity_b;              // +0x90: Blue velocity (signed)
    uint16_t unk_92;                 // +0x92: Alignment/padding
    
    uint32_t draw_mode_param;        // +0x94: Draw mode parameter for GPU
    int16_t timer_1;                 // +0x98: Animation timer 1 (decremented by g_vsync_frames_target)
    int16_t timer_2;                 // +0x9a: Animation timer 2 (used for auto-destruction)
    int16_t timer_3;                 // +0x9c: Animation timer 3 (sets flag 0x100 when expires)
    int16_t ot_priority;             // +0x9e: Ordering table depth/priority for rendering
    
    // ... more unknown fields ...
} entity_t;
```

## Animation Interpolation Algorithm

The `update_entity_animation_interpolation` function implements per-frame color interpolation:

```c
int update_entity_animation_interpolation(entity_t *entity) {
    // Check timer_1 - if positive, decrement and wait
    if (entity->timer_1 > 0) {
        entity->timer_1 -= g_vsync_frames_target;
        if (entity->timer_1 > 0) {
            return -1; // Still waiting
        }
    }
    
    // Decrement timer_3
    entity->timer_3 -= g_vsync_frames_target;
    if (entity->timer_3 < 0) {
        entity->anim_flags |= 0x100; // Set animation complete flag
        
        // Check timer_2 for auto-destruction
        if (entity->timer_2 >= 0) {
            entity->timer_2 -= g_vsync_frames_target;
            if (entity->timer_2 < 0) {
                entity->flags |= 0x8; // Mark for destruction
                return -1;
            }
        }
    }
    
    // Interpolate RGB components (3 iterations)
    for (int i = 0; i < 3; i++) {
        uint16_t *current = &entity->current_r + i;
        uint16_t *target = &entity->target_r + i;
        int16_t *velocity = &entity->velocity_r + i;
        
        // Update current value: current += velocity * frame_time
        *current += *velocity * g_vsync_frames_target;
        
        // Clamp to target based on velocity direction
        if (*velocity < 0) {
            if (*current < *target) *current = *target;
        } else {
            if (*current > *target) *current = *target;
        }
        
        // Clamp to valid range [0, 0x3fc0]
        if (*current > 0x3fc0) *current = 0x3fc0;
        if (*current < 0) *current = 0;
    }
    
    // Pack RGB into single int: (R << 16) | (G << 8) | B
    // Divide by 64 to convert from 0x3fc0 range to 0-255 range
    int r = entity->current_b >> 6;  // Blue (note: swapped!)
    int g = entity->current_g >> 6;  // Green
    int b = entity->current_r >> 6;  // Red (note: swapped!)
    
    return (r << 16) | (g << 8) | b;
}
```

**Key Observations**:
- Frame-rate-independent interpolation using `g_vsync_frames_target`
- RGB values stored in 0-0x3fc0 range (16,320 max), divided by 64 to get 0-255
- Velocity can be negative (fade out) or positive (fade in)
- Timers automatically mark entity for destruction when expired
- Returns -1 if waiting, or RGB color value if animation active

## Entity Type Descriptors

Found at **0x8007062c** - array of 24-byte structures:

```c
typedef struct entity_type_descriptor_t {
    uint16_t type_id;          // +0x00: Entity type ID (0x0015, 0x0000, etc.)
    uint16_t unk_02;           // +0x02: Unknown
    uint16_t unk_04;           // +0x04: Unknown
    uint16_t marker;           // +0x06: Always 0xffff (validation marker)
    void (*callback)(void*);   // +0x08: Update callback function pointer
    uint32_t flags;            // +0x0c: Entity initialization flags
    uint32_t unk_10;           // +0x10: Unknown
    uint32_t count;            // +0x14: Count or index
} entity_type_descriptor_t; // 24 bytes (0x18)
```

### Known Entity Type Descriptors

| Address    | Type ID | Callback       | Location | Flags      | Count | Purpose |
|------------|---------|----------------|----------|------------|-------|---------|
| 0x8007062c | 0x0015  | 0x80021df4     | CODE     | 0x00040082 | 5     | Character/combatant? (Ghidra detection issue) |
| 0x80070644 | 0x0015  | 0x801d820c     | RAM      | 0x00000080 | 1     | **Combat AI (dynamically loaded!)** |
| 0x8007065c | 0x0000  | 0x801f159c     | RAM      | 0x00000000 | 0     | **Combat logic (dynamically loaded!)** |
| 0x80070674 | 0x0000  | 0x80025000     | CODE     | 0x00000000 | 0     | Render update (RGB interpolation) |
| 0x8007068c | 0x0000  | 0x8002174c     | CODE     | 0x00000000 | 0     | Unknown (Ghidra detection issue) |
| 0x800706a4 | 0x0000  | 0x80025000     | CODE     | 0x00000002 | 0     | Render update (duplicate) |
| 0x800706bc | 0x0000  | 0x80024190     | CODE     | 0x00000002 | 0     | Unknown (Ghidra detection issue) |
| 0x800706d4 | 0x0000  | 0x801e36a0     | RAM      | 0x00000002 | 0     | **Combat logic (dynamically loaded!)** |
| 0x800706ec | 0x0000  | 0x80025044     | CODE     | 0x00001080 | 0     | No-op callback (empty) |
| 0x80070604 | ?       | 0x801d1344     | RAM      | ?          | ?     | **Combat logic (dynamically loaded!)** |
| 0x8007061c | ?       | 0x80025054     | CODE     | ?          | ?     | Sprite update |

**Critical Insight**: Type 0x0015 appears to be "character/combatant" entities with TWO implementations:
1. Static callback in CODE (0x80021df4) - likely for initialization or basic behavior
2. Dynamic callback in RAM (0x801d820c) - likely for actual combat AI

## Entity Callback Flow

### Per-Frame Execution During Battle

```
main() @ 0x80016194
└─ State Handler 0x15 (Battle) → state_handler_1_main_game_loop @ 0x80025eec
   ├─ prepare_frame_render @ 0x8001698c
   │  └─ update_controller_input @ 0x8001822c
   │
   ├─ debug_frame_diagnostics @ 0x80016444
   │  │
   │  ├─ update_entity_list_logic(g_battle_data_ptr)       ← PRIMARY GAME LOGIC
   │  │  └─ For each entity: entity->update_func(entity)
   │  │     ├─ entity_callback_render_update (0x80025000)
   │  │     ├─ entity_callback_sprite_update (0x80025054)
   │  │     ├─ COMBAT AI CALLBACKS (0x801d820c, 0x801f159c, etc.) ← DYNAMICALLY LOADED
   │  │     └─ entity_callback_noop (0x80025044)
   │  │
   │  ├─ update_entity_list_logic(DAT_8007c350)            ← 4 MORE ENTITY LISTS
   │  ├─ update_entity_list_logic(DAT_8007c354/c35c)
   │  ├─ update_entity_list_logic(DAT_8007c36c)
   │  │
   │  ├─ check_battle_timer_and_render()                   ← RENDERING PHASE
   │  │  └─ per_frame_rendering_coordinator()
   │  │
   │  ├─ render_entity_list(g_battle_data_ptr)             ← RENDER ALL ENTITIES
   │  ├─ render_entity_list(DAT_8007c350)
   │  ├─ render_entity_list(DAT_8007c354/c35c)
   │  └─ render_entity_list(DAT_8007c36c)
   │
   └─ render_and_display_frame @ 0x80016b6c
```

**Key Observation**: Entity update callbacks run BEFORE rendering, ensuring game logic completes before drawing.

## Entity Callback Functions

### Static Callbacks (In CODE Segment)

| Function | Address | Purpose |
|----------|---------|---------|
| `entity_callback_render_update` | 0x80025000 | Updates animation interpolation, sets GPU draw mode |
| `entity_callback_sprite_update` | 0x80025054 | Updates sprite animations and opacity |
| `entity_callback_noop` | 0x80025044 | Empty callback (returns immediately) |
| **UNKNOWN_1** | 0x80021df4 | Type 0x0015 callback (Ghidra didn't detect - needs manual analysis) |
| **UNKNOWN_2** | 0x8002174c | Type 0x0000 callback (Ghidra didn't detect - needs manual analysis) |
| **UNKNOWN_3** | 0x80024190 | Type 0x0000 callback (Ghidra didn't detect - needs manual analysis) |

### Dynamic Callbacks (Loaded to RAM at Runtime)

| Address | Range | Purpose |
|---------|-------|---------|
| 0x801d1344 | 0x801d0000-0x801dffff | **Combat AI (loaded from PROT.DAT)** |
| 0x801d820c | 0x801d0000-0x801dffff | **Combat AI (loaded from PROT.DAT)** |
| 0x801e36a0 | 0x801e0000-0x801effff | **Combat AI (loaded from PROT.DAT)** |
| 0x801f159c | 0x801f0000-0x801fffff | **Combat AI (loaded from PROT.DAT)** |

**CRITICAL**: The actual combat AI, damage formulas, and turn management are in these dynamically loaded callbacks!

## Helper Functions

### `update_entity_animation_interpolation` @ 0x80020c14
- **Purpose**: Interpolates RGB color values per-frame with velocity and clamping
- **Called by**: `entity_callback_render_update`
- **Returns**: RGB color packed as int, or -1 if waiting

### `set_entity_draw_mode_and_add_to_ot` @ 0x80024ee4
- **Purpose**: Sets GPU draw mode and adds primitives to ordering table
- **Parameters**:
  - `ot_priority`: Ordering table depth (from entity +0x9e)
  - `draw_mode_param`: GPU draw mode (from entity +0x94)
  - `rgb_color`: Packed RGB color from interpolation
- **Called by**: `entity_callback_render_update`

### `create_battle_effect_entity` @ 0x80021b04
- **Purpose**: Creates a new battle entity with assigned callback
- **Flow**:
  1. Calls `init_battle_graphics` with entity type descriptor table
  2. `init_battle_graphics` allocates entity via `allocate_entity_from_pool`
  3. **Assigns callback**: `entity[3] = descriptor[2]` (offset +0xc = descriptor +0x8)
  4. Initializes entity fields (position, timers, flags)
  5. Adds entity to linked list

### `allocate_entity_from_pool` @ 0x80020454
- **Purpose**: Allocates entity from memory pool
- **Returns**: Pointer to allocated entity, or NULL if pool exhausted

## Next Steps

1. **Extract Battle Overlays from PROT.DAT** ⭐⭐⭐
   - The combat AI callbacks are in dynamically loaded code
   - Need to identify which files in PROT.DAT contain executable code
   - Determine load addresses (likely 0x801d0000, 0x801e0000, 0x801f0000)
   - Import into Ghidra at correct addresses

2. **Complete Entity Structure**
   - Map remaining unknown offsets
   - Document all entity types and their purposes
   - Create comprehensive struct definition

3. **Analyze Undetected Callbacks**
   - Manually create functions at 0x80021df4, 0x8002174c, 0x80024190 in Ghidra
   - Apply DICK methodology once decompiled
   - Understand type 0x0015 entity behavior

## References

- Session doc: `docs/decompilation/sessions/2026-02-15-entity-callback-system-discovered.md`
- `update_entity_list_logic` @ 0x8002519c
- Entity type descriptor table @ 0x8007062c
- `init_battle_graphics` @ 0x80020de0
