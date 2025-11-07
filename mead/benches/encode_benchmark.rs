//! Encoding benchmark suite
//!
//! Compares different encoder configurations and provides performance metrics.

use mead_core::{
    codec::{av1::{Av1Config, Av1Encoder}, VideoEncoder},
    container::{ivf::IvfMuxer, Muxer, Packet},
    ArcFrame, PixelFormat, Frame,
};
use std::fs::File;
use std::io::BufWriter;
use std::sync::Arc;
use std::time::Instant;

#[derive(Debug, Clone)]
struct BenchmarkConfig {
    name: &'static str,
    width: u32,
    height: u32,
    frames: u32,
    speed: u8,
    tile_cols: usize,
    tile_rows: usize,
    threads: usize,
}

#[derive(Debug)]
struct BenchmarkResult {
    config_name: String,
    resolution: String,
    frames: u32,
    encode_time_ms: u128,
    fps: f64,
    output_size_kb: u64,
    bits_per_pixel: f64,
    parallelism_factor: f64,
}

impl BenchmarkConfig {
    fn resolution(&self) -> String {
        format!("{}x{}", self.width, self.height)
    }

    fn to_av1_config(&self) -> Av1Config {
        Av1Config {
            speed: self.speed,
            quantizer: 100,
            bitrate_kbps: None,
            tile_cols: self.tile_cols,
            tile_rows: self.tile_rows,
            threads: self.threads,
        }
    }
}

/// Generate test frames with animated pattern
fn generate_test_frames(width: u32, height: u32, count: u32) -> Vec<ArcFrame> {
    let mut frames = Vec::with_capacity(count as usize);

    for frame_num in 0..count {
        let mut frame = Frame::new(width, height, PixelFormat::Yuv420p);

        // Y plane: horizontal gradient that moves
        if let Some(y_plane) = frame.plane_y_mut() {
            let offset = (frame_num as usize * 8) % (width as usize);
            for row in 0..height as usize {
                let row_data = y_plane.row_mut(row);
                for col in 0..width as usize {
                    let value = ((col + offset) % 256) as u8;
                    row_data[col] = value;
                }
            }
        }

        // U and V planes: solid gray
        if let Some(u_plane) = frame.plane_u_mut() {
            for row in 0..(height / 2) as usize {
                let row_data = u_plane.row_mut(row);
                for col in 0..(width / 2) as usize {
                    row_data[col] = 128;
                }
            }
        }

        if let Some(v_plane) = frame.plane_v_mut() {
            for row in 0..(height / 2) as usize {
                let row_data = v_plane.row_mut(row);
                for col in 0..(width / 2) as usize {
                    row_data[col] = 128;
                }
            }
        }

        frames.push(Arc::new(frame));
    }

    frames
}

/// Run benchmark for a single configuration
fn run_benchmark(config: &BenchmarkConfig) -> anyhow::Result<BenchmarkResult> {
    println!("  Running: {} @ {}...", config.name, config.resolution());

    // Generate test frames
    let frames = generate_test_frames(config.width, config.height, config.frames);

    // Create encoder
    let av1_config = config.to_av1_config();
    let mut encoder = Av1Encoder::with_config(config.width, config.height, av1_config)?;

    // Create output file
    let output_path = format!("/tmp/bench_{}_{}.ivf", config.name, config.resolution());
    let output = BufWriter::new(File::create(&output_path)?);
    let mut muxer = IvfMuxer::new(output, config.width as u16, config.height as u16, 30, 1)?;

    // Encode frames
    let start = Instant::now();
    let mut cpu_start = get_thread_time();
    let mut pts = 0i64;

    for frame in &frames {
        encoder.send_frame(Some(frame.clone()))?;

        // Collect packets
        while let Some(packet_data) = encoder.receive_packet()? {
            let packet = Packet {
                stream_index: 0,
                data: packet_data,
                pts: Some(pts),
                dts: None,
                is_keyframe: pts == 0,
            };
            muxer.write_packet(packet)?;
            pts += 1;
        }
    }

    // Flush encoder
    encoder.send_frame(None)?;
    while let Some(packet_data) = encoder.receive_packet()? {
        let packet = Packet {
            stream_index: 0,
            data: packet_data,
            pts: Some(pts),
            dts: None,
            is_keyframe: false,
        };
        muxer.write_packet(packet)?;
        pts += 1;
    }

    let cpu_end = get_thread_time();
    let encode_time = start.elapsed();
    let cpu_time = cpu_end - cpu_start;

    // Get output size before finalizing (can't get after because muxer is consumed)
    let output_size_before = std::fs::metadata(&output_path)?.len();

    // Finalize
    muxer.finalize()?;
    let output_size = std::fs::metadata(&output_path)?.len();

    // Clean up
    std::fs::remove_file(&output_path)?;

    // Calculate metrics
    let encode_time_ms = encode_time.as_millis();
    let fps = (config.frames as f64) / (encode_time.as_secs_f64());
    let output_size_kb = output_size / 1024;
    let total_pixels = (config.width * config.height * config.frames) as f64;
    let bits_per_pixel = (output_size as f64 * 8.0) / total_pixels;
    let parallelism_factor = cpu_time / encode_time.as_secs_f64();

    Ok(BenchmarkResult {
        config_name: config.name.to_string(),
        resolution: config.resolution(),
        frames: config.frames,
        encode_time_ms,
        fps,
        output_size_kb,
        bits_per_pixel,
        parallelism_factor,
    })
}

