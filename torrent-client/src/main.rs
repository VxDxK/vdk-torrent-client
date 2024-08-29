use crate::client::{Client, Config};
use crate::file::TorrentFile;
use crate::peer::PeerId;
use crate::tracker::HttpTracker;
use bencode::BencodeDict;
use clap::Parser;
use std::fs::File;
use std::io::Read;

mod cli;
mod client;
mod file;
mod peer;
mod tracker;
mod util;

fn main() {
    let cli = cli::Args::parse();
    let mut file = File::open(cli.torrent_file).unwrap();
    let mut data = Vec::new();
    file.read_to_end(&mut data).unwrap();
    let value: BencodeDict = bencode::from_slice(data.as_mut_slice())
        .unwrap()
        .try_into()
        .unwrap();
    let torrent = TorrentFile::from_bencode(value).unwrap();

    println!(
        "{:#?}",
        torrent.info.files.iter().map(|x| x.length).sum::<usize>()
    );

    let client_id = PeerId::random();
    let tracker = Box::new(HttpTracker::new(&client_id).unwrap());
    let client = Client::new(client_id, Config::new(25), tracker);

    let res = client.download(torrent);
    println!("{res:#?}");
}
