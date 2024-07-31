use std::fs::File;
use std::io::Read;
use clap::Parser;
use bencode::BencodeDict;
use crate::client::{Client, Config};
use crate::file::TorrentFile;
use crate::peer::PeerId;

mod file;
mod cli;
mod client;
mod tracker;
mod peer;

fn main() {
    let cli = cli::Args::parse();
    let mut file = File::open(cli.torrent_file).unwrap();
    let mut data = Vec::new();
    file.read_to_end(&mut data).unwrap();
    let value: BencodeDict = bencode::from_slice(data.as_mut_slice()).unwrap().try_into().unwrap();
    let torrent  = TorrentFile::from_bencode(value).unwrap();
    println!("{}", torrent.announce);
    println!("{:#0x?}", torrent.info.info_hash);
    println!("{:#?}", torrent.info.files.iter().map(|x| x.length).sum::<i64>());
    let client = Client::new(PeerId::random(), Config::default());
    client.download(torrent);
}  
