use std::collections::BTreeMap;
use std::str::{from_utf8, FromStr};
use std::{i64, usize};

pub type BencodeInt = i64;
pub type BencodeString = Vec<u8>;
pub type BencodeList = Vec<Value>;
pub type BencodeDict = BTreeMap<BencodeString, Value>;

#[derive(Debug, PartialEq)]
pub enum Value {
    Int(BencodeInt),
    String(BencodeString),
    List(BencodeList),
    Dict(BencodeDict),
}

impl Value {
    pub fn is_int(&self) -> bool {
        if let Self::Int(_) = self {
            return true;
        }
        false
    }

    pub fn is_string(&self) -> bool {
        if let Self::String(_) = self {
            return true;
        }
        false
    }

    pub fn is_list(&self) -> bool {
        if let Self::List(_) = self {
            return true;
        }
        false
    }

    pub fn is_dict(&self) -> bool {
        if let Self::Dict(_) = self {
            return true;
        }
        false
    }
}

impl TryFrom<Value> for BencodeInt {
    type Error = ();
    fn try_from(value: Value) -> Result<Self, Self::Error> {
        if let Value::Int(int) = value {
            return Ok(int)
        }
        Err(())
    }
}

impl TryFrom<Value> for BencodeString {
    type Error = ();
    fn try_from(value: Value) -> Result<Self, Self::Error> {
        if let Value::String(string) = value {
            return Ok(string)
        }
        Err(())
    }
}

impl TryFrom<Value> for BencodeList {
    type Error = ();
    fn try_from(value: Value) -> Result<Self, Self::Error> {
        if let Value::List(list) = value {
            return Ok(list)
        }
        Err(())
    }
}

impl TryFrom<Value> for BencodeDict {
    type Error = ();
    fn try_from(value: Value) -> Result<Self, Self::Error> {
        if let Value::Dict(dict) = value {
            return Ok(dict)
        }
        Err(())
    }
}

impl From<Vec<u8>> for Value {
    fn from(value: Vec<u8>) -> Self {
        Value::String(value)
    }
}

impl From<String> for Value {
    fn from(value: String) -> Self {
        Value::String(value.into_bytes())
    }
}

impl From<BencodeInt> for Value {
    fn from(value: BencodeInt) -> Self {
        Value::Int(value)
    }
}

pub fn from_slice(data: &[u8]) -> Option<Value> {
    let mut parser = BencodeDecoder::new(data);
    parser.parse()
}

struct BencodeDecoder<'a> {
    data: &'a [u8],
}

impl<'a> BencodeDecoder<'a> {
    fn new(data: &'a [u8]) -> Self {
        Self { data }
    }

    fn parse_str(&mut self) -> Option<BencodeString> {
        let mut len_str = 0;
        for byte in self.data.iter() {
            match byte {
                b':' => {
                    break;
                }
                b'0'..=b'9' => {
                    len_str += 1;
                }
                _ => {
                    return None;
                }
            }
        }
        let len = usize::from_str(from_utf8(self.data.get(..len_str)?).ok()?).ok()?;
        let start_of_string = len_str + 1;
        let vec_data = self.data.get(start_of_string..start_of_string + len)?.to_vec();
        self.data = &self.data[start_of_string + len..];
        Some(vec_data)
    }

    fn parse_int(&mut self) -> Option<BencodeInt> {
        let mut len: usize = 0;
        for (num, byte) in self.data.iter().enumerate() {
            match (byte, num) {
                (b'i', 0) => {
                    continue
                }
                (b'0'..=b'9', _) | (b'-', 1) => {
                    len += 1;
                }
                (b'e', _) => {
                    break;
                }
                _ => {
                    return None
                }
            }
        }
        let ans = i64::from_str(from_utf8(self.data.get(1..1 + len)?).ok()?).ok()?;
        self.data = &self.data.get(len + 2..)?;
        Some(ans)
    }

    fn parse_list(&mut self) -> Option<BencodeList> {
        match self.data.first() {
            Some(b'l') => { self.data = &self.data[1..]; }
            _ => return None,
        }
        let mut ans: BencodeList = Vec::new();
        while *self.data.first()? != b'e' {
            if let Some(element) = self.parse() {
                ans.push(element);
            } else {
                return None;
            }
        }
        self.data = &self.data[1..];
        Some(ans)
    }

