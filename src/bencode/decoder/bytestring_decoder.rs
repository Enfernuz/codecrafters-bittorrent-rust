use crate::types::byte_string::ByteString;

use super::DecodeError;

pub fn decode_byte_string(bencoded: &[u8]) -> Result<(ByteString, usize), DecodeError> {
    //dbg!("decode_byte_string: {:?}", bencoded);

    if bencoded.is_empty() {
        Err(DecodeError::new("The input is empty."))?
    }

    let mut length_buf: Vec<u8> = vec![];
    let mut pos: usize = 0;
    let mut colon_found: bool = false;
    while pos < bencoded.len() {
        match bencoded[pos] {
            b':' => colon_found = true,
            b'0'..=b'9' => length_buf.push(bencoded[pos]),
            other => Err(DecodeError::new(&format!("Unexpected byte value '{other}' (ASCII: '{}') at position {pos}: bencoded strings must begin with a number (consisting of numeric characters from 0 to 9) indicating the length of the bencoded string and preceeding a colon character (':').", other as char)))?,
        }
        pos += 1;
        if colon_found {
            break;
        }
    }

    if !colon_found {
        Err(DecodeError::new("Colon (':') not found.".into()))?
    }

    let length = String::from_utf8(length_buf)
        .map_err(|err| DecodeError::new(&err.to_string()))?
        .parse::<usize>()
        .map_err(|err| DecodeError::new(&err.to_string()))?;

    let n: usize = pos + length;
    if n > bencoded.len() {
        Err(DecodeError::new(&format!("Not enough bytes to read after the colon (':') to read a byte string of length {length}: {}", bencoded.len() - pos)))?
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

        assert_eq!(result.is_err(), true);
        assert_eq!(result.unwrap_err().get_message(), "The input is empty.");
    }

    #[test]
    fn test_invalid_length() {
        let result = decode_byte_string("1b3:test_data".as_bytes());

        assert_eq!(result.is_err(), true);
        assert_eq!(result.unwrap_err().get_message(),
            format!(
                "Unexpected byte value '{}' (ASCII: '{}') at position {}: bencoded strings must begin with a number (consisting of numeric characters from 0 to 9) indicating the length of the bencoded string and preceeding a colon character (':').", 
                b'b',
                'b',
                1));
    }

    #[test]
    fn test_data_too_short() {
        let result = decode_byte_string("6:world".as_bytes());

        assert_eq!(result.is_err(), true);
        assert_eq!(result.unwrap_err().get_message(),
        format!(
            "Not enough bytes to read after the colon (':') to read a byte string of length {}: {}", 
            6,
            5));
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

        assert_eq!(result.is_err(), true);
        assert_eq!(result.unwrap_err().get_message(), "Colon (':') not found.");
    }

    #[test]
    fn test_bytes_colon_not_found() {
        let result = decode_byte_string(&[b'1', b'2', b'3', b'4', b'5']);

        assert_eq!(result.is_err(), true);
        assert_eq!(result.unwrap_err().get_message(), "Colon (':') not found.");
    }
}
