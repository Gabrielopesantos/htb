use std::fmt;

#[derive(Debug)]
pub struct Media {
    pub name: String,
    pub filename: String,
    pub library: String,
    pub url: String,
    pub tags: String,
    // pub format String,
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

    pub fn build(self) -> Result<Media, &'static str> {
        Ok(Media {
            name: self.name.ok_or("name is required")?,
            filename: self.filename.ok_or("filename is required")?,
            library: self.library.ok_or("library is required")?,
            url: self.url.ok_or("url is required")?,
            tags: self.tags.unwrap_or_default(),
        })
    }
}

impl fmt::Display for Media {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}\t{}\t{}", self.name, self.library, self.filename)
    }
}
