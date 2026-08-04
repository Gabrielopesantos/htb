use crate::{cli::TagOverrides, config, error::Result, media::Media};
use rusqlite::{params, Connection};

const DB_FILE_NAME: &str = "catalog.db";

/// Columns every read selects, in the order `media_from_row` expects.
const MEDIA_COLUMNS: &str =
    "name, filename, directory, url, tags, title, artist, album, genre, track, year";

pub trait Repository {
    fn insert_into_media(&self, media: &Media) -> Result<()>;
    fn query(&self, directory: &str, tags: &str) -> Result<Vec<Media>>;
    fn find_by_url(&self, url: &str) -> Result<Vec<Media>>;
    fn update_overrides(&self, url: &str, overrides: &TagOverrides) -> Result<usize>;
}

fn media_from_row(row: &rusqlite::Row) -> rusqlite::Result<Media> {
    Ok(Media {
        name: row.get(0)?,
        filename: row.get(1)?,
        library: row.get(2)?,
        url: row.get(3)?,
        tags: row.get::<_, Option<String>>(4)?.unwrap_or_default(),
        overrides: TagOverrides {
            title: row.get(5)?,
            artist: row.get(6)?,
            album: row.get(7)?,
            genre: row.get(8)?,
            track: row.get(9)?,
            year: row.get(10)?,
        },
    })
}

pub struct SQLiteRepository {
    conn: rusqlite::Connection,
}

impl SQLiteRepository {
    pub fn new(config: &config::Config) -> Result<Self> {
        // File in given path might not exist, create it before
        let conn = Connection::open(config.catalog_path.join(DB_FILE_NAME))?;
        Self::from_connection(conn)
    }

