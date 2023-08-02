mod cli;
mod config;
mod media;
mod media_handler;
mod repository;

use std::path::Path;

use clap::Parser;
use cli::DownloadArgs;
use cli::{Cli, Command};
use config::Config;
use log::debug;
use media_handler::{MediaHandler, YtDlp};
use repository::SQLiteRepository;

struct Api<T> {
    media_handler: T,
    repository: SQLiteRepository,
    config: Config,
}

impl<T: MediaHandler> Api<T> {
    fn new(
        media_handler: T,
        repository: SQLiteRepository,
        config: Config,
    ) -> Self {
        Api {
            media_handler,
            repository,
            config,
        }
    }

    fn download_media(&self, arguments: &DownloadArgs) -> anyhow::Result<()> {
        let directory = arguments.directory.as_deref().unwrap_or_default();

        let media_download_output = self.media_handler.download(
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
        let tags = arguments.tags.as_deref().unwrap_or_default();

        let filename =
            arguments.filename.as_ref().unwrap_or(&media_metadata.title);
        let directory = arguments.directory.as_deref().unwrap_or_default();

        self.repository.insert(
            &media_metadata.title,
            &filename,
            &directory,
            &arguments.url,
            &tags,
        );

        Ok(())
    }

    fn record_media(&self, arguments: &DownloadArgs) -> anyhow::Result<()> {
        let media_download_output = self
            .media_handler
            .get_media_metadata(arguments.url.as_ref())?;

        // This is exactly the same as what we have above
        let media_metadata =
            media_download_output
                .into_single_video()
                .ok_or(anyhow::Error::msg(
                "If download was successful, should have acess to single media",
            ))?;

        let tags = arguments.tags.as_deref().unwrap_or_default();

        let filename =
            arguments.filename.as_ref().unwrap_or(&media_metadata.title);
        let directory = arguments.directory.as_deref().unwrap_or_default();

        self.repository.insert(
            &media_metadata.title,
            &filename,
            &directory,
            &arguments.url,
            &tags,
        );
        // This is exactly the same as what we have above

        Ok(())
    }

    fn list_catalog(
        &self,
        directory: &Option<String>,
        tags: &Option<String>,
    ) -> anyhow::Result<()> {
        let catalog_items = self.repository.query(
            directory.as_deref().unwrap_or_default(),
            tags.as_deref().unwrap_or_default(),
        )?;
        if catalog_items.len() > 0 {
            for item in catalog_items {
                println!("{}", item);
            }
        } else {
            println!("No items to list");
        }

        Ok(())
    }

    // NOTE: Going with an iterative approach for now
    fn diff(&self) -> anyhow::Result<()> {
        let catalog_items = self.repository.query("", "")?;
        for media in catalog_items {
            let media_file_path = Path::new(&self.config.catalog_path)
                .join(&media.directory)
                .join(&media.filename);
            if !media_file_path.exists() {
                self.media_handler.download(
                    &media.url,
                    &self.config.catalog_path,
                    &media.directory,
                    Some(&media.filename),
                )?;
            }
        }

        Ok(())
    }
}

fn main() -> anyhow::Result<()> {
    // Init logger
    env_logger::init();

    // read config
    let config = config::Config::new();
    debug!("{:?}", config);

    // create repo
    let repository = repository::SQLiteRepository::new(&config);

    // create instance of API
    let api = Api::new(YtDlp, repository, config);

    let command = Cli::parse().command.ok_or_else(|| {
        log::error!("invalid command provided");
        anyhow::Error::msg("invalid command provided")
    })?;
    match &command {
        Command::Download(args) => api.download_media(args),
        Command::Record(args) => api.record_media(args),
        Command::List { directory, tags } => api.list_catalog(directory, tags),
        Command::Diff => api.diff(),
    }
}
