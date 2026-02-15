# Combat AI Extraction Guide

Complete guide for reverse engineering Legend of Legaia combat AI from PSX overlays and converting to Lua scripts.

## Overview

This guide covers:
1. Importing battle overlays into Ghidra
2. Applying DICK methodology to AI callbacks
3. Analyzing enemy behavior patterns
4. Extracting AI logic to Lua scripts
5. Mapping AI to enemy names

## Prerequisites

- **Ghidra project**: SCUS_942.54 loaded and analyzed
- **Battle overlay**: `/tmp/prot_dat_head.bin` (extracted from PROT.DAT)
- **Documentation**: Entity structure (`docs/decompilation/entity-structure.md`)
- **Tools**: Ghidra GUI, text editor for Lua scripts

## Part 1: Import Battle Overlay into Ghidra

### Step 1.1: Open Ghidra GUI

```bash
# Launch Ghidra
ghidra

# Open project containing SCUS_942.54
# Double-click SCUS_942.54 to open in CodeBrowser
```

### Step 1.2: Create Memory Block for Overlay

The battle overlay is dynamically loaded at runtime to address `0x801d0000`. We need to create a memory block for it:

1. **Open Memory Map**:
   - Menu: `Window → Memory Map`
   - Or press: `Ctrl+Shift+M`

2. **Add New Block**:
   - Click the **"+"** button (Add Block)
   - Configure the block:
     ```
     Block Name:      battle_overlay_0x801d
     Start Address:   801d0000
     Length:          0x40000   (262144 bytes = 256KB)
     Comment:         Battle AI overlay loaded from PROT.DAT
     
     Permissions:
     [✓] Read
     [✓] Write  
     [✓] Execute
     [✓] Volatile
     
     Type:
     [✓] Overlay      (IMPORTANT - marks as runtime-loaded)
     ```
   - Click **OK**

3. **Verify Block Created**:
   - You should see "battle_overlay_0x801d" in the memory map
   - Base address: `801d0000`
   - End address: `801effff`

### Step 1.3: Import Overlay Binary

Now we load the actual code into this memory block:

1. **Import Binary File**:
   - Menu: `File → Add To Program`
   - Or right-click in Memory Map → `Import File`

2. **Select File**:
   - Navigate to: `/tmp/prot_dat_head.bin`
   - Click **Open**

3. **Configure Import**:
   ```
   Language:           MIPS:LE:32:default (should match project)
   Destination:        battle_overlay_0x801d
   Base Address:       801d0000
   Add to Memory:      [✓] Yes
   Bytes:              262144 (all bytes from file)
   ```
   - Click **OK**

4. **Confirm Import**:
   - Ghidra will show import summary
   - Click **OK** to confirm

### Step 1.4: Run Auto-Analysis

The overlay code needs to be analyzed before we can reverse engineer it:

1. **Start Auto-Analysis**:
   - Menu: `Analysis → Auto Analyze 'SCUS_942.54'`
   - Or press: `A`

2. **Select Analyzers**:
   - **Essential analyzers** (must enable):
     - [✓] Decompiler Parameter ID
     - [✓] Function Start Search
     - [✓] MIPS Constant Reference Analyzer
     - [✓] Non-Returning Functions - Discovered
     - [✓] Reference
     - [✓] Shared Return Calls
     - [✓] Stack
     - [✓] Subroutine References
   
   - **Recommended analyzers**:
     - [✓] ASCII Strings
     - [✓] Create Address Tables
     - [✓] Data Reference
     - [✓] Embedded Media
     - [✓] Function ID

3. **Start Analysis**:
   - Click **Analyze**
   - Wait for completion (may take 2-5 minutes)
   - Progress shown in bottom-right corner

### Step 1.5: Verify Import Success

Navigate to known AI callback addresses to verify they were imported correctly:

1. **Navigate to First Callback**:
   - Press `G` (Go To)
   - Enter: `801d1344`
   - Press Enter

