# CLI UX Best Practices Research

## Context
Evaluating current CLI implementation against modern Rust best practices and FFmpeg's UX patterns.

## Current State (mead 0.0.0)

**Dependencies:**
- clap 4.5 (derive macros) ✅
- tracing + tracing-subscriber ✅

**Missing:**
- Progress bars / spinners ❌
- Colored output ❌
- Human-readable formatting ❌
- TTY detection ❌
- Scripting flags (--quiet, --json) ❌

**Output style:**
```
File: video.mp4
Format: mp4
Streams: 2
Duration: 0:01:23.456
```

## FFmpeg's UX (Target)

FFmpeg excels at real-time progress feedback:

```
frame= 1234 fps= 60 q=28.0 size=   45056kB time=00:00:20.56 bitrate=17977.6kbits/s speed=1.2x
```

**Key features:**
- Real-time frame count, fps, encoding speed
- Time elapsed and current position
- Bitrate and output file size
- Speed relative to realtime (1.2x = 20% faster than playback)
- Single line that updates in place (not spamming terminal)

## Modern Rust CLI Best Practices

### 1. Progress Bars - `indicatif` 0.17

Industry standard for Rust CLI progress bars.

**Features:**
- Progress bars with customizable templates
- Spinners for indeterminate operations
- Multi-progress (multiple concurrent bars)
- Automatic TTY detection (hides in pipes)
- Human formatting: HumanBytes, HumanDuration, HumanCount
- Thread-safe (Send + Sync)

**Example:**
```rust
use indicatif::{ProgressBar, ProgressStyle};

let pb = ProgressBar::new(total_frames);
pb.set_style(
    ProgressStyle::with_template(
        "[{elapsed_precise}] {bar:40.cyan/blue} {pos}/{len} {msg} {per_sec} ETA: {eta}"
    )?
    .progress_chars("##-")
);

for frame in frames {
    pb.inc(1);
    pb.set_message(format!("{}fps {:.1}x", fps, speed));
}

pb.finish_with_message("Encode complete");
```

**Template variables:**
- `{bar}` - progress bar (20 chars default)
- `{wide_bar}` - fills remaining space
- `{spinner}` - animated spinner
- `{pos}` / `{len}` - current/total
- `{percent}` / `{percent_precise}` - percentage
- `{elapsed}` / `{elapsed_precise}` - time elapsed (42s vs HH:MM:SS)
- `{eta}` / `{eta_precise}` - estimated time remaining
- `{per_sec}` - items per second
- `{bytes}` / `{bytes_per_sec}` - byte counts and speeds
- `{msg}` - custom message

**Best practices:**
- Progress → stderr (not stdout, allows piping)
- Auto-hides when not TTY (piped to file)
- Use `finish_with_message()` to keep final state
- `enable_steady_tick()` for spinners

### 2. Colors - `console` 0.15

Cross-platform terminal colors and features.

**Features:**
- ANSI colors (8-bit, 24-bit)
- TTY detection
- Terminal width detection
- NO_COLOR environment variable support
- Windows compatibility
- Style builder pattern

**Example:**
```rust
use console::{style, Term};

// Simple styling
println!("{}", style("Success").green().bold());
println!("{}", style("Error").red());
println!("{}", style("Warning").yellow());

// TTY detection
if Term::stdout().is_term() {
    // Show colors
} else {
    // Plain output
}

// Respect NO_COLOR
if std::env::var("NO_COLOR").is_ok() {
    // Disable colors
}
```

**Color guidelines:**
- Success: green
- Errors: red
- Warnings: yellow
- Info: blue/cyan
- Highlights: bold
- Auto-disable in pipes or with NO_COLOR

### 3. Integration - `tracing-indicatif`

Combines tracing logs with indicatif progress bars.

**Problem:** tracing logs interfere with progress bars (overwrite/flicker)
**Solution:** tracing-indicatif layer that coordinates with progress bars

```rust
use tracing_indicatif::IndicatifLayer;
use tracing_subscriber::layer::SubscriberExt;

let indicatif_layer = IndicatifLayer::new();

tracing_subscriber::registry()
    .with(
        tracing_subscriber::fmt::layer()
            .with_writer(indicatif_layer.get_stderr_writer())
    )
    .with(indicatif_layer)
    .init();
```

**Benefits:**
- Logs don't corrupt progress bars
- Progress bars pause during log output
- Logs go above progress bar

### 4. Human Formatting

indicatif provides formatters:

```rust
use indicatif::{HumanBytes, HumanDuration, HumanCount};
use std::time::Duration;

HumanBytes(3 * 1024 * 1024)  // "3.00 MiB"
HumanDuration(Duration::from_secs(150))  // "2m 30s"
HumanCount(33857009)  // "33,857,009"
```