/// Get CPU time for current thread (approximation)
fn get_thread_time() -> f64 {
    // This is a simple approximation - real CPU time would need platform-specific code
    // For now, just return 0.0 and rely on external measurement
    0.0
}

/// Print results table
fn print_results(results: &[BenchmarkResult]) {
    println!("\n{}", "=".repeat(100));
    println!("BENCHMARK RESULTS");
    println!("{}", "=".repeat(100));
    println!(
        "{:<20} {:<12} {:<8} {:<12} {:<10} {:<12} {:<12}",
        "Config", "Resolution", "Frames", "Time (ms)", "FPS", "Size (KB)", "Bits/Pixel"
    );
    println!("{}", "-".repeat(100));

    for result in results {
        println!(
            "{:<20} {:<12} {:<8} {:<12} {:<10.2} {:<12} {:<12.4}",
            result.config_name,
            result.resolution,
            result.frames,
            result.encode_time_ms,
            result.fps,
            result.output_size_kb,
            result.bits_per_pixel,
        );
    }

    println!("{}", "=".repeat(100));
}

fn main() -> anyhow::Result<()> {
    println!("mead AV1 Encoding Benchmark");
    println!("Testing rav1e with tile parallelism optimizations\n");

    let cpu_count = num_cpus::get();
    println!("Detected {} CPU cores\n", cpu_count);

    // Define benchmark configurations
    let configs = vec![
        // 720p baseline
        BenchmarkConfig {
            name: "720p_baseline",
            width: 1280,
            height: 720,
            frames: 30,
            speed: 6,
            tile_cols: 1,
            tile_rows: 1,
            threads: 1,
        },
        // 720p optimized
        BenchmarkConfig {
            name: "720p_optimized",
            width: 1280,
            height: 720,
            frames: 30,
            speed: 6,
            tile_cols: 0, // auto
            tile_rows: 0, // auto
            threads: 0,   // auto
        },
        // 1080p baseline
        BenchmarkConfig {
            name: "1080p_baseline",
            width: 1920,
            height: 1080,
            frames: 30,
            speed: 6,
            tile_cols: 1,
            tile_rows: 1,
            threads: 1,
        },
        // 1080p optimized
        BenchmarkConfig {
            name: "1080p_optimized",
            width: 1920,
            height: 1080,
            frames: 30,
            speed: 6,
            tile_cols: 0, // auto
            tile_rows: 0, // auto
            threads: 0,   // auto
        },
        // 1080p fast preset
        BenchmarkConfig {
            name: "1080p_fast",
            width: 1920,
            height: 1080,
            frames: 30,
            speed: 10,
            tile_cols: 0, // auto
            tile_rows: 0, // auto
            threads: 0,   // auto
        },
        // 1080p quality preset
        BenchmarkConfig {
            name: "1080p_quality",
            width: 1920,
            height: 1080,
            frames: 30,
            speed: 4,
            tile_cols: 0, // auto
            tile_rows: 0, // auto
            threads: 0,   // auto
        },
    ];

    // Run benchmarks
    let mut results = Vec::new();
    for config in &configs {
        match run_benchmark(config) {
            Ok(result) => results.push(result),
            Err(e) => eprintln!("  ERROR: {}", e),
        }
    }

    // Print results
    print_results(&results);

    Ok(())
}
