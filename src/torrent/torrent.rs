use std::sync::Arc;
use std::{collections::HashMap, fmt};

use sha1::{Digest, Sha1};

use crate::bencode::decoders;
use crate::bencode::encoders;
use crate::error::Error;
use crate::error::Result;
use crate::torrent::Block;
use crate::torrent::Piece;
use crate::types::ByteString;
use crate::types::DataType;

const DEFAULT_BLOCK_SIZE: u32 = 16 * 1024; // 16 KB

// region:      --- Torrent
pub struct Torrent {
    announce: String,
    length: u64,
    info_hash: [u8; 20],
    piece_length: u32,
    pieces: Arc<[Piece]>,
}

// region:      ---Getters
impl Torrent {
    pub fn get_announce(&self) -> &str {
        &self.announce
    }

    pub fn get_length(&self) -> u64 {
        self.length
    }

    pub fn get_info_hash(&self) -> &[u8; 20] {
        &self.info_hash
    }

    pub fn get_piece_length(&self) -> u32 {
        self.piece_length
    }

    pub fn get_pieces(&self) -> Arc<[Piece]> {
        Arc::clone(&self.pieces)
    }
}

impl TryFrom<&[u8]> for Torrent {
    type Error = Error;

    fn try_from(data: &[u8]) -> Result<Self> {
        let (decoded, _) = decoders::decode(data).map_err(|err| Error::DecodeError(err))?;
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
                        Error::TorrentParseError("Could not find the 'announce' key.".to_owned())
                    })?
                    .as_str()
                    .ok_or_else(|| {
                        Error::TorrentParseError(
                            "Could not convert the value for the 'announce' key into a string."
                                .to_owned(),
                        )
                    })?;
                let info = map.get("info").ok_or_else(|| {
                    Error::TorrentParseError("Could not find the 'info' key.".to_owned())
                })?;
                let info_as_dict = info.as_dict().ok_or_else(|| {
                    Error::TorrentParseError(
                        "Could not convert the value for the 'info' key into a dict.".to_owned(),
                    )
                })?;
                let length: u64 = info_as_dict
                    .get("length")
                    .ok_or_else(|| {
                        Error::TorrentParseError("Could not find the 'length' key.".to_owned())
                    })?
                    .as_i64()
                    .ok_or_else(|| {
                        Error::TorrentParseError(
                            "Could not convert the value of the 'length' key to i64.".to_owned(),
                        )
                    })? as u64;
                let info_bencoded: &[u8] = &encoders::bencode(info);
                let mut hasher = Sha1::new();
                hasher.update(info_bencoded);
                let sha1_hash = hasher.finalize();
                let piece_length: u32 = info_as_dict
                    .get("piece length")
                    .ok_or_else(|| {
                        Error::TorrentParseError("Could not find the 'length' key.".to_owned())
                    })?
                    .as_i64()
                    .ok_or_else(|| {
                        Error::TorrentParseError(
                            "Could not convert the value of the 'piece length' key into i64."
                                .to_owned(),
                        )
                    })? as u32;
                let pieces_byte_string: &ByteString = info_as_dict
                    .get("pieces")
                    .ok_or_else(|| {
                        Error::TorrentParseError("Could not find the 'pieces' key.".to_owned())
                    })?
                    .as_byte_string()
                    .ok_or_else(|| {
                        Error::TorrentParseError(
                            "Could not convert the value of the 'pieces' key into a byte string."
                                .to_owned(),
                        )
                    })?;
                let pieces = parse_pieces(pieces_byte_string, length, piece_length);
                Ok(Torrent {
                    announce,
                    length,
                    info_hash: sha1_hash.into(),
                    piece_length,
                    pieces,
                })
            }
            other => Err(Error::TorrentParseError(format!(
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
                .map(|piece| hex::encode(piece.get_hash()))
                .collect::<Vec<String>>()
                .join("\n")
        )
    }
}

fn parse_pieces(byte_str: &ByteString, torrent_length: u64, piece_length: u32) -> Arc<[Piece]> {
    let mut hashes: Vec<[u8; 20]> = vec![];
    // TODO: replace with an actual error
    assert!(
        byte_str.get_data().len() % 20 == 0,
        "Invalid 'pieces' value: the length of the byte string is {} and is not a multiple of 20.",
        byte_str.get_data().len()
    );
    for chunk in byte_str.get_data().chunks_exact(20) {
        let mut hash: [u8; 20] = [0; 20];
        // let hex_hash: String = hex::encode();
        hash.copy_from_slice(chunk);
        hashes.push(hash);
    }

    let mut pieces: Vec<Piece> = Vec::with_capacity(hashes.len());
    let mut position: u64 = 0;
    for (index, hash) in hashes.iter().enumerate() {
        let is_last_block = index == hashes.len() - 1;
        let piece_length = if is_last_block {
            get_last_piece_length(torrent_length, piece_length)
        } else {
            piece_length
        };
        let blocks_count: u32 = piece_length / DEFAULT_BLOCK_SIZE;
        let residue: u32 = piece_length % DEFAULT_BLOCK_SIZE;
        let mut blocks: Vec<Block> =
            Vec::with_capacity(blocks_count as usize + if residue > 0 { 1 } else { 0 });
        for i in 0..blocks_count {
            blocks.push(Block::new(i * DEFAULT_BLOCK_SIZE, DEFAULT_BLOCK_SIZE));
        }
        if residue > 0 {
            blocks.push(Block::new(blocks_count * DEFAULT_BLOCK_SIZE, residue));
        }
        pieces.push(Piece::new(
            index as u32,
            *hash,
            blocks.into_boxed_slice(),
            position,
        ));
        position += piece_length as u64;
    }

    let sum: u32 = pieces.iter().map(|p| p.get_length()).sum();
    println!(
        "get_pieces: hashes = {}, pieces = {}, torrent_length = {}, pieces_sum = {}",
        hashes.len(),
        pieces.len(),
        torrent_length,
        sum
    );

    pieces.into()
}

fn get_last_piece_length(torrent_length: u64, piece_length: u32) -> u32 {
    let residue = torrent_length % piece_length as u64;
    if residue > 0 {
        residue as u32
    } else {
        piece_length
    }
}

// endregion:   ---Torrent
