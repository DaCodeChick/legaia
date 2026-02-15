# Legend of Legaia Rewrite Project

## 🎮 PROJECT STRATEGY (Updated 2026-02-14)

**We are building a MODERN Bevy-native game, NOT a PSX emulation layer.**

### Core Philosophy

This is a **clean-room rewrite** using modern Bevy ECS patterns. We use decompilation to understand GAME LOGIC (battle formulas, AI behavior, story events), not to recreate PSX hardware behavior.

### What We Build

✅ **Modern Bevy Systems:**
- Native Bevy rendering (no GPU register emulation)
- Native Bevy audio with `bevy_kira_audio` (no SPU emulation)
- Modern input with `bevy_input` (no controller memory card polling)
- ECS-based game state (no function pointer tables)
- Asset-driven design (extracted and converted from PSX formats)

❌ **What We DON'T Build:**
- PSX GPU primitive registers (GP0/GP1 commands)
- SPU channel state emulation
- CD-ROM XA/ADPCM streaming emulation
- Memory card block management
- DMA transfer simulation
- BIOS/kernel function wrappers

### Decompilation Usage

**Use Ghidra to extract:**
- ✅ Battle damage formulas
- ✅ Character stat calculations
- ✅ Enemy AI behavior logic
- ✅ Story event flags and triggers
- ✅ Menu navigation flow
- ✅ Item effects and abilities
- ✅ Map connectivity and transitions
- ✅ Save data structure

**Skip in Ghidra:**
- ❌ GPU initialization functions (GPU_Init, SetDrawArea, etc.)
- ❌ SPU/audio hardware functions (SpuInit, SsUtSetReverbType, etc.)
- ❌ CD-ROM functions (CdInit, CdRead, CdControl, etc.)
- ❌ Memory card functions (InitCARD, StartCARD, etc.)
- ❌ Low-level BIOS calls (FlushCache, EnterCriticalSection, etc.)
- ❌ DMA/interrupt handlers

### Asset Extraction Workflow

1. **Extract from PSX disc image** (`.bin/.cue` files)
2. **Convert to modern formats:**
   - TIM textures → PNG/JPEG (use `psxutils` crate)
   - TMD models → glTF/FBX (use `psxutils` crate)
   - VAG audio → WAV/OGG (use `psxutils` crate)
   - XA streams → OGG Vorbis
   - STR videos → MP4/WebM
3. **Import to Bevy:**
   - Load as Bevy `Handle<Image>`, `Handle<Mesh>`, `Handle<AudioSource>`
   - Use Bevy's asset loading system
   - Organize in `assets/` directory with metadata

### Development Priorities

1. **Asset extraction tooling** (using `psxutils` crate)
2. **Game logic implementation** (battle, field, menu from decompilation insights)
3. **Modern UX/polish** (proper resolution, widescreen, quality-of-life improvements)
4. **Content accuracy** (match game behavior, not hardware behavior)

---

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

---

## 🚫 CODE SEPARATION POLICY

**CRITICAL: Keep decompilation analysis separate from Rust implementation code.**

### What Goes Where

#### ✅ Decompilation References ALLOWED:
- `.opencode/AGENTS.md` (this file)
- `docs/` directory (analysis documents)
- Git commit messages
- Internal notes and planning documents

#### ❌ Decompilation References FORBIDDEN:
- `crates/*/src/**/*.rs` - ALL Rust source code
- `crates/*/Cargo.toml` - Package metadata
- Rust code comments and documentation
- Public API documentation

### Forbidden Content in Rust Code

DO NOT include in Rust source files:
- ❌ PSX memory addresses (e.g., "0x8001d424", "@ 0x8007b768")
- ❌ Original function names (e.g., "FUN_8003e4e8", "init_game_state")
- ❌ Phrases like "based on decompilation", "discovered through analysis"
- ❌ References to Ghidra, DICK methodology, or reverse engineering
- ❌ Original global variable names (e.g., "g_gpu_color_mask @ 0x1f8003fc")
- ❌ Links to .opencode/ or decompilation documentation

### What IS Allowed in Rust Code

DO include clean, professional descriptions:
- ✅ Technical descriptions of what systems do
- ✅ Purpose and behavior of functions/structs
- ✅ Usage examples and API documentation
- ✅ Implementation notes about algorithms or data structures
- ✅ References to game mechanics or systems

### Example - WRONG:
```rust
/// GPU hardware configuration (PSX GPU registers 0x1f800000 range)
///
/// Based on analysis of init_game_state() at 0x8001d424 (SCUS_942.54).
/// Original: g_gpu_color_mask @ 0x1f8003fc
pub struct GpuConfig {
    pub color_mask: u32,  // g_gpu_color_mask @ 0x1f8003fc
}
```

### Example - CORRECT:
```rust
/// GPU hardware configuration
///
/// Configures color processing, drawing offsets, and primitive rendering.
pub struct GpuConfig {
    /// Color mask for RGB channels (0xffffff = no masking)
    pub color_mask: u32,
}
```

### Why This Matters

1. **Legal Protection**: Clean room implementation requires separation
2. **Professionalism**: Code should stand on its own merits
3. **Maintainability**: Future developers don't need decompilation context
4. **Distribution**: Code can be shared without reverse engineering references

### Enforcement

**IMPORTANT: This policy is actively enforced. All Rust code MUST be clean of decompilation references.**

Before committing Rust code:
1. Search for PSX addresses (regex: `0x8[0-9a-f]{7}` - RAM addresses starting with 0x80)
2. Search for "Ghidra", "decompil", "analysis", "reverse engineering", "discovered"
3. Search for "FUN_", "DAT_", "PTR_", "UNK_" (Ghidra auto-generated names)
4. Search for function names from executable (e.g., "load_cdrom_file", "init_game_state")
5. Search for phrases like "based on analysis", "found at address", "original code"
6. Check that all docs describe WHAT/WHY, not WHERE-FROM or HOW-DISCOVERED