    pub fn from_connection(conn: Connection) -> Result<Self> {
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
    title TEXT,
    artist TEXT,
    album TEXT,
    genre TEXT,
    track INTEGER,
    year INTEGER,
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
            "INSERT INTO media (name, filename, directory, url, tags, title, artist, album, genre, track, year)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11)",
            params![
                media.name,
                media.filename,
                media.library,
                media.url,
                media.tags,
                media.overrides.title,
                media.overrides.artist,
                media.overrides.album,
                media.overrides.genre,
                media.overrides.track,
                media.overrides.year,
            ],
        )?;
        Ok(())
    }

    fn query(&self, directory: &str, tags: &str) -> Result<Vec<Media>> {
        // Build query with tag filtering support
        let query = format!(
            "
            SELECT {MEDIA_COLUMNS}
            FROM media
            WHERE (directory = :directory OR :directory = '')
              AND (tags LIKE '%' || :tags || '%' OR :tags = '')
        "
        );

        let mut stmt = self.conn.prepare(&query)?;

        let rows = stmt.query_map(
            &[(":directory", directory), (":tags", tags)],
            media_from_row,
        )?;

        let mut catalog_items = Vec::new();
        for row in rows {
            catalog_items.push(row?);
        }

        Ok(catalog_items)
    }

    fn find_by_url(&self, url: &str) -> Result<Vec<Media>> {
        let query = format!("SELECT {MEDIA_COLUMNS} FROM media WHERE url = ?1");

        let mut stmt = self.conn.prepare(&query)?;
        let rows = stmt.query_map([url], media_from_row)?;

        let mut catalog_items = Vec::new();
        for row in rows {
            catalog_items.push(row?);
        }

        Ok(catalog_items)
    }

    fn update_overrides(&self, url: &str, overrides: &TagOverrides) -> Result<usize> {
        // Writes all six columns, so the caller is responsible for merging the
        // new values over the stored ones first (see `TagOverrides::overlay`).
        let updated = self.conn.execute(
            "UPDATE media
             SET title = ?1, artist = ?2, album = ?3, genre = ?4, track = ?5, year = ?6
             WHERE url = ?7",
            params![
                overrides.title,
                overrides.artist,
                overrides.album,
                overrides.genre,
                overrides.track,
                overrides.year,
                url,
            ],
        )?;

        Ok(updated)
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

    fn find_by_url(&self, _url: &str) -> Result<Vec<Media>> {
        Ok(vec![])
    }

    fn update_overrides(&self, _url: &str, _overrides: &TagOverrides) -> Result<usize> {
        Ok(0)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn repo() -> SQLiteRepository {
        SQLiteRepository::from_connection(Connection::open_in_memory().unwrap()).unwrap()
    }

    fn media(overrides: TagOverrides) -> Media {
        Media::builder()
            .name("Example Song")
            .filename("Example Song [aBcD1234xyz].mp3")
            .library("music/pop")
            .url("https://youtu.be/aBcD1234xyz")
            .tags("chill,instrumental")
            .overrides(overrides)
            .build()
            .unwrap()
    }

    fn all_overrides() -> TagOverrides {
        TagOverrides {
            title: Some("Example Song".into()),
            artist: Some("Example Artist".into()),
            album: Some("Example Album".into()),
            track: Some(1),
            year: Some(2001),
            genre: Some("Ambient".into()),
        }
    }

    #[test]
    fn round_trips_all_overrides() {
        let repo = repo();
        repo.insert_into_media(&media(all_overrides())).unwrap();

        let rows = repo.query("", "").unwrap();

        assert_eq!(rows.len(), 1);
        assert_eq!(rows[0].name, "Example Song");
        assert_eq!(rows[0].library, "music/pop");
        assert_eq!(rows[0].tags, "chill,instrumental");
        assert_eq!(rows[0].overrides, all_overrides());
    }

    #[test]
    fn round_trips_absent_overrides_as_none() {
        let repo = repo();
        repo.insert_into_media(&media(TagOverrides::default()))
            .unwrap();

        let rows = repo.query("", "").unwrap();

        assert_eq!(rows.len(), 1);
        assert_eq!(rows[0].overrides, TagOverrides::default());
        assert!(rows[0].overrides.is_empty());
    }

    #[test]
    fn filters_by_directory_and_tags() {
        let repo = repo();
        repo.insert_into_media(&media(all_overrides())).unwrap();

        assert_eq!(repo.query("music/pop", "").unwrap().len(), 1);
        assert_eq!(repo.query("", "chill").unwrap().len(), 1);
        assert_eq!(repo.query("music/rock", "").unwrap().len(), 0);
        assert_eq!(repo.query("", "jazz").unwrap().len(), 0);
    }

    #[test]
    fn finds_by_url() {
        let repo = repo();
        repo.insert_into_media(&media(all_overrides())).unwrap();

        let found = repo.find_by_url("https://youtu.be/aBcD1234xyz").unwrap();

        assert_eq!(found.len(), 1);
        assert_eq!(found[0].overrides, all_overrides());
        assert!(repo
            .find_by_url("https://youtu.be/nope")
            .unwrap()
            .is_empty());
    }

    #[test]
    fn updates_overrides_by_url() {
        let repo = repo();
        repo.insert_into_media(&media(TagOverrides::default()))
            .unwrap();

        let updated = repo
            .update_overrides("https://youtu.be/aBcD1234xyz", &all_overrides())
            .unwrap();

        assert_eq!(updated, 1);
        let found = repo.find_by_url("https://youtu.be/aBcD1234xyz").unwrap();
        assert_eq!(found[0].overrides, all_overrides());
    }

    #[test]
    fn updating_an_unknown_url_changes_nothing() {
        let repo = repo();
        repo.insert_into_media(&media(all_overrides())).unwrap();

        let updated = repo
            .update_overrides("https://youtu.be/nope", &TagOverrides::default())
            .unwrap();

        assert_eq!(updated, 0);
        let found = repo.find_by_url("https://youtu.be/aBcD1234xyz").unwrap();
        assert_eq!(found[0].overrides, all_overrides());
    }

    #[test]
    fn apply_schema_is_idempotent() {
        let repo = repo();
        repo.insert_into_media(&media(all_overrides())).unwrap();

        repo.apply_schema().unwrap();

        assert_eq!(repo.query("", "").unwrap().len(), 1);
    }
}
