mod bencode;
mod error;
mod torrent;
mod types;

use core::str;
use sha1::{Digest, Sha1};
use std::{
    env,
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
    let args: Vec<String> = env::args().collect();
    let command = &args[1];

    if command == "decode" {
        let encoded_value = &args[2];
        let (decoded, _) =
            decoders::decode(encoded_value.as_bytes()).map_err(|err| Error::DecodeError(err))?;
        let json: serde_json::Value = decoded.into();
        println!("{}", &json);
        return Ok(());
    } else if command == "info" {
        let path = &args[2];
        let torrent: Torrent = fs::read(path)
            .map(|s| s.as_slice().try_into().ok().unwrap())
            .map_err(|err| Error::FileError(err))?;
        println!("{}", &torrent);
        return Ok(());
    } else if command == "peers" {
        let path = &args[2];
        let torrent: Torrent = fs::read(path)
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
    } else if command == "handshake" {
        let path = &args[2];
        let addr = &args[3];
        let torrent: Torrent = fs::read(path)
            .map(|s| s.as_slice().try_into())
            .map_err(|err| Error::FileError(err))??;
        let handshake_message =
            HandshakeMessage::new(&Rc::new(*torrent.get_info_hash()), &Rc::new(PEER_ID_BYTES));
        let mut peer = Peer::new(addr).ok().unwrap();
        let response = peer.handshake(&handshake_message).ok().unwrap();
        println!("{}", &response);
        return Ok(());
    } else if command == "download_piece" {
        let out_file_path = &args[3];
        let torrent_file_path = &args[4];
        let piece_index: u32 = args[5].parse().expect("TODO: piece_index parse error");

        let torrent = parse_torrent_from_file(&torrent_file_path)?;
        let pieces = torrent.get_pieces();

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
        let mut out: File = fs::File::create(out_file_path).map_err(|err| Error::FileError(err))?;
        out.write_all(&piece).map_err(|err| Error::FileError(err))?;
        out.flush().map_err(|err| Error::FileError(err))?;
        Ok(())
    } else if command == "download" {
        let out_file_path = &args[3];
        let torrent_file_path = &args[4];

        download(torrent_file_path, out_file_path)
    } else {
        eprintln!("unknown command: {}", args[1]);
        return Err(Error::Unknown);
    }
}

fn parse_torrent_from_file(path: &str) -> Result<Torrent> {
    fs::read(path)
        .map(|s: Vec<u8>| s.as_slice().try_into())
        .map_err(|err| Error::FileError(err))?
}

fn download(torrent_file_path: &str, out_file_path: &str) -> Result<()> {
    let torrent = parse_torrent_from_file(&torrent_file_path)?;

    let peer_addresses = get_peers(&torrent).unwrap();

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

    let pieces = torrent.get_pieces();
    let piece_indices: Vec<usize> = (0..pieces.len()).collect();
    let tasks_shared = Arc::new(Mutex::new(piece_indices));

    let mut handles = Vec::new();
    // Convert `Box<[Piece]>` into `Vec<Piece>` to consume it
    let file_shared: Arc<Mutex<File>> = Arc::new(Mutex::new(file));
    let info_hash: [u8; 20] = *torrent.get_info_hash();
    for peer_addr in peer_addresses.as_ref() {
        let file_shared_clone = Arc::clone(&file_shared);
        let pieces_shared_clone = Arc::clone(&pieces);
        let tasks_shared_clone = Arc::clone(&tasks_shared);

        let addr = peer_addr.clone();
        // println!("Peer {} received {} pieces job.", &addr, chunk_boxed.len());
        let handle = thread::spawn(move || {
            let mut peer = Peer::new(addr).unwrap();
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
                    let mut tasks_guard = tasks_shared_clone.lock().unwrap();
                    tasks_guard.pop()
                };

                if let Some(index) = maybe_piece_index {
                    let piece: &Piece = &pieces_shared_clone[index];
                    let data = download_piece(&mut peer, piece).unwrap();
                    let mut file = file_shared_clone.lock().unwrap();

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
