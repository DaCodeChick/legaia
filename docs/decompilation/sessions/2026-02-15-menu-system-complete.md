# Menu System Analysis - Complete (2026-02-15)

## Session Overview

**Goal**: Achieve 100% DICK compliance for `process_menu_action` @ 0x80030628, the main menu item filtering and list building function.

**Status**: ✅ **100% DICK COMPLIANT** - All 77 symbols renamed

**Function Size**: 3KB (~500 lines of decompiled C code)

## What This Function Does

`process_menu_action` is a **menu item populator** that filters and builds item/skill lists for different menu types in both battle and field contexts. It handles:

- Item filtering by usability, status, and context (battle vs field)
- Equipment selection and sorting by rating
- Art/skill filtering by MP cost and learned status
- Shop purchase list generation with affordability checks
- Travel/warp location menu building
- Inventory management (discard items)

**IMPORTANT**: This is a **UI/menu function**, NOT combat logic. It does NOT contain:
- Damage formulas
- AI decision-making
- Turn management
- Status effect calculations
- Combat execution logic

## Symbols Renamed (77 Total)

### 1. Function Renamed (1)
- `FUN_80030628` → `process_menu_action`

### 2. Parameters Renamed (1)
- `param_1` → `menu_context`

### 3. Global Variables Renamed (39)

**Menu Buffers:**
- `DAT_8007bb28` → `g_menu_item_buffer_main` - Primary buffer for filtered menu items
- `DAT_8007bb68` → `g_menu_item_buffer_temp` - Temporary buffer for secondary sorting
- `DAT_8007bba8` → `g_menu_item_buffer_special` - Buffer for special/key items
- `DAT_8007bb2a` → `g_menu_item_buffer_main_2` - Secondary main buffer entry

**Character Data:**
- `DAT_8007bcf4` → `g_selected_character_index` - Current character (0=Vahn, 1=Noa, 2=Gala)
- `DAT_8007bcc8` → `g_active_party_size` - Number of characters in party (0-3)
- `DAT_80083fc1` → `g_character_arts_learned_count` - Arts learned per character
- `DAT_80083fc2` → `g_character_arts_learned_list` - List of learned art IDs
- `DAT_80084160` → `g_character_current_mp` - Current MP per character
- `DAT_8008414e` → `g_character_status_flags` - Status effect flags (poison, seal, etc.)
- `DAT_80083f89` → `g_character_art_data_base` - Base character art data array

**Equipment Slots:**
- `DAT_80083fc5` → `g_character_weapon_equipped` - Weapon ID equipped per character
- `DAT_80083fc6` → `g_character_armor1_equipped` - Armor slot 1 ID
- `DAT_80083fc7` → `g_character_armor2_equipped` - Armor slot 2 ID
- `DAT_80083fc8` → `g_character_armor3_equipped` - Armor slot 3 ID
- `DAT_80083fc4` → `g_character_helmet_equipped` - Helmet ID equipped
- `DAT_80084158` → `g_character_weapon_slots` - Weapon slot data
- `DAT_8008415a` → `g_character_accessory_slots` - Accessory slot data

**Item Databases:**
- `DAT_80074368` → `g_item_database` - Main item database (12-byte entries)
- `DAT_800752c2` → `g_item_flags_database` - Item usage flags (4-byte entries)
- `DAT_8007536a` → `g_item_price_database` - Item prices in gold
- `DAT_80075412` → `g_item_subtype_database` - Item subtypes/categories
- `DAT_800754cb` → `g_art_database` - Art/skill database (12-byte entries)
- `DAT_800755bb` → `g_art_flags_database` - Art usage flags

**Equipment Tables:**
- `DAT_80074f6f` → `g_equipment_slot_flags` - Equipment slot type flags
- `DAT_80074f70` → `g_equipment_character_flags` - Character equip compatibility
- `DAT_80074f6e` → `g_equipment_slot_types` - Equipment slot type mapping
- `DAT_80084141` → `g_character_equip_flags` - Per-character equip flags
- `DAT_8007538e` → `g_armor_type_database` - Armor type classification

**Inventory System:**
- `DAT_8007bcfe` → `g_inventory_start_index` - First inventory slot (0)
- `DAT_8007bd00` → `g_inventory_end_index` - Last inventory slot (255)
- `DAT_80084140` → `g_character_inventory_items` - Item IDs in inventory
- `DAT_800860e0` → `g_inventory_item_counts` - Item quantities (1-99)

**Shop System:**
- `DAT_8007bd6c` → `g_shop_data_ptr` - Pointer to current shop's item list
- `DAT_8007bcd4` → `g_player_gold` - Player's current gold amount

