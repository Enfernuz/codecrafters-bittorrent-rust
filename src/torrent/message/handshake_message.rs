use core::fmt;
use std::rc::Rc;

// region:      --- HandshakeMessage
pub struct HandshakeMessage {
    info_hash: Rc<[u8; 20]>,
    peer_id: Rc<[u8; 20]>,
    extension_supported: bool,
}

// region:      --- Constructors
impl HandshakeMessage {
    pub fn new(info_hash: &Rc<[u8; 20]>, peer_id: &Rc<[u8; 20]>) -> HandshakeMessage {
        HandshakeMessage {
            info_hash: Rc::clone(info_hash),
            peer_id: Rc::clone(peer_id),
            extension_supported: false,
        }
    }

    pub fn new_magnet(info_hash: &Rc<[u8; 20]>, peer_id: &Rc<[u8; 20]>) -> HandshakeMessage {
        HandshakeMessage {
            info_hash: Rc::clone(info_hash),
            peer_id: Rc::clone(peer_id),
            extension_supported: true,
        }
    }
}
// endregion:   --- Constructors

// region:      --- Getters
impl HandshakeMessage {
    pub fn get_info_hash(&self) -> &Rc<[u8; 20]> {
        &self.info_hash
    }

    pub fn get_peer_id(&self) -> &Rc<[u8; 20]> {
        &self.peer_id
    }

    pub fn is_extension_supported(&self) -> bool {
        self.extension_supported
    }
}
// endregion:   --- Getters

// region:      --- Traits impl
impl From<&[u8; 68]> for HandshakeMessage {
    fn from(arr: &[u8; 68]) -> Self {
        let mut reserved: [u8; 8] = [0; 8];
        reserved.copy_from_slice(&arr[20..28]);
        let mut info_hash: [u8; 20] = [0; 20];
        info_hash.copy_from_slice(&arr[28..48]);
        let mut peer_id: [u8; 20] = [0; 20];
        peer_id.copy_from_slice(&arr[48..68]);
        HandshakeMessage {
            info_hash: info_hash.into(),
            peer_id: peer_id.into(),
            extension_supported: reserved == [0x00_u8, 0x00, 0x00, 0x00, 0x00, 0x10, 0x00, 0x00],
        }
    }
}

impl Into<[u8; 68]> for &HandshakeMessage {
    fn into(self) -> [u8; 68] {
        let mut result: [u8; 68] = [0; 68];
        result[0] = 19;
        result[1..20].copy_from_slice(b"BitTorrent protocol");
        // [20..28] are reserved bytes
        if self.extension_supported {
            result[20..28].copy_from_slice(&[0x00_u8, 0x00, 0x00, 0x00, 0x00, 0x10, 0x00, 0x00]);
        }
        result[28..48].copy_from_slice(self.info_hash.as_slice());
        result[48..68].copy_from_slice(self.peer_id.as_slice());
        result
    }
}

impl fmt::Display for HandshakeMessage {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Peer ID: {}\n", hex::encode(self.peer_id.as_ref()))
    }
}
// endregion:   --- Traits impl

// endregion:      --- HandshakeMessage
