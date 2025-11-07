## High Priority - Phase 2f (SVT-AV1 Integration) - IN PROGRESS
- [x] Strategic decision: SVT-AV1 default, rav1e option (Servo model)
- [x] Update project positioning (README, CLAUDE.md, DECISIONS.md)
- [x] Create svt-av1-sys crate with FFI bindings
- [x] Set up bindgen for SVT-AV1 3.1.2
- [x] Create encoder module structure (trait, backend enum)
- [ ] Implement safe SvtAv1Encoder wrapper
  - [ ] Context initialization/cleanup
  - [ ] Frame send/receive
  - [ ] Config mapping (preset, resolution, etc.)
  - [ ] Error handling
- [ ] Add --encoder CLI flag (svt-av1 | rav1e)
- [ ] Integrate both encoders in encode command
- [ ] Update tests (encode with both backends)
- [ ] Update documentation (usage examples, installation)

## Next Up - Phase 2g (Preset System)
- [ ] Add preset enum (Fast, Balanced, Quality)
- [ ] Map presets to encoder settings (both rav1e and SVT-AV1)
- [ ] CLI --preset flag
- [ ] Documentation and examples

## Future - Phase 3 (H.264/H.265 decoders)
- [ ] Research H.264 decoder options (OpenH264 vs pure Rust)
- [ ] Implement H.264 decoder integration
- [ ] Add MP4 video demuxing (currently only metadata)
- [ ] Full transcode: MP4 (H.264) → AV1 → IVF
- [ ] Complete AAC decoder (ADTS parsing)
- [ ] Add cargo-fuzz integration for container parsing
- [ ] Set up CI/CD with GitHub Actions

## Completed (2025-11-06)

### Phase 2e - AV1 Optimization + Strategic Pivot
- [x] Implement tile parallelism in Av1Config (tile_cols, tile_rows, threads)
- [x] Smart tile calculation (respects 256×256 minimum, powers-of-2)
- [x] Auto-detect CPU cores with num_cpus
- [x] Create benchmark framework (mead/benches/encode_benchmark.rs)
- [x] Run performance benchmarks (720p, 1080p with various configs)
- [x] Create SVT-AV1 comparison script (scripts/compare_encoders.sh)
- [x] Benchmark rav1e vs SVT-AV1 (3-5× gap confirmed)
- [x] Research encoder comparison and document findings
- [x] Strategic decision: SVT-AV1 default, rav1e option
- [x] Update all documentation with new positioning
- [x] All 37 tests passing

### Phase 2d - Y4M Input Support
- [x] Add y4m crate dependency
- [x] Implement Y4mDemuxer wrapper (YUV420p, YUV422p, YUV444p)
- [x] Wire up Y4M module in container/mod.rs
- [x] Update encode command to accept Y4M input
- [x] Add stdin support for piped workflows
- [x] Test full transcode pipeline (Y4M → AV1 → IVF)
- [x] Update documentation (README, CLAUDE.md, ai/)
- [x] All 36 tests passing (30 core + 4 output + 2 doctests)

### Phase 2c - IVF Muxer + Encode Pipeline
- [x] Implement IVF muxer (32-byte header + 12-byte frame headers)
- [x] Add IVF muxer tests (6 comprehensive tests)
- [x] Wire up encode command with test pattern generation
- [x] Test full encode pipeline (generate → encode → mux)
- [x] Verify IVF output playable in VLC/ffmpeg

### Phase 2b - Production CLI UX
- [x] Add indicatif dependency for progress bars
- [x] Add console dependency for colors and TTY detection
- [x] Implement progress bar during decode
- [x] Add colored output (success=green, error=red, warning=yellow)
- [x] Add human-readable formatting (HumanBytes, HumanDuration)
- [x] Implement TTY detection (auto-disable progress/colors in pipes)
- [x] Add NO_COLOR environment variable support
- [x] Add --quiet flag (errors only)
- [x] Add --json flag (machine-readable output)
- [x] Add --no-color flag (explicit color disable)
- [x] Separate stdout/stderr (data → stdout, logs → stderr)
- [x] Create output module with Theme and formatting utilities
- [x] All tests passing (25 total), zero clippy warnings

## Completed (2025-11-05)

### Phase 1b - Streaming Fix
- [x] Replace mp4parse with mp4 crate (fixes DoS vulnerability)
- [x] Implement BufReader streaming (constant memory usage)
- [x] Add MP4 packet reading (read_sample API)
- [x] Update CLI to use mp4 crate API

### Phase 1a - SOTA Refactoring
- [x] Add MediaSource trait and implementations
- [x] Refactor Frame to use Arc and SIMD-aligned planes
- [x] Change encoder API to send-receive pattern
- [x] Add 10 new tests (frame, io)

### Initial Implementation
- [x] Implement MP4 demuxer metadata extraction using mp4parse-rust
- [x] Implement AV1 encoder using rav1e
- [x] Wire up CLI info command
- [x] Add basic tests (6 passing)
- [x] Project structure and name reservation (v0.0.0 published)

## Backlog
- [ ] Complete AAC decoder (ADTS parsing) - Phase 2a
- [ ] H.264, H.265 video decoders - Phase 3
- [ ] WebM/MKV container support - Phase 4
- [ ] Streaming protocols (HLS, DASH, RTMP) - Phase 5
- [ ] Documentation and examples
- [ ] Performance benchmarks vs FFmpeg
