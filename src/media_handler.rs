use log::debug;
use std::path::Path;

use youtube_dl::{YoutubeDl, YoutubeDlOutput};

const DOWNLOAD_ARCHIVE: &str = ".htb_downloaded.txt";

pub trait MediaHandler {
    fn download(
        &self,
        url: &str,
        base_path: &Path,
        library: &str,
        filename: Option<&str>,
        override_if_exists: bool,
    ) -> Result<YoutubeDlOutput, youtube_dl::Error>;

    fn get_media_metadata(&self, url: &str) -> Result<YoutubeDlOutput, youtube_dl::Error>;
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
    ) -> Result<YoutubeDlOutput, youtube_dl::Error> {
        // --default-search option doesn't seem to be working properly, when used
        // `into_single_video` returns None. Going to be expecting full URLs.

        let filename = filename.unwrap_or("%(title)s [%(id)s]");
        let output_path = base_path.join(library).join(filename);
        let output_file_path = output_path.to_str().ok_or_else(|| {
            youtube_dl::Error::Io(std::io::Error::new(
                std::io::ErrorKind::InvalidData,
                "Invalid path",
            ))
        })?;

        debug!("Downloading to: {}", output_file_path);

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
            .extra_arg(output_file_path);

        if !override_if_exists {
            let download_archive_path = base_path.join(DOWNLOAD_ARCHIVE);

            if let Ok(()) = Self::ensure_download_archive(&download_archive_path) {
                if let Some(path_str) = download_archive_path.to_str() {
                    debug!("Using download archive: {}", path_str);
                    yt_dl.extra_arg("--download-archive").extra_arg(path_str);
                }
            }
        }

        debug!("Executing yt-dlp command");
        yt_dl.run()
    }

    fn get_media_metadata(&self, url: &str) -> Result<YoutubeDlOutput, youtube_dl::Error> {
        debug!("Fetching metadata for URL: {}", url);
        YoutubeDl::new(url)
            .youtube_dl_path("yt-dlp")
            .download(false)
            .extra_arg("--no-playlist")
            .run()
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
