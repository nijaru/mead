# CLI and Library UX Design Research

**Date**: 2025-11-06
**Focus**: Modern media tool UX patterns vs FFmpeg pain points
**Goal**: Inform mead CLI/library design with SOTA practices

## Executive Summary

FFmpeg's UX problems stem from:
1. **Flag soup**: 1000+ options, cryptic names, order-dependent behavior
2. **No discoverable presets**: Users must research codec params
3. **Poor defaults**: Requires flags for common tasks
4. **Opaque errors**: Cryptic messages without actionable guidance
5. **Anti-composition**: Doesn't follow Unix pipeline patterns well

Modern Rust CLI best practices (ripgrep, fd, bat):
1. **Sensible defaults**: Works well without flags
2. **Progressive disclosure**: Simple → advanced as needed
3. **Excellent errors**: Clear messages with suggestions
4. **Fast help**: Built-in examples, preset discovery
5. **Machine-readable output**: JSON/structured formats
6. **TTY awareness**: Auto-adapt to pipes vs interactive

**Priority recommendations for mead**:
- HIGH: Preset-based encoding (`--preset web`, `--preset archive`)
- HIGH: Builder pattern library API with type safety
- HIGH: Excellent error messages with suggestions
- MEDIUM: Config file support (TOML)
- MEDIUM: Interactive mode for beginners
- LOW: Watch mode for automatic re-encoding

---

## 1. FFmpeg UX Pain Points

### 1.1 Flag Soup Problem

**Issue**: FFmpeg has 1000+ options with cryptic names and order-dependent behavior.

**Examples from research**:
```bash
# Common encoding task requires research and trial-error
ffmpeg -i input.mp4 \
  -c:v libx264 \
  -preset slow \
  -crf 23 \
  -pix_fmt yuv420p \
  -c:a aac \
  -b:a 128k \
  -movflags +faststart \
  output.mp4

# Users struggle with:
# - What's a good CRF value?
# - What preset should I use?
# - Why do I need movflags?
# - What's pix_fmt and why does it matter?
```

**User complaints** (from HN/Reddit):
- "I have to wade through all the docs, flags, values with trial and error"
- "If it's something you do infrequently, you have to rerun the gauntlet"
- "I needed a tutorial in addition to the built-in help pages"

**Why it happens**:
- FFmpeg exposes low-level codec parameters directly
- No abstraction layer for common use cases
- Documentation assumes expert knowledge
- Flags interact in non-obvious ways

### 1.2 No Discoverable Presets

**Issue**: FFmpeg has presets buried in docs, not discoverable via CLI.

**What users want**:
```bash
# Discoverable presets
ffmpeg list-presets    # Shows available presets
ffmpeg encode input.mp4 --preset web-streaming
ffmpeg encode input.mp4 --preset archive-quality
ffmpeg encode input.mp4 --preset draft-preview
```

**Current state**: Users must:
1. Google "ffmpeg best settings for X"
2. Copy-paste from StackOverflow
3. Hope the answer is current
4. Not understand what the flags do

**HandBrake solution**: Built-in presets with names like:
- "Fast 1080p30"
- "HQ 1080p30 Surround"
- "Production Standard"
- "Production Max"

Users understand intent without knowing codec details.

### 1.3 Poor Defaults

**Issue**: Common tasks require many flags.

**Examples**:
```bash
# Shrinking a file requires research
ffmpeg -i input.mp4 output.mp4  # Doesn't shrink, just re-muxes
ffmpeg -i input.mp4 -crf 28 output.mp4  # Better, but what CRF?

# Simple tasks need flags
ffmpeg -i video.mp4 audio.mp3  # Fails without -vn
ffmpeg -i video.mp4 -vn audio.mp3  # Works
```

**What users expect**:
```bash
mead encode input.mp4  # Smart defaults based on output type
mead encode input.mp4 -o output.webm  # Auto-selects AV1 for .webm
mead extract-audio input.mp4  # Obvious intent
```

### 1.4 Opaque Errors

**Issue**: Cryptic error messages without actionable guidance.

**FFmpeg errors**:
```
[h264 @ 0x7f8b9c000000] error while decoding MB 53 20, bytestream -7
Error while decoding stream #0:0: Invalid data found when processing input
```

**What users need**:
```
Error: Failed to decode H.264 video
  → File may be corrupted at timestamp 00:02:15
  → Try: mead decode --skip-errors input.mp4
  → Or: Use --verbose for detailed codec info
```

**Rust CLI examples** (ripgrep, fd):
```bash
$ rg pattern /nonexistent
/nonexistent: No such file or directory (os error 2)

$ fd pattern /no-access
fd: cannot read directory entry '/no-access': Permission denied
```

Clear, actionable, suggests solutions.

### 1.5 Anti-Composition

**Issue**: FFmpeg doesn't follow Unix pipeline patterns well.

