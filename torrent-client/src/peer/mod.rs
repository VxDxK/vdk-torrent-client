use std::borrow::Borrow;
use std::net::SocketAddr;
use rand::RngCore;

#[derive(Debug)]
pub struct PeerId(pub(crate) [u8; 20]);


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

impl Default for PeerId {
    fn default() -> Self {
        Self::new([0; 20])
    }
}