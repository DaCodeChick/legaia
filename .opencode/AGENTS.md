# Legend of Legaia Decompilation Directives

## DICK Methodology: Decompile It Correctly, Knucklehead

**Core Principle**: Every time you analyze code in Ghidra, you must analyze all function call chains and rename EVERYTHING. Leave nothing unnamed.

### 🚨 MANDATORY ENFORCEMENT 🚨

**RULE #1: NO ANALYSIS IS COMPLETE UNTIL EVERY SYMBOL IS RENAMED**

If you touch a function in Ghidra, you are REQUIRED to rename:
- ✅ The function itself (NO `FUN_*`)
- ✅ ALL parameters (NO `param_1`, `param_2`, etc.)
- ✅ ALL local variables (NO `local_8`, `local_10`, `uVar1`, `iVar2`, etc.)
- ✅ ALL globals accessed (NO `DAT_*`, `PTR_*`, `UNK_*`)
- ✅ ALL called functions that are unnamed (queue them for next analysis)

**ANY unnamed symbol = INCOMPLETE WORK = UNACCEPTABLE**

### Partial Analysis Exception

The ONLY acceptable time to leave symbols unnamed is when you explicitly:
1. Document it as "PARTIAL ANALYSIS - TO BE CONTINUED"
2. Create a TODO checklist of remaining unnamed symbols
3. Mark the function status as "In Progress" (NOT "Complete")
4. Commit to finishing it in the SAME SESSION or next immediate session

**If you claim a function is "Complete" but it has ANY unnamed symbols, you have FAILED the DICK methodology.**

### 🚫 Common DICK Violations (And How to Fix Them)

#### Violation Example: main() Analysis (2026-02-14)

**What was done WRONG:**
```c
// Only renamed globals, left ALL functions and locals unnamed
void main(void)
{
  undefined4 uVar1;           // ❌ UNNAMED LOCAL
  char *pcVar2;               // ❌ UNNAMED LOCAL
  uint *puVar3;               // ❌ UNNAMED LOCAL
  uint uVar4;                 // ❌ UNNAMED LOCAL
  
  FUN_8003ee7c(0);            // ❌ UNNAMED FUNCTION
  uVar1 = FUN_8003f084();     // ❌ UNNAMED FUNCTION
  g_config_mode = (short)uVar1;  // ✅ Global was renamed (good!)
  FUN_8003f024();             // ❌ UNNAMED FUNCTION
  FUN_80062310();             // ❌ UNNAMED FUNCTION
  // ... 20+ more unnamed functions and variables ...
}
```

**What should have been done:**
```c
// ALL symbols renamed with meaningful names
void main(void)
{
  int config_value;                    // ✅ RENAMED LOCAL
  char *debug_string;                  // ✅ RENAMED LOCAL
  uint *data_load_ptr;                 // ✅ RENAMED LOCAL
  uint flags;                          // ✅ RENAMED LOCAL
  
  init_graphics_system(0);             // ✅ RENAMED FUNCTION
  config_value = detect_system_mode(); // ✅ RENAMED FUNCTION
  g_config_mode = (short)config_value;
  init_sound_system();                 // ✅ RENAMED FUNCTION
  init_controller_system();            // ✅ RENAMED FUNCTION
  // ... all functions and variables renamed ...
}
```

**How to fix going forward:**
1. Go back to main() in Ghidra
2. Rename EVERY local variable (`uVar1` → `config_value`, etc.)
3. Examine EVERY called function and rename it
4. Update the documentation with complete analysis
5. ONLY THEN mark main() as "Complete"

---

## 🎯 Decompilation Workflow

### For Every Function Analysis Session

When you analyze a function using the Ghidra MCP tools, you **MUST** complete ALL of the following steps or explicitly document what remains:

#### ✅ **Mandatory Checklist**

**🔴 STOP: Do NOT mark a function as "Complete" until ALL boxes are checked 🔴**

- [ ] **Function Identification**
  - ✅ Determine the function's purpose
  - ✅ Rename function with descriptive snake_case name (NO `FUN_*` allowed)
  - ✅ Add function-level comment explaining what it does
  - ✅ Document return value purpose
  - ✅ Set function signature if non-default (return type, calling convention)
  
- [ ] **Parameter Analysis**
  - ✅ Identify ALL parameters (check function signature)
  - ✅ Determine type and purpose of each parameter
  - ✅ Rename ALL parameters with descriptive names
    - ❌ FORBIDDEN: `param_1`, `param_2`, `param_N`
    - ✅ REQUIRED: `player_data`, `enemy_index`, `damage_amount`, etc.
  - ✅ Add comments for non-obvious parameter usage
  
