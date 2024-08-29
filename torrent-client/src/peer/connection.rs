use crate::peer::connection::ConnectionError::HandshakeFailed;
use crate::peer::connection::HandshakeMessageError::{ProtocolString, ProtocolStringLen};
use crate::peer::PeerId;
use crate::util::Sha1;
use bytes::{Buf, BufMut};
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

#[derive(Error, Debug)]
pub enum ConnectionError {
    #[error("BitTorrent handshake failed {0}")]
    HandshakeFailed(String),
    #[error("Error in parsing handshake response {0}")]
    HandshakeResponse(#[from] HandshakeMessageError),
    #[error(transparent)]
    IoKind(#[from] io::Error),
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
        let read_bytes = tcp_connection.read(bytes.as_mut())?;
        if read_bytes != 68 {
            return Err(HandshakeFailed(format!("Invalid bytes count received {read_bytes}")))
        }

        let response = HandshakeMessage::from_bytes(bytes)?;

        Ok(Self {
            tcp_connection,
            peer_id: response.peer_id,
        })
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
