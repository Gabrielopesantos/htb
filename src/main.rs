mod cli;
mod config;
mod media_downloader;
mod repository;

use clap::Parser;
use cli::Download;
use cli::{Cli, Command};
use config::Config;
use media_downloader::{MediaDownloader, YtDlp};
use repository::SQLiteRepository;

struct Api<T> {
    media_downl: T,
    repository: SQLiteRepository,
    config: Config,
}

impl<T: MediaDownloader> Api<T> {
    fn new(
        media_downl: T,
        repository: SQLiteRepository,
        config: Config,
    ) -> Self {
        Api {
            media_downl,
            repository,
            config,
        }
    }

    fn download_media(&self, arguments: &Download) -> anyhow::Result<()> {
        let directory = arguments
            .directory
            .as_ref()
            .unwrap_or(&"".to_string())
            .to_owned();

        let media_download_output = self.media_downl.download(
            arguments.url.as_ref(),
            &self.config.catalog_path,
            &directory,
            arguments.filename.as_deref(),
        )?;

        let media_metadata =
            media_download_output
                .into_single_video()
                .ok_or(anyhow::Error::msg(
                "If download was successful, should have acess to single media",
            ))?;
        let tags = arguments
            .tags
            .as_ref()
            .unwrap_or(&"".to_string())
            .to_owned(); // FIXME: Is this the best way of having a default value for tags?

        let filename =
            arguments.filename.as_ref().unwrap_or(&media_metadata.title);
        let directory = arguments
            .directory
            .as_ref()
            .unwrap_or(&"".to_string())
            .to_owned();

        self.repository.insert_media(
            &media_metadata.title,
            &filename,
            &directory,
            &arguments.url,
            &tags,
        );

        Ok(())
    }

    fn record_media(&self, arguments: &Download) -> anyhow::Result<()> {
        let media_download_output = self
            .media_downl
            .get_media_metadata(arguments.url.as_ref())?;

        // This is exactly the same as what we have above
        let media_metadata =
            media_download_output
                .into_single_video()
                .ok_or(anyhow::Error::msg(
                "If download was successful, should have acess to single media",
            ))?;

        let tags = arguments
            .tags
            .as_ref()
            .unwrap_or(&"".to_string())
            .to_owned(); // FIXME: Is this the best way of having a default value for tags?

        let filename =
            arguments.filename.as_ref().unwrap_or(&media_metadata.title);
        let directory = arguments
            .directory
            .as_ref()
            .unwrap_or(&"".to_string())
            .to_owned();

        self.repository.insert_media(
            &media_metadata.title,
            &filename,
            &directory,
            &arguments.url,
            &tags,
        );
        // This is exactly the same as what we have above

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
    let api = Api::new(YtDlp, repository, config);

    // ??
    let command = Cli::parse().command.ok_or_else(|| {
        log::error!("invalid command provided");
        anyhow::Error::msg("invalid command provided")
    })?;
    match &command {
        Command::Download(args) => api.download_media(args),
        Command::Record(args) => api.record_media(args),
        Command::List(..) => {
            log::info!("Calling list",);
            Ok(())
        }
    }
}
// 1. no method named `download` found for type parameter `T` in the current scope
//    found the following associated functions; to be used as methods, functions must have a `self` parameter
