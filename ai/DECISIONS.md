## 2025-11-05: Project Name - "mead"

**Context**: Needed a short, memorable name for Rust media processing CLI. Considered: ox, oxi, oxy, vyx, vex, reel, vidx, ruvid.

**Decision**: "mead" (short for "media")

**Rationale**:
- 4 characters, easy to type
- Clear "media" connection
- Professional sound
- Completely available on crates.io and GitHub
- Broader scope than video-only names

**Tradeoffs**:
| Pro | Con |
|-----|-----|
| Short, memorable | Less obvious what it does vs "vidx" |
| Media = audio + video | Competes with "medea" (v0.2.0, media server) |
| Available everywhere | - |

**Commits**: 912fd6e

---

## 2025-11-05: Single License - Apache-2.0

**Context**: Standard Rust practice is dual MIT/Apache-2.0, but user preferred single license for simplicity.

**Decision**: Apache-2.0 only (not dual licensed)

**Rationale**:
- Patent protection critical for media codecs (H.264, H.265 are patent minefields)
- Competing with FFmpeg (legal/patent history)
- Corporate-friendly (explicit patent grants)
- Simpler than dual licensing

**Tradeoffs**:
| Pro | Con |
|-----|-----|
| Strong patent protection | More verbose than MIT |
| Corporate contributions easier | Deviates from Rust ecosystem norm |
| Clear legal terms | - |

**Commits**: fc50fe8

---

## 2025-11-05: Version 0.0.0 for Name Reservation

**Context**: Needed to claim "mead" and "mead-core" on crates.io without functional code.

**Decision**: Publish 0.0.0 (not 0.0.1 or 0.1.0)

**Rationale**:
- 0.0.0 is valid (Cargo's default)
- Clearest "placeholder, nothing here" signal
- Seen in wild: `ox = "0.0.0"`, `reel = "0.0.0"`
- 0.0.1 would suggest some minimal functionality exists

**Commits**: 111b8dc

---

## 2025-11-05: Safe Dependencies Over FFmpeg Wrappers

**Context**: Most Rust media tools wrap unsafe FFmpeg. We're building memory-safe alternative.

**Decision**: Use pure Rust or safe bindings only

**Dependencies chosen**:
- mp4parse-rust (Mozilla) - Pure Rust MP4 parser
- rav1e (Xiph) - Pure Rust AV1 encoder
- rav1d (planned) - Safe Rust port of dav1d AV1 decoder
- symphonia (planned) - Pure Rust audio codecs

**Rationale**:
- Project goal is memory safety
- FFmpeg's CVE history shows memory bugs in demuxers/decoders
- `#![forbid(unsafe_code)]` in mead-core enforces this
- Tradeoff: Smaller codec/format coverage vs safety

**Tradeoffs**:
| Pro | Con |
|-----|-----|
| Memory safety guarantees | Limited format support initially |
| No buffer overflow/UAF | Slower development |
| Fuzzing more effective | May need safe C bindings (OpenH264) |
| Clear safety story | Performance may lag FFmpeg |

---

## 2025-11-05: mp4 Crate Over mp4parse for Streaming

**Context**: mp4parse (Mozilla) loads entire file into memory via read_to_end(), creating DoS vulnerability with large files. Needed streaming support with constant memory usage.

**Decision**: Replace mp4parse with mp4 crate for Mp4Demuxer

**Rationale**:
- mp4parse: Loads full file into Vec<u8> (DoS risk with multi-GB files)
- mp4 crate: Uses BufReader, streams with Mp4Reader::read_header() + read_sample()
- mp4 crate: 527K downloads vs mp4parse 25K (more widely used)
- mp4 crate: Better API for sample reading (no manual sample table parsing)
- Memory usage: O(buffer_size) not O(file_size)

**Tradeoffs**:
| Pro | Con |
|-----|-----|
| Constant memory usage | Different API (requires rewrite) |
| No DoS vulnerability | Less Mozilla pedigree |
| Actual sample reading API | Larger dependency graph |
| 20x more downloads | - |

**Evidence**:
- ai/research/rust_media_api_design.md
- mead-core/tests/mp4_spike.rs (API exploration)

**Commits**: a2a9adf

---

<!-- Template:

## YYYY-MM-DD: Decision Title

**Context**: [situation]
**Decision**: [choice]
**Rationale**:
- [reason 1]
- [reason 2]

**Tradeoffs**:
| Pro | Con |
|-----|-----|
| | |

**Evidence**: [ai/research/ link]
**Commits**: [hash]

---
-->
