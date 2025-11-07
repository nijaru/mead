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

## 2025-11-06: Production CLI UX (Phase 2b)

**Context**: Current CLI uses plain println!, no progress bars, no colors, no human-readable formatting. FFmpeg's strength is real-time feedback during long encodes (frame count, fps, speed, ETA). If mead is to replace FFmpeg, users need production-quality UX.

**Decision**: Add Phase 2b (Production CLI UX) before completing more codecs

**Requirements**:
- Progress bars during encode/decode operations (indicatif)
- Colored output with TTY detection (console crate)
- Human-readable formatting (bytes, durations, speeds)
- Real-time metrics: fps, speed (x realtime), bitrate, ETA
- Respect NO_COLOR environment variable
- Output separation: progress/logs → stderr, data → stdout
- Scripting flags: --quiet, --json, --no-color

**Rationale**:
- Encoding can take hours - users need confidence tool isn't frozen
- FFmpeg's real-time progress is table stakes for media tools
- Modern Rust CLIs use indicatif (industry standard)
- Without progress bars, mead feels like a toy vs FFmpeg
- Better to build good UX patterns early than retrofit later

**Tradeoffs**:
| Pro | Con |
|-----|-----|
| Professional UX matching FFmpeg | Delays codec development |
| Users trust the tool during long ops | Additional dependencies |
| Good patterns for future features | More complex output handling |
| Essential for production use | - |

**Implementation**:
```rust
// Dependencies
indicatif = "0.17"  // Progress bars
console = "0.15"    // Colors, TTY detection

// Example usage
let pb = ProgressBar::new(total_frames);
pb.set_style(ProgressStyle::with_template(
  "[{elapsed_precise}] {bar:40.cyan/blue} {pos}/{len} {msg} {per_sec} ETA: {eta}"
)?);
```

**Evidence**: ai/research/cli_ux_best_practices.md

---

## 2025-11-06: Y4M Input for Real Transcoding (Phase 2d)

**Context**: After completing encode pipeline in Phase 2c, mead could only generate test patterns. To transcode real video, we needed either:
1. H.264 decoder (requires unsafe OpenH264 bindings or immature pure Rust)
2. Y4M raw video input (pure Rust, widely used in professional workflows)

**Decision**: Add Y4M demuxer support before H.264 decoder

**Rationale**:
- Y4M is raw YUV - no decoding needed (stays memory-safe)
- y4m crate is pure Rust, well-maintained (v0.8)
- Professional workflow: `ffmpeg -f yuv4mpegpipe - | mead encode -`
- Common in video processing pipelines (ffmpeg, x264, x265)
- Enables real transcoding testing without unsafe code
- Validates full encode pipeline with actual video frames
- Better to prove architecture works before adding complexity

**Tradeoffs**:
| Pro | Con |
|-----|-----|
| Pure Rust, memory-safe | Users need ffmpeg for MP4 input |
| Common professional workflow | Extra step vs direct MP4 |
| Tests encode pipeline with real video | Can't transcode MP4 standalone yet |
| Validates Arc<Frame> zero-copy design | - |

**Implementation**:
```rust
// Y4M supports common YUV formats
Y4mDemuxer::new(stdin())?  // Stdin piping
Y4mDemuxer::new(File::open("input.y4m")?)?  // File input

// Supports YUV420p, YUV422p, YUV444p
```

**Results**:
- Full transcode: Y4M → AV1 → IVF at 25-48 fps
- 36 tests passing (30 core + 4 output + 2 doctests)
- Valid IVF output playable in VLC/ffmpeg/dav1d
- Professional workflow integration confirmed

**Next**: Phase 3 will add H.264/H.265 decoders for standalone MP4 transcoding

---

## 2025-11-06: SVT-AV1 Default with rav1e Option (Phase 2e Strategic Decision)

