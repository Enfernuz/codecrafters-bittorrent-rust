use std::collections::HashMap;

use regex::Regex;

use crate::error::Error;
use crate::error::Result;

// region:      ---MagnetLinkV1
pub struct MagnetLinkV1 {
    info_hash: String,
    file_name: String,
    tracker_url: String,
}

// magnet:?xt=urn:btih:ad42ce8109f54c99613ce38f9b4d87e70f24a165&dn=magnet1.gif&tr=http%3A%2F%2Fbittorrent-test-tracker.codecrafters.io%2Fannounce

// region:      ---Constructors
impl MagnetLinkV1 {
    pub fn parse(input: &str) -> Result<MagnetLinkV1> {
        // Regex to match key=value pairs in the magnet URL
        let params_str = input
            .strip_prefix("magnet:?")
            .ok_or_else(|| Error::InvalidMagnetLink)?;
        let params: HashMap<String, String> =
            serde_urlencoded::from_str::<HashMap<String, String>>(params_str)
                .map_err(|err| Error::InvalidMagnetLink)?;

        // let re = Regex::new(r"([a-z0-9]+)=([^&]+)").unwrap();
        // let mut params = HashMap::new();

        // for cap in re.captures_iter(input) {
        //     let key = &cap[1];
        //     let value = &cap[2];
        //     params.insert(key.to_string(), value.to_string());
        // }

        let hash: String = params
            .get("xt")
            .ok_or_else(|| Error::InvalidMagnetLink)?
            .strip_prefix("urn:btih:")
            .ok_or_else(|| Error::InvalidMagnetLink)?
            .to_owned();
        let file_name = params
            .get("dn")
            .ok_or_else(|| Error::InvalidMagnetLink)?
            .to_owned();
        let tracker_url = params
            .get("tr")
            .ok_or_else(|| Error::InvalidMagnetLink)?
            .to_owned();

        // let kek = "kek";
        // serde_urlencoded::from_str(input)

        Ok(MagnetLinkV1 {
            info_hash: hash,
            file_name: file_name,
            tracker_url: tracker_url,
        })
    }
}
// endregion:   ---Constructors
// region:      ---Getters
impl MagnetLinkV1 {
    pub fn get_info_hash(&self) -> &str {
        &self.info_hash
    }

    pub fn get_file_name(&self) -> &str {
        &self.file_name
    }

    pub fn get_tracker_url(&self) -> &str {
        &self.tracker_url
    }
}
// endregion:   ---Getters
// endregion:   ---MagnetLinkV1
