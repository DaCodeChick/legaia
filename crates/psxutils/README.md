# psxutils

PlayStation 1 utilities for parsing game assets and formats.

## Features

### Format Parsers

- **TIM** - PlayStation texture format
- **TMD** - 3D model format
- **VAB** - Audio bank format
- **VAG** - Audio format
- **XA** - CD-ROM XA audio streams
- **LZSS** - Lempel-Ziv-Storer-Szymanski compression (NEW!)

### Compression

#### LZSS Decompression

LZSS is a dictionary-based compression algorithm commonly used in PS1 games. Legend of Legaia uses it for various game assets (identified by `.lzs` extension).

**Library Usage:**

```rust
use psxutils::formats::lzss;

// One-shot decompression
let compressed = std::fs::read("player.lzs")?;
let decompressed = lzss::decompress(&compressed)?;

// Streaming decompression
use std::fs::File;
let mut decoder = lzss::LzssDecoder::standard();
let mut input = File::open("data.lzs")?;
let mut output = File::create("data.bin")?;
decoder.decompress(&mut input, &mut output)?;

// Custom configuration
let config = lzss::LzssConfig {
    window_size: 4096,
    max_match_len: 18,
    min_match_len: 3,
    offset_bits: 12,
    length_bits: 4,
};
let mut decoder = lzss::LzssDecoder::new(config);
```

**Command-Line Tool:**

```bash
# Decompress a single file
cargo run --example lzss_decompress player.lzs

# Specify output path
cargo run --example lzss_decompress player.lzs player.bin
```

## Examples

All examples can be run from the `crates/psxutils` directory:

```bash
# LZSS decompression
cargo run --example lzss_decompress <input.lzs> [output]
```

## Testing

```bash
cargo test
```

## License

See LICENSE file in repository root.
