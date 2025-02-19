use std::num::ParseIntError;

use thiserror::Error;

use crate::types::ByteString;

#[derive(Error, Debug, PartialEq)]
pub enum ByteStringDecodeError {
    #[error("The input is empty.")]
    EmptyInput,
    #[error("Unexpected byte value {unexpected_byte:?} (ASCII: {unexpected_byte_ascii:?}) at position {position:?}: bencoded strings must begin with a number (consisting of numeric characters from 0 to 9) indicating the length of the bencoded string and preceeding a colon character (':').")]
    UnexpectedByte {
        unexpected_byte: u8,
        unexpected_byte_ascii: char,
        position: usize,
    },
    #[error("Invalid length.")]
    InvalidLength(#[from] ParseIntError),
    #[error("The input does not have a colon (':').")]
    ColonNotFound,
    #[error("Expected {expected:?} bytes after the colon (':') but found {found:?}")]
    ContentTooShort { expected: usize, found: usize },
}

pub fn decode_byte_string(bencoded: &[u8]) -> Result<(ByteString, usize), ByteStringDecodeError> {
    if bencoded.is_empty() {
        return Err(ByteStringDecodeError::EmptyInput);
    }

    let mut length_str: String = String::new();
    let mut pos: usize = 0;
    let mut colon_found: bool = false;
    while pos < bencoded.len() {
        match bencoded[pos] {
            b':' => colon_found = true,
            b'0'..=b'9' => length_str.push(bencoded[pos] as char),
            other => Err(ByteStringDecodeError::UnexpectedByte {
                unexpected_byte: other,
                unexpected_byte_ascii: other as char,
                position: pos,
            })?,
        }
        pos += 1;
        if colon_found {
            break;
        }
    }

    if !colon_found {
        return Err(ByteStringDecodeError::ColonNotFound);
    }

    let length: usize = length_str.parse::<usize>()?;

    let n: usize = pos + length;
    if n > bencoded.len() {
        Err(ByteStringDecodeError::ContentTooShort {
            expected: length,
            found: bencoded.len() - pos,
        })?
    }

    let data: &[u8] = &bencoded[pos..n];
    Ok((ByteString::new(&data.into()), n))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_empty_input() {
        let result = decode_byte_string(&[]);
        assert!(result.is_err());
        assert_eq!(result.unwrap_err(), ByteStringDecodeError::EmptyInput);
    }

    #[test]
    fn test_unexpected_byte() {
        let result = decode_byte_string("1b3:test_data".as_bytes());
        assert!(result.is_err());
        assert_eq!(
            result.unwrap_err(),
            ByteStringDecodeError::UnexpectedByte {
                unexpected_byte: b'b',
                unexpected_byte_ascii: 'b',
                position: 1,
            }
        );
    }

    // #[test]
    // fn test_invalid_length() {
    //     let result = decode_byte_string("003:test_data".as_bytes());
    //     assert!(result.is_err());
    //     assert!(matches!(result.unwrap_err(), ByteStringDecodeError::InvalidLength(_)));
    // }

    #[test]
    fn test_content_too_short() {
        let result = decode_byte_string("6:world".as_bytes());
        assert!(result.is_err());
        assert_eq!(
            result.unwrap_err(),
            ByteStringDecodeError::ContentTooShort {
                expected: 6,
                found: 5
            }
        );
    }

    #[test]
    fn test_ascii_exact() {
        let (decoded, bytes_read) = decode_byte_string("5:hello".as_bytes()).unwrap();
        assert_eq!(bytes_read, 7);
        assert_eq!(decoded.get_data().as_ref(), "hello".as_bytes());
    }

    #[test]
    fn test_ascii_with_extra() {
        let (decoded, bytes_read) = decode_byte_string("5:helloxyz".as_bytes()).unwrap();
        assert_eq!(bytes_read, 7);
        assert_eq!(decoded.get_data().as_ref(), "hello".as_bytes());
    }

    #[test]
    fn test_bytes_exact() {
        let (decoded, bytes_read) = decode_byte_string(&[b'4', b':', 0xC, 0xA, 0xF, 0xE]).unwrap();
        assert_eq!(bytes_read, 6);
        assert_eq!(decoded.get_data().as_ref(), &[0xC, 0xA, 0xF, 0xE]);
    }

    #[test]
    fn test_bytes_with_extra() {
        let (result, bytes_read) =
            decode_byte_string(&[b'4', b':', 0xC, 0xA, 0xF, 0xE, 0xB, 0xA, 0xB, 0xE]).unwrap();
        assert_eq!(bytes_read, 6);
        assert_eq!(result.get_data().as_ref(), &[0xC, 0xA, 0xF, 0xE]);
    }

    #[test]
    fn test_url() {
        let (result, bytes_read) = decode_byte_string(
            "60:http://bittorrent-test-tracker.codecrafters.io:8080/announce".as_bytes(),
        )
        .unwrap();
        assert_eq!(bytes_read, 63);
        assert_eq!(
            result.get_data().as_ref(),
            "http://bittorrent-test-tracker.codecrafters.io:8080/announce".as_bytes()
        );
    }

    #[test]
    fn test_ascii_colon_not_found() {
        let result = decode_byte_string("12345".as_bytes());
        assert!(result.is_err());
        assert_eq!(result.unwrap_err(), ByteStringDecodeError::ColonNotFound);
    }

    #[test]
    fn test_bytes_colon_not_found() {
        let result = decode_byte_string(&[b'1', b'2', b'3', b'4', b'5']);
        assert!(result.is_err());
        assert_eq!(result.unwrap_err(), ByteStringDecodeError::ColonNotFound);
    }
}
