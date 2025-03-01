mod bencode;
mod cli;
mod error;
mod torrent;
mod types;

use clap::Parser;
use cli::{Cli, CliCommand};
use core::str;
use sha1::{Digest, Sha1};
use std::{
    fs::{self, File},
    io::{BufWriter, Write},
    os::unix::fs::FileExt,
    rc::Rc,
    sync::{Arc, Mutex},
    thread,
};
use torrent::Piece;

use crate::bencode::decoders;
use crate::error::Error;
use crate::error::Result;
use crate::torrent::tracker;
use crate::torrent::HandshakeMessage;
use crate::torrent::Peer;
use crate::torrent::Torrent;
use crate::torrent::TrackerResponse;

const PEER_ID: &str = "12345678901234567890";
const PEER_ID_BYTES: [u8; 20] = *b"12345678901234567890";
const BLOCK_SIZE: usize = 16 * 1024;

// Usage: your_bittorrent.sh decode "<encoded_value>"
fn main() -> Result<()> {
    let cli: Cli = Cli::parse();
    match cli.command {
        CliCommand::Decode { input } => {
            let (decoded, _) =
                decoders::decode(input.as_bytes()).map_err(|err| Error::DecodeError(err))?;
            let json: serde_json::Value = decoded.into();
            println!("{}", &json);
            return Ok(());
        }
        CliCommand::Download {
            torrent_file,
            output,
        } => {
            return download(&torrent_file, &output);
        }
        CliCommand::DownloadPiece {
            torrent_file,
            piece_index,
            output,
        } => {
            let torrent: Torrent = parse_torrent_from_file(&torrent_file)?;
            let pieces: Arc<[Piece]> = torrent.get_pieces();

            let peers = get_peers(&torrent).unwrap();
            let mut peer = Peer::new(&peers[0])?;
            let handshake_message =
                HandshakeMessage::new(&Rc::new(*torrent.get_info_hash()), &Rc::new(PEER_ID_BYTES));
            let handshake_response: HandshakeMessage = peer.handshake(&handshake_message)?;
            println!(
                "Received handshake from {}: {}",
                &peer.get_address(),
                &handshake_response
            );
            // Handshake end
            peer.receive_bitfield()?;
            peer.send_interested()?;
            peer.receive_unchoke()?;

            let piece = download_piece(&mut peer, &pieces[piece_index as usize])?;
            let mut out: File = fs::File::create(&output).map_err(|err| Error::FileError(err))?;
            out.write_all(&piece).map_err(|err| Error::FileError(err))?;
            out.flush().map_err(|err| Error::FileError(err))?;
            return Ok(());
        }
        CliCommand::Handshake {
            torrent_file,
            address,
        } => {
            let torrent: Torrent = fs::read(&torrent_file)
                .map(|s| s.as_slice().try_into())
                .map_err(|err| Error::FileError(err))??;
            let handshake_message =
                HandshakeMessage::new(&Rc::new(*torrent.get_info_hash()), &Rc::new(PEER_ID_BYTES));
            let mut peer = Peer::new(&address).ok().unwrap();
            let response = peer.handshake(&handshake_message).ok().unwrap();
            println!("{}", &response);
            return Ok(());
        }
        CliCommand::Info { torrent_file } => {
            let torrent: Torrent = fs::read(&torrent_file)
                .map(|s| s.as_slice().try_into().ok().unwrap())
                .map_err(|err| Error::FileError(err))?;
            println!("{}", &torrent);
            return Ok(());
        }
        CliCommand::Peers { torrent_file } => {
            let torrent: Torrent = fs::read(&torrent_file)
                .map(|s| s.as_slice().try_into().ok().unwrap())
                .map_err(|err| Error::FileError(err))?;
            let tracker_response: TrackerResponse = tracker::get(
                /* torrent= */ &torrent,
                /* peer_id= */ PEER_ID,
                /* port= */ 6881,
                /* uploaded= */ 0,
                /* downloaded= */ 0,
                /* left= */ torrent.get_length(),
            )?;

            match tracker_response {
                TrackerResponse::Ok { interval: _, peers } => {
                    for peer in peers.as_ref() {
                        println!("{}", peer);
                    }
                    return Ok(());
                }
                TrackerResponse::Failure(reason) => {
                    return Err(Error::TrackerFailureInResponse {
                        failure_reason: reason,
                    })
                }
            }
        }
    }
}

fn parse_torrent_from_file(path: &str) -> Result<Torrent> {
    fs::read(path)
        .map(|s: Vec<u8>| s.as_slice().try_into())
        .map_err(|err| Error::FileError(err))?
}

