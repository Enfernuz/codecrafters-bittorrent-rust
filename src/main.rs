use serde_json::{self, Number};
use std::{env, str::FromStr};

// Available if you need it!
// use serde_bencode

#[allow(dead_code)]
fn decode_bencoded_value(encoded_value: &str) -> Result<serde_json::Value, String> {
    // If encoded_value starts with a digit, it's a number
    let first_char: char = encoded_value.chars().next().unwrap();
    match first_char {
        '0'..='9' => {
            // Example: "5:hello" -> "hello"
            match encoded_value.find(':') {
                Some(colon_index) => {
                    let number_string = &encoded_value[..colon_index];
                    match number_string.parse::<i64>() {
                        Ok(number) => {
                            let string =
                                &encoded_value[colon_index + 1..colon_index + 1 + number as usize];
                            return Ok(serde_json::Value::String(string.to_string()));
                        }
                        Err(err) => return Err(err.to_string()),
                    }
                }
                None => {
                    return Err(format!(
                        "Could not find ':' in the encoded input {}.",
                        encoded_value
                    ))
                }
            }
        }
        'i' => {
            // Example: "i1229367652e" -> 1229367652
            match encoded_value.ends_with('e') {
                true => {
                    let num_str = &encoded_value[1..encoded_value.len() - 1];
                    match Number::from_str(num_str) {
                        Ok(number) => return Ok(serde_json::Value::Number(number)),
                        Err(err) => return Err(err.to_string()),
                    }
                }
                false => {
                    return Err(format!(
                        "Could not find 'e' at the end encoded input {}.",
                        encoded_value
                    ))
                }
            }
        }
        _ => Err(format!("Unhandled encoded value: {}", encoded_value)),
    }
}

// Usage: your_bittorrent.sh decode "<encoded_value>"
fn main() {
    let args: Vec<String> = env::args().collect();
    let command = &args[1];

    if command == "decode" {
        let encoded_value = &args[2];
        let decoded_value = decode_bencoded_value(encoded_value);
        match decoded_value {
            Ok(result) => println!("{}", result.to_string()),
            Err(err) => panic!("{}", &err),
        }
    } else {
        println!("unknown command: {}", args[1])
    }
}
