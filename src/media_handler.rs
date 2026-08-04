use log::{debug, warn};
use std::path::{Path, PathBuf};

use youtube_dl::{YoutubeDl, YoutubeDlOutput};

use crate::error::HtbError;

const DOWNLOAD_ARCHIVE: &str = ".htb_downloaded.txt";

/// Outcome of a download attempt.
pub enum DownloadOutcome {
    /// A file was written. `path` is the true on-disk path yt-dlp wrote
    /// (post audio-extraction, post-move).
    Downloaded {
        output: YoutubeDlOutput,
        path: PathBuf,
    },
    /// Nothing was downloaded, e.g. a `--download-archive` skip.
    AlreadyDownloaded,
}

pub trait MediaHandler {
    fn download(
        &self,
        url: &str,
        base_path: &Path,
        library: &str,
        filename: Option<&str>,
        override_if_exists: bool,
    ) -> crate::error::Result<DownloadOutcome>;

    fn get_media_metadata(&self, url: &str) -> crate::error::Result<YoutubeDlOutput>;
}

pub struct YtDlp;

impl MediaHandler for YtDlp {
    fn download(
        &self,
        url: &str,
        base_path: &Path,
        library: &str,
        filename: Option<&str>,
        override_if_exists: bool,
    ) -> crate::error::Result<DownloadOutcome> {
        // --default-search option doesn't seem to be working properly, when used
        // `into_single_video` returns None. Going to be expecting full URLs.

        let filename = filename.unwrap_or("%(title)s [%(id)s]");
        let output_path = base_path.join(library).join(filename);
        let output_file_path = output_path
            .to_str()
            .ok_or_else(|| HtbError::Other(format!("Invalid path: {}", output_path.display())))?;

        debug!("Downloading to: {}", output_file_path);

        // Scratch file yt-dlp prints the true post-move output path to, via
        // --print-to-file below. It appends, so any stale content from a
        // prior run reusing this PID must be cleared first.
        let scratch_path =
            std::env::temp_dir().join(format!("htb-filepath-{}.txt", std::process::id()));
        reset_scratch_file(&scratch_path)?;
        let scratch_arg = scratch_path
            .to_str()
            .ok_or_else(|| HtbError::Other(format!("Invalid scratch path: {}", scratch_path.display())))?
            .replace('%', "%%"); // the FILE operand is itself an outtmpl

        let mut yt_dl = YoutubeDl::new(url);
        yt_dl
            .youtube_dl_path("yt-dlp")
            .download(true)
            .extract_audio(true)
            .extra_arg("--no-playlist")
            .extra_arg("--no-continue")
            .extra_arg("-f")
            .extra_arg("bestaudio")
            .extra_arg("--downloader")
            .extra_arg("ffmpeg")
            .extra_arg("--audio-format")
            .extra_arg("mp3")
            .extra_arg("--audio-quality")
            .extra_arg("0")
            .extra_arg("--no-keep-video")
            .extra_arg("-o")
            .extra_arg(output_file_path)
            .extra_arg("--print-to-file")
            .extra_arg("after_move:filepath")
            .extra_arg(scratch_arg.as_str());

        let mut archive_active = false;
        if !override_if_exists {
            let download_archive_path = base_path.join(DOWNLOAD_ARCHIVE);

            match Self::ensure_download_archive(&download_archive_path) {
                Ok(()) => {
                    if let Some(path_str) = download_archive_path.to_str() {
                        debug!("Using download archive: {}", path_str);
                        yt_dl.extra_arg("--download-archive").extra_arg(path_str);
                        archive_active = true;
                    }
                }
                Err(e) => warn!(
                    "Could not create/access download archive {}: {}. Proceeding without --download-archive.",
                    download_archive_path.display(),
                    e
                ),
            }
        }

        debug!("Executing yt-dlp command");
        let run_result = yt_dl.run();

        let raw = match std::fs::read_to_string(&scratch_path) {
            Ok(s) => Some(s),
            Err(e) if e.kind() == std::io::ErrorKind::NotFound => None,
            Err(e) => return Err(e.into()),
        };
        let resolved_path = parse_printed_path(raw.as_deref());
        debug!("Resolved output path: {:?}", resolved_path);

        if let Err(e) = std::fs::remove_file(&scratch_path) {
            if e.kind() != std::io::ErrorKind::NotFound {
                warn!("Failed to clean up scratch file {}: {}", scratch_path.display(), e);
            }
        }

        match run_result {
            // yt-dlp exits 0 but prints `null` on a `--download-archive` skip;
            // the crate can't deserialize that into `SingleVideo` and surfaces
            // it as a Json error rather than a successful empty output.
            Err(youtube_dl::Error::Json(_)) if archive_active => Ok(DownloadOutcome::AlreadyDownloaded),
            Err(e) => Err(e.into()),
            Ok(output) => match resolved_path {
                Some(path) => Ok(DownloadOutcome::Downloaded { output, path }),
                None => Ok(DownloadOutcome::AlreadyDownloaded),
            },
        }
    }

