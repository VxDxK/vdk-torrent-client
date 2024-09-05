use crate::peer::connection::ConnectionError::{
    HandshakeFailed, MessageId, PayloadLength, Todo, UnexpectedEOF,
};
use crate::peer::connection::HandshakeMessageError::{ProtocolString, ProtocolStringLen};
use crate::peer::PeerId;
use crate::util::{BitField, Sha1};
use bytes::{Buf, BufMut};
use std::cmp::PartialEq;
use std::fmt::{format, Display, Formatter};
use std::io;
use std::io::{Read, Write};
use std::net::TcpStream;
use thiserror::Error;

type Result<T> = std::result::Result<T, ConnectionError>;

static BIT_TORRENT_PROTOCOL_STRING: &[u8; 19] = b"BitTorrent protocol";

#[derive(Error, Debug)]
enum HandshakeMessageError {
    #[error("Invalid protocol string(pstr) length, expected 19, but got {0}")]
    ProtocolStringLen(u8),
    #[error("Unexpected protocol string, expected \"BitTorrent protocol\", but got {0}")]
    ProtocolString(String),
}

#[derive(Debug, PartialEq, Clone)]
struct HandshakeMessage {
    // need to replace with appropriate structure
    extension_bytes: [u8; 8],
    info_hash: Sha1,
    peer_id: PeerId,
}

impl HandshakeMessage {
    fn to_bytes(&self) -> Box<[u8; 68]> {
        let mut res = Box::new([0; 68]);
        res[0] = 19u8;
        res[1..20].copy_from_slice(BIT_TORRENT_PROTOCOL_STRING.as_slice());
        res[20..28].copy_from_slice(self.extension_bytes.as_slice());
        res[28..48].copy_from_slice(self.info_hash.as_slice());
        res[48..68].copy_from_slice(self.peer_id.as_slice());
        res
    }

    fn from_bytes(raw: Box<[u8; 68]>) -> std::result::Result<Self, HandshakeMessageError> {
        let pstr_len = raw[0];
        if pstr_len != 19 {
            return Err(ProtocolStringLen(pstr_len));
        }
        let pstr: [u8; 19] = raw[1..20].try_into().unwrap();
        if pstr.as_slice() != BIT_TORRENT_PROTOCOL_STRING {
            return Err(ProtocolString(
                String::from_utf8_lossy(pstr.as_slice()).to_string(),
            ));
        }
        let extension_bytes: [u8; 8] = raw[20..28].try_into().expect("Slice with incorrect length");
        let info_hash: [u8; 20] = raw[28..48].try_into().expect("Slice with incorrect length");
        let peer_id: [u8; 20] = raw[48..68].try_into().expect("Slice with incorrect length");

        Ok(Self::new(extension_bytes, info_hash, PeerId::new(peer_id)))
    }

    pub fn new(extension_bytes: [u8; 8], info_hash: Sha1, peer_id: PeerId) -> Self {
        Self {
            extension_bytes,
            info_hash,
            peer_id,
        }
    }
}

impl From<HandshakeMessage> for Box<[u8; 68]> {
    fn from(value: HandshakeMessage) -> Self {
        value.to_bytes()
    }
}

#[derive(Error, Debug)]
pub enum ConnectionError {
    #[error("BitTorrent handshake failed {0}")]
    HandshakeFailed(String),
    #[error("Error in parsing handshake response {0}")]
    HandshakeResponse(#[from] HandshakeMessageError),
    #[error(transparent)]
    IoKind(#[from] io::Error),
    #[error("Unexpected end of file")]
    UnexpectedEOF,
    #[error("Undefined message id {0}")]
    MessageId(u8),
    #[error("Unexpected payload length {0}")]
    PayloadLength(usize),
    #[error("todo")]
    Todo,
}

pub struct PeerConnection {
    tcp_connection: TcpStream,
    peer_id: PeerId,
}

impl PeerConnection {
    pub fn handshake(
        mut tcp_connection: TcpStream,
        info_hash: &Sha1,
        peer_id: &PeerId,
    ) -> Result<Self> {
        let mut bytes =
            HandshakeMessage::new([0; 8], info_hash.clone(), peer_id.clone()).to_bytes();
        let _ = tcp_connection.write_all(bytes.as_ref())?;
        let _ = tcp_connection.read_exact(bytes.as_mut())?;
        let response = HandshakeMessage::from_bytes(bytes)?;

        Ok(Self {
            tcp_connection,
            peer_id: response.peer_id,
        })
    }

    pub fn recv(&mut self) -> Result<Message> {
        let mut length_prefix = [0u8; 4];
        let _ = self.tcp_connection.read_exact(&mut length_prefix)?;
        let length_prefix = u32::from_be_bytes(length_prefix);
        if length_prefix == 0 {
            return Ok(Message::KeepAlive);
        }
        let mut data = Vec::with_capacity(length_prefix as usize);
        data.resize(length_prefix as usize, 0);
        let _ = self.tcp_connection.read_exact(data.as_mut_slice())?;
        let message = Message::try_from(data.as_slice())?;
        Ok(message)
    }

    pub fn send(&mut self, message: Message) -> Result<()> {
        let bytes: Vec<u8> = message.into();
        self.tcp_connection.write_all(bytes.as_slice())?;
        Ok(())
    }
}

#[derive(Debug, Clone)]
pub struct BlockRequest {
    index: u32,
    begin: u32,
    length: u32,
}

impl BlockRequest {
    pub fn new(index: u32, begin: u32, length: u32) -> Self {
        Self {
            index,
            begin,
            length,
        }
    }
}

impl TryFrom<&[u8]> for BlockRequest {
    type Error = ConnectionError;

    fn try_from(mut value: &[u8]) -> std::result::Result<Self, Self::Error> {
        if value.len() != 12 {
            return Err(PayloadLength(value.len()));
        }
        Ok(BlockRequest::new(
            value.get_u32_ne(),
            value.get_u32_ne(),
            value.get_u32_ne(),
        ))
    }
}

#[derive(Debug, Clone)]
pub struct Piece {
    index: u32,
    begin: u32,
    data: Vec<u8>,
}

impl Piece {
    pub fn new(index: u32, begin: u32, data: Vec<u8>) -> Self {
        Self { index, begin, data }
    }
}

impl TryFrom<&[u8]> for Piece {
    type Error = ConnectionError;

    fn try_from(mut value: &[u8]) -> std::result::Result<Self, Self::Error> {
        if value.len() < 8 {
            return Err(PayloadLength(value.len()));
        }
        Ok(Piece::new(
            value.get_u32_ne(),
            value.get_u32_ne(),
            value.to_vec(),
        ))
    }
}

#[derive(Debug, Clone)]
pub enum Message {
    KeepAlive,
    Choke,
    UnChoke,
    Interested,
    NotInterested,
    Have(u32),
    Bitfield(Vec<BitField>),
    Request(BlockRequest),
    Piece(Piece),
    Cancel(BlockRequest),
    Port(u16),
}

impl Display for Message {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            Message::KeepAlive => write!(f, "KeepAlive"),
            Message::Choke => write!(f, "Choke"),
            Message::UnChoke => write!(f, "UnChoke"),
            Message::Interested => write!(f, "Interested"),
            Message::NotInterested => write!(f, "NotInterested"),
            Message::Have(have) => write!(f, "Have({})", have),
            Message::Bitfield(_) => write!(f, "Bitfield"),
            Message::Request(_) => write!(f, "Request"),
            Message::Piece(_) => write!(f, "Piece"),
            Message::Cancel(_) => write!(f, "Cancel"),
            Message::Port(port) => write!(f, "Port({})", port),
        }
    }
}

