use anyhow::Result;
use clap::{Parser, Subcommand};
use tracing::info;

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

#[tokio::main]
async fn main() -> Result<()> {
    tracing_subscriber::fmt::init();

    let cli = Cli::parse();

    match cli.command {
        Commands::Info { input } => {
            info!("Reading info from: {}", input);
            println!("Info command not yet implemented");
            Ok(())
        }
        Commands::Encode { input, output, codec } => {
            info!("Encoding {} -> {} (codec: {})", input, output, codec);
            println!("Encode command not yet implemented");
            Ok(())
        }
        Commands::Decode { input, output } => {
            info!("Decoding {} -> {}", input, output);
            println!("Decode command not yet implemented");
            Ok(())
        }
    }
}