**Problems**:
```bash
# Piping is awkward
ffmpeg -i input.mp4 -f rawvideo - | process | ffmpeg -f rawvideo -i - output.mp4
# Requires explicit format specification both ends

# Progress output pollutes stderr
ffmpeg ... 2>&1 | grep "frame="  # Hacky progress tracking

# No structured output
ffmpeg ... --json  # Doesn't exist
```

**Better patterns** (from modern CLIs):
```bash
# ripgrep: Clean output for piping
rg "pattern" | wc -l  # Just results, no progress

# fd: Structured output options
fd --type f | xargs process

# jq: JSON output for composition
curl api.com | jq '.data' | process
```

---

## 2. Modern CLI Best Practices

### 2.1 Sensible Defaults (ripgrep, fd, bat)

**Pattern**: Work well with zero flags for common cases.

**ripgrep**:
```bash
rg pattern              # Smart defaults:
                        # - Respects .gitignore
                        # - Colors in TTY
                        # - Case-smart search
                        # - Skips hidden files
```

**fd**:
```bash
fd filename             # Smart defaults:
                        # - Respects .gitignore
                        # - Shows colors
                        # - Excludes .git/
                        # - Fast by default
```

**For mead**:
```bash
mead encode input.mp4           # Smart defaults:
                                # - CRF 28 (balanced quality/size)
                                # - Auto-select codec from extension
                                # - Fast preset (good speed/quality)
                                # - Progress bar in TTY

mead encode input.mp4 -o out.av1  # Extension → AV1 codec
mead encode input.mp4 -o out.mp4  # Extension → H.264 (compat)
```

### 2.2 Progressive Disclosure

**Pattern**: Simple by default, expose complexity as needed.

**clap design**:
```bash
$ mead encode --help
Encode video to modern codecs

Usage: mead encode [OPTIONS] <INPUT>

Arguments:
  <INPUT>  Input video file

Options:
  -o, --output <FILE>      Output file [default: out.ivf]
  -p, --preset <PRESET>    Quality preset [default: balanced]
  -h, --help               Print help (see more with '--help')

$ mead encode --help --help  # Extended help
# Shows advanced options:
#   --crf <CRF>             Quality (0-51, lower=better)
#   --speed <SPEED>         Encoding speed (0-10)
#   --threads <N>           Encoder threads
```

**For mead library**:
```rust
// Simple API (95% of users)
let encoder = Encoder::new(Codec::AV1)?;
encoder.encode_file("input.mp4", "output.ivf")?;

// Progressive complexity (5% of users)
let encoder = Encoder::builder()
    .codec(Codec::AV1)
    .crf(28)
    .speed(6)
    .preset(Preset::Balanced)
    .threads(4)
    .build()?;
```

### 2.3 Excellent Error Messages

**Pattern**: Clear, actionable errors with suggestions.

**Rust compiler** (gold standard):
```
error[E0308]: mismatched types
  --> src/main.rs:5:5
   |
5  |     "hello"
   |     ^^^^^^^ expected `i32`, found `&str`
   |
help: try converting the string to an integer
   |
5  |     "hello".parse::<i32>()
   |
```

**clap errors**:
```bash
$ mead encode --invalid
error: unexpected argument '--invalid' found

  tip: a similar argument exists: '--input'

Usage: mead encode [OPTIONS] <INPUT>

For more information, try '--help'.
```

**For mead**:
```bash
$ mead encode input.mp4 --crf 100
Error: CRF value out of range
  → Got: 100
  → Expected: 0-51 (lower = better quality, higher = smaller file)
  → Suggestion: Try --crf 28 for balanced quality/size
  → See: mead encode --help

$ mead encode missing.mp4
Error: Input file not found
  → Path: missing.mp4
  → Did you mean: existing.mp4?
  → See: mead info --list for available files
```

### 2.4 Fast Help and Discovery

**Pattern**: Built-in examples, preset listing, interactive help.

**Modern CLIs**:
```bash
# fzf-style interactive selection
mead preset --interactive
> balanced
  fast
  archive
  streaming-hd
  streaming-4k

# List all presets with details
mead preset --list
balanced         CRF 28, Speed 6   General purpose
fast             CRF 30, Speed 8   Quick encoding
archive          CRF 20, Speed 2   Maximum quality
streaming-hd     2500 kbps, 1080p  Web streaming
streaming-4k     8000 kbps, 2160p  4K streaming

# Show preset details
mead preset show archive
Preset: archive
  Codec: AV1
  CRF: 20
  Speed: 2
  Use case: Long-term archival, maximum quality
  Encoding speed: ~2 fps on typical hardware
  Output size: ~60% of source (lossless)
```

**For mead library**:
```rust
// Discoverable presets via API
let presets = Preset::list();
for preset in presets {
    println!("{}: {}", preset.name(), preset.description());
}

// Preset details
let preset = Preset::Archive;
println!("CRF: {}", preset.crf());
println!("Speed: {}", preset.speed());
```

### 2.5 Machine-Readable Output

**Pattern**: Structured output for scripting (JSON, TSV).

