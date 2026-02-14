# Legend of Legaia Architecture

## System Overview

Legend of Legaia is structured around several major systems that work together:

```
┌─────────────────────────────────────────────────────┐
│                   Main Game Loop                    │
├─────────────────────────────────────────────────────┤
│  ┌─────────────┐  ┌─────────────┐  ┌────────────┐ │
│  │   Input     │  │    State    │  │  Renderer  │ │
│  │   System    │→ │   Machine   │→ │   System   │ │
│  └─────────────┘  └─────────────┘  └────────────┘ │
│         ↓                ↓                 ↑        │
│  ┌─────────────────────────────────────────────┐  │
│  │         Active Game System                   │  │
│  │  ┌──────┐  ┌──────┐  ┌──────┐  ┌────────┐  │  │
│  │  │Field │  │Battle│  │ Menu │  │Cutscene│  │  │
│  │  └──────┘  └──────┘  └──────┘  └────────┘  │  │
│  └─────────────────────────────────────────────┘  │
│         ↓                ↓                 ↑        │
│  ┌─────────────┐  ┌─────────────┐  ┌────────────┐ │
│  │   Audio     │  │   Assets    │  │  Save/Load │ │
│  │   System    │  │   Manager   │  │   System   │ │
│  └─────────────┘  └─────────────┘  └────────────┘ │
└─────────────────────────────────────────────────────┘
```

## Major Systems

### 1. Field System
**Purpose**: Overworld and dungeon exploration

**Components**:
- Character controller (movement, jumping, climbing)
- Camera system (following player, cinematic angles)
- Collision detection (walls, floors, triggers)
- NPC management (AI, dialogue)
- Event triggers (doors, chests, story events)
- Map rendering (3D environments)

**Key Data Structures**:
- `MapData`: Level geometry and metadata
- `CharacterState`: Player/NPC position and state
- `EventTrigger`: Zone-based event activation
- `CollisionMesh`: Collision geometry

### 2. Battle System
**Purpose**: Turn-based combat with unique Art system

**Components**:
- Turn manager (initiative, turn order)
- **Art System** (combo input, command recognition)
- Damage calculation (formulas, element system)
- Enemy AI (decision trees, behavior patterns)
- Battle animations (attacks, spells, items)
- Status effects (poison, paralysis, etc.)
- Victory/defeat handling (EXP, level up, game over)

**Key Data Structures**:
- `BattleCharacter`: Stats, HP/MP, status
- `EnemyData`: Enemy stats, AI patterns
- `ArtDefinition`: Art combos and effects
- `BattleState`: Current battle context

**Art System** (Unique to Legaia):
- Input buffering (store button sequences)
- Command recognition (match sequences to Arts)
- Combo display (visual feedback)
- Art animations (unique per character)

### 3. Menu System
**Purpose**: UI for inventory, equipment, status, etc.

**Menus**:
- Main Menu (New Game, Continue, Options)
- Pause Menu (in-game access)
- Item Menu (use, organize items)
- Equipment Menu (weapons, armor, accessories)
- Magic/Arts Menu (view and organize abilities)
- Status Screens (character stats, party info)
- Save/Load Interface (memory card operations)

**Key Data Structures**:
- `MenuItem`: Menu option data
- `Inventory`: Item storage and organization
- `Equipment`: Equipped items per character
- `PartyData`: Current party composition

### 4. Graphics System
**Purpose**: Rendering 3D environments and characters

**Components**:
- Model rendering (characters, enemies, environments)
- Texture management (TIM texture loading)
- Animation system (skeletal animation, frame-based)
- Camera control (field camera, battle camera)
- Effects rendering (spells, attacks)
- UI rendering (menus, HUD)

**PSX-Specific Considerations**:
- Original used fixed-point math for coordinates
- GTE (Geometry Transform Engine) for 3D calculations
- Limited polygon count and texture memory
- Affine texture mapping (no perspective correction)
- Vertex color lighting (no per-pixel lighting)

**Modern Implementation**:
- Use modern floating-point for smoother rendering
- Preserve original art style and aesthetic
- Optional enhancements (higher resolution, anti-aliasing)

### 5. Audio System
**Purpose**: Music and sound effects playback

**Components**:
- Music streaming (background music)
- Sound effect playback (UI, battle, field)
- Voice playback (if any)
- Audio mixing (multiple channels)
- Volume control (music, SFX, master)

**PSX Formats**:
- VAB (Voice Attribute Bank): Sample bank + metadata
- VAG: Individual ADPCM-compressed samples
- XA: CD-XA streaming audio (music, voice)

