# Refactoring Plan: SOTA API Design

Created: 2025-11-05
Based on: ai/research/rust_media_api_design.md

## Executive Summary

Current implementation has **3 critical issues** and **3 design weaknesses** that prevent production use. This plan addresses them in order of severity.

## Critical Issues (Fix Immediately)

### Issue 1: Loading Entire Files Into Memory

**Current** (mead-core/src/container/mp4.rs:18-19):
```rust
let mut buffer = Vec::new();
reader.read_to_end(&mut buffer)?;  // Loads entire file - FAILS on large files
```

**Problem**:
- 4GB movie = 4GB RAM
- DoS via malformed metadata claiming huge size
- Violates SOTA pattern from mp4parse/symphonia

**Fix**: Location-based references
```rust
pub struct Mp4Demuxer<R: Read + Seek> {
    source: R,
    tracks: Vec<Track>,
    mdat_locations: Vec<DataLocation>,  // Offset + size, not data
}

pub struct DataLocation {
    offset: u64,
    size: u64,
}

impl<R: Read + Seek> Mp4Demuxer<R> {
    pub fn new(mut source: R) -> Result<Self> {
        // Parse box structure, storing offsets only
        let mut locations = Vec::new();

        while let Some(box_header) = read_box_header(&mut source)? {
            match box_header.box_type {
                BoxType::Mdat => {
                    // Store location, don't read data
                    locations.push(DataLocation {
                        offset: source.stream_position()?,
                        size: box_header.size,
                    });
                    // Skip past data
                    source.seek(SeekFrom::Current(box_header.size as i64))?;
                }
                _ => {
                    // Parse metadata boxes normally
                }
            }
        }

        Ok(Self { source, tracks, mdat_locations: locations })
    }

    pub fn read_packet(&mut self) -> Result<Option<Packet>> {
        // Seek to sample offset, read only that sample
        self.source.seek(SeekFrom::Start(sample_offset))?;
        let mut data = vec![0; sample_size];
        self.source.read_exact(&mut data)?;
        // ...
    }
}
```

**Impact**:
- Constant memory usage regardless of file size ✅
- Prevents DoS attacks ✅
- Matches symphonia/mp4parse patterns ✅

---

### Issue 2: Encoder API Doesn't Match rav1e Reality

**Current** (mead-core/src/codec/av1.rs:101-145):
```rust
impl VideoEncoder for Av1Encoder {
    fn encode(&mut self, frame: &Frame) -> Result<Vec<u8>> {
        self.context.send_frame(frame)?;
        match self.context.receive_packet() {
            Ok(packet) => Ok(packet.data.to_vec()),
            Err(EncoderStatus::NeedMoreData) => Ok(Vec::new()),  // ❌ Hides complexity
        }
    }
}
```

**Problem**:
- rav1e buffers frames for lookahead (2-pass encoding)
- Need to send multiple frames before getting packets
- Current API makes this invisible, leads to confusing empty returns

**Fix**: Expose send-receive pattern
```rust
pub trait Encoder {
    /// Send frame to encoder (None = EOF signal)
    fn send_frame(&mut self, frame: Option<Arc<Frame>>) -> Result<()>;

    /// Receive encoded packet (None = need more frames)
    fn receive_packet(&mut self) -> Result<Option<Packet>>;

    /// Convenience: Flush all remaining packets
    fn finish(&mut self) -> Result<Vec<Packet>> {
        self.send_frame(None)?;
        let mut packets = Vec::new();
        while let Some(packet) = self.receive_packet()? {
            packets.push(packet);
        }
        Ok(packets)
    }
}

// Usage:
let mut encoder = Av1Encoder::new(64, 64)?;

// Fill lookahead buffer
for frame in &frames[0..10] {
    encoder.send_frame(Some(frame.clone()))?;
}

// Main loop
loop {
    match encoder.receive_packet()? {
        Some(packet) => write_packet(packet),
        None => {
            // Need more input
            if let Some(frame) = next_frame() {
                encoder.send_frame(Some(frame))?;
            } else {
                break;  // No more input
            }
        }
    }
}

// Drain
for packet in encoder.finish()? {
    write_packet(packet);
}
```

**Impact**:
- Clear API that matches hardware encoders ✅
- Explicit lookahead control ✅
- Matches rav1e's actual behavior ✅

---

### Issue 3: No Arc<Frame> for Zero-Copy

**Current**:
```rust
pub struct Frame {
    pub data: Vec<u8>,  // Owned data
}

fn encode(&mut self, frame: &Frame) -> Result<Vec<u8>> {
    // Must copy frame data into encoder
}
```

**Problem**:
- Every pipeline stage copies frame data
- 1920x1080 YUV420 = 3MB per frame
- 30fps = 90MB/s copying