**Modern CLIs**:
```bash
# fd: JSON output
fd --type f --format json
{"type":"file","path":"src/main.rs","size":1234}

# ripgrep: JSON output
rg pattern --json
{"type":"match","data":{"path":{"text":"file.rs"}}}

# hyperfine: Multiple formats
hyperfine --export-json results.json command
```

**For mead**:
```bash
# Progress updates as JSON
mead encode input.mp4 --json
{"type":"progress","frame":100,"fps":25,"eta_sec":120}
{"type":"progress","frame":200,"fps":28,"eta_sec":95}
{"type":"complete","frames":500,"duration_sec":250}

# Info as JSON
mead info input.mp4 --json
{
  "format": "MP4",
  "duration_sec": 120.5,
  "video": {
    "codec": "H.264",
    "resolution": [1920, 1080],
    "fps": 30.0
  },
  "audio": {
    "codec": "AAC",
    "sample_rate": 48000,
    "channels": 2
  }
}
```

**For mead library**:
```rust
// Structured events
encoder.on_progress(|event| {
    let json = serde_json::to_string(&event)?;
    println!("{}", json);
});
```

### 2.6 TTY Awareness

**Pattern**: Auto-adapt to terminal vs pipe.

**Implemented in mead** (Phase 2b):
```rust
pub struct OutputConfig {
    pub is_tty: bool,           // Auto-detect TTY
    pub use_color: bool,        // Respects NO_COLOR
    pub show_progress: bool,    // Auto-disable in pipes
}

impl Default for OutputConfig {
    fn default() -> Self {
        Self {
            is_tty: atty::is(atty::Stream::Stdout),
            use_color: !env::var("NO_COLOR").is_ok(),
            show_progress: atty::is(atty::Stream::Stderr),
        }
    }
}
```

**Behavior**:
```bash
# Interactive terminal: Colors + progress
mead encode input.mp4
[00:02:35] ████████████░ 1234/2000 60fps

# Piped: No colors, no progress, clean output
mead encode input.mp4 | tee log.txt
Frame 1234/2000 encoded

# Explicit control
mead encode input.mp4 --no-color --quiet
```

---

## 3. Media Tool Specific Patterns

### 3.1 HandBrake Preset System

**Key insight**: Named presets for use cases, not technical specs.

**Preset naming**:
```
Production Standard     # Clear intent
Fast 1080p30            # Speed + quality indicator
HQ 1080p30 Surround     # Quality level explicit
Gmail Small 3 Minutes   # Use case specific
```

**NOT**:
```
x264-crf28-preset-slow  # Technical jargon
avc-2500kbps-aac-128k   # Codec details
profile-4.1-8bit        # Spec numbers
```

**For mead**:
```bash
# Use case presets
mead encode input.mp4 --preset web           # Web streaming
mead encode input.mp4 --preset email         # Email attachment
mead encode input.mp4 --preset archive       # Long-term storage
mead encode input.mp4 --preset youtube-hd    # YouTube upload
mead encode input.mp4 --preset social-media  # Instagram/TikTok

# Technical users can still customize
mead encode input.mp4 --preset web --crf 25 --speed 8
```

### 3.2 av1an Chunking Strategy

**Key insight**: Expose parallelization clearly.

**av1an approach**:
```bash
av1an -i input.mp4 \
      --encoder rav1e \
      --workers 8 \          # Explicit parallelism
      --split-method scene \  # Smart chunking
      --target-quality 95     # VMAF target, not CRF
```

**Benefits**:
- Clear performance tuning (--workers)
- Quality-based targeting (VMAF)
- Smart splitting for parallel encode

**For mead**:
```bash
# Simple (auto-detect CPU cores)
mead encode input.mp4 --parallel

# Advanced
mead encode input.mp4 \
    --workers 8 \
    --chunk-method scene \
    --target-vmaf 95
```

### 3.3 Quality Presets (Research-Backed)

**From research** (HandBrake, av1an, x264):

| Preset | CRF | Speed | Use Case | Output Size |
|--------|-----|-------|----------|-------------|
| draft | 35 | 10 | Quick preview | ~20% of source |
| fast | 30 | 8 | Fast turnaround | ~30% of source |
| balanced | 28 | 6 | General purpose | ~40% of source |
| quality | 23 | 4 | High quality | ~60% of source |
| archive | 18 | 2 | Archival/master | ~80% of source |

**Web streaming presets**:
| Preset | Bitrate | Resolution | FPS | Use Case |
|--------|---------|------------|-----|----------|
| web-360p | 500 kbps | 640x360 | 30 | Mobile, low bandwidth |
| web-720p | 1500 kbps | 1280x720 | 30 | HD web |
| web-1080p | 3000 kbps | 1920x1080 | 30 | Full HD |
| web-4k | 10000 kbps | 3840x2160 | 30 | 4K streaming |

### 3.4 Progress Reporting

**Best practices** from modern encoders:

**Good (HandBrake)**:
```
Encoding: task 1 of 1, 45.23 % (28.50 fps, avg 30.12 fps, ETA 00h15m42s)
```

