// mod bencode;
mod torrent;
mod types;

mod bencode;

use std::{env, fs};

use bencode::decoder;
use torrent::Torrent;

pub use crate::bencode::decoder::bytestring_decoder;

// Usage: your_bittorrent.sh decode "<encoded_value>"
fn main() {
    let args: Vec<String> = env::args().collect();
    let command = &args[1];

    if command == "decode" {
        let encoded_value = &args[2];
        let res = decoder::decode(encoded_value.as_bytes());
        match res {
            Ok((decoded, length)) => {
                let json: serde_json::Value = decoded.into();
                println!("{}", &json)
            }
            Err(err) => panic!("{}", err.get_message()),
        }
    } else if command == "info" {
        let path = &args[2];
        let torrent: Result<Torrent, std::io::Error> =
            fs::read(path).map(|s| s.as_slice().try_into().ok().unwrap());
        match torrent {
            Ok(t) => println!("{}", &t),
            Err(err) => println!("Error! -- {}", &err),
        }
    } else {
        println!("unknown command: {}", args[1])
    }
}
