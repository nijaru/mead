# mead

**M**emory-safe **E**ncoding **A**nd **D**ecoding

A media processing toolkit focused on safety and modern codecs.

## Features

- **AV1 encoding** with rav1e
- **Y4M input** for raw video processing
- **IVF output** for AV1 streams
- **MP4 demuxing** with streaming support
- **Audio decoding** (Opus, AAC)
- **Stdin/stdout piping** for integration with existing tools
- **Memory safety** through pure Rust implementation

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
# Encode Y4M file to AV1/IVF
mead encode input.y4m -o output.ivf --codec av1

# Pipe from ffmpeg
ffmpeg -i input.mp4 -f yuv4mpegpipe - | mead encode - -o output.ivf --codec av1
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

| Codec  | Decode | Encode |
|--------|--------|--------|
| AV1    | ⏳     | ✅     |
| Opus   | ✅     | ⏳     |
| AAC    | 🚧     | ⏳     |
| H.264  | ⏳     | ⏳     |

✅ Implemented | 🚧 Partial | ⏳ Planned

## Project Status

Early development. Core video encoding pipeline is functional. Suitable for experimentation and testing, not yet recommended for production use.

**Current capabilities:**
- Transcode Y4M to AV1/IVF at 25-48 fps (640x480)
- Extract Opus audio from MP4
- Stream processing with constant memory usage
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
