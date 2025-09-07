mod cli;
mod config;
mod media;
mod media_handler;
mod repository;

use clap::Parser;
use cli::{Cli, Command, DownloadArgs};
use config::Config;
use log::{debug, warn};
use media::Media;
use media_handler::{MediaHandler, YtDlp};

use crate::repository::Repository;

struct Api<T, R> {
    media_handler: T,
    repository: R,
    config: Config,
}

impl<T: MediaHandler, R: Repository> Api<T, R> {
    fn new(media_handler: T, repository: R, config: Config) -> Self {
        Api {
            media_handler,
            repository,
            config,
        }
    }

    fn download_media(&self, arguments: &DownloadArgs) -> anyhow::Result<()> {
        let directory = arguments.directory.as_deref().unwrap_or("");

        let media_download_output = self.media_handler.download(
            &arguments.url,
            &self.config.catalog_path,
            directory,
            arguments.filename.as_deref(),
            self.config.override_if_exists,
        )?;

        if !arguments.no_record {
            let media_metadata = media_download_output.into_single_video().ok_or_else(|| {
                anyhow::anyhow!("If download was successful, should have access to single media")
            })?;

            let media = self.create_media_from_metadata(&media_metadata, arguments)?;
            debug!("Recording media in catalog");
            self.repository.insert_into_media(&media)?;
        } else {
            debug!("Skipping recording media as --no-record was provided");
        }

        Ok(())
    }

    fn record_media(&self, args: &DownloadArgs) -> anyhow::Result<()> {
        let media_download_output = self.media_handler.get_media_metadata(&args.url)?;

        let media_metadata = media_download_output.into_single_video().ok_or_else(|| {
            anyhow::anyhow!("If metadata fetch was successful, should have access to single media")
        })?;

        let media = self.create_media_from_metadata(&media_metadata, args)?;
        self.repository.insert_into_media(&media)?;

        Ok(())
    }

    // Helper method to reduce duplication
    fn create_media_from_metadata(
        &self,
        metadata: &youtube_dl::SingleVideo,
        args: &DownloadArgs,
    ) -> anyhow::Result<Media> {
        let name = &metadata.title;
        let filename = args.filename.as_ref().unwrap_or(name);
        let directory = args.directory.as_ref().map_or("", |v| v);
        let tags = args.tags.as_deref().unwrap_or_default();

        Media::builder()
            .name(name)
            .filename(filename)
            .library(directory)
            .url(&args.url)
            .tags(tags)
            .build()
            .map_err(|e| anyhow::anyhow!(e)) // Convert to anyhow::Error
    }

    fn list_catalog(&self, args: cli::ListArgs) -> anyhow::Result<()> {
        let catalog_items = self.repository.query(
            args.directory.as_deref().unwrap_or(""),
            args.tags.as_deref().unwrap_or(""),
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

    fn diff(&self) -> anyhow::Result<()> {
        let catalog_items = self.repository.query("", "")?;
        for media in catalog_items {
            let media_file_path = self
                .config
                .catalog_path
                .join(&media.library)
                .join(&media.filename);
            if !media_file_path.exists() {
                self.media_handler.download(
                    &media.url,
                    &self.config.catalog_path,
                    &media.library,
                    Some(&media.filename),
                    self.config.override_if_exists,
                )?;
            }
        }

        Ok(())
    }
}

fn main() -> anyhow::Result<()> {
    // Init logger
    env_logger::init();

    // Read config
    let config =
        config::Config::new().or_else(|e| Err(anyhow::anyhow!("Error reading config: {}.", e)))?;
    debug!("{:?}", config);

    // Parse command once
    let command = Cli::parse()
        .command
        .ok_or_else(|| anyhow::anyhow!("command is required"))?;

    // Branch on repository type and create different Api instances
    if config.no_record {
        warn!("--no-record is set in config, catalog will not be created or updated.");
        let repository = repository::DummyRepository;
        let api = Api::new(YtDlp, repository, config);
        run_command(api, command)
    } else {
        let repository = repository::SQLiteRepository::new(&config)?;
        let api = Api::new(YtDlp, repository, config);
        run_command(api, command)
    }
}

fn run_command<T: MediaHandler, R: Repository>(
    api: Api<T, R>,
    command: Command,
) -> anyhow::Result<()> {
    match command {
        Command::Download(args) => api.download_media(&args),
        Command::Record(args) => api.record_media(&args),
        Command::List(args) => api.list_catalog(args),
        Command::Diff => api.diff(),
    }
}
