use std::rc::Rc;

use thiserror::Error;

use crate::bencode::decoders;
use crate::types::DataType;

#[derive(Error, Debug, PartialEq)]
pub enum ListDecodeError {
    #[error("The input is empty.")]
    EmptyInput,
    #[error(
        "Bencoded lists must start with 'l' but got '{found:?}' (ASCII: '{found_as_ascii:?}')."
    )]
    StartNotFound { found: u8, found_as_ascii: char },
    #[error("The end of list ('e') not found.")]
    EndNotFound,
    #[error("Could not decode the element at {position}.")]
    ElementDecodeError { position: usize },
}

pub fn decode_list(bencoded: &[u8]) -> Result<(Rc<[DataType]>, usize), ListDecodeError> {
    if let [start, ..] = bencoded {
        if *start != b'l' {
            return Err(ListDecodeError::StartNotFound {
                found: *start,
                found_as_ascii: *start as char,
            });
        }

        let mut decoded_elements: Vec<DataType> = vec![];
        let mut pos: usize = 1;
        let mut end_of_list_found = false;
        while pos < bencoded.len() {
            match bencoded[pos] {
                b'e' => {
                    end_of_list_found = true;
                    pos += 1;
                    break;
                }
                _ => {
                    let (decoded_element, bytes_processed) = decoders::decode(&bencoded[pos..])
                        .map_err(|err| ListDecodeError::ElementDecodeError { position: pos })?;
                    decoded_elements.push(decoded_element);
                    pos += bytes_processed;
                }
            }
        }

        if !end_of_list_found {
            return Err(ListDecodeError::EndNotFound);
        }

        Ok((decoded_elements.into(), pos))
    } else {
        return Err(ListDecodeError::EmptyInput);
    }
}

#[cfg(test)]
mod tests {
    use crate::types::ByteString;

    use super::*;

    #[test]
    fn test_empty_list() {
        let (result, bytes_processed) = decode_list("le".as_bytes()).unwrap();
        assert_eq!(bytes_processed, 2);
        assert_eq!(result.is_empty(), true);
    }

    #[test]
    fn test_list_1_number() {
        let (result, bytes_processed) = decode_list("li12345ee".as_bytes()).unwrap();
        assert_eq!(bytes_processed, 9);
        assert_eq!(result.len(), 1);
        assert_eq!(result[0], DataType::Integer(12345));
    }

    #[test]
    fn test_list_2_numbers() {
        let (result, bytes_processed) = decode_list("li12345ei-100500ee".as_bytes()).unwrap();
        assert_eq!(bytes_processed, 18);
        assert_eq!(result.len(), 2);
        assert_eq!(result[0], DataType::Integer(12345));
        assert_eq!(result[1], DataType::Integer(-100500));
    }

    #[test]
    fn test_list_number_and_string() {
        let (result, bytes_processed) = decode_list("li12345e5:helloe".as_bytes()).unwrap();
        assert_eq!(bytes_processed, 16);
        assert_eq!(result.len(), 2);
        assert_eq!(result[0], DataType::Integer(12345));
        assert_eq!(
            result[1],
            DataType::ByteString(ByteString::new(&"hello".as_bytes().into()))
        );
    }

    #[test]
    fn test_list_string_and_number() {
        let (result, bytes_processed) = decode_list("l5:helloi12345ee".as_bytes()).unwrap();
        assert_eq!(bytes_processed, 16);
        assert_eq!(result.len(), 2);
        assert_eq!(
            result[0],
            DataType::ByteString(ByteString::new(&"hello".as_bytes().into()))
        );
        assert_eq!(result[1], DataType::Integer(12345));
    }

    #[test]
    fn test_nested_list_both_empty() {
        let (result, bytes_processed) = decode_list("llee".as_bytes()).unwrap();
        assert_eq!(bytes_processed, 4);
        assert_eq!(result.len(), 1);
        assert_eq!(result[0], DataType::List(vec![]));
    }

    #[test]
    fn test_nested_list_with_numbers() {
        let (result, bytes_processed) = decode_list("lli456ei-10eee".as_bytes()).unwrap();
        assert_eq!(bytes_processed, 14);
        assert_eq!(result.len(), 1);
        assert_eq!(
            result[0],
            DataType::List(vec![DataType::Integer(456), DataType::Integer(-10)])
        );
    }

    #[test]
    fn test_nested_list_with_strings() {
        let (result, bytes_processed) = decode_list("ll7:Hello, 6:World!ee".as_bytes()).unwrap();
        assert_eq!(bytes_processed, 21);
        assert_eq!(result.len(), 1);
        assert_eq!(
            result[0],
            DataType::List(vec![
                DataType::ByteString(ByteString::new(&"Hello, ".as_bytes().into())),
                DataType::ByteString(ByteString::new(&"World!".as_bytes().into()))
            ])
        );
    }

