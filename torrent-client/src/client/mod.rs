mod worker;

use crate::file::TorrentFile;
use crate::peer::connection::PeerConnection;
use crate::peer::PeerId;
use crate::tracker::{AnnounceParameters, RequestMode, TrackerClient, TrackerError};
use rand::seq::SliceRandom;
use std::collections::VecDeque;
use std::net::TcpStream;
use std::sync::{Arc, Mutex};
use std::time::Duration;
use thiserror::Error;
use crate::client::worker::PeerWorker;

#[derive(Error, Debug)]
pub enum ClientError {
    #[error("Failed to retrieve peers from client {0}")]
    PeersRetrieve(#[from] TrackerError),
}
type Result<T> = std::result::Result<T, ClientError>;

#[derive(Default, Debug)]
pub struct Config {
    connection_numbers: usize,
}

impl Config {
    pub fn new(connection_numbers: usize) -> Self {
        if connection_numbers == 0 {
            panic!("connection numbers cannot be zero")
        }
        Self { connection_numbers }
    }
}

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
        let mut torrent_info = self.tracker_client.announce(meta.announce.clone(), params)?;
        torrent_info.peers.shuffle(&mut rand::thread_rng());
        let peers = Arc::new(Mutex::new(torrent_info.peers));
        let mut handles = vec![];

        let meta = Arc::new(meta);
        let id = Arc::new(self.client_id.clone());
        for _ in 0..self.config.connection_numbers {
            let mut worker = PeerWorker::new(peers.clone(), meta.clone(), id.clone());
            let handle = std::thread::spawn(move || {
                worker.go();
            });
            handles.push(handle);
        }
        while let Some(cur_thread) = handles.pop() {
            cur_thread.join().unwrap();
        }
        Ok(())
    }
}
