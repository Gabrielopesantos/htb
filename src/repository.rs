use crate::{config, error::Result, media::Media};
use rusqlite::Connection;

const DB_FILE_NAME: &str = "catalog.db";

pub trait Repository {
    fn insert_into_media(&self, media: &Media) -> Result<()>;
    fn query(&self, directory: &str, tags: &str) -> Result<Vec<Media>>;
}

pub struct SQLiteRepository {
    conn: rusqlite::Connection,
}

impl SQLiteRepository {
    pub fn new(config: &config::Config) -> Result<Self> {
        // File in given path might not exist, create it before
        let conn = Connection::open(config.catalog_path.join(DB_FILE_NAME))?;
        let repo = SQLiteRepository { conn };
        repo.apply_schema()?;

        Ok(repo)
    }

    fn apply_schema(&self) -> Result<()> {
        self.conn.execute(
            "
CREATE TABLE IF NOT EXISTS media (
    id INTEGER PRIMARY KEY ASC,
    name TEXT NOT NULL,
    filename TEXT NOT NULL,
    directory TEXT NOT NULL,
    url TEXT NOT NULL,
    tags TEXT,
    inserted_at TEXT NOT NULL DEFAULT CURRENT_TIMESTAMP
)",
            (),
        )?;

        // Create indexes for commonly queried columns
        self.conn.execute(
            "CREATE INDEX IF NOT EXISTS idx_media_directory ON media(directory)",
            (),
        )?;

        self.conn.execute(
            "CREATE INDEX IF NOT EXISTS idx_media_tags ON media(tags)",
            (),
        )?;

        self.conn.execute(
            "CREATE INDEX IF NOT EXISTS idx_media_inserted_at ON media(inserted_at)",
            (),
        )?;

        Ok(())
    }
}

impl Repository for SQLiteRepository {
    fn insert_into_media(&self, media: &Media) -> Result<()> {
        self.conn.execute(
            "INSERT INTO media (name, filename, directory, url, tags) VALUES (?1, ?2, ?3, ?4, ?5)",
            [
                &media.name,
                &media.filename,
                &media.library,
                &media.url,
                &media.tags,
            ],
        )?;
        Ok(())
    }

    fn query(
        &self,
        directory: &str,
        tags: &str,
    ) -> Result<Vec<Media>> {
        // Build query with tag filtering support
        let query = "
            SELECT name, filename, directory, url, tags
            FROM media
            WHERE (directory = :directory OR :directory = '')
              AND (tags LIKE '%' || :tags || '%' OR :tags = '')
        ";

        let mut stmt = self.conn.prepare(query)?;

        let rows = stmt.query_map(
            &[(":directory", directory), (":directory", directory), (":tags", tags), (":tags", tags)],
            |row| {
                // NOTE: Maybe have a `new` instead?
                Ok(Media {
                    name: row.get(0)?,
                    filename: row.get(1)?,
                    library: row.get(2)?,
                    url: row.get(3)?,
                    tags: row.get(4)?,
                })
            },
        )?;

        let mut catalog_items = Vec::new();
        for row in rows {
            catalog_items.push(row?);
        }

        Ok(catalog_items)
    }
}

pub struct DummyRepository;

impl Repository for DummyRepository {
    fn insert_into_media(&self, _media: &Media) -> Result<()> {
        Ok(())
    }

    fn query(&self, _directory: &str, _tags: &str) -> Result<Vec<Media>> {
        Ok(vec![])
    }
}
