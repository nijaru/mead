mod output;

use anyhow::Result;
use clap::{Parser, Subcommand};
use mead_core::container::{mp4::Mp4Demuxer, ivf::IvfMuxer, Demuxer, Muxer, Packet};
use mead_core::codec::opus::OpusDecoderImpl;
use mead_core::codec::av1::Av1Encoder;
use mead_core::codec::{AudioDecoder, VideoEncoder};
use mead_core::{Frame, PixelFormat, Plane, ArcFrame};
use audiopus::{SampleRate, Channels};
use std::fs::File;
use std::io::Write;
use std::time::Instant;
use output::{OutputConfig, Theme};

#[derive(Parser)]
#[command(name = "mead")]
#[command(about = "Memory-safe encoding and decoding", long_about = None)]
#[command(version)]
struct Cli {
    /// Suppress progress output (errors only)
    #[arg(long, short, global = true)]
    quiet: bool,

    /// Output JSON format
    #[arg(long, global = true)]
    json: bool,

    /// Disable colored output
    #[arg(long, global = true)]
    no_color: bool,

    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Display container and stream information
    Info {
        /// Input file path
        input: String,
    },
    /// Encode video/audio
    Encode {
        /// Input file path
        input: String,
        /// Output file path
        #[arg(short, long)]
        output: String,
        /// Video codec (av1, h264)
        #[arg(long, default_value = "av1")]
        codec: String,
    },
    /// Decode video/audio
    Decode {
        /// Input file path
        input: String,
        /// Output file path
        #[arg(short, long)]
        output: String,
    },
}

fn main() -> Result<()> {
    let cli = Cli::parse();

    // Initialize tracing only if not quiet
    if !cli.quiet {
        tracing_subscriber::fmt()
            .with_writer(std::io::stderr)
            .init();
    }

    // Create output config
    let output_config = OutputConfig::new(cli.quiet, cli.json, cli.no_color);
    let theme = Theme::new(output_config.use_colors());

    match cli.command {
        Commands::Info { input } => {
            if cli.json {
                handle_info_json(&input)?;
            } else {
                handle_info_human(&input, &theme)?;
            }
            Ok(())
        }
        Commands::Encode { input, output, codec } => {
            handle_encode(&input, &output, &codec, &output_config, &theme)?;
            Ok(())
        }
        Commands::Decode { input, output } => {
            handle_decode(&input, &output, &output_config, &theme)?;
            Ok(())
        }
    }
}

fn handle_info_json(input: &str) -> Result<()> {
    let file = File::open(input)?;
    let demuxer = Mp4Demuxer::new(file)?;
    let metadata = demuxer.metadata();

    let json = serde_json::json!({
        "file": input,
        "format": metadata.format,
        "stream_count": metadata.stream_count,
        "duration_ms": metadata.duration_ms,
        "tracks": demuxer.tracks().iter().map(|(track_id, track)| {
            serde_json::json!({
                "id": track_id,
                "type": format!("{:?}", track.track_type().ok()),
                "media_type": format!("{:?}", track.media_type()),
                "language": track.language(),
                "sample_count": track.sample_count(),
                "width": track.width(),
                "height": track.height(),
            })
        }).collect::<Vec<_>>(),
    });

    println!("{}", serde_json::to_string_pretty(&json)?);
    Ok(())
}

fn handle_info_human(input: &str, theme: &Theme) -> Result<()> {
    let file = File::open(input)?;
    let demuxer = Mp4Demuxer::new(file)?;
    let metadata = demuxer.metadata();

    println!("{}: {}", theme.highlight("File"), input);
    println!("{}: {}", theme.highlight("Format"), metadata.format);
    println!("{}: {}", theme.highlight("Streams"), metadata.stream_count);

    if let Some(duration_ms) = metadata.duration_ms {
        let seconds = duration_ms / 1000;
        let minutes = seconds / 60;
        let hours = minutes / 60;
        println!(
            "{}: {}:{:02}:{:02}.{:03}",
            theme.highlight("Duration"),
            hours,
            minutes % 60,
            seconds % 60,
            duration_ms % 1000
        );
    } else {
        println!("{}: Unknown", theme.highlight("Duration"));
    }

    println!("\n{}:", theme.highlight("Tracks"));
    for (track_id, track) in demuxer.tracks().iter() {
        println!("  Track {}: {:?}", track_id, track.track_type());
        println!("    Media Type: {:?}", track.media_type());
        println!("    Language: {}", track.language());
        println!("    Sample Count: {}", track.sample_count());

        match track.track_type() {
            Ok(mp4::TrackType::Video) => {
                if let Ok(video_profile) = track.video_profile() {
                    println!("    Video Profile: {:?}", video_profile);
                }
                println!("    Width: {}", track.width());
                println!("    Height: {}", track.height());
            }
            Ok(mp4::TrackType::Audio) => {
                if let Ok(audio_profile) = track.audio_profile() {
                    println!("    Audio Profile: {:?}", audio_profile);
                }
            }
            _ => {}
        }
    }

    Ok(())
}

