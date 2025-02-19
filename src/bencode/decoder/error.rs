use thiserror::Error;

#[derive(Error, Debug, PartialEq)]
pub enum DecodeError {
    #[error("The input is empty.")]
    EmptyInput,
    #[error("ByteString decode error")]
    ByteStringDecodeError(
        #[from] crate::bencode::decoder::bytestring_decoder::ByteStringDecodeError,
    ),
    #[error("Int64 decode error")]
    Int64DecodeError(#[from] crate::bencode::decoder::i64_decoder::Int64DecodeError),
    #[error("List decode error")]
    ListDecodeError(#[from] crate::bencode::decoder::list_decoder::ListDecodeError),
    #[error("Dict decode error")]
    DictDecodeError(#[from] crate::bencode::decoder::dict_decoder::DictDecodeError),
    #[error("Other")]
    Other(String),
}
