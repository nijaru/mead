## Goal
Build a memory-safe media processing toolkit in Rust that prevents the memory safety vulnerabilities (buffer overflows, use-after-free) that plague FFmpeg. Focus on modern, widely-used codecs rather than comprehensive format support.

**Why**: Google's security research revealed 20+ FFmpeg vulnerabilities. FFmpeg's 1.5M LOC C codebase and 400+ formats make comprehensive fixes difficult. Rust's ownership system prevents these vulnerability classes at compile time.

## Phases

| Phase | Status | Deliverables | Success Criteria |
|-------|--------|--------------|------------------|
| Phase 1 | ← CURRENT | MP4 demuxer/muxer + AV1 encode/decode | Parse MP4, encode/decode AV1 frames, CLI commands work |
| Phase 2 | Planned | Audio codec support (AAC, Opus) | Handle audio streams, sync with video |
| Phase 3 | Planned | H.264, H.265, VP9 codecs | Cover 80%+ of streaming use cases |
| Phase 4 | Planned | WebM/MKV container support | Alternative container formats |
| Phase 5 | Future | Streaming protocols (HLS, DASH, RTMP) | Network streaming ingress/egress |

## Dependencies

| Must Complete | Before Starting | Why |
|---------------|-----------------|-----|
| Phase 1 (MP4 + AV1) | Phase 2 (Audio) | Need container parsing before audio muxing |
| Phase 2 (Audio) | Phase 3 (H.264/H.265) | Audio sync complexity - learn on AV1 first |
| Phase 1-3 (Codecs/containers) | Phase 5 (Streaming) | Need solid encode/decode before network protocols |
| Fuzzing setup | Any container work | Container parsers are high-risk attack surface |

## Technical Architecture

| Component | Approach | Rationale |
|-----------|----------|-----------|
| **Container layer** | mp4parse-rust (Mozilla) | Battle-tested in Firefox, pure Rust |
| **AV1 codec** | rav1e (encode), rav1d (decode) | Production-ready, pure Rust, Xiph-backed |
| **Audio codecs** | symphonia (AAC), libopus bindings | Pure Rust decoder, safe Opus wrapper |
| **H.264** | OpenH264 safe bindings | Cisco-provided, patent-free baseline |
| **Safety** | `#![forbid(unsafe_code)]` in core | Compile-time guarantee, exceptions require justification |
| **Fuzzing** | cargo-fuzz from day 1 | Parsers are attack surface - continuous fuzzing |
| **CLI** | clap + tracing | Standard Rust tools, good UX |
| **Async** | tokio (network only) | Sync I/O for files, async for streaming |

## Phase 1 Details (Current)

**Scope:**
- MP4 container: demux (read), mux (write), metadata extraction
- AV1 codec: encode frames, decode frames
- CLI: `mead info`, `mead encode`, `mead decode` commands
- Error handling: comprehensive Result types, clear error messages
- Fuzzing: mp4parse corpus, basic CI integration

**Not in Phase 1:**
- Audio (Phase 2)
- Other codecs (Phase 3+)
- Streaming protocols (Phase 5)
- Hardware acceleration (future optimization)

**Success = Published v0.1.0:**
```bash
mead info video.mp4          # Show metadata
mead encode in.mp4 -o out.mp4 --codec av1  # Transcode to AV1
mead decode video.mp4 -o frames/  # Extract frames
```

## Out of Scope

**Never:**
- Unsafe FFmpeg bindings (defeats safety purpose)
- Legacy/obscure codecs (the "1990s decoders" causing FFmpeg issues)
- Comprehensive format support (focus on modern, common formats)

**Deferred:**
- Hardware acceleration (VAAPI, NVDEC) - optimization phase
- Real-time processing - correctness first, performance later
- GUI - CLI/library only
- Codec development - use existing safe implementations

## Risk Mitigation

| Risk | Mitigation |
|------|-----------|
| Performance slower than FFmpeg | Accept tradeoff - safety over speed. Profile and optimize critical paths. |
| Limited codec support | Focus on 80% use case (H.264, AV1, AAC). Niche formats out of scope. |
| H.264 requires C bindings | Use safe wrappers (openh264-sys), justify unsafe blocks, fuzz extensively. |
| Adoption challenges | Clear safety story, good docs, professional presentation. Target security-conscious users. |