**Better (av1an)**:
```
[00:12:35] ████████████████░░░░░░░░ 1234/2000 frames 60fps 1.2x ⏱ 00:10:22
Bitrate: 2.5 Mbit/s | Size: 45.2 MiB | VMAF: 95.3
```

**For mead** (implemented in Phase 2b):
```rust
pub fn create_progress_bar(total: u64) -> ProgressBar {
    let pb = ProgressBar::new(total);
    pb.set_style(
        ProgressStyle::default_bar()
            .template("[{elapsed_precise}] {bar:40.cyan/blue} {pos}/{len} {msg}")
            .progress_chars("█▓▒░ ")
    );
    pb
}

// Usage
pb.set_message(format!("{:.1} fps | {:.1}x speed", fps, speed_factor));
```

---

## 4. Library API Design Patterns

### 4.1 Builder Pattern (Type-Safe)

**Pattern**: Type-safe builders prevent invalid configurations.

**Bad (stringly-typed)**:
```rust
encoder.set("crf", "28")?;      // Runtime validation
encoder.set("preset", "slwo")?; // Typo causes runtime error
```

**Good (type-safe)**:
```rust
let encoder = Encoder::builder()
    .codec(Codec::AV1)
    .crf(28)                     // Compile-time type check
    .preset(Preset::Balanced)    // Enum, not string
    .pixel_format(PixelFormat::Yuv420p)
    .build()?;                   // Validation at build time
```

**From Rust best practices**:
```rust
pub struct EncoderBuilder {
    codec: Option<Codec>,
    crf: Option<u8>,
    preset: Option<Preset>,
    // ... other fields
}

impl EncoderBuilder {
    pub fn crf(mut self, crf: u8) -> Result<Self> {
        if crf > 51 {
            return Err(Error::InvalidCrf(crf));
        }
        self.crf = Some(crf);
        Ok(self)
    }

    pub fn preset(mut self, preset: Preset) -> Self {
        self.preset = Some(preset);
        self
    }

    pub fn build(self) -> Result<Encoder> {
        let codec = self.codec.ok_or(Error::MissingCodec)?;
        let crf = self.crf.unwrap_or(28); // Smart default
        let preset = self.preset.unwrap_or(Preset::Balanced);

        Ok(Encoder {
            codec,
            crf,
            preset,
        })
    }
}
```

### 4.2 Preset-Based API

**Pattern**: Presets as first-class API objects.

```rust
// Preset enum
#[derive(Debug, Clone, Copy)]
pub enum Preset {
    Draft,
    Fast,
    Balanced,
    Quality,
    Archive,
    // Web streaming presets
    Web360p,
    Web720p,
    Web1080p,
    Web4k,
}

impl Preset {
    pub fn crf(&self) -> u8 {
        match self {
            Preset::Draft => 35,
            Preset::Fast => 30,
            Preset::Balanced => 28,
            Preset::Quality => 23,
            Preset::Archive => 18,
            // Bitrate-based presets return default CRF
            _ => 28,
        }
    }

    pub fn speed(&self) -> u8 {
        match self {
            Preset::Draft => 10,
            Preset::Fast => 8,
            Preset::Balanced => 6,
            Preset::Quality => 4,
            Preset::Archive => 2,
            _ => 6,
        }
    }

    pub fn description(&self) -> &'static str {
        match self {
            Preset::Balanced => "General purpose encoding",
            Preset::Archive => "Maximum quality for archival",
            Preset::Web1080p => "1080p web streaming (3 Mbps)",
            // ...
        }
    }
}

// Usage
let encoder = Encoder::from_preset(Preset::Balanced)?;

// Preset with overrides
let encoder = Encoder::builder()
    .preset(Preset::Balanced)
    .crf(25)  // Override preset CRF
    .build()?;
```

### 4.3 Callback-Based Progress

**Pattern**: Event callbacks for progress/metrics.

```rust
pub trait ProgressCallback: Send {
    fn on_frame(&mut self, frame: u64, total: u64, fps: f32);
    fn on_complete(&mut self, summary: EncodingSummary);
    fn on_error(&mut self, error: &Error);
}

// Simple function callback
encoder.on_progress(|frame, total, fps| {
    println!("Frame {}/{} @ {:.1} fps", frame, total, fps);
});

// Structured event callback
encoder.on_event(|event| match event {
    EncoderEvent::Frame { num, total, fps } => { /* update UI */ },
    EncoderEvent::Complete { summary } => { /* show results */ },
    EncoderEvent::Error { error } => { /* handle error */ },
});
```

### 4.4 Error Handling (anyhow vs thiserror)

**Pattern**: Library uses thiserror, app uses anyhow.

**Library (mead-core)**:
```rust
use thiserror::Error;

#[derive(Error, Debug)]
pub enum Error {
    #[error("Invalid CRF value: {0} (expected 0-51)")]
    InvalidCrf(u8),

    #[error("Codec {codec:?} not supported for container {container:?}")]
    UnsupportedCodec {
        codec: Codec,
        container: Container,
    },

    #[error("Failed to encode frame {frame}")]
    EncodeFailed {
        frame: u64,
        #[source]
        source: Box<dyn std::error::Error + Send + Sync>,
    },
}
```

