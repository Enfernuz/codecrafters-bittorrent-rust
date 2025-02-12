use crate::types::DataType;

pub mod bytestring_decoder;
pub mod dict_decoder;
pub mod i64_decoder;
pub mod list_decoder;

#[derive(Debug)]
pub struct DecodeError {
    msg: String,
}

impl DecodeError {
    pub fn new(msg: &str) -> DecodeError {
        DecodeError {
            msg: msg.to_owned(),
        }
    }

    pub fn get_message(&self) -> &str {
        &self.msg
    }
}

pub fn decode(bencoded: &[u8]) -> Result<(DataType, usize), DecodeError> {
    if let [first, ..] = bencoded {
        return match first {
            b'0'..=b'9' => bytestring_decoder::decode_byte_string(bencoded).map(|(val, len)| (DataType::ByteString(val), len)),
            b'i' => i64_decoder::decode_i64(bencoded).map(|(val, len)| (DataType::Integer(val), len)),
            b'l' => list_decoder::decode_list(bencoded).map(|(val, len)| (DataType::List(val.to_vec()), len)),
            b'd' => dict_decoder::decode_dict(bencoded).map(|(val, len)| (DataType::Dict(val), len)),
            other => Err(DecodeError::new(&format!("Unexpected byte value '{other}' (ASCII: '{}') for the start of a bencoded entity: expected it to be either '0'-'9' (which indicates the start of the length of a bencoded string), or 'i' (bencoded integer), or 'l' (bencoded list), or 'd' (bencoded dict).", *other as char))),
        };
    } else {
        Err(DecodeError::new("The input is empty."))?
    }
}
