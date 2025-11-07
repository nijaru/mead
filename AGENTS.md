# mead

**M**emory-safe **E**ncoding **A**nd **D**ecoding

A modern, safe media processing toolkit written in Rust, designed to prevent the memory safety vulnerabilities that plague traditional media libraries like FFmpeg.

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
cargo run -p mead -- encode input.mp4 -o output.mp4 --codec av1

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

**Phase**: Phase 2b (Production CLI UX) - **NEXT**

Latest work:
- Phase 1: Complete (MP4 streaming, AV1 encoder, SOTA patterns)
- Phase 2a: Started (Opus decoder, AAC placeholder, audio demuxing)
- Phase 2b: Planning (Production CLI UX - progress bars, colors, human formatting)
- 21 tests passing, zero warnings

**Why Phase 2b now?**
FFmpeg's strength is real-time progress feedback during encodes. If mead is to replace FFmpeg, we need:
- Progress bars showing frame count, fps, speed, ETA (indicatif)
- Colored output (console crate)
- Human-readable formatting (3.5 MiB not 3670016)
- TTY detection (auto-hide progress when piped)
- Scripting flags (--quiet, --json, --no-color)

Current CLI uses plain println! with no progress indicators. This makes mead feel like a toy vs FFmpeg. Better to build production UX patterns now before adding more complexity.

See **ai/STATUS.md** for current state and blockers.
See **ai/PLAN.md** for full roadmap and technical architecture.
See **ai/research/cli_ux_best_practices.md** for CLI UX research.

## Project Goals

1. **Memory Safety First**: Eliminate buffer overflows, use-after-free, and other memory safety bugs that plague FFmpeg
2. **Modern Codecs**: Focus on widely-used formats (AV1, H.264, AAC, Opus) not legacy codec sprawl
3. **Clean Architecture**: Modular design with clear separation between containers, codecs, and pipeline
4. **Production Ready**: Comprehensive error handling, logging, and fuzzing from day one

## Safety Approach

- Pure Rust libraries: `rav1e`, `rav1d`, `mp4parse-rust`, `symphonia`
- Safe bindings for mature C libraries where necessary (OpenH264)
- NO unsafe FFmpeg bindings
- Fuzzing integrated in CI from day 1
- `#![forbid(unsafe_code)]` enforced in safe modules

## Links

- **GitHub**: https://github.com/nijaru/mead
- **crates.io**: https://crates.io/crates/mead
- **Library**: https://crates.io/crates/mead-core
- **License**: Apache-2.0 (patent protection critical for media codecs)
- **Reference**: https://github.com/nijaru/agent-contexts