**App (mead CLI)**:
```rust
use anyhow::{Context, Result};

fn encode_file(input: &Path, output: &Path) -> Result<()> {
    let encoder = Encoder::new(Codec::AV1)
        .context("Failed to create encoder")?;

    encoder.encode_file(input, output)
        .with_context(|| format!("Failed to encode {:?}", input))?;

    Ok(())
}
```

### 4.5 Resource Management (RAII)

**Pattern**: Automatic resource cleanup, no manual close().

```rust
// Good: RAII pattern
{
    let mut encoder = Encoder::new(Codec::AV1)?;
    encoder.send_frame(Some(frame))?;
    encoder.send_frame(None)?; // Flush

    while let Some(packet) = encoder.receive_packet()? {
        output.write(packet)?;
    }
} // encoder dropped, resources freed

// Bad: Manual resource management
let mut encoder = Encoder::new(Codec::AV1)?;
// ... encoding ...
encoder.close()?; // Easy to forget, resource leak if error before close
```

**Implementation**:
```rust
pub struct Encoder {
    context: *mut rav1e::Context,
}

impl Drop for Encoder {
    fn drop(&mut self) {
        // Automatic cleanup
        unsafe { rav1e::context_free(self.context) };
    }
}
```

---

## 5. Concrete Recommendations for Mead

### 5.1 HIGH Priority

#### Preset-Based Encoding

**CLI**:
```bash
# Simple presets
mead encode input.mp4 --preset balanced
mead encode input.mp4 --preset archive
mead encode input.mp4 --preset web-1080p

# Discovery
mead preset list
mead preset show balanced

# Preset with overrides
mead encode input.mp4 --preset balanced --crf 25
```

**Library**:
```rust
// Simple
let encoder = Encoder::from_preset(Preset::Balanced)?;

// Builder with preset
let encoder = Encoder::builder()
    .preset(Preset::Balanced)
    .crf(25)  // Override
    .build()?;

// Preset API
let presets = Preset::all();
let info = Preset::Balanced.info();
```

**Implementation**:
```rust
// cli/src/commands/preset.rs
pub fn list_presets() {
    for preset in Preset::all() {
        println!("{:15} {}", preset.name(), preset.description());
    }
}

pub fn show_preset(preset: Preset) {
    println!("Preset: {}", preset.name());
    println!("  Description: {}", preset.description());
    println!("  CRF: {}", preset.crf());
    println!("  Speed: {}", preset.speed());
    println!("  Use case: {}", preset.use_case());
}
```

#### Builder Pattern Library API

**Type-safe configuration**:
```rust
let encoder = Encoder::builder()
    .codec(Codec::AV1)
    .crf(28)                    // u8, validated
    .preset(Preset::Balanced)   // Enum, not string
    .pixel_format(PixelFormat::Yuv420p)
    .threads(4)
    .build()?;                  // Validation here

// Invalid at compile time:
encoder.crf("28");              // ❌ Wrong type
encoder.preset("balanced");     // ❌ String not enum
```

**Implementation**:
```rust
// mead-core/src/codec/encoder.rs
pub struct EncoderBuilder {
    codec: Option<Codec>,
    crf: Option<u8>,
    preset: Option<Preset>,
    pixel_format: Option<PixelFormat>,
    threads: Option<usize>,
}

impl EncoderBuilder {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn codec(mut self, codec: Codec) -> Self {
        self.codec = Some(codec);
        self
    }

    pub fn crf(mut self, crf: u8) -> Result<Self> {
        if crf > 51 {
            return Err(Error::InvalidCrf(crf));
        }
        self.crf = Some(crf);
        Ok(self)
    }

    pub fn preset(mut self, preset: Preset) -> Self {
        self.preset = Some(preset);
        self
    }

    pub fn build(self) -> Result<Encoder> {
        let codec = self.codec.ok_or(Error::MissingCodec)?;

        // Apply preset defaults
        let preset = self.preset.unwrap_or(Preset::Balanced);
        let crf = self.crf.unwrap_or_else(|| preset.crf());
        let speed = preset.speed();

        Ok(Encoder::new_with_config(EncoderConfig {
            codec,
            crf,
            speed,
            pixel_format: self.pixel_format.unwrap_or(PixelFormat::Yuv420p),
            threads: self.threads.unwrap_or_else(num_cpus::get),
        }))
    }
}
```

#### Excellent Error Messages