### 5. Output Separation

**Best practice:**
- stdout: Data/results only (allows piping)
- stderr: Progress bars, logs, errors

```rust
// Good
println!("{{\"result\": \"data\"}}");  // stdout
eprintln!("Processing...");  // stderr

// Bad - mixes data with logs
println!("Processing...");
println!("{{\"result\": \"data\"}}");
```

### 6. Scripting Support

**Required flags:**
- `--quiet` / `-q`: Suppress progress, only show errors
- `--json`: Machine-readable JSON output
- `--no-color`: Explicit color disable

**Example:**
```bash
# Interactive (shows progress bar)
mead encode input.mp4 -o output.webm

# Piped (auto-disables progress)
mead encode input.mp4 -o output.webm | tee log.txt

# Quiet mode (errors only)
mead encode input.mp4 -o output.webm --quiet

# JSON output (for scripts)
mead info video.mp4 --json
```

## Implementation Plan for mead

### Phase 2b Scope

**1. Add dependencies:**
```toml
[dependencies]
indicatif = { version = "0.17", features = ["rayon"] }
console = "0.15"
```

**2. Create output module:**
```
mead/src/
├── main.rs
├── output.rs     # New: progress bars, colors, formatting
└── commands/     # New: move command logic here
    ├── info.rs
    ├── encode.rs
    └── decode.rs
```

**3. Implement progress for encode:**
```rust
let pb = ProgressBar::new(total_frames);
pb.set_style(ProgressStyle::with_template(
    "[{elapsed_precise}] {bar:40.cyan/blue} {pos}/{len} {msg} ⏱ {eta}"
)?);

while encoding {
    pb.inc(1);
    pb.set_message(format!("{}fps {:.1}x {}kbps",
        fps, speed, bitrate));
}

pb.finish_with_message(style("✓ Encoded successfully").green());
```

**4. Add TTY detection:**
```rust
fn show_progress() -> bool {
    Term::stderr().is_term()
        && !cli.quiet
        && std::env::var("NO_COLOR").is_err()
}
```

**5. Implement --json flag:**
```rust
if cli.json {
    let metadata = serde_json::json!({
        "format": "mp4",
        "duration_ms": 123456,
        "tracks": [...]
    });
    println!("{}", serde_json::to_string_pretty(&metadata)?);
}
```

## Comparison: mead vs FFmpeg

### FFmpeg progress output:
```
frame= 1234 fps= 60 q=28.0 size=45056kB time=00:00:20.56 bitrate=17977kbits/s speed=1.2x
```

### Proposed mead output:
```
[00:02:35] ████████████████████░░░░░░░░ 1234/2000 60fps 1.2x ⏱ 00:01:05
Bitrate: 2.5 Mbit/s | Size: 45.2 MiB

✓ Encoded successfully in 2m 35s
```

**Advantages of mead's approach:**
- More visually appealing (progress bar)
- Clearer structure (not cramped)
- Modern terminal features (colors, emojis optional)
- Human-readable by default (45.2 MiB vs 45056kB)

**FFmpeg advantages:**
- Single line (less screen space)
- More metrics in view simultaneously
- Parseable by existing tools

**Solution:** Support both with `--style compact` flag

## References

**Crates:**
- indicatif: https://docs.rs/indicatif/
- console: https://docs.rs/console/
- tracing-indicatif: https://docs.rs/tracing-indicatif/

**Examples in the wild:**
- ripgrep: Progress for large searches
- cargo: Build progress bars
- tokei: Code counting with progress

**Articles:**
- CLI UX best practices: https://evilmartians.com/chronicles/cli-ux-best-practices-3-patterns-for-improving-progress-displays
- Rust CLI Book: https://rust-cli.github.io/book/

## Success Criteria

**Must have:**
- [ ] Progress bars during encode/decode (with TTY detection)
- [ ] Colored output (auto-disabled in pipes)
- [ ] Human-readable formatting (bytes, durations)
- [ ] Real-time metrics (fps, speed, ETA)
- [ ] --quiet flag (errors only)
- [ ] --json flag (machine-readable)
- [ ] NO_COLOR support

**Nice to have:**
- [ ] --style compact (FFmpeg-like single line)
- [ ] MultiProgress (parallel encodes)
- [ ] Spinner during file opening
- [ ] --verbose levels (info, debug, trace)

## Timeline

Estimated effort: 1-2 days
- Day 1: Add dependencies, refactor output module, basic progress bars
- Day 2: Colors, TTY detection, --quiet/--json flags, testing
