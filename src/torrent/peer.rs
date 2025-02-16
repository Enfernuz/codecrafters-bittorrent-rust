use std::{
    fmt,
    io::{BufReader, BufWriter, Read, Write},
    net::TcpStream,
    rc::Rc,
};

pub struct Peer {
    addr: String,
    stream: Box<TcpStream>,
}

pub struct HandshakeMessage {
    info_hash: Rc<[u8; 20]>,
    peer_id: Rc<[u8; 20]>,
}

pub struct PeerMessage {
    message_id: PeerMessageId,
    payload: Box<[u8]>,
}

#[derive(Debug)]
pub enum PeerMessageId {
    Choke,
    Unchoke,
    Interested,
    NotInterested,
    Have,
    Bitfield,
    Request,
    Piece,
    Cancel,
    //     0 - choke
    // 1 - unchoke
    // 2 - interested
    // 3 - not interested
    // 4 - have
    // 5 - bitfield
    // 6 - request
    // 7 - piece
    // 8 - cancel
}

impl PeerMessageId {
    pub fn as_byte(&self) -> u8 {
        match self {
            Self::Choke => 0,
            Self::Unchoke => 1,
            Self::Interested => 2,
            Self::NotInterested => 3,
            Self::Have => 4,
            Self::Bitfield => 5,
            Self::Request => 6,
            Self::Piece => 7,
            Self::Cancel => 8,
        }
    }

    pub fn from_byte(b: u8) -> Self {
        match b {
            0 => Self::Choke,
            1 => Self::Unchoke,
            2 => Self::Interested,
            3 => Self::NotInterested,
            4 => Self::Have,
            5 => Self::Bitfield,
            6 => Self::Request,
            7 => Self::Piece,
            8 => Self::Cancel,
            other => panic!("PeerMessageId::from_byte - {}", other),
        }
    }
}

impl Peer {
    pub fn new(addr: &str) -> Peer {
        let stream = TcpStream::connect(addr).expect(&format!("Could not connect to {}", addr));
        println!("Created TCP connection with {addr}");
        Peer {
            addr: addr.to_owned(),
            stream: Box::new(stream),
        }
    }

    pub fn handshake(&mut self, message: &HandshakeMessage) -> HandshakeMessage {
        let stream: &mut TcpStream = self.stream.as_mut();
        stream
            .write_all(message.as_bytes().as_ref())
            .expect(&format!("Could not write to TCP socket for {}", &self.addr));
        let mut buf: [u8; 68] = [0; 68];
        stream.read_exact(&mut buf).expect(&format!(
            "Could not read Handshake from TCP socket for {}",
            &self.addr
        ));
        HandshakeMessage::parse(&buf)
    }

    pub fn receive_bitfield(&mut self) {
        println!("receive_bitfield: start");
        let stream: &mut TcpStream = self.stream.as_mut();
        let mut buf: [u8; 512] = [0; 512];
        let recv = stream.read(&mut buf).unwrap();
        println!("receive_bitfield: received {} bytes", recv);
        let msg = PeerMessage::from_bytes(&buf);
        println!("receive_bitfield: {}", &msg);
        println!("receive_bitfield: end");
        // .expect(&format!("Could not read BitField from TCP socket for {}", &self.addr));
    }

    pub fn send_interested(&mut self) {
        println!("send_interested: start");
        let message = PeerMessage {
            message_id: PeerMessageId::Interested,
            payload: [].into(),
        };
        let stream: &mut TcpStream = self.stream.as_mut();
        let bytes = message.as_bytes();
        println!("send_interested: Sending {} bytes", bytes.len());
        stream
            .write_all(message.as_bytes().as_ref())
            .expect(&format!("Could not write to TCP socket for {}", &self.addr));
        println!("send_interested: end");
    }