**With suggestions and context**:
```rust
// mead-core/src/error.rs
use thiserror::Error;

#[derive(Error, Debug)]
pub enum Error {
    #[error("CRF value out of range: {value} (expected 0-51)")]
    InvalidCrf {
        value: u8,
    },

    #[error("Unsupported codec {codec:?} for container {container:?}")]
    UnsupportedCodec {
        codec: Codec,
        container: Container,
        suggestion: String,
    },

    #[error("Failed to open input file: {path}")]
    InputNotFound {
        path: PathBuf,
        similar: Vec<PathBuf>,  // Did you mean?
    },
}

impl Error {
    pub fn suggestion(&self) -> Option<&str> {
        match self {
            Error::InvalidCrf { value } if *value > 51 => {
                Some("Try --crf 28 for balanced quality/size")
            }
            Error::UnsupportedCodec { suggestion, .. } => {
                Some(suggestion)
            }
            _ => None,
        }
    }
}
```

**CLI error display**:
```rust
// mead/src/main.rs
fn main() {
    if let Err(e) = run() {
        eprintln!("{}", format_error(&e));
        std::process::exit(1);
    }
}

fn format_error(error: &anyhow::Error) -> String {
    use console::style;

    let mut output = String::new();
    output.push_str(&format!("{}: {}\n",
        style("Error").red().bold(),
        error
    ));

    // Add context chain
    for cause in error.chain().skip(1) {
        output.push_str(&format!("  {} {}\n",
            style("→").yellow(),
            cause
        ));
    }

    // Add suggestion if available
    if let Some(mead_err) = error.downcast_ref::<mead_core::Error>() {
        if let Some(suggestion) = mead_err.suggestion() {
            output.push_str(&format!("  {} {}\n",
                style("Suggestion:").cyan().bold(),
                suggestion
            ));
        }
    }

    output
}
```

### 5.2 MEDIUM Priority

#### Config File Support

**TOML configuration**:
```toml
# ~/.config/mead/config.toml or ./mead.toml

[defaults]
preset = "balanced"
threads = 8

[presets.custom-web]
codec = "av1"
crf = 30
speed = 8
pixel-format = "yuv420p"

[presets.custom-archive]
codec = "av1"
crf = 18
speed = 2
pixel-format = "yuv420p"
```

**CLI integration**:
```bash
# Uses config file
mead encode input.mp4

# Override config
mead encode input.mp4 --preset archive

# Specify config
mead encode input.mp4 --config ./custom.toml

# Use custom preset from config
mead encode input.mp4 --preset custom-web
```

**Implementation**:
```rust
// mead/src/config.rs
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Debug, Deserialize, Serialize)]
pub struct Config {
    #[serde(default)]
    pub defaults: Defaults,

    #[serde(default)]
    pub presets: HashMap<String, CustomPreset>,
}

#[derive(Debug, Deserialize, Serialize, Default)]
pub struct Defaults {
    pub preset: Option<String>,
    pub threads: Option<usize>,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct CustomPreset {
    pub codec: String,
    pub crf: u8,
    pub speed: u8,
    pub pixel_format: String,
}

impl Config {
    pub fn load() -> Result<Self> {
        // Try in order: ./mead.toml, ~/.config/mead/config.toml
        let paths = vec![
            PathBuf::from("./mead.toml"),
            dirs::config_dir()?.join("mead/config.toml"),
        ];

        for path in paths {
            if path.exists() {
                let content = fs::read_to_string(&path)?;
                return toml::from_str(&content)
                    .with_context(|| format!("Failed to parse {:?}", path));
            }
        }

        Ok(Config::default())
    }
}
```

#### Interactive Mode for Beginners

**Wizard-style workflow**:
```bash
$ mead encode --interactive
Welcome to mead encoder!

Input file: input.mp4
  ✓ Detected: H.264 1920x1080 30fps, AAC stereo

What would you like to do?
  > Compress for web streaming
    Maximum quality archival
    Quick draft preview
    Email attachment (<25MB)
    Custom settings

Selected: Compress for web streaming

Target resolution?
  > Keep original (1080p)
    Downscale to 720p
    Downscale to 480p

Target quality?
    Fast encoding, larger file
  > Balanced quality/size
    High quality, slower encoding

Output file: output.webm

Summary:
  Input:  input.mp4 (1920x1080, H.264)
  Output: output.webm (1920x1080, AV1)
  Preset: web-1080p (CRF 28, Speed 6)

Start encoding? [Y/n] y

[00:02:35] ████████████░ 1234/2000 60fps
✓ Encoded successfully in 2m 35s
```

**Implementation** (using dialoguer):
```rust
// mead/src/interactive.rs
use dialoguer::{Select, Confirm, Input};

pub fn interactive_encode() -> Result<EncodeOptions> {
    println!("Welcome to mead encoder!");

    // Get input file
    let input: String = Input::new()
        .with_prompt("Input file")
        .interact_text()?;

    // Detect input properties
    let info = mead_core::info::probe(&input)?;
    println!("  ✓ Detected: {} {}x{} {:.0}fps",
        info.video_codec,
        info.width,
        info.height,
        info.fps
    );

    // Use case selection
    let use_cases = vec![
        "Compress for web streaming",
        "Maximum quality archival",
        "Quick draft preview",
        "Email attachment (<25MB)",
        "Custom settings",
    ];

    let selection = Select::new()
        .with_prompt("What would you like to do?")
        .items(&use_cases)
        .default(0)
        .interact()?;

    let preset = match selection {
        0 => Preset::Web1080p,
        1 => Preset::Archive,
        2 => Preset::Draft,
        3 => Preset::Fast,
        4 => return custom_settings(),
        _ => unreachable!(),
    };

    // Output file
    let output: String = Input::new()
        .with_prompt("Output file")
        .default("output.webm".into())
        .interact_text()?;

    // Confirm
    show_summary(&input, &output, &preset);
    let confirmed = Confirm::new()
        .with_prompt("Start encoding?")
        .default(true)
        .interact()?;

    if !confirmed {
        return Err(anyhow!("Cancelled by user"));
    }

    Ok(EncodeOptions {
        input: input.into(),
        output: output.into(),
        preset,
    })
}
```

