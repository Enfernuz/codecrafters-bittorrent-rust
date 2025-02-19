mod bencode;
mod error;
mod torrent;
mod types;

use std::io::{BufWriter, Write};
use std::rc::Rc;
use std::{env, fs};

use bencode::decoder;
use error::Error;
use torrent::message::handshake_message::HandshakeMessage;
use torrent::peer::Peer;
use torrent::tracker::{self, TrackerResponse};
use torrent::Torrent;

pub use crate::bencode::decoder::bytestring_decoder;

// Usage: your_bittorrent.sh decode "<encoded_value>"
fn main() {
    let args: Vec<String> = env::args().collect();
    let command = &args[1];

    if command == "decode" {
        let encoded_value = &args[2];
        let result: Result<(types::DataType, usize), decoder::DecodeError> =
            decoder::decode(encoded_value.as_bytes());
        match result {
            Ok((decoded, _)) => {
                let json: serde_json::Value = decoded.into();
                println!("{}", &json)
            }
            Err(err) => eprintln!("Error! -- {}", err),
        }
    } else if command == "info" {
        let path = &args[2];
        let result: Result<Torrent, std::io::Error> =
            fs::read(path).map(|s| s.as_slice().try_into().ok().unwrap());
        match result {
            Ok(torrent) => println!("{}", &torrent),
            Err(err) => eprintln!("Error! -- {}", &err),
        }
    } else if command == "peers" {
        let path = &args[2];
        let result: Result<Torrent, std::io::Error> =
            fs::read(path).map(|s| s.as_slice().try_into().ok().unwrap());

        match result {
            Ok(torrent) => {
                let tracker_response: TrackerResponse = tracker::get(
                    /* torrent= */ &torrent,
                    /* peer_id= */ "12345678901234567890",
                    /* port= */ 6881,
                    /* uploaded= */ 0,
                    /* downloaded= */ 0,
                    /* left= */ torrent.get_length(),
                )
                .unwrap();
                match tracker_response {
                    TrackerResponse::Ok { interval: _, peers } => {
                        for peer in peers.as_ref() {
                            println!("{}", peer);
                        }
                    }
                    TrackerResponse::Failure(reason) => {
                        eprintln!("Got failure response from the tracker: {}", &reason);
                    }
                }
            }
            Err(err) => eprintln!("Error! -- {}", &err),
        }
    } else if command == "handshake" {
        let path = &args[2];
        let addr = &args[3];
        let result: Result<Torrent, std::io::Error> =
            fs::read(path).map(|s| s.as_slice().try_into().ok().unwrap());
        match result {
            Ok(torrent) => {
                let peer_id: [u8; 20] = *b"12345678901234567890";
                let handshake_message =
                    HandshakeMessage::new(torrent.get_info_hash(), &Rc::new(peer_id));
                let mut peer = Peer::new(addr).ok().unwrap();
                let response = peer.handshake(&handshake_message).ok().unwrap();
                println!("{}", &response);
            }
            Err(err) => {
                eprintln!("Got failure response from the peer: {}", &err);
            }
        }
    } else if command == "download_piece" {
        let out = &args[3];
        let path = &args[4];
        let piece_index: u32 = args[5].parse().expect("TODO: piece_index parse error");

        let result: Result<Torrent, std::io::Error> =
            fs::read(path).map(|s| s.as_slice().try_into().ok().unwrap());

        match result {
            Ok(torrent) => {
                let tracker_response: TrackerResponse = tracker::get(
                    /* torrent= */ &torrent,
                    /* peer_id= */ "12345678901234567890",
                    /* port= */ 6881,
                    /* uploaded= */ 0,
                    /* downloaded= */ 0,
                    /* left= */ torrent.get_length(),
                )
                .unwrap();
                match tracker_response {
                    TrackerResponse::Ok { interval: _, peers } => {
                        // Connect to a peer and download the piece.
                        // Handshake start
                        for peer in peers.as_ref() {
                            println!("Found a peer: {peer}");
                        }
                        let peer_id: [u8; 20] = *b"12345678901234567890";
                        let handshake_message =
                            HandshakeMessage::new(torrent.get_info_hash(), &Rc::new(peer_id));
                        let mut peer_0 = Peer::new(&peers[0].clone()).ok().unwrap();
                        let response = peer_0.handshake(&handshake_message).ok().unwrap();
                        println!("Received handshake: {}", &response);
                        // Handshake end
                        peer_0.receive_bitfield();
                        peer_0.send_interested();
                        peer_0.receive_unchoke();
                        //
                        let mut peer_1 = Peer::new(&peers[1].clone()).ok().unwrap();
                        let response = peer_1.handshake(&handshake_message).ok().unwrap();
                        println!("Received handshake: {}", &response);
                        // Handshake end
                        peer_1.receive_bitfield();
                        peer_1.send_interested();
                        peer_1.receive_unchoke();
                        //
                        //
                        let mut peer_2 = Peer::new(&peers[2].clone()).ok().unwrap();
                        let response = peer_2.handshake(&handshake_message).ok().unwrap();
                        println!("Received handshake: {}", &response);
                        // Handshake end
                        peer_2.receive_bitfield();
                        peer_2.send_interested();
                        peer_2.receive_unchoke();
                        //
                        let mut file = BufWriter::new(fs::File::create(out).unwrap());
                        let mut begin: u32 = 0;
                        let mut left: u64 =
                            if piece_index as usize == torrent.get_pieces().len() - 1 {
                                let residue = torrent.get_length() % torrent.get_piece_length();
                                if residue > 0 {
                                    residue
                                } else {
                                    torrent.get_piece_length()
                                }
                            } else {
                                torrent.get_piece_length()
                            };

                        let default_block_length = 16 * 1024 as u32;
                        while left > 0 {
                            let block_length: u32 = if left > default_block_length as u64 {
                                default_block_length
                            } else {
                                left as u32
                            };
                            println!("Should receive block of size {block_length} (left {left}).");
                            let mut res: Result<Box<[u8]>, Error> = Err(Error::Mock);
                            while res.is_err() {
                                println!("Getting a block from peer #0...");
                                res = peer_0.get_piece_block(piece_index, begin, block_length);
                                if res.is_err() {
                                    println!("Unable to get a block from peer #0.");
                                    peer_0 = Peer::new(&peers[0].clone()).ok().unwrap();
                                    let response =
                                        peer_0.handshake(&handshake_message).ok().unwrap();
                                    println!("Received handshake: {}", &response);
                                    // Handshake end
                                    peer_0.receive_bitfield();
                                    peer_0.send_interested();
                                    peer_0.receive_unchoke();
                                    println!("Getting a block from peer #1...");
                                    res = peer_1.get_piece_block(piece_index, begin, block_length);
                                }
                                if res.is_err() {
                                    println!("Unable to get a block from peer #1.");
                                    peer_1 = Peer::new(&peers[1].clone()).ok().unwrap();
                                    let response =
                                        peer_1.handshake(&handshake_message).ok().unwrap();
                                    println!("Received handshake: {}", &response);
                                    // Handshake end
                                    peer_1.receive_bitfield();
                                    peer_1.send_interested();
                                    peer_1.receive_unchoke();
                                    println!("Getting a block from peer #2...");
                                    res = peer_2.get_piece_block(piece_index, begin, block_length);
                                }
                                if res.is_err() {
                                    println!("Unable to get a block from peer #2.");
                                    peer_2 = Peer::new(&peers[2].clone()).ok().unwrap();
                                    let response =
                                        peer_2.handshake(&handshake_message).ok().unwrap();
                                    println!("Received handshake: {}", &response);
                                    // Handshake end
                                    peer_2.receive_bitfield();
                                    peer_2.send_interested();
                                    peer_2.receive_unchoke();
                                }
                                //     if res.is_err() {
                                //         println!("Error while getting the piece block from peer_0. Retrying with peer_1...");
                                //         peer_0.shutdown();
                                //         res = peer_1.get_piece_block(piece_index, begin, block_length);
                                //     }
                                // if res.is_err() {
                                //     println!("Error while getting the piece block from peer_1. Retrying with peer_2...");
                                //     res = peer_2.get_piece_block(piece_index, begin, block_length);
                                // }

                                // if res.is_err() {
                                //     peer_0 = Peer::new(&peers[0].clone());
                                //     let response = peer_0.handshake(&handshake_message);
                                //     println!("Received handshake: {}", &response);
                                //     // Handshake end
                                //     peer_0.receive_bitfield();
                                //     peer_0.send_interested();
                                //     peer_0.receive_unchoke();
                                //     res = peer_0.get_piece_block(piece_index, begin, block_length);
                                // }

                                // if res.is_err() {
                                //     panic!("FOR FUCK SAKE!")
                                // }
                            }
                            begin += block_length;
                            file.write_all(&res.ok().unwrap()).unwrap();
                            left -= block_length as u64;
                        }
                        file.flush().unwrap();
                    }
                    TrackerResponse::Failure(reason) => {
                        eprintln!("Got failure response from the tracker: {}", &reason);
                    }
                }
            }
            Err(err) => eprintln!("Error! -- {}", &err),
        }
    } else {
        eprintln!("unknown command: {}", args[1])
    }
}
