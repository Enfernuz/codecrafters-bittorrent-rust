use std::io;

pub type Result<T> = core::result::Result<T, Error>;

use crate::bencode::decoders;
use crate::torrent::{self, MessageTag};

#[derive(Debug)]
pub enum Error {
    DecodeError(decoders::DecodeError),
    FileError(io::Error),
    TrackerHttpError(reqwest::Error),
    KeyNotFoundInTrackerResponse {
        key: String,
    },
    TrackerIntervalIsNotInteger,
    TrackerIntervalIsNotByteString,
    TrackerFailureReasonIsNotUtf8,
    TrackerFailureInResponse {
        failure_reason: String,
    },
    InvalidMessageLength {
        minimum_length: u32,
        actual_length: u32,
    },
    MessageParsingNotImplemented(MessageTag),
    NotEnoughData {
        minimum_length: u32,
        actual_length: u32,
    },
    InvalidPeerIdLength {
        peer_id: String,
        expected_length: u8,
    },
    SocketError(io::Error),
    TorrentParseError(torrent::TorrentParseError),
    Unknown,
    UnrecognizedMessageTag(u8),
}