fn download(torrent_file_path: &str, out_file_path: &str) -> Result<()> {
    let torrent = parse_torrent_from_file(&torrent_file_path)?;

    let peer_addresses: Box<[String]> = get_peers(&torrent).unwrap();

    let mut file = fs::File::create(out_file_path).map_err(|err| Error::FileError(err))?;
    reserve_space(&mut file, torrent.get_length())?;
    for p in torrent.get_pieces().as_ref() {
        println!(
            "Piece #{}: begin={}, length = {}",
            p.get_index(),
            p.get_begin(),
            p.get_length()
        );
    }

    let pieces_shared: Arc<[Piece]> = torrent.get_pieces();
    let piece_indices_shared = Arc::new(Mutex::new(Vec::from_iter(0..pieces_shared.len())));
    let file_shared: Arc<Mutex<File>> = Arc::new(Mutex::new(file));
    let info_hash: [u8; 20] = *torrent.get_info_hash();
    let mut handles: Vec<thread::JoinHandle<()>> = Vec::new();
    for peer_addr in peer_addresses.as_ref() {
        let file_shared: Arc<Mutex<File>> = Arc::clone(&file_shared);
        let pieces_shared: Arc<[Piece]> = Arc::clone(&pieces_shared);
        let piece_indices_shared: Arc<Mutex<Vec<usize>>> = Arc::clone(&piece_indices_shared);

        let peer_addr: String = peer_addr.clone();
        // println!("Peer {} received {} pieces job.", &addr, chunk_boxed.len());
        let handle = thread::spawn(move || {
            let mut peer = Peer::new(peer_addr).unwrap();
            let handshake_message =
                HandshakeMessage::new(&Rc::new(info_hash), &Rc::new(PEER_ID_BYTES));
            let handshake_response: HandshakeMessage = peer.handshake(&handshake_message).unwrap();
            println!(
                "Received handshake from {}: {}",
                &peer.get_address(),
                &handshake_response
            );
            // Handshake end
            peer.receive_bitfield().unwrap();
            peer.send_interested().unwrap();
            peer.receive_unchoke().unwrap();

            loop {
                let maybe_piece_index = {
                    let mut piece_indices_guard = piece_indices_shared.lock().unwrap();
                    piece_indices_guard.pop()
                };

                if let Some(index) = maybe_piece_index {
                    let piece: &Piece = &pieces_shared[index];
                    let data: Box<[u8]> = download_piece(&mut peer, piece).unwrap();
                    let mut file = file_shared.lock().unwrap();

                    file.write_all_at(&data, piece.get_begin()).unwrap();
                    file.flush().unwrap();
                } else {
                    break;
                }
            }
        });
        handles.push(handle);
    }

    // Join all threads
    for handle in handles {
        handle.join().unwrap();
    }

    println!("{}", torrent);

    Ok(())
}

pub fn get_peers(torrent: &Torrent) -> Result<Box<[String]>> {
    let tracker_response: TrackerResponse = tracker::get(
        /* torrent= */ &torrent,
        /* peer_id= */ PEER_ID,
        /* port= */ 6881,
        /* uploaded= */ 0,
        /* downloaded= */ 0,
        /* left= */ torrent.get_length(),
    )?;
    return match tracker_response {
        TrackerResponse::Ok { interval: _, peers } => Ok(peers),
        TrackerResponse::Failure(reason) => Err(Error::TrackerFailureInResponse {
            failure_reason: reason,
        }),
    };
}

fn download_piece(peer: &mut Peer, piece: &Piece) -> Result<Box<[u8]>> {
    println!("Downloading piece #{}...", piece.get_index());
    let blocks_count: usize = piece.get_blocks().len();
    let mut piece_data: Box<[u8]> = vec![
        0;
        piece
            .get_blocks()
            .iter()
            .map(|block| block.get_length())
            .sum::<u32>() as usize
    ]
    .into_boxed_slice();
    for (i, block) in piece.get_blocks().iter().enumerate() {
        println!(
            "[Peer @{}] Downloading block {}/{} for piece #{}...",
            &peer.get_address(),
            i + 1,
            blocks_count,
            piece.get_index()
        );
        let block_data =
            peer.get_piece_block(piece.get_index(), block.get_begin(), block.get_length())?;
        piece_data
            [block.get_begin() as usize..block.get_begin() as usize + block.get_length() as usize]
            .copy_from_slice(&block_data);
        println!(
            "[Peer @{}] Downloaded block {}/{} for piece #{}.",
            &peer.get_address(),
            i + 1,
            blocks_count,
            piece.get_index()
        );
    }

    println!(
        "[Peer @{}] Downloaded piece #{}. Verifying SHA1 checksum...",
        &peer.get_address(),
        piece.get_index()
    );
    let mut hasher = Sha1::new();
    hasher.update(piece_data.as_ref());
    let sha1_hash: [u8; 20] = hasher.finalize().into();
    if *piece.get_hash() != sha1_hash {
        println!(
            "[Peer @{}] SHA1 for downloaded piece #{} does not match with torrent file.",
            &peer.get_address(),
            piece.get_index()
        );
        Err(Error::Unknown)? // TODO: set actual error
    }

    println!(
        "[Peer @{}] SHA1 for downloaded piece #{} matches with torrent file.",
        &peer.get_address(),
        piece.get_index()
    );

    Ok(piece_data)
}

fn reserve_space(file: &mut File, size_in_bytes: u64) -> Result<()> {
    let mut file: BufWriter<&File> = BufWriter::new(file);
    if size_in_bytes > BLOCK_SIZE as u64 {
        let blocks_count: u64 = size_in_bytes / BLOCK_SIZE as u64;
        let residue: u64 = size_in_bytes % BLOCK_SIZE as u64;
        for _ in 0..blocks_count {
            let bytes = vec![0u8; BLOCK_SIZE as usize];
            file.write_all(&bytes)
                .map_err(|err| Error::FileError(err))?;
        }
        if residue > 0 {
            let bytes = vec![0u8; residue as usize];
            file.write_all(&bytes)
                .map_err(|err| Error::FileError(err))?;
        }
    } else {
        let bytes = vec![0u8; size_in_bytes as usize];
        file.write_all(&bytes)
            .map_err(|err| Error::FileError(err))?;
    }
    file.flush().map_err(|err| Error::FileError(err))
}