2. **Verify Function**:
   - You should see disassembled MIPS code
   - Function should have a name like `FUN_801d1344`
   - Decompiler window (right side) should show C pseudocode

3. **Check Other Callbacks**:
   - `801d820c` - AI callback 2
   - `801e36a0` - AI callback 3  
   - `801f159c` - AI callback 4

If you see valid MIPS instructions and decompiled C code, the import was successful!

---

## Part 2: DICK Methodology for AI Callbacks

**DICK** = **D**ecompile **I**dentify **C**larify **K**nowledge

The goal is to have ZERO unnamed symbols (`FUN_*`, `param_*`, `local_*`, `DAT_*`).

### Step 2.1: Choose a Callback Function

Start with the first AI callback:

```
Address: 0x801d1344
Current Name: FUN_801d1344
Purpose: Enemy AI callback (entity->update_func)
```

### Step 2.2: Understand the Function Signature

All AI callbacks follow the PSX entity callback pattern:

```c
void enemy_ai_callback(entity_t* entity)
```

**Expected parameters**:
- `param_1` → `entity` (pointer to entity structure)

**Entity structure** (see `docs/decompilation/entity-structure.md`):
```c
typedef struct entity_t {
    struct entity_t *next;           // +0x00
    void *vftable;                   // +0x04
    void *model_data;                // +0x08
    void (*update_func)(entity_t*);  // +0x0c: THIS IS THE CALLBACK
    uint32_t flags;                  // +0x10
    // ... (24+ documented offsets)
    int16_t timer_1;                 // +0x98
    int16_t timer_2;                 // +0x9a: Auto-destruction
    int16_t timer_3;                 // +0x9c
} entity_t;
```

### Step 2.3: Rename the Function

Right-click on `FUN_801d1344` → `Edit Function Signature`

**Naming convention**:
```
Pattern: enemy_ai_<enemy_name>_callback
Example: enemy_ai_goblin_callback

If enemy name unknown:
Pattern: enemy_ai_type<ID>_callback
Example: enemy_ai_type00_callback
```

For now, use type IDs until we map enemy names:
```c
void enemy_ai_type00_callback(entity_t* entity)
```

### Step 2.4: Rename Parameters

In the decompiler window, right-click on `param_1` → `Rename Variable`:

```
Old: param_1
New: entity
```

The function signature should now look like:
```c
void enemy_ai_type00_callback(entity_t* entity)
```

### Step 2.5: Identify Entity Field Accesses

Look for patterns like:
```c
*(uint *)(entity + 0x10) = ...    // Accessing flags at offset +0x10
*(short *)(entity + 0x98) = ...   // Accessing timer_1 at offset +0x98
```

**Replace with structure access**:
1. Right-click on `entity` parameter
2. Select `Retype Variable`
3. Enter: `entity_t*`
4. Click OK

Ghidra should now show:
```c
entity->flags = ...
entity->timer_1 = ...
```

### Step 2.6: Rename Local Variables

For each `local_*` variable:

1. **Identify purpose** from usage:
   - Loop counters → `i`, `j`, `count`
   - Temporary values → `temp_*`
   - Flags/states → `is_*`, `has_*`
   - Calculations → descriptive name (`damage`, `target_hp`, etc.)

2. **Rename systematically**:
   - Start from function top
   - Work down through each local
   - Use consistent naming

**Example transformation**:
```c
// BEFORE (bad - generic names)
int local_10;
int local_14;
local_10 = entity->hp;
local_14 = entity->max_hp;
if (local_10 < local_14 / 4) {
    return ACTION_FLEE;
}

// AFTER (good - descriptive names)
int current_hp;
int max_hp;
current_hp = entity->hp;
max_hp = entity->max_hp;
if (current_hp < max_hp / 4) {
    return ACTION_FLEE;
}
```

### Step 2.7: Identify Called Functions

When the AI calls other functions:

