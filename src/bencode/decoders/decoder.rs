pub fn decode(
    bencoded: &[u8],
) -> Result<(crate::types::data_type::DataType, usize), crate::bencode::decoders::error::DecodeError>
{
    if let [first, ..] = bencoded {
        return match first {
            b'0'..=b'9' => crate::bencode::decoders::bytestring_decoder::decode_byte_string(
                bencoded,
            )
            .map(|(val, len)| Ok((crate::types::data_type::DataType::ByteString(val), len)))?,
            b'i' => crate::bencode::decoders::i64_decoder::decode_i64(bencoded)
                .map(|(val, len)| Ok((crate::types::data_type::DataType::Integer(val), len)))?,
            b'l' => {
                crate::bencode::decoders::list_decoder::decode_list(bencoded).map(|(val, len)| {
                    Ok((crate::types::data_type::DataType::List(val.to_vec()), len))
                })?
            }
            b'd' => crate::bencode::decoders::dict_decoder::decode_dict(bencoded)
                .map(|(val, len)| Ok((crate::types::data_type::DataType::Dict(val), len)))?,
            other => Err(crate::bencode::decoders::error::DecodeError::Other(
                "TODO".to_owned(),
            )),
            // other => Err(DecodeError::InvalidEntity(format!("Unexpected byte value '{other}' (ASCII: '{}') for the start of a bencoded entity: expected it to be either '0'-'9' (which indicates the start of the length of a bencoded string), or 'i' (bencoded integer), or 'l' (bencoded list), or 'd' (bencoded dict).", *other as char))),
        };
    } else {
        return Err(crate::bencode::decoders::error::DecodeError::EmptyInput);
    }
}
