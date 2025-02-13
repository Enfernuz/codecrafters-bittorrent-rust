use std::collections::BTreeMap;

use byte_string::ByteString;

pub mod byte_string;

#[derive(Clone, Debug, PartialEq)]
pub enum DataType {
    Integer(i64),
    ByteString(ByteString),
    List(Vec<DataType>),
    Dict(BTreeMap<String, DataType>),
}

impl DataType {
    pub fn as_i64(&self) -> Option<i64> {
        match self {
            Self::Integer(value) => Some(*value),
            _ => None,
        }
    }

    pub fn as_str(&self) -> Option<String> {
        match self {
            Self::ByteString(x) => Some(String::from_utf8_lossy(&x.get_data()).into_owned()),
            _ => None,
        }
    }

    pub fn as_dict(&self) -> Option<&BTreeMap<String, DataType>> {
        match self {
            Self::Dict(dict) => Some(dict),
            _ => None,
        }
    }

    pub fn as_byte_string(&self) -> Option<&ByteString> {
        match self {
            Self::ByteString(byte_str) => Some(byte_str),
            _ => None,
        }
    }
}

impl From<DataType> for serde_json::Value {
    fn from(value: DataType) -> Self {
        match value {
            DataType::Integer(number) => number.into(),
            DataType::ByteString(byte_str) => {
                serde_json::Value::String(String::from_utf8_lossy(byte_str.get_data()).into())
            }
            DataType::List(list) => {
                let values: Vec<serde_json::Value> = list.into_iter().map(|el| el.into()).collect();
                serde_json::Value::Array(values)
            }
            DataType::Dict(dict) => {
                let mut map: serde_json::Map<String, serde_json::Value> = serde_json::Map::new();
                for (key, value) in dict {
                    map.insert(key, value.into());
                }
                serde_json::Value::Object(map)
            }
        }
    }
}
