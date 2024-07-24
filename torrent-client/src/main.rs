use std::fs::File;
use std::io::Read;
use clap::Parser;
use vdk_bencode::BencodeDict;
use crate::structs::TorrentFile;

mod structs;
mod cli;

fn main() {
    let cli = cli::Args::parse();
    let mut file = File::open(cli.torrent_file).unwrap();
    let mut data = Vec::new();
    file.read_to_end(&mut data).unwrap();
    let value: BencodeDict = vdk_bencode::from_slice(data.as_mut_slice()).unwrap().try_into().unwrap();
    let _  = TorrentFile::try_from_bencode(value);
    
}
