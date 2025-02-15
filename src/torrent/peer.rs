use std::{fmt, rc::Rc};

pub struct HandshakeMessage {
    info_hash: Rc<[u8; 20]>,
    peer_id: Rc<[u8; 20]>,
}

impl HandshakeMessage {
    pub fn new(info_hash: &Rc<[u8; 20]>, peer_id: &Rc<[u8; 20]>) -> HandshakeMessage {
        HandshakeMessage {
            info_hash: info_hash.clone(),
            peer_id: peer_id.clone(),
        }
    }

    pub fn as_bytes(&self) -> [u8; 68] {
        let mut result: [u8; 68] = [0; 68];
        result[0] = 19;
        for (i, byte) in "BitTorrent protocol".as_bytes().iter().enumerate() {
            result[1 + i] = *byte;
        }
        for (i, byte) in self.info_hash.iter().enumerate() {
            result[28 + i] = *byte;
        }
        for (i, byte) in self.peer_id.iter().enumerate() {
            result[48 + i] = *byte;
        }
        result
    }

    pub fn parse(data: &[u8; 68]) -> Self {
        let mut info_hash: [u8; 20] = [0; 20];
        info_hash.copy_from_slice(&data[28..48]);
        let mut peer_id: [u8; 20] = [0; 20];
        peer_id.copy_from_slice(&data[48..68]);
        HandshakeMessage {
            info_hash: info_hash.into(),
            peer_id: peer_id.into(),
        }
    }
}

impl fmt::Display for HandshakeMessage {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Peer ID: {}\n", hex::encode(self.peer_id.as_ref()))
    }
}
