use core::fmt;
use std::u8;

use crate::error::Error;
use crate::error::Result;

// region:      --- Message
pub struct Message {
    tag: MessageTag,
    payload: Box<[u8]>,
}

// region:      --- Constructors
impl Message {
    pub fn choke() -> Self {
        Self::new_empty(MessageTag::Choke)
    }

    pub fn unchoke() -> Self {
        Self::new_empty(MessageTag::Unchoke)
    }

    pub fn interested() -> Self {
        Self::new_empty(MessageTag::Interested)
    }

    pub fn not_interested() -> Self {
        Self::new_empty(MessageTag::NotInterested)
    }

    pub fn have(piece_index: u32) -> Self {
        Self::new(MessageTag::Have, piece_index.to_be_bytes().into())
    }

    pub fn bitfield(bitfield: &[u8]) -> Self {
        Self::new(MessageTag::Bitfield, bitfield.into())
    }

    pub fn request(index: u32, begin: u32, length: u32) -> Self {
        Self::new(
            MessageTag::Request,
            [
                index.to_be_bytes(),
                begin.to_be_bytes(),
                length.to_be_bytes(),
            ]
            .concat()
            .into(),
        )
    }

    pub fn piece(index: u32, begin: u32, block: &[u8]) -> Self {
        Self::new(
            MessageTag::Piece,
            [
                index.to_be_bytes().as_slice(),
                begin.to_be_bytes().as_slice(),
                block,
            ]
            .concat()
            .into(),
        )
    }

    pub fn cancel(index: u32, begin: u32, length: u32) -> Self {
        Self::new(
            MessageTag::Cancel,
            [
                index.to_be_bytes(),
                begin.to_be_bytes(),
                length.to_be_bytes(),
            ]
            .concat()
            .into(),
        )
    }

    pub fn extended(extended_message_id: u8, data: &[u8]) -> Self {
        Self::new(
            MessageTag::Extended,
            [&[extended_message_id], data].concat().into(),
        )
    }

    fn new(tag: MessageTag, payload: Box<[u8]>) -> Self {
        Message { tag, payload }
    }

    fn new_empty(tag: MessageTag) -> Self {
        Self::new(tag, [].into())
    }
}
// endregion:   --- Constructors

// region:      --- Getters
impl Message {
    pub fn get_tag(&self) -> &MessageTag {
        &self.tag
    }

    pub fn get_payload(&self) -> &Box<[u8]> {
        &self.payload
    }
}
// endregion:   --- Getters

// region:      --- Traits impl

impl TryFrom<&[u8]> for Message {
    type Error = Error;

    fn try_from(slice: &[u8]) -> Result<Self> {
        let arr_len: u32 = slice.len() as u32;
        if arr_len < 5 {
            Err(Error::NotEnoughData {
                minimum_length: 5,
                actual_length: arr_len,
            })?
        }
        let message_len = u32::from_be_bytes([slice[0], slice[1], slice[2], slice[3]]);
        let tag: MessageTag = slice[4].try_into()?;
        match tag {
            MessageTag::Bitfield => {
                Ok(Message::bitfield(&slice[5..5 + (message_len as usize - 1)]))
            }
            MessageTag::Unchoke => Ok(Message::unchoke()),
            MessageTag::Piece => {
                if message_len < 9 {
                    Err(Error::InvalidMessageLength {
                        minimum_length: 9,
                        actual_length: message_len,
                    })?
                }
                if arr_len < message_len + 4 {
                    Err(Error::NotEnoughData {
                        minimum_length: message_len + 4,
                        actual_length: arr_len,
                    })?
                }
                let index = u32::from_be_bytes([slice[5], slice[6], slice[7], slice[8]]);
                let begin = u32::from_be_bytes([slice[9], slice[10], slice[11], slice[12]]);
                let block = if message_len == 9 {
                    &[]
                } else {
                    &slice[13..message_len as usize + 4]
                };
                // println!("Writing block size of {}", block.len());
                Ok(Message::piece(index, begin, block))
            }
            MessageTag::Extended => {
                let extended_message_id: u8 = slice[5];
                Ok(Message::extended(
                    extended_message_id,
                    &slice[6..6 + (message_len as usize - 2)],
                ))
            }
            other => Err(Error::MessageParsingNotImplemented(other))?,
        }
    }
}

impl Into<Box<[u8]>> for &Message {
    fn into(self) -> Box<[u8]> {
        let payload_length = self.payload.len();
        let length = (1 + payload_length) as u32;
        let mut result: Vec<u8> = vec![];
        result.extend_from_slice(&length.to_be_bytes());
        result.push((&self.tag).into());
        if payload_length > 0 {
            result.extend_from_slice(&self.payload.as_ref());
        }

        result.into_boxed_slice()
    }
}

impl fmt::Display for Message {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "Type: {:?}, Payload length: {}, \n",
            self.tag,
            self.payload.len()
        )
    }
}
// endregion:   --- Traits impl

// endregion:   --- Message

// region:      --- MessageTag
#[derive(Debug)]
pub enum MessageTag {
    Choke,
    Unchoke,
    Interested,
    NotInterested,
    Have,
    Bitfield,
    Request,
    Piece,
    Cancel,
    Port,
    Extended,
}

// region:      --- Traits impl
impl TryFrom<u8> for MessageTag {
    type Error = Error;

    fn try_from(value: u8) -> Result<Self> {
        match value {
            0 => Ok(Self::Choke),
            1 => Ok(Self::Unchoke),
            2 => Ok(Self::Interested),
            3 => Ok(Self::NotInterested),
            4 => Ok(Self::Have),
            5 => Ok(Self::Bitfield),
            6 => Ok(Self::Request),
            7 => Ok(Self::Piece),
            8 => Ok(Self::Cancel),
            9 => Ok(Self::Port),
            20 => Ok(Self::Extended),
            other => Err(Error::UnrecognizedMessageTag(other)),
        }
    }
}

impl Into<u8> for &MessageTag {
    fn into(self) -> u8 {
        match self {
            MessageTag::Choke => 0,
            MessageTag::Unchoke => 1,
            MessageTag::Interested => 2,
            MessageTag::NotInterested => 3,
            MessageTag::Have => 4,
            MessageTag::Bitfield => 5,
            MessageTag::Request => 6,
            MessageTag::Piece => 7,
            MessageTag::Cancel => 8,
            MessageTag::Port => 9,
            MessageTag::Extended => 20,
        }
    }
}
// endregion:   --- Traits impl

// endregion:   --- MessageTag
