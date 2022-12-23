use std::fs;

use serde_derive::Deserialize;

#[allow(unused)]
#[derive(Deserialize, Debug)]
struct Listener {
    address: String,
    port: i16,
}

#[allow(unused)]
#[derive(Deserialize, Debug)]
struct Server {
    address: String,
    port: i16,
}

#[allow(unused)]
#[derive(Deserialize, Debug)]
struct Config {
    listener: Listener,
    servers: Vec<Server>
}

fn main() {
    let contents = fs::read_to_string("servers.toml").unwrap();

    let config: Config = match toml::from_str(&contents) {
        Ok(config) => config,
        Err(_) => panic!("Failed to parse the config file.")
    };

    dbg!(config);
}
