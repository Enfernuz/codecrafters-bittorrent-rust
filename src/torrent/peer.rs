use std::{
    io::{Read, Write},
    net::{TcpStream, ToSocketAddrs},
    time::Duration,
};

use crate::error::Error;
use crate::error::Result;
use crate::torrent::HandshakeMessage;
use crate::torrent::Message;

// region:      --- Peer
pub struct Peer {
    socket: TcpStream,
}

// region:      --- Constructors
impl Peer {
    pub fn new<A: ToSocketAddrs>(addr: A) -> Result<Peer> {
        let socket = TcpStream::connect(addr).map_err(|err| Error::SocketError(err))?;
        socket
            .set_read_timeout(Some(Duration::from_secs(5)))
            .map_err(|err| Error::SocketError(err))?;
        Ok(Peer { socket })
    }
}
// endregion:   --- Constructors

// region:      --- API
impl Peer {
    pub fn handshake(&mut self, message: &HandshakeMessage) -> Result<HandshakeMessage> {
        let bytes: [u8; 68] = message.into();
        self.socket
            .write_all(&bytes)
            .map_err(|err| Error::SocketError(err))?;
        let mut buf: [u8; 68] = [0; 68];
        self.socket
            .read_exact(&mut buf)
            .map_err(|err| Error::SocketError(err))?;

        Ok((&buf).into())
    }

    pub fn receive_bitfield(&mut self) -> Result<Message> {
        println!("receive_bitfield: start");
        let mut length_buf = [0u8; 4];
        self.socket
            .read_exact(&mut length_buf)
            .map_err(|err| Error::SocketError(err))?;
        let length: u32 = u32::from_be_bytes(length_buf);
        let mut buf = vec![0u8; length as usize];
        self.socket
            .read_exact(&mut buf)
            .map_err(|err| Error::SocketError(err))?;

        let combined: &[u8] = &[length_buf.as_slice(), buf.as_slice()].concat();
        println!("receive_bitfield: received {} bytes", combined.len());

        let msg: Message = Message::try_from(combined)?;
        println!("receive_bitfield: {}", &msg);
        println!("receive_bitfield: end");
        Ok(msg)
    }

    pub fn send_interested(&mut self) -> Result<()> {
        println!("send_interested: start");
        let bytes: Box<[u8]> = (&Message::interested()).into();
        println!("send_interested: Sending {} bytes", bytes.len());
        self.socket
            .write_all(bytes.as_ref())
            .map_err(|err| Error::SocketError(err))?;
        println!("send_interested: end");

        Ok(())
    }

    pub fn receive_unchoke(&mut self) -> Result<Message> {
        println!("receive_unchoke: start");
        let mut buf: [u8; 5] = [0; 5];
        self.socket
            .read_exact(&mut buf[..])
            .map_err(|err| Error::SocketError(err))?;
        println!("receive_unchoke: received {} bytes", buf.len());
        let msg = Message::try_from(buf.as_slice())?;
        println!("receive_unchoke: {}", &msg);
        println!("receive_unchoke: end");
        Ok(msg)
    }

    pub fn send_piece_request(
        &mut self,
        piece_index: u32,
        begin: u32,
        block_length: u32,
    ) -> Result<()> {
        println!("send_piece_request: start");
        println!(
            "send_piece_request: piece_index={}, begin={}, block_length={}",
            piece_index, begin, block_length
        );
        let message = Message::request(piece_index, begin, block_length);
        let bytes: Box<[u8]> = (&message).into();
        self.socket
            .write_all(bytes.as_ref())
            .map_err(|err| Error::SocketError(err))?;
        println!("send_piece_request: end");

        Ok(())
    }

    pub fn receive_piece_block(&mut self, block_length: u32) -> Result<Box<[u8]>> {
        // Len{4}|Type{1}|Index{4}|Begin{4}|Piece{~}
        println!("receive_piece_block: start");
        println!("receive_piece_block: block_length={}", block_length);
        let capacity: usize = 13 + block_length as usize;
        let mut buf: Vec<u8> = vec![0; capacity];
        self.socket
            .read_exact(&mut buf)
            .map_err(|err| Error::SocketError(err))?;
        println!("receive_piece_block: received {} bytes", buf.len());

        let msg = Message::try_from(buf.as_slice())?;
        println!("receive_piece_block: {}", &msg);
        println!("receive_piece_block: end");
        Ok(msg.get_payload()[8..].into())
    }

    pub fn get_piece_block(
        &mut self,
        piece_index: u32,
        begin: u32,
        block_length: u32,
    ) -> Result<Box<[u8]>> {
        self.send_piece_request(piece_index, begin, block_length)?;
        self.receive_piece_block(block_length)
    }
}
// endregion:   --- API

// endregion:   --- Peer
