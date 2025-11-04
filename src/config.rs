/// Configuration management module
///
/// Handles loading and saving the application configuration,
/// including all watchers, to ~/.config/web-watcher-alert/config.json

use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::PathBuf;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Config {
    pub watchers: Vec<crate::watcher::Watcher>,
}

impl Config {
    /// Load configuration from disk, or create new if doesn't exist
    pub fn load() -> Result<Self> {
        let config_path = Self::config_path()?;

        // If config doesn't exist, return empty config
        if !config_path.exists() {
            return Ok(Self {
                watchers: Vec::new(),
            });
        }

        // Read and parse the config file
        let contents = fs::read_to_string(&config_path)
            .context("Failed to read config file")?;

        let config: Config = serde_json::from_str(&contents)
            .context("Failed to parse config file")?;

        Ok(config)
    }

    /// Save configuration to disk
    pub fn save(&self) -> Result<()> {
        let config_path = Self::config_path()?;

        // Create config directory if it doesn't exist
        if let Some(parent) = config_path.parent() {
            fs::create_dir_all(parent)
                .context("Failed to create config directory")?;
        }

        // Serialize and write config
        let contents = serde_json::to_string_pretty(self)
            .context("Failed to serialize config")?;

        fs::write(&config_path, contents)
            .context("Failed to write config file")?;

        Ok(())
    }

    /// Get the config file path
    pub fn config_path() -> Result<PathBuf> {
        // Get home directory
        let home = dirs::home_dir()
            .context("Could not find home directory")?;

        // Build path: ~/.config/web-watcher-alert/config.json
        let config_path = home
            .join(".config")
            .join("web-watcher-alert")
            .join("config.json");

        Ok(config_path)
    }

    /// Get the cache directory path
    pub fn cache_dir() -> Result<PathBuf> {
        let home = dirs::home_dir()
            .context("Could not find home directory")?;

        let cache_dir = home
            .join(".cache")
            .join("web-watcher-alert");

        // Create cache directory if it doesn't exist
        fs::create_dir_all(&cache_dir)
            .context("Failed to create cache directory")?;

        Ok(cache_dir)
    }
}
