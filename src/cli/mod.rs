use std::{
    collections::BTreeMap,
    fs::{self, File},
    io::{BufWriter, Write},
    os::unix::fs::FileExt,
    rc::Rc,
    sync::{Arc, Mutex},
    thread,
};

use clap::{Parser, Subcommand};
use sha1::{Digest, Sha1};

use crate::{
    decoders,
    error::{Error, Result},
    magnet::magnet_link_v1::MagnetLinkV1,
    tracker,
    types::DataType,
    HandshakeMessage, Peer, Piece, Torrent, TrackerResponse,
};

const PEER_ID: &str = "12345678901234567890";
const PEER_ID_BYTES: [u8; 20] = *b"12345678901234567890";
const BLOCK_SIZE: usize = 16 * 1024;

#[derive(Parser, Debug)]
#[command(name = "codecrafters-bittorrent")]
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
    /// Parse a Magnet link and perform a handshake with a peer
    #[command(name = "magnet_handshake")]
    MagnetHandshake {
        /// The Magnet link to parse
        magnet_url: String,
    },
    /// Parse a Magnet link
    #[command(name = "magnet_parse")]
    MagnetParse {
        /// The Magnet link to parse
        magnet_url: String,
    },
    /// Print the list of peers for a .torrent file
    Peers {
        /// The path to a .torrent file (positional argument)
        torrent_file: String,
    },
}

impl CliCommand {
    pub fn handle(&self) -> Result<()> {
        match self {
            CliCommand::Decode { input } => handle_decode(input),
            CliCommand::Download {
                torrent_file,
                output,
            } => handle_download(torrent_file, output),
            CliCommand::DownloadPiece {
                torrent_file,
                piece_index,
                output,
            } => handle_download_piece(torrent_file, *piece_index, output),
            CliCommand::Handshake {
                torrent_file,
                address,
            } => handle_handshake(torrent_file, address),
            CliCommand::Info { torrent_file } => handle_info(torrent_file),
            CliCommand::MagnetHandshake { magnet_url } => handle_magnet_handshake(magnet_url),
            CliCommand::MagnetParse { magnet_url } => handle_magnet_parse(magnet_url),
            CliCommand::Peers { torrent_file } => handle_peers(torrent_file),
        }
    }
}

fn handle_decode(input: &str) -> Result<()> {
    let (decoded, _) = decoders::decode(input.as_bytes()).map_err(|err| Error::DecodeError(err))?;
    let json: serde_json::Value = decoded.into();
    println!("{}", &json);
    Ok(())
}

fn handle_download(torrent_file_path: &str, out_file_path: &str) -> Result<()> {
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

fn handle_download_piece(
    torrent_file_path: &str,
    piece_index: usize,
    output_file_path: &str,
) -> Result<()> {
    let torrent: Torrent = parse_torrent_from_file(torrent_file_path)?;
    let pieces: Arc<[Piece]> = torrent.get_pieces();

    let peers: Box<[String]> = get_peers(&torrent).unwrap();
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

    let piece: Box<[u8]> = download_piece(&mut peer, &pieces[piece_index as usize])?;
    let mut out: File = fs::File::create(output_file_path).map_err(|err| Error::FileError(err))?;
    out.write_all(&piece).map_err(|err| Error::FileError(err))?;
    out.flush().map_err(|err| Error::FileError(err))?;
    Ok(())
}

fn handle_handshake(torrent_file_path: &str, peer_address: &str) -> Result<()> {
    let torrent: Torrent = fs::read(&torrent_file_path)
        .map(|s| s.as_slice().try_into())
        .map_err(|err| Error::FileError(err))??;
    let handshake_message =
        HandshakeMessage::new(&Rc::new(*torrent.get_info_hash()), &Rc::new(PEER_ID_BYTES));
    let mut peer = Peer::new(&peer_address).ok().unwrap();
    let response = peer.handshake(&handshake_message).ok().unwrap();
    println!("{}", &response);
    Ok(())
}

fn handle_info(torrent_file_path: &str) -> Result<()> {
    let torrent: Torrent = fs::read(torrent_file_path)
        .map(|s| s.as_slice().try_into().ok().unwrap())
        .map_err(|err| Error::FileError(err))?;
    println!("{}", &torrent);
    Ok(())
}

fn handle_magnet_handshake(magnet_url: &str) -> Result<()> {
    let link: MagnetLinkV1 = MagnetLinkV1::parse(magnet_url)?;
    let hash_str_bytes = hex::decode(link.get_info_hash().as_bytes()).unwrap();
    let info_hash: [u8; 20] = hash_str_bytes.as_slice().try_into().unwrap();
    let tracker_response: TrackerResponse = tracker::get(
        /* tracker_url= */ link.get_tracker_url(),
        /* info_hash= */ &info_hash,
        /* peer_id= */ PEER_ID,
        /* port= */ 6881,
        /* uploaded= */ 0,
        /* downloaded= */ 0,
        /* left= */ 12345,
    )?;
    match tracker_response {
        TrackerResponse::Ok { interval: _, peers } => {
            let peer_address = peers.as_ref().get(0).unwrap();
            let handshake_message =
                HandshakeMessage::new_magnet(&Rc::new(info_hash), &Rc::new(PEER_ID_BYTES));
            let mut peer = Peer::new(peer_address).ok().unwrap();
            let response = peer.handshake(&handshake_message).ok().unwrap();
            // TODO: send bitfield
            peer.receive_bitfield()?;
            if response.is_extension_supported() {
                let mut m_dict: BTreeMap<String, DataType> = BTreeMap::new();
                m_dict.insert("ut_metadata".to_owned(), DataType::Integer(123));
                let mut payload_dict: BTreeMap<String, DataType> = BTreeMap::new();
                payload_dict.insert("m".to_owned(), DataType::Dict(m_dict));
                let extended_handshake_response = peer.extended_handshake(payload_dict)?;
            }
            println!("{}", &response);
            return Ok(());
        }
        TrackerResponse::Failure(reason) => {
            return Err(Error::TrackerFailureInResponse {
                failure_reason: reason,
            })
        }
    }
    todo!();
}

fn handle_magnet_parse(magnet_url: &str) -> Result<()> {
    let link: MagnetLinkV1 = MagnetLinkV1::parse(magnet_url)?;
    println!(
        "Tracker URL: {}\nInfo Hash: {}",
        link.get_tracker_url(),
        link.get_info_hash()
    );
    Ok(())
}

fn handle_peers(torrent_file_path: &str) -> Result<()> {
    let torrent: Torrent = fs::read(torrent_file_path)
        .map(|s| s.as_slice().try_into().ok().unwrap())
        .map_err(|err| Error::FileError(err))?;
    let tracker_response: TrackerResponse = tracker::get(
        /* tracker_url= */ torrent.get_announce(),
        /* info_hash= */ torrent.get_info_hash(),
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

fn get_peers(torrent: &Torrent) -> Result<Box<[String]>> {
    let tracker_response: TrackerResponse = tracker::get(
        /* tracker_url= */ torrent.get_announce(),
        /* info_hash= */ torrent.get_info_hash(),
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

fn parse_torrent_from_file(path: &str) -> Result<Torrent> {
    fs::read(path)
        .map(|s: Vec<u8>| s.as_slice().try_into())
        .map_err(|err| Error::FileError(err))?
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
