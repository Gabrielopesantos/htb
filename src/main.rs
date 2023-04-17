use rusqlite::Connection;
use serde::Deserialize;
#[allow(deprecated)]
use std::{env::home_dir, fs::File};

#[derive(Debug, Deserialize)]
#[serde(rename_all(deserialize = "camelCase"))]
struct Config {
    database_file_path: String,
    catalog_path: String,
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

struct SQLiteRepository {
    conn: rusqlite::Connection,
}

// Implement a Catalog interface
impl SQLiteRepository {
    pub fn new(config: &Config) -> SQLiteRepository {
        // NOTE: File in given path might not exist, create it before
        let conn =
            Connection::open(&config.database_file_path).expect("Failed to establish connection");

        return SQLiteRepository { conn };
    }
}

fn _yt_download() {
    let url = "watch?v=";
    // TODO: .extra_arg("-f bestaudio")
    youtube_dl::YoutubeDl::new(url)
        .youtube_dl_path("yt-dlp")
        // .extract_audio(true)
        .download(true)
        // Don't allow downloading playlists
        .extra_arg("--no-playlist")
        // Don't continue a paused download, always restart
        .extra_arg("--no-continue")
        .extra_arg("--default-search")
        .extra_arg("ytsearch")
        .extra_arg("--downloader")
        .extra_arg("ffmpeg")
        .extra_arg("--extract-audio")
        .extra_arg("--audio-format")
        .extra_arg("mp3")
        .extra_arg("-o")
        .extra_arg("/tmp/downloads/audio-rs")
        .run()
        .expect("Failed to download video");

    // info!("successfully downloaded yt clip");
}

fn main() {
    let config = Config::new();
    println!("Config: {:?}", config);

    let _catalog = SQLiteRepository::new(&config);
}
