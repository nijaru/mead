//! CLI output handling: progress bars, colors, formatting
//!
//! Provides utilities for production-quality CLI UX:
//! - Progress bars with indicatif
//! - Colored output with TTY detection
//! - Human-readable formatting
//! - Respects NO_COLOR and --no-color

use console::{style, Term};
use indicatif::{HumanBytes, HumanDuration, ProgressBar, ProgressStyle};
use std::time::Duration;

/// Configuration for CLI output behavior
#[derive(Debug, Clone)]
pub struct OutputConfig {
    pub quiet: bool,
    pub json: bool,
    pub no_color: bool,
    pub is_tty: bool,
}

impl OutputConfig {
    /// Create output config from CLI flags and environment
    pub fn new(quiet: bool, json: bool, no_color: bool) -> Self {
        let is_tty = Term::stderr().is_term();

        // Check NO_COLOR environment variable
        let no_color = no_color || std::env::var("NO_COLOR").is_ok();

        Self {
            quiet,
            json,
            no_color,
            is_tty,
        }
    }

    /// Should we show progress bars?
    pub fn show_progress(&self) -> bool {
        !self.quiet && !self.json && self.is_tty
    }

    /// Should we use colors?
    pub fn use_colors(&self) -> bool {
        !self.no_color && self.is_tty
    }
}

/// Theme for consistent colored output
pub struct Theme {
    use_colors: bool,
}

impl Theme {
    pub fn new(use_colors: bool) -> Self {
        Self { use_colors }
    }

    pub fn success(&self, text: &str) -> String {
        if self.use_colors {
            format!("{} {}", style("✓").green().bold(), style(text).green())
        } else {
            format!("✓ {}", text)
        }
    }

    pub fn error(&self, text: &str) -> String {
        if self.use_colors {
            format!("{} {}", style("✗").red().bold(), style(text).red())
        } else {
            format!("✗ {}", text)
        }
    }

    #[allow(dead_code)]
    pub fn warning(&self, text: &str) -> String {
        if self.use_colors {
            format!("{} {}", style("⚠").yellow().bold(), style(text).yellow())
        } else {
            format!("⚠ {}", text)
        }
    }

    #[allow(dead_code)]
    pub fn info(&self, text: &str) -> String {
        if self.use_colors {
            format!("{} {}", style("ℹ").blue().bold(), style(text).blue())
        } else {
            format!("ℹ {}", text)
        }
    }

    pub fn highlight(&self, text: &str) -> String {
        if self.use_colors {
            style(text).bold().to_string()
        } else {
            text.to_string()
        }
    }
}

/// Create a progress bar for encoding/decoding operations
pub fn create_progress_bar(total: u64, operation: &str) -> ProgressBar {
    let pb = ProgressBar::new(total);

    // Custom style matching FFmpeg's output but prettier
    let style = ProgressStyle::with_template(
        "[{elapsed_precise}] {bar:40.cyan/blue} {pos}/{len} {msg}"
    )
    .unwrap()
    .progress_chars("█▓░");

    pb.set_style(style);
    pb.set_message(format!("{} frames", operation));

    pb
}

/// Create a spinner for indeterminate operations
#[allow(dead_code)]
pub fn create_spinner(message: &str) -> ProgressBar {
    let pb = ProgressBar::new_spinner();
    pb.set_message(message.to_string());
    pb.enable_steady_tick(Duration::from_millis(100));
    pb
}

/// Format bytes in human-readable format
#[allow(dead_code)]
pub fn format_bytes(bytes: u64) -> String {
    HumanBytes(bytes).to_string()
}

/// Format duration in human-readable format
pub fn format_duration(duration: Duration) -> String {
    HumanDuration(duration).to_string()
}

/// Calculate and format encoding speed (frames per second)
#[allow(dead_code)]
pub fn format_speed(frames: u64, elapsed: Duration) -> String {
    if elapsed.as_secs() == 0 {
        return "-- fps".to_string();
    }

    let fps = frames as f64 / elapsed.as_secs_f64();
    format!("{:.1} fps", fps)
}

/// Calculate speed relative to realtime
/// Assumes 30 fps as baseline for video
#[allow(dead_code)]
pub fn format_realtime_speed(frames: u64, elapsed: Duration, fps: f64) -> String {
    if elapsed.as_secs() == 0 {
        return "-- x".to_string();
    }

    let actual_frames = frames as f64;
    let expected_frames = elapsed.as_secs_f64() * fps;
    let speed = actual_frames / expected_frames;

    format!("{:.1}x", speed)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_output_config() {
        let config = OutputConfig::new(false, false, false);
        assert_eq!(config.quiet, false);
        assert_eq!(config.json, false);
    }

    #[test]
    fn test_format_bytes() {
        assert!(format_bytes(1024).contains("KiB") || format_bytes(1024).contains("KB"));
        assert!(format_bytes(1024 * 1024).contains("MiB") || format_bytes(1024 * 1024).contains("MB"));
    }

    #[test]
    fn test_format_speed() {
        let duration = Duration::from_secs(2);
        let speed = format_speed(60, duration);
        assert!(speed.contains("30"));
    }

    #[test]
    fn test_theme_without_colors() {
        let theme = Theme::new(false);
        assert_eq!(theme.success("test"), "✓ test");
        assert_eq!(theme.error("fail"), "✗ fail");
    }
}
