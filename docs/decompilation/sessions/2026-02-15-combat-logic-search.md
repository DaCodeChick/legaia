# Combat Logic Search Session - 2026-02-15

## Objective
Locate the main combat logic functions in Legend of Legaia (SCUS_942.54): AI decision-making, damage formulas, turn management, and HP/MP manipulation.

## Background
Previous sessions identified battle initialization (`battle_system_main`) and state machine architecture (12 handlers). However, initialization ≠ combat logic. The actual per-turn combat processing has not been found.

## Search Strategies Completed

### Strategy A: Function Pointer Tables ❌
**Hypothesis**: Combat logic dispatched via function pointers in battle buffers.

**Method**: Examined battle memory structures:
- `g_battle_memory_buffer` @ 0x8007bd3c (31KB buffer, base pointer)
- `g_battle_char_data_array` @ 0x80084140 (4 chars × 1044 bytes)

**Result**: All zeros in static memory (BSS section). Buffers populated at runtime during `battle_system_main`. No function pointers found in static analysis.

**Conclusion**: Combat logic not dispatched via static function pointer tables.

---

### Strategy B: Data Flow Analysis ❌
**Hypothesis**: Trace HP/stat manipulation to find combat formulas.

**Method**: Found cross-references to `g_battle_char_data_array`:
- 13 references total
- All lead to initialization functions (`init_state_environment`, `init_rendering_tables`)
- Functions zero out arrays during setup

**Result**: No HP read/write operations found. References are initialization only.

**Conclusion**: Combat stat manipulation happens in unexplored functions.

---

### Strategy C: Architectural Analysis ✅ (Partial Success)
**Hypothesis**: Map state machine flow to identify where combat logic runs.

**Method**: Traced execution path during battle:
1. State 11 → `state_handler_11_battle_handler` @ 0x800565d8
2. Calls `battle_system_main` @ 0x80055b6c (initialization)
3. Sets `g_current_game_state = 0x15` (21 decimal)
4. State 21 → `state_handler_1_main_game_loop` @ 0x80025eec
5. Main game loop in `main` @ 0x80016194:
   ```c
   while (-1 < g_current_game_state) {
       gte_load_h_register((int)g_frame_counter);
       (*(code *)(&g_state_handler_table)[g_current_game_state * 6])();
   }
   ```

**Per-Frame Call Chain (State 21)**:
```
state_handler_1_main_game_loop @ 0x80025eec
├─ prepare_frame_render @ 0x8001698c
│  └─ update_controller_input @ 0x8001822c
├─ debug_frame_diagnostics @ 0x80016444
│  ├─ FUN_8001d058 @ 0x8001d058 (battle-specific)
│  │  └─ FUN_80026ce4 @ 0x80026ce4
│  │     └─ FUN_80026f50 @ 0x80026f50 (battle camera)
│  └─ FUN_8001d140 @ 0x8001d140 (called 4x with different data)
│     └─ FUN_8001ada4 @ 0x8001ada4 (entity renderer, 2.4KB)
└─ render_and_display_frame @ 0x80016b6c
```

**Result**: Identified per-frame battle rendering chain.

**Conclusion**: Combat logic NOT in state handlers or rendering functions.

---

### Strategy D: Random Number Generation Analysis ❌
**Hypothesis**: Damage formulas use `rand()`, trace callers to find combat logic.

**Method**: Found 5 functions calling `rand()` @ 0x80056798:

| Function | Address | Size | Purpose |
|----------|---------|------|---------|
| `battle_system_main` | 0x80055b6c | 3KB | Battle initialization only |
| `FUN_80038158` | 0x80038158 | 1,819 lines | Animation/sprite controller |
| `FUN_8004998c` | 0x8004998c | - | 3D model vertex interpolation |
| `FUN_80028158` | 0x80028158 | - | Graphics primitive generation |
| `FUN_8003a1e4` | 0x8003a1e4 | - | Animation data loader |

**Result**: All `rand()` callers are rendering/animation functions.

**Conclusion**: Damage RNG either uses different RNG function or hasn't been explored yet.

---

### Strategy E: Rendering Function Analysis ✅ (Partial Success)
**Hypothesis**: Battle-specific functions called per-frame contain or reference combat logic.

**Method**: Analyzed per-frame call chain during state 0x15 (battle):