    pub fn receive_unchoke(&mut self) {
        println!("receive_unchoke: start");
        let stream: &mut TcpStream = self.stream.as_mut();
        let mut buf: [u8; 512] = [0; 512];
        let mut received: usize = 0;
        while received == 0 {
            let recv = stream.read(&mut buf).unwrap();
            received += recv;
        }

        println!("receive_unchoke: received {} bytes", received);
        let msg = PeerMessage::from_bytes(&buf);
        println!("receive_unchoke: {}", &msg);
        println!("receive_unchoke: end");
        // .expect(&format!("Could not read Unchoke from TCP socket for {}", &self.addr));
    }

    pub fn send_piece_request(&mut self, piece_index: u32, begin: u32, block_length: u32) {
        println!("send_piece_request: start");
        println!(
            "send_piece_request: piece_index={}, begin={}, block_length={}",
            piece_index, begin, block_length
        );
        let mut payload: Vec<u8> = vec![];
        payload.extend_from_slice(&piece_index.to_be_bytes());
        payload.extend_from_slice(&begin.to_be_bytes());
        payload.extend_from_slice(&block_length.to_be_bytes());

        let message = PeerMessage {
            message_id: PeerMessageId::Request,
            payload: payload.into(),
        };
        let stream: &mut TcpStream = self.stream.as_mut();
        stream
            .write_all(message.as_bytes().as_ref())
            .expect(&format!("Could not write to TCP socket for {}", &self.addr));
        println!("send_piece_request: end");
        // dbg!("Sent piece request: {:?}", message.as_bytes());
    }

    pub fn receive_piece_block(&mut self, block_length: u32) -> Box<[u8]> {
        // Len{4}|Type{1}|Index{4}|Begin{4}|Piece{~}
        println!("receive_piece_block: start");
        println!("receive_piece_block: block_length={}", block_length);
        let stream: &mut TcpStream = self.stream.as_mut();
        let capacity: usize = 13 + block_length as usize;
        let mut received: Vec<u8> = Vec::with_capacity(capacity);
        let mut reader = BufReader::new(stream);
        let mut buf: [u8; 16 * 1024] = [0; 16 * 1024];
        while received.len() < capacity {
            // println!("receive_piece_block: start read iter.");
            let recv = reader.read(&mut buf).unwrap();
            received.extend_from_slice(&buf[..recv]);
            println!("receive_piece_block: end read iter (received {recv} bytes).");
        }
        println!("receive_piece_block: received {} bytes", received.len());

        let msg = PeerMessage::from_bytes(&received);
        println!("receive_piece_block: {}", &msg);
        println!("receive_piece_block: end");
        msg.payload
    }
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

impl fmt::Display for PeerMessage {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "Type: {:?}, Payload length: {}, \n",
            self.message_id,
            self.payload.len()
        )
    }
}

impl PeerMessage {
    pub fn as_bytes(&self) -> Box<[u8]> {
        let mut v: Vec<u8> = vec![];
        let payload_length = self.payload.len();
        let length = (1 + payload_length) as u32;
        v.extend_from_slice(&length.to_be_bytes());
        v.push(self.message_id.as_byte());
        if payload_length > 0 {
            v.extend_from_slice(&self.payload.as_ref());
        }

        v.into_boxed_slice()
    }

    pub fn from_bytes(data: &[u8]) -> Self {
        println!("from_bytes: data.len()={}", data.len());
        // TODO: length check
        let length = u32::from_be_bytes([data[0], data[1], data[2], data[3]]) as usize;
        let _type = PeerMessageId::from_byte(data[4]);
        println!(
            "from_bytes: PeerMessage: length={}, type={:?}",
            length, &_type
        );
        match _type {
            PeerMessageId::Bitfield => {
                let payload = &data[5..5 + length];
                return PeerMessage {
                    message_id: PeerMessageId::Bitfield,
                    payload: payload.into(),
                };
            }
            PeerMessageId::Unchoke => {
                return PeerMessage {
                    message_id: PeerMessageId::Unchoke,
                    payload: [].into(),
                }
            }
            PeerMessageId::Piece => {
                let payload = &data[13..];
                return PeerMessage {
                    message_id: PeerMessageId::Piece,
                    payload: payload.into(),
                };
            }

            other => panic!("PeerMessage::from_bytes - KEK - {:?}", other),
        }
    }
}
