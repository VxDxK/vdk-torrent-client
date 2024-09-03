use std::net::TcpStream;
use std::sync::{Arc, Mutex};
use std::time::Duration;
use crate::file::TorrentFile;
use crate::peer::connection::{Message, PeerConnection};
use crate::peer::{Peer, PeerId};

pub struct PeerWorker {
    peers: Arc<Mutex<Vec<Peer>>>,
    meta: Arc<TorrentFile>,
    client_id: Arc<PeerId>
}

impl PeerWorker {
    pub fn go(&mut self) {
        loop {
            let mut q = self.peers.lock().unwrap();
            if q.len() == 0 {
                return;
            }
            let peer = q.pop().unwrap();
            drop(q);
            let connection = TcpStream::connect_timeout(&peer.addr, Duration::from_secs(5));
            if connection.is_err() {
                continue;
            }
            let connection =
                PeerConnection::handshake(connection.unwrap(), &self.meta.info.info_hash, &self.client_id);
            if connection.is_err() {
                continue;
            }
            let mut connection = connection.unwrap();
            loop {
                match connection.recv() {
                    Ok(message) => {
                        println!("message {message}");
                        match message {
                            Message::KeepAlive => {}
                            Message::Choke => {}
                            Message::UnChoke => {

                            }
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

    pub fn new(peers: Arc<Mutex<Vec<Peer>>>, meta: Arc<TorrentFile>, client_id: Arc<PeerId>) -> Self {
        Self { peers, meta, client_id }
    }
}