```c
FUN_80025abc(entity, 5);  // What does this do?
```

**Steps to identify**:

1. **Navigate to function** (`Ctrl+Click` or `G` to go to `80025abc`)

2. **Analyze function purpose**:
   - Look at decompiled code
   - Check cross-references (where else is it called?)
   - Look for string references or patterns

3. **Rename based on purpose**:
   ```
   Old: FUN_80025abc
   New: set_entity_animation
   New: deal_damage_to_target
   New: calculate_attack_damage
   ```

4. **Update call site**:
   ```c
   // Before
   FUN_80025abc(entity, 5);
   
   // After
   set_entity_animation(entity, ANIM_ATTACK);
   ```

### Step 2.8: Identify Constants and Enums

Look for magic numbers:

```c
if (entity->flags & 0x200) {  // What is 0x200?
    // ...
}
```

**Create enums** for clarity:

1. **Identify constant meaning**:
   - `0x200` → Entity flag for "low HP"
   - `0x1` → Action type "attack"
   - `5` → Animation ID for attack

2. **Document in comments** (Ghidra: `;` to add comment):
   ```c
   if (entity->flags & 0x200) {  // FLAG_LOW_HP
       return 1;  // ACTION_DEFEND
   }
   ```

3. **Create header file** for reusable constants:
   ```c
   // entity_flags.h
   #define FLAG_LOW_HP     0x200
   #define FLAG_STUNNED    0x400
   
   // ai_actions.h
   #define ACTION_ATTACK   1
   #define ACTION_DEFEND   2
   #define ACTION_FLEE     3
   ```

### Step 2.9: Identify Global Variables

When AI accesses globals:

```c
DAT_8007f480 = 1;  // What is this?
```

**Steps**:

1. **Navigate to address** (`G` → `8007f480`)

2. **Check cross-references** (Ctrl+Shift+F for "Find References"):
   - Where else is it read/written?
   - What functions access it?

3. **Identify purpose** from usage context:
   - Battle state variables
   - Turn counters
   - Random number generators
   - Target selection

4. **Rename global**:
   ```
   Old: DAT_8007f480
   New: g_current_turn
   New: g_battle_state
   New: g_target_entity_id
   ```

### Step 2.10: Document AI Logic

Add **plate comments** (`;` in Ghidra) at top of function:

```c
/*
 * enemy_ai_type00_callback
 * 
 * AI behavior for [Enemy Name TBD]
 * 
 * Pattern: Aggressive melee attacker
 * - Attacks most turns
 * - Defends when HP < 25%
 * - Targets random player
 * 
 * Special behaviors:
 * - Uses special attack every 5 turns
 * - Flashes red when damaged (timer_1)
 */
void enemy_ai_type00_callback(entity_t* entity) {
    // ... function body
}
```

### Step 2.11: DICK Checklist

Before moving to next function, verify:

- [ ] Function renamed with descriptive name
- [ ] All parameters renamed (`entity`, not `param_1`)
- [ ] All local variables renamed (no `local_*`)
- [ ] All called functions identified and renamed
- [ ] All global variables renamed (no `DAT_*`)
- [ ] All constants documented with comments
- [ ] AI logic documented in plate comment
- [ ] Code structure is clear and readable

**Quality test**: Can someone read the decompiled code and understand what it does without reverse engineering experience?

---

## Part 3: AI Pattern Analysis

### Step 3.1: Identify Decision Tree Structure

Most AI follows this pattern:

```c
void enemy_ai_callback(entity_t* entity) {
    // 1. Check HP percentage
    int hp_percent = (entity->hp * 100) / entity->max_hp;
    
    // 2. Make decision based on state
    if (hp_percent < 25) {
        // Low HP behavior (flee, heal, desperate attack)
        return ACTION_FLEE;
    }
    else if (hp_percent < 50) {
        // Mid HP behavior (balanced)
        if (turn_count % 4 == 0) {
            return ACTION_SPECIAL;
        }
        return ACTION_ATTACK;
    }
    else {
        // High HP behavior (aggressive)
        return ACTION_ATTACK;
    }
}
```

