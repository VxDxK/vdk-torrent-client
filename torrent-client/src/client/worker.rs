use crate::file::Info;
use crate::peer::connection::{ConnectionError, PeerConnection};
use crate::peer::{Peer, PeerId};
use std::collections::VecDeque;
use std::net::TcpStream;
use std::sync::{mpsc, Arc, Mutex};
use std::time::Duration;

pub struct Task {}

pub struct Downloader {
    peers: VecDeque<Peer>,
    peer_id: Arc<PeerId>,
    info: Arc<Info>,
}

impl Downloader {
    pub fn run(&mut self) {
        let peer = self.peers.pop_front().unwrap();
    }

    pub fn new<T>(peers: T, info: Info) -> Self
    where
        T: Into<VecDeque<Peer>>,
    {
        Self {
            peers: peers.into(),
            peer_id: Arc::new(PeerId::random()),
            info: Arc::new(info),
        }
    }
}

pub struct Peering {
    received: Arc<Mutex<mpsc::Receiver<Peer>>>,
    peer_id: Arc<PeerId>,
    info: Arc<Info>,
}

impl Peering {
    fn connect(&self, peer: &Peer) -> Result<PeerConnection, ConnectionError> {
        let tcp = TcpStream::connect_timeout(&peer.addr, Duration::from_secs(5))?;
        let connection = PeerConnection::handshake(tcp, &self.info.info_hash, &self.peer_id)?;
        Ok(connection)
    }

    fn run(&mut self) {
        let ch = self.received.lock().unwrap();
        if let Ok(peer) = ch.recv() {
            // if let Ok(conn) = self.connect(&peer) {
            //     self.work(conn);
            // }
        }
    }

    fn work(&mut self, conn: PeerConnection) {}
}
