use std::fmt;

pub struct Media {
    pub name: String,
    pub filename: String,
    pub directory: String,
    pub url: String,
    pub tags: String,
}

impl fmt::Display for Media {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(
            f,
            "Name: {}\t| Library: {}\t| Filename: {}",
            self.name, self.directory, self.filename
        )
    }
}
