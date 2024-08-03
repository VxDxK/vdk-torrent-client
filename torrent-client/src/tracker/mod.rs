use std::fmt::{Display, Formatter};
use std::net::IpAddr;
use std::time::Duration;
use percent_encoding::{NON_ALPHANUMERIC, percent_encode};
use thiserror::Error;
use url::Url;
use bencode::{BencodeDict, Value};
use torrent_client::Sha1;
use crate::peer::{Peer, PeerId};
use crate::tracker::TrackerError::{AnnounceRequestError, InternalError, TrackerResponse, UnsupportedProtocol};

type Result<T> = std::result::Result<T, TrackerError>;

#[derive(Error, Debug)]
pub enum TrackerError {
    #[error("Bencode error: {0}")]
    Bencode(#[from] bencode::BencodeError),

    #[error("Internal error: {0}")]
    InternalError(String),

    #[error("Unsupported protocol {0}")]
    UnsupportedProtocol(String),

    #[error("Announce request error {0}")]
    AnnounceRequestError(String),

    #[error("Tracker sent error as answer {0}")]
    TrackerResponse(String),
}

pub enum TrackerEvent {
    Started,
    Stopped,
    Completed,
}

impl Display for TrackerEvent {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        let string = match self {
            TrackerEvent::Started => "started",
            TrackerEvent::Stopped => "stopped",
            TrackerEvent::Completed => "completed",
        };
        write!(f, "{string}")
    }
}

pub enum RequestMode {
    Verbose,
    NoPeerId,
    Compact,
}

pub struct AnnounceParameters {
    info_hash: Sha1,
    port: u16,
    uploaded: usize,
    downloaded: usize,
    left: usize,
    request_mode: RequestMode,
    event: Option<TrackerEvent>,
    num_want: Option<usize>,
    ip: Option<IpAddr>,
}

impl AnnounceParameters {
    pub fn new(info_hash: Sha1) -> Self {
        Self { info_hash, port: 0, uploaded: 0, downloaded: 0, left: 0, request_mode: RequestMode::Verbose, event: None, num_want: None, ip: None }
    }

    pub fn set_port(&mut self, port: u16) -> &mut Self {
        self.port = port;
        self
    }
    pub fn set_uploaded(&mut self, uploaded: usize) -> &mut Self {
        self.uploaded = uploaded;
        self
    }
    pub fn set_downloaded(&mut self, downloaded: usize) -> &mut Self {
        self.downloaded = downloaded;
        self
    }
    pub fn set_left(&mut self, left: usize) -> &mut Self {
        self.left = left;
        self
    }
    pub fn set_request_mode(&mut self, request_mode: RequestMode) -> &mut Self {
        self.request_mode = request_mode;
        self
    }
    pub fn set_event(&mut self, event: Option<TrackerEvent>) -> &mut Self {
        self.event = event;
        self
    }
    pub fn set_num_want(&mut self, num_want: Option<usize>) -> &mut Self {
        self.num_want = num_want;
        self
    }
    pub fn set_ip(&mut self, ip: Option<IpAddr>) -> &mut Self {
        self.ip = ip;
        self
    }
}


#[derive(Debug)]
pub struct AnnounceResponse {
    interval: Duration,
    min_interval: Option<Duration>,
    complete: Option<i64>,
    incomplete: Option<i64>,
    peers: Vec<Peer>,
}

pub struct ScrapeResponse;


pub trait TrackerClient {
    fn announce(&self, url: Url, params: AnnounceParameters) -> Result<AnnounceResponse>;
    fn scrape(&self) -> Result<ScrapeResponse>;
}

pub struct HttpTracker {
    http_client: reqwest::blocking::Client,
    encoded_peer_id: String,
}

impl HttpTracker {
    pub fn new(peer_id: &PeerId) -> Result<Self> {
        let http_client = reqwest::blocking::ClientBuilder::new()
            .user_agent("reqwest/0.12")
            .build().map_err(|x| InternalError(format!("failed to create http client {}", x)))?;
        let encoded_peer_id = percent_encode(peer_id.as_ref(), NON_ALPHANUMERIC).to_string();
        Ok(Self { http_client, encoded_peer_id })
    }

    fn build_announce_url(&self, mut url: Url, request: AnnounceParameters) -> Url {
        let info_hash = percent_encode(request.info_hash.as_slice(), NON_ALPHANUMERIC);

        let query = format!("info_hash={}&peer_id={}", info_hash, self.encoded_peer_id);
        let new_query = if let Some(url_query) = url.query() {
            format!("{url_query}&{query}")
        } else {
            query
        };
        url.set_query(Some(new_query.as_str()));
        url.query_pairs_mut()
            .append_pair("port", request.port.to_string().as_str())
            .append_pair("uploaded", request.uploaded.to_string().as_str())
            .append_pair("downloaded", request.downloaded.to_string().as_str())
            .append_pair("left", request.left.to_string().as_str());
        match request.request_mode {
            RequestMode::Verbose => {}
            RequestMode::NoPeerId => { url.query_pairs_mut().append_key_only("no_peer_id"); }
            RequestMode::Compact => { url.query_pairs_mut().append_pair("compact", "1"); }
        }

        if let Some(event) = request.event {
            url.query_pairs_mut().append_pair("event", event.to_string().as_str());
        }

        if let Some(num_want) = request.num_want {
            url.query_pairs_mut().append_pair("numwant", num_want.to_string().as_str());
        }

        if let Some(ip) = request.ip {
            url.query_pairs_mut().append_pair("ip", ip.to_string().as_str());
        }

        println!("'{url}'");

        url
    }
}


impl TrackerClient for HttpTracker {
    fn announce(&self, url: Url, params: AnnounceParameters) -> Result<AnnounceResponse> {
        if !(url.scheme() != "http" || url.scheme() != "https") {
            return Err(UnsupportedProtocol(String::from(url.scheme())));
        }
        let tracker_response = self.http_client
            .get(self.build_announce_url(url, params))
            .send()
            .map_err(|e| AnnounceRequestError(format!("send request to tracker failed {e}")))?;

        println!("{}", tracker_response.status());

        let mut bencode: BencodeDict = bencode::from_slice(tracker_response.bytes()
            .map_err(|e| AnnounceRequestError(format!("failed to retrieve response body {e}")))?
            .to_vec().as_slice())?
            .try_into()?;

        if let Some(failure_reason) = bencode.remove(b"failure reason".as_ref()) {
            let error = match failure_reason {
                Value::String(string) => String::from_utf8(string).unwrap_or(String::from("tracker response error, unknown string format")),
                x => format!("error getting tracker 'failure_reason' reason expected string got {}", x.name()),
            };
            return Err(TrackerResponse(error));
        }


        for (k, v) in bencode {
            println!("k: '{}' vt: {}", String::from_utf8(k).unwrap(), v.name());
        }


        Err(InternalError("unimplemented".to_string()))
    }

    fn scrape(&self) -> Result<ScrapeResponse> {
        unimplemented!("Tracker scraping not implemented for http client")
    }
}