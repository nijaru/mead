## High Priority
- [ ] Wire up CLI encode command (transcode to AV1)
- [ ] Add cargo-fuzz integration for container parsing
- [ ] Implement full MP4 packet reading (sample table parsing)
- [ ] Add AV1 decoder using rav1d
- [ ] Set up CI/CD with GitHub Actions

## Completed (2025-11-05)
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
