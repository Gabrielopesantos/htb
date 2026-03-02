use std::io;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum HtbError {
    #[error("Database error: {0}")]
    Database(#[from] rusqlite::Error),

    #[error("Media handler error: {0}")]
    MediaHandler(#[from] youtube_dl::Error),

    #[error("Configuration error: {0}")]
    Config(String),

    #[error("I/O error: {0}")]
    Io(#[from] io::Error),

    #[error("JSON serialization error: {0}")]
    Json(#[from] serde_json::Error),

    #[error("Media builder error: {field} is required")]
    Builder { field: &'static str },

    #[error("{0}")]
    Other(String),
}

pub type Result<T> = std::result::Result<T, HtbError>;
