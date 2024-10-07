mod worker;
mod task;

use crate::client::worker::Peering;
use crate::client::ClientError::InboundConnection;
use crate::file::{Info, TorrentFile};
use crate::peer::{Peer, PeerId};
use crate::tracker::{AnnounceParameters, RequestMode, TrackerClient, TrackerError};
use rand::seq::SliceRandom;
use std::borrow::Cow;
use std::net::{IpAddr, Ipv4Addr, SocketAddr, TcpListener};
use std::sync::mpsc;
use std::sync::{Arc, Mutex};
use thiserror::Error;

#[derive(Error, Debug)]
pub enum ClientError {
    #[error("Failed to retrieve peers from client {0}")]
    PeersRetrieve(#[from] TrackerError),

    #[error("Inbound connection error {0}")]
    InboundConnection(Cow<'static, str>),
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

struct Leeching {
    free_peers: mpsc::Sender<Peer>,
    working_peers: Vec<Arc<Peering>>,
}

impl Leeching {
    fn new(client_id: Arc<PeerId>, meta: Info, peers: Vec<Peer>) {
        let (mut sender, receiver) = mpsc::channel::<Peer>();
        let receiver = Arc::new(Mutex::new(receiver));

        peers.into_iter().for_each(|e| {
            let _ = sender.send(e);
        });

        let mut workers = Vec::new();

        let mut meta = Arc::new(meta);
        for _ in 0..20 {
            let mut worker = Peering::new(receiver.clone(), meta.clone(), client_id.clone());
            let worker = std::thread::spawn(move || {
                worker.go();
            });
            workers.push(worker);
        }
        while let Some(t) = workers.pop() {
            t.join().unwrap();
        }
    }
}

pub struct Client {
    client_id: Arc<PeerId>,
    config: Config,
    tracker_client: Box<dyn TrackerClient>,
    inbound: TcpListener,
}

impl Client {
    pub fn new(
        client_id: PeerId,
        config: Config,
        tracker_client: Box<dyn TrackerClient>,
    ) -> Result<Self> {
        let inbound =
            TcpListener::bind(SocketAddr::new(IpAddr::V4(Ipv4Addr::new(0, 0, 0, 0)), 6881))
                .map_err(|e| InboundConnection(Cow::Owned(e.to_string())))?;
        Ok(Self {
            client_id: Arc::new(client_id),
            config,
            tracker_client,
            inbound,
        })
    }

    pub fn download(&self, meta: TorrentFile) -> Result<()> {
        let mut params = AnnounceParameters::new(&meta.info.info_hash);
        params
            .set_port(6881)
            .set_num_want(Some(100))
            .set_request_mode(RequestMode::Compact);
        let mut torrent_info = self.tracker_client.announce(&meta.announce, params)?;
        // println!("files: {}", meta.info.files.len());
        // println!("pieces: {}", meta.info.pieces.len());
        // println!("piece length: {}", meta.info.piece_length);
        // Leeching::new(self.client_id.clone(), meta.info, torrent_info.peers);
        Ok(())
    }
}
