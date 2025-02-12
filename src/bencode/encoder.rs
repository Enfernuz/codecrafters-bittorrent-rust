use std::{collections::BTreeMap, rc::Rc};

use crate::types::{byte_string::ByteString, DataType};

pub fn bencode_i64(value: i64) -> Rc<[u8]> {
    let mut v: Vec<u8> = vec![b'i'];
    v.extend_from_slice(value.to_string().as_bytes());
    v.push(b'e');
    v.into()
}

pub fn bencode_byte_string(value: &ByteString) -> Rc<[u8]> {
    let mut result: Vec<u8> = vec![];
    result.extend_from_slice(value.get_data().len().to_string().as_bytes());
    result.push(b':');
    result.extend_from_slice(value.get_data());
    result.into()
}

pub fn bencode_list(list: &Vec<DataType>) -> Rc<[u8]> {
    let mut result: Vec<u8> = vec![b'l'];
    for element in list {
        result.extend_from_slice(&bencode(element));
    }
    result.push(b'e');
    result.into()
}

pub fn bencode_string(value: &str) -> Rc<[u8]> {
    let bytes = value.as_bytes();
    let mut result: Vec<u8> = vec![];
    result.extend_from_slice(&bytes.len().to_string().as_bytes());
    result.push(b':');
    result.extend_from_slice(bytes);
    result.into()
}

pub fn bencode_dict(dict: &BTreeMap<String, DataType>) -> Rc<[u8]> {
    let mut result: Vec<u8> = vec![b'd'];
    for (key, value) in dict {
        result.extend_from_slice(&bencode_string(key));
        result.extend_from_slice(&bencode(value));
    }
    result.push(b'e');
    result.into()
}

pub fn bencode(value: &DataType) -> Rc<[u8]> {
    match value {
        DataType::Integer(num) => bencode_i64(*num),
        DataType::ByteString(byte_str) => bencode_byte_string(byte_str),
        DataType::List(list) => bencode_list(list),
        DataType::Dict(dict) => bencode_dict(dict),
    }
}
