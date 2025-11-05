use anyhow::Result;
use clap::{Parser, Subcommand};
use mead_core::container::{mp4::Mp4Demuxer, Demuxer};
use std::fs::File;
use tracing::{info, error};

#[derive(Parser)]
#[command(name = "mead")]
#[command(about = "Memory-safe encoding and decoding", long_about = None)]
#[command(version)]
struct Cli {
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
    tracing_subscriber::fmt::init();

    let cli = Cli::parse();

    match cli.command {
        Commands::Info { input } => {
            info!("Reading info from: {}", input);

            let file = File::open(&input)?;
            let demuxer = Mp4Demuxer::new(file)?;

            let metadata = demuxer.metadata();

            println!("File: {}", input);
            println!("Format: {}", metadata.format);
            println!("Streams: {}", metadata.stream_count);

            if let Some(duration_ms) = metadata.duration_ms {
                let seconds = duration_ms / 1000;
                let minutes = seconds / 60;
                let hours = minutes / 60;
                println!(
                    "Duration: {}:{:02}:{:02}.{:03}",
                    hours,
                    minutes % 60,
                    seconds % 60,
                    duration_ms % 1000
                );
            } else {
                println!("Duration: Unknown");
            }

            println!("\nTracks:");
            for (track_id, track) in demuxer.tracks().iter() {
                println!("  Track {}: {:?}", track_id, track.track_type());
                println!("    Media Type: {:?}", track.media_type());
                println!("    Language: {}", track.language());
                println!("    Sample Count: {}", track.sample_count());

                // Display codec-specific info
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
        Commands::Encode { input, output, codec } => {
            info!("Encoding {} -> {} (codec: {})", input, output, codec);
            error!("Encode command not yet implemented");
            Ok(())
        }
        Commands::Decode { input, output } => {
            info!("Decoding {} -> {}", input, output);
            error!("Decode command not yet implemented");
            Ok(())
        }
    }
}
