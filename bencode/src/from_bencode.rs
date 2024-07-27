use crate::{Value, Result, BencodeString};

pub trait FromBencode {
    fn from_bencode(bencode: Value) -> Result<Self> where Self: Sized;
}

impl FromBencode for String {
    fn from_bencode(bencode: Value) -> Result<Self>
    where
        Self: Sized
    {
        Ok(String::from_utf8(BencodeString::try_from(bencode)?).map_err(|e| e.utf8_error())?)
    }
}