impl From<Message> for Vec<u8> {
    fn from(value: Message) -> Self {
        let mut result = Vec::new();
        result.extend_from_slice(1u32.to_ne_bytes().as_slice());

        result
    }
}

impl TryFrom<&[u8]> for Message {
    type Error = ConnectionError;

    fn try_from(mut value: &[u8]) -> std::result::Result<Self, Self::Error> {
        let id = value.get(0).ok_or(UnexpectedEOF)?.to_owned();
        value = &value[1..];

        let message: Message = match id {
            0 => Message::Choke,
            1 => Message::UnChoke,
            2 => Message::Interested,
            3 => Message::NotInterested,
            4 => Message::Have(u32::from_be_bytes(
                value
                    .get(0..4)
                    .ok_or(UnexpectedEOF)?
                    .try_into()
                    .map_err(|_| UnexpectedEOF)?,
            )),
            5 => Message::Bitfield(
                value
                    .into_iter()
                    .map(|x| BitField::new(x.to_owned()))
                    .collect::<Vec<_>>(),
            ),
            6 => Message::Request(BlockRequest::try_from(value)?),
            7 => Message::Piece(Piece::try_from(value)?),
            8 => Message::Cancel(BlockRequest::try_from(value)?),
            9 => Message::Port(u16::from_be_bytes(
                value
                    .get(0..2)
                    .ok_or(UnexpectedEOF)?
                    .try_into()
                    .map_err(|_| UnexpectedEOF)?,
            )),
            _ => return Err(MessageId(id)),
        };

        Ok(message)
    }
}

#[cfg(test)]
mod tests {
    use crate::peer::connection::{HandshakeMessage, BIT_TORRENT_PROTOCOL_STRING};
    use crate::peer::PeerId;
    use bytes::{BufMut, BytesMut};
    use rand::RngCore;

    #[test]
    fn handshake_message_as_bytes() {
        let mut extensions_bytes = [0; 8];
        rand::thread_rng().fill_bytes(&mut extensions_bytes);
        let mut info_hash = [0; 20];
        rand::thread_rng().fill_bytes(&mut info_hash);
        let peed_id = PeerId::random();

        let mut bytes = BytesMut::with_capacity(68);
        bytes.put_u8(19u8);
        bytes.extend_from_slice(BIT_TORRENT_PROTOCOL_STRING);
        bytes.extend_from_slice(extensions_bytes.as_slice());
        bytes.extend_from_slice(info_hash.as_slice());
        bytes.extend_from_slice(peed_id.as_ref());

        let message = HandshakeMessage::new(extensions_bytes, info_hash, peed_id);
        let message_bytes = message.to_bytes();

        assert_eq!(bytes.as_ref(), message_bytes.as_slice());
    }

    #[test]
    fn handshake_message_from_bytes() {
        let mut extensions_bytes = [0; 8];
        rand::thread_rng().fill_bytes(&mut extensions_bytes);
        let mut info_hash = [0; 20];
        rand::thread_rng().fill_bytes(&mut info_hash);
        let peed_id = PeerId::random();

        let mut bytes = BytesMut::with_capacity(68);
        bytes.put_u8(19u8);
        bytes.extend_from_slice(BIT_TORRENT_PROTOCOL_STRING);
        bytes.extend_from_slice(extensions_bytes.as_slice());
        bytes.extend_from_slice(info_hash.as_slice());
        bytes.extend_from_slice(peed_id.as_ref());

        let message = HandshakeMessage::new(extensions_bytes, info_hash, peed_id);

        let message_from_bytes =
            HandshakeMessage::from_bytes(bytes.to_vec().try_into().unwrap()).unwrap();

        assert_eq!(message_from_bytes, message)
    }
}