**Modern Implementation**:
- Convert to OGG Vorbis or WAV
- Use Bevy's audio system for playback
- Maintain original audio quality/style

### 6. Save/Load System
**Purpose**: Game state persistence

**Save Data**:
- Player party (characters, levels, stats)
- Inventory (items, equipment)
- Progress flags (story events, quests)
- Map state (doors opened, chests looted)
- Playtime and statistics

**PSX Original**:
- Memory card saves (15 blocks per save)
- Multiple save slots
- Save points in the world

**Modern Implementation**:
- Local file system saves
- JSON or binary format
- Multiple save slots
- Auto-save support (optional)

### 7. Input System
**Purpose**: Controller and keyboard input handling

**Input Types**:
- Movement (D-Pad/Analog stick/WASD)
- Confirm/Cancel (○/✕ or A/B)
- Menu navigation
- Battle commands
- **Art input sequences** (for battle system)

**Features**:
- Input buffering (essential for Art system)
- Button remapping
- Gamepad support (DualShock style)
- Keyboard support

### 8. Event/Scripting System
**Purpose**: Story progression and game events

**Components**:
- Script interpreter (dialogue, cutscenes)
- Flag management (story progress tracking)
- Dialogue system (text display, choices)
- Cutscene playback (scripted sequences)
- Quest tracking (side quests, objectives)

**Key Data Structures**:
- `GameFlags`: Boolean flags for events
- `DialogueData`: Text and speaker info
- `CutsceneScript`: Scripted event sequence
- `QuestData`: Quest objectives and rewards

### 9. Asset Management
**Purpose**: Loading and caching game assets

**Asset Types**:
- Textures (TIM → PNG)
- Models (custom format → glTF or internal)
- Audio (VAB/VAG → OGG/WAV)
- Text (dialogue, item descriptions)
- Maps/Levels
- Animations

**Loading Strategy**:
- Load on demand (per-scene)
- Asset streaming (for large maps)
- Asset caching (frequently used)
- Memory management (unload unused)

## Game Flow

### Startup Sequence
1. Initialize engine (Bevy systems)
2. Load configuration (settings, controls)
3. Load asset manifest
4. Display splash screens
5. Main menu

### Field → Battle Transition
1. Encounter triggered (random or scripted)
2. Save field state
3. Transition animation
4. Load battle arena
5. Initialize battle state
6. Battle begins

### Battle → Field Return
1. Battle ends (victory/defeat/escape)
2. Play victory/defeat sequence
3. Restore field state
4. Transition animation
5. Resume field exploration

### Save/Load Flow
1. Player interacts with save point
2. Open save menu
3. Select slot (new/overwrite)
4. Serialize game state
5. Write to file
6. Confirm save success

## Data Flow

### Runtime Data Organization

```
┌─────────────────────────────────────────┐
│         Bevy World (ECS)                │
│                                         │
│  ┌───────────┐  ┌────────────────────┐ │
│  │ Resources │  │    Entities        │ │
│  │           │  │                    │ │
│  │ GameState │  │  Player Components │ │
│  │ AssetMgr  │  │  Enemy Components  │ │
│  │ Input     │  │  Item Components   │ │
│  │ Audio     │  │  UI Components     │ │
│  └───────────┘  └────────────────────┘ │
│                                         │
│  ┌─────────────────────────────────┐   │
│  │         Systems                 │   │
│  │  • Field Update                 │   │
│  │  • Battle Logic                 │   │
│  │  • Rendering                    │   │
│  │  • Input Handling               │   │
│  └─────────────────────────────────┘   │
└─────────────────────────────────────────┘
```

## Performance Considerations

### Original PSX Constraints
- 33.8688 MHz CPU (MIPS R3000)
- 2 MB RAM
- 1 MB VRAM
- 30 FPS target (NTSC)

### Modern Target
- 60+ FPS on modern hardware
- Higher resolution support
- Faster loading times
- Optional enhancements

### Optimization Strategies
- Asset streaming and LOD
- Efficient ECS usage in Bevy
- Multithreading where possible
- GPU instancing for repeated models
- Texture atlasing

## Extension Points

### Future Enhancements (Post-Initial Release)
- Higher resolution textures
- Widescreen support
- Additional save slots
- Fast travel options
- Difficulty options
- Debug/cheat menu
- Speedrun timer
- Achievement system
- Mod support

---

*This architecture document will evolve as decompilation progresses and we understand more about the original implementation.*
