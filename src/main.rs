mod cli;
mod config;
mod error;
mod list;
mod media;
mod media_handler;
mod progress;
mod repository;
mod tagger;
mod title;

use clap::Parser;
use cli::{Cli, Command, DownloadArgs, TagOverrides};
use config::Config;
use error::{HtbError, Result};
use log::{debug, info, warn};
use media::{Id3Tags, Media};
use media_handler::{DownloadOutcome, MediaHandler, YtDlp};

use std::io::{self, BufWriter, Write};
use std::path::{Path, PathBuf};

use crate::repository::Repository;

/// Drops promotional noise from a freshly downloaded file's name. The
/// `[<video id>]` the output template appends is preserved, so the file stays
/// unambiguous without the marketing text. Returns the path to use from here on.
fn rename_cleaned(path: &Path) -> Result<PathBuf> {
    let Some(stem) = path.file_stem().and_then(|s| s.to_str()) else {
        return Ok(path.to_path_buf());
    };

    let cleaned = title::clean(stem);
    if cleaned == stem || cleaned.is_empty() {
        return Ok(path.to_path_buf());
    }

    // Built as a string rather than via `set_extension`, which would corrupt a
    // name that itself contains a dot ("Song feat. Someone").
    let filename = match path.extension().and_then(|e| e.to_str()) {
        Some(extension) => format!("{cleaned}.{extension}"),
        None => cleaned,
    };
    let target = path.with_file_name(filename);

    if target.exists() {
        warn!(
            "Cleaned name {} already exists, keeping {}",
            target.display(),
            path.display()
        );
        return Ok(path.to_path_buf());
    }

    debug!("Renaming to cleaned name: {}", target.display());
    std::fs::rename(path, &target)?;

    Ok(target)
}

