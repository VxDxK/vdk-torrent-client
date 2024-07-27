use std::fs::File;
use std::io::Read;
use clap::Parser;
use bencode::BencodeDict;
use bencode::from_bencode::FromBencode;
use crate::file::TorrentFile;

mod file;
mod cli;

fn main() {
    let cli = cli::Args::parse();
    let mut file = File::open(cli.torrent_file).unwrap();
    let mut data = Vec::new();
    file.read_to_end(&mut data).unwrap();
    let value: BencodeDict = bencode::from_slice(data.as_mut_slice()).unwrap().try_into().unwrap();
    let torrent  = TorrentFile::from_bencode(value).unwrap();
    print!("{torrent:#?}");
    // String::from_bencode(bencode::Value::Dict(value)).unwrap();
}
