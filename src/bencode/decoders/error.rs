use thiserror::Error;

use crate::bencode::decoders::bytestring_decoder;
use crate::bencode::decoders::dict_decoder;
use crate::bencode::decoders::i64_decoder;
use crate::bencode::decoders::list_decoder;

#[derive(Error, Debug, PartialEq)]
pub enum DecodeError {
    #[error("The input is empty.")]
    EmptyInput,
    #[error("ByteString decode error")]
    ByteStringDecodeError(#[from] bytestring_decoder::ByteStringDecodeError),
    #[error("Int64 decode error")]
    Int64DecodeError(#[from] i64_decoder::Int64DecodeError),
    #[error("List decode error")]
    ListDecodeError(#[from] list_decoder::ListDecodeError),
    #[error("Dict decode error")]
    DictDecodeError(#[from] dict_decoder::DictDecodeError),
    #[error("Other")]
    Other(String),
}