**Common patterns to look for**:

1. **HP-based phases**:
   ```c
   if (hp < max_hp / 4) { /* Phase 3 */ }
   else if (hp < max_hp / 2) { /* Phase 2 */ }
   else { /* Phase 1 */ }
   ```

2. **Turn counters**:
   ```c
   if (turn_count % 5 == 0) { /* Special every 5 turns */ }
   ```

3. **Random decisions**:
   ```c
   int random = rand() % 100;
   if (random < 30) { /* 30% chance */ }
   ```

4. **Target selection**:
   ```c
   target = find_lowest_hp_player();
   target = select_random_player();
   ```

5. **Status checks**:
   ```c
   if (entity->flags & FLAG_STUNNED) { /* Can't act */ }
   if (entity->flags & FLAG_POISONED) { /* Lose HP */ }
   ```

### Step 3.2: Extract AI State Machine

Document the state machine in pseudocode:

```
State Machine: Enemy Type 00
================================

States:
  - AGGRESSIVE (HP > 50%)
  - BALANCED   (25% < HP <= 50%)
  - DESPERATE  (HP <= 25%)

State: AGGRESSIVE
  - Every turn: Attack random player (70%)
  - Every turn: Strong attack (30%)
  - Every 5 turns: Special attack

State: BALANCED
  - Every turn: Attack (50%)
  - Every turn: Defend (30%)
  - Every turn: Item use (20%)
  - Every 4 turns: Special attack

State: DESPERATE
  - First entry: Enrage (damage boost)
  - Every turn: Strong attack (80%)
  - Every turn: Flee attempt (20%)
  - Every 3 turns: Desperation special
```

### Step 3.3: Identify Target Selection Logic

```c
// Pattern 1: Random target
int target_index = rand() % num_alive_players;

// Pattern 2: Lowest HP target  
int lowest_hp = 9999;
entity_t* target = NULL;
for (entity_t* player : players) {
    if (player->hp < lowest_hp) {
        lowest_hp = player->hp;
        target = player;
    }
}

// Pattern 3: Last attacker
target = get_entity_that_last_attacked(entity);

// Pattern 4: Front row priority
target = find_first_front_row_player();
```

### Step 3.4: Identify Damage Calculation

Look for damage formulas:

```c
int base_damage = (entity->attack * 10) / 10;
int defense_reduction = target->defense / 2;
int final_damage = base_damage - defense_reduction;

// Apply variance
int variance = (rand() % 11) - 5;  // ±5%
final_damage = final_damage * (100 + variance) / 100;

// Apply to target
deal_damage(target, final_damage);
```

Document the formula:
```
Damage Formula:
  Base = (Attack × 10) / 10
  Reduction = Defense / 2
  Final = Base - Reduction
  Variance = ±5% random
```

### Step 3.5: Identify Animation/Visual Logic

```c
// Color flashing on damage
entity->target_r = 0x3fc0;  // Max red
entity->target_g = 0x0000;  // No green
entity->target_b = 0x0000;  // No blue
entity->timer_1 = 15;       // Flash for 15 frames

// Animation triggers
set_animation(entity, ANIM_ATTACK);
set_animation_speed(entity, 1.5);

// Sound effects
play_sound(SFX_ATTACK);
```

---

## Part 4: Converting to Lua Scripts

### Step 4.1: Lua Script Template

Create file: `scripts/entities/enemy_type00.lua`

