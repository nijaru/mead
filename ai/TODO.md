## High Priority - Phase 3 or complete Phase 2a
- [ ] Wire up CLI encode command (transcode to AV1)
- [ ] Add AV1 decoder using rav1d
- [ ] Complete AAC decoder (ADTS parsing)
- [ ] Add cargo-fuzz integration for container parsing
- [ ] Set up CI/CD with GitHub Actions

## Completed (2025-11-06)

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
- [ ] Audio codec support (AAC, Opus) - Phase 2
- [ ] H.264, H.265, VP9 codec support - Phase 3
- [ ] WebM/MKV container support - Phase 4
- [ ] Streaming protocols (HLS, DASH, RTMP) - Phase 5
- [ ] Documentation and examples
- [ ] Performance benchmarks vs FFmpeg