    fn get_media_metadata(&self, url: &str) -> crate::error::Result<YoutubeDlOutput> {
        debug!("Fetching metadata for URL: {}", url);
        let output = YoutubeDl::new(url)
            .youtube_dl_path("yt-dlp")
            .download(false)
            .extra_arg("--no-playlist")
            .run()?;
        Ok(output)
    }
}

impl YtDlp {
    fn ensure_download_archive(archive_path: &Path) -> std::io::Result<()> {
        // Ensure parent directory exists
        if let Some(parent) = archive_path.parent() {
            std::fs::create_dir_all(parent)?;
        }

        // Create the archive file
        std::fs::OpenOptions::new()
            .create(true)
            .append(true)
            .open(archive_path)
            .map(|_| ()) // We don't need the file handle, just ensure it exists
    }
}

/// Clears a possibly-stale scratch file left over from a prior run reusing
/// this PID. `--print-to-file` appends, so leftover content here would
/// silently corrupt this run's parse.
fn reset_scratch_file(path: &Path) -> crate::error::Result<()> {
    match std::fs::remove_file(path) {
        Ok(()) => Ok(()),
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => Ok(()),
        Err(e) => Err(e.into()),
    }
}

/// Parses the scratch file's raw content into a resolved path, or `None` if
/// nothing was downloaded. Treats absent content, a blank first line, and
/// yt-dlp's literal "NA" placeholder as "no file produced".
fn parse_printed_path(raw: Option<&str>) -> Option<PathBuf> {
    let raw = raw?;
    let line = raw.lines().find(|l| !l.trim().is_empty())?.trim();
    if line.is_empty() || line == "NA" {
        return None;
    }
    Some(PathBuf::from(line))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_printed_path_none_when_absent() {
        assert_eq!(parse_printed_path(None), None);
    }

    #[test]
    fn parse_printed_path_none_when_empty() {
        assert_eq!(parse_printed_path(Some("")), None);
    }

    #[test]
    fn parse_printed_path_none_when_na() {
        assert_eq!(parse_printed_path(Some("NA\n")), None);
    }

    #[test]
    fn parse_printed_path_trims_trailing_newline() {
        assert_eq!(
            parse_printed_path(Some("/music/Title [id].mp3\n")),
            Some(PathBuf::from("/music/Title [id].mp3"))
        );
    }

    #[test]
    fn parse_printed_path_handles_spaces_and_brackets() {
        assert_eq!(
            parse_printed_path(Some("/music/My Song [dQw4w9WgXcQ].mp3\n")),
            Some(PathBuf::from("/music/My Song [dQw4w9WgXcQ].mp3"))
        );
    }

    #[test]
    fn parse_printed_path_takes_first_non_empty_line() {
        assert_eq!(
            parse_printed_path(Some("\n\n/music/Title [id].mp3\n")),
            Some(PathBuf::from("/music/Title [id].mp3"))
        );
    }
}