```lua
-- Enemy AI: [Name] (Type 00)
-- Based on reverse-engineered PSX callback at 0x801d1344
--
-- Behavior Pattern: [Aggressive / Defensive / Balanced / Cunning]
-- Special Traits: [List unique behaviors]

-- State tracking (persistent between calls)
local phase = 1
local turn_count = 0
local special_used = false

-- Main AI callback (called every turn)
function on_update(entity)
    turn_count = turn_count + 1
    
    -- Calculate HP percentage
    local hp_percent = entity.hp / entity.max_hp
    
    -- Phase transitions
    if hp_percent < 0.25 and phase < 3 then
        phase = 3
        set_color_target(entity, 0x3fc0, 0, 0x3fc0)  -- Purple (enraged)
        return "special_enrage"
    elseif hp_percent < 0.50 and phase < 2 then
        phase = 2
        set_color_target(entity, 0x3fc0, 0x2000, 0)  -- Orange
    end
    
    -- Phase-based behavior
    if phase == 3 then
        -- Desperate phase
        if turn_count % 3 == 0 then
            return "special_desperation"
        end
        return (random() < 0.8) and "attack_strong" or "flee"
        
    elseif phase == 2 then
        -- Balanced phase
        if turn_count % 4 == 0 then
            return "special_attack"
        end
        local roll = random()
        if roll < 0.5 then return "attack" end
        if roll < 0.8 then return "defend" end
        return "use_item"
        
    else
        -- Aggressive phase
        if turn_count % 5 == 0 and not special_used then
            special_used = true
            return "special_attack"
        end
        return (random() < 0.7) and "attack" or "attack_strong"
    end
end

-- Target selection callback
function choose_target(entity)
    -- Desperate phase: target weakest player
    if phase == 3 then
        return find_weakest_target(entity)
    end
    
    -- Normal: random target
    return random_range(0, entity.alive_allies - 1)
end

-- Helper: Find player with lowest HP
function find_weakest_target(entity)
    -- TODO: Needs engine support to query all players
    -- For now: random
    return random_range(0, entity.alive_allies - 1)
end

-- Visual feedback when taking damage
function on_damage_taken(entity, damage_amount)
    -- Flash intensity based on phase
    if phase == 3 then
        set_color_target(entity, 0x3fc0, 0, 0)  -- Bright red
        set_timer(entity, 1, 30)                 -- Long flash
    else
        set_color_target(entity, 0x3000, 0, 0)  -- Normal red  
        set_timer(entity, 1, 15)                 -- Short flash
    end
end

-- Animation updates (called every frame)
function on_animation_update(entity)
    local timer = get_timer(entity, 1)
    
    if timer > 0 then
        -- Still flashing from damage
    else
        -- Restore phase color
        if phase == 3 then
            set_color_target(entity, 0x3fc0, 0, 0x3fc0)  -- Purple
        elseif phase == 2 then
            set_color_target(entity, 0x3fc0, 0x2000, 0)  -- Orange
        else
            set_color_target(entity, 0x3fc0, 0x3fc0, 0x3fc0)  -- White
        end
    end
end
```

### Step 4.2: Translation Guidelines

**PSX C Code → Lua Conversion**:

| PSX C | Lua | Notes |
|-------|-----|-------|
| `int hp_pct = hp * 100 / max_hp;` | `local hp_pct = hp / max_hp` | Use float division |
| `if (hp < max_hp / 4)` | `if hp < max_hp / 4 then` | Add `then` |
| `return ACTION_ATTACK;` | `return "attack"` | Use string actions |
| `rand() % 100 < 30` | `random() < 0.3` | 0.0-1.0 range |
| `for (i=0; i<count; i++)` | `for i = 0, count-1 do` | Inclusive range |
| `entity->hp` | `entity.hp` | Dot syntax |
| `entity->flags & 0x200` | `bit32.band(entity.flags, 0x200) ~= 0` | Bitwise ops |
| `value++` | `value = value + 1` | No `++` in Lua |
| `a && b` | `a and b` | Word operators |
| `a \|\| b` | `a or b` | Word operators |
| `!flag` | `not flag` | Word operators |
| `a != b` | `a ~= b` | Not equal |

### Step 4.3: Testing the Lua Script