    fn parse_dict(&mut self) -> Option<BencodeDict> {
        match self.data.first() {
            Some(b'd') => { self.data = &self.data[1..]; }
            _ => return None,
        }

        let mut ans: BencodeDict = BTreeMap::new();
        while *self.data.first()? != b'e' {
            match (self.parse(), self.parse()) {
                (Some(Value::String(key)), Some(value)) => {
                    ans.insert(key, value);
                }
                (_, _) => {
                    return None;
                }
            }
        }
        self.data = &self.data[1..];
        Some(ans)
    }

    fn parse(&mut self) -> Option<Value> {
        match self.data.first()? {
            b'i' => self.parse_int().map(Value::Int),
            b'l' => self.parse_list().map(Value::List),
            b'd' => self.parse_dict().map(Value::Dict),
            b'0'..=b'9' => self.parse_str().map(Value::String),
            _ => None
        }
    }
}

pub fn into_vec(value: &Value) -> Vec<u8> {
    let mut res = Vec::new();
    let mut encoder = BencodeEncoder::new(&mut res);
    encoder.encode(value);
    res
}

struct BencodeEncoder<'a> {
    data: &'a mut Vec<u8>,
}

impl<'a> BencodeEncoder<'a> {
    fn new(data: &'a mut Vec<u8>) -> Self {
        Self { data }
    }

    fn encode(&mut self, value: &Value) {
        match value {
            Value::Int(int) => self.encode_int(int.to_owned()),
            Value::String(str) => self.encode_bytes(str.as_slice()),
            Value::List(list) => {
                self.data.push(b'l');
                for item in list { self.encode(item) }
                self.data.push(b'e');
            }
            Value::Dict(dict) => {
                self.data.push(b'd');
                for (key, value) in dict {
                    self.encode_bytes(key);
                    self.encode(value);
                }
                self.data.push(b'e');
            }
        }
    }

    fn encode_int(&mut self, int: BencodeInt) {
        self.data.push(b'i');
        self.data.extend_from_slice(int.to_string().as_bytes());
        self.data.push(b'e');
    }

    fn encode_bytes(&mut self, bytes: &[u8]) {
        self.data.extend_from_slice(bytes.len().to_string().as_bytes());
        self.data.push(b':');
        self.data.extend_from_slice(bytes);
    }
}

#[cfg(test)]
mod tests {
    use crate::Value::{Dict, Int, List, String};
    use super::*;

    #[test]
    fn parse_valid_string() {
        let data = b"5:aboba";
        let mut parser = BencodeDecoder::new(data);
        let str = parser.parse_str();
        assert_eq!(str, Some(Vec::from("aboba")));
        assert_eq!(parser.data.len(), 0);
    }

    #[test]
    fn parse_zero_string() {
        let data = b"0:";
        let mut parser = BencodeDecoder::new(data);
        let str = parser.parse_str();
        assert_eq!(str, Some(Vec::from("")));
        assert_eq!(parser.data.len(), 0);
    }

    #[test]
    fn parse_invalid_string() {
        let data = b"5:abfd";
        let mut parser = BencodeDecoder::new(data);
        let str = parser.parse_str();
        assert_eq!(str, None);
    }

    #[test]
    fn parse_valid_int() {
        let data = b"i452e";
        let mut parser = BencodeDecoder::new(data);
        let int = parser.parse_int();
        assert_eq!(int, Some(452));
        assert_eq!(parser.data.len(), 0);
    }

    #[test]
    fn parse_invalid_int() {
        let data = b"i4f52e";
        let mut parser = BencodeDecoder::new(data);
        let int = parser.parse_int();
        assert_eq!(int, None);
    }

    #[test]
    fn parse_invalid_int_without_ending_e() {
        let data = b"i452";
        let mut parser = BencodeDecoder::new(data);
        let int = parser.parse_int();
        assert_eq!(int, None);
    }

