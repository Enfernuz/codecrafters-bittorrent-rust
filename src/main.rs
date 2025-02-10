use serde_json::{self, Number};
use std::{env, str::FromStr};

// Available if you need it!
// use serde_bencode

#[allow(dead_code)]
fn decode_bencoded_value(encoded_value: &str) -> Result<(serde_json::Value, usize), String> {
    eprintln!("decode_bencoded_value: {}", encoded_value);
    // If encoded_value starts with a digit, it's a number
    let first_char: char = encoded_value.chars().next().unwrap();
    match first_char {
        '0'..='9' => return decode_string(encoded_value),
        'i' => return decode_number(encoded_value),
        'l' => return decode_list(encoded_value),
        _ => Err(format!("Unhandled encoded value: {}", encoded_value)),
    }
}

fn decode_string(encoded_value: &str) -> Result<(serde_json::Value, usize), String> {
    // Example: "5:hello" -> "hello"
    match encoded_value.find(':') {
        Some(colon_index) => {
            let length: &str = &encoded_value[..colon_index];
            match length.parse::<usize>() {
                Ok(len) => {
                    let string: &str = &encoded_value[colon_index + 1..colon_index + 1 + len];
                    return Ok((serde_json::Value::String(string.to_string()), len + 2));
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

fn decode_number(encoded_value: &str) -> Result<(serde_json::Value, usize), String> {
    // Example: "i1229367652e" -> 1229367652
    match encoded_value.find('e') {
        Some(index) => {
            let num_str: &str = &encoded_value[1..index];
            return match Number::from_str(num_str) {
                Ok(number) => Ok((serde_json::Value::Number(number), num_str.len() + 2)),
                Err(err) => Err(err.to_string()),
            };
        }
        None => {
            return Err(format!(
                "Could not find 'e' at the end encoded integer input {}.",
                encoded_value
            ))
        }
    }
}

fn decode_list(encoded_value: &str) -> Result<(serde_json::Value, usize), String> {
    // Example: "l5:helloi52ee" -> ["hello", 52]
    // Example: "lli796e5:appleee" -> [[796, "apple"]]
    // Example: "l6:orangei695ee" -> ["orange",695]
    // Example: "lli4eei5ee" -> [[4],5]

    let mut decoded_elements: Vec<serde_json::Value> = vec![];
    let encoded_elements: &str = &encoded_value[1..];
    let mut index = 0;
    let mut end_of_list_found = false;
    while index < encoded_elements.len() {
        if encoded_elements[index..=index].eq("e") {
            end_of_list_found = true;
            break;
        }
        let (decoded_element, processed_characters_count) =
            decode_bencoded_value(&encoded_elements[index..])?;
        decoded_elements.push(decoded_element);
        index += processed_characters_count;
    }

    if end_of_list_found {
        Ok((
            serde_json::Value::Array(decoded_elements),
            index + 2, // l, e and the overall number of processed characters of decoded entries
        ))
    } else {
        return Err(format!(
            "Could not find the end of list ('e') in the encoded list input {}.",
            encoded_value
        ));
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
            Ok(result) => println!("{}", &result.0.to_string()),
            Err(err) => panic!("{}", &err),
        }
    } else {
        println!("unknown command: {}", args[1])
    }
}