/// Fills in a title override from the cleaned name when the user did not give
/// one, so the ID3 title matches the catalog and survives a later `diff`.
fn effective_overrides(
    provided: &TagOverrides,
    raw_title: &str,
    cleaned_title: &str,
) -> TagOverrides {
    let mut overrides = provided.clone();

    if overrides.title.is_none() && cleaned_title != raw_title {
        overrides.title = Some(cleaned_title.to_string());
    }

    overrides
}

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

    fn download_media(&self, arguments: &DownloadArgs) -> Result<()> {
        let directory = arguments.directory.as_deref().unwrap_or("");

        info!("Starting download from: {}", arguments.url);

        let outcome = self.media_handler.download(
            &arguments.url,
            &self.config.catalog_path,
            directory,
            arguments.filename.as_deref(),
            self.config.override_if_exists,
        )?;

        let (output, resolved_path) = match outcome {
            DownloadOutcome::Downloaded { output, path } => (output, path),
            DownloadOutcome::AlreadyDownloaded => {
                if arguments.overrides.is_empty() {
                    info!("Already downloaded, skipping: {}", arguments.url);
                } else {
                    // The file is already on disk, so nothing was tagged. Saying
                    // so beats exiting 0 as if the flags had been applied.
                    warn!(
                        "Already downloaded, so tag options were ignored. \
                         Use `htb tag -u {}` to change its tags.",
                        arguments.url
                    );
                }
                return Ok(());
            }
        };

        // An explicit -f is the user's own name, so leave it alone.
        let resolved_path = if arguments.filename.is_none() {
            rename_cleaned(&resolved_path)?
        } else {
            resolved_path
        };

        let media_metadata = output.into_single_video().ok_or_else(|| {
            HtbError::Other(
                "If download was successful, should have access to a single audio track"
                    .to_string(),
            )
        })?;

        let name = title::clean(&media_metadata.title);
        let title_override =
            effective_overrides(&arguments.overrides, &media_metadata.title, &name);

        // yt-dlp already embedded its own baseline tags (title, uploader as
        // artist, etc.) during the download; read them back so the catalog
        // ends up with the file's real tags, not just the CLI-supplied delta.
        let base_tags = tagger::read_tags(&resolved_path)?;
        let tags = base_tags.overlay(&title_override);

        // Tag before recording, so a tagging failure never leaves a catalog row
        // claiming tags that were never written to the file. Only rewrite the
        // file when there is an actual delta to apply.
        if !title_override.is_empty() {
            tagger::write_tags(&resolved_path, &tags)?;
        }

        if !arguments.no_record {
            let filename = resolved_path
                .file_name()
                .and_then(|f| f.to_str())
                .ok_or_else(|| {
                    HtbError::Other(format!(
                        "Resolved path has no valid filename: {}",
                        resolved_path.display()
                    ))
                })?;

            let media = self.create_media(&name, filename, arguments, &tags)?;
            debug!("Recording in catalog");
            self.repository.insert_into_media(&media)?;
            info!("Download completed and recorded: {}", media.name);
        } else {
            debug!("Skipping catalog recording as --no-record was provided");
            info!("Download completed (not recorded)");
        }

        Ok(())
    }

    fn record_media(&self, args: &DownloadArgs) -> Result<()> {
        info!("Fetching metadata for: {}", args.url);
        let media_download_output = self.media_handler.get_media_metadata(&args.url)?;

        let media_metadata = media_download_output.into_single_video().ok_or_else(|| {
            HtbError::Other(
                "If metadata fetch was successful, should have access to a single audio track"
                    .to_string(),
            )
        })?;

        let name = title::clean(&media_metadata.title);
        let title_override = effective_overrides(&args.overrides, &media_metadata.title, &name);
        // No file exists yet, so there's nothing to read baseline tags from -
        // this persists intent only. A later `diff` writes it to the file.
        let tags = Id3Tags::default().overlay(&title_override);

        let filename = args.filename.as_deref().unwrap_or(&name);
        let media = self.create_media(&name, filename, args, &tags)?;
        self.repository.insert_into_media(&media)?;
        info!("Recorded in catalog: {}", media.name);

        Ok(())
    }

    // Helper method to reduce duplication
    fn create_media(
        &self,
        name: &str,
        filename: &str,
        args: &DownloadArgs,
        tags: &Id3Tags,
    ) -> Result<Media> {
        let directory = args.directory.as_ref().map_or("", |v| v);
        let catalog_tags = args.tags.as_deref().unwrap_or_default();

        Media::builder()
            .name(name)
            .filename(filename)
            .library(directory)
            .url(&args.url)
            .tags(catalog_tags)
            .id3(tags.clone())
            .build()
    }

    fn list_catalog(&self, args: &cli::ListArgs) -> Result<()> {
        info!(
            "Querying catalog with filters - directory: {:?}, tags: {:?}",
            args.directory, args.tags
        );
        let mut catalog_items = self.repository.query(
            args.directory.as_deref().unwrap_or(""),
            args.tags.as_deref().unwrap_or(""),
        )?;

        if catalog_items.is_empty() {
            // On stderr, so a piped `htb list` yields rows or nothing at all.
            eprintln!("No items to list");
            return Ok(());
        }

        info!("Found {} items", catalog_items.len());
        list::sort(&mut catalog_items, args.sort, args.reverse);

        let stdout = io::stdout();
        let mut out = BufWriter::new(stdout.lock());
        let written = list::render(&catalog_items, args.format, args.long, &mut out)
            .and_then(|()| out.flush());

        // `htb list | head` closes the pipe early; that is a normal way to stop
        // reading, not a failure.
        match written {
            Err(error) if error.kind() == io::ErrorKind::BrokenPipe => Ok(()),
            other => Ok(other?),
        }
    }

    fn diff(&self) -> Result<()> {
        info!("Running diff to find missing files");
        let catalog_items = self.repository.query("", "")?;
        let mut missing_count = 0;

        for media in catalog_items {
            let media_file_path = self
                .config
                .catalog_path
                .join(&media.library)
                .join(&media.filename);
            if !media_file_path.exists() {
                info!("Missing file detected, downloading: {}", media.name);
                let outcome = self.media_handler.download(
                    &media.url,
                    &self.config.catalog_path,
                    &media.library,
                    Some(&media.filename),
                    self.config.override_if_exists,
                )?;

                match outcome {
                    DownloadOutcome::Downloaded { path, .. } => {
                        // A re-download only produces yt-dlp's baseline tags,
                        // so the catalog's known tags are written back onto it.
                        // The catalog row is already canonical - no DB update needed.
                        tagger::write_tags(&path, &media.id3)?;
                        missing_count += 1;
                    }
                    DownloadOutcome::AlreadyDownloaded => warn!(
                        "Still missing after re-download attempt (already in download archive): {}",
                        media.name
                    ),
                }
            }
        }

        info!("Diff completed: {} files downloaded", missing_count);
        Ok(())
    }

    fn tag_media(&self, args: &cli::TagArgs) -> Result<()> {
        if args.overrides.is_empty() {
            return Err(HtbError::Other(
                "No tag options given; pass at least one of --title/--artist/--album/--track/--year/--genre"
                    .to_string(),
            ));
        }

        let entries = self.repository.find_by_url(&args.url)?;
        let Some(first) = entries.first() else {
            return Err(HtbError::Other(format!(
                "No catalog entry found for {}",
                args.url
            )));
        };
        if entries.len() > 1 {
            warn!(
                "{} catalog entries share this URL, updating all of them",
                entries.len()
            );
        }

        let merged = first.id3.overlay(&args.overrides);

        // Write the files before the catalog, so a tagging failure never leaves
        // rows claiming tags that are not on disk.
        for media in &entries {
            let media_file_path = self
                .config
                .catalog_path
                .join(&media.library)
                .join(&media.filename);

            if media_file_path.exists() {
                tagger::write_tags(&media_file_path, &merged)?;
            } else {
                warn!(
                    "File not found, updating catalog only (run `htb diff` to restore it): {}",
                    media_file_path.display()
                );
            }
        }

        let updated = self.repository.update_tags(&args.url, &merged)?;
        info!("Tags updated for {} catalog entrie(s)", updated);

        Ok(())
    }
}