Create test harness: `crates/legaia_scripting/examples/test_enemy_ai.rs`

```rust
use bevy::prelude::*;
use legaia_scripting::*;

fn main() {
    App::new()
        .add_plugins(MinimalPlugins)
        .add_plugins(CombatPlugin)
        .add_systems(Startup, test_ai)
        .run();
}

fn test_ai(mut script_engine: ResMut<ScriptEngine>) {
    // Load AI script
    script_engine
        .load_script("scripts/entities/enemy_type00.lua")
        .expect("Failed to load AI script");
    
    // Create test entity
    let context = EntityScriptContext {
        stats: ScriptStats {
            hp: 50,
            max_hp: 100,
            attack: 20,
            defense: 15,
            // ...
        },
        // ...
    };
    
    // Test AI decision
    script_engine
        .call_entity_callback("on_update", context)
        .expect("AI callback failed");
    
    println!("✓ AI script executed successfully");
}
```

Run test:
```bash
cargo run --example test_enemy_ai
```

---

## Part 5: Enemy Name Mapping

### Step 5.1: Extract MONSTER.DAT

MONSTER.DAT contains enemy stats and names. To extract it:

**Option A: Manual disc browsing**
1. Use a PSX disc tool (CDMage, UltraISO)
2. Browse PROT.DAT archive structure
3. Extract MONSTER.DAT

**Option B: Parse PROT.DAT programmatically**
1. Reverse engineer PROT.DAT file index format
2. Find MONSTER.DAT entry
3. Extract based on LBA/offset

**Option C: Use existing tools**
1. Check for Legend of Legaia modding tools online
2. Use community-created extractors

### Step 5.2: Parse MONSTER.DAT Structure

Expected structure (hypothesis):
```
Header:
  uint32 magic;          // File identifier
  uint32 num_monsters;   // Number of entries
  uint32 entry_size;     // Size of each entry

Entry[] (repeat num_monsters times):
  uint16 monster_id;     // Unique ID
  uint16 type_id;        // Links to AI callback
  char   name[32];       // English name
  uint32 hp;
  uint32 mp;
  uint16 attack;
  uint16 defense;
  uint16 speed;
  uint16 exp_reward;
  uint32 gold_reward;
  // ... more stats
```

Create parser:
```rust
// crates/psxutils/src/formats/monster_dat.rs
pub struct MonsterEntry {
    pub id: u16,
    pub type_id: u16,
    pub name: String,
    pub hp: u32,
    pub mp: u32,
    pub attack: u16,
    pub defense: u16,
    pub speed: u16,
    // ...
}

pub struct MonsterDatabase {
    pub monsters: Vec<MonsterEntry>,
}

impl MonsterDatabase {
    pub fn parse(data: &[u8]) -> Result<Self> {
        // Parse header
        // Parse entries
        // Return database
    }
}
```

### Step 5.3: Map Entity Type ID → AI Callback

From entity descriptor table analysis (see `entity-structure.md`):

```
Entity Type ID → AI Callback Address
==================================================
0x0000         → 0x801d1344  (enemy_ai_type00)
0x0001         → 0x801d820c  (enemy_ai_type01)
0x0002         → 0x801e36a0  (enemy_ai_type02)
0x0003         → 0x801f159c  (enemy_ai_type03)
...
```

Create mapping file: `docs/decompilation/enemy-ai-mapping.md`

```markdown
# Enemy AI Callback Mapping

| Type ID | Callback Address | Enemy Name | Behavior Pattern | Script File |
|---------|-----------------|------------|------------------|-------------|
| 0x0000  | 0x801d1344      | Goblin     | Aggressive       | goblin.lua  |
| 0x0001  | 0x801d820c      | Slime      | Defensive        | slime.lua   |
| 0x0002  | 0x801e36a0      | Wolf       | Pack tactics     | wolf.lua    |
| ...     | ...             | ...        | ...              | ...         |
```

