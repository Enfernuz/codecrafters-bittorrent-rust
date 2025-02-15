use std::{num::ParseIntError, string::FromUtf8Error};

use bytestring_decoder::ByteStringDecodeError;
use dict_decoder::DictDecodeError;
use i64_decoder::Int64DecodeError;
use list_decoder::ListDecodeError;
use thiserror::Error;

use crate::types::DataType;

pub mod bytestring_decoder;
pub mod dict_decoder;
pub mod i64_decoder;
pub mod list_decoder;

#[derive(Error, Debug, PartialEq)]
pub enum DecodeError {
    #[error("The input is empty.")]
    EmptyInput,
    #[error("ByteString decode error")]
    ByteStringDecodeError(#[from] ByteStringDecodeError),
    #[error("Int64 decode error")]
    Int64DecodeError(#[from] Int64DecodeError),
    #[error("List decode error")]
    ListDecodeError(#[from] ListDecodeError),
    #[error("Dict decode error")]
    DictDecodeError(#[from] DictDecodeError),
    #[error("Other")]
    Other(String),
}

pub fn decode(bencoded: &[u8]) -> Result<(DataType, usize), DecodeError> {
    if let [first, ..] = bencoded {
        return match first {
            b'0'..=b'9' => bytestring_decoder::decode_byte_string(bencoded)
                .map(|(val, len)| Ok((DataType::ByteString(val), len)))?,
            b'i' => i64_decoder::decode_i64(bencoded)
                .map(|(val, len)| Ok((DataType::Integer(val), len)))?,
            b'l' => list_decoder::decode_list(bencoded)
                .map(|(val, len)| Ok((DataType::List(val.to_vec()), len)))?,
            b'd' => dict_decoder::decode_dict(bencoded)
                .map(|(val, len)| Ok((DataType::Dict(val), len)))?,
            other => Err(DecodeError::Other("TODO".to_owned())),
            // other => Err(DecodeError::InvalidEntity(format!("Unexpected byte value '{other}' (ASCII: '{}') for the start of a bencoded entity: expected it to be either '0'-'9' (which indicates the start of the length of a bencoded string), or 'i' (bencoded integer), or 'l' (bencoded list), or 'd' (bencoded dict).", *other as char))),
        };
    } else {
        return Err(DecodeError::EmptyInput);
    }
}
