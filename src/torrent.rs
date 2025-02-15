pub mod peer;
pub mod tracker;

use std::{collections::HashMap, fmt, rc::Rc};

use sha1::{Digest, Sha1};
use thiserror::Error;

use crate::{
    bencode::{
        decoder::{self, DecodeError},
        encoder,
    },
    types::{byte_string::ByteString, DataType},
};

type Error = TorrentParseError;
type Result<T> = core::result::Result<T, Error>;

#[derive(Error, Debug, PartialEq)]
pub enum TorrentParseError {
    #[error("Decode error")]
    DecodeError(#[from] DecodeError),
    #[error("Invalid meta info")]
    InvalidMetaInfo(String),
}

pub struct Torrent {
    announce: String,
    length: u64,
    info_hash: Rc<[u8; 20]>,
    piece_length: u64,
    pieces: Rc<[[u8; 20]]>,
}

impl Torrent {
    pub fn get_announce(&self) -> &str {
        &self.announce
    }

    pub fn get_length(&self) -> u64 {
        self.length
    }

    pub fn get_info_hash(&self) -> &Rc<[u8; 20]> {
        &self.info_hash
    }

    pub fn get_piece_length(&self) -> u64 {
        self.piece_length
    }

    pub fn get_pieces(&self) -> &Rc<[[u8; 20]]> {
        &self.pieces
    }
}

impl TryFrom<&[u8]> for Torrent {
    type Error = Error;

    fn try_from(data: &[u8]) -> Result<Self> {
        let (decoded, _) = decoder::decode(data)?;
        Torrent::try_from(&decoded)
    }
}

impl TryFrom<&DataType> for Torrent {
    type Error = Error;

    fn try_from(data: &DataType) -> Result<Self> {
        match data {
            DataType::Dict(v) => {
                let mut map: HashMap<String, DataType> = HashMap::new();
                for (key, value) in v {
                    map.insert(key.clone(), value.clone());
                }
                let announce = map
                    .get("announce")
                    .ok_or_else(|| {
                        TorrentParseError::InvalidMetaInfo(
                            "Could not find the 'announce' key.".to_owned(),
                        )
                    })?
                    .as_str()
                    .ok_or_else(|| {
                        TorrentParseError::InvalidMetaInfo(
                            "Could not convert the value for the 'announce' key into a string."
                                .to_owned(),
                        )
                    })?;
                let info = map.get("info").ok_or_else(|| {
                    TorrentParseError::InvalidMetaInfo("Could not find the 'info' key.".to_owned())
                })?;
                let info_as_dict = info.as_dict().ok_or_else(|| {
                    TorrentParseError::InvalidMetaInfo(
                        "Could not convert the value for the 'info' key into a dict.".to_owned(),
                    )
                })?;
                let length: u64 = info_as_dict
                    .get("length")
                    .ok_or_else(|| {
                        TorrentParseError::InvalidMetaInfo(
                            "Could not find the 'length' key.".to_owned(),
                        )
                    })?
                    .as_i64()
                    .ok_or_else(|| {
                        TorrentParseError::InvalidMetaInfo(
                            "Could not convert the value of the 'length' key to i64.".to_owned(),
                        )
                    })? as u64;
                let info_bencoded: &[u8] = &encoder::bencode(info);
                let mut hasher = Sha1::new();
                hasher.update(info_bencoded);
                let sha1_hash = hasher.finalize();
                let piece_length: u64 = info_as_dict
                    .get("piece length")
                    .ok_or_else(|| {
                        TorrentParseError::InvalidMetaInfo(
                            "Could not find the 'length' key.".to_owned(),
                        )
                    })?
                    .as_i64()
                    .ok_or_else(|| {
                        TorrentParseError::InvalidMetaInfo(
                            "Could not convert the value of the 'piece length' key into i64."
                                .to_owned(),
                        )
                    })? as u64;
                let pieces_byte_string: &ByteString = info_as_dict
                    .get("pieces")
                    .ok_or_else(|| {
                        TorrentParseError::InvalidMetaInfo(
                            "Could not find the 'pieces' key.".to_owned(),
                        )
                    })?
                    .as_byte_string()
                    .ok_or_else(|| {
                        TorrentParseError::InvalidMetaInfo(
                            "Could not convert the value of the 'pieces' key into a byte string."
                                .to_owned(),
                        )
                    })?;
                let pieces = parse_pieces(pieces_byte_string);
                Ok(Torrent {
                    announce,
                    length,
                    info_hash: Rc::new(sha1_hash.into()),
                    piece_length,
                    pieces,
                })
            }
            other => Err(TorrentParseError::InvalidMetaInfo(format!(
                "Could not convert into a Torrent. Only dicts are supported, but got {:?}",
                other
            ))),
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
            hex::encode(self.get_info_hash().as_ref()),
            self.piece_length,
            self.pieces
                .iter()
                .map(|piece| hex::encode(piece))
                .collect::<Vec<String>>()
                .join("\n")
        )
    }
}

fn parse_pieces(byte_str: &ByteString) -> Rc<[[u8; 20]]> {
    let mut pieces: Vec<[u8; 20]> = vec![];
    // TODO: replace with an actual error
    assert!(
        byte_str.get_data().len() % 20 == 0,
        "Invalid 'pieces' value: the length of the byte string is {} and is not a multiple of 20.",
        byte_str.get_data().len()
    );
    for chunk in byte_str.get_data().chunks_exact(20) {
        let mut piece: [u8; 20] = [0; 20];
        // let hex_hash: String = hex::encode();
        piece.copy_from_slice(chunk);
        pieces.push(piece);
    }
    pieces.into()
}
