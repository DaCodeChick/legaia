# Legend of Legaia Rewrite

A modern rewrite of the classic PSX RPG **Legend of Legaia** using Rust and Bevy.

> ⚠️ **This project is in early development.** Requires a legal retail copy of Legend of Legaia.

## 🎮 About

This project recreates Legend of Legaia for modern platforms by:
- Extracting assets from a retail disc image
- Decompiling the original game logic
- Reimplementing systems in Rust with Bevy engine
- Preserving the original game feel while adding modern improvements

# Legend of Legaia Rewrite

A modern rewrite of the classic PSX RPG **Legend of Legaia** using Rust and Bevy.

> ⚠️ **This project is in early development.** Requires a legal retail copy of Legend of Legaia.

## 🎮 About

This project recreates Legend of Legaia for modern platforms with a **clean-room approach**:
- Extract and convert assets from retail disc image to modern formats
- Use decompilation to understand **game logic** (battle formulas, AI, events)
- Build native Bevy ECS systems (NOT PSX hardware emulation)
- Preserve the original game feel while adding modern improvements

**Philosophy**: We're building a modern game, not a PSX emulator. Decompilation informs implementation but doesn't dictate architecture.

## 📋 Project Status

**Infrastructure:**
- [x] Project structure and build system
- [x] Asset extraction CLI (`legaia-extract`)
- [x] PSX format parsers (TIM ✅, TMD ✅, VAG ✅, VAB ✅, CD-ROM ✅)
- [x] Modern Bevy engine scaffold
- [x] Code separation policy (decompilation ≠ Rust code)

**Decompilation Progress:**
- [x] 21/1,121 functions analyzed (1.9%)
- [x] DICK methodology established
- [x] 130+ globals renamed and categorized
- [ ] Focus shift: Game logic only (skip hardware functions)

**Game Systems:**
- [ ] Battle system (HIGH PRIORITY)
- [ ] Field/world system
- [ ] Menu system
- [ ] Event/scripting system
- [ ] Save/load system

See [.opencode/AGENTS.md](.opencode/AGENTS.md) for detailed decompilation progress and [docs/asset-extraction.md](docs/asset-extraction.md) for asset workflow.

## 🏗️ Project Structure

```
legaia/
├── crates/
│   ├── psxutils/         # PSX format utilities (CD-XA, debug symbols)
│   ├── legaia-assets/    # Asset extraction and conversion
│   ├── legaia-engine/    # Game engine (Bevy-based)
│   └── legaia-game/      # Main game executable
├── docs/                 # Documentation
│   ├── architecture.md   # System architecture
│   ├── asset-formats.md  # PSX format documentation
│   └── decompilation/    # Decompilation progress
├── .opencode/
│   └── AGENTS.md         # DICK methodology & decompilation directives
└── Cargo.toml           # Workspace configuration
```

## 🚀 Getting Started

### Prerequisites