### 5.3 LOW Priority

#### Watch Mode

**Auto re-encode on file changes**:
```bash
# Watch directory, auto-encode on changes
mead watch input/ --output output/ --preset web-1080p

# Detected change: input/video1.mp4
# Encoding to: output/video1.webm
# [progress bar]
# ✓ Encoded successfully

# Detected change: input/video2.mp4
# ...
```

**Implementation** (using notify):
```rust
// mead/src/commands/watch.rs
use notify::{Watcher, RecursiveMode, watcher};
use std::sync::mpsc::channel;
use std::time::Duration;

pub fn watch(input_dir: &Path, output_dir: &Path, preset: Preset) -> Result<()> {
    let (tx, rx) = channel();
    let mut watcher = watcher(tx, Duration::from_secs(2))?;

    watcher.watch(input_dir, RecursiveMode::Recursive)?;

    println!("Watching {:?} for changes...", input_dir);

    loop {
        match rx.recv() {
            Ok(event) => {
                if let Some(path) = event.path {
                    if is_video_file(&path) {
                        println!("Detected change: {:?}", path);

                        let output = output_dir.join(
                            path.file_stem().unwrap()
                        ).with_extension("webm");

                        println!("Encoding to: {:?}", output);

                        if let Err(e) = encode_file(&path, &output, preset) {
                            eprintln!("Error: {}", e);
                        } else {
                            println!("✓ Encoded successfully");
                        }
                    }
                }
            }
            Err(e) => eprintln!("Watch error: {}", e),
        }
    }
}
```

#### Composition/Piping Support

**Unix-style composition**:
```bash
# Decode to raw frames, process, re-encode
mead decode input.mp4 --format yuv4mpeg | \
    process-frames | \
    mead encode --format yuv4mpeg - -o output.webm

# Extract audio, process, re-mux
mead extract-audio input.mp4 --format wav - | \
    audio-filter | \
    mead mux video.webm - -o final.webm
```

**Implementation**:
```rust
// mead/src/commands/decode.rs
pub fn decode_to_stdout(input: &Path, format: OutputFormat) -> Result<()> {
    let demuxer = Mp4Demuxer::open(input)?;
    let mut stdout = std::io::stdout();

    match format {
        OutputFormat::Yuv4mpeg => {
            // Write Y4M header
            writeln!(stdout, "YUV4MPEG2 W{} H{} F{}:1 Ip C420",
                width, height, fps)?;

            // Stream frames
            for frame in demuxer.frames() {
                write!(stdout, "FRAME\n")?;
                stdout.write_all(&frame.data)?;
            }
        }
        OutputFormat::Raw => {
            // Raw frame data, no headers
            for frame in demuxer.frames() {
                stdout.write_all(&frame.data)?;
            }
        }
    }

    Ok(())
}
```

---

## 6. Implementation Roadmap

### Phase 1: Core UX Improvements (2-3 days)
- ✅ **Already done**: TTY awareness, colors, progress bars (Phase 2b)
- **Preset system**: Enum, descriptions, CLI integration
- **Builder pattern**: Type-safe encoder configuration
- **Error improvements**: Better messages with suggestions

### Phase 2: Discoverability (1-2 days)
- `mead preset list` command
- `mead preset show <name>` command
- Extended `--help` with examples
- Config file support (TOML)

### Phase 3: Advanced Features (2-3 days)
- Interactive mode (`--interactive` flag)
- Watch mode (`mead watch`)
- Composition support (stdin/stdout piping)
- JSON output for scripting

### Phase 4: Polish (1 day)
- Shell completion (bash, zsh, fish)
- Man page generation
- Example gallery
- Video tutorials

---

## 7. Success Metrics

### Compared to FFmpeg

| Metric | FFmpeg | mead (target) |
|--------|--------|---------------|
| **Time to first encode** | 15-30 min (research flags) | 30 seconds (preset) |
| **Flags for common task** | 8-12 flags | 1 flag (--preset) |
| **Error clarity** | Cryptic codec errors | Clear with suggestions |
| **Discovery** | Google/StackOverflow | Built-in (`preset list`) |
| **Composition** | Awkward, manual formats | Natural Unix pipes |

### User Experience Goals

