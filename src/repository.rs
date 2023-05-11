use crate::config;
use log::info;
use rusqlite::Connection;

pub struct SQLiteRepository {
    conn: rusqlite::Connection,
}

// Implement a Catalog interface
// The idea is to eventually have a service (business layer) to interact with
// the repository for now we are doing it directly
impl SQLiteRepository {
    pub fn new(config: &config::Config) -> SQLiteRepository {
        // NOTE: File in given path might not exist, create it before
        let conn = Connection::open(&config.database_file_path)
            .expect("Failed to establish connection");
        let repo = SQLiteRepository { conn };
        repo.create_schema();

        repo
    }

    fn create_schema(&self) {
        self.conn
            .execute(
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
            )
            .expect("Failed to create schema"); // FIXME: `expect()`
    }
    pub fn insert_media(
        &self,
        name: &str,
        filename: &str,
        directory: &str,
        url: &str,
        tags: &str,
    ) {
        self.conn
            .execute(
                "INSERT INTO media (name, filename, directory, url, tags) VALUES (?1, ?2, ?3, ?4, ?5)",
                [name, filename, directory, url, tags],
            )
            .expect("failed to insert record"); // FIX_ME: `expect()`
    }
}