**Menu State Flags:**
- `DAT_8007bb16` → `g_menu_flag_1` - Menu state flag 1
- `DAT_8007bb1a` → `g_menu_flag_2` - Menu state flag 2
- `DAT_8007bb1e` → `g_menu_flag_3` - Menu state flag 3

**Battle Flags:**
- `DAT_8007bd04` → `g_battle_active_flag` - Set to 1 during battle, 0 in field
- `DAT_8007bce4` → `g_frame_timer_flags` - Frame-based timer flags

**Miscellaneous:**
- `DAT_80084000` → `g_battle_char_data_array` - Character data array base
- `DAT_800843fb` → `g_character_data_offset_1` - Character data offset helper

### 4. Helper Functions Renamed (7)

- `FUN_8004313c` → `clear_menu_buffers` - Clears all menu item buffers
- `FUN_80030104` → `create_menu_window` - Creates menu window structure
- `FUN_800302e4` → `calculate_equipment_rating` - Calculates equipment rating for sorting
- `FUN_8003043c` → `check_item_usability_field` - Checks if item is usable in field
- `FUN_8003053c` → `check_art_learnable` - Checks if art can be learned
- `FUN_80042ee0` → `get_item_inventory_slot` - Gets inventory slot for item
- `FUN_80042f4c` → `check_shop_special_items` - Checks for special shop items
- `FUN_8003ce64` → `check_location_unlocked` - Checks if travel location is unlocked
- `FUN_8003cc88` → `get_menu_result_code` - Gets menu result code

### 5. Local Variables Renamed (29)

**Stack Variables:**
- `local_40` → `created_menu_window` - Pointer to created menu window structure
- `local_3c` → `saved_char_index` - Saved character index for restoration
- `local_38` → `sort_count` - Counter for sorting operations
- `local_30` → `saved_db_ptr` - Saved database pointer for restoration

**Loop Counters:**
- `iVar21` → `counter` - General-purpose loop counter
- `iVar24` → `loop_index` - Primary loop index
- `iVar25` → `usable_count` - Count of usable items
- `iVar26` → `special_count` - Count of special/key items
- `iVar8` → `art_index` - Index into arts array

**Item Data:**
- `uVar19` → `item_id` - Item ID from database
- `uVar9` → `temp_item_value` - Temporary item value (reused for ID/cost/flags)
- `uVar3` → `temp_short` - Temporary short value for copying
- `uVar7` → `equipped_id` - Currently equipped item ID
- `uVar6` → `item_flags` - Item flag bits
- `uVar4` → `menu_entry_value` - Menu entry value with flags
- `uVar1` → `rating_value` - Equipment rating calculation result

**Character/Art Data:**
- `bVar1` → `art_id` - Art/skill ID
- `bVar3` → `check_bool` - Boolean check result
- `uVar2` → `check_byte` - Byte check result

**Location Data:**
- `cVar1` → `location_char` - Current location character code
- `cVar12` → `prev_location_char` - Previous location character code
- `pcVar15` → `location_data_ptr` - Pointer to location data

**Pointer Variables:**
- `puVar2` → `menu_buffer_read_ptr` - Pointer for reading from menu buffers
- `puVar3` → `item_db_ptr` - Pointer to item database
- `puVar4` → `menu_buffer_write_ptr_1` - Write pointer 1 for menu buffers
- `puVar5` → `menu_buffer_write_ptr_2` - Write pointer 2 for menu buffers
- `puVar6` → `menu_buffer_write_ptr_3` - Write pointer 3 for menu buffers
- `puVar7` → `inventory_item_ptr` - Pointer to inventory items
- `puVar10` → `menu_output_buffer_ptr` - Pointer to output menu buffer
- `pbVar8` → `char_equip_flags_ptr` - Pointer to character equip flags

## Menu Types (Switch Cases)

The function handles 18 different menu types via a switch statement on `menu_context + 0x1c`:

### Case 2: Consumable Item Filter
- Filters items by remaining uses (price == 0 means consumed)
- Separates consumable vs. non-consumable items
- Used for item usage tracking

### Case 3: Item Menu (Field/Battle)
- Filters by usability context (`g_battle_active_flag`)
- Checks item flags for field/battle usage (flags & 2, flags & 4)
- Special handling for time-based items (0x88, 0x89)
- Calls `check_item_usability_field()` for field items
- Marks armor items with flag 8 as special (key items)

### Cases 4, 6, 0x21: Menu Cleanup
- Frees menu window memory buffers
- Resets menu state
- Retrieves menu result code

