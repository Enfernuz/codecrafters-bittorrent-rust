use thiserror::Error;

use crate::bencode::decoder;

pub enum TrackerResponse {
    Ok { interval: u32, peers: Box<[String]> },
    Failure(String),
}

impl TrackerResponse {
    pub fn failure(reason: String) -> Self {
        Self::Failure(reason)
    }

    pub fn ok(interval: u32, peers: Box<[String]>) -> Self {
        Self::Ok { interval, peers }
    }
}

#[derive(Error, Debug)]
pub enum TrackerError {
    #[error("HTTP Error")]
    Http(#[from] reqwest::Error),
    #[error("Decode Error")]
    Decode(#[from] decoder::DecodeError),
    #[error("Invalid Response: {0}")]
    InvalidResponse(String),
}

pub fn get(
    torrent: &crate::torrent::torrent::Torrent,
    peer_id: &str,
    port: u16,
    uploaded: u64,
    downloaded: u64,
    left: u64,
) -> Result<TrackerResponse, TrackerError> {
    let urlencoded_info_hash: String = urlencode_bytes(torrent.get_info_hash().as_ref());
    let tracker_url: &str = torrent.get_announce();
    let url = format!("{tracker_url}?info_hash={urlencoded_info_hash}&peer_id={peer_id}&port={port}&uploaded={uploaded}&downloaded={downloaded}&left={left}&compact=1");
    let res = reqwest::blocking::get(url)?;
    let (body, _) = decoder::decode(&res.bytes()?.as_ref())?;
    let body_dict = body.as_dict().ok_or_else(|| {
        TrackerError::InvalidResponse("Could not find the 'info' key.".to_owned())
    })?;
    if body_dict.contains_key("failure reason") {
        let reason = body_dict
            .get("failure reason")
            .ok_or_else(|| {
                TrackerError::InvalidResponse("Could not find the 'failure reason' key.".to_owned())
            })?
            .as_str()
            .ok_or_else(|| {
                TrackerError::InvalidResponse(
                    "Could not convert the value of the 'failure reason' key into a UTF-8 string."
                        .to_owned(),
                )
            })?;
        return Ok(TrackerResponse::failure(reason));
    }
    let interval = body_dict
        .get("interval")
        .ok_or_else(|| {
            TrackerError::InvalidResponse("Could not find the 'interval' key.".to_owned())
        })?
        .as_i64()
        .ok_or_else(|| {
            TrackerError::InvalidResponse(
                "Could not convert the value of the 'interval' key into i64".to_owned(),
            )
        })? as u32;
    let peers = body_dict
        .get("peers")
        .ok_or_else(|| TrackerError::InvalidResponse("Could not find the 'peers' key.".to_owned()))?
        .as_byte_string()
        .ok_or_else(|| {
            TrackerError::InvalidResponse(
                "Could not convert the value of the 'interval' key into a byte string".to_owned(),
            )
        })?
        .get_data()
        .chunks_exact(6)
        .map(|chunk| {
            format!(
                "{}.{}.{}.{}:{}",
                chunk[0],
                chunk[1],
                chunk[2],
                chunk[3],
                u16::from_be_bytes([chunk[4], chunk[5]])
            )
        })
        .collect::<Vec<String>>();
    Ok(TrackerResponse::ok(interval, peers.into()))
}

fn urlencode_bytes(bytes: &[u8]) -> String {
    // TODO: add unit tests
    let mut result = String::with_capacity(3 * bytes.len());
    for byte in bytes {
        result.push('%');
        result.push_str(&hex::encode(&[*byte]));
    }
    result
}
