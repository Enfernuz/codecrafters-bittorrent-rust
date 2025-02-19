use thiserror::Error;

#[derive(Error, Debug, PartialEq)]
pub enum DecodeError {
    #[error("The input is empty.")]
    EmptyInput,
    #[error("ByteString decode error")]
    ByteStringDecodeError(
        #[from] crate::bencode::decoders::bytestring_decoder::ByteStringDecodeError,
    ),
    #[error("Int64 decode error")]
    Int64DecodeError(#[from] crate::bencode::decoders::i64_decoder::Int64DecodeError),
    #[error("List decode error")]
    ListDecodeError(#[from] crate::bencode::decoders::list_decoder::ListDecodeError),
    #[error("Dict decode error")]
    DictDecodeError(#[from] crate::bencode::decoders::dict_decoder::DictDecodeError),
    #[error("Other")]
    Other(String),
}
