use super::DecodeError;

pub fn decode_i64(bencoded: &[u8]) -> Result<(i64, usize), DecodeError> {
    //dbg!("decode_i64: {:?}", bencoded);

    if let [start, ..] = bencoded {
        if *start != b'i' {
            Err(DecodeError::new(&format!(
                "Bencoded integers must start with 'i' but got '{}' (ASCII: '{}').",
                *start, *start as char
            )))?
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
            Err(DecodeError::new("End of number ('e') not found.".into()))?
        }

        if num_chars_buf.len() == 0 {
            Err(DecodeError::new("Number not found."))?
        }

        let number_as_string = String::from_utf8_lossy(&num_chars_buf);

        if number_as_string.eq("-0") {
            Err(DecodeError::new("-0 is not valid for bencoded integers."))?
        } else if number_as_string.starts_with('0') && number_as_string.len() > 1 {
            Err(DecodeError::new(
                "Bencoded integers other than 0 can not have leading zeroes.",
            ))?
        }

        let number: i64 = number_as_string.parse::<i64>().map_err(|_| {
            DecodeError::new(&format!("Could not parse '{number_as_string}' to i64."))
        })?;

        Ok((number, pos))
    } else {
        Err(DecodeError::new("The input is empty."))?
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

        assert_eq!(result.is_err(), true);
        assert_eq!(result.unwrap_err().get_message(), "The input is empty.");
    }

    #[test]
    fn test_i64_number_not_found_error() {
        let result = decode_i64("ie".as_bytes());

        assert_eq!(result.is_err(), true);
        assert_eq!(result.unwrap_err().get_message(), "Number not found.");
    }

    #[test]
    fn test_i64_end_not_found_error() {
        let result = decode_i64("i123".as_bytes());

        assert_eq!(result.is_err(), true);
        assert_eq!(
            result.unwrap_err().get_message(),
            "End of number ('e') not found."
        );
    }

    #[test]
    fn test_negative_zero_error() {
        let result = decode_i64("i-0e".as_bytes());
        assert_eq!(result.is_err(), true);
        assert_eq!(
            result.unwrap_err().get_message(),
            "-0 is not valid for bencoded integers."
        );
    }

    #[test]
    fn test_leading_zero_error() {
        let result = decode_i64("i03e".as_bytes());
        assert_eq!(result.is_err(), true);
        assert_eq!(
            result.unwrap_err().get_message(),
            "Bencoded integers other than 0 can not have leading zeroes."
        );
    }

    #[test]
    fn test_invalid_start_character() {
        let result = decode_i64("12345".as_bytes());
        assert_eq!(result.is_err(), true);
        assert_eq!(
            result.unwrap_err().get_message(),
            &format!(
                "Bencoded integers must start with 'i' but got '{}' (ASCII: '{}').",
                b'1', 1
            )
        );

        let result = decode_i64("e123i".as_bytes());
        assert_eq!(result.is_err(), true);
        assert_eq!(
            result.unwrap_err().get_message(),
            &format!(
                "Bencoded integers must start with 'i' but got '{}' (ASCII: '{}').",
                b'e', 'e'
            )
        );

        let result = decode_i64("xyz".as_bytes());
        assert_eq!(result.is_err(), true);
        assert_eq!(
            result.unwrap_err().get_message(),
            &format!(
                "Bencoded integers must start with 'i' but got '{}' (ASCII: '{}').",
                b'x', 'x'
            )
        );
    }

    #[test]
    fn test_number_parse_error() {
        let result = decode_i64("ia2ce".as_bytes());
        assert_eq!(result.is_err(), true);
        assert_eq!(
            result.unwrap_err().get_message(),
            &format!("Could not parse 'a2c' to i64.")
        );

        let result = decode_i64(&[b'i', 0xC, 0xA, 0xF, 0xE, b'e']);
        assert_eq!(result.is_err(), true);
        assert_eq!(
            result.unwrap_err().get_message(),
            &format!(
                "Could not parse '{}{}{}{}' to i64.",
                0xC as char, 0xA as char, 0xF as char, 0xE as char
            )
        );
    }
}
