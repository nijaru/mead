# State-of-the-Art Rust Media API Design Patterns

Research conducted: 2025-11-05

## Executive Summary

This document synthesizes API design patterns from leading Rust media libraries: symphonia (audio), mp4parse-rust (Mozilla MP4), rav1e (AV1 encoder), and v_frame (video frames). Key patterns include trait-based extensibility, typestate machines, multiple buffer abstractions, and zero-copy sharing via Arc.

## Table of Contents

1. [Trait-Based Architecture](#trait-based-architecture)
2. [I/O Abstraction Patterns](#io-abstraction-patterns)
3. [Buffer Management](#buffer-management)
4. [Memory Efficiency Patterns](#memory-efficiency-patterns)
5. [Type Safety Patterns](#type-safety-patterns)
6. [Error Handling](#error-handling)
7. [State Management](#state-management)
8. [Seeking and Track Selection](#seeking-and-track-selection)
9. [Send-Receive Pattern](#send-receive-pattern)
10. [Code Examples](#code-examples)
11. [Recommendations for mead](#recommendations-for-mead)

---

## 1. Trait-Based Architecture

### symphonia: Pluggable Codec/Format System

**Core Traits**:
- `FormatReader` - Container demuxers (MP4, MKV, WebM)
- `Decoder` - Audio/video codec decoders
- `MediaSource` - I/O abstraction (Read + Seek)

**Pattern**: Registry-based plugin system
```rust
// Format readers implement FormatReader trait
trait FormatReader {
    fn try_new(options: &FormatOptions, source: MediaSourceStream) -> Result<Self>;
    fn next_packet(&mut self) -> Result<Packet>;
    fn tracks(&self) -> &[Track];
    fn seek(&mut self, to: SeekTo) -> Result<SeekedTo>;
    fn metadata(&mut self) -> Metadata;
}

// Register implementations with probe
let probed = symphonia::default::get_probe()
    .format(&hint, mss, &fmt_opts, &meta_opts)?;
```

**Benefits**:
- Users enable only needed codecs via feature flags
- Custom implementations via trait implementation
- NO unsafe blocks outside of symphonia-core
- Extensible without modifying core library

### Recommendations for mead

```rust
// mead-core/src/format/mod.rs
pub trait FormatReader {
    fn open(source: MediaSource) -> Result<Self> where Self: Sized;
    fn tracks(&self) -> &[Track];
    fn read_packet(&mut self) -> Result<Option<Packet>>;
    fn seek(&mut self, timestamp: u64) -> Result<()>;
}

pub trait Decoder {
    fn new(params: &CodecParams) -> Result<Self> where Self: Sized;
    fn decode(&mut self, packet: &Packet) -> Result<Frame>;
    fn flush(&mut self) -> Result<()>;
}

pub trait Encoder {
    fn new(params: &EncoderParams) -> Result<Self> where Self: Sized;
    fn encode(&mut self, frame: &Frame) -> Result<Vec<Packet>>;
    fn finish(&mut self) -> Result<Vec<Packet>>;
}
```

---

## 2. I/O Abstraction Patterns

### symphonia: MediaSource Trait

**Pattern**: Composite trait combining Read + Seek
```rust
pub trait MediaSource: Read + Seek {
    fn is_seekable(&self) -> bool;
    fn byte_len(&self) -> Option<u64>;
}
```

**Key Insight**: Runtime seekability detection
- Files implement MediaSource with `is_seekable() = true`
- Streams wrap with `ReadOnlySource::new()` for `is_seekable() = false`
- Allows same API for files and network streams

**Usage Pattern**:
```rust
let src = std::fs::File::open(&path)?;
let mss = MediaSourceStream::new(Box::new(src), Default::default());
// OR for stdin:
let stdin = std::io::stdin();
let src = ReadOnlySource::new(stdin.lock());
let mss = MediaSourceStream::new(Box::new(src), Default::default());
```

### mp4parse-rust: Offset Tracking

**Pattern**: Wrap readers to track byte positions
```rust
struct OffsetReader<T> {
    inner: T,
    offset: u64,
}
```

**Benefits**:
- Track position without seeking
- Enable location-based data references (avoid copies)
- Support unknown-length boxes (stream until EOF)

### Recommendations for mead

```rust
// mead-core/src/io.rs
pub trait MediaSource: Read + Seek {
    fn is_seekable(&self) -> bool;
    fn len(&self) -> Option<u64>;
}

pub struct MediaSourceStream<T: Read> {
    inner: T,
    buffer: Vec<u8>,
    pos: u64,
}

impl<T: Read> MediaSourceStream<T> {
    pub fn new(inner: T, buffer_size: usize) -> Self;
    pub fn position(&self) -> u64;
}
```

---

## 3. Buffer Management

### symphonia: Multiple Buffer Abstractions

**Four buffer types for different use cases**:

1. **AudioBuffer<S>** - Planar, strongly-typed working buffer
   - Generic over sample type (i8, i16, i32, f32, f64)
   - Planar layout (separate arrays per channel)
   - Used internally by decoders

2. **AudioBufferRef** - Type-erased enum for any AudioBuffer
   - Copy-on-write semantics
   - Allows handling unknown sample formats at compile time
   - Used for decoder output

3. **SampleBuffer<S>** - Native sample type interop
   - Sample-oriented (agnostic to layout)
   - For moving data in/out of Symphonia
   - Uses native in-memory representation

4. **RawSampleBuffer** - Byte-oriented export
   - Converts all samples to packed data type
   - Stream of bytes for FFI or raw I/O
   - For interfacing with C libraries or file formats

**Pattern**: Choose buffer type based on use case
- Internal processing: AudioBuffer (planar, type-safe)
- API boundaries: AudioBufferRef (type-erased)
- User interop: SampleBuffer (native layout)
- FFI/raw I/O: RawSampleBuffer (bytes)

### v_frame: Aligned Video Frames

**Dependencies**:
- `aligned-vec` - SIMD-aligned memory allocation
- Frame and Plane types for YUV data

**Pattern**: Alignment for SIMD operations
```rust
// Uses aligned-vec for proper memory alignment
// Critical for vectorized operations (SSE, AVX, NEON)
pub struct Plane<T> {
    data: AVec<T>,  // aligned-vec type
    stride: usize,
}
```

### Audio Layout Patterns

**Three main layouts**:

1. **Interleaved** - `[L, R, L, R, L, R]`
   - Good cache locality for small channel counts
   - Natural for stereo audio
   - Used by audio hardware APIs

2. **Sequential/Planar** - `[L, L, L, R, R, R]` in one allocation
   - Better for DSP (process entire channel at once)
   - Simplified SIMD operations

3. **Planar** - Separate buffers per channel
   - Maximum flexibility
   - Used by most codecs internally
   - Symphonia's AudioBuffer uses this

### Recommendations for mead

```rust
// mead-core/src/buffer.rs

// Video frames - use Arc for zero-copy sharing
pub struct Frame {
    pub planes: Vec<Plane>,
    pub width: usize,
    pub height: usize,
    pub format: PixelFormat,
    pub timestamp: u64,
}

pub struct Plane {
    data: AVec<u8>,  // aligned for SIMD
    stride: usize,
}

// Audio samples - planar by default
pub struct AudioBuffer {
    samples: Vec<AVec<f32>>,  // one AVec per channel
    channels: usize,
    sample_rate: u32,
}

// For interop - byte-oriented
pub struct RawBuffer {
    data: Vec<u8>,
    format: BufferFormat,
}
```

---

## 4. Memory Efficiency Patterns

### Core Principle: Avoid Loading Entire Files

All libraries avoid loading full files into memory.

### mp4parse-rust: Location-Based References

**Pattern**: Store offsets instead of copying data
```rust
pub enum IsobmffItem {
    MdatLocation {
        offset: u64,
        size: u64,
    },
    // ... other variants
}
```

**For unknown-size boxes** (final mdat in stream):
- Read in 64KB chunks
- Process incrementally
- Don't buffer entire box

### symphonia: Streaming Box Iterator

**Pattern**: Iterator over boxes
```rust
struct BoxIter<'a, T> {
    reader: &'a mut T,
}

impl<'a, T: Read> Iterator for BoxIter<'a, T> {
    type Item = Result<Box>;
    fn next(&mut self) -> Option<Self::Item> {
        // Read next box header
        // Return box metadata without reading payload
    }
}
```

**Benefits**:
- Parse structure without reading payload
- User decides which boxes to read fully
- Constant memory regardless of file size

### symphonia: Fallible Collections

**Pattern**: TryVec - fail gracefully on allocation
```rust
// Instead of Vec::with_capacity() that can panic
struct TryVec<T> { /* ... */ }

impl<T> TryVec<T> {
    fn try_with_capacity(n: usize) -> Result<Self, Error>;
}
```

**Why**: Malformed files can claim huge sizes
- Standard Vec panics on allocation failure
- TryVec returns Err(OutOfMemory)
- Prevents DoS via malformed metadata

### rav1e: Arc<Frame> for Zero-Copy

**Pattern**: Reference-counted frames
```rust
pub fn send_frame<F>(&mut self, frame: F) -> Result<(), EncoderStatus>
where F: Into<Option<Arc<Frame<T>>>>
```

**Benefits**:
- Multiple references to same frame data
- No copying between encoder stages
- Automatic cleanup when all refs dropped
- Thread-safe (Arc uses atomic refcounts)

**Trade-offs**:
- Small overhead (atomic increment/decrement)
- Can't mutate shared frames
- Risk of reference cycles (rare in media pipelines)

### Recommendations for mead

```rust
// Use Arc<Frame> for sharing between pipeline stages
pub fn decode(&mut self, packet: &Packet) -> Result<Arc<Frame>>;

// Location-based references for large data
pub struct SampleTable {
    location: DataLocation,  // offset + size
}

pub enum DataLocation {
    Buffered(Vec<u8>),
    External { offset: u64, size: u64 },
}

// Fallible allocations for untrusted input
pub struct TryVec<T> { /* ... */ }
```

---

## 5. Type Safety Patterns

### symphonia: Strong Box Typing

**Pattern**: Enum for known box types + FourCC wrapper
```rust
pub enum BoxType {
    Ftyp,
    Moov,
    Mdat,
    // ... known types
    Unknown(FourCC),
}

pub struct FourCC([u8; 4]);

pub struct FileTypeBox {
    pub major_brand: FourCC,
    pub minor_version: u32,
    pub compatible_brands: Vec<FourCC>,
}
```

**Benefits**:
- Type-safe access to known structures
- Can't mix up box types
- Unknown boxes handled gracefully
- No string comparisons at runtime

### v_frame: Pixel Format Type Safety

**Challenge**: FFmpeg uses runtime ints for pixel formats
```c
enum AVPixelFormat {
    AV_PIX_FMT_YUV420P,
    AV_PIX_FMT_RGB24,
    // ... 200+ formats
};
```

**Rust approach**: Type-safe enums or generics
```rust
pub enum PixelFormat {
    Yuv420p,
    Yuv422p,
    Rgb24,
}

// Or generic over pixel type
pub struct Frame<T: Pixel> {
    planes: Vec<Plane<T>>,
}
```

**v_frame issue**: Frame::copy_from_raw_parts is unsafe without `unsafe` block
- Doesn't validate input pointers
- Rust-av issue #136 documents this
- Lesson: FFI boundaries need extra validation

### Typestate Pattern for State Machines

**Pattern**: Encode states in type system

**File API example**:
```rust
struct File<State> {
    inner: fs::File,
    _state: PhantomData<State>,
}

struct Reading;
struct Eof;

impl File<Reading> {
    pub fn read(self) -> Result<(Vec<u8>, File<Reading>), File<Eof>> {
        // Returns either more data + Reading state
        // or EOF + Eof state
    }
}

impl File<Eof> {
    // Can't call read() - method doesn't exist
    pub fn close(self) { /* ... */ }
}
```

**Benefits**:
- Compile-time state validation
- Can't call read() on closed file
- No runtime state checks
- Clear API contract

**Trade-offs**:
- More verbose (separate types/type params)
- Users handle type transitions
- Slightly more complex error handling

**Application to decoders**:
```rust
struct Decoder<State> {
    inner: DecoderImpl,
    _state: PhantomData<State>,
}

struct Uninitialized;
struct Initialized;
struct Decoding;
struct Flushed;

impl Decoder<Uninitialized> {
    pub fn new() -> Self;
    pub fn init(self, params: CodecParams) -> Result<Decoder<Initialized>>;
}

impl Decoder<Initialized> {
    pub fn decode(self, packet: Packet) -> Result<(Frame, Decoder<Decoding>)>;
}

impl Decoder<Decoding> {
    pub fn decode(self, packet: Packet) -> Result<(Frame, Decoder<Decoding>)>;
    pub fn flush(self) -> Decoder<Flushed>;
}

impl Decoder<Flushed> {
    // Can't decode anymore
    pub fn reset(self) -> Decoder<Initialized>;
}
```

### Recommendations for mead

```rust
// Strong typing for containers
pub enum ContainerFormat {
    Mp4,
    Mkv,
    Webm,
}

pub enum CodecType {
    Av1,
    H264,
    Aac,
    Opus,
}

// Pixel format type safety
pub enum PixelFormat {
    Yuv420p,
    Yuv422p,
    Yuv444p,
    Rgb24,
}

// Consider typestate for decoder/encoder APIs
// Start with simple approach, add typestate if needed
```

---

## 6. Error Handling

### symphonia: Structured Error Types

**Pattern**: Enum covering all error cases
```rust
pub enum Error {
    IoError(io::Error),
    DecodeError(&'static str),
    SeekError(SeekErrorKind),
    Unsupported(&'static str),
    LimitError(&'static str),
    ResetRequired,
}
```

**Key insight**: ResetRequired error
- Not a failure - signals track change
- Decoder needs to be recreated for new codec params
- Part of normal operation for multi-track files

### mp4parse-rust: FFI-Compatible Errors

**Pattern**: Simple enum + rich Error type
```rust
// C-compatible status codes
pub enum Status {
    Ok = 0,
    BadArg = 1,
    Invalid = 2,
    Unsupported = 3,
    Eof = 4,
    Io = 5,
    Oom = 6,
}

// Rich Rust error type
pub enum Error {
    InvalidData(Status),
    UnexpectedEof,
    Unsupported,
    OutOfMemory,
    // ...
}

impl From<Error> for Status {
    fn from(err: Error) -> Status {
        match err {
            Error::InvalidData(s) => s,
            Error::UnexpectedEof => Status::Eof,
            // ...
        }
    }
}
```

**Why**: Support both Rust and C APIs
- Rust code uses Result<T, Error>
- C API uses Status return codes
- Conversion via From trait

### Recommendations for mead

```rust
// mead-core/src/error.rs
use thiserror::Error;

#[derive(Error, Debug)]
pub enum Error {
    #[error("I/O error: {0}")]
    Io(#[from] std::io::Error),

    #[error("Invalid data: {0}")]
    InvalidData(&'static str),

    #[error("Unsupported: {0}")]
    Unsupported(&'static str),

    #[error("Decoder needs reset (track changed)")]
    ResetRequired,

    #[error("Out of memory")]
    OutOfMemory,

    #[error("Seek error: {0}")]
    Seek(#[from] SeekError),
}

pub type Result<T> = std::result::Result<T, Error>;
```

---

## 7. State Management

### Decoder State Lifecycle

**Common pattern across libraries**:
1. **Create** - Allocate decoder, no resources yet
2. **Initialize** - Configure with codec parameters
3. **Decode** - Process packets → frames
4. **Flush** - Drain buffered frames (end of stream)
5. **Reset** - Reinitialize (track change, seek)
6. **Destroy** - Cleanup (via Drop trait)

### symphonia: State in Decoder Trait

**Pattern**: Methods handle state implicitly
```rust
pub trait Decoder {
    fn try_new(params: &CodecParameters, options: &DecoderOptions) -> Result<Box<dyn Decoder>>;
    fn decode(&mut self, packet: &Packet) -> Result<AudioBufferRef>;
    fn finalize(&mut self) -> FinalizeResult;
    fn reset(&mut self);
}
```

**State tracking**: Internal to decoder implementation
- User doesn't see state directly
- Methods return errors for invalid operations
- ResetRequired error signals state change needed

### rav1e: Explicit State in Status Enum

**Pattern**: Operations return status indicating state
```rust
pub enum EncoderStatus {
    Success,
    EnoughData,  // Encoder has enough frames buffered
    LimitReached,
    Failure,
    NotReady,  // Encoder not configured yet
    NeedMoreData,  // Need to send more frames
}

impl Context {
    pub fn send_frame(&mut self, frame: Option<Arc<Frame>>) -> Result<(), EncoderStatus>;
    pub fn receive_packet(&mut self) -> Result<Packet, EncoderStatus>;
}
```

**Usage pattern**: Check status to drive encoder
```rust
loop {
    match encoder.receive_packet() {
        Ok(packet) => handle_packet(packet),
        Err(EncoderStatus::NeedMoreData) => {
            encoder.send_frame(next_frame)?;
        },
        Err(EncoderStatus::LimitReached) => break,
        Err(e) => return Err(e.into()),
    }
}
```

### Recommendations for mead

**Option 1: Simple (like symphonia)**
```rust
pub trait Decoder {
    fn new(params: &CodecParams) -> Result<Self>;
    fn decode(&mut self, packet: &Packet) -> Result<Frame>;
    fn flush(&mut self) -> Result<Vec<Frame>>;  // Drain buffered
}
```

**Option 2: Explicit status (like rav1e)**
```rust
pub enum DecoderStatus {
    Success,
    NeedMoreData,
    Finished,
}

pub trait Decoder {
    fn decode(&mut self, packet: Option<&Packet>) -> Result<Option<Frame>, DecoderStatus>;
}
```

**Recommendation**: Start with Option 1 (simpler), add explicit status if needed.

---

## 8. Seeking and Track Selection

### symphonia: Track Selection Pattern

**Pattern**: Find first suitable track
```rust
let track = format.tracks()
    .iter()
    .find(|t| t.codec_params.codec != CODEC_TYPE_NULL)
    .expect("no supported audio tracks");

let track_id = track.id;
```

**Important**: Default track may not be what you want
- Video files have multiple tracks (video, audio, subtitles)
- Must explicitly select audio track for audio processing
- Track filtering by codec, language, or other metadata

### symphonia: Seeking API

**Pattern**: Seek to timestamp or frame
```rust
pub trait FormatReader {
    fn seek(&mut self, to: SeekTo) -> Result<SeekedTo>;
}

pub enum SeekTo {
    Time { time: Time },
    Frame { frame: u64 },
}

pub struct SeekedTo {
    pub track_id: u32,
    pub required_ts: u64,
    pub actual_ts: u64,
}
```

**Key insight**: Actual position may differ from requested
- Codecs often require keyframe alignment
- Seek returns actual position for decoder reset
- Required for gapless playback

### mp4parse-rust: Sample Table Access

**Pattern**: Expose raw tables, user calculates
```rust
pub struct Track {
    pub stts: TimeToSampleBox,  // decode time
    pub stsc: SampleToChunkBox,  // chunk mapping
    pub stsz: SampleSizeBox,     // sample sizes
    pub stco: ChunkOffsetBox,    // chunk positions
}
```

**Trade-off**: Low-level but efficient
- No iterator abstraction
- User must walk tables manually
- Avoids allocations for index structures

### Recommendations for mead

```rust
// High-level seeking
pub trait FormatReader {
    fn seek(&mut self, timestamp: u64) -> Result<SeekResult>;
}

pub struct SeekResult {
    pub actual_timestamp: u64,
    pub is_keyframe: bool,
}

// Track selection with filters
pub trait FormatReader {
    fn tracks(&self) -> &[Track];

    fn select_track(&mut self, id: u32) -> Result<()>;

    fn find_track<F>(&self, predicate: F) -> Option<&Track>
    where F: Fn(&Track) -> bool;
}

// Helper methods
impl FormatReader {
    fn default_video_track(&self) -> Option<&Track> {
        self.find_track(|t| t.codec_type.is_video())
    }

    fn default_audio_track(&self) -> Option<&Track> {
        self.find_track(|t| t.codec_type.is_audio())
    }
}
```

---

## 9. Send-Receive Pattern

### rav1e: Producer-Consumer Model

**Pattern**: Separate frame submission from packet retrieval
```rust
impl Context<T> {
    pub fn send_frame<F>(&mut self, frame: F) -> Result<(), EncoderStatus>
    where F: Into<Option<Arc<Frame<T>>>>;

    pub fn receive_packet(&mut self) -> Result<Packet<T>, EncoderStatus>;
}
```

**Usage cycle**:
1. Setup: Create encoder context
2. Loop:
   - Call `receive_packet()` until `NeedMoreData`
   - Call `send_frame()` to provide input
3. Finalize: Send None to signal end, drain packets

**Why this pattern**:
- Encoder buffers frames internally (lookahead)
- Decouples input from output
- Natural for streaming (continuous input/output)
- Matches hardware encoder APIs

**Example**:
```rust
let mut encoder = Context::new(&config)?;

// Send several frames to fill lookahead buffer
for _ in 0..lookahead_frames {
    encoder.send_frame(Some(next_frame()))?;
}

// Main encode loop
loop {
    match encoder.receive_packet() {
        Ok(packet) => {
            write_packet(packet);
        },
        Err(EncoderStatus::NeedMoreData) => {
            match next_frame() {
                Some(frame) => encoder.send_frame(Some(frame))?,
                None => encoder.send_frame(None)?,  // Signal EOF
            }
        },
        Err(EncoderStatus::LimitReached) => break,
        Err(e) => return Err(e.into()),
    }
}

// Drain remaining packets
while let Ok(packet) = encoder.receive_packet() {
    write_packet(packet);
}
```

### Alternative: Iterator Pattern

**Pattern**: Decoder as iterator over frames
```rust
impl Decoder {
    fn frames(&mut self) -> impl Iterator<Item = Result<Frame>> + '_ {
        std::iter::from_fn(move || {
            match self.decode_next() {
                Ok(frame) => Some(Ok(frame)),
                Err(Error::Eof) => None,
                Err(e) => Some(Err(e)),
            }
        })
    }
}
```

**When to use**:
- Simpler for synchronous decoding
- Natural Rust idiom
- Good for batch processing
- Less flexible for streaming

### Recommendations for mead

**Decoder**: Iterator pattern (simpler for most use cases)
```rust
pub trait Decoder {
    fn decode(&mut self, packet: &Packet) -> Result<Frame>;
}

// Extension trait for iterator
pub trait DecoderExt: Decoder {
    fn frames<'a>(&'a mut self, packets: impl Iterator<Item = Packet> + 'a)
        -> impl Iterator<Item = Result<Frame>> + 'a;
}
```

**Encoder**: Send-receive pattern (matches hardware APIs)
```rust
pub trait Encoder {
    fn send_frame(&mut self, frame: Option<Arc<Frame>>) -> Result<()>;
    fn receive_packet(&mut self) -> Result<Option<Packet>>;
}
```

---

## 10. Code Examples

### Complete Decoding Pipeline (symphonia-style)

```rust
use mead_core::{FormatReader, Decoder, MediaSource};

fn decode_audio(path: &Path) -> Result<Vec<AudioBuffer>> {
    // 1. Open media source
    let file = std::fs::File::open(path)?;
    let source = MediaSource::new(file);

    // 2. Detect format and open reader
    let mut reader = probe_format(source)?;

    // 3. Select audio track
    let track = reader.tracks()
        .iter()
        .find(|t| t.codec_type.is_audio())
        .ok_or(Error::NoAudioTrack)?;

    // 4. Create decoder
    let mut decoder = create_decoder(&track.codec_params)?;

    // 5. Decode loop
    let mut frames = Vec::new();
    loop {
        // Read packet
        let packet = match reader.read_packet()? {
            Some(pkt) if pkt.track_id() == track.id => pkt,
            Some(_) => continue,  // Skip other tracks
            None => break,  // EOF
        };

        // Decode packet
        match decoder.decode(&packet) {
            Ok(frame) => frames.push(frame),
            Err(Error::ResetRequired) => {
                // Track changed, recreate decoder
                decoder = create_decoder(&track.codec_params)?;
            },
            Err(e) => return Err(e),
        }
    }

    // 6. Flush decoder
    frames.extend(decoder.flush()?);

    Ok(frames)
}
```

### Complete Encoding Pipeline (rav1e-style)

```rust
use mead_core::{Encoder, EncoderConfig, Frame};

fn encode_video(frames: Vec<Frame>, output: &Path) -> Result<()> {
    // 1. Configure encoder
    let config = EncoderConfig::default()
        .with_speed(6)
        .with_width(1920)
        .with_height(1080);

    // 2. Create encoder
    let mut encoder = Av1Encoder::new(config)?;

    // 3. Send initial frames (lookahead buffer)
    for frame in frames.iter().take(encoder.lookahead_frames()) {
        encoder.send_frame(Some(frame.clone()))?;
    }

    // 4. Encode loop
    let mut output_file = std::fs::File::create(output)?;
    let mut frame_idx = encoder.lookahead_frames();

    loop {
        match encoder.receive_packet()? {
            Some(packet) => {
                // Write packet to file
                output_file.write_all(packet.data())?;
            },
            None => {
                // Encoder needs more frames
                if frame_idx < frames.len() {
                    encoder.send_frame(Some(frames[frame_idx].clone()))?;
                    frame_idx += 1;
                } else {
                    // Signal EOF
                    encoder.send_frame(None)?;
                    break;
                }
            }
        }
    }

    // 5. Drain remaining packets
    while let Some(packet) = encoder.receive_packet()? {
        output_file.write_all(packet.data())?;
    }

    Ok(())
}
```

### Using Typestate Pattern for Decoder

```rust
use std::marker::PhantomData;

// State types
struct Uninitialized;
struct Ready;
struct Decoding;

struct Decoder<State> {
    inner: Box<dyn DecoderImpl>,
    _state: PhantomData<State>,
}

// Only available in Uninitialized state
impl Decoder<Uninitialized> {
    pub fn new() -> Self {
        Self {
            inner: Box::new(DecoderImpl::new()),
            _state: PhantomData,
        }
    }

    pub fn initialize(self, params: &CodecParams) -> Result<Decoder<Ready>> {
        self.inner.init(params)?;
        Ok(Decoder {
            inner: self.inner,
            _state: PhantomData,
        })
    }
}

// Available in Ready or Decoding state
impl Decoder<Ready> {
    pub fn decode(self, packet: &Packet) -> Result<(Frame, Decoder<Decoding>)> {
        let frame = self.inner.decode(packet)?;
        Ok((frame, Decoder {
            inner: self.inner,
            _state: PhantomData,
        }))
    }
}

impl Decoder<Decoding> {
    pub fn decode(self, packet: &Packet) -> Result<(Frame, Decoder<Decoding>)> {
        let frame = self.inner.decode(packet)?;
        Ok((frame, self))
    }

    pub fn finish(mut self) -> Result<Vec<Frame>> {
        self.inner.flush()
    }
}

// Usage - compile-time enforcement
fn main() {
    let decoder = Decoder::new();
    // decoder.decode(&packet);  // ERROR: method not found

    let decoder = decoder.initialize(&params)?;
    let (frame1, decoder) = decoder.decode(&packet1)?;
    let (frame2, decoder) = decoder.decode(&packet2)?;
    let remaining = decoder.finish()?;
}
```

---

## 11. Recommendations for mead

### Priority 1: Core Architecture

1. **Trait-based plugin system**
   - FormatReader, Decoder, Encoder traits
   - Registry for codec/format detection
   - Feature flags for optional codecs

2. **MediaSource abstraction**
   - Composite Read + Seek trait
   - Runtime seekability detection
   - Support files and streams

3. **Multiple buffer types**
   - Frame (video, aligned memory)
   - AudioBuffer (planar, type-safe)
   - RawBuffer (interop, bytes)

4. **Arc<Frame> for sharing**
   - Zero-copy between pipeline stages
   - Thread-safe for parallel processing

### Priority 2: Safety and Efficiency

5. **Location-based references**
   - Store offsets for large data (mdat)
   - Avoid loading entire files
   - Chunk unknown-size boxes

6. **Fallible allocations**
   - TryVec for untrusted sizes
   - Return OutOfMemory errors
   - Prevent panic on malformed input

7. **Strong typing**
   - Enum for container/codec types
   - Type-safe pixel formats
   - No string comparisons

### Priority 3: API Ergonomics

8. **Iterator pattern for decoding**
   - Natural Rust idiom
   - Lazy evaluation
   - Composable with adapters

9. **Send-receive for encoding**
   - Matches hardware APIs
   - Handles lookahead naturally
   - Explicit buffering control

10. **Structured errors**
    - thiserror for library
    - Specific variants for each failure mode
    - ResetRequired for track changes

### Priority 4: Advanced Features (Later)

11. **Typestate pattern for state machines**
    - Consider for Phase 2+
    - Start simple, add if complexity grows
    - Good for advanced safety guarantees

12. **Seeking with keyframe alignment**
    - Return actual seek position
    - Include keyframe indicator
    - Support for gapless playback

### Implementation Order

**Phase 1 (MP4 + AV1)**:
- MediaSource trait
- FormatReader trait + MP4 implementation
- Decoder trait + AV1 implementation (rav1e wrapper)
- Frame/AudioBuffer types
- Basic error handling

**Phase 2 (Expand codecs)**:
- Encoder trait
- Send-receive pattern for encoders
- Registry system for plugins
- Multiple buffer types

**Phase 3 (Advanced features)**:
- Typestate pattern if complexity warrants
- Sophisticated seeking
- Parallel decoding support

---

## Key Takeaways

### What Makes Rust Media APIs Different

1. **Zero-cost abstractions**: Traits compile to direct calls
2. **Ownership prevents copies**: Borrow checker enforces zero-copy
3. **Type safety eliminates runtime checks**: Compile-time format validation
4. **Fearless concurrency**: Arc + Send/Sync for parallel processing

### Critical Patterns

1. **Trait-based extensibility**: Plugin system without dynamic dispatch overhead
2. **Multiple buffer abstractions**: Match use case (internal/API/interop/FFI)
3. **Location-based references**: Avoid loading entire files
4. **Arc for sharing**: Zero-copy between pipeline stages
5. **Typestate for state machines**: Compile-time enforcement of valid operations

### Avoid These Pitfalls

1. **Loading entire files**: Use streaming iterators and location refs
2. **Allocations that can panic**: Use TryVec for untrusted input
3. **Runtime type checks**: Use strong typing and generics
4. **Copying frames**: Use Arc and references
5. **Unsafe without validation**: v_frame's copy_from_raw_parts issue

### Performance Priorities

1. **Memory efficiency**: Constant memory for any file size
2. **Zero-copy**: References and Arc, not clones
3. **SIMD-aligned memory**: Use aligned-vec for video frames
4. **Lazy evaluation**: Iterator pattern defers work
5. **No dynamic dispatch in hot paths**: Monomorphization over trait objects

---

## References

### Primary Sources

- symphonia: https://github.com/pdeljanov/Symphonia
  - docs.rs/symphonia-core
  - GETTING_STARTED.md guide
- mp4parse-rust: https://github.com/mozilla/mp4parse-rust
  - Used in Firefox media pipeline
- rav1e: https://github.com/xiph/rav1e
  - "Using rav1e from your own code" blog post
- v_frame: https://github.com/rust-av/v_frame
  - Originally from rav1e, now standalone

### Supporting Resources

- "Elegant Library APIs in Rust": https://deterministic.space/elegant-apis-in-rust.html
- "Typestate Pattern in Rust": https://cliffle.com/blog/rust-typestate/
- "Type-Driven API Design in Rust": https://willcrichton.net/rust-api-type-patterns/
- Rust API Guidelines: https://rust-lang.github.io/api-guidelines/

### Rust Ecosystem

- nom: Zero-copy parser combinators
- bytes: Efficient byte buffers
- aligned-vec: SIMD-aligned allocations
- thiserror: Error derive macros

---

## Appendix: Library Comparison Matrix

| Feature | symphonia | mp4parse | rav1e | v_frame |
|---------|-----------|----------|-------|---------|
| **Domain** | Audio decode | MP4 parse | AV1 encode | Video frames |
| **Trait-based** | Yes (FormatReader, Decoder) | No (functions) | Partial (Config) | No (structs) |
| **Zero-copy** | AudioBufferRef | Location refs | Arc<Frame> | Aligned vecs |
| **I/O abstraction** | MediaSource | Generic Read | Direct API | N/A |
| **State machine** | Implicit | N/A | Explicit status | N/A |
| **Seeking** | FormatReader::seek | Sample tables | N/A | N/A |
| **Error handling** | Rich Error enum | Status + Error | EncoderStatus | Panics |
| **Buffer types** | 4 types (planar, ref, sample, raw) | N/A | Arc<Frame> | Frame + Plane |
| **Memory safety** | No unsafe (except core) | Careful unsafe | Mostly safe | Some unsafe |
| **FFI support** | No | Yes (C API) | Yes (C API) | No |
| **Production use** | Yes (apps) | Yes (Firefox) | Yes (encoders) | Yes (rav1e) |

---

## Next Steps for mead

1. **Review this document with team/maintainers**
2. **Define core traits** (FormatReader, Decoder, Encoder)
3. **Implement MediaSource** (I/O abstraction)
4. **Design Frame/Buffer types** (aligned, Arc-wrapped)
5. **Create error types** (thiserror-based)
6. **Build MP4 FormatReader** (using mp4parse or custom)
7. **Wrap rav1e in Decoder trait**
8. **Write integration tests**
9. **Document API patterns in rustdoc**
10. **Iterate based on usage feedback**

See ai/STATUS.md and ai/TODO.md for current implementation status.
