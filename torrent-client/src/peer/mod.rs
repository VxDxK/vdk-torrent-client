use rand::RngCore;
use std::borrow::Borrow;
use std::net::SocketAddr;
use std::ops::Deref;

#[derive(Debug, Clone)]
pub struct PeerId([u8; 20]);

#[derive(Debug)]
pub struct Peer {
    peer_id: Option<PeerId>,
    addr: SocketAddr,
}

impl PeerId {
    pub fn new(peer_id: [u8; 20]) -> Self {
        Self(peer_id)
    }

    pub fn random() -> Self {
        let mut peer_id = [0; 20];
        rand::thread_rng().fill_bytes(&mut peer_id);
        Self::new(peer_id)
    }
}

impl Borrow<[u8]> for PeerId {
    fn borrow(&self) -> &[u8] {
        self.0.as_slice()
    }
}

impl AsRef<[u8]> for PeerId {
    fn as_ref(&self) -> &[u8] {
        self.0.as_slice()
    }
}

impl Deref for PeerId {
    type Target = [u8; 20];

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl Default for PeerId {
    fn default() -> Self {
        Self::new([0; 20])
    }
}

impl Peer {
    pub fn new(peer_id: Option<PeerId>, addr: SocketAddr) -> Self {
        Self { peer_id, addr }
    }
}
