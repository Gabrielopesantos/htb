use std::path::PathBuf;

use youtube_dl::{YoutubeDl, YoutubeDlOutput};

pub trait MediaDownloader {
    fn download(
        &self,
        url: &str,
        base_path: &PathBuf,
        library: &str,
        filename: Option<&str>,
    ) -> Result<YoutubeDlOutput, youtube_dl::Error>;
}

pub struct YtDlp;

impl MediaDownloader for YtDlp {
    fn download(
        &self,
        url: &str,
        base_path: &PathBuf,
        library: &str,
        filename: Option<&str>,
    ) -> Result<YoutubeDlOutput, youtube_dl::Error> {
        // --default-search option doesn't seem to be working properly, when used
        // `into_single_video` returns None. Going to be expecting full URLs.

        let filename = filename.unwrap_or("%(title)s.ext");
        let output_file_path = base_path
            .to_owned()
            .join(library)
            .join(filename)
            .to_str()
            .expect("Path has to always be valid")
            .to_owned(); // `to_owned` needed because `expects` consumes the ownership of
                         // `Option` value

        // NOTE: Eventually also support providing time range
        YoutubeDl::new(url)
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
            .extra_arg(output_file_path)
            .run()
    }
}