    #[test]
    fn test_nested_list_mixed() {
        let (result, bytes_processed) = decode_list("ll5:helloi123eee".as_bytes()).unwrap();
        assert_eq!(bytes_processed, 16);
        assert_eq!(result.len(), 1);
        assert_eq!(
            result[0],
            DataType::List(vec![
                DataType::ByteString(ByteString::new(&"hello".as_bytes().into())),
                DataType::Integer(123)
            ])
        );
    }

    #[test]
    fn test_two_nested_lists() {
        let (result, bytes_processed) = decode_list("ll5:helloeli123eee".as_bytes()).unwrap();
        assert_eq!(bytes_processed, 18);
        assert_eq!(result.len(), 2);
        assert_eq!(
            result[0],
            DataType::List(vec![DataType::ByteString(ByteString::new(
                &"hello".as_bytes().into()
            ))])
        );
        assert_eq!(result[1], DataType::List(vec![DataType::Integer(123)]));
    }

    #[test]
    fn test_list_and_number() {
        let (result, bytes_processed) = decode_list("ll5:helloei123ee".as_bytes()).unwrap();
        assert_eq!(bytes_processed, 16);
        assert_eq!(result.len(), 2);
        assert_eq!(
            result[0],
            DataType::List(vec![DataType::ByteString(ByteString::new(
                &"hello".as_bytes().into()
            ))])
        );
        assert_eq!(result[1], DataType::Integer(123));
    }

    #[test]
    fn test_number_and_list() {
        let (result, bytes_processed) = decode_list("li123el5:helloee".as_bytes()).unwrap();
        assert_eq!(bytes_processed, 16);
        assert_eq!(result.len(), 2);
        assert_eq!(result[0], DataType::Integer(123));
        assert_eq!(
            result[1],
            DataType::List(vec![DataType::ByteString(ByteString::new(
                &"hello".as_bytes().into()
            ))])
        );
    }

    #[test]
    fn test_list_and_string() {
        let (result, bytes_processed) = decode_list("ll5:helloe3:abce".as_bytes()).unwrap();
        assert_eq!(bytes_processed, 16);
        assert_eq!(result.len(), 2);
        assert_eq!(
            result[0],
            DataType::List(vec![DataType::ByteString(ByteString::new(
                &"hello".as_bytes().into()
            ))])
        );
        assert_eq!(
            result[1],
            DataType::ByteString(ByteString::new(&"abc".as_bytes().into()))
        );
    }

    #[test]
    fn test_string_and_list() {
        let (result, bytes_processed) = decode_list("l3:abcl5:helloee".as_bytes()).unwrap();
        assert_eq!(bytes_processed, 16);
        assert_eq!(result.len(), 2);
        assert_eq!(
            result[0],
            DataType::ByteString(ByteString::new(&"abc".as_bytes().into()))
        );
        assert_eq!(
            result[1],
            DataType::List(vec![DataType::ByteString(ByteString::new(
                &"hello".as_bytes().into()
            ))])
        );
    }

    #[test]
    fn test_empty_input() {
        let result = decode_list(&[]);
        assert!(result.is_err());
        assert_eq!(result.unwrap_err(), ListDecodeError::EmptyInput);
    }

    #[test]
    fn test_end_not_found_error() {
        let result = decode_list("l".as_bytes());
        assert!(result.is_err());
        assert_eq!(result.unwrap_err(), ListDecodeError::EndNotFound);

        let result = decode_list("lle".as_bytes());
        assert!(result.is_err());
        assert_eq!(result.unwrap_err(), ListDecodeError::EndNotFound);

        let result = decode_list("li123e".as_bytes());
        assert!(result.is_err());
        assert_eq!(result.unwrap_err(), ListDecodeError::EndNotFound);

        let result = decode_list("l4:care".as_bytes());
        assert!(result.is_err());
        assert_eq!(result.unwrap_err(), ListDecodeError::EndNotFound);
    }

    #[test]
    fn test_invalid_start_character() {
        let result = decode_list("12345".as_bytes());
        assert!(result.is_err());
        assert_eq!(
            result.unwrap_err(),
            ListDecodeError::StartNotFound {
                found: b'1',
                found_as_ascii: '1'
            }
        );

        let result = decode_list("ei123el".as_bytes());
        assert!(result.is_err());
        assert_eq!(
            result.unwrap_err(),
            ListDecodeError::StartNotFound {
                found: b'e',
                found_as_ascii: 'e'
            }
        );

        let result = decode_list("xyz".as_bytes());
        assert!(result.is_err());
        assert_eq!(
            result.unwrap_err(),
            ListDecodeError::StartNotFound {
                found: b'x',
                found_as_ascii: 'x'
            }
        );
    }
}
