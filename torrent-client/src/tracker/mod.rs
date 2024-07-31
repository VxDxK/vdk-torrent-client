use std::time::Duration;
use thiserror::Error;
use url::Url;
use torrent_client::Sha1;
use crate::peer::{Peer, PeerId};
use crate::tracker::TrackerError::InternalError;

type Result<T> = std::result::Result<T, TrackerError>;

pub enum TrackerEvent {
    Started,
    Stopped,
    Completed,
}

struct TrackerRequest {
    tracker: Url,
    info_hash: Sha1,
    peer_id: PeerId,
    port: u16,
    uploaded: usize,
    downloaded: usize,
    left: usize,
    compact: bool,
    event: TrackerEvent,
}

struct TrackerResponse {
    interval: Duration,
    min_interval: Duration,
    complete: i64,
    incomplete: i64,
    peers: Vec<Peer>,
}

#[derive(Error, Debug)]
enum TrackerError {
    #[error("Bencode error: {0}")]
    Bencode(#[from] bencode::BencodeError),
    
    #[error("Internal error: {0}")]
    InternalError(String)
}

trait Tracker {
    fn get_peers(meta: &TrackerRequest) -> Result<TrackerResponse>;
}

struct HttpTracker {
    http_client: reqwest::blocking::Client,
}

impl HttpTracker {
    pub fn new() -> Result<Self> {
        let http_client = reqwest::blocking::ClientBuilder::new()
            .user_agent("reqwest/0.12")
            .build().map_err(|x| InternalError(String::from("failed to create http client")))?;

        Ok(Self { http_client })
    }
    
    fn create_url() -> Url {
        todo!()
    }
}

impl Tracker for HttpTracker {
    fn get_peers(meta: &TrackerRequest) -> Result<TrackerResponse> {
        
        todo!()
    }
}