use std::{collections::HashMap, fmt};

use sha1::{Digest, Sha1};

use crate::{
    bencode::{decoder, encoder},
    types::DataType,
};

pub struct Torrent {
    tracker_url: String,
    length: u64,
    info_hash: String,
}

impl Torrent {
    pub fn get_tracker_url(&self) -> &str {
        &self.tracker_url
    }

    pub fn get_length(&self) -> u64 {
        self.length
    }

    pub fn get_info_hash(&self) -> &str {
        &self.info_hash
    }
}

impl TryFrom<&[u8]> for Torrent {
    type Error = String;

    fn try_from(data: &[u8]) -> Result<Self, Self::Error> {
        let (decoded, _) = decoder::decode(data).map_err(|err| err.get_message().to_owned())?;
        Torrent::try_from(&decoded)
    }
}

impl TryFrom<&DataType> for Torrent {
    type Error = String;

    fn try_from(data: &DataType) -> Result<Self, Self::Error> {
        match data {
            DataType::Dict(v) => {
                let mut map: HashMap<String, DataType> = HashMap::new();
                for (key, value) in v {
                    map.insert(key.clone(), value.clone());
                }
                let tracker_url = map
                    .get("announce")
                    .ok_or_else(|| "Could not find the 'announce' key.")?
                    .as_str()
                    .ok_or_else(|| {
                        "Could not convert the value for the 'announce' key into a string."
                    })?;
                let info = map
                    .get("info")
                    .ok_or_else(|| "Could not find the 'info' key.")?;
                let length = info
                    .as_dict()
                    .ok_or_else(|| "Could not convert the value for the 'info' key into a dict.")?
                    .get("length")
                    .ok_or_else(|| "Could not find the 'length' key.")?
                    .as_i64()
                    .ok_or_else(|| "Could not convert the value of the 'length' key to i64.")?
                    as u64;
                let info_bencoded: &[u8] = &encoder::bencode(info);
                let mut hasher = Sha1::new();
                hasher.update(&info_bencoded);
                let sha1_hash = hasher.finalize();
                let info_hash: String = hex::encode(&sha1_hash.as_slice());
                Ok(Torrent {
                    tracker_url,
                    length,
                    info_hash,
                })
            }
            other => Err(format!(
                "Could not convert into a Torrent. Only dicts are supported, but got {:?}",
                other
            )),
        }
    }
}

impl fmt::Display for Torrent {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "Tracker URL: {}\nLength: {}\nInfo Hash: {}",
            &self.tracker_url, self.length, self.info_hash
        )
    }
}
