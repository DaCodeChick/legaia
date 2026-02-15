# Asset Extraction Workflow

This document describes how to extract and convert assets from the Legend of Legaia PSX disc for use in the modern Bevy rewrite.

## Overview

We extract game assets (textures, models, audio, etc.) from the original PSX disc image and convert them to modern formats that Bevy can load natively.

## Source Files

**PSX Disc Image:**
- Format: `.bin/.cue` files
- Source: Legend of Legaia (USA) - `SCUS_942.54`
- Contains: Game data, textures, 3D models, audio, videos, scripts

## Asset Types & Conversion

### 1. Textures (TIM → PNG)

**Source Format:** TIM (PlayStation Image Format)
- 4-bit, 8-bit, 15-bit, or 24-bit color
- Optional CLUT (Color Lookup Table) for indexed color
- Multiple sub-images per file

**Conversion:**
```rust
use psxutils::formats::Tim;

// Parse TIM file
let tim_data = std::fs::read("texture.tim")?;
let tim = Tim::parse(&tim_data)?;

// Convert to RGBA8888
let rgba_data = tim.to_rgba8888();
let width = tim.width();
let height = tim.height();

// Save as PNG
image::save_buffer(
    "texture.png",
    &rgba_data,
    width,
    height,
    image::ColorType::Rgba8
)?;
```

**Target Format:** PNG (lossless, RGBA8888)
**Bevy Loading:** `asset_server.load::<Image>("textures/texture.png")`

### 2. 3D Models (TMD → glTF)

**Source Format:** TMD (PlayStation Model Data)
- Vertex positions, normals, UVs
- Face indices (triangles/quads)
- Material references

**Conversion:** (TODO: Implement TMD parser)
```rust
// Future API:
use psxutils::formats::Tmd;

let tmd_data = std::fs::read("model.tmd")?;
let tmd = Tmd::parse(&tmd_data)?;

// Export as glTF 2.0
let gltf = tmd.to_gltf();
std::fs::write("model.gltf", gltf.to_json()?)?;
```

**Target Format:** glTF 2.0 (JSON + binary buffers)
**Bevy Loading:** `asset_server.load::<Scene>("models/model.gltf#Scene0")`

### 3. Audio Samples (VAG → WAV)

**Source Format:** VAG (PlayStation ADPCM Audio)
- Sony ADPCM compression (4-bit samples)
- 44.1kHz mono audio
- Loop points for music/ambient sounds

**Conversion:**
```rust
use psxutils::formats::Vag;

// Parse VAG file
let vag_data = std::fs::read("sound.vag")?;
let vag = Vag::parse(&vag_data)?;

// Decode ADPCM to PCM16
let pcm_samples = vag.decode_to_pcm16();
let sample_rate = vag.sample_rate;

// Save as WAV
let wav_spec = hound::WavSpec {
    channels: 1,
    sample_rate,
    bits_per_sample: 16,
    sample_format: hound::SampleFormat::Int,
};

let mut wav_writer = hound::WavWriter::create("sound.wav", wav_spec)?;
for sample in pcm_samples {
    wav_writer.write_sample(sample)?;
}
wav_writer.finalize()?;
```

**Target Format:** WAV (PCM16, 44.1kHz) or OGG Vorbis (compressed)
**Bevy Loading:** `asset_server.load::<AudioSource>("audio/sound.ogg")`

### 4. Sound Banks (VAB → Individual WAV files)

**Source Format:** VAB (PlayStation Audio Bank)
- Collection of VAG samples
- Program definitions (instrument mappings)
- MIDI-like sequencing data

**Conversion:**
```rust
use psxutils::formats::Vab;

// Parse VAB file
let vab_data = std::fs::read("sounds.vab")?;
let vab = Vab::parse(&vab_data)?;

// Extract individual samples
for (i, sample) in vab.samples.iter().enumerate() {
    let pcm = sample.decode_to_pcm16();
    // Save as WAV...
    save_wav(&format!("sound_{:03}.wav", i), &pcm)?;
}
```

**Target Format:** Individual WAV/OGG files per sample
**Bevy Loading:** Load each sound effect individually

### 5. XA Streams (CD-XA → OGG Vorbis)

