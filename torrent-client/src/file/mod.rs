use thiserror::Error;
use url::Url;

use bencode::{BencodeError, BencodeString};
use bencode::from_bencode::FromBencode;

use crate::file::TorrentError::MissingField;

type Result<T> = std::result::Result<T, TorrentError>;
type Sha1 = [u8; 20];

#[derive(Debug)]
pub struct TorrentFile {
    announce: Url,
    info: Info,
}

#[derive(Debug)]
struct Info {
    files: Vec<File>,
    name: String,
    piece_length: i64, //TODO: change to usize
    pieces: Vec<Sha1>,
}

#[derive(Debug)]
struct File {
    length: usize,
    path: Vec<String>,
}

#[derive(Error, Debug)]
pub enum TorrentError {
    #[error("Bencode error: {0}")]
    Bencode(#[from] BencodeError),
    #[error("Url error: {0}")]
    Url(#[from] url::ParseError),
    #[error("Missing field: {0}")]
    MissingField(String),
    #[error("Invalid field type: {0}")]
    InvalidFieldType(String),
    #[error("Invalid pieces length")]
    InvalidPiecesLength,
    #[error("Invalid info hash")]
    InvalidInfoHash,
    #[error("Invalid file list")]
    InvalidFileList,
}

// Byte sequence as slice :)
macro_rules! bss {
    ($bytes:expr) => {
        $bytes.as_slice()
    };
}

impl TorrentFile {
    pub fn from_bencode(mut dict: bencode::BencodeDict) -> Result<Self> {
        let announce = Url::parse(&String::from_bencode(dict.remove(bss!(b"announce")).ok_or(MissingField("announce".to_string()))?)?)?;
        let info = Info::from_bencode(dict.remove(bss!(b"info")).ok_or(MissingField("info".to_string()))?.try_into()?)?;
        Ok(Self { announce, info })
    }
}

impl Info {
    pub fn from_bencode(mut dict: bencode::BencodeDict) -> Result<Self> {
        let name = String::from_bencode(dict.remove(bss!(b"name")).ok_or(MissingField("name".to_string()))?)?;
        let piece_length: i64 = dict.remove(bss!(b"piece length")).ok_or(MissingField("piece length".to_string()))?.try_into()?;
        let pieces: BencodeString = dict.remove(bss!(b"pieces")).ok_or(MissingField("pieces".to_string()))?.try_into()?;

        println!("{}", pieces.len());
        println!("{:#0x?}", &pieces[0..19]);

        //stub
        Ok(Info {
            files: vec![],
            name,
            piece_length,
            pieces: vec![],
        })
    }
}