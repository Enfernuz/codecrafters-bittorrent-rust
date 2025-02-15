mod bencode;
mod torrent;
mod types;

use std::{env, fs};

use bencode::decoder;
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
    } else {
        eprintln!("unknown command: {}", args[1])
    }
}
