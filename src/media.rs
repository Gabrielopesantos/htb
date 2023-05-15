pub struct Media {
    pub name: String,
    pub filename: String,
    pub directory: String,
    pub url: String,
    pub tags: String,
}

impl Media {
    pub fn info(&self) -> String {
        format!(
            "Name: {}\t | Library: {}\t | Filename: {}",
            self.name, self.directory, self.filename
        )
    }
}
