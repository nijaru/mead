# mead - Session Context

## What We Built (2025-11-05)

Built **mead** - a memory-safe media processing toolkit in Rust as an alternative to FFmpeg. Motivated by Google's discovery of 20+ FFmpeg vulnerabilities and FFmpeg's difficulty patching due to 1.5M LOC C codebase.

### Completed

**✅ Project Setup**
- Workspace: `mead` (CLI) + `mead-core` (library)
- Edition 2024, rust-version 1.85
- Apache-2.0 license (patent protection for codecs)
- `#![forbid(unsafe_code)]` in mead-core

**✅ Published to crates.io**
- `mead` v0.0.0: https://crates.io/crates/mead
- `mead-core` v0.0.0: https://crates.io/crates/mead-core
- GitHub: https://github.com/nijaru/mead

**✅ AI Agent Configuration**
- `AGENTS.md` (primary, tool-agnostic)
- `CLAUDE.md` → `AGENTS.md` (symlink for Claude Code)
- `ai/PLAN.md` - 5-phase roadmap with dependencies
- `ai/STATUS.md` - current state (read first!)
- `ai/TODO.md` - prioritized tasks
- `ai/DECISIONS.md` - key decisions with rationale
- `ai/RESEARCH.md` - FFmpeg vulnerabilities, Rust ecosystem analysis

**✅ Architecture**
```
mead/
├── mead/              # CLI binary
│   └── src/main.rs    # clap commands: info, encode, decode
├── mead-core/         # Library
│   └── src/
│       ├── container/ # MP4, WebM, MKV (placeholders)
│       ├── codec/     # AV1, H.264, AAC (placeholders)
│       └── error.rs   # Error types
└── ai/                # Agent context
```

### Technology Decisions

| Decision | Choice | Rationale |
|----------|--------|-----------|
| **Name** | mead (MEdia And Decoding) | Short, available, professional |
| **License** | Apache-2.0 only | Patent protection (codecs are minefield) |
| **Safety** | Pure Rust + safe bindings | Prevent FFmpeg's memory bugs |
| **Version** | 0.0.0 | Name reservation, no functionality yet |
| **Dependencies** | mp4parse, rav1e, symphonia | Battle-tested safe libraries |

### Safe Dependencies Selected

- **mp4parse** 0.17 - Mozilla's pure Rust MP4 parser (used in Firefox)
- **rav1e** 0.7 - Xiph's pure Rust AV1 encoder
- **rav1d** (planned) - Safe Rust port of dav1d (5% slower but memory-safe)
- **symphonia** (planned) - Pure Rust audio codecs
- **clap** 4.5 - CLI argument parsing
- **tokio** 1.0 - Async runtime (network I/O only)

## Current State

**Version**: 0.0.0 (name reservation only)
**Phase**: Phase 1 (MP4 + AV1) - Ready to implement
**Code**: Skeleton with placeholders, no functionality
**Blockers**: None

## Next Steps (Priority Order)

See `ai/TODO.md` for full list. High priority:

1. **Implement MP4 demuxer** using `mp4parse-rust`
   - Read MP4 files
   - Extract metadata (duration, tracks, codecs)
   - Parse packets from tracks

2. **Implement AV1 encoder** using `rav1e`
   - Encode video frames to AV1
   - Configure bitrate, quality settings
   - Generate keyframes

3. **Add fuzzing** with `cargo-fuzz`
   - MP4 parser fuzzing (high-risk attack surface)
   - Integrate into CI

4. **CLI commands**
   - `mead info video.mp4` - display metadata
   - `mead encode input.mp4 -o output.mp4` - transcode
   - `mead decode video.mp4 -o frames/` - extract frames

5. **Publish v0.0.1** when basic functionality works

## Roadmap (5 Phases)

See `ai/PLAN.md` for details:

- **Phase 1** (current): MP4 container + AV1 codec
- **Phase 2**: Audio support (AAC, Opus)
- **Phase 3**: H.264, H.265, VP9 codecs
- **Phase 4**: WebM/MKV containers
- **Phase 5**: Streaming protocols (HLS, DASH, RTMP)

## Key Constraints

**Must:**
- Keep `#![forbid(unsafe_code)]` in safe modules
- Use pure Rust or safe bindings only
- Fuzz all parsers continuously
- Comprehensive error handling (no unwrap/expect in lib)
- Clear commit messages (no AI attribution)

**Avoid:**
- Unsafe FFmpeg bindings (defeats safety purpose)
- Legacy/obscure codecs (focus on modern formats)
- Per-token API billing for LLM usage

## Quick Commands

```bash
# Build
cargo build

# Test
cargo test --workspace

# Run CLI
cargo run -p mead -- --help
cargo run -p mead -- info video.mp4

# Check without building
cargo check

# Lint
cargo clippy --workspace

# Format
cargo fmt --all
```

## References

- **Agent context**: Read `ai/STATUS.md` first, then `ai/PLAN.md`
- **Code standards**: See `AGENTS.md` → Code Standards section
- **Research**: `ai/RESEARCH.md` for FFmpeg vulnerability analysis
- **Decisions**: `ai/DECISIONS.md` for rationale on key choices

## Git State

- **Branch**: main
- **Commits**: 4 commits
  - 912fd6e - Initial project structure
  - fc50fe8 - Version 0.0.0 for reservation
  - 111b8dc - Add mead-core version dependency
  - 60cb1da - AI agent configuration
- **Remote**: git@github.com:nijaru/mead.git
- **Published**: crates.io (both crates)

---

**Session ready to continue** - Start with implementing MP4 demuxer or AV1 encoder.
