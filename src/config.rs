extern crate serde_json;

use std::string::String;
use std::fs::File;
use std::io::prelude::*;
use std::vec::Vec;

#[derive(Serialize, Deserialize)]
pub struct SyncBlock {
    pub local : String,
    pub remote: String,
    pub host  : String,
    pub key   : String,
}

#[derive(Serialize, Deserialize)]
pub struct Config {
    pub synchronize : Vec<SyncBlock>
}

pub fn read_cfg(path : String) -> Config {
    let mut contents = String::new();
    let mut file = File::open(path).expect("File not found");
    println!("{}", contents);
    file.read_to_string(&mut contents).unwrap();
    let cfg : Config = serde_json::from_str(&contents).unwrap();
    return cfg;
}
