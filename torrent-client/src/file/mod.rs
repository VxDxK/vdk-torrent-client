use std::path::PathBuf;

use sha1::Digest;
use thiserror::Error;
use url::Url;

use bencode::{BencodeEncoder, BencodeError, BencodeList, BencodeString, Value};

use crate::file::TorrentError::{IntegerOutOfBound, InvalidInfoHash, MissingField};
use crate::util::Sha1;

type Result<T> = std::result::Result<T, TorrentError>;

#[derive(Debug)]
pub struct TorrentFile {
    pub announce: Url,
    pub info: Info,
}

#[derive(Debug)]
pub struct Info {
    pub files: Vec<File>,
    pub name: PathBuf,
    pub info_hash: Sha1,
    pub piece_length: usize,
    pub pieces: Vec<Sha1>,
}

#[derive(Debug)]
pub struct File {
    pub length: usize,
    pub path: PathBuf,
}

#[derive(Error, Debug)]
pub enum TorrentError {
    #[error("Bencode error: {0}")]
    Bencode(#[from] BencodeError),
    #[error("Url error: {0}")]
    Url(#[from] url::ParseError),
    #[error("Missing field: {0}")]
    MissingField(String),
    #[error("Invalid pieces length")]
    InvalidPiecesLength,
    #[error("Invalid info hash")]
    InvalidInfoHash,
    #[error("Invalid file list")]
    InvalidFileList,
    #[error("Integer out of bounds for field {0}")]
    IntegerOutOfBound(String),
}

// Byte sequence as slice :)
macro_rules! bss {
    ($bytes:expr) => {
        $bytes.as_slice()
    };
}

impl TorrentFile {
    pub fn from_bencode(mut dict: bencode::BencodeDict) -> Result<Self> {
        let announce = Url::parse(&String::try_from(
            dict.remove(bss!(b"announce"))
                .ok_or(MissingField("announce".to_string()))?,
        )?)?;
        let info = Info::from_bencode(
            dict.remove(bss!(b"info"))
                .ok_or(MissingField("info".to_string()))?
                .try_into()?,
        )?;
        Ok(Self { announce, info })
    }
}

impl Info {
    pub fn from_bencode(mut dict: bencode::BencodeDict) -> Result<Self> {
        let mut raw_info = Vec::new();
        BencodeEncoder::new(&mut raw_info).encode_dict(&dict);
        let info_hash = sha1::Sha1::digest(raw_info.as_slice()).into();
        let mut name = PathBuf::from(String::try_from(
            dict.remove(bss!(b"name"))
                .ok_or(MissingField("name".to_string()))?,
        )?);
        let piece_length = usize::try_from(i64::try_from(
            dict.remove(bss!(b"piece length"))
                .ok_or(MissingField("piece length".to_string()))?,
        )?)
        .map_err(|_| IntegerOutOfBound(String::from("piece_length")))?;
        let pieces: BencodeString = dict
            .remove(bss!(b"pieces"))
            .ok_or(MissingField("pieces".to_string()))?
            .try_into()?;
        if pieces.len() % 20 != 0 {
            return Err(InvalidInfoHash);
        }
        let pieces: Vec<[u8; 20]> = pieces
            .chunks_exact(20)
            .map(|chunk| <[u8; 20]>::try_from(chunk).unwrap())
            .collect();

        let mut files = vec![];
        if let Some(Value::Int(length)) = dict.get(bss!(b"length")) {
            // Single file mode
            let length =
                usize::try_from(*length).map_err(|_| IntegerOutOfBound(String::from("length")))?;
            files.push(File { length, path: name });
            name = PathBuf::default();
        } else {
            // Multi file mode
            let files_list: BencodeList = dict
                .remove(bss!(b"files"))
                .ok_or(MissingField("files".to_string()))?
                .try_into()?;
            for file in files_list {
                files.push(File::from_bencode(file.try_into()?)?);
            }
        }
        Ok(Info {
            files,
            name,
            info_hash,
            piece_length,
            pieces,
        })
    }
}

impl File {
    fn from_bencode(mut dict: bencode::BencodeDict) -> Result<Self> {
        let length = usize::try_from(i64::try_from(
            dict.remove(bss!(b"length"))
                .ok_or(MissingField("length".to_string()))?,
        )?)
        .map_err(|_| IntegerOutOfBound(String::from("length")))?;
        let path: BencodeList = dict
            .remove(bss!(b"path"))
            .ok_or(MissingField("path".to_string()))?
            .try_into()?;
        let path = path
            .into_iter()
            .map(String::try_from)
            .collect::<std::result::Result<PathBuf, _>>()?;
        Ok(File { length, path })
    }
}
