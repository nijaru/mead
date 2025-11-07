# mead

**M**emory-safe **E**ncoding **A**nd **D**ecoding

Modern AV1 encoding with Rust ergonomics. Fast by default (SVT-AV1), pure Rust option available (rav1e).

## Project Structure

- **mead/** - CLI binary (users install with `cargo install mead`)
- **mead-core/** - Library crate (developers use in their projects)
- **ai/** - AI agent working context
  - **PLAN.md** - Strategic roadmap (5 phases, dependencies)
  - **STATUS.md** - Current state (read this first!)
  - **TODO.md** - Next steps
  - **DECISIONS.md** - Key decisions with rationale
  - **RESEARCH.md** - Research findings
- **docs/** - User documentation (none yet)

### mead-core Structure
```
mead-core/src/
├── container/    # Format handlers (MP4, WebM, MKV)
│   └── mp4.rs    # MP4 demuxer/muxer
├── codec/        # Codec implementations
│   └── av1.rs    # AV1 encoder/decoder
├── error.rs      # Error types
└── lib.rs        # Public API
```

## Technology Stack

- **Language**: Rust (edition 2024, rust-version 1.85)
- **Framework**: None (library + CLI)
- **Package Manager**: cargo
- **CLI**: clap 4.5 with derive macros
- **Async**: tokio 1.0 (network I/O only, sync for files)
- **Logging**: tracing + tracing-subscriber
- **Error Handling**: anyhow (app) + thiserror (lib)

### Media Libraries (Safety-Critical)
- **mp4parse** 0.17 - Mozilla's pure Rust MP4 parser
- **rav1e** 0.7 - Xiph's pure Rust AV1 encoder
- **rav1d** (planned) - Safe Rust port of dav1d AV1 decoder

## Development Commands

```bash
# Build
cargo build
cargo build --release

# Test
cargo test
cargo test --workspace

# Run CLI
cargo run -p mead -- info video.mp4
cargo run -p mead -- decode input.mp4 -o output.pcm

# Encode (Y4M → AV1)
cargo run -p mead -- encode input.y4m -o output.ivf --codec av1

# Professional workflow (ffmpeg piping)
ffmpeg -i input.mp4 -f yuv4mpegpipe - | cargo run -p mead -- encode - -o output.ivf --codec av1

# Check (fast, no codegen)
cargo check

# Lint
cargo clippy --workspace

# Format
cargo fmt --all

# Publish (library maintainers only)
cargo publish -p mead-core
cargo publish -p mead
```

## Code Standards

### Safety
- **`#![forbid(unsafe_code)]`** in `mead-core/src/lib.rs`
- Pure Rust preferred over C bindings
- Any `unsafe` blocks require:
  - Justification comment explaining why necessary
  - Safety invariants documented
  - Extensive fuzzing coverage
  - Review in PR

### Error Handling
- Library (`mead-core`): Use `thiserror` for error types
- Application (`mead`): Use `anyhow` for error propagation
- Never `unwrap()` or `expect()` in library code (tests OK)
- Return `Result<T, Error>` for all fallible operations

### Logging
- Use `tracing` macros: `trace!`, `debug!`, `info!`, `warn!`, `error!`
- Log at appropriate levels:
  - `trace!` - Detailed internal state
  - `debug!` - Developer debugging info
  - `info!` - User-relevant events
  - `warn!` - Recoverable issues
  - `error!` - Operation failures

### Naming
- Boolean: `is_keyframe`, `has_audio`, `can_decode`
- Constants: `MAX_PACKET_SIZE`, `DEFAULT_BITRATE`
- With units: `duration_ms`, `bitrate_kbps`
- Collections: plural (`packets`, `frames`)

### Comments
- **WHY, not WHAT** - code should be self-documenting
- Add comments for:
  - Non-obvious decisions
  - External requirements (format specs, patents)
  - Algorithm rationale
  - Workarounds with links to issues
- NO change tracking (git does this)
- NO TODOs (use ai/TODO.md or issues)

## Current Focus

**Version**: 0.0.0 (staying on 0.0.x until production-ready)

**IMPORTANT**: We are staying on 0.0.x versions for a long time. Not ready for 0.1.0 until core functionality is solid and well-tested. Version bumps happen only when explicitly instructed.

**Phase**: Phase 2e (AV1 Optimization + Strategic Pivot) - **COMPLETE** ✅

Latest work (2025-11-06):
- Phase 1: Complete (MP4 streaming, AV1 encoder, SOTA patterns)
- Phase 2a: Complete (Opus decoder, AAC placeholder, audio demuxing)
- Phase 2b: Complete (Production CLI UX with progress bars, colors, human formatting)
- Phase 2c: Complete (IVF muxer, encode pipeline)
- Phase 2d: Complete (Y4M demuxer, full transcode, stdin piping)
- Phase 2e: **Complete** (Tile parallelism, SVT-AV1 strategy)
- 37 tests passing (31 core + 4 output + 2 doc), zero warnings

**Phase 2e Achievements:**
✅ Tile parallelism optimization (4-5× speedup: 720p 8.81→37.96 fps, 1080p 4.00→18.50 fps)
✅ Benchmark framework for performance testing
✅ SVT-AV1 comparison (3-5× gap, down from 7× baseline)
✅ Strategic decision: SVT-AV1 default, rav1e option
✅ Research docs (encoder comparison, CLI UX, AV1 settings)

**Previous Achievements:**
✅ Y4M input with full color space support (420p/422p/444p)
✅ Full transcode pipeline at real-time speeds
✅ Production CLI UX matching modern Rust tools
✅ IVF output playable in VLC/ffmpeg/dav1d

See **ai/STATUS.md** for current state and blockers.
See **ai/PLAN.md** for full roadmap and technical architecture.
See **ai/research/cli_ux_best_practices.md** for CLI UX research.

## Project Goals

1. **Competitive Performance**: Fast by default with SVT-AV1 (100+ fps), pure Rust option available
2. **Modern UX**: Better CLI than ffmpeg (presets, progress bars, sane defaults)
3. **Clean Architecture**: Modular design with safe Rust core, C bindings where beneficial
4. **Production Ready**: Comprehensive error handling, logging, and testing from day one
5. **Incremental Safety**: Servo/Firefox model - start hybrid, move to more Rust over time

## Safety Approach

**Hybrid Strategy** (like Servo → Firefox):
- **mead-core**: `#![forbid(unsafe_code)]` - pure Rust API remains safe
- **mead CLI**: C bindings where necessary for performance/ecosystem
- **Default encoder**: SVT-AV1 (production-proven, 0 CVEs in 4 years)
- **Pure Rust option**: rav1e available via `--encoder rav1e`
- **Future path**: Incremental adoption of safe Rust alternatives as they mature

Libraries:
- Pure Rust: `rav1e` (encoder), `mp4` (demuxer), `symphonia` (audio), `y4m` (YUV)
- Safe C bindings: SVT-AV1 (default encoder, battle-tested)
- Future: `rav1d` decoder (safe Rust dav1d port), pure Rust H.264/H.265 as they mature

Architecture:
- Streaming I/O to prevent resource exhaustion
- Zero-copy frame handling with Arc<Frame>
- Safe Rust APIs in mead-core (library consumers get safety guarantees)

## Links

- **GitHub**: https://github.com/nijaru/mead
- **crates.io**: https://crates.io/crates/mead
- **Library**: https://crates.io/crates/mead-core
- **License**: Apache-2.0 (patent protection critical for media codecs)
- **Reference**: https://github.com/nijaru/agent-contexts
