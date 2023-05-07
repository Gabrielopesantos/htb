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
        let yt_dl_output = yt_download(&arguments.url, arguments.filename.as_ref())?;
        let video_content = yt_dl_output.into_single_video().ok_or(anyhow::Error::msg(
            "If download was successful, should have acess to single video",
        ))?;

        let filename = arguments.filename.as_ref().unwrap_or(&video_content.title);
        let empty_tags = String::from("");
        let tags = arguments.tags.as_ref().unwrap_or(&empty_tags);

        self.repository
            .insert_media(&video_content.title, filename, &arguments.url, tags);

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

    let command = Cli::parse().command.ok_or({
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

fn yt_download(url: &str, filename: Option<&String>) -> Result<YoutubeDlOutput, youtube_dl::Error> {
    // --default-search option doesn't seem to be working properly, when used
    // `into_single_video` returns None. Going to be expecting full URLs.

    let media_name = String::from("%(title)s");
    let filename = filename.unwrap_or(&media_name);
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
        .extra_arg(format!("/tmp/downloads/{}.%(ext)s", filename))
        .run()
}
