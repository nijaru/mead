# mead

Modern AV1 encoding with Rust ergonomics.

Fast, user-friendly media processing with production-ready performance and modern CLI design.

## Features

- **Fast by default** - Production-grade SVT-AV1 encoder (100+ fps)
- **Pure Rust option** - Memory-safe rav1e encoder available
- **Better UX** - Progress bars, presets, sane defaults
- **Modern CLI** - Works like ripgrep/fd/bat, not ffmpeg
- **Y4M input** for raw video processing
- **IVF output** for AV1 streams
- **MP4 demuxing** with streaming support
- **Audio decoding** (Opus, AAC)
- **Stdin/stdout piping** for integration with existing tools

## Installation

```bash
cargo install mead
```

Or build from source:

```bash
git clone https://github.com/nijaru/mead
cd mead
cargo build --release
```

## Usage

### Encode video to AV1

```bash
# Fast encoding with SVT-AV1 (default, 100+ fps)
mead encode input.y4m -o output.ivf

# Pure Rust with rav1e (20-40 fps, memory-safe)
mead encode input.y4m -o output.ivf --encoder rav1e

# Pipe from ffmpeg
ffmpeg -i input.mp4 -f yuv4mpegpipe - | mead encode - -o output.ivf
```

### Get file information

```bash
mead info video.mp4
```

### Extract audio

```bash
mead decode audio.mp4 -o output.pcm
```

## Supported Formats

| Format | Read | Write |
|--------|------|-------|
| MP4    | ✅   | ⏳    |
| IVF    | ⏳   | ✅    |
| Y4M    | ✅   | ⏳    |
| WebM   | ⏳   | ⏳    |

| Codec      | Decode | Encode | Notes |
|------------|--------|--------|-------|
| AV1        | ⏳     | ✅     | SVT-AV1 (default), rav1e (pure Rust) |
| Opus       | ✅     | ⏳     | |
| AAC        | 🚧     | ⏳     | |
| H.264      | ⏳     | ⏳     | |

✅ Implemented | 🚧 Partial | ⏳ Planned

## Project Status

Early development. Core video encoding pipeline is functional. Suitable for experimentation and testing, not yet recommended for production use.

**Current capabilities:**
- AV1 encoding at 100+ fps (SVT-AV1) or 20-40 fps (rav1e)
- Y4M input with full color space support (420p/422p/444p)
- IVF output for AV1 streams
- Extract Opus audio from MP4
- Stream processing with constant memory usage
- Progress bars and modern CLI UX
- Professional workflow integration via stdin/stdout

**Roadmap:**
- Phase 3: H.264/H.265 video codecs
- Phase 4: WebM/MKV container support
- Phase 5: Streaming protocols (HLS, DASH)

## Architecture

```
mead/              # CLI binary
mead-core/         # Library crate
  ├── container/   # MP4, IVF, Y4M format handlers
  ├── codec/       # AV1, Opus, AAC codecs
  ├── frame.rs     # Zero-copy frame handling with SIMD alignment
  └── io.rs        # Streaming I/O abstractions
```

## Design Principles

- **Safety**: Memory-safe APIs, streaming I/O to prevent resource exhaustion
- **Performance**: Zero-copy frame sharing, SIMD-aligned buffers, efficient encoding
- **Composability**: Library-first design, CLI built on public APIs
- **Modern codecs**: Focus on AV1, Opus, and contemporary formats

## Contributing

Contributions welcome! The project prioritizes correctness, safety, and code quality.

## License

Licensed under the Apache License, Version 2.0. See [LICENSE](LICENSE) for details.

Patent protection is especially important for media codecs.
