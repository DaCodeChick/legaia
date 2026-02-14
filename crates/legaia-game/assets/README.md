# Legend of Legaia - Extracted Assets

This directory contains assets extracted from the retail Legend of Legaia disc.

## Structure

```
assets/
├── textures/       # TIM textures converted to PNG
├── audio/          # VAB/VAG audio converted to OGG/WAV
├── models/         # 3D models
├── animations/     # Animation data
├── maps/           # Map/level data
├── text/           # Dialogue and text
└── manifest.json   # Asset manifest
```

## Extraction

### Manual Extraction (Current Approach)

Assets must be manually extracted from your retail disc using third-party tools:

**Recommended Tools:**
- **jPSXdec** - Extract TIM textures, STR videos, audio
  - Download: https://github.com/m35/jpsxdec
  - Usage: Load your disc image and browse/extract files

- **PSound** - Extract VAB/VAG audio banks
  - Can convert PSX audio to WAV format

- **PSX-Mode2** - Extract files from CD-XA format discs
  - Useful for raw file extraction

- **TIMViewer** - View and convert TIM textures
  - Can batch convert TIM → PNG

**Extraction Workflow:**
1. Create a disc image from your retail disc (BIN/CUE format)
2. Use jPSXdec to browse and extract known file types
3. Use PSX-Mode2 for raw file extraction if needed
4. Convert assets to modern formats:
   - TIM → PNG (for textures)
   - VAB/VAG → OGG/WAV (for audio)
5. Organize extracted files into the structure above
6. Document asset locations for future automation

### Future: Automated Extraction

Once asset locations are identified, we can build an automated extraction tool using `legaia-assets` crate. This will allow:

```bash
cargo run --example extract -- --input /path/to/disc.bin --output ./assets
```

## Legal Notice

You must own a legal retail copy of Legend of Legaia to use these tools.
Distribution of extracted game assets is prohibited.

