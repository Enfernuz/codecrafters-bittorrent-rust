use std::{collections::BTreeMap, string::FromUtf8Error};
use thiserror::Error;

#[derive(Error, Debug, PartialEq)]
pub enum DictDecodeError {
    #[error("The input is empty.")]
    EmptyInput,
    #[error(
        "Bencoded dicts must start with 'd' but got '{found:?}' (ASCII: '{found_as_ascii:?}')."
    )]
    StartNotFound { found: u8, found_as_ascii: char },
    #[error("The end of dict ('e') not found.")]
    EndNotFound,
    #[error("Could not decode bencoded key.")]
    KeyDecodeError(#[from] crate::bencode::decoder::bytestring_decoder::ByteStringDecodeError),
    #[error("Could not parse bencoded key as UTF-8.")]
    KeyUtf8ParseError(#[from] FromUtf8Error),
    #[error("Could not decode value at {position}")]
    ValueDecodeError { position: usize },
}

pub fn decode_dict(
    bencoded: &[u8],
) -> Result<(BTreeMap<String, crate::types::data_type::DataType>, usize), DictDecodeError> {
    if let [start, ..] = bencoded {
        if *start != b'd' {
            return Err(DictDecodeError::StartNotFound {
                found: *start,
                found_as_ascii: *start as char,
            });
        }

        let mut dict: BTreeMap<String, crate::types::data_type::DataType> = BTreeMap::new();
        let mut pos: usize = 1;
        let mut end_of_dict_found = false;
        while pos < bencoded.len() {
            match bencoded[pos] {
                b'e' => {
                    end_of_dict_found = true;
                    pos += 1;
                    break;
                }
                b'0'..=b'9' => {
                    let (key, key_bytes_processed) =
                        crate::bencode::decoder::bytestring_decoder::decode_byte_string(&bencoded[pos..])?;
                    let key_str = String::from_utf8(key.get_data().to_vec())?;
                    pos += key_bytes_processed;
                    let (value, value_bytes_processed) = crate::bencode::decoder::decoder::decode(&bencoded[pos..])
                        .map_err(|err| DictDecodeError::ValueDecodeError { position: pos })?;
                    pos += value_bytes_processed;
                    dict.insert(key_str, value);
                }
                other => {
                    return Err(DictDecodeError::KeyDecodeError(
                        crate::bencode::decoder::bytestring_decoder::ByteStringDecodeError::UnexpectedByte {
                            unexpected_byte: other,
                            unexpected_byte_ascii: other as char,
                            position: pos,
                        },
                    ))
                }
            }
        }

        if !end_of_dict_found {
            return Err(DictDecodeError::EndNotFound);
        }

        Ok((dict, pos))
    } else {
        return Err(DictDecodeError::EmptyInput);
    }
}

#[cfg(test)]
mod tests {
    use crate::types::byte_string::ByteString;

    use super::*;

    #[test]
    fn test_empty_dict() {
        let (result, bytes_processed) = decode_dict("de".as_bytes()).unwrap();
        assert_eq!(bytes_processed, 2);
        assert!(result.is_empty());
    }

    #[test]
    fn test_dict_number() {
        let (result, bytes_processed) = decode_dict("d5:helloi123ee".as_bytes()).unwrap();
        assert_eq!(bytes_processed, 14);
        assert_eq!(
            result,
            BTreeMap::from_iter([(
                "hello".into(),
                crate::types::data_type::DataType::Integer(123)
            )])
        );
    }

    #[test]
    fn test_dict_string() {
        let (result, bytes_processed) = decode_dict("d5:hello8:usernamee".as_bytes()).unwrap();
        assert_eq!(bytes_processed, 19);
        assert_eq!(
            result,
            BTreeMap::from_iter([(
                "hello".into(),
                crate::types::data_type::DataType::ByteString(ByteString::new(
                    &"username".as_bytes().into()
                ))
            )])
        );
    }

    #[test]
    fn test_dict_list() {
        let (result, bytes_processed) =
            decode_dict("d5:worldl4:Asia6:Europeee".as_bytes()).unwrap();
        assert_eq!(bytes_processed, 25);
        assert_eq!(
            result,
            BTreeMap::from_iter([(
                "world".into(),
                crate::types::data_type::DataType::List(vec![
                    crate::types::data_type::DataType::ByteString(ByteString::new(
                        &"Asia".as_bytes().into()
                    )),
                    crate::types::data_type::DataType::ByteString(ByteString::new(
                        &"Europe".as_bytes().into()
                    )),
                ])
            )])
        );
    }

    #[test]
    fn test_dict_string_2_entries() {
        let (result, bytes_processed) =
            decode_dict("d5:hello8:username3:agei42ee".as_bytes()).unwrap();
        assert_eq!(bytes_processed, 28);
        assert_eq!(
            result,
            BTreeMap::from_iter([
                (
                    "hello".into(),
                    crate::types::data_type::DataType::ByteString(ByteString::new(
                        &"username".as_bytes().into()
                    ))
                ),
                ("age".into(), crate::types::data_type::DataType::Integer(42))
            ])
        );
    }

    #[test]
    fn test_dict_string_3_entries() {
        let (result, bytes_processed) =
            // hello: username, age: 42, passwords: [abc, 12345]
            decode_dict("d5:hello8:username3:agei42e9:passwordsl3:abci12345eee".as_bytes()).unwrap();
        assert_eq!(bytes_processed, 53);
        assert_eq!(
            result,
            BTreeMap::from_iter([
                (
                    "hello".into(),
                    crate::types::data_type::DataType::ByteString(ByteString::new(
                        &"username".as_bytes().into()
                    ))
                ),
                ("age".into(), crate::types::data_type::DataType::Integer(42),),
                (
                    "passwords".into(),
                    crate::types::data_type::DataType::List(vec![
                        crate::types::data_type::DataType::ByteString(ByteString::new(
                            &"abc".as_bytes().into()
                        )),
                        crate::types::data_type::DataType::Integer(12345)
                    ])
                )
            ])
        );
    }
}