### Step 5.4: Rename AI Functions

Once names are mapped, go back to Ghidra:

```
Old: enemy_ai_type00_callback
New: enemy_ai_goblin_callback

Old: enemy_type00.lua
New: goblin.lua
```

### Step 5.5: Update Lua Script Metadata

```lua
-- Enemy AI: Goblin (Type 0x0000)
-- PSX Callback: 0x801d1344
-- Behavior: Aggressive melee attacker
-- Locations: Drake Castle, Rim Elm outskirts
-- Drop: Goblin Tears (common), Short Sword (rare)
```

---

## Part 6: Automation & Tools

### Step 6.1: AI Analysis Checklist

Create checklist for each callback:

```markdown
## AI Callback: 0x801d1344

- [ ] Function imported and analyzed in Ghidra
- [ ] DICK methodology applied (all symbols renamed)
- [ ] AI pattern documented (state machine diagram)
- [ ] Damage formulas extracted
- [ ] Target selection logic documented
- [ ] Lua script created and tested
- [ ] Enemy name mapped (if available)
- [ ] Script file renamed with enemy name
- [ ] Cross-references checked (shared helper functions)
```

### Step 6.2: Batch Processing Script

For processing multiple AI callbacks:

```bash
#!/bin/bash
# scripts/extract_all_ai.sh

# List of callback addresses
CALLBACKS=(
    0x801d1344
    0x801d820c
    0x801e36a0
    0x801f159c
)

for addr in "${CALLBACKS[@]}"; do
    echo "Processing callback at $addr"
    
    # 1. Export from Ghidra (manual step)
    # 2. Generate Lua template
    # 3. Run tests
    
    echo "✓ $addr complete"
done
```

### Step 6.3: Lua Script Generator

Template generator for new AI:

```rust
// tools/generate_ai_script.rs
fn generate_lua_template(
    callback_addr: u32,
    enemy_name: Option<&str>,
    behavior_type: BehaviorType,
) -> String {
    let name = enemy_name.unwrap_or(&format!("type{:04x}", callback_addr));
    
    format!(r#"
-- Enemy AI: {name}
-- PSX Callback: 0x{callback_addr:08x}
-- Behavior: {behavior_type}

local phase = 1
local turn_count = 0

function on_update(entity)
    turn_count = turn_count + 1
    
    -- TODO: Implement AI logic based on decompiled code
    
    return "attack"
end

-- TODO: Add other callbacks as needed
    "#, name=name, callback_addr=callback_addr, behavior_type=behavior_type)
}
```

---

## Part 7: Quality Assurance

### Step 7.1: Verify AI Accuracy

Compare Lua behavior to PSX:

1. **Test Cases**:
   ```
   Test: Goblin at 100% HP
   Expected: Attack 70%, Strong Attack 30%
   Lua Result: [Test output]
   PSX Result: [Recorded gameplay]
   Match: ✓/✗
   ```

2. **Edge Cases**:
   - HP exactly at phase boundary
   - Turn counter overflow
   - Random roll at 0% and 100%
   - Dead target selection

3. **Frame-Perfect Timing**:
   - Color flash duration
   - Animation frame counts
   - Sound effect timing

### Step 7.2: Performance Testing

```rust
#[test]
fn test_ai_performance() {
    let engine = ScriptEngine::new();
    engine.load_script("scripts/entities/goblin.lua").unwrap();
    
    let start = Instant::now();
    for _ in 0..10000 {
        engine.call_entity_callback("on_update", test_context());
    }
    let duration = start.elapsed();
    
    // Should complete 10,000 calls in < 100ms
    assert!(duration.as_millis() < 100);
}
```

### Step 7.3: Documentation Completeness

Each enemy AI must have:

- [✓] Lua script file
- [✓] Behavior description
- [✓] State machine diagram  
- [✓] PSX callback address
- [✓] Test cases
- [✓] Known bugs/quirks
- [✓] Decompiled C reference (commented)

