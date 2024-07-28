use std::borrow::Borrow;
use percent_encoding::{NON_ALPHANUMERIC, percent_encode};
use rand::RngCore;
use crate::file::TorrentFile;

#[derive(Default, Debug)]
pub struct Config {}

#[derive(Default, Debug)]
pub struct Client {
    peer_id: PeerId,
    config: Config,
}

#[derive(Debug)]
pub struct PeerId([u8; 20]);

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
        Self::random()
    }
}

impl Client {
    pub fn new(peer_id: PeerId, config: Config) -> Self {
        Self { peer_id, config }
    }

    pub fn download(&self, meta_info: TorrentFile) {
        let client = reqwest::blocking::Client::new();

        let response = client.get(meta_info.announce)
            .query(&[
                ("info_hash", percent_encode(meta_info.info.info_hash.as_slice(), NON_ALPHANUMERIC).to_string()),
                ("peer_id", percent_encode(self.peer_id.borrow(), NON_ALPHANUMERIC).to_string()),
            ])
            .query(&[
                ("port", "6139"),
                ("uploaded", "0"),
                ("downloaded", "0"),
                ("compact", "1"),
                ("left", "0"),
            ])
            .send().unwrap();
        println!("url: {}", response.url());
        println!("response: {:#?}", response);
    }
}