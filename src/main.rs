use std::fs;
use std::path::PathBuf;

use clap::{Parser, Subcommand};
use rainbow::rainbow::Rainbow;
use rainbow::{DecodeResult, EncodeOptions, EncodeResult, NetworkSteganographyProcessor};
use tracing::info;

#[derive(Parser)]
#[command(author, version, about, long_about = None)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Encode data into HTTP packets
    Encode {
        /// Input file path
        #[arg(short, long)]
        input: PathBuf,

        /// Output directory path
        #[arg(short, long)]
        output: PathBuf,

        /// Whether to encode as client
        #[arg(short, long)]
        client: bool,

        /// MIME type
        #[arg(short, long)]
        mime_type: Option<String>,
    },

    /// Decode a single HTTP packet
    Decode {
        /// Input file path
        #[arg(short, long)]
        input: PathBuf,

        /// Output file path
        #[arg(short, long)]
        output: PathBuf,

        /// Packet index
        #[arg(short, long, default_value = "0")]
        index: usize,

        /// Whether to decode as client
        #[arg(short, long)]
        client: bool,
    },
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Initialize logging
    tracing_subscriber::fmt::init();

    let cli = Cli::parse();
    let rainbow = Rainbow::new();

    match cli.command {
        Commands::Encode {
            input,
            output,
            client,
            mime_type,
        } => {
            // Read input file
            let data = fs::read(&input)?;

            // Encode data
            let EncodeResult {
                encoded_packets: packets,
                expected_return_packet_lengths: lengths,
            } = rainbow.encode_write(
                &data,
                client,
                EncodeOptions {
                    mime_type: mime_type,
                    ..Default::default()
                },
            )?;

            // Create output directory
            fs::create_dir_all(&output)?;

            // Write each packet to a separate file
            for (i, (packet, length)) in packets.iter().zip(lengths.iter()).enumerate() {
                let file_path = output.join(format!("packet_{}.http", i));
                fs::write(&file_path, packet)?;
                info!(
                    "Writing packet {} to {:?}, length: {}",
                    i, file_path, length
                );
            }
        }

        Commands::Decode {
            input,
            output,
            index,
            client,
        } => {
            // Read input file
            let data = fs::read(&input)?;

            // Decode data
            let DecodeResult {
                data: decoded,
                expected_return_length: expected_length,
                is_read_end: is_end,
            } = rainbow.decrypt_single_read(data, index, client)?;

            // Write decoded data
            fs::write(&output, decoded)?;
            info!(
                "Decoded packet {} to {:?}, expected length: {}, is last packet: {}",
                index, output, expected_length, is_end
            );
        }
    }

    Ok(())
}
