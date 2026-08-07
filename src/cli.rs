use crate::list::{OutputFormat, SortKey};
use clap::{Args, Parser, Subcommand};
use serde::Serialize;

#[derive(Parser)]
#[command(author, version)]
#[command(about = "Download and keep track of audio content")]
#[command(arg_required_else_help = true)]
pub struct Cli {
    #[command(subcommand)]
    pub command: Option<Command>,
}

#[derive(Subcommand)]
pub enum Command {
    #[command(about = "Download and record audio content")]
    Download(DownloadArgs),
    #[command(about = "Record audio content")]
    Record(DownloadArgs),
    #[command(about = "List all audio in a catalog")]
    List(ListArgs),
    #[command(about = "Download recorded audio that is not persisted locally")]
    Diff,
    #[command(about = "Update the ID3 tags of already-downloaded audio")]
    Tag(TagArgs),
}

#[derive(Args)]
pub struct TagArgs {
    #[arg(short = 'u', long = "url", help = "URL of the catalog entry to update")]
    pub url: String,

    #[command(flatten)]
    pub overrides: TagOverrides,
}

#[derive(Args)]
pub struct DownloadArgs {
    #[arg(short = 'u', long = "url")]
    pub url: String,

    #[arg(
        short = 'd',
        long = "directory",
        help = "Directory to save the audio, if not provided it will be saved in the root catalog (default)"
    )]
    pub directory: Option<String>,

    #[arg(
        short = 'f',
        long = "filename",
        help = "Filename to save the audio, if not provided it will be from the audio title (sanitized)"
    )]
    pub filename: Option<String>,

    #[arg(
        short = 't',
        long = "tags",
        help = "Comma separated key values used as catalog labels. Unrelated to the ID3 tags written to the file (see --genre). If `--no-record` is provided, tags will not be recorded."
    )]
    pub tags: Option<String>,

    #[arg(
        long = "no-record",
        help = "If provided, the audio will not be recorded in the catalog"
    )]
    pub no_record: bool,

    #[command(flatten)]
    pub overrides: TagOverrides,
}

/// ID3 tag values that take precedence over whatever yt-dlp derived from the
/// source. For a plain YouTube URL yt-dlp falls back to the uploader for
/// `artist` and keeps the full title, so these are usually needed.
///
/// `record` persists these but writes no file, since there is nothing on disk
/// yet; `diff` applies them when it downloads the file.
#[derive(Args, Debug, Default, Clone, PartialEq, Serialize)]
pub struct TagOverrides {
    #[arg(long = "title", help = "ID3 title (TIT2)")]
    pub title: Option<String>,

    #[arg(long = "artist", help = "ID3 artist (TPE1)")]
    pub artist: Option<String>,

    #[arg(long = "album", help = "ID3 album (TALB)")]
    pub album: Option<String>,

    #[arg(long = "track", help = "ID3 track number (TRCK)")]
    pub track: Option<u32>,

    #[arg(long = "year", help = "ID3 recording year (TDRC)")]
    pub year: Option<u16>,

    #[arg(
        long = "genre",
        help = "ID3 genre (TCON). Not to be confused with -t/--tags, which are catalog labels."
    )]
    pub genre: Option<String>,
}

impl TagOverrides {
    /// Whether there is nothing to write. Gates every file rewrite, so an
    /// inversion here means either every download rewrites the file or none does.
    pub fn is_empty(&self) -> bool {
        self.title.is_none()
            && self.artist.is_none()
            && self.album.is_none()
            && self.track.is_none()
            && self.year.is_none()
            && self.genre.is_none()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn is_empty_when_default() {
        assert!(TagOverrides::default().is_empty());
    }

    #[test]
    fn not_empty_when_any_field_is_set() {
        let set_each: [TagOverrides; 6] = [
            TagOverrides {
                title: Some("t".into()),
                ..Default::default()
            },
            TagOverrides {
                artist: Some("a".into()),
                ..Default::default()
            },
            TagOverrides {
                album: Some("a".into()),
                ..Default::default()
            },
            TagOverrides {
                track: Some(1),
                ..Default::default()
            },
            TagOverrides {
                year: Some(2000),
                ..Default::default()
            },
            TagOverrides {
                genre: Some("g".into()),
                ..Default::default()
            },
        ];

        for overrides in set_each {
            assert!(!overrides.is_empty(), "{:?} should not be empty", overrides);
        }
    }
}

#[derive(Args)]
pub struct ListArgs {
    #[arg(
        short = 'd',
        long = "directory",
        help = "Only list entries in this directory"
    )]
    pub directory: Option<String>,

    #[arg(
        short = 't',
        long = "tags",
        help = "Comma separated catalog labels to filter by. Unrelated to the ID3 genre."
    )]
    pub tags: Option<String>,

    #[arg(
        short = 'l',
        long = "long",
        help = "Show every column, including track, year, genre, filename, URL and added date"
    )]
    pub long: bool,

    #[arg(
        long = "sort",
        value_enum,
        default_value_t = SortKey::Library,
        help = "Column to sort by"
    )]
    pub sort: SortKey,

    #[arg(short = 'r', long = "reverse", help = "Reverse the sort order")]
    pub reverse: bool,

    // Long-only: `-f` is `--filename` on `download`/`record`, so reusing it here
    // would be a confusing.
    #[arg(
        long = "format",
        value_enum,
        default_value_t = OutputFormat::Table,
        help = "Output format"
    )]
    pub format: OutputFormat,
}
