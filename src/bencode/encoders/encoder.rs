use std::{collections::BTreeMap, rc::Rc};

pub fn bencode_i64(value: i64) -> Rc<[u8]> {
    let mut v: Vec<u8> = vec![b'i'];
    v.extend_from_slice(value.to_string().as_bytes());
    v.push(b'e');
    v.into()
}

pub fn bencode_byte_string(value: &crate::types::byte_string::ByteString) -> Rc<[u8]> {
    let mut result: Vec<u8> = vec![];
    result.extend_from_slice(value.get_data().len().to_string().as_bytes());
    result.push(b':');
    result.extend_from_slice(value.get_data());
    result.into()
}

pub fn bencode_list(list: &Vec<crate::types::data_type::DataType>) -> Rc<[u8]> {
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

pub fn bencode_dict(dict: &BTreeMap<String, crate::types::data_type::DataType>) -> Rc<[u8]> {
    let mut result: Vec<u8> = vec![b'd'];
    for (key, value) in dict {
        result.extend_from_slice(&bencode_string(key));
        result.extend_from_slice(&bencode(value));
    }
    result.push(b'e');
    result.into()
}

pub fn bencode(value: &crate::types::data_type::DataType) -> Rc<[u8]> {
    match value {
        crate::types::data_type::DataType::Integer(num) => bencode_i64(*num),
        crate::types::data_type::DataType::ByteString(byte_str) => bencode_byte_string(byte_str),
        crate::types::data_type::DataType::List(list) => bencode_list(list),
        crate::types::data_type::DataType::Dict(dict) => bencode_dict(dict),
    }
}
