# PSX Asset Formats

Documentation of PlayStation 1 asset formats used in Legend of Legaia.

## TIM (Texture Image)

### Overview
TIM is the standard texture format for PSX games.

### File Structure
```
Header (8 bytes):
  0x00: u32 - Magic number (0x00000010)
  0x04: u32 - Flags (bit depth, CLUT presence)

CLUT Section (if present):
  0x00: u32 - CLUT size
  0x04: u16 - X position in VRAM
  0x06: u16 - Y position in VRAM
  0x08: u16 - Width (colors)
  0x0A: u16 - Height (palettes)
  0x0C: [color data]

Image Section:
  0x00: u32 - Image data size
  0x04: u16 - X position in VRAM
  0x06: u16 - Y position in VRAM
  0x08: u16 - Width
  0x0A: u16 - Height
  0x0C: [pixel data]
```

### Bit Depths
- **4-bit**: 16 colors per palette, needs CLUT
- **8-bit**: 256 colors per palette, needs CLUT
- **16-bit**: Direct color (5-5-5-1 RGB)
- **24-bit**: True color (rare on PSX)

### Color Format (16-bit)
```
Bit layout: ABBBBBGGGGGRRRRR
A = Alpha/STP (semi-transparency)
R = Red (5 bits)
G = Green (5 bits)
B = Blue (5 bits)
```

### Conversion Notes
- Scale 5-bit RGB to 8-bit: `value * 255 / 31`
- Handle semi-transparency flag
- Multiple palettes may exist for same texture

## VAB (Voice Attribute Bank)

### Overview
VAB is a container format for audio samples and their metadata.

### File Structure
```
Header (32 bytes):
  0x00: u32 - Magic ("VABp")
  0x04: u32 - Version
  0x08: u32 - VAB ID
  0x0C: u32 - File size
  0x10: u16 - Number of programs
  0x12: u16 - Number of tones
  0x14: u16 - Number of VAGs
  0x16: u16 - Master volume
  0x18: u16 - Master pan
  0x1A: u16 - Attributes
  0x1C: u32 - Reserved

Program Table (128 * 16 bytes):
  Per program (16 bytes):
    0x00: u8 - Number of tones
    0x01: u8 - Volume
    0x02: u8 - Priority
    0x03: u8 - Mode
    0x04: u8 - Pan
    0x05: u8 - Reserved
    0x06: u8 - Attributes
    0x07: u8 - Reserved
    0x08: u32 - Reserved
    0x0C: u32 - Reserved

Tone Table (variable):
  Per tone (32 bytes):
    0x00: u8 - Priority
    0x01: u8 - Mode
    0x02: u8 - Volume
    0x03: u8 - Pan
    0x04: u8 - Center note
    0x05: u8 - Center fine
    0x06: u8 - Min note
    0x07: u8 - Max note
    0x08: u8 - Vibrato width
    0x09: u8 - Vibrato time
    0x0A: u8 - Portamento width
    0x0B: u8 - Portamento time
    0x0C: u8 - ADSR1
    0x0D: u8 - ADSR2
    0x0E: u16 - Parent program
    0x10: u16 - VAG index
    0x12: u16 - Reserved
    [14 bytes reserved/attributes]

VAG Data Section:
  Concatenated VAG samples
```

### ADSR Envelope
```
ADSR1: AAAASSSS DDDDDDDR
ADSR2: SSSSSSSS SSRRRRRM

A = Attack rate
S = Sustain level
D = Decay rate
R = Release rate
M = Sustain mode
```

## VAG (Audio Sample)

### Overview
VAG is ADPCM-compressed mono audio format.

### File Structure
```
Header (48 bytes):
  0x00: u32 - Magic ("VAGp")
  0x04: u32 - Version
  0x08: u32 - Reserved
  0x0C: u32 - Data size (bytes)
  0x10: u32 - Sample rate (Hz)
  0x14: [12 bytes reserved]
  0x20: [16 bytes name]

Audio Data:
  Blocks of 16 bytes each:
    0x00: u8 - Shift/filter parameter
    0x01: u8 - Flags (loop start/end)
    0x02: [14 bytes ADPCM data]
```

### ADPCM Decoding
- 4 bits per sample (28 samples per 16-byte block)
- Uses prediction filter for compression
- Shift parameter determines scale
- 5 filter types (0-4)

### Loop Flags
```
0x00 = Normal block
0x01 = Loop end (jump to loop start)
0x02 = Loop start
0x03 = Loop end + loop start
0x07 = End of audio
```

