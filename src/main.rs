mod bencode;
mod torrent;
mod types;

use std::io::{Read, Write};
use std::net::TcpStream;
use std::rc::Rc;
use std::{env, fs};

use bencode::decoder;
use torrent::peer::HandshakeMessage;
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
                let mut peer = Peer::new(addr);
                let response = peer.handshake(&handshake_message);
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
                        let addr = &peers[0].clone();
                        // Handshake start
                        let peer_id: [u8; 20] = *b"12345678901234567890";
                        let handshake_message =
                            HandshakeMessage::new(torrent.get_info_hash(), &Rc::new(peer_id));
                        let mut peer = Peer::new(addr);
                        let response = peer.handshake(&handshake_message);
                        println!("Received handshake: {}", &response);
                        // Handshake end
                        peer.receive_bitfield();
                        peer.send_interested();
                        peer.receive_unchoke();
                        let mut file = fs::File::create(out).unwrap();
                        let mut begin: u32 = 0;
                        let mut left: u64 = torrent.get_piece_length();
                        let default_block_length = 16 * 1024 as u32;
                        while left > 0 {
                            let block_length: u32 = if left > default_block_length as u64 {
                                default_block_length
                            } else {
                                left as u32
                            };
                            println!("Should receive block of size {block_length} (left {left}).");
                            peer.send_piece_request(piece_index, begin, block_length);
                            begin += block_length;
                            let recv = peer.receive_piece_block(block_length);
                            file.write_all(recv.as_ref()).unwrap();
                            left -= block_length as u64;
                        }
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
