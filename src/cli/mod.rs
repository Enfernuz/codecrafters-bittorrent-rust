use clap::{Parser, Subcommand};

#[derive(Parser, Debug)]
#[command(name = "torrent-cli")]
#[command(about = "A CLI for managing torrent downloads", long_about = None)]
pub struct Cli {
    #[command(subcommand)]
    pub command: CliCommand,
}

#[derive(Subcommand, Debug)]
pub enum CliCommand {
    /// Decode bencoded string
    Decode {
        /// The string to decode (positional argument)
        input: String,
    },
    /// Download the whole content of a .torrent file
    #[command()]
    Download {
        /// Path to the torrent file (positional argument)
        torrent_file: String,

        /// Output file path (named argument)
        #[arg(short, long)]
        output: String,
    },
    /// Download a specific piece of a .torrent file
    #[command(name = "download_piece")]
    DownloadPiece {
        /// Path to the torrent file (positional argument)
        torrent_file: String,

        /// Piece index to download (positional argument)
        piece_index: usize,

        /// Output file path (named argument)
        #[arg(short, long)]
        output: String,
    },
    /// Send and receive handshake message to a peer of a .torrent file
    Handshake {
        /// The path to a .torrent file (positional argument)
        torrent_file: String,

        // The address of a peer to handshake with
        address: String,
    },
    /// Print info about a .torrent file
    Info {
        /// The path to a .torrent file (positional argument)
        torrent_file: String,
    },
    /// Print the list of peers for a .torrent file
    Peers {
        /// The path to a .torrent file (positional argument)
        torrent_file: String,
    },
}
