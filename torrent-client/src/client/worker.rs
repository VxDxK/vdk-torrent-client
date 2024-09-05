use crate::file::Info;
use crate::peer::connection::{Message, PeerConnection};
use crate::peer::{Peer, PeerId};
use std::net::TcpStream;
use std::sync::{mpsc, Arc, Mutex};
use std::time::Duration;

pub struct Peering {
    free_peers: Arc<Mutex<mpsc::Receiver<Peer>>>,
    meta: Arc<Info>,
    client_id: Arc<PeerId>,
    choked: bool,
}

impl Peering {
    fn connect(&mut self, peer: Peer) -> Option<PeerConnection> {
        let tcp = TcpStream::connect_timeout(&peer.addr, Duration::from_secs(5)).ok()?;
        let connection =
            PeerConnection::handshake(tcp, &self.meta.info_hash, &self.client_id).ok()?;
        Some(connection)
    }

    pub fn go(&mut self) {
        loop {
            let guard = self.free_peers.lock().unwrap();
            let peer = match guard.recv() {
                Ok(peer) => peer,
                Err(_) => break,
            };
            drop(guard);
            let mut connection = match self.connect(peer) {
                None => continue,
                Some(c) => c,
            };
            loop {
                match connection.recv() {
                    Ok(message) => {
                        println!("message {message}");
                        match message {
                            Message::KeepAlive => {}
                            Message::Choke => self.choked = true,
                            Message::UnChoke => self.choked = false,
                            Message::Interested => {}
                            Message::NotInterested => {}
                            Message::Have(_) => {}
                            Message::Bitfield(_) => {}
                            Message::Request(_) => {}
                            Message::Piece(_) => {}
                            Message::Cancel(_) => {}
                            Message::Port(_) => {}
                        }
                    }
                    Err(err) => {
                        println!("error {err:?}")
                    }
                }
            }
        }
    }

    pub fn new(
        peers: Arc<Mutex<mpsc::Receiver<Peer>>>,
        meta: Arc<Info>,
        client_id: Arc<PeerId>,
    ) -> Self {
        Self {
            free_peers: peers,
            meta,
            client_id,
            choked: true,
        }
    }
}
