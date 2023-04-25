mod cli;
mod config;
mod repository;

use clap::Parser;
use cli::Download;
use cli::{Cli, Command};
use config::Config;
use repository::SQLiteRepository;

struct Api {
    repository: SQLiteRepository,
    config: Config,
}

impl Api {
    fn new(repository: SQLiteRepository, config: Config) -> Api {
        Api { repository, config }
    }

    fn download_media(&self, arguments: &Download) -> anyhow::Result<()> {
        let _yt_dl_output = yt_download(&arguments.link);
        self.repository
            .insert_media(&arguments.filename, &arguments.link);

        Ok(())
    }
}

fn main() -> anyhow::Result<()> {
    let config = config::Config::new();
    println!("Config: {:?}", config);

    let repository = repository::SQLiteRepository::new(&config);
    let api = Api::new(repository, config);

    let command = Cli::parse()
        .command
        .ok_or(anyhow::Error::msg("unexpected command used"))?;
    match &command {
        Command::Download(args) => {
            println!("Calling download");
            api.download_media(args)
        }
        Command::Record(..) => {
            println!("Calling decord",);
            Ok(())
        }
        Command::List(..) => {
            println!("Calling list",);
            Ok(())
        }
    }
}

fn yt_download(watch: &str) -> youtube_dl::YoutubeDlOutput {
    // let url = "watch?v=";
    // TODO: .extra_arg("-f bestaudio")
    youtube_dl::YoutubeDl::new(watch)
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
        .expect("Failed to download video")
}
