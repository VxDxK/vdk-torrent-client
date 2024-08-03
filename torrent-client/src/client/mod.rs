use crate::file::TorrentFile;
use crate::peer::PeerId;
use crate::tracker::{AnnounceParameters, RequestMode, TrackerClient};

#[derive(Default, Debug)]
pub struct Config {}

pub struct Client {
    client_id: PeerId,
    config: Config,
    tracker_client: Box<dyn TrackerClient>,
}


impl Client {
    pub fn new(client_id: PeerId, config: Config, tracker_client: Box<dyn TrackerClient>) -> Self {
        Self { client_id, config, tracker_client }
    }

    pub fn download(&self, meta: TorrentFile) {
        let mut params = AnnounceParameters::new(meta.info.info_hash);
        params.set_port(6881)
            .set_request_mode(RequestMode::Verbose);
        let r = 
            self.tracker_client.announce(meta.announce, params);
        println!("{:?}", r);
    }
}