#### FUN_8001d058 @ 0x8001d058 (8 bytes)
```c
void FUN_8001d058(void) {
    if ((g_frame_timer_flags & 8) != 0) {
        FUN_80026ce4();
    }
}
```
Thin wrapper checking flag before calling renderer.

#### FUN_80026ce4 @ 0x80026ce4 (619 bytes)
**Purpose**: Per-frame rendering coordinator.
- Sets GPU draw modes and color values
- Checks `if (g_current_game_state == 0x15)` (battle state)
- Calls `FUN_80026f50()` for battle rendering
- Handles VSync and state-specific rendering (states 3, 0xd, 0x19, 0xf)

**Not combat logic** - this is graphics coordination.

#### FUN_80026f50 @ 0x80026f50 (599 bytes)
**Purpose**: Battle camera and matrix setup.
- Builds 3D transformation matrices from battle camera values
- Sets rotation/translation for scene rendering
- Uses globals: `g_battle_value_6`, `g_battle_value_1`, `g_battle_value_2`
- Calls low-level GTE (Geometry Transformation Engine) functions

**Not combat logic** - this is 3D camera positioning.

#### FUN_8001d140 @ 0x8001d140 (67 bytes)
```c
void FUN_8001d140(int *param_1) {
    FUN_8001ada4(param_1);
}
```
Thin wrapper calling entity renderer with data pointer.

Called 4 times per frame with different pointers:
- `g_battle_data_ptr`
- `DAT_8007c350`
- `DAT_8007c354` or `DAT_8007c35c` (conditional on `g_frame_timer_flags`)
- `DAT_8007c36c`

#### FUN_8001ada4 @ 0x8001ada4 (2,455 bytes)
**Purpose**: Entity/3D model rendering loop.
- Iterates through linked list of entities (`param_1` = entity array pointer)
- Switch statement with 11 entity types:
  - **Type 1**: Visibility/distance culling (Z > 0xa0)
  - **Type 2**: Call `FUN_800480d8()` (Z > 0xa0)
  - **Type 3**: 3D model with scaling/mirroring (matrix transforms)
  - **Type 4**: Sprite-based entities with animation frames
  - **Type 5**: Main 3D character model rendering (largest case, ~100 lines)
  - **Type 6-8**: Additional entity handlers
  - **Type 0xb**: Texture animation with `MoveImage()` (frame-by-frame VRAM updates)
- Handles:
  - Z-depth culling (`0xa0 < (int)puVar10[0xd]`)
  - Matrix transformations (rotation, scaling, translation)
  - GTE operations (geometry engine)
  - Model rendering via `FUN_80043390()`, `FUN_80029888()`, `FUN_8002735c()`
  - Lighting and fog calculations

**Not combat logic** - this is the main 3D rendering pipeline.

**Result**: All analyzed functions are rendering, not game logic.

**Conclusion**: Combat logic runs in a SEPARATE system not yet identified.

---

## Key Discoveries

### 1. Battle State Flow Confirmed
```
[State 11] battle_system_main (one-time init)
    ↓
[State 21 = 0x15] Main game loop (field + battle)
    ↓ per-frame
prepare_frame_render → update_controller_input
debug_frame_diagnostics → FUN_8001d058 → FUN_80026ce4 → FUN_80026f50 (battle camera)
                        → FUN_8001d140 → FUN_8001ada4 (entity renderer)
render_and_display_frame → DrawOTag, PutDispEnv, etc.
```

### 2. Battle vs. Field Rendering
The same state (21) handles BOTH field gameplay AND battle. The code checks `if (g_current_game_state == 0x15)` to branch to battle-specific rendering.

**Implication**: Combat logic might also be behind a similar conditional check in an unexplored function.

### 3. Entity System Architecture
`FUN_8001ada4` implements an entity-component-like system:
- Entities stored as linked list (entity[0] points to entity[1], etc.)
- Each entity has a type field @ offset +0x56 (switch on `*(short*)(entity + 0x56)`)
- Entities have transform data @ offset +0x24-0x34 (rotation angles)
- Z-coordinate @ offset +0x34 used for depth culling

**Potential for modding**: This entity system could be replicated in Bevy with proper component mapping.

### 4. Battle Memory Structure
- 31KB buffer @ `g_battle_memory_buffer` (0x8007bd3c)
- 4 character structs @ `g_battle_char_data_array` (0x80084140, 1044 bytes each)
- Buffers are zeroed in BSS section, populated during `battle_system_main`
- Offsets used: base+0xded, base+0x12d2, base+0x2d4

---

