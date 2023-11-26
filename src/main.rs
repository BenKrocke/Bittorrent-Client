#[macro_use]
extern crate serde_derive;

use reqwest::blocking::Client;
use reqwest::Url;
use serde_bencode::de;
use serde_bencode::ser;
use serde_bencode::value::Value;
use serde_bytes::ByteBuf;
use sha1::{Digest, Sha1};
use std::fs::File as FsFile;
use std::io::Read;
use urlencoding::encode;

#[derive(Debug, Deserialize, Serialize)]
struct Node(String, i64);

#[derive(Debug, Deserialize, Serialize)]
struct File {
    path: Vec<String>,
    length: i64,
    #[serde(default)]
    md5sum: Option<String>,
}

#[allow(dead_code)]
#[derive(Debug, Deserialize, Serialize)]
struct Info {
    pub name: String,
    pub pieces: ByteBuf,
    #[serde(rename = "piece length")]
    pub piece_length: i64,
    #[serde(default)]
    pub md5sum: Option<String>,
    #[serde(default)]
    pub length: Option<i64>,
    #[serde(default)]
    pub files: Option<Vec<File>>,
    #[serde(default)]
    pub private: Option<u8>,
    #[serde(default)]
    pub path: Option<Vec<String>>,
    #[serde(default)]
    #[serde(rename = "root hash")]
    pub root_hash: Option<String>,
}

#[derive(Debug, Deserialize, Serialize)]
struct Torrent {
    info: Info,
    #[serde(skip)]
    info_hash: String,
    #[serde(default)]
    announce: Option<String>,
    #[serde(default)]
    nodes: Option<Vec<Node>>,
    #[serde(default)]
    encoding: Option<String>,
    #[serde(default)]
    httpseeds: Option<Vec<String>>,
    #[serde(default)]
    #[serde(rename = "announce-list")]
    announce_list: Option<Vec<Vec<String>>>,
    #[serde(default)]
    #[serde(rename = "creation date")]
    creation_date: Option<i64>,
    #[serde(rename = "comment")]
    comment: Option<String>,
    #[serde(default)]
    #[serde(rename = "created by")]
    created_by: Option<String>,
}

fn render_torrent(torrent: &Torrent) {
    println!("name:\t\t{}", torrent.info.name);
    println!("announce:\t{:?}", torrent.announce);
    println!("nodes:\t\t{:?}", torrent.nodes);
    if let Some(al) = &torrent.announce_list {
        for a in al {
            println!("announce list:\t{}", a[0]);
        }
    }
    println!("httpseeds:\t{:?}", torrent.httpseeds);
    println!("creation date:\t{:?}", torrent.creation_date);
    println!("comment:\t{:?}", torrent.comment);
    println!("created by:\t{:?}", torrent.created_by);
    println!("encoding:\t{:?}", torrent.encoding);
    println!("piece length:\t{:?}", torrent.info.piece_length);
    println!("private:\t{:?}", torrent.info.private);
    println!("root hash:\t{:?}", torrent.info.root_hash);
    println!("md5sum:\t\t{:?}", torrent.info.md5sum);
    println!("path:\t\t{:?}", torrent.info.path);
    if let Some(files) = &torrent.info.files {
        for f in files {
            println!("file path:\t{:?}", f.path);
            println!("file length:\t{}", f.length);
            println!("file md5sum:\t{:?}", f.md5sum);
        }
    }
}

fn escape_info_hash(info_hash: &str) -> String {
    let zipped = info_hash.chars().zip(info_hash.chars().skip(1));

    let mut escaped_info_hash = String::new();

    for (hex1, hex2) in zipped.step_by(2) {
        let new_str = format!("%{hex1}{hex2}");
        escaped_info_hash.push_str(new_str.as_str());
    }

    escaped_info_hash
}

fn initial_request(torrent: &Torrent) -> Result<(), Box<dyn std::error::Error>> {
    let announce = torrent.announce.as_ref().unwrap();

    let info_hash = escape_info_hash(torrent.info_hash.as_str());
    let params = [
        ("peer_id", encode("makethisuniqueplease").to_string()), // TODO: Better id
        ("port", "6881".to_string()),
        ("uploaded", "0".to_string()),
        ("downloaded", "0".to_string()),
        ("left", torrent.info.piece_length.to_string()),
        ("compact", "1".to_string()),
        ("event", "started".to_string()),
    ];

    let client = Client::new();
    let mut url = Url::parse_with_params(announce, &params)?;
    url.set_query(Some(format!("info_hash={info_hash}").as_str()));
    let response = client.get(url).send();

    println!("{:#?}", response);
    Ok(())
}

fn get_info_hash(buffer: &Vec<u8>) -> String {
    #[derive(Deserialize)]
    struct MetaInfo {
        info: Value,
    }

    let meta_info = de::from_bytes::<MetaInfo>(&buffer).expect("Torrent file could not be parsed.");
    let info = ser::to_bytes(&meta_info.info).expect("Info could not be serialized.");
    let mut hasher = Sha1::new();
    hasher.update(&info);
    let info_hash = hasher.finalize();
    hex::encode(&info_hash).to_string()
}

fn load_file_into_buffer() -> Vec<u8> {
    let mut file = FsFile::open("/home/benkrocke/code/rust/bittorrent/tor.torrent")
        .expect("Torrent file could not be found.");
    let mut buffer = Vec::new();
    file.read_to_end(&mut buffer).unwrap();
    buffer
}

fn parse_buffer(buffer: Vec<u8>) -> Torrent {
    let mut torrent =
        de::from_bytes::<Torrent>(&buffer).expect("Torrent file could not be parsed.");
    let info_hash = get_info_hash(&buffer);
    torrent.info_hash = info_hash;
    torrent
}

fn main() {
    let buffer = load_file_into_buffer();
    let torrent = parse_buffer(buffer);
    
    initial_request(&torrent).unwrap();
}