    #[test]
    fn parse_valid_list() {
        let data = Vec::from(b"l4:spami42ee");
        let mut parser = BencodeDecoder::new(data.as_slice());
        let list = parser.parse_list();
        assert_eq!(list, Some(vec![Value::String(Vec::from(b"spam")), Value::Int(42)]));
        assert_eq!(parser.data.len(), 0);
    }

    #[test]
    fn parse_invalid_list_without_ending_e() {
        let data = Vec::from(b"l4:spami42e");
        let mut parser = BencodeDecoder::new(data.as_slice());
        let list = parser.parse_list();
        assert_eq!(list, None);
    }

    #[test]
    fn parse_invalid_list_with_incorrect_element() {
        let data = Vec::from(b"l4:spamuperi42ee");
        let mut parser = BencodeDecoder::new(data.as_slice());
        let list = parser.parse_list();
        assert_eq!(list, None);
    }

    #[test]
    fn parse_shizo_inherit_structs() {
        let data = b"lli43e5:abobaed3:bari52eee";
        let list = from_slice(data);
        let mut map: BencodeDict = BTreeMap::new();
        map.insert(b"bar".to_vec(), Int(52));
        assert_eq!(list, Some(List(vec![List(vec![Int(43), String(b"aboba".to_vec())]), Dict(map)])));
    }

    #[test]
    fn parse_valid_dict() {
        let data = Vec::from(b"d3:bar4:spam3:fooi42ee");
        let mut parser = BencodeDecoder::new(data.as_slice());
        let mut map_dict = BencodeDict::new();
        map_dict.insert(b"bar".to_vec(), Value::String(b"spam".to_vec()));
        map_dict.insert(b"foo".to_vec(), Value::Int(42));
        let dict = parser.parse_dict();
        assert_eq!(dict, Some(map_dict));
        assert_eq!(parser.data.len(), 0);
    }

    #[test]
    fn parse_invalid_dict_without_ending_e() {
        let data = Vec::from(b"d3:bar4:spam3:fooi42e");
        let mut parser = BencodeDecoder::new(data.as_slice());
        let dict = parser.parse_dict();
        assert_eq!(dict, None);
    }

    #[test]
    fn encode_string() {
        let mut vec = Vec::new();
        let mut encoder = BencodeEncoder::new(&mut vec);
        encoder.encode(&String(b"aboba".to_vec()));
        assert_eq!(vec.as_slice(), b"5:aboba");
    }

    #[test]
    fn encode_int_positive() {
        let mut vec = Vec::new();
        let mut encoder = BencodeEncoder::new(&mut vec);
        encoder.encode(&Int(50));
        assert_eq!(vec.as_slice(), b"i50e");
    }

    #[test]
    fn encode_int_negative() {
        let mut vec = Vec::new();
        let mut encoder = BencodeEncoder::new(&mut vec);
        encoder.encode(&Int(-354));
        assert_eq!(vec.as_slice(), b"i-354e");
    }

    #[test]
    fn encode_int_zero() {
        let mut vec = Vec::new();
        let mut encoder = BencodeEncoder::new(&mut vec);
        encoder.encode(&Int(0));
        assert_eq!(vec.as_slice(), b"i0e");
    }

    #[test]
    fn encode_list() {
        let mut vec = Vec::new();
        let mut encoder = BencodeEncoder::new(&mut vec);
        encoder.encode(&List(vec![345.into()]));
        assert_eq!(vec.as_slice(), b"li345ee");
    }

    #[test]
    fn encode_dict() {
        let mut vec = Vec::new();
        let mut encoder = BencodeEncoder::new(&mut vec);
        let mut map: BencodeDict = BTreeMap::new();
        map.insert(b"first".to_vec(), 3546.into());
        map.insert(b"second".to_vec(), "go here dgf".to_owned().into());
        encoder.encode(&Dict(map));
        assert_eq!(vec.as_slice(), b"d5:firsti3546e6:second11:go here dgfe");
    }

    #[test]
    fn into_vec() {
        let mut map: BencodeDict = BTreeMap::new();
        map.insert(b"first".to_vec(), 3546.into());
        map.insert(b"second".to_vec(), "go here dgf".to_owned().into());
        assert_eq!(crate::into_vec(&Dict(map)).as_slice(), b"d5:firsti3546e6:second11:go here dgfe");
    }
}