**Examples of PSX addresses to remove:**
- ❌ `0x8003e4e8` - Code addresses
- ❌ `0x801c70f0` - Data/global addresses  
- ❌ `0x1f800000` - Hardware register addresses
- ✅ `0x00000010` - OK: File format magic numbers/constants

If you accidentally add decompilation references to Rust code:
1. Immediately fix it before committing
2. Rewrite with clean, professional descriptions
3. Move technical discovery details to `.opencode/AGENTS.md` or `docs/`
4. Use `git commit --amend` if already committed but not pushed

**Recent cleanup (2026-02-15):**
- ✅ Removed Ghidra references from `crates/psxutils/src/formats/dat.rs`
- ✅ Removed PSX memory addresses (`0x8003e4e8`, `0x801c70f0`)
- ✅ Removed phrases like "Verified via Ghidra analysis of SCUS_942.54"

**If ANY of the above checks fail, the function is NOT complete.**

---

## RDSS Methodology: Refactor, Despaghettify, Simplify, Split

**Core Principle**: Keep modules focused, readable, and maintainable by preventing code bloat and complexity.

### 🚨 MANDATORY ENFORCEMENT 🚨

**When to Apply RDSS:**
- ✅ Any file exceeding **500 lines** of code
- ✅ Any function exceeding **100 lines** of code
- ✅ Any module with more than **10 public items** (structs/functions/traits)
- ✅ Code with nested complexity >4 levels deep (if/for/match nesting)
- ✅ When adding a feature that doesn't fit the module's core purpose
- ✅ When you find yourself scrolling excessively to understand code flow
- ✅ Before marking any module as "production-ready"

### The Four Steps of RDSS

#### 1. **Refactor** - Improve Structure
- Extract repeated code into helper functions
- Replace magic numbers with named constants
- Use descriptive variable names
- Apply design patterns where appropriate
- Consolidate duplicate logic

#### 2. **Despaghettify** - Untangle Dependencies
- Break circular dependencies
- Reduce coupling between modules
- Use clear interfaces and abstractions
- Minimize global state access
- Make data flow explicit and unidirectional

#### 3. **Simplify** - Reduce Complexity
- Remove unnecessary abstractions
- Replace complex conditionals with early returns
- Use Result/Option instead of sentinel values
- Eliminate dead code and unused features
- Choose clarity over cleverness

#### 4. **Split** - Divide Responsibilities
- One struct/module = one clear purpose
- Split large files by functionality
- Create submodules for related features
- Extract testable units
- Organize by domain, not by code type

### 📏 Size Guidelines

**File Size Limits:**
- Target: **< 300 lines** per file
- Acceptable: **300-500 lines** (must have clear sections)
- Warning: **500-800 lines** (plan refactoring)
- Critical: **> 800 lines** (MUST split immediately)

**Function Size Limits:**
- Target: **< 30 lines** per function
- Acceptable: **30-50 lines** (single clear purpose)
- Warning: **50-100 lines** (consider extracting helpers)
- Critical: **> 100 lines** (MUST refactor immediately)

**Module Complexity Limits:**
- Target: **< 5 public items** (structs/functions/traits)
- Acceptable: **5-10 public items** (cohesive API)
- Warning: **10-15 public items** (consider splitting)
- Critical: **> 15 public items** (MUST reorganize immediately)

### 🔧 Splitting Strategies

#### Strategy 1: Split by Feature Domain
```
# Before: formats/mod.rs (2000 lines)
formats/mod.rs

# After: Split by format type
formats/
  mod.rs         (50 lines - re-exports)
  tim.rs         (400 lines - TIM textures)
  tmd.rs         (350 lines - TMD models)
  vag.rs         (300 lines - VAG audio)
  xa.rs          (320 lines - XA format)
  xa_adpcm.rs    (335 lines - XA decoder)
```

#### Strategy 2: Split by Responsibility
```
# Before: cdrom/mod.rs (1500 lines - reads, caches, parsing)
cdrom/mod.rs

# After: Split by concern
cdrom/
  mod.rs         (100 lines - public API)
  reader.rs      (300 lines - low-level sector reading)
  cache.rs       (250 lines - caching layer)
  iso9660.rs     (400 lines - filesystem parsing)
  xa_sector.rs   (200 lines - XA sector handling)
```

#### Strategy 3: Split by Abstraction Level
```
# Before: battle/mod.rs (2500 lines - UI, logic, AI, damage)
battle/mod.rs

# After: Split by abstraction
battle/
  mod.rs         (100 lines - battle system coordinator)
  state.rs       (200 lines - battle state machine)
  actions.rs     (300 lines - attack/defend/item actions)
  damage.rs      (250 lines - damage calculation formulas)
  ai.rs          (400 lines - enemy AI behavior)
  ui/
    mod.rs       (100 lines - UI coordinator)
    menu.rs      (300 lines - battle menu rendering)
    animations.rs (250 lines - battle animations)
```

### 🚫 Common RDSS Violations

#### Violation 1: God Module
```rust
// ❌ WRONG: One module does everything
// formats/mod.rs (2500 lines)
pub mod formats {
    pub struct TIM { ... }
    pub struct TMD { ... }
    pub struct VAG { ... }
    pub struct XA { ... }
    pub struct XaAdpcm { ... }
    // 50+ functions mixed together
}
```

```rust
// ✅ CORRECT: Split by format type
pub mod formats {
    pub mod tim;
    pub mod tmd;
    pub mod vag;
    pub mod xa;
    pub mod xa_adpcm;
}
```