- **Rust** (latest stable): [Install Rust](https://rustup.rs/)
- **Legal retail copy** of Legend of Legaia (NTSC-U: SCUS-94254)
- **Ghidra** (optional, for decompilation work): [Download Ghidra](https://ghidra-sre.org/)

### Building

```bash
# Clone the repository
git clone https://github.com/YOUR_USERNAME/legaia.git
cd legaia

# Build all crates
cargo build --release

# Run the game (currently shows a placeholder)
cargo run --release -p legaia-game
```

### Extracting Assets

The `legaia-extract` CLI tool handles asset extraction and conversion:

```bash
# Build the extraction tool
cargo build --release -p legaia-assets

# List files on disc
./target/release/legaia-extract list --disc /path/to/Legend_of_Legaia.bin

# Extract all assets and auto-convert (TIM→PNG, etc.)
./target/release/legaia-extract extract-all \
  --disc /path/to/Legend_of_Legaia.bin \
  --output ./assets \
  --type all

# Extract specific file
./target/release/legaia-extract extract \
  --disc /path/to/Legend_of_Legaia.bin \
  --file SCUS_942.54 \
  --output game.exe

# Convert TIM texture to PNG
./target/release/legaia-extract convert-tim input.TIM output.png

# Convert VAG audio to WAV
./target/release/legaia-extract convert-vag input.VAG output.wav

# Convert TMD model to glTF
./target/release/legaia-extract convert-tmd model.TMD model.gltf

# Show TMD model info
./target/release/legaia-extract info-tmd model.TMD
```

**Supported Formats:**
- ✅ **TIM** (textures) → PNG conversion
- ✅ **TMD** (3D models) → glTF 2.0 export
- ✅ **VAG** (audio samples) → WAV conversion
- ✅ **VAB** (sound banks) → Parser ready
- ✅ **CD-ROM ISO 9660** → File extraction

See [docs/asset-extraction.md](docs/asset-extraction.md) for detailed workflow.

## 📚 Documentation

- [Project Strategy & Decompilation Guide](.opencode/AGENTS.md) - DICK methodology, priorities
- [Asset Extraction Workflow](docs/asset-extraction.md) - Converting PSX assets to modern formats
- Architecture docs (coming soon)

## 🛠️ Development

### Development Philosophy

**Modern Bevy-Native Rewrite:**
- ✅ Use decompilation for: Battle formulas, AI logic, event scripts, save format
- ❌ Skip decompilation for: GPU/SPU hardware, rendering, DMA, BIOS calls
- Build clean Bevy ECS systems, not PSX hardware emulation
- Extract assets → convert to native formats → load with Bevy

### Decompilation Workflow

This project follows the **DICK** methodology:
**D**ecompile **I**t **C**orrectly, **K**nucklehead

**Key principles**:
- Analyze complete call chains
- Rename EVERY function, parameter, variable, and global
- Leave nothing unnamed (no `FUN_*`, `param_*`, `local_*`, `DAT_*`)
- Document purpose and behavior (not just mechanics)
- **Focus on game logic**, skip hardware abstraction

**Priority Systems for Decompilation:**
1. ⭐⭐⭐ Battle formulas and AI
2. ⭐⭐ Event/script system
3. ⭐ Character stats, items, save data
4. Skip: GPU, SPU, CD-ROM, memory card functions

See [.opencode/AGENTS.md](.opencode/AGENTS.md) for detailed guidelines.

### Running Tests

```bash
# Run all tests
cargo test --workspace

# Run tests for a specific crate
cargo test -p psxutils
```

### Code Style

```bash
# Format code
cargo fmt --all

# Run clippy lints
cargo clippy --workspace -- -D warnings
```

## 🎯 Roadmap

### Phase 1: Infrastructure ✅ (Current)
- [x] Project structure and workspace
- [x] Asset extraction CLI tool (`legaia-extract`)
- [x] PSX format parsers (TIM, TMD, VAG, VAB, CD-ROM)
- [x] Modern Bevy engine scaffold
- [x] Documentation and development guidelines
- [x] Decompilation methodology (DICK)

### Phase 2: Asset Pipeline
- [ ] Complete TMD → glTF exporter
- [ ] VAG → WAV/OGG converter with ADPCM decode
- [ ] Batch asset extraction from disc
- [ ] Asset metadata and organization
- [ ] Test assets loading in Bevy

### Phase 3: Game Logic (Battle System Priority)
- [ ] Decompile battle damage formulas
- [ ] Decompile Art/combo system mechanics
- [ ] Decompile enemy AI logic
- [ ] Implement battle system in Bevy
- [ ] Character stats and progression

### Phase 4: World Systems
- [ ] Field movement and collision
- [ ] Event script interpreter
- [ ] Menu system
- [ ] Save/load system
- [ ] Map connectivity

### Phase 5: Content Integration
- [ ] Import all extracted assets
- [ ] Implement all battle mechanics
- [ ] Complete story event scripts
- [ ] Full game playthrough possible

### Phase 6: Polish
- [ ] Bug fixes and testing
- [ ] Performance optimization
- [ ] Modern enhancements (optional: widescreen, HD, QoL)
- [ ] Release builds

## 🤝 Contributing

Contributions are welcome! This is a large project that can benefit from help in several areas:

- **Decompilation**: Analyzing functions in Ghidra
- **Asset Parsing**: Implementing TIM/VAB/VAG parsers
- **Engine Development**: Implementing game systems in Rust/Bevy
- **Testing**: Comparing behavior with original game
- **Documentation**: Improving docs and guides

See [CONTRIBUTING.md](CONTRIBUTING.md) for guidelines (coming soon).

## ⚖️ Legal

This is a fan project for educational purposes. You must own a legal retail copy of Legend of Legaia to use this software.

**Important**:
- This project does NOT include any game assets
- You must extract assets from your own retail copy
- Distribution of game assets is prohibited
- This project is not affiliated with Sony, Contrail, or any rights holders

## 📜 License

See [LICENSE](LICENSE) for details.

## 🙏 Acknowledgments

- **Contrail** - Original developers of Legend of Legaia
- **Sony Computer Entertainment** - Original publishers
- **PSX Community** - Documentation and tools
- **Bevy Community** - Amazing game engine
- **Ghidra Team** - Reverse engineering tools

## 📞 Contact

- **Issues**: Use GitHub Issues for bug reports and feature requests
- **Discussions**: Use GitHub Discussions for questions and ideas

---

*Legend of Legaia © 1998 Contrail, © 1998-1999 Sony Computer Entertainment*
*This fan project is not affiliated with or endorsed by the rights holders.*