## Functions Analyzed (100% Confirmed Rendering)

| Function | Address | Size (bytes) | Purpose | DICK Status |
|----------|---------|--------------|---------|-------------|
| `FUN_8001d058` | 0x8001d058 | 8 | Battle flag check wrapper | N/A (trivial) |
| `FUN_80026ce4` | 0x80026ce4 | 619 | Rendering coordinator | Needs renaming |
| `FUN_80026f50` | 0x80026f50 | 599 | Battle camera setup | Needs renaming |
| `FUN_8001d140` | 0x8001d140 | 67 | Entity render wrapper | N/A (trivial) |
| `FUN_8001ada4` | 0x8001ada4 | 2,455 | Entity/model renderer | Needs renaming |

**None of these contain combat logic.**

---

## What's Still Missing

### Core Combat Logic (HIGH PRIORITY)
1. **Turn Management**:
   - Turn order calculation
   - Action Point (AP) system
   - Speed/Initiative formulas
   
2. **Damage Formulas**:
   - Physical attack damage = f(ATK, DEF, level, ...)
   - Magic damage = f(MP cost, INT, RES, ...)
   - Critical hit calculation
   - Elemental weakness/resistance modifiers

3. **AI Decision-Making**:
   - Enemy AI behavior trees
   - Target selection algorithms
   - Skill/spell selection logic
   
4. **HP/MP Manipulation**:
   - `entity.hp -= damage`
   - `entity.mp -= cost`
   - Death detection (`if hp <= 0`)
   - Victory/defeat conditions

5. **Command Processing**:
   - Player input → action execution
   - "Attack" → calculate damage → apply to target
   - "Item" → apply effect → consume item
   - "Arts" → check AP cost → execute combo

---

## Hypotheses for Combat Logic Location

### Hypothesis 1: Event/Command Queue System ⭐⭐⭐
**High Likelihood**

Turn-based RPGs often use command queues:
```
Player selects "Attack" → Queue command
Enemy AI decides action → Queue command
Process queue in turn order → Execute commands → Apply results
```

**Search for**:
- Functions with "queue", "command", "action" in behavior
- Functions that read controller input during battle (not just `update_controller_input`)
- Functions called when battle menu selection is confirmed

**Potential addresses**:
- Functions in 0x80030000-0x80040000 (menu/UI region)
- Functions in 0x80050000-0x80060000 (battle region)

### Hypothesis 2: Large Unexplored Functions ⭐⭐
**Medium Likelihood**

Combat logic might be in one large function (1KB+) that hasn't been examined yet.

**Search strategy**:
- List all functions 500+ bytes in 0x80020000-0x80060000
- Filter out known rendering functions
- Examine functions with 4+ parameters (combat formulas take attacker, defender, skill, etc.)
- Look for functions with many conditional branches (AI logic)

**Previously identified candidates**:
- `FUN_80048310` @ 0x80048310 (683 bytes, 4 params)
- `FUN_80049348` @ 0x80049348 (639 bytes, 1 param)
- `FUN_800485bc` @ 0x800485bc (5 params)
- `FUN_8004a908` @ 0x8004a908 (1 param)

### Hypothesis 3: Callback System ⭐
**Low Likelihood**

Combat logic invoked via function pointers set at runtime.

**Why unlikely**: No function pointers found in static battle buffers.

**Search strategy**: Examine runtime behavior by tracing function pointer assignments during `battle_system_main`.

---

## Next Steps (Priority Order)

### 1. Search for Combat Command Processing (HIGHEST PRIORITY) ⭐⭐⭐
**Goal**: Find where battle menu selections are executed.

**Method**:
- Trace menu selection confirmation (when player presses X on "Attack")
- Look for functions that process battle commands
- Search for string references: "command", "action", "skill"
- Examine functions that call or are called by the menu system

**Files to check**:
- Menu system functions @ 0x80030000-0x80032000
- Battle state functions @ 0x80055000-0x80057000

### 2. Systematic Large Function Analysis ⭐⭐
**Goal**: Examine all unexplored functions 500+ bytes.

**Method**:
```bash
ghidra_list_functions | filter size > 500 | exclude known functions
```

**Priority regions**:
- 0x80040000-0x80050000 (unexplored combat region)
- 0x80050000-0x80056000 (battle functions)

### 3. Runtime Buffer Analysis ⭐
**Goal**: Examine battle buffer contents after initialization.