### Cases 5, 0x20: Art/Skill Selection Menu
- Filters arts by MP cost vs. current MP
- Checks if character has weapon equipped
- Applies status effect MP modifiers (Seal -50%, Curse -75%)
- Separates learnable vs. learned arts (for field menu)
- Battle mode: only shows usable arts
- Field mode: shows all arts, marks learnable ones

### Cases 7-10, 0x15-0x18: Equipment Selection
- **Case 7/0x15**: Helmet slot
- **Case 8/0x16**: Weapon slot (reads from art data base)
- **Case 9/0x17**: Accessory slot
- **Case 10/0x18**: Weapon equipped

Behavior:
- Shows "Remove" option (0x4000)
- Shows currently equipped item (0x7000 | item_id)
- Filters inventory by equipment slot type and character compatibility
- Sorts by equipment rating (descending)

### Case 0xb: Shop Purchase Menu
- Checks if player can afford items
- Checks inventory space (max 99 per stack)
- Checks for special shop items
- Separates regular items (0x3000) from special items (0xa000)
- Marks unaffordable/full items as disabled (| 0x800)

### Cases 0xe-0x10, 0x1c-0x1e: Armor Slot Selection
- **Case 0xe/0x1c**: Armor slot 1
- **Case 0xf/0x1d**: Armor slot 2
- **Case 0x10/0x1e**: Armor slot 3

Behavior:
- Shows "Remove" option (0x4000)
- Shows currently equipped armor (0x7000 | item_id)
- Filters armors (type != 'A' in armor_type_database)
- No sorting (unlike weapon/helmet/accessory)

### Case 0x19: Travel/Warp Location Menu
- Iterates through location data (6 bytes per entry)
- Checks if locations are unlocked via `check_location_unlocked()`
- Deduplicates locations by character code
- Menu entry format: (location_index | 0x8000)

### Case 0x22: Discard Item Menu
- Separates items into 3 categories:
  1. Consumable items (0x1000 = discardable, 0x1800 = non-discardable)
  2. Equipment (0x1000 = discardable if not "important", 0x1800 = locked)
  3. Key items (0x1c00 - armor with flag 8)
- Equipment discardability checked via `g_equipment_slot_flags & 1`
- Item discardability checked via `g_item_flags_database & 1`

## Menu Entry Format

All menu entries are 16-bit values with flags:

```
[15:12] - Entry type flags
  0x1000 = Normal/usable item (white text)
  0x1800 = Disabled/unusable item (gray text)
  0x1c00 = Key item (special color)
  0x3000 = Shop regular item
  0x4000 = "Remove" option
  0x5000 = Usable art
  0x5800 = Disabled art (insufficient MP)
  0x6000 = Equipment item
  0x7000 = Currently equipped
  0x8000 = Travel location
  0x9000 = Armor item
  0xa000 = Shop special item

[11:0] - Item/art/location ID or inventory slot index
```

## Item Database Structure

### g_item_database (12-byte entries)
```
+0x00: byte - Item type (0x01 = equipment, 0x02 = consumable/armor)
+0x01: byte - Subtype index (into g_item_subtype_database)
+0x02-0x0b: (structure TBD)
```

### g_item_flags_database (4-byte entries, indexed by subtype)
```
Bit 0: Discardable flag
Bit 1: Usable in field
Bit 2: Usable in battle
Bit 3: Key item flag
```

### g_equipment_slot_flags (8-byte entries, indexed by subtype)
```
Bits 5-6 (& 0x60): Equipment slot type
  >> 5 == 0: Weapon
  >> 5 == 1: Helmet
  >> 5 == 2: Accessory
  >> 5 == 3: (unused)
Bit 0: Discardable flag
```

### g_art_database (12-byte entries)
```
+0x00: byte - Base MP cost
(MP cost modified by status effects: Seal -50%, Curse -75%)
```

### g_art_flags_database (12-byte entries)
```
Bit 1: Learnable in field
Bit 2: (unknown flag)
```

## Character Data Structure

### Character array stride: 0x414 bytes (1044 bytes)

Key offsets:
- `+0x00` (0x80083f89): Art data base
- `+0x3b` (0x80083fc4): Helmet equipped
- `+0x3c` (0x80083fc5): Weapon equipped
- `+0x3d` (0x80083fc6): Armor 1 equipped
- `+0x3e` (0x80083fc7): Armor 2 equipped
- `+0x3f` (0x80083fc8): Armor 3 equipped
- `+0x39` (0x80083fc2): Arts learned list start
- `+0x38` (0x80083fc1): Arts learned count
- `+0x1c5` (0x8008414e): Status flags
- `+0x1d7` (0x80084160): Current MP

## Memory Layout

