use serde::Deserialize;
#[allow(deprecated)]
use std::{env::home_dir, fs::File};

#[derive(Debug, Deserialize)]
#[serde(rename_all(deserialize = "camelCase"))]
pub struct Config {
    pub database_file_path: String,
    pub catalog_path: String,
}

impl Config {
    pub fn new() -> Config {
        // NOTE: $HOME/ and ~/ do not work
        // #[allow(deprecated)]
        // let default_config_path = match home_dir() {
        //     Some(path) => path.display().to_string() + "/.config/htb/config.json",
        //     None => panic!("couldn't find home directory"),
        // };
        let default_config_path = "./config-example.json";

        println!("{}", default_config_path);

        let file = File::open(default_config_path).expect("could not find the configuration file");
        let config = serde_json::from_reader(file).expect("JSON isn't well-formatted");

        return config;
    }
}