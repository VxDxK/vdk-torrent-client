use std::fs::File;
use std::io::Read;
use clap::Parser;
use bencode::BencodeDict;
use crate::client::{Client, Config, PeerId};
use crate::file::TorrentFile;

mod file;
mod cli;
mod client;

fn main() {
    let cli = cli::Args::parse();
    let mut file = File::open(cli.torrent_file).unwrap();
    let mut data = Vec::new();
    file.read_to_end(&mut data).unwrap();
    let value: BencodeDict = bencode::from_slice(data.as_mut_slice()).unwrap().try_into().unwrap();
    let torrent  = TorrentFile::from_bencode(value).unwrap();
    let client = Client::new(PeerId::random(), Config::default());
    client.download(torrent);
}  
