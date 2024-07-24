use thiserror::Error;
use url::Url;

use vdk_bencode::BencodeList;

pub struct TorrentFile {
    announce: Url,
    announce_list: Vec<Vec<Url>>,
    info: TorrentInfo,
}

struct TorrentInfo {
    files: Vec<File>,
    name: String,
    piece_length: usize,
}

struct File {
    length: usize,
    path: Vec<String>,
}

#[derive(Error, Debug)]
pub enum TorrentFileError {
    #[error("Bencode '{0}' error {1}")]
    InvalidBencode(String, String),

    #[error("Incorrect bencode value type {0}")]
    TypeMismatch(String),

    #[error("Error in format {0}")]
    InvalidFormat(String),
}

impl TorrentFile {
    pub fn try_from_bencode(mut dict: vdk_bencode::BencodeDict) -> Result<Self, Box<dyn std::error::Error>> {
        let announce = Self::get_announce(&mut dict)?;
        println!("announce '{announce}'");
        let announce_list = Self::get_announce_list(&mut dict)?;
        println!("announce_list '{announce_list:?}'");

        Ok(Self {
            announce,
            announce_list,
            info: TorrentInfo {
                files: vec![],
                name: "".to_string(),
                piece_length: 0,
            }
        })
    }
    fn get_announce(dict: &mut vdk_bencode::BencodeDict) -> Result<Url, TorrentFileError> {
        use TorrentFileError::*;
        let announce = String::from_utf8(dict.remove(&b"announce"[..])
            .ok_or(InvalidBencode("announce".into(), "no such field".into()))?
            .try_into().map_err(|_| TypeMismatch("type mismatch".to_string()))?).map_err(|x| InvalidFormat(x.to_string()))?;
        Url::parse(announce.as_str()).map_err(|x| InvalidFormat(x.to_string()))
    }

    fn get_announce_list(dict: &mut vdk_bencode::BencodeDict) -> Result<Vec<Vec<Url>>, TorrentFileError> {
        use TorrentFileError::*;
        match dict.remove(&b"announce_list"[..]) {
            None => Ok(vec![]),
            Some(list) => {
                let list: BencodeList = list.try_into().map_err(|_| TypeMismatch("type mismatch".to_string()))?;

                TryInto::<BencodeList>::try_into(list);

                // let list: Vec<Url>  = list.into_iter().map(|x| Url::parse(String::from_utf8(x.try_into().map_err(|_| TypeMismatch("types fuck".to_string()))?).map_err(|_err| InvalidFormat("".to_string()))?.as_str()).map_err(|err| InvalidFormat(err.to_string())) ).collect::<Result<Vec<Url>, TorrentFileError>>()?;
                // Ok(list)
                Err(InvalidFormat("dfd".to_string()))
            }
        }
    }
}