1. **Beginner-friendly**: First encode in <1 minute without docs
2. **Expert-friendly**: Full control via builder API
3. **Scriptable**: JSON output, exit codes, quiet mode
4. **Discoverable**: Presets, examples, interactive mode
5. **Professional**: Progress bars, ETAs, clean output

---

## 8. Code Examples: Before/After

### Example 1: Simple Encode

**FFmpeg (current)**:
```bash
# User must know:
# - Which codec (libx264 vs x264 vs libx265 vs libsvtav1)
# - CRF values (18? 23? 28?)
# - Presets (fast? slow? medium?)
# - Pixel format (why yuv420p?)
# - Container flags (movflags?)

ffmpeg -i input.mp4 \
  -c:v libx264 \
  -preset slow \
  -crf 23 \
  -pix_fmt yuv420p \
  -c:a aac \
  -b:a 128k \
  -movflags +faststart \
  output.mp4
```

**mead (proposed)**:
```bash
# Smart defaults based on use case
mead encode input.mp4 --preset balanced

# Or even simpler (auto-detects intent from extension)
mead encode input.mp4 -o output.webm
```

### Example 2: Web Streaming

**FFmpeg**:
```bash
# Multiple passes, complex flags
ffmpeg -i input.mp4 -c:v libsvtav1 -preset 8 -crf 35 -b:v 2500k \
  -maxrate 3000k -bufsize 6000k -c:a libopus -b:a 128k \
  -vf scale=-2:1080 -movflags +faststart output.webm
```

**mead**:
```bash
mead encode input.mp4 --preset web-1080p
```

### Example 3: Library API

**FFmpeg C API** (complex, unsafe):
```c
AVCodecContext *ctx = avcodec_alloc_context3(codec);
ctx->bit_rate = 400000;
ctx->width = 1920;
ctx->height = 1080;
ctx->time_base = (AVRational){1, 30};
ctx->framerate = (AVRational){30, 1};
ctx->gop_size = 10;
ctx->max_b_frames = 1;
ctx->pix_fmt = AV_PIX_FMT_YUV420P;

if (avcodec_open2(ctx, codec, NULL) < 0) {
    // Error handling
}

// Don't forget to free!
avcodec_free_context(&ctx);
```

**mead** (safe, builder pattern):
```rust
let encoder = Encoder::builder()
    .preset(Preset::Balanced)
    .resolution(1920, 1080)
    .fps(30.0)
    .build()?;

// RAII: automatic cleanup on drop
```

---

## 9. Research Sources Summary

### FFmpeg UX Issues
- HackerNews threads on FFmpeg complexity
- Reddit posts about learning curves
- StackOverflow "ffmpeg how to" questions
- Forum posts about trial-and-error workflows

### Modern CLI Best Practices
- ripgrep: Sensible defaults, TTY awareness, fast help
- fd: Simple syntax, color output, .gitignore integration
- bat: Syntax highlighting, paging, git integration
- exa: Better defaults than ls, tree view
- clap: Derive macros, auto-help, completion

### Media Tool Patterns
- HandBrake: Named presets for use cases
- av1an: Chunking, parallelization, VMAF targeting
- x264: Quality presets (ultrafast→placebo)
- SVT-AV1: Speed presets with clear tradeoffs

### Rust Library Design
- Builder pattern (derive_builder, bon)
- Type-safe APIs (no stringly-typed)
- RAII resource management
- thiserror for libraries, anyhow for apps
- Progress callbacks and events

---

## 10. Next Steps

1. **Implement preset system** (highest ROI)
   - Create Preset enum with standard presets
   - Add `mead preset list` and `mead preset show` commands
   - Integrate presets into encode command

2. **Improve error messages**
   - Add suggestions to Error enum
   - Format errors with colors and context
   - Include "did you mean?" for typos

3. **Builder pattern for library**
   - Create EncoderBuilder with type safety
   - Validate at build time, not runtime
   - Good error messages for invalid configs

4. **Config file support**
   - TOML format for config
   - Support custom presets
   - Merge CLI args with config

5. **Interactive mode** (lower priority)
   - Use dialoguer for prompts
   - Wizard-style workflow
   - Good defaults based on detection

---

## Conclusion

**Key insight**: FFmpeg's UX problems come from exposing low-level codec details without abstraction. Modern CLIs succeed by:
1. Smart defaults that "just work"
2. Progressive disclosure (simple → advanced)
3. Excellent error messages
4. Built-in discovery (help, examples, presets)

**For mead**: Preset-based encoding is the highest-value improvement. Users should rarely need to know what CRF means - they should pick "balanced", "archive", or "web-streaming" and get good results.

**Implementation priority**:
1. HIGH: Presets (biggest UX win)
2. HIGH: Builder pattern (library API quality)
3. HIGH: Better errors (debugging experience)
4. MEDIUM: Config files (power users)
5. MEDIUM: Interactive mode (beginners)
6. LOW: Watch mode (niche use case)

The research shows a clear path: make mead feel like ripgrep/fd (modern, fast, intuitive) not FFmpeg (powerful but complex).