**Method**:
- Set breakpoint after `battle_system_main` completes
- Dump `g_battle_memory_buffer` + 31KB
- Look for function pointers, vtables, or command structures

**Tools**: Ghidra debugger or PCSX-Redux breakpoints

### 4. Cross-Reference Analysis ⭐
**Goal**: Find functions that manipulate battle character data (HP/MP).

**Method**:
- Search for write operations to `g_battle_char_data_array` + offset
- Offset +0x?? = HP field (unknown offset)
- Offset +0x?? = MP field (unknown offset)
- Trace these writes backwards to find combat formulas

---

## Important Global Addresses

### Battle System Globals
| Variable | Address | Size | Purpose |
|----------|---------|------|---------|
| `g_current_game_state` | ? | 4 bytes | Current state (0x15 = battle) |
| `g_battle_mode` | 0x8007bd20 | 4 bytes | Battle mode (0-3) |
| `g_battle_running_flag` | 0x8007bd04 | 4 bytes | Set to 1 during battle |
| `g_battle_active_flag` | 0x8007bd00 | 4 bytes | Set to 1 during battle |
| `g_battle_memory_buffer` | 0x8007bd3c | 4 bytes | Pointer to 31KB buffer |
| `g_battle_char_data_array` | 0x80084140 | 4176 bytes | 4 chars × 1044 bytes |
| `g_battle_value_1` | ? | 2 bytes | Camera Y position |
| `g_battle_value_2` | ? | 2 bytes | Camera Z position |
| `g_battle_value_6` | ? | 2 bytes | Camera X position |
| `g_frame_timer_flags` | ? | 4 bytes | Per-frame flags (bit 8 = battle) |
| `g_state_handler_table` | 0x8007079c | 0x18*12 | State handler function pointers |

### Entity Rendering Globals
| Variable | Address | Size | Purpose |
|----------|---------|------|---------|
| `g_battle_data_ptr` | 0x8007???? | 4 bytes | Entity list pointer 1 |
| `DAT_8007c350` | 0x8007c350 | 4 bytes | Entity list pointer 2 |
| `DAT_8007c354` | 0x8007c354 | 4 bytes | Entity list pointer 3a |
| `DAT_8007c35c` | 0x8007c35c | 4 bytes | Entity list pointer 3b |
| `DAT_8007c36c` | 0x8007c36c | 4 bytes | Entity list pointer 4 |

---

## Lessons Learned

### 1. Initialization ≠ Logic
`battle_system_main` is 3KB and does a lot, but it's all setup:
- Load assets from CD-ROM
- Initialize graphics buffers
- Set up character slots
- Allocate memory

**The combat logic runs AFTER initialization, per-frame or per-turn.**

### 2. Rendering ≠ Logic
Functions like `FUN_8001ada4` are large (2.4KB) but purely rendering:
- Transform 3D models
- Apply lighting
- Cull invisible objects
- Submit draw commands

**Combat logic is separate from rendering pipeline.**

### 3. State Machine is Just Coordination
State handlers (`state_handler_1_main_game_loop`) coordinate frame flow:
- Read input
- Update state
- Render output

**Combat logic is called FROM a state handler, not IN the handler itself.**

### 4. Same State for Field + Battle
State 21 (0x15) handles both overworld and battle. Functions check:
```c
if (g_current_game_state == 0x15) {
    // Battle-specific code
}
```

**Implication**: Combat logic might be behind similar conditionals in unexplored functions.

---

## Recommendations for Future Sessions

### DO:
- ✅ Focus on command/event processing functions
- ✅ Analyze large functions (500+ bytes) systematically
- ✅ Trace battle menu selection → execution path
- ✅ Look for HP/MP write operations in character array

### DON'T:
- ❌ Re-analyze rendering functions (`FUN_8001ada4`, `FUN_80026f50`)
- ❌ Examine graphics primitives (GPU, GTE, draw functions)
- ❌ Investigate CD-ROM loading (already documented)
- ❌ Study state machine handlers (already mapped)

---

## Conclusion

**Progress**: Eliminated 5 search strategies, confirmed battle rendering architecture, mapped per-frame call chain.

**Status**: Combat logic (AI, damage, turns) **still missing**. All analyzed functions are rendering/initialization.

**Next Action**: Search for combat command processing functions (Hypothesis 1) or systematically analyze large unexplored functions (Hypothesis 2).

**Confidence**: HIGH that combat logic exists in unexplored functions in 0x80030000-0x80060000 range. The search continues.
