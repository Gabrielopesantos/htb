mod cli;
mod config;
mod repository;

use clap::Parser;
use cli::Download;
use cli::{Cli, Command};
use config::Config;
use repository::SQLiteRepository;
use youtube_dl::YoutubeDlOutput;

struct Api {
    repository: SQLiteRepository,
    config: Config,
}

impl Api {
    fn new(repository: SQLiteRepository, config: Config) -> Api {
        Api { repository, config }
    }

    fn download_media(&self, arguments: &Download) -> anyhow::Result<()> {
        let yt_dl_output = yt_download(&arguments.url)?;
        let video_content = yt_dl_output.into_single_video().ok_or(anyhow::Error::msg(
            "If download was successful, should have acess to single video",
        ))?;

        println!("{:?}", video_content.title);

        self.repository
            .insert_media(&arguments.filename, &arguments.url);

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

fn yt_download(url: &str) -> Result<YoutubeDlOutput, youtube_dl::Error> {
    // --default-search option doesn't seem to be working properly, when used
    // `into_single_video` returns None. Going to be expecting full URLs.
    youtube_dl::YoutubeDl::new(url)
        .youtube_dl_path("yt-dlp")
        .download(true)
        .extract_audio(true)
        .extra_arg("--no-playlist")
        .extra_arg("--no-continue")
        // .extra_arg("--default-search")
        // .extra_arg("auto") // ytsearch
        .extra_arg("-f bestaudio")
        .extra_arg("--downloader")
        .extra_arg("ffmpeg")
        .extra_arg("--audio-format")
        .extra_arg("mp3")
        .extra_arg("--no-keep-video")
        .extra_arg("-o")
        .extra_arg("/tmp/downloads/%(title)s.%(ext)s")
        .run()
}