---

## Part 8: Advanced Topics

### Step 8.1: Shared AI Libraries

Extract common patterns:

```lua
-- scripts/entities/lib/ai_common.lua

-- Standard HP-based phase detection
function get_hp_phase(entity)
    local pct = entity.hp / entity.max_hp
    if pct < 0.25 then return 3 end
    if pct < 0.50 then return 2 end
    return 1
end

-- Weighted random action selection
function choose_action(weights)
    local total = 0
    for _, weight in ipairs(weights) do
        total = total + weight
    end
    
    local roll = random() * total
    local sum = 0
    for action, weight in pairs(weights) do
        sum = sum + weight
        if roll < sum then
            return action
        end
    end
end
```

Use in AI scripts:
```lua
require("scripts/entities/lib/ai_common")

function on_update(entity)
    local phase = get_hp_phase(entity)
    
    if phase == 3 then
        return choose_action({
            attack_strong = 0.8,
            flee = 0.2
        })
    end
    -- ...
end
```

### Step 8.2: Boss Multi-Phase Systems

Complex bosses may have multiple callbacks:

```
Boss: Zeto (3 phases)
  Phase 1: 0x801d5000 (Normal attacks)
  Phase 2: 0x801d5100 (Summons adds)
  Phase 3: 0x801d5200 (Enraged)
  
Transition: HP thresholds trigger callback swap
```

Handle in Lua:
```lua
-- Boss changes callback dynamically
function on_phase_transition(entity, new_phase)
    if new_phase == 2 then
        -- Load phase 2 script
        entity.callback_script = "boss_zeto_phase2.lua"
    end
end
```

### Step 8.3: Debugging Tools

Add debug output to scripts:

```lua
-- Enable debug mode
local DEBUG = true

function on_update(entity)
    if DEBUG then
        print(string.format(
            "[AI] HP: %d/%d (%.1f%%) Phase: %d Turn: %d",
            entity.hp, entity.max_hp,
            entity.hp / entity.max_hp * 100,
            phase, turn_count
        ))
    end
    
    local action = choose_action_internal(entity)
    
    if DEBUG then
        print(string.format("[AI] Action chosen: %s", action))
    end
    
    return action
end
```

---

## Summary Checklist

### For Each AI Callback:

**Ghidra Steps**:
- [ ] Import overlay at correct address
- [ ] Run auto-analysis
- [ ] Navigate to callback function
- [ ] Apply DICK methodology (rename everything)
- [ ] Document AI logic in comments
- [ ] Export decompiled C code as reference

**Analysis Steps**:
- [ ] Identify decision tree / state machine
- [ ] Extract target selection logic
- [ ] Document damage formulas
- [ ] Note special behaviors
- [ ] Map to enemy name (if available)

**Lua Translation Steps**:
- [ ] Create script file with template
- [ ] Implement main callback (`on_update`)
- [ ] Implement supporting callbacks
- [ ] Add state tracking variables
- [ ] Test with example harness
- [ ] Verify behavior matches PSX

**Documentation Steps**:
- [ ] Update AI mapping table
- [ ] Add entry to enemy database
- [ ] Create test cases
- [ ] Note any quirks or bugs
- [ ] Cross-reference related enemies

---

## Next Steps

1. **Import first overlay** following Part 1
2. **Analyze first callback** (0x801d1344) using Part 2
3. **Extract to Lua** using Part 4 template
4. **Repeat** for each callback
5. **Map names** once MONSTER.DAT is parsed (Part 5)

**Goal**: Create complete Lua AI library for all enemies, ready for modding!

---

## References

- Entity structure: `docs/decompilation/entity-structure.md`
- Scripting API: `crates/legaia_scripting/README.md`
- Example scripts: `scripts/entities/*.lua`
- Session notes: `docs/decompilation/sessions/2026-02-15-entity-callback-system-discovered.md`