- [ ] **Local Variable Analysis**
  - ✅ Identify ALL local variables in function body
  - ✅ Determine type and purpose of each local
  - ✅ Rename ALL locals with descriptive names
    - ❌ FORBIDDEN: `local_8`, `local_10`, `uVar1`, `iVar2`, `pcVar3`, etc.
    - ✅ REQUIRED: `battle_state`, `temp_hp`, `animation_frame`, etc.
  - ✅ Consider variable lifetime and scope
  - ✅ If truly temporary (used for 2-3 lines), acceptable names:
    - `temp_ptr`, `temp_value`, `loop_counter`, `i`, `j`, `k` (for loops only)
  
- [ ] **Global Variable Analysis**
  - ✅ Identify ALL global variable accesses (reads/writes)
  - ✅ Determine global's purpose from usage context
  - ✅ Rename globals with descriptive names
    - ❌ FORBIDDEN: `DAT_80012345`, `PTR_LAB_80012345`, `UNK_80012345`
    - ✅ REQUIRED: `g_current_battle_state`, `g_player_party_data`, `g_frame_counter`
  - ✅ Use `g_` prefix for globals (mutable state)
  - ✅ Use `G_` prefix for global constants (read-only data)
  - ✅ Document which functions access shared globals
  
- [ ] **Call Chain Analysis**
  - ✅ Examine ALL functions called by this function
  - ✅ Rename OR queue each unnamed called function
  - ✅ Examine ALL functions that call this function (use xrefs)
  - ✅ Document call relationships in comments
  - ✅ Create TODO list for unnamed functions discovered
  
- [ ] **Structure/Type Inference**
  - ✅ Identify structs passed by pointer
  - ✅ Document struct field accesses (offset, type, purpose)
  - ✅ Create Ghidra struct definitions when pattern is clear
  - ✅ Apply struct types to function parameters
  - ✅ Cross-reference with other functions using same struct
  
- [ ] **Control Flow Documentation**
  - ✅ Add comments explaining conditional logic (WHY, not just WHAT)
  - ✅ Document loop purposes and iteration patterns
  - ✅ Explain switch/case statement meanings
  - ✅ Note any unusual or complex control flow
  - ✅ Document magic numbers (convert to named constants)
  
