use percent_encoding::{NON_ALPHANUMERIC, percent_encode};
use bencode::{BencodeDict, Value};
use crate::file::TorrentFile;
use crate::peer::PeerId;

#[derive(Default, Debug)]
pub struct Config {}

#[derive(Default, Debug)]
pub struct Client {
    peer_id: PeerId,
    config: Config,
}


impl Client {
    pub fn new(peer_id: PeerId, config: Config) -> Self {
        Self { peer_id, config }
    }

    pub fn download(&self, meta_info: TorrentFile) {
        let client = reqwest::blocking::ClientBuilder::new()
            .user_agent("reqwest/0.12")
            .build().unwrap();

        let info_hash_url_encoded = percent_encode(meta_info.info.info_hash.as_slice(), NON_ALPHANUMERIC);
        let peer_id_url_encoded = percent_encode(self.peer_id.0.as_slice(), NON_ALPHANUMERIC);

        let mut url = meta_info.announce.clone();
        url.set_query(Some(format!("info_hash={}&peer_id={}", info_hash_url_encoded, peer_id_url_encoded).as_str()));

        println!("url: {url}");

        let response = client.get(url)
            .query(&[
                ("port", 6881),
                ("uploaded", 0),
                ("downloaded", 0),
                ("compact", 1),
                ("left", 4697096192i64),
            ])
            .send().unwrap();
        println!("url: {}", response.url());
        println!("response: {:#?}", response);
        let bencode: BencodeDict = bencode::from_slice(response.bytes().unwrap().to_vec().as_slice()).unwrap().try_into().unwrap();
        for (k, v) in bencode {
            match v {
                Value::String(_) => {
                    println!("{}: {:?}", String::from_utf8(k).unwrap(), String::try_from(v));
                }
                _ => {
                    println!("{}: {:#?}", String::from_utf8(k).unwrap(), v);
                }
            }
        }
    }
}