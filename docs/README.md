# Legend of Legaia Rewrite Documentation

Welcome to the Legend of Legaia rewrite project documentation!

## Project Overview

This project aims to recreate Legend of Legaia, the classic PSX RPG, for modern platforms using:
- **Rust** programming language
- **Bevy** game engine
- Assets from a legal retail copy of the game
- Decompiled game logic from the original binary

## Documentation Structure

### [Decompilation](./decompilation/)
Notes and progress on decompiling the original SCUS-94254 executable.

### [Architecture](./architecture.md)
High-level architecture and system design.

### [Asset Formats](./asset-formats.md)
Documentation of PSX asset formats used in Legend of Legaia.

## Getting Started

### Prerequisites
- Rust toolchain (latest stable)
- A legal retail copy of Legend of Legaia (NTSC-U version)
- Ghidra (for decompilation work)

### Building the Project

```bash
# Build all crates
cargo build

# Run the game
cargo run -p legaia-game

# Extract assets from disc
cargo run -p legaia-assets --example extract -- --input /path/to/disc.bin
```

## Project Structure

```
legaia/
├── crates/
│   ├── psxutils/         # PSX format utilities (CD-XA, debug symbols)
│   ├── legaia-assets/    # Asset extraction and conversion
│   ├── legaia-engine/    # Game engine (Bevy-based)
│   └── legaia-game/      # Main game executable
├── docs/                 # Documentation
├── .opencode/            # AI agent directives
└── Cargo.toml           # Workspace configuration
```

## Contributing

See [CONTRIBUTING.md](./CONTRIBUTING.md) for guidelines (coming soon).

## Decompilation Methodology

This project follows the **DICK** methodology:
**D**ecompile **I**t **C**orrectly, **K**nucklehead

See [.opencode/AGENTS.md](../.opencode/AGENTS.md) for detailed decompilation directives.

## Legal

This project requires a legal retail copy of Legend of Legaia.
Distribution of game assets is prohibited.
This is a fan project and is not affiliated with Sony or Contrail.

## License

See [LICENSE](../LICENSE) for project license information.
