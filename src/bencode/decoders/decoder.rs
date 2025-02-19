use crate::bencode::decoders;
use crate::types::DataType;

pub fn decode(bencoded: &[u8]) -> Result<(DataType, usize), decoders::DecodeError> {
    if let [first, ..] = bencoded {
        return match first {
            b'0'..=b'9' => decoders::decode_byte_string(bencoded)
                .map(|(val, len)| Ok((DataType::ByteString(val), len)))?,
            b'i' => decoders::decode_i64(bencoded)
                .map(|(val, len)| Ok((DataType::Integer(val), len)))?,
            b'l' => decoders::decode_list(bencoded)
                .map(|(val, len)| Ok((DataType::List(val.to_vec()), len)))?,
            b'd' => {
                decoders::decode_dict(bencoded).map(|(val, len)| Ok((DataType::Dict(val), len)))?
            }
            other => Err(decoders::DecodeError::Other("TODO".to_owned())),
            // other => Err(DecodeError::InvalidEntity(format!("Unexpected byte value '{other}' (ASCII: '{}') for the start of a bencoded entity: expected it to be either '0'-'9' (which indicates the start of the length of a bencoded string), or 'i' (bencoded integer), or 'l' (bencoded list), or 'd' (bencoded dict).", *other as char))),
        };
    } else {
        return Err(decoders::DecodeError::EmptyInput);
    }
}