**Source Format:** CD-XA (CD-ROM Extended Architecture)
- Compressed audio streams for music/dialogue
- Interleaved with video data
- ADPCM compression

**Conversion:** (TODO: Implement XA parser)
```bash
# Use external tools like vgmstream or ffmpeg
vgmstream -o music.wav music.xa
ffmpeg -i music.wav -c:a libvorbis -q:a 6 music.ogg
```

**Target Format:** OGG Vorbis (compressed, loopable)
**Bevy Loading:** Stream using `bevy_kira_audio`

### 6. STR Videos (STR → MP4/WebM)

**Source Format:** STR (PlayStation Movie Format)
- MDEC video compression
- Interleaved audio (XA ADPCM)
- 15fps typical frame rate

**Conversion:** (TODO: Document external tools)
```bash
# Use jpsxdec or similar PSX video tools
jpsxdec -x video.str -dir ./output/
ffmpeg -i output/video_%04d.png -i output/audio.wav \
       -c:v libx264 -c:a aac video.mp4
```

**Target Format:** MP4 (H.264 video, AAC audio)
**Bevy Loading:** Use video playback plugin (external)

## Extraction Tools

### Command-Line Extractor

Build the extraction tool:
```bash
cargo build --release -p legaia-assets
```

Extract all assets from disc:
```bash
legaia-assets extract \
    --disc "path/to/SCUS_942.54.bin" \
    --output "./assets/"
```

Extract specific asset type:
```bash
legaia-assets extract --disc SCUS_942.54.bin --type textures
legaia-assets extract --disc SCUS_942.54.bin --type audio
legaia-assets extract --disc SCUS_942.54.bin --type models
```

### Programmatic API

```rust
use legaia_assets::AssetExtractor;

let extractor = AssetExtractor::new("SCUS_942.54.bin")?;

// Extract all textures
extractor.extract_textures("./assets/textures")?;

// Extract all audio
extractor.extract_audio("./assets/audio")?;

// Extract specific file by path
let data = extractor.read_file("BTLDAT/ENEMY.BIN")?;
```

## Asset Organization

Organize extracted assets in the `assets/` directory:

```
assets/
├── textures/
│   ├── characters/
│   │   ├── vahn_body.png
│   │   ├── vahn_face_001.png
│   │   └── ...
│   ├── environments/
│   │   ├── town_rimelmit.png
│   │   └── ...
│   └── ui/
│       ├── menu_bg.png
│       └── ...
├── models/
│   ├── characters/
│   │   ├── vahn.gltf
│   │   └── ...
│   ├── enemies/
│   │   ├── slime.gltf
│   │   └── ...
│   └── environments/
│       └── ...
├── audio/
│   ├── bgm/
│   │   ├── title_theme.ogg
│   │   ├── battle_theme_01.ogg
│   │   └── ...
│   ├── sfx/
│   │   ├── menu_cursor.wav
│   │   ├── attack_hit.wav
│   │   └── ...
│   └── voice/
│       └── ...
└── videos/
    ├── intro.mp4
    └── ...
```

## Metadata Files

Create `.meta.toml` files alongside assets to document properties:

```toml
# assets/textures/characters/vahn_body.meta.toml
[texture]
original_file = "CHR/VAHN.TIM"
original_clut = 0
palette_mode = "8bit"
transparency = true

[usage]
character_id = "vahn"
body_part = "torso"
```

```toml
# assets/audio/bgm/title_theme.meta.toml
[audio]
original_file = "SOUND/TITLE.XA"
loop_start = 0.0
loop_end = 120.5
bpm = 128

[usage]
scene = "title_screen"
priority = "high"
```

## Implementation Status

- ✅ TIM parser (textures)
- ✅ VAG parser (audio samples)
- ✅ VAB parser (sound banks)
- ⏳ TMD parser (3D models) - In Progress
- ⏳ XA parser (music streams) - TODO
- ⏳ STR parser (videos) - TODO
- ⏳ Asset extractor CLI - TODO
- ⏳ Batch conversion scripts - TODO

## Next Steps

1. Complete TMD parser for 3D model extraction
2. Build CLI tool in `legaia-assets` crate
3. Document disc file structure (which files contain what)
4. Create batch extraction scripts
5. Verify all extracted assets load in Bevy
6. Build asset catalog with metadata
