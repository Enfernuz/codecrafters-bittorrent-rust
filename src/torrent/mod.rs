use std::fmt;

use crate::bencode::decoder;

pub struct Torrent {
    tracker_url: String,
    length: u64,
}

impl Torrent {
    pub fn get_tracker_url(&self) -> &str {
        &self.tracker_url
    }

    pub fn get_length(&self) -> u64 {
        self.length
    }
}

impl TryFrom<&[u8]> for Torrent {
    type Error = String;

    fn try_from(data: &[u8]) -> Result<Self, Self::Error> {
        let (decoded, _) = decoder::decode(data).map_err(|err| err.get_message().to_owned())?;
        let json_value: serde_json::Value = decoded.into();
        json_value
            .as_object()
            .ok_or_else(|| "Could not convert the decoded JSON into a JSON object.")?
            .try_into()
    }
}

impl TryFrom<&serde_json::Map<String, serde_json::Value>> for Torrent {
    type Error = String;

    fn try_from(map: &serde_json::Map<String, serde_json::Value>) -> Result<Self, Self::Error> {
        let tracker_url = map
            .get("announce")
            .ok_or_else(|| "Could not find the 'announce' key.")?
            .as_str()
            .ok_or_else(|| "Could not convert the value for the 'announce' key into a string.")?
            .to_owned();
        let length = map
            .get("info")
            .ok_or_else(|| "Could not find the 'info' key.")?
            .as_object()
            .ok_or_else(|| "Could not convert the value for the 'info' key into a JSON object.")?
            .get("length")
            .ok_or_else(|| "Could not find the 'length' key.")?
            .as_u64()
            .ok_or_else(|| "Could not convert the value of the 'length' key to u64.")?;
        Ok(Torrent {
            tracker_url,
            length,
        })
    }
}

impl fmt::Display for Torrent {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "Tracker URL: {}\nLength: {}",
            &self.tracker_url, self.length
        )
    }
}
