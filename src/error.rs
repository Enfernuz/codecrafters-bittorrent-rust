use std::io;

pub type Result<T> = core::result::Result<T, Error>;

use crate::torrent::MessageTag;

pub enum Error {
    UnrecognizedMessageTag(u8),
    NotEnoughData {
        minimum_length: u32,
        actual_length: u32,
    },
    InvalidMessageLength {
        minimum_length: u32,
        actual_length: u32,
    },
    MessageParsingNotImplemented(MessageTag),
    SocketError(io::Error),
    Mock,
}