### Menu Buffers (Global)
```
g_menu_item_buffer_main    @ 0x8007bb28 (32 shorts = 64 bytes)
g_menu_item_buffer_temp    @ 0x8007bb68 (32 shorts = 64 bytes)
g_menu_item_buffer_special @ 0x8007bba8 (32 shorts = 64 bytes)
```

### Inventory System
- Start index: 0
- End index: 255
- Item storage: `g_character_inventory_items` @ 0x80084140
- Item counts: `g_inventory_item_counts` @ 0x800860e0

### Character Data Array
- Base: `g_battle_char_data_array` @ 0x80084000
- Stride: 0x414 bytes per character
- Characters: Vahn (0), Noa (1), Gala (2), Terra (3)

## Key Discoveries

### 1. Menu Entry Encoding
Menu entries use high 4 bits for display state (color/usability) and low 12 bits for item ID/inventory slot.

### 2. Equipment Sorting
Equipment menus sort by rating (calculated by `calculate_equipment_rating`), but armor menus do not.

### 3. Status Effect MP Modifiers
Art MP costs are modified by status effects:
- Seal: -50% (cost >> 1)
- Curse: -75% (cost >> 2)

### 4. Shop Item Categories
Shops have 2 categories: regular items (first N-3 items) and special items (last 3 items), where N is shop size.

### 5. Key Item Detection
Key items are armor (type 0x02) with flag 8 set in item_flags_database.

### 6. Travel Location Data
Location data is 6 bytes per entry with character codes for deduplication.

### 7. Battle vs Field Context
Many item/art checks differ based on `g_battle_active_flag`:
- Field: More checks, learnable arts, special items
- Battle: Simpler usability checks

## Next Steps

### 1. Find Combat Logic Functions ⭐⭐⭐
The actual combat system (AI, damage formulas, turn management) is **NOT in this menu function**. Next priorities:

**Search Strategies:**
1. Look for functions that reference battle HP offsets (`g_battle_memory_buffer + 0x6d4`, `+0x6e8`)
2. Search for multiplication/division operations (damage calculations)
3. Find functions that call `rand()` (for RNG in combat)
4. Look for button input handlers during battle
5. Search functions that check `g_battle_running_flag` or `g_battle_active_flag`
6. Examine rendering functions (FUN_800480d8, FUN_80048a08) - may contain mixed logic

**Address Ranges to Explore:**
- 0x80020000-0x80040000 (unexplored functions)
- 0x80048000-0x80050000 (rendering/animation - may have combat logic mixed in)

### 2. Helper Function Analysis
The 9 renamed helper functions need DICK compliance analysis:
- `clear_menu_buffers` @ 0x8004313c
- `create_menu_window` @ 0x80030104
- `calculate_equipment_rating` @ 0x800302e4
- `check_item_usability_field` @ 0x8003043c
- `check_art_learnable` @ 0x8003053c
- `get_item_inventory_slot` @ 0x80042ee0
- `check_shop_special_items` @ 0x80042f4c
- `check_location_unlocked` @ 0x8003ce64
- `get_menu_result_code` @ 0x8003cc88

### 3. Data Structure Documentation
Document the full structures for:
- Item database entries (12 bytes)
- Art database entries (12 bytes)
- Character data structure (1044 bytes)
- Menu context structure (`menu_context` parameter)
- Shop data structure

## Modding Implications

For the Bevy rewrite, this menu system should be:

### Data-Driven
- Item properties (flags, prices, subtypes) → JSON/RON
- Art properties (MP cost, flags) → JSON/RON
- Shop inventories → JSON/RON
- Location unlock conditions → JSON/RON

### Scriptable
- Item usability checks → Lua/Rhai scripts
- Art learnability checks → Lua/Rhai scripts
- Equipment rating calculations → Lua/Rhai scripts
- Menu filtering logic → Lua/Rhai scripts

### UI System
- Menu entry encoding → Bevy UI component states
- Menu buffers → Rust Vec<MenuEntry>
- Menu context → Bevy Resource/Component

### Clean Separation
This analysis confirms the PSX code has good separation between:
- Menu UI (this function)
- Item data (database tables)
- Character state (character array)
- Combat logic (still to be found)

The Bevy rewrite should maintain this separation.

## Summary

**Achievement**: ✅ 100% DICK compliance for `process_menu_action` (77 symbols renamed)

**Function Purpose**: Menu item filtering and list building for 18 different menu types

**Key Insight**: This is UI logic, NOT combat logic. The actual battle system (AI, damage, turns) is still missing and must be found.

**Next Priority**: Resume search for combat logic functions using the strategies outlined above.