**Context**: After implementing tile parallelism optimization, rav1e achieves 20-40 fps while SVT-AV1 reaches 100-150 fps. Users trying mead vs ffmpeg would experience 5× slower encoding, making adoption difficult despite superior UX. Initial positioning as "pure Rust only" creates competitive disadvantage.

**Decision**: Use SVT-AV1 as default encoder, offer rav1e as `--encoder rav1e` option

**Strategic Framing**: Pivot from "pure Rust media tool" to "modern AV1 tool with Rust ergonomics"

**CLI Design**:
```bash
# Default: Fast (SVT-AV1, 100+ fps)
mead encode input.y4m -o output.ivf

# Pure Rust option (rav1e, 20-40 fps)
mead encode input.y4m -o output.ivf --encoder rav1e

# Future: More encoders
mead encode input.y4m -o output.ivf --encoder x264
```

**Rationale**:
- **Competitive performance**: Must match ffmpeg speed to gain adoption (100+ fps)
- **SVT-AV1 is safe**: Zero CVEs in 4 years, heavily fuzzed by Netflix/Intel/YouTube
- **Honest positioning**: Don't die on "pure Rust" hill, compete on UX instead
- **User choice preserved**: Pure Rust still available for those who want it
- **Better first impression**: Users won't immediately bounce due to slowness
- **Servo model**: Hybrid approach, incrementally move to more Rust over time
- **mead-core stays safe**: `#![forbid(unsafe_code)]` remains, bindings in CLI only

**Performance Data**:
| Resolution | rav1e (optimized) | SVT-AV1 | Gap |
|------------|-------------------|---------|-----|
| 720p | 41.55 fps | 142.29 fps | 3.42× |
| 1080p | 20.14 fps | 108.35 fps | 5.37× |

**Tradeoffs**:
| Pro | Con |
|-----|-----|
| Competitive with ffmpeg performance | C dependency (libsvtav1) |
| Real adoption opportunity | Weakens "pure Rust" positioning |
| SVT-AV1 battle-tested (Netflix, YouTube) | Requires system library or vendoring |
| User choice (can still use rav1e) | Bindings maintenance burden |
| Honest safety claims (SVT-AV1 is safe!) | Installation complexity on some systems |

**Architecture Impact**:
- **mead-core**: Remains `#![forbid(unsafe_code)]`, pure Rust API
- **mead CLI**: Can use C bindings where necessary for performance
- **Incremental path**: Add safe Rust implementations over time
  - rav1e already available (pure Rust)
  - Future: rav1d decoder (safe Rust dav1d port)
  - Future: pure Rust H.264/H.265 as they mature

**Positioning Change**:
```markdown
# Before: "Memory-safe media processing"
# Problem: Implies existing tools are unsafe (controversial)
# Problem: 5× slower = dead on arrival

# After: "Modern AV1 encoding with Rust ergonomics"
# Benefits:
- Fast by default (SVT-AV1)
- Pure Rust option available (rav1e)
- Better UX than ffmpeg (presets, progress bars)
- Modern CLI patterns (rg/fd/bat style)
```

**Implementation Plan**:
1. Evaluate SVT-AV1 Rust bindings (`svt-av1-enc`, `svt-av1-rs`)
2. Add `--encoder` flag to CLI (`svt-av1` default, `rav1e` option)
3. Keep rav1e fully supported (tests, benchmarks, documentation)
4. Update README/CLAUDE.md with new positioning
5. Document installation (system svtav1 or vendored)

**Evidence**:
- ai/research/encoder_comparison.md (benchmark results)
- ai/research/av1_encoder_settings.md (performance analysis)
- ai/OPTIMIZATION_SUMMARY.md (tile parallelism work)

**Analogy**: Servo/Firefox model
- Servo: Pure Rust browser engine experiments
- Firefox: Integrated Servo components incrementally (Quantum project)
- mead: Start hybrid, incrementally adopt more pure Rust as ecosystem matures

**Commits**: TBD (implementation in progress)

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
