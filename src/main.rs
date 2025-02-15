mod bencode;
mod torrent;
mod types;

use std::io::{Read, Write};
use std::net::TcpStream;
use std::rc::Rc;
use std::{env, fs};

use bencode::decoder;
use torrent::peer::HandshakeMessage;
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
                let peer_id: [u8; 20] = "12345678901234567890".as_bytes()[0..20]
                    .try_into()
                    .expect("Could not parse 20-byte long peer_id");
                let handshake_message =
                    HandshakeMessage::new(torrent.get_info_hash(), &Rc::new(peer_id));
                let mut stream =
                    TcpStream::connect(addr).expect(&format!("Could not connect to {}", addr));
                stream
                    .write(handshake_message.as_bytes().as_ref())
                    .expect(&format!("Could not write to TCP socket for {}", addr));
                let mut buf: [u8; 68] = [0; 68];
                stream
                    .read(&mut buf)
                    .expect(&format!("Could not read from TCP socket for {}", addr));
                let response = HandshakeMessage::parse(&buf);
                println!("{}", &response);
            }
            Err(err) => {
                eprintln!("Got failure response from the peer: {}", &err);
            }
        }
    } else {
        eprintln!("unknown command: {}", args[1])
    }
}