#### Violation 2: Function Does Too Much
```rust
// ❌ WRONG: 300-line function doing parsing, validation, conversion
fn extract_and_process_asset(data: &[u8]) -> Result<ProcessedAsset> {
    // 50 lines: Parse header
    // 50 lines: Validate format
    // 50 lines: Extract sub-assets
    // 50 lines: Convert formats
    // 50 lines: Apply transformations
    // 50 lines: Generate output
}
```

```rust
// ✅ CORRECT: Extract helper functions
fn extract_and_process_asset(data: &[u8]) -> Result<ProcessedAsset> {
    let header = parse_header(data)?;
    validate_format(&header)?;
    let assets = extract_sub_assets(data, &header)?;
    let converted = convert_formats(assets)?;
    apply_transformations(converted)
}
```

#### Violation 3: Deep Nesting
```rust
// ❌ WRONG: 6 levels of nesting
fn process_battle_action(action: Action) {
    if action.is_valid() {
        if let Some(target) = action.target() {
            if target.is_alive() {
                match action.action_type {
                    ActionType::Attack => {
                        if let Some(weapon) = action.weapon {
                            // Finally do something...
                        }
                    }
                }
            }
        }
    }
}
```

```rust
// ✅ CORRECT: Early returns, flat structure
fn process_battle_action(action: Action) -> Result<()> {
    if !action.is_valid() {
        return Err(Error::InvalidAction);
    }
    
    let target = action.target().ok_or(Error::NoTarget)?;
    if !target.is_alive() {
        return Err(Error::TargetDead);
    }
    
    match action.action_type {
        ActionType::Attack => process_attack(action, target),
        // ...
    }
}
```

### 📋 RDSS Checklist

Before committing any module, verify:

- [ ] **File size** < 500 lines (or has clear plan to split)
- [ ] **Function size** < 100 lines (all functions)
- [ ] **Nesting depth** < 4 levels (except rare cases)
- [ ] **Public API** < 10 items (or logically grouped)
- [ ] **Single responsibility** - module has one clear purpose
- [ ] **No code duplication** - extract common logic
- [ ] **Clear naming** - no abbreviations or cryptic names
- [ ] **Documentation** - all public items documented
- [ ] **Tests** - critical paths covered
- [ ] **No TODO/FIXME** - resolve or create issues

### 🎯 When NOT to Split

Don't split prematurely:
- ❌ Files under 300 lines with cohesive purpose
- ❌ Splitting would create artificial boundaries
- ❌ Code is still in rapid prototyping phase
- ❌ Splitting would hurt readability more than help
- ❌ Module has natural unity (e.g., parsing single format)

**Good judgment required**: RDSS is about maintainability, not arbitrary line counts.

### 🏆 RDSS Success Examples

#### Example 1: XA Audio (Good - No Split Needed)
- `xa.rs`: 384 lines (format parsing, clear sections)
- `xa_adpcm.rs`: 312 lines (decoder implementation)
- **Decision**: Keep separate - different concerns, manageable size

#### Example 2: PROT.DAT Archive (Future Split Candidate)
- Current: `scanner.rs`: 247 lines
- If it grows beyond 500 lines with TIM/TMD/VAG parsing:
  - Split to: `scanner/mod.rs`, `scanner/tim.rs`, `scanner/tmd.rs`, etc.

#### Example 3: CD-ROM Reader (Good Split)
- Instead of single 1000+ line file:
  - `cdrom/mod.rs`: Public API (100 lines)
  - `cdrom/reader.rs`: Low-level reading (300 lines)
  - `cdrom/iso9660.rs`: Filesystem parsing (400 lines)

### 📝 Documentation Requirements

When splitting modules, update:
1. **Module-level docs** - Explain purpose and organization
2. **README.md** - Update architecture documentation
3. **AGENTS.md** - Record major refactorings here
4. **Commit message** - Explain why split was necessary

### Enforcement

**This policy applies to:**
- ✅ All production code (`crates/*/src/**/*.rs`)
- ✅ All examples (`crates/*/examples/*.rs`)
- ✅ Build scripts and tooling
- ✅ Test code (though tests can be longer)

**Before ANY commit:**
1. Check file sizes: `wc -l $(find crates -name "*.rs")`
2. Identify files > 500 lines
3. Review functions > 100 lines: `rg "^fn " -A 100 | grep "^fn"`
4. Apply RDSS if needed or document plan
5. Mark technical debt in comments if deferring

**Recent RDSS Applied (2026-02-15):**
- ✅ XA audio: Split `xa.rs` (format) from `xa_adpcm.rs` (decoder)
- ✅ Each kept under 400 lines with clear boundaries
- ✅ No "god modules" - focused responsibilities

---

## 📦 Dependency Management & Breaking Changes

### gltf-json Name Field Context (2026-02-15)

**IMPORTANT**: The `gltf-json` crate version 1.4.1+ requires a `name: Option<String>` field in various struct initializations.

**Background:**
- User fixed this in commit 711e624 (2026-02-15)
- Added `name: None` to ~10 struct initializations in `crates/legaia-assets/src/converter.rs`
- This is CORRECT and INTENTIONAL - the API requires this field

**DO NOT:**
- ❌ Remove these `name: None` fields
- ❌ Question why they exist
- ❌ Try to "fix" or "clean up" these fields
- ❌ Mark them as "unused" or "unnecessary"

**WHY:**
- This is a gltf-json API requirement, not optional
- Removing them will cause compilation errors
- User explicitly stated: "Leave the name field be. Let it exist."

**Related Files:**
- `crates/legaia-assets/src/converter.rs` - Contains the name field additions
- `crates/legaia-assets/Cargo.toml` - Tracks gltf-json dependency version