**Fix**: Arc<Frame> for sharing
```rust
pub struct Frame {
    planes: Vec<Plane>,
    width: u32,
    height: u32,
    format: PixelFormat,
}

pub trait Encoder {
    fn send_frame(&mut self, frame: Option<Arc<Frame>>) -> Result<()>;
}

pub trait Decoder {
    fn decode(&mut self, packet: &Packet) -> Result<Arc<Frame>>;
}

// Usage - zero copies:
let frame: Arc<Frame> = decoder.decode(&packet)?;
encoder.send_frame(Some(frame.clone()))?;  // Only increments refcount
filter.process(frame)?;  // Can share same frame
```

**Impact**:
- Zero-copy between pipeline stages ✅
- Thread-safe sharing ✅
- Matches rav1e pattern ✅

---

## Design Improvements (Fix in Phase 2)

### Issue 4: Frame Type Lacks SIMD Alignment

**Current**:
```rust
pub struct Frame {
    pub data: Vec<u8>,  // Standard allocation, may not be aligned
}
```

**Problem**:
- SSE/AVX require 16/32-byte alignment
- Unaligned access = slower or crashes

**Fix**: Use aligned-vec
```rust
use aligned_vec::AVec;

pub struct Plane {
    data: AVec<u8>,  // Aligned for SIMD
    stride: usize,
    width: usize,
    height: usize,
}

pub struct Frame {
    planes: Vec<Plane>,  // Y, U, V planes
    format: PixelFormat,
}

// Add to Cargo.toml:
// aligned-vec = "0.6"
```

**Impact**:
- Enables SIMD optimizations ✅
- Matches v_frame/rav1e patterns ✅
- ~2-4x speedup for pixel processing ✅

---

### Issue 5: No MediaSource Abstraction

**Current**:
```rust
impl Mp4Demuxer {
    pub fn new<R: Read>(reader: R) -> Result<Self>
}
```

**Problem**:
- Can't detect if source is seekable
- Can't handle stdin vs file differently
- Can't report stream length

**Fix**: MediaSource trait
```rust
pub trait MediaSource: Read + Seek {
    /// Returns true if source supports seeking
    fn is_seekable(&self) -> bool;

    /// Returns total size if known
    fn len(&self) -> Option<u64>;
}

// Implement for File
impl MediaSource for std::fs::File {
    fn is_seekable(&self) -> bool { true }
    fn len(&self) -> Option<u64> {
        self.metadata().ok().map(|m| m.len())
    }
}

// Wrapper for non-seekable sources
pub struct ReadOnlySource<R: Read> {
    inner: R,
}

impl<R: Read> MediaSource for ReadOnlySource<R> {
    fn is_seekable(&self) -> bool { false }
    fn len(&self) -> Option<u64> { None }
}

// Usage:
let file = std::fs::File::open("video.mp4")?;
let demuxer = Mp4Demuxer::new(file)?;  // Seekable

let stdin = ReadOnlySource::new(std::io::stdin());
let demuxer = Mp4Demuxer::new(stdin)?;  // Non-seekable
```

**Impact**:
- Unified API for files and streams ✅
- Runtime seekability detection ✅
- Matches symphonia pattern ✅

---

### Issue 6: Weak Trait Abstractions

**Current**:
- `Demuxer` trait exists but limited
- No `Encoder` trait
- No `FormatReader` trait
- Not extensible

**Fix**: Complete trait hierarchy
```rust
// mead-core/src/format/mod.rs
pub trait FormatReader {
    fn open<S: MediaSource>(source: S) -> Result<Self> where Self: Sized;
    fn tracks(&self) -> &[Track];
    fn read_packet(&mut self) -> Result<Option<Packet>>;
    fn seek(&mut self, timestamp: u64) -> Result<SeekResult>;
}

// mead-core/src/codec/mod.rs
pub trait Decoder {
    fn new(params: &CodecParams) -> Result<Self> where Self: Sized;
    fn decode(&mut self, packet: &Packet) -> Result<Arc<Frame>>;
    fn flush(&mut self) -> Result<Vec<Arc<Frame>>>;
}

pub trait Encoder {
    fn new(params: &EncoderParams) -> Result<Self> where Self: Sized;
    fn send_frame(&mut self, frame: Option<Arc<Frame>>) -> Result<()>;
    fn receive_packet(&mut self) -> Result<Option<Packet>>;
}

// Implementations
impl FormatReader for Mp4Demuxer { /* ... */ }
impl Decoder for Av1Decoder { /* ... */ }
impl Encoder for Av1Encoder { /* ... */ }
```

**Impact**:
- Pluggable formats/codecs ✅
- Feature flags for optional codecs ✅
- Extensible without forking ✅

---

## Minor Improvements

### 7. Add TryVec for Untrusted Sizes
```rust
// Prevent DoS from malformed metadata
fn try_allocate(size: usize) -> Result<Vec<u8>> {
    if size > MAX_REASONABLE_SIZE {
        return Err(Error::InvalidData("Size too large"));
    }
    Vec::try_with_capacity(size)
        .map_err(|_| Error::OutOfMemory)?
}
```

### 8. Add ResetRequired Error
```rust
pub enum Error {
    // ...
    #[error("Decoder needs reset (track changed)")]
    ResetRequired,
}
```

