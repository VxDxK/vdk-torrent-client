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
        let mut distribution = self.tracker_client.announce(meta.announce, params)?;
        let mut rng = rand::thread_rng();
        distribution.peers.shuffle(&mut rng);
        let peers = Arc::new(Mutex::new(VecDeque::from(distribution.peers)));
        let mut handles = vec![];
        for worker_id in 0..self.config.connection_numbers {
            let peers_a = peers.clone();
            let client_id = self.client_id.clone();
            let handle = std::thread::spawn(move || {
                let mut q = peers_a.lock().unwrap();
                if q.len() == 0 {
                    println!("thread {worker_id} closes due no peers");
                }
                let peer = q.pop_back().unwrap();
                drop(q);
                println!("{:#?}", peer);

                println!();
                let connection = TcpStream::connect_timeout(&peer.addr, Duration::from_secs(5));
                if connection.is_err() {
                    println!("timeout ");
                    return;
                }
                let connection = connection.unwrap();
                let bt_conn = PeerConnection::handshake(
                    connection,
                    &meta.info.info_hash.clone(),
                    &client_id,
                );
                match bt_conn {
                    Ok(_) => println!("conn ok"),
                    Err(e) => println!("err {}", e.to_string()),
                }
            });
            handles.push(handle);
        }
        while let Some(cur_thread) = handles.pop() {
            cur_thread.join().unwrap();
        }
        Ok(())
    }
}