fn main() -> Result<()> {
    // Init logger
    env_logger::init();

    // Read config
    let config = config::Config::new()
        .map_err(|e| HtbError::Config(format!("Error reading config: {}", e)))?;
    debug!("{:?}", config);

    // Parse command once
    let command = Cli::parse()
        .command
        .ok_or_else(|| HtbError::Other("command is required".to_string()))?;

    // Branch on repository type and create different Api instances
    if config.no_record {
        warn!("--no-record is set in config, catalog will not be created or updated.");
        let repository = repository::DummyRepository;
        let api = Api::new(YtDlp, repository, config);
        run_command(api, command)
    } else {
        let repository = repository::SQLiteRepository::new(&config).map_err(|e| {
            HtbError::Other(format!("Could not find or create catalog database: {}", e))
        })?;
        let api = Api::new(YtDlp, repository, config);
        run_command(api, command)
    }
}

fn run_command<T: MediaHandler, R: Repository>(api: Api<T, R>, command: Command) -> Result<()> {
    match command {
        Command::Download(args) => api.download_media(&args),
        Command::Record(args) => api.record_media(&args),
        Command::List(args) => api.list_catalog(&args),
        Command::Diff => api.diff(),
        Command::Tag(args) => api.tag_media(&args),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Creates `name` in a per-test scratch directory and returns its path.
    fn touch(dir: &str, name: &str) -> PathBuf {
        let dir = std::env::temp_dir().join(format!("htb-test-{}-{}", std::process::id(), dir));
        std::fs::create_dir_all(&dir).unwrap();

        let path = dir.join(name);
        std::fs::write(&path, b"").unwrap();
        path
    }

    #[test]
    fn rename_cleaned_strips_noise_and_keeps_the_id() {
        let path = touch("rename", "Song (Official Video) [aBcD1234xyz].mp3");

        let renamed = rename_cleaned(&path).unwrap();

        assert_eq!(renamed.file_name().unwrap(), "Song [aBcD1234xyz].mp3");
        assert!(renamed.exists());
        assert!(!path.exists());
    }

    #[test]
    fn rename_cleaned_preserves_a_dot_in_the_name() {
        let path = touch("dot", "Song feat. Someone (Official Video).mp3");

        let renamed = rename_cleaned(&path).unwrap();

        assert_eq!(renamed.file_name().unwrap(), "Song feat. Someone.mp3");
    }

    #[test]
    fn rename_cleaned_is_a_noop_for_clean_names() {
        let path = touch("noop", "Song [aBcD1234xyz].mp3");

        assert_eq!(rename_cleaned(&path).unwrap(), path);
        assert!(path.exists());
    }

    #[test]
    fn rename_cleaned_keeps_the_original_on_collision() {
        let path = touch("collision", "Song (Official Video).mp3");
        touch("collision", "Song.mp3");

        assert_eq!(rename_cleaned(&path).unwrap(), path);
        assert!(path.exists());
    }

    #[test]
    fn effective_overrides_fills_in_the_cleaned_title() {
        let overrides =
            effective_overrides(&TagOverrides::default(), "Song (Official Video)", "Song");

        assert_eq!(overrides.title.as_deref(), Some("Song"));
    }

    #[test]
    fn effective_overrides_respects_an_explicit_title() {
        let provided = TagOverrides {
            title: Some("Mine".into()),
            ..Default::default()
        };

        let overrides = effective_overrides(&provided, "Song (Official Video)", "Song");

        assert_eq!(overrides.title.as_deref(), Some("Mine"));
    }

    #[test]
    fn effective_overrides_stays_empty_when_nothing_was_cleaned() {
        let overrides = effective_overrides(&TagOverrides::default(), "Song", "Song");

        assert!(overrides.is_empty());
    }
}
