use crate::cli::TagOverrides;
use crate::error::{HtbError, Result};
use std::fmt;

/// A full ID3 tag snapshot - either what was read off a file or what should
/// be written to one. Unlike `TagOverrides` (a CLI-supplied delta), every
/// field here represents the tag's actual current value, or `None` because
/// the tag genuinely has none.
#[derive(Debug, Default, Clone, PartialEq)]
pub struct Id3Tags {
    pub title: Option<String>,
    pub artist: Option<String>,
    pub album: Option<String>,
    pub genre: Option<String>,
    pub track: Option<u32>,
    pub year: Option<u16>,
}

impl Id3Tags {
    /// Whether there is nothing here. Gates every file rewrite, so an
    /// inversion here means either every download rewrites the file or none does.
    pub fn is_empty(&self) -> bool {
        self.title.is_none()
            && self.artist.is_none()
            && self.album.is_none()
            && self.track.is_none()
            && self.year.is_none()
            && self.genre.is_none()
    }

    /// Layers a CLI-supplied delta over this snapshot; only fields `updates`
    /// actually set change.
    pub fn overlay(&self, updates: &TagOverrides) -> Id3Tags {
        Id3Tags {
            title: updates.title.clone().or_else(|| self.title.clone()),
            artist: updates.artist.clone().or_else(|| self.artist.clone()),
            album: updates.album.clone().or_else(|| self.album.clone()),
            track: updates.track.or(self.track),
            year: updates.year.or(self.year),
            genre: updates.genre.clone().or_else(|| self.genre.clone()),
        }
    }
}

#[derive(Debug)]
pub struct Media {
    pub name: String,
    pub filename: String,
    pub library: String,
    pub url: String,
    pub tags: String,
    pub id3: Id3Tags,
}

impl Media {
    pub fn builder() -> MediaBuilder {
        MediaBuilder::default()
    }
}

#[derive(Default)]
pub struct MediaBuilder {
    name: Option<String>,
    filename: Option<String>,
    library: Option<String>,
    url: Option<String>,
    tags: Option<String>,
    id3: Option<Id3Tags>,
}

impl MediaBuilder {
    pub fn name(mut self, name: impl Into<String>) -> Self {
        self.name = Some(name.into());
        self
    }

    pub fn filename(mut self, filename: impl Into<String>) -> Self {
        self.filename = Some(filename.into());
        self
    }

    pub fn library(mut self, library: impl Into<String>) -> Self {
        self.library = Some(library.into());
        self
    }

    pub fn url(mut self, url: impl Into<String>) -> Self {
        self.url = Some(url.into());
        self
    }

    pub fn tags(mut self, tags: impl Into<String>) -> Self {
        self.tags = Some(tags.into());
        self
    }

    // The six tag fields always travel together, so they get one setter
    // rather than six.
    pub fn id3(mut self, id3: Id3Tags) -> Self {
        self.id3 = Some(id3);
        self
    }

    pub fn build(self) -> Result<Media> {
        Ok(Media {
            name: self.name.ok_or(HtbError::Builder { field: "name" })?,
            filename: self
                .filename
                .ok_or(HtbError::Builder { field: "filename" })?,
            library: self.library.ok_or(HtbError::Builder { field: "library" })?,
            url: self.url.ok_or(HtbError::Builder { field: "url" })?,
            tags: self.tags.unwrap_or_default(),
            id3: self.id3.unwrap_or_default(),
        })
    }
}

impl fmt::Display for Media {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}\t{}\t{}", self.name, self.library, self.filename)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn stored() -> Id3Tags {
        Id3Tags {
            title: Some("Stored Title".into()),
            artist: Some("Stored Artist".into()),
            year: Some(2000),
            ..Default::default()
        }
    }

    #[test]
    fn overlay_replaces_only_the_given_fields() {
        let updates = TagOverrides {
            artist: Some("New Artist".into()),
            genre: Some("Pop".into()),
            ..Default::default()
        };

        let merged = stored().overlay(&updates);

        assert_eq!(merged.artist.as_deref(), Some("New Artist"));
        assert_eq!(merged.genre.as_deref(), Some("Pop"));
        // Untouched fields survive.
        assert_eq!(merged.title.as_deref(), Some("Stored Title"));
        assert_eq!(merged.year, Some(2000));
        assert_eq!(merged.album, None);
    }

    #[test]
    fn overlay_with_nothing_is_a_noop() {
        assert_eq!(stored().overlay(&TagOverrides::default()), stored());
    }

    #[test]
    fn is_empty_when_default() {
        assert!(Id3Tags::default().is_empty());
    }

    #[test]
    fn not_empty_when_any_field_is_set() {
        let tags = Id3Tags {
            artist: Some("a".into()),
            ..Default::default()
        };

        assert!(!tags.is_empty());
    }
}
