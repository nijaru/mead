# mead - Session Context

**Current state**: Phase 1c (Large File Tests) - COMPLETE

**Last session**: Completed Phase 1c by adding comprehensive large file tests. Verified streaming MP4 implementation handles large files efficiently with constant memory usage. All 18 tests passing.

## Quick Navigation

| File | Purpose |
|------|---------|
| **CLAUDE.md** | Project overview, tech stack, code standards |
| **ai/STATUS.md** | Current state (READ THIS FIRST) |
| **ai/TODO.md** | Next tasks |
| **ai/DECISIONS.md** | Key decisions with rationale |
| **ai/RESEARCH.md** | Research findings |
| **ai/PLAN.md** | Strategic roadmap (5 phases) |

## What Changed (Latest Session - 2025-11-05)

**Phase 1b - Streaming Fix** (Commit: a2a9adf)

Before (DoS vulnerability):
```rust
// mp4parse loads entire file into memory
let mut buffer = Vec::new();
reader.read_to_end(&mut buffer)?;  // DoS risk!
```

After (streaming):
```rust
// mp4 crate uses BufReader for constant memory
let buf_reader = BufReader::new(source);
let reader = mp4::Mp4Reader::read_header(buf_reader, size)?;
let sample = reader.read_sample(track_id, sample_id)?;
// Memory: O(buffer_size) not O(file_size)
```

**Impact**:
- ✅ Fixed DoS vulnerability with large MP4 files
- ✅ Implemented actual packet reading (read_packet now works)
- ✅ Added track selection API for multi-track files
- ✅ All 16 tests passing

**Phase 1a - SOTA Refactoring** (Commit: 77086cd)

- ✅ MediaSource trait for runtime seekability detection
- ✅ Arc<Frame> with SIMD-aligned planes (aligned-vec)
- ✅ Send-receive encoder pattern (matches rav1e/hardware APIs)
- ✅ PixelFormat type safety (Yuv420p, Yuv422p, Yuv444p, Rgb24)
- ✅ 10 new tests for frame alignment and I/O

**Phase 1c - Large File Tests** (Commit: 26480be)

- ✅ Added streaming memory usage tests (verify O(buffer_size) not O(file_size))
- ✅ Added buffered I/O verification tests
- ✅ Created large file simulation (1MB+ test data)
- ✅ 18 tests passing (up from 16)

## Project Status

| Metric | Status |
|--------|--------|
| Version | 0.0.0 (staying on 0.0.x for long time, not ready for 0.1.0) |
| Published | crates.io: mead, mead-core (v0.0.0 placeholder) |
| Phase | Phase 1c complete, ready for Phase 2 |
| Tests | 18 passing (frame, io, codec, container, large files) |
| Architecture | Streaming MP4, MediaSource, Arc<Frame>, send-receive encoder |
| Clippy | Zero warnings |

## What Works

✅ **MP4 demuxer**: Streaming with BufReader, constant memory usage
✅ **Metadata extraction**: CLI `info` command works
✅ **Packet reading**: read_packet() returns actual sample data
✅ **AV1 encoder**: Send-receive API pattern
✅ **Frame handling**: SIMD-aligned, Arc for zero-copy
✅ **I/O abstraction**: MediaSource trait for files/streams

## What Doesn't Work

