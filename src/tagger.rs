use crate::cli::TagOverrides;
use crate::error::Result;
use lofty::config::{ParseOptions, WriteOptions};
use lofty::file::AudioFile;
use lofty::id3::v2::{Frame, FrameId, Id3v2Tag, TimestampFrame};
use lofty::mpeg::MpegFile;
use lofty::tag::items::Timestamp;
use lofty::tag::Accessor;
use lofty::TextEncoding;
use log::debug;
use std::path::Path;

/// Writes `overrides` over whatever tags are already on the file, leaving
/// every field the caller did not set alone (including the cover art yt-dlp
/// embedded).
pub fn apply_overrides(path: &Path, overrides: &TagOverrides) -> Result<()> {
    if overrides.is_empty() {
        return Ok(());
    }

    debug!("Applying tag overrides to {}", path.display());

    let mut file = std::fs::File::open(path)?;
    let mut mpeg = MpegFile::read_from(&mut file, ParseOptions::new().read_properties(false))?;
    drop(file);

    if mpeg.id3v2().is_none() {
        mpeg.set_id3v2(Id3v2Tag::new());
    }
    let tag = mpeg.id3v2_mut().expect("just inserted");

    if let Some(title) = &overrides.title {
        tag.set_title(title.clone()); // TIT2
    }
    if let Some(artist) = &overrides.artist {
        tag.set_artist(artist.clone()); // TPE1
    }
    if let Some(album) = &overrides.album {
        tag.set_album(album.clone()); // TALB
    }
    if let Some(genre) = &overrides.genre {
        tag.set_genre(genre.clone()); // TCON
    }
    if let Some(track) = overrides.track {
        tag.set_track(track); // TRCK
    }
    if let Some(year) = overrides.year {
        // The year needs a real `Frame::Timestamp`, inserted by hand.
        //
        // `Accessor` has no `set_year`, and `insert_text(ItemKey::Year, ..)`
        // silently returns false because the ID3v2 key map only has
        // `TDRC => RecordingDate`. `Accessor::set_date` looks like the answer
        // but inserts a `Frame::Text`, and the ID3v2.3 writer only splits
        // TDRC into TYER/TDAT for `Frame::Timestamp` - anything else is
        // discarded with a warning, silently losing the year.
        //
        // Latin1 covers a 4-digit year and avoids the UTF-8 -> UTF-16
        // substitution v2.3 would otherwise apply.
        tag.insert(Frame::Timestamp(TimestampFrame::new(
            FrameId::Valid("TDRC".into()),
            TextEncoding::Latin1,
            Timestamp {
                year,
                ..Timestamp::default()
            },
        )));
    }

    // v2.3 matches what ffmpeg wrote via --embed-thumbnail and is the most
    // widely supported.
    mpeg.save_to_path(path, WriteOptions::new().use_id3v23(true))?;

    Ok(())
}
