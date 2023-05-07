mod cli;
mod config;
mod repository;

use clap::Parser;
use cli::Download;
use cli::{Cli, Command};
use config::Config;
use repository::SQLiteRepository;
use std::path::PathBuf;
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
        let yt_dl_output = yt_download(
            &arguments.url,
            &self.config.catalog_path,
            arguments.directory.as_ref(),
            arguments.filename.as_ref(),
        )?;
        let media_content = yt_dl_output.into_single_video().ok_or(anyhow::Error::msg(
            "If download was successful, should have acess to single media",
        ))?;

        let filename = arguments.filename.as_ref().unwrap_or(&media_content.title);
        let directory = arguments.directory.as_ref().unwrap_or(&"".to_string()).to_owned();
        let tags = arguments.tags.as_ref().unwrap_or(&"".to_string()).to_owned();

        self.repository
            .insert_media(&media_content.title, filename, &directory, &arguments.url, &tags);

        Ok(())
    }
}

fn main() -> anyhow::Result<()> {
    // initialize logger
    env_logger::init();

    // read config
    let config = config::Config::new();
    log::debug!("Config read: {:?}", config);

    // create repo
    let repository = repository::SQLiteRepository::new(&config);

    // create instance of API
    let api = Api::new(repository, config);

    // ??
    let command = Cli::parse().command.ok_or_else(|| {
        log::error!("invalid command provided");
        anyhow::Error::msg("invalid command provided")
    })?;
    match &command {
        Command::Download(args) => api.download_media(args),
        Command::Record(..) => {
            log::info!("Calling decord",);
            Ok(())
        }
        Command::List(..) => {
            log::info!("Calling list",);
            Ok(())
        }
    }
}

fn yt_download(
    url: &str,
    base_path: &PathBuf,
    directory: Option<&String>,
    filename: Option<&String>,
) -> Result<YoutubeDlOutput, youtube_dl::Error> {
    // --default-search option doesn't seem to be working properly, when used
    // `into_single_video` returns None. Going to be expecting full URLs.

    let filename = filename.unwrap_or(&String::from("%(title)s")).to_owned();
    let directory = directory.unwrap_or(&String::from("")).to_owned();
    let output_path = base_path
        .clone()
        .join(directory)
        .join(filename)
        .to_str()
        .expect("Path is always valid")
        .to_owned();

    log::info!("Output path: {}", output_path);

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
        .extra_arg(output_path)
        .run()
}