❌ **CLI encode command**: Not wired up yet (encoder exists but CLI doesn't use it)
❌ **AV1 decoder**: Planned using rav1d
❌ **Audio codecs**: Phase 2 (AAC, Opus)
❌ **Other containers**: WebM/MKV - Phase 4
❌ **AV1 decoder**: Planned for future phase

## Next Phase: Phase 2 (Audio Support)

**Phase 1 Complete**: MP4 + AV1 video support with streaming and large file handling

**Phase 2 Options**:
- Add AAC audio codec support
- Add Opus audio codec support
- Extend container muxing for audio tracks
- Add audio demuxing from MP4

## Architecture

**Core Abstractions**:

```rust
// MediaSource - Runtime seekability detection
pub trait MediaSource: Read + Seek {
    fn is_seekable(&self) -> bool;
    fn len(&self) -> Option<u64>;
}

// Arc<Frame> - Zero-copy with SIMD alignment
pub type ArcFrame = Arc<Frame>;
pub struct Frame {
    planes: Vec<Plane>,  // Y, U, V with 32-byte alignment
    format: PixelFormat,
}

// Send-receive encoder - Matches hardware APIs
pub trait VideoEncoder {
    fn send_frame(&mut self, frame: Option<ArcFrame>) -> Result<()>;
    fn receive_packet(&mut self) -> Result<Option<Vec<u8>>>;
    fn finish(&mut self) -> Result<Vec<Vec<u8>>>;
}
```

**Streaming MP4**:
- Uses `mp4` crate (527K downloads) not `mp4parse` (25K downloads)
- BufReader for constant memory: O(buffer_size) not O(file_size)
- Read samples on-demand via read_sample() API

## Key Files

```
mead/
├── mead/                       # CLI binary
│   ├── src/main.rs             # Commands: info ✅, encode ❌, decode ❌
│   └── Cargo.toml              # Depends on mead-core + mp4
├── mead-core/                  # Library
│   ├── src/
│   │   ├── container/mp4.rs    # Mp4Demuxer with mp4 crate streaming
│   │   ├── codec/av1.rs        # Av1Encoder with send-receive pattern
│   │   ├── frame.rs            # Arc<Frame> with SIMD-aligned planes
│   │   ├── io.rs               # MediaSource trait
│   │   └── error.rs            # Error types (thiserror)
│   ├── tests/
│   │   └── mp4_spike.rs        # mp4 crate API exploration
│   └── Cargo.toml              # Pure Rust deps: mp4, rav1e, aligned-vec
└── ai/                         # AI agent context
    ├── STATUS.md               # Current state ← READ THIS FIRST
    ├── TODO.md                 # Next tasks
    ├── PLAN.md                 # 5-phase roadmap
    ├── DECISIONS.md            # Architecture decisions
    ├── RESEARCH.md             # FFmpeg CVEs, Rust ecosystem
    └── research/
        └── rust_media_api_design.md  # SOTA patterns (1260 lines)
```

## Technology Stack

**Language**: Rust edition 2024, rust-version 1.85
**License**: Apache-2.0 (patent protection for codecs)
**Safety**: `#![forbid(unsafe_code)]` in mead-core

**Dependencies** (all pure Rust):
- `mp4` 0.14 - Streaming MP4 parser/writer (replaced mp4parse)
- `rav1e` 0.7 - Xiph AV1 encoder
- `aligned-vec` 0.6 - SIMD-aligned allocations
- `clap` 4.5 - CLI with derive macros
- `thiserror` 2.0 - Error types (library)
- `anyhow` 1.0 - Error handling (application)
- `tracing` 0.1 - Structured logging

## Commands

```bash
# Build and test
cargo build
cargo test --workspace
cargo clippy --workspace

# Run CLI
cargo run -p mead -- info video.mp4

# Check (fast, no codegen)
cargo check

# Run spike test (mp4 crate exploration)
cargo test mp4_spike -- --ignored --nocapture
```

## Code Quality

✅ `#![forbid(unsafe_code)]` enforced
✅ Zero clippy warnings
✅ 16 tests passing
✅ All dependencies pure Rust
✅ Comprehensive error handling (no unwrap/expect in lib)
✅ Structured logging with tracing

## Git State

**Branch**: main (ahead of origin by 4 commits)

**Recent commits**:
- a2a9adf - fix: replace mp4parse with mp4 crate (streaming)
- 77086cd - refactor: Phase 1a SOTA patterns
- 6b899e6 - docs: add 0.0.x version policy
- d5eff2b - feat: initial MP4 + AV1 implementation

**Remote**: git@github.com:nijaru/mead.git
**Published**: crates.io v0.0.0 (both crates - name reservation only)

## Constraints

**Must**:
- Keep `#![forbid(unsafe_code)]` in safe modules
- Use pure Rust or safe bindings only
- Ask before publishing (version 0.0.x for long time)
- No AI attribution in commits (strip if found)

**Avoid**:
- Unsafe FFmpeg bindings (defeats safety purpose)
- Loading entire files into memory (DoS risk)
- Legacy/obscure codecs (focus on modern formats)

## Roadmap

See `ai/PLAN.md` for full details:

- **Phase 1** (current): MP4 + AV1
  - Phase 1a ✅: SOTA refactoring (MediaSource, Arc<Frame>, send-receive)
  - Phase 1b ✅: Streaming fix (mp4 crate)
  - Phase 1c: Wire up encode command OR add decoder
- **Phase 2**: Audio support (AAC, Opus)
- **Phase 3**: H.264, H.265, VP9 codecs
- **Phase 4**: WebM/MKV containers
- **Phase 5**: Streaming protocols (HLS, DASH, RTMP)

---

**Phase 1 Complete** - Next: Phase 2 (Audio Support) or other priorities
