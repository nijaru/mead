## Goal
Build a memory-safe media processing toolkit focused on modern codecs and safety.

**Why**: Media processing tools handle untrusted input and complex formats, making them high-risk for security issues. Pure Rust implementation provides memory safety guarantees, while focusing on modern codecs (AV1, Opus) rather than comprehensive legacy format support allows for a cleaner, more maintainable architecture.

## Phases

| Phase | Status | Deliverables | Success Criteria |
|-------|--------|--------------|------------------|
| Phase 1 | COMPLETE | MP4 demuxer + AV1 encoder + SOTA patterns | Parse MP4, encode AV1, Arc<Frame>, send-receive |
| Phase 2a | COMPLETE | Audio codec support (Opus, AAC) | Opus decoder working, AAC placeholder |
| Phase 2b | COMPLETE | Production CLI UX | Progress bars, colors, human formatting, TTY detection |
| Phase 2c | COMPLETE | IVF muxer + encode pipeline | Write IVF files, test pattern encoding |
| Phase 2d | COMPLETE | Y4M demuxer + full transcode | Read Y4M, transcode Y4M→AV1→IVF, stdin piping |
| Phase 3 | ← NEXT | H.264/H.265 video codecs | Cover 80%+ of streaming use cases |
| Phase 4 | Planned | WebM/MKV container support | Alternative container formats |
| Phase 5 | Future | Streaming protocols (HLS, DASH) | Network streaming ingress/egress |

## Dependencies

| Must Complete | Before Starting | Why |
|---------------|-----------------|-----|
| Phase 1 (MP4 + AV1) | Phase 2a (Audio) | Need container parsing before audio muxing |
| Phase 2b (CLI UX) | Phase 3 (More codecs) | Good UX patterns needed before adding complexity |
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
| **CLI** | clap + tracing + indicatif + console | Standard Rust tools, production UX with progress bars |
| **Async** | tokio (network only) | Sync I/O for files, async for streaming |

## Phase 3 Details (Next)

**Scope:**
- H.264 decoder (OpenH264 or pure Rust alternative)
- H.265/HEVC decoder (investigate pure Rust options)
- MP4 input with H.264/H.265 video → AV1 output
- Full transcode pipeline: MP4 → decode → AV1 encode → IVF/MP4

**Success criteria:**
- Transcode MP4 (H.264) to AV1/IVF
- Memory-safe decoder implementation
- Performance comparable to existing tools
- All tests passing, zero unsafe violations

## Phase 2b Details (Complete)

**Scope:**
- Progress bars with indicatif (frame count, fps, speed, ETA)
- Colored output with console crate (errors=red, success=green, warnings=yellow)
- Human-readable formatting (HumanBytes, HumanDuration from indicatif)
- TTY detection (auto-disable progress/colors in pipes)
- Real-time metrics (fps, speed relative to realtime, bitrate)
- Scripting support (--quiet, --json, --no-color flags)
- Output separation (progress/logs → stderr, data → stdout)

**Example output:**
```
$ mead encode input.mp4 -o output.webm --codec av1
[00:02:35] ████████████████████░░░░░░░░ 1234/2000 frames 60fps 1.2x ⏱ 00:01:05
Bitrate: 2.5 Mbit/s | Size: 45.2 MiB

✓ Encoded successfully in 2m 35s
```

**Success criteria:**
- Progress bars work in TTY, hidden when piped
- Colors auto-disabled in non-TTY or with NO_COLOR
- --json flag produces machine-readable output
- Real-time fps/speed metrics during encode
- ETA accurate within 20% for long encodes

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
