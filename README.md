# Legend of Legaia Rewrite

A modern rewrite of the classic PSX RPG **Legend of Legaia** using Rust and Bevy.

> ⚠️ **This project is in early development.** Requires a legal retail copy of Legend of Legaia.

## 🎮 About

This project recreates Legend of Legaia for modern platforms by:
- Extracting assets from a retail disc image
- Decompiling the original game logic
- Reimplementing systems in Rust with Bevy engine
- Preserving the original game feel while adding modern improvements

## 📋 Project Status

- [x] Project structure established
- [x] Asset extraction framework
- [x] Engine scaffolding (Bevy)
- [ ] Asset format parsers (TIM, VAB, VAG)
- [ ] Decompilation in progress (0/1121 functions)
- [ ] Battle system
- [ ] Field system
- [ ] Menu system
- [ ] Full playthrough possible

See [docs/decompilation/README.md](docs/decompilation/README.md) for detailed progress.

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

```bash
# Extract assets from your retail disc
cargo run --release -p legaia-assets --example extract -- \
  --input /path/to/Legend_of_Legaia.bin \
  --output ./crates/legaia-game/assets

# This will create:
# - Extracted textures (TIM → PNG)
# - Converted audio (VAB/VAG → OGG/WAV)
# - Models and other game data
# - Asset manifest (manifest.json)
```

> **Note**: Asset extraction is not yet fully implemented. This is a work in progress.

## 📚 Documentation

- [Architecture Overview](docs/architecture.md) - System design and game flow
- [Asset Formats](docs/asset-formats.md) - PSX format specifications
- [Decompilation Guide](.opencode/AGENTS.md) - DICK methodology for decompilation
- [Decompilation Progress](docs/decompilation/README.md) - Function analysis tracking

## 🛠️ Development

### Decompilation Workflow

This project follows the **DICK** methodology:
**D**ecompile **I**t **C**orrectly, **K**nucklehead

**Key principles**:
- Analyze complete call chains
- Rename EVERY function, parameter, variable, and global
- Leave nothing unnamed (no `FUN_*`, `param_*`, `local_*`, `DAT_*`)
- Document as you go

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

### Phase 1: Foundation (Current)
- [x] Project structure
- [x] Documentation
- [ ] Asset format parsers (TIM, VAB, VAG)
- [ ] Asset extraction tool
- [ ] Basic rendering test

### Phase 2: Core Systems
- [ ] Decompile main loop and initialization
- [ ] Field system (movement, camera)
- [ ] Graphics rendering (models, textures)
- [ ] Input handling
- [ ] Audio playback

### Phase 3: Battle System
- [ ] Battle initialization
- [ ] Turn-based combat
- [ ] Art system (combo input)
- [ ] Damage calculation
- [ ] Enemy AI
- [ ] Battle animations

### Phase 4: Content
- [ ] Menu system
- [ ] Event/scripting system
- [ ] Save/load system
- [ ] Complete asset integration
- [ ] Full game playthrough

### Phase 5: Polish
- [ ] Bug fixes
- [ ] Performance optimization
- [ ] Modern enhancements (optional)
- [ ] Testing and validation

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