- [ ] **Progress Tracking**
  - ✅ Update the Decompilation Progress section in this file
  - ✅ Mark function status accurately:
    - "Complete" = ZERO unnamed symbols remaining
    - "In Progress" = Some symbols renamed, work continues
    - "Identified" = Purpose known but not yet fully analyzed
  - ✅ Note any blockers or uncertainties as TODOs
  - ✅ Document relationships to game systems
  - ✅ Update related documentation files (docs/decompilation/*.md)

#### 🔍 **Self-Verification Before Marking Complete**

Before you mark ANY function as "Complete", run this checklist:

1. **Re-fetch the decompiled code**: `ghidra_get_code(function="...", format="decompiler")`
2. **Search for forbidden patterns**:
   - Search for `FUN_` - If found, rename those functions
   - Search for `param_` - If found, rename those parameters  
   - Search for `local_` - If found, rename those locals
   - Search for `uVar`, `iVar`, `pcVar`, etc. - If found, rename them
   - Search for `DAT_`, `PTR_`, `UNK_` - If found, rename those globals
3. **Verify all magic numbers have comments**:
   - `0x280` → `// 640 pixels width`
   - `0x1ff` → `// 511 pixels height`
   - `0x3C` → `// 60 frames (1 second at 60fps)`
4. **Check all control flow has explanatory comments**:
   - Every `if` statement explains WHY the condition is checked
   - Every loop explains WHAT is being iterated over and WHY
   - Every `switch` case explains WHAT that case represents
5. **Verify all called functions are queued or renamed**:
   - List ALL called functions
   - Each one should be either renamed OR documented as "TODO: analyze next"

**If ANY of the above checks fail, the function is NOT complete.**

---

## 📋 Naming Conventions

### Functions
- **Format**: `snake_case`
- **Examples**:
  - `initialize_battle_system`
  - `load_character_data`
  - `process_player_input`
  - `render_3d_model`
  - `calculate_damage_formula`
- **Prefixes** (when applicable):
  - `init_*` - Initialization functions
  - `update_*` - Per-frame update functions
  - `render_*` - Rendering functions
  - `load_*` / `save_*` - I/O operations
  - `calculate_*` - Math/computation functions
  - `handle_*` - Event handlers
  - `parse_*` - Data parsing functions

### Variables & Parameters
- **Format**: `snake_case`
- **Examples**:
  - `player_health`
  - `enemy_count`
  - `current_animation_frame`
  - `texture_address`
  - `battle_state`
- **Avoid**: Generic names like `temp`, `var`, `data` unless truly temporary

### Constants & Globals
- **Format**: `SCREAMING_SNAKE_CASE`
- **Examples**:
  - `MAX_PARTY_SIZE`
  - `SCREEN_WIDTH`
  - `BATTLE_MENU_OPTIONS`
  - `DEFAULT_WALK_SPEED`
  - `TEXTURE_BASE_ADDRESS`

### Structures & Types
- **Format**: `PascalCase`
- **Examples**:
  - `BattleCharacter`
  - `EnemyData`
  - `ItemDefinition`
  - `Vec3D`
  - `TextureInfo`

### Enums
- **Type**: `PascalCase`
- **Variants**: `PascalCase`
- **Examples**:
  ```rust
  enum BattleState {
      PlayerTurn,
      EnemyTurn,
      AnimationPlaying,
      Victory,
      Defeat,
  }
  ```

---

## 🗺️ Memory Map Reference

### PSX Memory Layout (SCUS_942.54)

```
┌─────────────────────────────────────────┐
│ 1F800000-1F8003FF  CACHE (1KB)         │  Scratchpad RAM
├─────────────────────────────────────────┤
│ 1F800400-1F800FFF  UNK1 (3KB)          │  Unknown/Stack?
├─────────────────────────────────────────┤
│ 1F801000-1F801023  MCTRL1              │  Memory Control 1
│ 1F801040-1F80105F  IO_PORTS            │  I/O Ports
│ 1F801060-1F801063  MCTRL2              │  Memory Control 2
│ 1F801070-1F801075  INT_CTRL            │  Interrupt Control
├─────────────────────────────────────────┤
│ 1F801080-1F8010EB  DMA                 │  DMA Channels
│   • 1F801080  MDEC_IN                  │
│   • 1F801090  MDEC_OUT                 │
│   • 1F8010A0  GPU                      │
│   • 1F8010B0  CDROM                    │
│   • 1F8010C0  SPU                      │
│   • 1F8010D0  PIO                      │
│   • 1F8010E0  OTC                      │
├─────────────────────────────────────────┤
│ 1F8010F0-1F8010F7  DMA_CTRL_INT        │  DMA Control/Interrupt
├─────────────────────────────────────────┤
│ 1F801100-1F80112F  TIMERS (3×16B)      │  Hardware Timers
├─────────────────────────────────────────┤
│ 1F801800-1F801803  CDROM_REGS          │  CD-ROM Registers
│ 1F801810-1F801817  GPU_REGS            │  GPU Registers
│ 1F801820-1F801827  MDEC_REGS           │  MDEC Registers
├─────────────────────────────────────────┤
│ 1F801C00-1F801DBF  SPU                 │  Sound Processing Unit
├─────────────────────────────────────────┤
│ 20000000-20000263  GTEMAC              │  GTE Macro Library (612B)
├─────────────────────────────────────────┤
│ 80000000-8000FFFF  RAM (64KB)          │  Low RAM
│ 80010000-8007B7FF  CODE (430KB)        │  ⭐ MAIN GAME CODE ⭐
│ 8007B800-801FFFFF  RAM (1.5MB)         │  High RAM / Data
└─────────────────────────────────────────┘
```

### Key Address Ranges

- **GTEMAC (0x20000000)**: GTE (Geometry Transform Engine) macro functions
  - All the `gte_*` functions for 3D math operations
  - Used extensively for rendering
  
- **Main Code (0x80010000-0x8007B7FF)**: Primary game logic
  - Entry point likely near 0x80010000
  - Contains all major game systems
  - 1,121 functions total to decompile
  
- **Data Sections**: Asset pointers, lookup tables, constants
  - Enemy data tables
  - Item definitions
  - Text/dialogue pointers
  - Animation data

---

## 🎮 Game System Architecture

### Major Systems (To Be Mapped)

#### 1. **Initialization & Main Loop**
- [ ] Entry point function
- [ ] Main game loop
- [ ] System initialization
- [ ] BIOS/library setup

#### 2. **Graphics/Rendering System**
- [ ] GPU command submission
- [ ] Display list management
- [ ] Texture management (TIM format)
- [ ] 3D model rendering
- [ ] 2D sprite rendering
- [ ] Animation system
- [ ] Camera system
- [ ] Lighting calculations
- **GTE Functions** (0x20000000-0x20000263):
  - Matrix operations
  - Vertex transformations
  - Perspective projection
  - Color interpolation

#### 3. **Field/World System**
- [ ] Character controller
- [ ] Collision detection
- [ ] Map loading
- [ ] NPC management
- [ ] Event triggers
- [ ] World map navigation
- [ ] Door/transition handling

#### 4. **Battle System** ⭐ (Most Complex)
- [ ] Battle initialization
- [ ] Turn management
- [ ] **Art System** (unique combo input system)
  - Input buffering
  - Command recognition
  - Art animations
- [ ] Damage calculation
- [ ] Enemy AI
- [ ] Status effects
- [ ] Item usage in battle
- [ ] Magic/Ra-Seru usage
- [ ] Victory/defeat handling
- [ ] Experience/level up

#### 5. **Menu System**
- [ ] Main menu
- [ ] Pause menu
- [ ] Equipment menu
- [ ] Item menu
- [ ] Magic/Arts menu
- [ ] Status screens
- [ ] Save/Load interface

#### 6. **Audio System**
- [ ] SPU (Sound Processing Unit) management
- [ ] VAB (Voice Attribute Bank) loading
- [ ] VAG (audio) playback
- [ ] Music sequencing
- [ ] Sound effect triggers

#### 7. **Save/Load System**
- [ ] Memory card operations
- [ ] Save data format
- [ ] Game state serialization
- [ ] Slot management

#### 8. **Input System**
- [ ] Controller reading
- [ ] Button mapping
- [ ] Input buffering
- [ ] Menu navigation

#### 9. **Asset Management**
- [ ] CD-ROM file loading
- [ ] Asset decompression
- [ ] Memory management
- [ ] Texture caching

#### 10. **Event/Scripting System**
- [ ] Event script interpreter
- [ ] Flag management
- [ ] Dialogue system
- [ ] Cutscene playback
- [ ] Quest tracking

---

## 📊 Decompilation Progress

### Status Definitions

**🔴 CRITICAL: Status definitions are STRICT. Do not misrepresent work quality.**

- **Unanalyzed**: Function exists but not yet examined at all
- **Identified**: Purpose roughly determined, function renamed, but parameters/locals/calls are still unnamed
- **In Progress**: Actively being analyzed, some symbols renamed, work incomplete
- **Complete**: ✅ **ZERO unnamed symbols** - ALL functions, parameters, locals, and globals renamed
  - If ANY `FUN_*`, `param_*`, `local_*`, `DAT_*`, `uVar*`, etc. remain → NOT COMPLETE
- **Verified**: Complete + tested/compared with original behavior in running game
- **Rust Impl**: Rust implementation exists and is functionally equivalent

**IMPORTANT**: Moving from "In Progress" to "Complete" requires 100% symbol coverage. No exceptions.

### Progress Tracking

#### GTE Functions (GTEMAC 0x20000000-0x20000263)
Total: ~100 functions | Status: Partially Named

| Address    | Function Name | Status | System | Notes |
|------------|---------------|--------|--------|-------|
| 0x20000000 | gte_ldv0 | Complete | Graphics | Load vertex 0 |
| 0x20000004 | gte_ldv1 | Complete | Graphics | Load vertex 1 |
| 0x20000008 | gte_ldv2 | Complete | Graphics | Load vertex 2 |
| 0x2000000C | gte_ldv3 | Complete | Graphics | Load vertex 3 |
| ... | ... | ... | ... | ... |

#### Main Game Code (0x80010000-0x8007B7FF)
Total: 1,121 functions | Status: 14 Complete, 1,107 Remaining

| Address    | Function Name | Status | System | Notes |
|------------|---------------|--------|--------|-------|
| 0x80015e90 | main | **✅ Complete** | Entry | Entry point - ALL symbols renamed, ZERO unnamed locals/params/functions ✅ |
| 0x80026c20 | __main | **✅ Complete** | Entry | Empty function (returns immediately) |
| 0x8003f084 | get_config_mode | **✅ Complete** | Config | Returns config mode (1 = retail). No params/locals/calls. Commented. |
| 0x8002b92c | get_system_mode | **✅ Complete** | Config | Returns system mode (0 = no vibration). No params/locals/calls. Commented. |
| 0x8003ee7c | init_serial_audio | **✅ Complete** | Audio | Initializes CD audio. All params/locals renamed. Commented. |
| 0x8003e104 | load_monster_audio_data | **✅ Complete** | Audio | Loads monster audio from CD or host. ALL symbols renamed. Fully documented. |
| 0x80060910 | PCclose | **✅ Complete** | Library | PSX library function to close file opened with PCopen |
| 0x8003f024 | init_cdrom_system | **✅ Complete** | CD-ROM | Initializes CD-ROM drive. 3 globals renamed. Fully documented. |
| 0x80062310 | init_sound_system | **✅ Complete** | Audio | High-level sound system init. Calls init_spu. |
| 0x800693b8 | init_spu | **✅ Complete** | Audio | SPU hardware initialization wrapper. |
| 0x800644c0 | init_sprite_buffer | **✅ Complete** | Graphics | 2D sprite buffer grid (9 locals + 3 globals). Fully documented. |
| 0x8002b934 | vibration_stub | **✅ Complete** | Input | Empty stub for vibration (disabled in this build). |
| 0x8001d230 | init_memory_card_system | **✅ Complete** | Save | Memory card init for both slots (5 functions + 13 globals renamed). |
| 0x8002b3d4 | init_memory_allocator | **✅ Complete** | Memory | Custom heap allocator (9 locals + 1 global). Complex. |
| ... | ... | Unanalyzed | ... | ... |

**Recently Completed (2026-02-14 DICK Session #2):**
- ✅ init_cdrom_system() - CD-ROM hardware initialization
- ✅ init_sound_system() - Sound system wrapper
- ✅ init_spu() - SPU hardware init
- ✅ init_sprite_buffer() - Complex 2D sprite grid allocator
- ✅ vibration_stub() - Disabled vibration stub
- ✅ init_memory_card_system() - Memory card system (renamed 5 called functions + 13 globals)
- ✅ init_memory_allocator() - Custom heap allocator with free list

**Functions Renamed (2026-02-14 Session #2):**
- init_memory_card_slot_0, init_memory_card_slot_1
- setup_memory_card_buffers, configure_memory_card_slot
- finalize_memory_card_setup

**Globals Renamed (2026-02-14 Session #1):**
- g_monster_count, g_monster_audio_base_sector, g_monster_audio_offset_table
- g_cdrom_error_counter, g_loaded_audio_size

**Globals Renamed (2026-02-14 Session #2):**
- g_cdrom_mode_param, g_cdrom_counter_1, g_cdrom_counter_2
- g_sprite_buffer_row_count, g_sprite_buffer_sprites_per_row, g_sprite_buffer_row_pointers
- g_memory_card_buffer_1/2/3, g_memory_card_slot_0/1_data
- g_memory_card_event_1 through _8 (8 event handles)
- g_memory_allocator_base

#### Current Work Queue (DICK Methodology)

Functions called from main() that need analysis (in call order):
- [x] `init_serial_audio` (0x8003ee7c) - Already renamed ✅
- [x] `get_config_mode` (0x8003f084) - Already renamed ✅
- [x] `get_system_mode` (0x8002b92c) - Already renamed ✅
- [x] `init_cdrom_system` (0x8003f024) - Already renamed ✅
- [x] `init_sound_system` (0x80062310) - Already renamed ✅
- [x] `init_sprite_buffer` (0x800644c0) - Already renamed ✅
- [x] `vibration_stub` (0x8002b934) - Already renamed ✅
- [x] `init_memory_card_system` (0x8001d230) - Already renamed ✅
- [x] `init_memory_allocator` (0x8002b3d4) - Already renamed ✅
- [x] `init_data_tables` (0x800265e8) - Already renamed ✅
- [x] `init_game_state` (0x8001d424) - Already renamed ✅
- [x] `init_cdrom_protection` (0x8003f08c) - Already renamed ✅
- [x] `load_file_from_host` (0x8003e6bc) - Already renamed ✅
- [x] `init_display_buffers` (0x8001daf8) - Already renamed ✅
- [x] `init_state_environment` (0x8001dcf8) - Already renamed ✅
- [x] `allocate_graphics_buffers` (0x8001e3b8) - Already renamed ✅
- [x] `prepare_frame_render` (0x8001698c) - Already renamed ✅
- [x] `render_and_display_frame` (0x80016b6c) - Already renamed ✅
- [x] `prepare_cdrom_data_load` (0x8001822c) - Already renamed ✅
- [x] `wait_for_cdrom_read` (0x8003ebe4) - Already renamed ✅
- [x] `init_sound_playback_system` (0x8003de7c) - Already renamed ✅
- [x] `gte_load_h_register` (0x8002666c) - Already renamed ✅
- [ ] State handler functions (from g_state_handler_table) - **Need to analyze next**
- [x] `abort_cdrom_operations` (0x8003ed04) - Already renamed ✅
- [x] `cleanup_and_transition_state` (0x80016230) - Already renamed ✅
- [x] `update_controller_input` (0x8003d254) - Already renamed ✅
- [x] `exit_to_executable` (0x80017714) - Already renamed ✅

**Next Priority**: Analyze state handler functions and ensure each has all symbols renamed

---

## 🔧 Ghidra MCP Tool Usage Patterns

### Initial Function Discovery

```markdown
1. List functions in address range:
   ghidra_list_functions(pattern="*", offset=0, limit=100)

2. Get function details:
   ghidra_get_function_info(function_name="FUN_80012345")

3. View decompiled code:
   ghidra_get_code(function="FUN_80012345", format="decompiler")

4. Examine call graph:
   ghidra_get_call_graph(function="FUN_80012345", depth=2, direction="both")
```

### Analysis Workflow

```markdown
For function at address 0x80012345:

1. Get decompiled view:
   ghidra_get_code(function="0x80012345", format="decompiler")

2. Get call graph (who calls this, who does this call):
   ghidra_get_call_graph(function="0x80012345", depth=2)

3. Check cross-references:
   ghidra_xrefs(function="0x80012345", direction="both")

4. Examine data accessed:
   - Note any DAT_* references
   - Use ghidra_get_hexdump for data structures
   - Use ghidra_list_data to find defined data

5. Rename function:
   ghidra_rename_symbol(
     target_type="function",
     identifier="FUN_80012345",
     new_name="initialize_battle_system"
   )

6. Add comments:
   ghidra_set_comment(
     target="function",
     function_name="initialize_battle_system",
     comment="Initializes the battle system state machine and loads initial data"
   )

7. For complex functions, rename variables:
   ghidra_rename_symbol(
     target_type="variable",
     identifier="initialize_battle_system",
     variable_name="local_10",
     new_name="battle_state_ptr"
   )
```

### Data Structure Discovery

```markdown
1. Find data at address:
   ghidra_get_hexdump(address="0x8007C000", len=256)

2. Look for patterns suggesting structures:
   - Repeating byte patterns
   - Pointer references (0x80XXXXXX)
   - String data (ASCII sequences)

3. Create structure definition:
   ghidra_struct(
     action="create",
     name="EnemyData",
     c_definition="struct EnemyData { uint hp; uint mp; ushort atk; ushort def; };"
   )

4. Apply structure to data:
   ghidra_set_data_type(address="0x8007C000", data_type="EnemyData")
```

### Batch Operations

**Use batch operations to rename multiple symbols efficiently:**

```markdown
# Example: Rename all locals in main() at once
ghidra_rename_symbol_batch(renames=[
  {"target_type": "variable", "identifier": "main", "variable_name": "uVar1", "new_name": "config_value"},
  {"target_type": "variable", "identifier": "main", "variable_name": "pcVar2", "new_name": "debug_string"},
  {"target_type": "variable", "identifier": "main", "variable_name": "puVar3", "new_name": "data_load_ptr"},
  {"target_type": "variable", "identifier": "main", "variable_name": "uVar4", "new_name": "flags"},
  {"target_type": "variable", "identifier": "main", "variable_name": "puVar5", "new_name": "asset_ptr"},
  {"target_type": "variable", "identifier": "main", "variable_name": "piVar6", "new_name": "sector_offset"}
])

# Example: Rename multiple functions called from main
ghidra_rename_symbol_batch(renames=[
  {"target_type": "function", "identifier": "FUN_8003ee7c", "new_name": "init_graphics_system"},
  {"target_type": "function", "identifier": "FUN_8003f084", "new_name": "detect_system_mode"},
  {"target_type": "function", "identifier": "FUN_8002b92c", "new_name": "check_debug_flag"},
  {"target_type": "function", "identifier": "FUN_8003f024", "new_name": "init_sound_system"}
])
```

**Best practice**: Rename in batches of 5-10 symbols, then verify with `ghidra_get_code` to ensure names make sense in context.

---

## 📝 Documentation Standards

### Function Documentation Template

```c
/**
 * Brief one-line description of function purpose
 * 
 * Detailed explanation of what the function does, including:
 * - Context of when/why it's called
 * - Side effects or state changes
 * - Any important implementation details
 * 
 * @param parameter_name Description of parameter and its valid range/values
 * @param another_param Description
 * @return Description of return value (if non-void)
 * 
 * Related Functions:
 * - function_that_calls_this()
 * - function_that_this_calls()
 * 
 * Global Variables Accessed:
 * - GLOBAL_BATTLE_STATE (read/write)
 * - PLAYER_PARTY_DATA (read)
 * 
 * Rust Implementation: src/battle/mod.rs:init_battle_system()
 */
void initialize_battle_system(BattleCharacter* player_party, int party_size) {
    // ... implementation ...
}
```

### Inline Comment Standards

```c
// Good: Explains WHY, not just WHAT
// Check if player has enough MP before allowing art execution
if (player->current_mp < art->mp_cost) {
    return BATTLE_ERROR_INSUFFICIENT_MP;
}

// Bad: Just restates the code
// Check if current_mp is less than mp_cost
if (player->current_mp < art->mp_cost) {
    return BATTLE_ERROR_INSUFFICIENT_MP;
}
```

### Uncertainty Documentation

When you're unsure about something, document it clearly:

```c
// TODO: Verify this is actually enemy count, could be enemy type ID
// Based on usage context in calculate_damage(), seems like count
int enemy_count_or_type = *(int*)(DAT_8007C100 + 0x10);

// FIXME: Magic number - need to determine meaning
// Appears to be related to animation frame timing
if (frame_counter > 0x3C) {  // 0x3C = 60 decimal, possibly 1 second at 60fps?
    reset_animation();
}
```

---

## 🎯 Priority Analysis Targets

### Phase 1: Find Core Entry Points
1. **Main entry point** - First function executed
2. **Main loop** - Per-frame update function
3. **Initialization routines** - Setup functions called at start
4. **State machine** - Game mode switching (field → battle → menu)

**How to find**:
- Look for functions called very early (low addresses in CODE segment)
- Check for functions with very high call counts (central hub functions)
- Look for infinite loops (main loop characteristic)
- Find BIOS interrupt handlers

### Phase 2: Graphics System
Priority: HIGH (needed for any visual output)

**Key functions to find**:
- GPU initialization
- Display list submission
- Texture upload
- Primitive rendering (triangles, sprites)
- Frame buffer management
- GTE macro usage

**Starting point**: GTE functions are already named (0x20000000)

### Phase 3: Battle System
Priority: HIGH (most complex and unique system)

**Key functions to find**:
- Battle initialization
- Art command input system
- Combo recognition
- Damage calculation
- Enemy AI decision making
- Turn management

**Look for**:
- Input buffering code (for Art system)
- Large switch statements (likely command handlers)
- Damage formula calculations (mathematical operations on stats)

### Phase 4: Field System
Priority: MEDIUM

**Key functions to find**:
- Character movement
- Collision detection
- Map loading
- Event triggers

### Phase 5: Menu & UI
Priority: MEDIUM

**Key functions to find**:
- Menu navigation
- Item management
- Equipment changes
- Status displays

### Phase 6: Audio & Assets
Priority: LOW (can use placeholders initially)

---

## 🧪 Verification Strategy

### Behavioral Testing
For each implemented system, create test cases that compare with original:

1. **Unit Tests**: Individual function behavior
   - Input → Output verification
   - Edge case handling
   
2. **Integration Tests**: System interactions
   - Battle damage calculations
   - Item usage effects
   - Status effect stacking

3. **Playthrough Tests**: End-to-end verification
   - Run through game sections
   - Compare outcomes with original
   - Record any discrepancies

### Data Validation
- Extract asset tables from original
- Compare with our parsed versions
- Verify all items/enemies/arts are present
- Check stat values match

---

## 🚀 Quick Start for New Analysis Session

### Session Checklist

```markdown
□ Load Ghidra project with SCUS_942.54
□ Review previous session notes
□ Identify function(s) to analyze
□ Set status to "In Progress" in progress table
□ Get decompiled code
□ Get call graph
□ Analyze and rename according to DICK methodology
□ Update progress table
□ Commit changes with descriptive message
□ Update this document with findings
```

### Common Patterns to Look For

**Initialization Pattern**:
```c
void init_something(void) {
    // Zeroing memory
    memset(&GLOBAL_STATE, 0, sizeof(GLOBAL_STATE));
    
    // Setting defaults
    SOME_FLAG = 0;
    SOME_COUNTER = 0;
    
    // Calling sub-initializers
    init_subsystem_a();
    init_subsystem_b();
}
```

**Update Loop Pattern**:
```c
void update_something(void) {
    // State machine
    switch (current_state) {
        case STATE_IDLE:
            handle_idle();
            break;
        case STATE_ACTIVE:
            handle_active();
            break;
        // ...
    }
    
    // Always runs
    update_animations();
    update_timers();
}
```

**Rendering Pattern**:
```c
void render_something(void) {
    // Setup
    gte_SetRotMatrix(&transform_matrix);
    gte_SetTransVector(&position);
    
    // Transform vertices
    for (i = 0; i < vertex_count; i++) {
        gte_ldv0(&vertices[i]);
        gte_rtps();  // Rotate, translate, perspective transform
        gte_stsxy(&screen_coords[i]);
    }
    
    // Submit primitives
    submit_gpu_primitive(&primitive);
}
```

**Data Table Access Pattern**:
```c
// Common pattern for accessing item/enemy/art data
ItemData* get_item_data(int item_id) {
    // Bounds check
    if (item_id >= ITEM_COUNT) return NULL;
    
    // Array lookup or calculated offset
    return &ITEM_TABLE[item_id];
    // or: return (ItemData*)(ITEM_BASE_ADDR + (item_id * sizeof(ItemData)));
}
```

---

## 📚 Resources

### PSX Hardware Documentation
- [PSX Specifications](http://problemkaputt.de/psx-spx.htm) - Comprehensive hardware reference
- GTE (Geometry Transform Engine) - 3D math coprocessor
- GPU - Graphics rendering
- SPU - Audio processing

### File Formats
- **TIM**: PSX texture format
- **VAB**: Voice Attribute Bank (audio samples + metadata)
- **VAG**: Individual audio sample format
- **STR**: Streaming video format
- **XA**: CD-XA audio format

### Legend of Legaia Specific
- [The Cutting Room Floor](https://tcrf.net/Legend_of_Legaia) - Unused content and debug info
- Community speedrun resources
- Fan sites with game data

---

## 🎓 Learning Resources for Agents

### Understanding PSX Assembly
- MIPS R3000 instruction set
- Common compiler patterns (GCC for PSX)
- Register usage conventions:
  - `$a0-$a3`: Argument registers
  - `$v0-$v1`: Return value registers
  - `$t0-$t9`: Temporary registers
  - `$s0-$s7`: Saved registers
  - `$sp`: Stack pointer
  - `$ra`: Return address

### Recognizing Patterns
- **Struct access**: Base pointer + offset
  - `lw $t0, 0x10($a0)` → Reading struct field at offset 0x10
- **Array indexing**: Base + (index × element_size)
- **Function calls**: `jal function_address` followed by delay slot
- **Loops**: Backward branches with counter

---

## 💾 Rust Implementation Mapping

As functions are decompiled and understood, track the Rust implementation:

### Example Mapping Entry

**Original Function**: `initialize_battle_system` @ 0x80012345
**Rust Location**: `crates/legaia-engine/src/battle/init.rs:15`
**Status**: Implemented
**Verification**: Tested with sample battle data
**Differences**: 
- Original uses fixed-point math, Rust uses f32
- Original allocates on heap, Rust uses arena allocator
**Notes**: Rust version adds safety checks not present in original

---

## 🔄 Workflow Integration

### Daily Workflow
1. **Morning**: Review progress, pick next target function(s)
2. **Analysis**: Decompile, rename, document using DICK methodology
3. **Implementation**: Write Rust equivalent in appropriate crate
4. **Testing**: Verify behavior matches original
5. **Documentation**: Update progress table and notes
6. **Commit**: Descriptive commit message with function names

### Weekly Goals
- Set weekly targets (e.g., "Fully analyze battle damage calculation this week")
- Review progress vs. targets
- Adjust priorities based on blockers
- Document discoveries and insights

---

## ⚠️ Important Reminders

### 🚨 DO NOT SKIP STEPS - THIS IS NON-NEGOTIABLE 🚨

**EVERY symbol must be renamed. NO EXCEPTIONS.**

- ❌ Every `FUN_*` function MUST be renamed or queued
- ❌ Every `param_*` parameter MUST be renamed
- ❌ Every `local_*` variable MUST be renamed
- ❌ Every `uVar*`, `iVar*`, `pcVar*` variable MUST be renamed
- ❌ Every `DAT_*`, `PTR_*`, `UNK_*` global MUST be renamed
- ✅ This is DICK methodology - no shortcuts, no compromises

**If you can't determine a good name:**
- Use descriptive placeholders: `unknown_init_param`, `mystery_buffer_ptr`, `temp_calculation_result`
- Add TODO comments: `// TODO: Determine purpose by analyzing caller functions`
- Mark function as "In Progress", NOT "Complete"
- Document your uncertainty in the decompilation notes

**"I don't know what it does" is NOT an excuse to leave it as `uVar1`.**
- Minimum acceptable: `unknown_uint_1` with TODO comment
- Better: `calculation_result` (describes usage, not purpose)
- Best: `damage_multiplier` (describes purpose from context analysis)

### When Stuck
1. Look at calling functions for context
2. Look at called functions for clues
3. Search for similar patterns in already-analyzed code
4. Check cross-references to data
5. Run the original game and observe behavior
6. Document uncertainty with TODO and descriptive placeholder name
7. **NEVER leave it as `FUN_*`, `param_*`, or `local_*`**

### Quality Over Speed
- Better to analyze 5 functions thoroughly than 50 functions poorly
- Proper names now save hours of confusion later
- Good documentation compounds in value over time
- **Incomplete analysis creates technical debt that must be paid back**

### Accountability

When documenting your analysis:
- Be honest about completion status
- "In Progress" with TODOs is better than falsely claiming "Complete"
- Partial analysis is acceptable IF properly documented
- Claiming completion with unnamed symbols is UNACCEPTABLE

---

## 🎮 Let's Decompile This Correctly, Knucklehead!

Remember: 
- **Every unnamed symbol is a failure.**
- **Every magic number without a comment is a future headache.**
- **Every skipped step is technical debt.**
- **Every false "Complete" status is a lie that will haunt you.**

### The DICK Standard

**D**ecompile: Examine the function thoroughly  
**I**dentify: Determine purpose of every symbol  
**C**hristian: Name everything descriptively (wait, that doesn't work...)  
**K**eep: Keep no unnamed symbols

Okay, maybe it's just "Decompile It Correctly, Knucklehead" without a clever acronym. The point stands:

✅ **Be thorough**  
✅ **Be systematic**  
✅ **Be honest about completion**  
✅ **Be the DICK**

---

*Last Updated: 2026-02-14*  
*Status: Strengthened enforcement after incomplete main() analysis*  
*Next Review: After first function reaches TRUE "Complete" status (zero unnamed symbols)*
