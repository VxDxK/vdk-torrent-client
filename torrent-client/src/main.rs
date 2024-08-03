use std::fs::File;
use std::io::Read;
use std::ops::Deref;
use clap::Parser;
use bencode::BencodeDict;
use crate::client::{Client, Config};
use crate::file::TorrentFile;
use crate::peer::PeerId;
use crate::tracker::HttpTracker;

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
    let torrent = TorrentFile::from_bencode(value).unwrap();
    
    println!("{:#?}", torrent.info.files.iter().map(|x| x.length).sum::<i64>());

    let client_id = PeerId::random();
    let tracker =  Box::new(HttpTracker::new(&client_id).unwrap());
    let client = Client::new(client_id, Config::default(), tracker);
    
    client.download(torrent);
}  
