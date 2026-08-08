use crate::error::{HtbError, Result};
use dirs::{audio_dir, config_dir, home_dir};
use serde::{Deserialize, Serialize};
use std::{fs::File, path::PathBuf};

const CONFIG_FILE_NAME: &str = "config.json";
const CONFIG_PATH_ENV_VAR: &str = "HTB_CONFIG";

#[derive(Debug, Deserialize, Serialize)]
pub struct Config {
    #[serde(skip)]
    pub path: PathBuf, // Where this config was loaded from; not part of the file's own content

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

// Default path to the config file, used when neither --config nor HTB_CONFIG is given.
fn default_config_path() -> Result<PathBuf> {
    Ok(config_dir()
        .ok_or_else(|| HtbError::Config("Could not get config directory".to_string()))?
        .join("htb")
        .join(CONFIG_FILE_NAME))
}

// Resolves the config path: --config flag, then HTB_CONFIG, then the default location.
pub fn resolve_path(cli_override: Option<PathBuf>) -> Result<PathBuf> {
    if let Some(path) = cli_override {
        return Ok(path);
    }
    if let Ok(path) = std::env::var(CONFIG_PATH_ENV_VAR) {
        return Ok(PathBuf::from(path));
    }
    default_config_path()
}

impl Config {
    pub fn from_path(config_path: PathBuf) -> Result<Self> {
        let mut config = match create_if_not_exists(&config_path)? {
            Some(config) => config,
            None => {
                // If the configuration file exists, read it and deserialize it
                let file = File::open(&config_path)?;
                serde_json::from_reader(file)?
            }
        };
        config.path = config_path;

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
        path: PathBuf::new(), // overwritten by the caller once loaded
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