### Sample Rate
Common rates:
- 11025 Hz (low quality)
- 22050 Hz (medium quality)
- 44100 Hz (CD quality, rare on PSX)

## CD-XA (CD eXtended Architecture)

### Overview
CD-XA is used for streaming audio and video from CD.

### Sector Format (2352 bytes)
```
0x000: [12 bytes] Sync pattern
0x00C: [4 bytes]  Header (minute, second, frame, mode)
0x010: [8 bytes]  Sub-header (×2, duplicated)
0x018: [2048 bytes] Payload data
0x818: [4 bytes]  EDC (Error Detection Code)
0x81C: [276 bytes] ECC (Error Correction Code)
```

### Sub-header
```
0x00: u8 - File number
0x01: u8 - Channel number
0x02: u8 - Sub-mode flags
0x03: u8 - Coding info (audio attributes)
```

### Sub-mode Flags
```
Bit 0: End of Record
Bit 1: Video
Bit 2: Audio
Bit 3: Data
Bit 4: Trigger
Bit 5: Form 2
Bit 6: Real-time
Bit 7: End of File
```

### Audio Coding Info
```
Bits 0-1: Mono (0) or Stereo (1)
Bits 2-3: Sample rate (37.8kHz=0, 18.9kHz=1)
Bits 4-5: Bits per sample (4-bit=0, 8-bit=1)
Bit 6: Emphasis
```

## Model Formats (Game-Specific)

### TMD (3D Model Data)
Standard PSX 3D model format.

```
Header:
  0x00: u32 - Magic (0x41)
  0x04: u32 - Flags
  0x08: u32 - Number of objects

Object Table:
  Per object:
    Vertex data
    Normal data
    Primitive data (triangles, quads)
```

### Animation Data
- Frame-based animation
- Skeletal/bone-based animation
- Morph target animation
- Format varies by game

## Text Formats

### Font Data
- Bitmap fonts stored as textures
- Character mapping tables
- Variable-width font support

### Dialogue/Script Data
- Text stored in game binary or separate files
- May use compression
- Character encoding (Shift-JIS for Japanese, ASCII for English)
- Control codes for:
  - Speaker name
  - Text speed
  - Wait for input
  - Sound effects
  - Variable insertion

## Map/Level Data

### Geometry
- Vertex positions
- Texture coordinates
- Face definitions
- Collision mesh (separate or embedded)

### Metadata
- Spawn points
- Camera zones
- Event triggers
- NPC placement
- Item locations

### Format
Game-specific, typically:
- Header with counts and offsets
- Vertex array
- Face array
- Texture reference array
- Collision data
- Trigger/event data

## File System

### Disc Layout
PSX games typically use ISO 9660 filesystem:
- System area (boot executable)
- Application area (game data)
- Files organized in directories

### File Extensions (Common)
- `.TIM` - Textures
- `.VAB` - Audio banks
- `.VAG` - Audio samples
- `.TMD` - 3D models
- `.MOT` - Motion/animation data
- `.BIN` - Binary data (various)
- `.STR` - Streaming data (video/audio)

### Legend of Legaia Specifics
To be determined through decompilation:
- Custom archive formats
- Compression methods
- File naming conventions
- Asset organization

## Conversion Strategy

### Textures (TIM → PNG)
1. Parse TIM header and flags
2. Extract CLUT if present
3. Extract pixel data
4. Apply palette (if paletted)
5. Convert 5-bit RGB to 8-bit
6. Save as PNG with alpha channel

### Audio (VAB/VAG → OGG/WAV)
1. Parse VAB structure
2. Extract individual VAG samples
3. Decode ADPCM to PCM
4. Resample if needed
5. Convert to OGG (compressed) or WAV (lossless)
6. Preserve sample rate and metadata

### Models (TMD → glTF/Internal)
1. Parse TMD structure
2. Extract vertices, normals, UVs
3. Reconstruct faces
4. Map texture references
5. Export to glTF or internal format
6. Preserve scale and coordinate system

### Text (Binary → JSON/TOML)
1. Extract text from binary
2. Parse control codes
3. Map to modern format
4. Preserve formatting info
5. Enable localization support

---

## References

- [PSX Specifications](http://problemkaputt.de/psx-spx.htm) - Comprehensive PSX hardware documentation
- [TIM Format](http://www.raphnet.net/electronique/psx_adaptor/Playstation.txt)
- [VAB/VAG Format](http://wiki.multimedia.cx/index.php/PlayStation_Audio)
- [CD-XA Format](https://en.wikipedia.org/wiki/CD-ROM_XA)

---

*This document will be updated as we discover Legend of Legaia-specific format details through decompilation.*