### 9. Strong Typing for Formats
```rust
pub enum ContainerFormat {
    Mp4,
    Mkv,
    Webm,
}

pub enum CodecType {
    Av1,
    H264,
    H265,
    Aac,
    Opus,
}

pub enum PixelFormat {
    Yuv420p,
    Yuv422p,
    Yuv444p,
}
```

---

## Implementation Strategy

### Phase 1a (Immediate - Fix Critical)
Priority: **HIGH** - Current code not production-ready

1. ✅ Add `MediaSource` trait + implementations
2. ✅ Refactor `Mp4Demuxer` to use location references (not load full file)
3. ✅ Change encoder API to send-receive pattern
4. ✅ Add `Arc<Frame>` to decoder/encoder APIs
5. ✅ Add `aligned-vec` dependency
6. ✅ Refactor `Frame` type to use aligned planes

**Estimate**: 1-2 sessions
**Risk**: Breaking changes to API (but current API is v0.0.0)
**Benefit**: Fixes DoS vulnerability, enables large files, correct encoder API

### Phase 1b (Soon - Complete Traits)
Priority: **MEDIUM** - Nice to have before v0.1.0

7. ⏳ Define `FormatReader`, `Decoder`, `Encoder` traits
8. ⏳ Implement traits for existing types
9. ⏳ Add strong typing (enums for formats/codecs)
10. ⏳ Add error variants (ResetRequired, OutOfMemory)

**Estimate**: 1 session
**Risk**: Low (additions, not changes)
**Benefit**: Clean API for Phase 2

### Phase 2 (Later - Advanced)
Priority: **LOW** - After codec expansion

11. 🔮 Registry system for codecs
12. 🔮 Typestate pattern if complexity grows
13. 🔮 Advanced seeking with keyframe alignment

---

## Breaking Changes

All of Phase 1a involves breaking changes:

**Before**:
```rust
let file = File::open("video.mp4")?;
let demuxer = Mp4Demuxer::new(file)?;

let mut encoder = Av1Encoder::new(64, 64)?;
let data = encoder.encode(&frame)?;
```

**After**:
```rust
let file = File::open("video.mp4")?;
let demuxer = Mp4Demuxer::new(file)?;  // Now uses MediaSource

let mut encoder = Av1Encoder::new(64, 64)?;
encoder.send_frame(Some(Arc::new(frame)))?;
if let Some(packet) = encoder.receive_packet()? {
    // ...
}
```

**Justification**:
- Still v0.0.0 - no stability guarantees
- Fixes critical issues
- Aligns with SOTA patterns
- Prepares for v0.1.0 release

---

## Testing Strategy

After each refactor:

1. **Unit tests**: Core functionality
2. **Integration tests**: Real MP4 files
3. **Memory tests**: Verify constant memory (large files)
4. **Performance tests**: Benchmark vs pre-refactor

---

## Documentation Updates

After refactoring:

1. Update `ai/STATUS.md` with new architecture
2. Update `ai/DECISIONS.md` with why we chose these patterns
3. Add rustdoc examples for new APIs
4. Create `docs/ARCHITECTURE.md` explaining trait system

---

## Decision Points

### Q1: Do all of Phase 1a now, or incrementally?

**Option A: All at once**
- Pros: One breaking change, cleaner
- Cons: Larger PR, more to test

**Option B: Incremental**
- Pros: Smaller changes, easier to review
- Cons: Multiple breaking changes

**Recommendation**: Option A - we're v0.0.0, no users yet, might as well do it right

### Q2: Add v_frame dependency or roll our own Frame type?

**Option A: Use v_frame**
- Pros: Battle-tested, used by rav1e
- Cons: Brings in dependencies, Issue #136 (unsafe without validation)

**Option B: Custom Frame type**
- Pros: Full control, minimal dependencies
- Cons: More code to maintain

**Recommendation**: Option B for now - v_frame's unsafe issues concern us given safety focus. Can switch later if needed.

### Q3: Support non-seekable sources in Phase 1?

MP4 format requires seeking (sample tables at end of file). Non-seekable sources need:
- Fragmented MP4 (fMP4) with moof boxes
- Or buffer entire stream

**Recommendation**: Require seekable sources for Phase 1. Add fragmented MP4 support in Phase 3.

---

## Risks & Mitigations

| Risk | Impact | Mitigation |
|------|--------|------------|
| Breaking API changes | Users need updates | v0.0.0 = no stability promise |
| Increased complexity | Harder to maintain | Comprehensive tests + docs |
| Performance regression | Slower than current | Benchmark before/after |
| New bugs introduced | Regressions | Keep old code in tests for comparison |

---

## Success Criteria

Phase 1a is complete when:

✅ MP4 demuxer uses constant memory (tested with 10GB file)
✅ Encoder API uses send-receive pattern correctly
✅ All tests pass (6 existing + new tests)
✅ Zero clippy warnings
✅ Can handle `mead info large-video.mp4` without loading full file
✅ Documentation updated

---

## References

- Research: `ai/research/rust_media_api_design.md`
- Current status: `ai/STATUS.md`
- Todos: `ai/TODO.md`