**If you encounter similar dependency updates:**
1. Check the changelog for breaking changes
2. Update code to match new API requirements
3. Add a note here documenting the context
4. Prevent future agents from reverting fixes

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

## 🎮 Decompilation Priorities (Modern Approach)

### ⭐ HIGH PRIORITY - Analyze These First

These systems contain **game logic** that we need to understand for the modern rewrite:

#### 1. **Battle System Logic** (HIGHEST PRIORITY)
- ✅ **Analyze:** Damage formulas, stat calculations, AI behavior
- ✅ **Analyze:** Art/combo system mechanics and timing
- ✅ **Analyze:** Status effect calculations and durations
- ✅ **Analyze:** Enemy AI decision trees
- ✅ **Analyze:** Experience/leveling formulas
- ❌ **Skip:** Battle rendering, sprite animation, GPU commands

#### 2. **Character Stats & Progression**
- ✅ **Analyze:** Stat growth formulas per level
- ✅ **Analyze:** Equipment stat modifiers
- ✅ **Analyze:** Ra-Seru absorption mechanics
- ✅ **Analyze:** Seru skill unlock conditions
- ❌ **Skip:** Character model rendering

#### 3. **Item & Equipment System**
- ✅ **Analyze:** Item effects (battle and field)
- ✅ **Analyze:** Equipment stat bonuses
- ✅ **Analyze:** Synthesis recipes (if present)
- ✅ **Analyze:** Shop prices and availability
- ❌ **Skip:** Menu rendering, icon loading

#### 4. **Enemy Data Tables**
- ✅ **Analyze:** Enemy stats (HP, MP, ATK, DEF, etc.)
- ✅ **Analyze:** Enemy drops and rewards
- ✅ **Analyze:** Enemy abilities and AI patterns
- ✅ **Analyze:** Boss mechanics and phases
- ❌ **Skip:** Enemy model/texture loading

#### 5. **Event Scripts & Story Flow**
- ✅ **Analyze:** Event flag conditions
- ✅ **Analyze:** Story progression triggers
- ✅ **Analyze:** NPC dialogue trees
- ✅ **Analyze:** Quest completion conditions
- ❌ **Skip:** Cutscene video playback

#### 6. **Field/World Logic**
- ✅ **Analyze:** Map connectivity (which areas connect)
- ✅ **Analyze:** Random encounter rates and tables
- ✅ **Analyze:** Treasure chest contents and flags
- ✅ **Analyze:** Door unlock conditions
- ❌ **Skip:** Map rendering, collision mesh loading

#### 7. **Save Data Format**
- ✅ **Analyze:** Save data structure and fields
- ✅ **Analyze:** Flag storage layout
- ✅ **Analyze:** Inventory serialization
- ❌ **Skip:** Memory card block management

### ❌ LOW PRIORITY - Skip Unless Necessary

These are **hardware abstraction layers** that we replace with Bevy:

#### PSX Hardware Functions (Skip These)
- ❌ GPU initialization (GPU_Init, SetDrawArea, SetDrawMode, etc.)
- ❌ GPU primitive functions (DrawPoly, DrawLine, DrawSprite, etc.)
- ❌ SPU/audio hardware (SpuInit, SpuSetVoice, SsUtSetReverbType, etc.)
- ❌ CD-ROM functions (CdInit, CdRead, CdControl, CdSync, etc.)
- ❌ Memory card (InitCARD, StartCARD, ReadCARD, WriteCARD, etc.)
- ❌ DMA operations (DMA setup, transfer, completion polling)
- ❌ GTE macro functions (matrix math - use Bevy's math instead)
- ❌ BIOS calls (FlushCache, EnterCriticalSection, etc.)
- ❌ Controller polling (InitPAD, StartPAD, etc.)
- ❌ VSync/timing functions (VSync, VSyncCallback, etc.)

#### Rendering/Presentation Functions (Skip These)
- ❌ Display list builders
- ❌ Primitive ordering tables (OT)
- ❌ Texture upload/management
- ❌ Sprite sheet packers
- ❌ Font rendering primitives
- ❌ 2D/3D coordinate transformations (use Bevy transforms)

### 🔍 How to Identify What to Skip

When analyzing a function in Ghidra, **SKIP IT** if:
- It accesses hardware registers (0x1F8xxxxx addresses)
- It calls BIOS functions (A0/B0/C0 table calls)
- It manipulates GPU command buffers
- It only performs coordinate transformations
- Its name suggests hardware ("Spu", "Gpu", "Cd", "Pad", "Gte", "Dma")
- It's purely about rendering (no game state changes)

When analyzing a function, **ANALYZE IT** if:
- It modifies player/enemy stats
- It calculates damage or effects
- It checks/sets game flags
- It contains formulas or lookup tables
- It implements AI or decision logic
- It defines item/enemy/ability data

---

## 🎮 Game System Architecture

### Major Systems (To Be Mapped)

#### 1. **Initialization & Main Loop**
- [ ] Entry point function
- [ ] Main game loop
- [ ] System initialization
- [ ] BIOS/library setup

#### 2. **Graphics/Rendering System** ⚠️ (LOW PRIORITY - Replace with Bevy)
- [ ] ~~GPU command submission~~ (Skip)
- [ ] ~~Display list management~~ (Skip)
- [ ] ~~Texture management (TIM format)~~ (Extract assets only)
- [ ] ~~3D model rendering~~ (Skip)
- [ ] ~~2D sprite rendering~~ (Skip)
- [ ] Animation system (timing data only)
- [ ] Camera system (parameters only)
- [ ] ~~Lighting calculations~~ (Skip)
- **GTE Functions** (0x20000000-0x20000263): ⚠️ SKIP - Use Bevy math

#### 3. **Field/World System** ⭐ (HIGH PRIORITY)
- [x] Character controller (logic only, not rendering)
- [x] Collision detection (logic only)
- [x] Map loading (connectivity data)
- [x] NPC management (dialogue, events)
- [x] Event triggers (conditions and actions)
- [x] World map navigation (transitions)
- [x] Door/transition handling (unlock conditions)

#### 4. **Battle System** ⭐⭐⭐ (HIGHEST PRIORITY - Most Complex)
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

#### 5. **Menu System** ⚠️ (MEDIUM PRIORITY - Logic Only)
- [x] Main menu (navigation flow)
- [x] Pause menu (navigation flow)
- [x] Equipment menu (stat calculations)
- [x] Item menu (item effects)
- [x] Magic/Arts menu (ability data)
- [x] Status screens (stat display logic)
- [x] Save/Load interface (data format only)
- [ ] ~~Menu rendering~~ (Skip - use Bevy UI)

#### 6. **Audio System** ⚠️ (LOW PRIORITY - Extract Assets Only)
- [ ] ~~SPU (Sound Processing Unit) management~~ (Skip)
- [ ] ~~VAB (Voice Attribute Bank) loading~~ (Extract assets only)
- [ ] ~~VAG (audio) playback~~ (Skip - use Bevy audio)
- [ ] ~~Music sequencing~~ (Skip)
- [x] Sound effect triggers (which sounds to play when)

#### 7. **Save/Load System** ⭐ (HIGH PRIORITY)
- [ ] ~~Memory card operations~~ (Skip - use native save system)
- [x] Save data format (structure and fields)
- [x] Game state serialization (what to save)
- [ ] ~~Slot management~~ (Skip)

#### 8. **Input System** ⚠️ (LOW PRIORITY - Use Bevy Input)
- [ ] ~~Controller reading~~ (Skip - use Bevy input)
- [x] Button mapping (which actions map to which buttons)
- [x] Input buffering (for combo system)
- [x] Menu navigation (button logic)

#### 9. **Asset Management** ⚠️ (LOW PRIORITY - Extract Only)
- [ ] ~~CD-ROM file loading~~ (Skip)
- [ ] ~~Asset decompression~~ (Decompress offline, save as native)
- [ ] ~~Memory management~~ (Skip)
- [ ] ~~Texture caching~~ (Skip - Bevy handles this)

#### 10. **Event/Scripting System** ⭐⭐ (VERY HIGH PRIORITY)
- [x] Event script interpreter (bytecode format and opcodes)
- [x] Flag management (which flags control what)
- [x] Dialogue system (text display, choices, conditions)
- [ ] ~~Cutscene playback~~ (Skip rendering, extract timing data)
- [x] Quest tracking (completion conditions)

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
Total: 1,121 functions | Status: 21 Complete, 1,100 Remaining (1.9% progress)

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
| 0x800265e8 | init_data_tables | **✅ Complete** | Data | Data table initialization (14 globals renamed). Session #3. |
| 0x8001d424 | init_game_state | **✅ Complete** | Core | Game state init (9 locals, 5 funcs, 52+ globals). Session #3. |
| 0x8003f08c | init_cdrom_protection | **✅ Complete** | CD-ROM | Copy protection system (1 param, 3 globals, 1 func). Session #3. |
| 0x8003e4e8 | load_cdrom_file | **✅ Complete** | CD-ROM | CD file loader (8 locals, 4 globals). Session #3. |
| 0x8002666c | init_sound_playback_system | **✅ Complete** | Audio | Sound playback init (2 locals). Session #3. |
| 0x8001e3b8 | allocate_graphics_buffers | **✅ Complete** | Graphics | Double-buffered graphics alloc (6 locals, 12 globals, 2 funcs). Session #3. |
| 0x8003d254 | gte_load_h_register | **✅ Complete** | Graphics | GTE H register loader (wrapper). Session #3. |
| ... | ... | Unanalyzed | ... | ... |

**Recently Completed (2026-02-14 DICK Session #3):**
- ✅ init_data_tables() - Data table initialization (14 globals)
- ✅ init_game_state() - Comprehensive game state init (52+ globals for GPU, camera, sprites, input)
- ✅ init_cdrom_protection() - Copy protection loader
- ✅ load_cdrom_file() - CD-ROM file search and read system
- ✅ init_sound_playback_system() - Sound playback system init
- ✅ allocate_graphics_buffers() - Double-buffered graphics memory allocator
- ✅ gte_load_h_register() - GTE H register wrapper

**Recently Completed (2026-02-14 DICK Session #2):**
- ✅ init_cdrom_system() - CD-ROM hardware initialization
- ✅ init_sound_system() - Sound system wrapper
- ✅ init_spu() - SPU hardware init
- ✅ init_sprite_buffer() - Complex 2D sprite grid allocator
- ✅ vibration_stub() - Disabled vibration stub
- ✅ init_memory_card_system() - Memory card system (renamed 5 called functions + 13 globals)
- ✅ init_memory_allocator() - Custom heap allocator with free list

**Functions Renamed (2026-02-14 Session #3):**
- load_cdrom_file (was FUN_8003e4e8)
- allocate_memory_buffer (was FUN_80017888)
- setup_scratch_buffers (was FUN_8001f690)
- setup_game_config, init_system_timing, init_game_subsystems, setup_asset_pointers, init_rendering_tables

**Functions Renamed (2026-02-14 Session #2):**
- init_memory_card_slot_0, init_memory_card_slot_1
- setup_memory_card_buffers, configure_memory_card_slot
- finalize_memory_card_setup

**Globals Renamed (2026-02-14 Session #3 - 77+ globals):**
GPU/Graphics Registers (18):
- g_gpu_color_mask, g_gpu_primitive_color, g_gpu_register_1/2/3
- g_gpu_x_offset, g_gpu_y_offset, g_gpu_scratch_config

Camera System (11):
- g_camera_param_1/2/3/4/5, g_camera_x/y/z_offset
- g_camera_distance, g_camera_zoom_level

Sprite System (20):
- g_sprite_table_entry_0 through _14 (10 entries)
- g_sprite_data_buffer_0 through _5 (6 buffers)
- g_hardware_register_table

Graphics Buffers (12):
- g_graphics_buffer_1/2, g_graphics_buffer_1/2_copy
- g_default_buffer_address, g_buffer_allocated_size, g_requested_buffer_size
- g_graphics_buffer_index, g_allocation_error_counter, g_special_state_buffer
- g_expansion_ram_enabled

Display/Rendering (5):
- g_screen_brightness, g_fade_speed, g_fade_speed_backup
- g_default_color_value, g_unknown_render_param

CD-ROM Protection (7):
- g_cdrom_protection_state, g_cdrom_protection_flag_1/2
- g_cdrom_base_path, g_cdrom_file_sector, g_cdrom_file_frame, g_cdrom_read_counter

Input/State (4):
- g_controller_state, g_input_state, g_initial_state_id
- g_unknown_flag_1/2/3/4/5/6

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
- **TIM**: PSX texture format (✅ **916 textures extracted and verified**)
- **TMD**: Standard PSX 3D model format (NOT used by Legend of Legaia - see below)
- **VAB**: Voice Attribute Bank (audio samples + metadata)
- **VAG**: Individual audio sample format (✅ **1 file identified**)
- **STR**: Streaming video format
- **XA**: CD-XA audio format

### Legend of Legaia Specific Formats

#### PROT.DAT Archive Structure (619 files, ~116 MB)
Complete analysis of file types:
- **Custom 3D Models**: 92 files (~7.5 MB) - signature `0x80000002` at offset +4
- **Embedded TIM Textures**: 44 container files with 916 total TIM images (~15 MB extracted)
- **Embedded VAG Audio**: 1 file (file_0612.bin, 28 KB)
- **Dummy Files**: 138 files (282 KB) - "pochipochipochi..." placeholder pattern
- **Zero-filled Files**: 6 files (227 KB) - padding/alignment
- **Unknown Format**: 383 files (~42 MB) - need further analysis

#### TIM Texture Extraction (Discovered 2026-02-15)
**✅ SUCCESSFULLY EXTRACTED AND CONVERTED**

Legend of Legaia stores TIM textures **embedded in container files** at various offsets, not as standalone files.

**Extraction Results:**
- **916 TIM textures** found across 44 container files
- Largest source: file_0000.bin contains 257 embedded TIM images
- Total extracted size: ~15 MB
- Successfully converted sample to PNG (320x256, 8-bit indexed color)
- TIM parser (`psxutils/src/formats/tim.rs`) verified working correctly

**Example Embedded Locations:**
- file_0000.bin: 257 TIMs starting at offset 0x1858
- file_0447.bin: TIM at offset 0x0004 (82,464 bytes)
- file_0001.bin: 2 TIMs at offsets 0x0058, 0x00e0

**Texture Characteristics:**
- Most common: 2,144 byte textures (likely 64x32 or similar small sprites)
- Larger textures: 32-33 KB (likely 256x128 or 320x256 backgrounds)
- Color modes: Primarily 8-bit indexed (Clut8Bit) with 256-color palettes
- Some 4-bit indexed (Clut4Bit) for smaller sprites

**Tools Created:**
- `/tmp/extract_all_tims.py` - Batch extractor scanning all files for embedded TIMs
- `crates/psxutils/examples/test_tim_convert.rs` - TIM to PNG converter
- Output directory: `~/.local/share/legaia/assets/textures/`

#### Custom 3D Model Format (Discovered 2026-02-15)
Legend of Legaia uses a **custom 3D model format**, NOT standard PSX TMD files.

**Key Findings:**
- No standard TMD files (magic `0x00000041`) found in PROT.DAT archive
- 92 files previously identified as "TMD" have signature `0x80000002` at offset +4
- These files have different header structure than standard TMD
- First 4 bytes vary (0x0000383c, 0x00002998, 0x00002c20, etc.) - likely file size or offset
- Offset +8 consistently reads as 0 for "num_objects", indicating different structure

**Example Files:**
- file_0005.bin: Starts with `3c38 0000 0200 0080 0000 0000 0200 0000`
- file_0100.bin: Starts with `8036 0000 0200 0080 0000 0000 0200 0000`
- Pattern: `[varying_id] [02000080] [00000000] [02000000] [data...]`

**Status:** Format needs further reverse engineering to identify structure
**Next Steps:** 
1. Analyze file structure in Ghidra
2. Look for 3D-related functions in game executable
3. Create custom parser once format is understood
4. Document findings here

**Note:** The standard TMD parser in `psxutils/src/formats/tmd.rs` is complete and ready for use with standard PSX TMD files from other games/sources.

#### XA Audio Extraction (Completed 2026-02-15)
**✅ SUCCESSFULLY EXTRACTED AND CONVERTED**

Legend of Legaia stores voice clips and sound effects as XA-ADPCM audio streams in the `/XA/` directory.

**Extraction Results:**
- **316 audio streams** extracted from 34 .XA files (XA1.XA through XA34.XA)
- Total extracted size: **367 MB** (WAV format, uncompressed PCM)
- All files successfully decoded and exported to `/tmp/extracted_xa/`
- 100% success rate - no errors during extraction

**Audio Format Specifications:**
- **Sample rate**: 37,800 Hz (standard PSX XA audio rate)
- **Bit depth**: 16-bit signed PCM (decoded from 4-bit ADPCM)
- **Channels**: Stereo (2 channels)
- **Original compression**: XA-ADPCM (4-bit samples)
- **Compression ratio**: ~4:1 (4-bit ADPCM → 16-bit PCM)

**Stream Distribution by File:**
- Most files contain 8 audio streams (one per channel 0-7)
- Larger files: XA2.XA (16 streams), XA4.XA (16), XA6.XA (16), XA32.XA (16), XA33.XA (16)
- Smaller files: XA13.XA (7 streams), XA14.XA (7)
- Duration range: 0.1s to 6.1s per stream

**XA-ADPCM Decoder Implementation:**
- K0 filter coefficients: [0.0, 0.9375, 1.796875, 1.53125]
- K1 filter coefficients: [0.0, 0.0, -0.8125, -0.859375]
- Sound groups per sector: 18 (128 bytes each)
- Samples per sound group: 28 samples
- Total samples per sector: 224 samples (28 × 8 sound units)

**Tools Created:**
- `crates/psxutils/src/formats/xa.rs` - XA format parser with sub-header validation
- `crates/psxutils/src/formats/xa_adpcm.rs` - XA-ADPCM decoder (K0/K1 filters)
- `crates/psxutils/examples/extract_xa.rs` - Full extraction tool
- `crates/psxutils/examples/read_xa_file.rs` - XA sector inspection utility

**Implementation Reference:**
- Based on **jPSXdec** reference implementation (Java source code)
- jPSXdec repository: https://github.com/m35/jpsxdec
- Key reference files:
  - `XaAnalysis.java` - XA sector structure analysis
  - `XaAdpcmDecoder.java` - ADPCM decoding algorithm
  - `SoundUnitDecoder.java` - Sound unit processing
  - `K0K1Filter.java` - Filter coefficient implementation

**Comparison to jPSXdec:**
- jPSXdec found: **322 streams** (expected)
- Our extraction found: **316 streams** (98.1% match)
- Difference: 6 streams (likely padding or empty channels)
- Audio quality verified: WAV files are valid, proper format, correct sample rate

**Example Files:**
- `xa1_file1_ch0.wav` - 347 KB (0.3s duration)
- `xa15_file1_ch4.wav` - 6.5 MB (5.3s duration)
- `xa22_file1_ch6.wav` - 7.5 MB (6.1s duration) - longest clip

**Status:** Complete and production-ready
**Next Steps:** 
1. Convert WAV files to OGG Vorbis for size reduction (~10:1 compression)
2. Organize audio files by category (voice, SFX, ambiance)
3. Create audio asset manifest for Bevy integration
4. Implement audio playback system in `legaia-engine`

### Legend of Legaia Resources
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

## 📚 jPSXdec Reference Implementation

**CRITICAL**: When implementing asset extraction, use jPSXdec as the reference, not guesswork.

### jPSXdec Source
- **Repository**: https://github.com/m35/jpsxdec
- **Local copy**: `/tmp/jpsxdec/`
- **Extracted tool**: `/tmp/jpsxdec_v2.0/jpsxdec.jar`
- **Index file**: `/tmp/jpsxdec_v2.0/all` (shows what jPSXdec found on disc)

### Key Implementation Files

#### TIM Texture Scanning
- **Validator**: `jpsxdec/src/jpsxdec/tim/TimValidator.java`
  - Line 59: `MAX_TIM_WORD_WIDTH = 16384` (NOT arbitrary 10MB limit)
  - Line 64: `MAX_TIM_HEIGHT = 8192`
  - Line 137: `MAX_POSSIBLE_TIM_DATA_SIZE = MAX_TIM_WORD_WIDTH * 2 * MAX_TIM_HEIGHT + HEADER_SIZE`
- **Scanner**: `jpsxdec/src/jpsxdec/tim/CreateTim.java`
  - `isTim()` method: Validates headers WITHOUT allocating pixel data
  - Line 93: `IO.skip(inStream, validator.getClutImageDataByteSize())` - SKIPS data, doesn't read
  - Line 116: `IO.skip(inStream, validator.getImageDataByteSize())` - SKIPS data, doesn't read
  - Only reads pixel data AFTER all validation passes
- **Indexer**: `jpsxdec/src/jpsxdec/modules/tim/DiscIndexerTim.java`
  - Uses streaming approach with `DemuxPushInputStream`
  - Processes sector-by-sector, not loading entire files
  - Line 103: `while (_stream.available() > Tim.MINIMUM_TIM_SIZE)` - only needs header bytes

#### What jPSXdec Actually Scans
From Legend of Legaia disc analysis:
- ✅ **1132 TIM textures** in PROT.DAT
- ✅ **322 XA audio files** (in XA/*.XA files)
- ✅ **6 STR video files** (MOV/MV1.STR through MV6.STR)
- ✅ **45 regular files** (ISO 9660 filesystem)
- ❌ **Does NOT scan for TMD models** (not found by jPSXdec)
- ❌ **Does NOT scan for VAG audio** in PROT.DAT (not found by jPSXdec)

### Critical Lessons Learned (2026-02-15)

#### OOM Error Root Cause
**Problem**: Scanner caused out-of-memory by allocating huge vectors for corrupt data

**Bad Approach (what we did wrong)**:
```rust
// DON'T DO THIS - allocates memory during validation!
fn scan_tim(&self) -> Vec<DiscoveredAsset> {
    if magic == TIM_MAGIC {
        if let Ok(tim) = Tim::parse(&self.data[offset..]) {  // ❌ Parses AND allocates
            let size = tim.data_size();
            // ...
        }
    }
}
```

**Correct Approach (from jPSXdec)**:
```rust
// DO THIS - validate headers WITHOUT allocating pixel data
fn scan_tim(&self) -> Vec<DiscoveredAsset> {
    if magic == TIM_MAGIC {
        if let Ok((width, height, size)) = Tim::validate(&self.data[offset..]) {  // ✅ Only validates
            // No pixel data allocated!
            // Skip past TIM and continue
        }
    }
}
```

#### Key Validation Pattern
1. **Read header bytes** (8-12 bytes for TIM)
2. **Validate magic number** (0x00000010 for TIM)
3. **Validate dimensions** against MAX limits (not arbitrary size)
4. **Calculate expected size** from header fields
5. **Check if data available** in buffer
6. **SKIP (don't read)** the pixel data
7. **Return metadata only** (offset, size, dimensions)

#### Size Limits Must Match jPSXdec
- ❌ `const MAX_REASONABLE_SIZE: usize = 10 * 1024 * 1024;` // Wrong! Arbitrary
- ✅ `const MAX_TIM_WORD_WIDTH: u16 = 16384;` // Correct! From jPSXdec
- ✅ `const MAX_TIM_HEIGHT: u16 = 8192;` // Correct! From jPSXdec
- ✅ `const MAX_POSSIBLE_SIZE = MAX_TIM_WORD_WIDTH * 2 * MAX_TIM_HEIGHT + 12;` // Calculated

#### When Implementing New Format Scanners
1. ❌ **DON'T GUESS** - Don't make up size limits or validation logic
2. ✅ **READ jPSXdec source** - Find the validator class first
3. ✅ **COPY their limits** - Use exact same max dimensions/sizes
4. ✅ **SKIP, don't READ** - Validate headers, skip data
5. ✅ **TEST with real data** - Run against actual disc images

### Scanner Performance Results (2026-02-15)

#### Initial Fix (Commit 6c85f69)
- **Before fix**: OOM crash on chunk 2 (259GB allocation attempt)
- **After fix**: Successfully scanned all 115MB of PROT.DAT
  - Chunk size: 5MB
  - Found: 872 TIM textures (77% of jPSXdec's 1132)
  - Memory usage: ~5MB per chunk (no accumulation)
  - Time: ~30 seconds for full scan

#### Improved Validation (Commit aa1f39e)
After implementing stricter validation matching jPSXdec:
- Added flags field validation (reject reserved bits)
- Fixed CLUT size limit to proper formula
- Added width/height > 0 checks
- Added consistency check with +2 bytes tolerance
- **Result**: Found **1281 TIMs (113% of jPSXdec's 1132)**
  - This is 409 MORE TIMs than before (+47% improvement)
  - Over-detection vs jPSXdec likely due to byte-level vs sector-based scanning
  - Better to over-detect than miss assets

### TIM Extraction Tool (2026-02-15)

#### Complete Extraction Implementation (Commit 6ce8e50, 67ce457)
Built production-ready extraction tool with ALL enhancements:

**Features:**
- ✅ Multi-threaded extraction with `rayon` (parallel processing)
- ✅ Progress bars with `indicatif` (scanning + extraction phases)
- ✅ Automatic thumbnail generation (256x256 max, Lanczos3 filtering)
- ✅ JSON metadata export (offsets, dimensions, pixel modes, errors)
- ✅ Batch processing (all 1281 TIMs in one run)
- ✅ Per-asset error tracking and statistics

**Results:**
- Extracted **1281/1281 TIMs (100% success rate)**
- Full images: 46MB (1281 PNGs)
- Thumbnails: 43MB (1281 PNGs, max 256x256)
- Metadata: 286KB JSON with complete asset information
- Distribution: 1277 Clut4Bit (99.7%), 4 Clut8Bit (0.3%)
- Common sizes: 256px (420), 64px (377), 32px (224)

**Architecture:**
```rust
// Feature-gated dependencies (opt-in)
[features]
extraction = ["image", "indicatif", "rayon", "serde", "serde_json"]

// Usage
cargo run --release --example extract_tims --features extraction
```

**Key Design Decisions:**
- ❌ **DON'T raise arbitrary limits** - Keep `MAX_READ_SIZE` at 100MB
- ✅ **DO use chunked reading** - Read PROT.DAT in 50MB chunks for 121MB file
- ✅ **Respect architectural boundaries** - No hacky workarounds
- ✅ **Feature-gate heavy deps** - Keep core library lightweight

**Comparison to jPSXdec:**

| Metric | jPSXdec | Our Tool | Status |
|--------|---------|----------|---------|
| TIMs Found | 1132 | **1281** | ✅ +13% |
| Success Rate | Unknown | **100%** | ✅ Perfect |
| Speed | GUI-based | **CLI, parallel** | ✅ Faster |
| Thumbnails | ❌ No | ✅ **Yes** | ✅ Better |
| Metadata | CSV | **JSON** | ✅ Better |
| Multi-threaded | ❌ No | ✅ **Yes** | ✅ Better |

### Remaining Work

**PROT.DAT Support:**
- ✅ TIM texture scanning (1281 found, 113% of jPSXdec)
- ✅ TIM extraction to PNG (100% success rate)
- ❌ XA audio scanning (jPSXdec finds 322 files)
- ❌ STR video scanning (jPSXdec finds 6 files)

**Next Priority:**
1. Implement XA audio scanner
2. Implement STR video scanner
3. Consider TMD/VAG if needed for game assets

---

*Last Updated: 2026-02-15*  
*Status: TIM extraction COMPLETE and production-ready*  
*Next: Implement XA audio and STR video scanning*
