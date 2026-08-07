use crate::error::{HtbError, Result};
use dirs::{audio_dir, config_dir, home_dir};
use serde::{Deserialize, Serialize};
use std::{fs::File, path::PathBuf};

const CONFIG_FILE_NAME: &str = "config.json";

#[derive(Debug, Deserialize, Serialize)]
pub struct Config {
    pub catalog_path: PathBuf, // Path where the catalog database and audio files are stored
    pub no_record: bool,       // If true, will not record downloaded audio in the catalog
    pub override_if_exists: bool, // If true, will override existing files when downloading
}

// Falls back to the home directory if the platform doesn't report a music directory.
fn default_catalog_path() -> Result<PathBuf> {
    audio_dir()
        .map(|music| music.join("htb"))
        .or_else(|| home_dir().map(|home| home.join("htb")))
        .ok_or_else(|| HtbError::Config("Could not determine a default catalog path".to_string()))
}

// Path to the config file.
pub fn config_path() -> Result<PathBuf> {
    Ok(config_dir()
        .ok_or_else(|| HtbError::Config("Could not get config directory".to_string()))?
        .join("htb")
        .join(CONFIG_FILE_NAME))
}

impl Config {
    pub fn new() -> Result<Self> {
        let config_path = config_path()?;

        let config = match create_if_not_exists(&config_path)? {
            Some(config) => config,
            None => {
                // If the configuration file exists, read it and deserialize it
                let file = File::open(&config_path)?;
                serde_json::from_reader(file)?
            }
        };

        // catalog_path might not exist yet (fresh default, or a hand-edited
        // config pointing somewhere new), so create it if necessary
        std::fs::create_dir_all(&config.catalog_path)?;

        Ok(config)
    }
}

// Create a new configuration file if it does not exist
fn create_if_not_exists(config_path: &PathBuf) -> Result<Option<Config>> {
    if config_path.exists() {
        return Ok(None);
    }

    let default_config = Config {
        catalog_path: default_catalog_path()?,
        no_record: false,
        override_if_exists: false,
    };
    let config_json = serde_json::to_string_pretty(&default_config)?;

    // Safely handle parent directory creation
    if let Some(parent) = config_path.parent() {
        std::fs::create_dir_all(parent)?;
    } else {
        return Err(HtbError::Config(
            "Config path has no parent directory".to_string(),
        ));
    }

    std::fs::write(config_path, config_json)?;

    Ok(Some(default_config))
}
