use crate::file::TorrentFile;
use crate::peer::PeerId;
use crate::tracker::{AnnounceParameters, RequestMode, TrackerClient, TrackerError};
use rand::seq::SliceRandom;
use std::io::{Read, Write};
use std::net::TcpStream;
use std::str::from_utf8;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum ClientError {
    #[error("Failed to retrieve peers from client {0}")]
    PeersRetrieve(#[from] TrackerError),
}
type Result<T> = std::result::Result<T, ClientError>;

#[derive(Default, Debug)]
pub struct Config {}

pub struct Client {
    client_id: PeerId,
    config: Config,
    tracker_client: Box<dyn TrackerClient>,
}

impl Client {
    pub fn new(client_id: PeerId, config: Config, tracker_client: Box<dyn TrackerClient>) -> Self {
        Self {
            client_id,
            config,
            tracker_client,
        }
    }

    pub fn download(&self, meta: TorrentFile) -> Result<()> {
        let mut params = AnnounceParameters::new(meta.info.info_hash);
        params
            .set_port(6881)
            .set_num_want(Some(100))
            .set_request_mode(RequestMode::Verbose);
        let mut distribution = self.tracker_client.announce(meta.announce, params)?;
        let mut rng = rand::thread_rng();
        distribution.peers.shuffle(&mut rng);
        let peer = distribution.peers.pop().unwrap();

        println!("{:#?}", peer);
        let mut message = Vec::new();
        message.push(19u8);
        message.extend_from_slice(b"BitTorrent protocol");
        message.extend_from_slice(b"\0\0\0\0\0\0\0\0");
        message.extend_from_slice(meta.info.info_hash.as_slice());
        message.extend_from_slice(self.client_id.as_slice());
        for byte in &message {
            print!("\\x{:0x} ", byte);
        }
        println!();
        assert_eq!(message.len(), 68);
        let mut connection = TcpStream::connect(peer.addr).unwrap();
        println!("connected");
        let _ = connection.write_all(message.as_slice()).unwrap();
        let mut response: [u8; 68] = [0; 68];
        let read = connection.read(response.as_mut_slice()).unwrap();
        println!("{read}");
        println!("{:?}", from_utf8(response.as_slice()[48..68].as_ref()));

        Ok(())
    }
}
