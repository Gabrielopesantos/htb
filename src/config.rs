use crate::error::{HtbError, Result};
use dirs::config_dir;
use serde::{Deserialize, Serialize};
use std::{fs::File, path::PathBuf};

const CONFIG_FILE_NAME: &str = "config.json";

#[derive(Debug, Deserialize, Serialize)]
pub struct Config {
    pub catalog_path: PathBuf, // Path where the catalog database and media files are stored
    pub no_record: bool,       // If true, will not record downloaded media in the catalog
    pub override_if_exists: bool, // If true, will override existing files when downloading
}

impl Default for Config {
    fn default() -> Self {
        Self {
            catalog_path: PathBuf::from("/tmp/htb"),
            no_record: false,
            override_if_exists: false,
        }
    }
}

impl Config {
    pub fn new() -> Result<Self> {
        let config_path = config_dir()
            .ok_or_else(|| HtbError::Config("Could not get config directory".to_string()))?
            .join("htb")
            .join(CONFIG_FILE_NAME);

        match create_if_not_exists(&config_path)? {
            Some(config) => Ok(config),
            None => {
                // If the configuration file exists, read it and deserialize it
                let file = File::open(&config_path)?;
                let config: Config = serde_json::from_reader(file)?;
                Ok(config)
            }
        }
    }
}

// Create a new configuration file if it does not exist
fn create_if_not_exists(config_path: &PathBuf) -> Result<Option<Config>> {
    if config_path.exists() {
        return Ok(None);
    }

    let default_config = Config::default();
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
