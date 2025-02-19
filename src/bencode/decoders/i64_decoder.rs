use std::{num::ParseIntError, string::FromUtf8Error};

use thiserror::Error;

#[derive(Error, Debug, PartialEq)]
pub enum Int64DecodeError {
    #[error("The input is empty.")]
    EmptyInput,
    #[error(
        "Bencoded numbers must start with 'i' but got '{found:?}' (ASCII: '{found_as_ascii:?}')."
    )]
    StartNotFound { found: u8, found_as_ascii: char },
    #[error("The end of number ('e') not found.")]
    EndNotFound,
    #[error("Number not found.")]
    NumberNotFound,
    #[error("Could not parse the number as UTF-8.")]
    NumberUtf8ParseError(#[from] FromUtf8Error),
    #[error("Invalid number: {0}")]
    InvalidNumberError(String),
    #[error("")]
    NumberParseIntError(#[from] ParseIntError),
}

pub fn decode_i64(bencoded: &[u8]) -> Result<(i64, usize), Int64DecodeError> {
    if let [start, ..] = bencoded {
        if *start != b'i' {
            return Err(Int64DecodeError::StartNotFound {
                found: *start,
                found_as_ascii: *start as char,
            });
        }

        let mut num_chars_buf: Vec<u8> = vec![];
        let mut pos: usize = 1;
        let mut end_of_number_found: bool = false;
        while pos < bencoded.len() {
            match bencoded[pos] {
                b'e' => end_of_number_found = true,
                other => num_chars_buf.push(other),
            }
            pos += 1;
            if end_of_number_found {
                break;
            }
        }

        if !end_of_number_found {
            return Err(Int64DecodeError::EndNotFound);
        }

        if num_chars_buf.len() == 0 {
            return Err(Int64DecodeError::NumberNotFound);
        }

        let number_as_string = String::from_utf8(num_chars_buf)?;

        if number_as_string.eq("-0") {
            return Err(Int64DecodeError::InvalidNumberError(
                "-0 is not valid for bencoded integers.".to_owned(),
            ));
        } else if number_as_string.starts_with('0') && number_as_string.len() > 1 {
            return Err(Int64DecodeError::InvalidNumberError(
                "Bencoded integers other than 0 can not have leading zeroes.".to_owned(),
            ));
        }

        let number: i64 = number_as_string.parse::<i64>()?;

        Ok((number, pos))
    } else {
        return Err(Int64DecodeError::EmptyInput);
    }
}

#[cfg(test)]
mod tests {

    use super::*;

    #[test]
    fn test_zero() {
        let (result, bytes_read) = decode_i64("i0e".as_bytes()).unwrap();
        assert_eq!(bytes_read, 3);
        assert_eq!(result, 0);
    }

    #[test]
    fn test_i64_nonzero() {
        let (result, bytes_read) = decode_i64("i12345e".as_bytes()).unwrap();
        assert_eq!(bytes_read, 7);
        assert_eq!(result, 12345);

        let (result, bytes_read) = decode_i64("i2048550915e".as_bytes()).unwrap();
        assert_eq!(bytes_read, 12);
        assert_eq!(result, 2048550915);

        let (result, bytes_read) = decode_i64("i4294967300e".as_bytes()).unwrap();
        assert_eq!(bytes_read, 12);
        assert_eq!(result, 4294967300);

        let (result, bytes_read) = decode_i64("i-52e".as_bytes()).unwrap();
        assert_eq!(bytes_read, 5);
        assert_eq!(result, -52);
    }

    #[test]
    fn test_empty_input() {
        let result = decode_i64(&[]);
        assert!(result.is_err());
        assert_eq!(result.unwrap_err(), Int64DecodeError::EmptyInput);
    }

    #[test]
    fn test_i64_number_not_found_error() {
        let result = decode_i64("ie".as_bytes());
        assert!(result.is_err());
        assert_eq!(result.unwrap_err(), Int64DecodeError::NumberNotFound);
    }

    #[test]
    fn test_i64_end_not_found_error() {
        let result = decode_i64("i123".as_bytes());
        assert!(result.is_err());
        assert_eq!(result.unwrap_err(), Int64DecodeError::EndNotFound);
    }

    #[test]
    fn test_negative_zero_error() {
        let result = decode_i64("i-0e".as_bytes());
        assert!(result.is_err());
        assert_eq!(
            result.unwrap_err(),
            Int64DecodeError::InvalidNumberError(
                "-0 is not valid for bencoded integers.".to_owned()
            )
        );
    }

    #[test]
    fn test_leading_zero_error() {
        let result = decode_i64("i03e".as_bytes());
        assert!(result.is_err());
        assert_eq!(
            result.unwrap_err(),
            Int64DecodeError::InvalidNumberError(
                "Bencoded integers other than 0 can not have leading zeroes.".to_owned()
            )
        );
    }

    #[test]
    fn test_invalid_start_character() {
        let result = decode_i64("12345".as_bytes());
        assert!(result.is_err());
        assert_eq!(
            result.unwrap_err(),
            Int64DecodeError::StartNotFound {
                found: b'1',
                found_as_ascii: '1'
            }
        );

        let result = decode_i64("e123i".as_bytes());
        assert!(result.is_err());
        assert_eq!(
            result.unwrap_err(),
            Int64DecodeError::StartNotFound {
                found: b'e',
                found_as_ascii: 'e'
            }
        );

        let result = decode_i64("xyz".as_bytes());
        assert!(result.is_err());
        assert_eq!(
            result.unwrap_err(),
            Int64DecodeError::StartNotFound {
                found: b'x',
                found_as_ascii: 'x'
            }
        );
    }

    #[test]
    fn test_number_parse_error() {
        let result = decode_i64("ia2ce".as_bytes());
        assert!(result.is_err());
        assert!(matches!(
            result.unwrap_err(),
            Int64DecodeError::NumberParseIntError(_)
        ));

        let result = decode_i64(&[b'i', 0xC, 0xA, 0xF, 0xE, b'e']);
        assert!(result.is_err());
        assert!(matches!(
            result.unwrap_err(),
            Int64DecodeError::NumberParseIntError(_)
        ));
    }
}
