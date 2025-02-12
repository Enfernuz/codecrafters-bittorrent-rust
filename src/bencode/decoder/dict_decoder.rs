use std::{collections::BTreeMap, rc::Rc};

use crate::{bytestring_decoder, types::DataType};

use super::DecodeError;

pub fn decode_dict(bencoded: &[u8]) -> Result<(BTreeMap<String, DataType>, usize), DecodeError> {
    //dbg!("decode_dict: {:?}", bencoded);

    if let [start, ..] = bencoded {
        if *start != b'd' {
            Err(DecodeError::new(&format!(
                "Bencoded dicts must start with 'd' but got '{}' (ASCII: '{}').",
                *start, *start as char
            )))?
        }

        let mut dict: BTreeMap<String, DataType> = BTreeMap::new();
        let mut pos: usize = 1;
        let mut end_of_dict_found = false;
        while pos < bencoded.len() {
            match bencoded[pos] {
                b'e' => {
                    end_of_dict_found = true;
                    pos += 1;
                    break;
                },
                b'0'..=b'9' => {
                    let (key, key_bytes_processed) = bytestring_decoder::decode_byte_string(&bencoded[pos..])?;
                    let key_str = String::from_utf8(key.get_data().to_vec())
                        .map_err(|err| DecodeError::new(&format!("Could not convert bencoded dict key at position {pos} to a UTF-8 string.")))?;
                    pos += key_bytes_processed;
                    let (value, value_bytes_processed) = super::decode(&bencoded[pos..]).map_err(|err| DecodeError::new(&format!("Invalid byte at position {pos}: {}", err.get_message())))?;
                    pos += value_bytes_processed;
                    dict.insert(key_str, value);
                },
                other => return Err(DecodeError::new(&format!("Unexpected byte value '{other}' (ASCII: '{}') at position {pos}: bencoded dict keys must be bencoded strings beginning with a number (consisting of numeric characters from 0 to 9) indicating the length of the bencoded string and preceeding a colon character (':').", other as char))),
            }
        }

        if !end_of_dict_found {
            Err(DecodeError::new("End of dict ('e') not found.".into()))?
        }

        Ok((dict, pos))
    } else {
        Err(DecodeError::new("The input is empty."))?
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
        assert_eq!(result.is_empty(), true);
    }

    #[test]
    fn test_dict_number() {
        let (result, bytes_processed) = decode_dict("d5:helloi123ee".as_bytes()).unwrap();
        assert_eq!(bytes_processed, 14);
        assert_eq!(
            result,
            BTreeMap::from_iter([("hello".into(), DataType::Integer(123))])
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
                DataType::ByteString(ByteString::new(&"username".as_bytes().into()))
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
                DataType::List(vec![
                    DataType::ByteString(ByteString::new(&"Asia".as_bytes().into())),
                    DataType::ByteString(ByteString::new(&"Europe".as_bytes().into())),
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
                    DataType::ByteString(ByteString::new(&"username".as_bytes().into()))
                ),
                ("age".into(), DataType::Integer(42))
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
                    DataType::ByteString(ByteString::new(&"username".as_bytes().into()))
                ),
                ("age".into(), DataType::Integer(42),),
                (
                    "passwords".into(),
                    DataType::List(vec![
                        DataType::ByteString(ByteString::new(&"abc".as_bytes().into())),
                        DataType::Integer(12345)
                    ])
                )
            ])
        );
    }
}
