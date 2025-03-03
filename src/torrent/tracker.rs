use crate::bencode::decoders;
use crate::torrent::Torrent;

use crate::error::Error;
use crate::error::Result;

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

pub fn get(
    tracker_url: &str,
    info_hash: &[u8; 20],
    peer_id: &str,
    port: u16,
    uploaded: u64,
    downloaded: u64,
    left: u64,
) -> Result<TrackerResponse> {
    let urlencoded_info_hash: String = urlencode_bytes(info_hash);
    let url = format!("{tracker_url}?info_hash={urlencoded_info_hash}&peer_id={peer_id}&port={port}&uploaded={uploaded}&downloaded={downloaded}&left={left}&compact=1");
    let res = reqwest::blocking::get(url).map_err(|err| Error::TrackerHttpError(err))?;
    let bytes = res.bytes().map_err(|err| Error::TrackerHttpError(err))?;
    let (body, _) = decoders::decode(bytes.as_ref()).map_err(|err| Error::DecodeError(err))?;
    let body_dict = body
        .as_dict()
        .ok_or_else(|| Error::KeyNotFoundInTrackerResponse { key: "info".into() })?;
    if body_dict.contains_key("failure reason") {
        let reason = body_dict
            .get("failure reason")
            .ok_or_else(|| Error::KeyNotFoundInTrackerResponse {
                key: "failure reason".into(),
            })?
            .as_str()
            .ok_or_else(|| Error::TrackerFailureReasonIsNotUtf8)?;
        return Ok(TrackerResponse::failure(reason));
    }
    let interval = body_dict
        .get("interval")
        .ok_or_else(|| Error::KeyNotFoundInTrackerResponse {
            key: "interval".into(),
        })?
        .as_i64()
        .ok_or_else(|| Error::TrackerIntervalIsNotInteger)? as u32;
    let peers = body_dict
        .get("peers")
        .ok_or_else(|| Error::KeyNotFoundInTrackerResponse {
            key: "peers".into(),
        })?
        .as_byte_string()
        .ok_or_else(|| Error::TrackerIntervalIsNotByteString)?
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
