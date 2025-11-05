## FFmpeg Vulnerabilities vs Rust Safety (2025-11-05)
**Sources**:
- Google Big Sleep AI findings (2025)
- FFmpeg CVE database
- Risky Business #813 podcast
- Prossimo memory safety initiative (rav1d)

**Key Finding**:
- Google's Big Sleep discovered 20+ vulnerabilities in FFmpeg/ImageMagick
- FFmpeg maintainers frustrated with volume/nature of reports ("obscure 1990s decoders")
- Core vulnerability types: buffer overflows, use-after-free, double-free
- Rust prevents these at compile time via ownership system

**Decision**: Build memory-safe alternative focusing on modern codecs only

---

## Rust Media Ecosystem Analysis (2025-11-05)
**Sources**:
- lib.rs multimedia categories
- crates.io package search
- Mozilla mp4parse-rust
- Xiph rav1e project

**Key Finding**:
- No pure Rust FFmpeg alternative exists
- Most tools are FFmpeg wrappers (rsmpeg, ez-ffmpeg, video-rs)
- Battle-tested safe components available:
  - mp4parse-rust (Mozilla, used in Firefox)
  - rav1e (Xiph, production-ready AV1 encoder)
  - rav1d (safe dav1d port, 5% slower but memory-safe)
  - symphonia (pure Rust audio)

**Decision**: Use existing safe libraries, build glue + missing pieces

---

## Codec Usage Statistics (2025-11-05)
**Sources**:
- Statista 2018 data
- Gumlet State of Video Codecs 2024
- Industry recommendations 2025

**Key Finding**:
- H.264: 82% market share (streaming/broadcasting)
- MP4 container: Most widely used
- AAC audio: 38% of publishers, superior quality at low bitrates
- Modern: VP9, AV1 gaining adoption (web/streaming)
- Professional: ProRes, DNxHR (editing workflows)

**Decision**: Phase 1 = MP4 + AV1 (modern, safe libs exist). H.264 in Phase 3 (requires safe bindings).

---

<!-- Template:

## Topic (YYYY-MM-DD)
**Sources**: [links, books, docs]
**Key Finding**: [main takeaway]
**Decision**: [action]
→ Details: ai/research/topic.md

## State-of-the-Art Rust Media API Design (2025-11-05)
**Sources**:
- symphonia (pure Rust audio library)
- mp4parse-rust (Mozilla MP4 parser)
- rav1e (Xiph AV1 encoder)
- v_frame (video frame data structures)
- rust-av ecosystem

**Key Finding**: Modern Rust media libraries converge on these patterns:
→ Details: ai/research/rust_media_api_design.md

**Decisions for mead**:
1. Use trait-based extensibility (FormatReader, Decoder, Encoder)
2. Type-state pattern for decoder/encoder state machines
3. Multiple buffer abstractions (zero-copy refs, owned, byte-oriented)
4. MediaSource trait for I/O abstraction (seekable + non-seekable streams)
5. Arc<Frame> for zero-copy frame sharing
6. Iterator-based packet/frame reading
7. Aligned memory for SIMD operations

**Analysis**: Current implementation has 3 critical issues vs SOTA patterns
→ Action plan: ai/REFACTORING_PLAN.md

---

## Open Questions
- [ ] Question needing research
-->
