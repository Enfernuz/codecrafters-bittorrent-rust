use std::{collections::HashMap, fmt, rc::Rc};

use sha1::{Digest, Sha1};

use crate::{
    bencode::{decoder, encoder},
    types::{byte_string::ByteString, DataType},
};

pub struct Torrent {
    announce: String,
    length: u64,
    info_hash: String,
    piece_length: u64,
    pieces: Rc<[String]>,
}

impl Torrent {
    pub fn get_announce(&self) -> &str {
        &self.announce
    }

    pub fn get_length(&self) -> u64 {
        self.length
    }

    pub fn get_info_hash(&self) -> &str {
        &self.info_hash
    }

    pub fn get_piece_length(&self) -> u64 {
        self.piece_length
    }

    pub fn get_pieces(&self) -> &Rc<[String]> {
        &self.pieces
    }
}

impl TryFrom<&[u8]> for Torrent {
    type Error = String;

    fn try_from(data: &[u8]) -> Result<Self, Self::Error> {
        let (decoded, _) = decoder::decode(data).map_err(|err| err.get_message().to_owned())?;
        Torrent::try_from(&decoded)
    }
}

impl TryFrom<&DataType> for Torrent {
    type Error = String;

    fn try_from(data: &DataType) -> Result<Self, Self::Error> {
        match data {
            DataType::Dict(v) => {
                let mut map: HashMap<String, DataType> = HashMap::new();
                for (key, value) in v {
                    map.insert(key.clone(), value.clone());
                }
                let announce = map
                    .get("announce")
                    .ok_or_else(|| "Could not find the 'announce' key.")?
                    .as_str()
                    .ok_or_else(|| {
                        "Could not convert the value for the 'announce' key into a string."
                    })?;
                let info = map
                    .get("info")
                    .ok_or_else(|| "Could not find the 'info' key.")?;
                let info_as_dict = info
                    .as_dict()
                    .ok_or_else(|| "Could not convert the value for the 'info' key into a dict.")?;
                let length: u64 = info_as_dict
                    .get("length")
                    .ok_or_else(|| "Could not find the 'length' key.")?
                    .as_i64()
                    .ok_or_else(|| "Could not convert the value of the 'length' key to i64.")?
                    as u64;
                let info_bencoded: &[u8] = &encoder::bencode(info);
                let mut hasher = Sha1::new();
                hasher.update(&info_bencoded);
                let sha1_hash = hasher.finalize();
                let info_hash: String = hex::encode(&sha1_hash.as_slice());
                let piece_length: u64 = info_as_dict
                    .get("piece length")
                    .ok_or_else(|| "Could not find the 'length' key.")?
                    .as_i64()
                    .ok_or_else(|| {
                        "Could not convert the value of the 'piece length' key into i64."
                    })? as u64;
                let pieces_byte_string: &ByteString = info_as_dict
                    .get("pieces")
                    .ok_or_else(|| "Could not find the 'pieces' key.")?
                    .as_byte_string()
                    .ok_or_else(|| {
                        "Could not convert the value of the 'pieces' key into a byte string."
                    })?;
                let pieces = parse_pieces(pieces_byte_string);
                Ok(Torrent {
                    announce,
                    length,
                    info_hash,
                    piece_length,
                    pieces,
                })
            }
            other => Err(format!(
                "Could not convert into a Torrent. Only dicts are supported, but got {:?}",
                other
            )),
        }
    }
}

impl fmt::Display for Torrent {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "Tracker URL: {}\nLength: {}\nInfo Hash: {}\nPiece Length: {}\nPiece Hashes:\n{}",
            &self.announce,
            self.length,
            self.info_hash,
            self.piece_length,
            self.pieces.join("\n")
        )
    }
}

fn parse_pieces(byte_str: &ByteString) -> Rc<[String]> {
    let mut result: Vec<String> = vec![];
    // TODO: replace with an actual error
    assert!(
        byte_str.get_data().len() % 20 == 0,
        "Invalid 'pieces' value: the length of the byte string is {} and is not a multiple of 20.",
        byte_str.get_data().len()
    );
    for chunk in byte_str.get_data().chunks_exact(20) {
        let hex_hash: String = hex::encode(chunk);
        result.push(hex_hash);
    }
    result.into()
}
