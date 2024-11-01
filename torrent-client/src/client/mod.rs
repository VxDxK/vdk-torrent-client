mod worker;

use crate::client::worker::Downloader;
use crate::client::ClientError::InboundConnection;
use crate::file::TorrentFile;
use crate::peer::{Peer, PeerId};
use crate::tracker::{AnnounceParameters, RequestMode, TrackerClient, TrackerError};
use std::borrow::Cow;
use std::collections::VecDeque;
use std::net::{IpAddr, Ipv4Addr, SocketAddr, TcpListener};
use std::sync::Arc;
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
        let torrent_info = self.tracker_client.announce(&meta.announce, params)?;
        let peers: VecDeque<Peer> = torrent_info.peers.into_iter().collect();

        let mut downloader = Downloader::new(peers, meta.info);
        downloader.run();

        Ok(())
    }
}
