use serde_json::{self, Map};
use std::env;

// Available if you need it!
// use serde_bencode

#[allow(dead_code)]
fn decode_bencoded_value(encoded_value: &str) -> Result<(serde_json::Value, usize), String> {
    eprintln!("decode_bencoded_value: {}", encoded_value);
    // If encoded_value starts with a digit, it's a number
    let first_char: char = encoded_value.chars().next().unwrap();
    return match first_char {
        '0'..='9' => decode_string(encoded_value).map(|(value, len)| (value.into(), len)),
        'i' => decode_number(encoded_value).map(|(value, len)| (value.into(), len)),
        'l' => decode_list(encoded_value).map(|(value, len)| (value.into(), len)),
        'd' => decode_dict(encoded_value).map(|(value, len)| (value.into(), len)),
        _ => Err(format!("Unhandled encoded value: {}", encoded_value)),
    };
}

fn decode_string(encoded_value: &str) -> Result<(String, usize), String> {
    // Example: "5:hello" -> "hello"
    match encoded_value.find(':') {
        Some(colon_index) => {
            let length: &str = &encoded_value[..colon_index];
            match length.parse::<usize>() {
                Ok(len) => {
                    let string: &str = &encoded_value[colon_index + 1..colon_index + 1 + len];
                    return Ok((string.into(), len + colon_index + 1));
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

fn decode_number(encoded_value: &str) -> Result<(i64, usize), String> {
    // Example: "i1229367652e" -> 1229367652
    match encoded_value.find('e') {
        Some(index) => {
            let num_str: &str = &encoded_value[1..index];
            return match num_str.parse::<i64>() {
                Ok(number) => Ok((number, num_str.len() + 2)),
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

fn decode_list(encoded_value: &str) -> Result<(Vec<serde_json::Value>, usize), String> {
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
            decoded_elements,
            // l + e + the overall number of processed characters for decoded elements of the list
            index + 2,
        ))
    } else {
        return Err(format!(
            "Could not find the end of list ('e') in the encoded list input {}.",
            encoded_value
        ));
    }
}

fn decode_dict(encoded_value: &str) -> Result<(Map<String, serde_json::Value>, usize), String> {
    // Example: d3:foo3:bar5:helloi52ee -> {"hello": 52, "foo":"bar"}
    let mut decoded_dict: Map<String, serde_json::Value> = Map::new();
    let encoded_dict: &str = &encoded_value[1..];
    let mut index = 0;
    let mut end_of_dict_found = false;
    while index < encoded_dict.len() {
        if encoded_dict[index..=index].eq("e") {
            end_of_dict_found = true;
            break;
        }
        let (key, key_characters_count) = decode_string(&encoded_dict[index..])?;
        index += key_characters_count;
        if index >= encoded_dict.len() {
            return Err(format!(
                "Incorrect dict format: reached the end of input before reaching the value for key {}.\nBencoded dict:\n{}",
                &key,
                &encoded_value
            ));
        } else if encoded_dict[index..=index].eq("e") {
            return Err(format!(
                "Incorrect dict format: reached the end of dict ('e') before reaching the value for key {}.\nBencoded dict:\n{}",
                &key,
                &encoded_value[0..=index]
            ));
        }
        let (value, value_characters_count) = decode_bencoded_value(&encoded_dict[index..])?;
        index += value_characters_count;
        decoded_dict.insert(key, value);
    }

    if end_of_dict_found {
        Ok((
            decoded_dict.into(),
            // d + e + the overall number of processed characters for decoded elements of the dict
            index + 2,
        ))
    } else {
        return Err(format!(
            "Could not find the end of dict ('e') in the encoded dict input {}.",
            &encoded_value[0..=index]
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
