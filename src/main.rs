mod bencode;
mod cli;
mod error;
mod torrent;
mod types;

use clap::Parser;
use cli::Cli;
use torrent::Piece;

use crate::bencode::decoders;
use crate::error::Result;
use crate::torrent::tracker;
use crate::torrent::HandshakeMessage;
use crate::torrent::Peer;
use crate::torrent::Torrent;
use crate::torrent::TrackerResponse;

// Usage: your_bittorrent.sh decode "<encoded_value>"
fn main() -> Result<()> {
    let cli: Cli = Cli::parse();
    cli.command.handle()
}