fn handle_decode(
    input: &str,
    output: &str,
    config: &OutputConfig,
    theme: &Theme,
) -> Result<()> {
    let start_time = Instant::now();

    let file = File::open(input)?;
    let mut demuxer = Mp4Demuxer::new(file)?;

    // Try to select an audio track
    if demuxer.select_audio_track().is_err() {
        return Err(anyhow::anyhow!("No audio tracks found in file"));
    }

    // Get total sample count for progress bar
    let total_samples = demuxer
        .audio_tracks()
        .first()
        .map(|(_, track)| track.sample_count() as u64)
        .unwrap_or(0);

    // Create output file
    let mut output_file = File::create(output)?;

    // Create audio decoder (assume Opus for now)
    let mut audio_decoder = OpusDecoderImpl::new(SampleRate::Hz48000, Channels::Stereo)?;

    // Create progress bar if appropriate
    let pb = if config.show_progress() {
        Some(output::create_progress_bar(total_samples, "Decoding"))
    } else {
        None
    };

    // Decode packets
    let mut packet_count = 0;
    while let Some(packet) = demuxer.read_packet()? {
        packet_count += 1;

        // Decode the audio packet
        if let Some(samples) = audio_decoder.decode(&packet.data)? {
            // Write raw PCM samples (little-endian f32)
            for &sample in &samples {
                output_file.write_all(&sample.to_le_bytes())?;
            }
        }

        // Update progress
        if let Some(ref pb) = pb {
            pb.inc(1);
        }
    }

    // Finish progress bar
    if let Some(pb) = pb {
        pb.finish_and_clear();
    }

    let elapsed = start_time.elapsed();

    if !config.quiet {
        eprintln!(
            "{}",
            theme.success(&format!(
                "Decoded {} packets to {} in {}",
                packet_count,
                output,
                output::format_duration(elapsed)
            ))
        );
    }

    Ok(())
}

fn handle_encode(
    input: &str,
    output: &str,
    codec: &str,
    config: &OutputConfig,
    theme: &Theme,
) -> Result<()> {
    if codec != "av1" {
        return Err(anyhow::anyhow!("Only AV1 codec is supported currently"));
    }

    let start_time = Instant::now();

    eprintln!("{}", theme.info(&format!("Encoding {} -> {} (codec: {})", input, output, codec)));

    // For MVP: Generate a test pattern (solid color frames)
    // TODO: Add video decoding from input file in future
    let width = 1280;
    let height = 720;
    let fps = 30;
    let num_frames = 100; // Generate 100 test frames

    // Create AV1 encoder
    let mut encoder = Av1Encoder::new(width, height)?;

    // Create IVF muxer
    let output_file = File::create(output)?;
    let mut muxer = IvfMuxer::new(output_file, width as u16, height as u16, fps, 1)?;

    // Create progress bar
    let pb = if config.show_progress() {
        Some(output::create_progress_bar(num_frames, "Encoding"))
    } else {
        None
    };

    // Generate and encode test frames
    for frame_idx in 0..num_frames {
        // Generate a simple test pattern (gray frame)
        let frame = generate_test_frame(width, height, frame_idx)?;

        // Encode frame
        encoder.send_frame(Some(frame))?;

        // Receive encoded packets
        while let Some(packet_data) = encoder.receive_packet()? {
            let packet = Packet {
                stream_index: 0,
                data: packet_data,
                pts: Some(frame_idx as i64),
                dts: None,
                is_keyframe: frame_idx == 0,  // First frame is keyframe
            };
            muxer.write_packet(packet)?;
        }

        if let Some(ref pb) = pb {
            pb.inc(1);
            if frame_idx % 10 == 0 {
                let elapsed = start_time.elapsed();
                let fps_actual = (frame_idx + 1) as f64 / elapsed.as_secs_f64();
                pb.set_message(format!("{:.1} fps", fps_actual));
            }
        }
    }

    // Flush encoder
    encoder.send_frame(None)?;
    while let Some(packet_data) = encoder.receive_packet()? {
        let packet = Packet {
            stream_index: 0,
            data: packet_data,
            pts: Some(num_frames as i64),
            dts: None,
            is_keyframe: false,
        };
        muxer.write_packet(packet)?;
    }

    if let Some(pb) = pb {
        pb.finish_and_clear();
    }

    // Finalize muxer
    muxer.finalize()?;

    let elapsed = start_time.elapsed();
    let actual_fps = num_frames as f64 / elapsed.as_secs_f64();

    if !config.quiet {
        eprintln!(
            "{}",
            theme.success(&format!(
                "Encoded {} frames to {} in {} ({:.1} fps)",
                num_frames,
                output,
                output::format_duration(elapsed),
                actual_fps
            ))
        );
    }

    Ok(())
}

/// Generate a simple test frame (gray color with animated brightness)
fn generate_test_frame(width: u32, height: u32, frame_idx: u64) -> Result<ArcFrame> {
    use std::sync::Arc;

    // Create frame with YUV420p format
    let mut frame = Frame::new(width, height, PixelFormat::Yuv420p);

    // Animate brightness (cycles from 64 to 192)
    let brightness = 128 + ((frame_idx % 100) as i32 - 50);
    let brightness = brightness.clamp(64, 192) as u8;

    let planes = frame.planes_mut();

    // Fill Y plane (luma) with animated brightness
    for pixel in planes[0].data_mut() {
        *pixel = brightness;
    }

    // Fill U and V planes (chroma) with neutral gray (128 = neutral)
    for plane in &mut planes[1..=2] {
        for pixel in plane.data_mut() {
            *pixel = 128;
        }
    }

    Ok(Arc::new(frame))
}
