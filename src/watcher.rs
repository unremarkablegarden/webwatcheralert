/// Watcher data structure
///
/// Represents a single website being monitored

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use std::time::Duration;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Watcher {
    /// Unique identifier
    pub id: String,

    /// URL to monitor
    pub url: String,

    /// Keywords to search for in new content
    pub keywords: Vec<String>,

    /// How often to check (in seconds)
    #[serde(with = "duration_serde")]
    pub check_interval: Duration,

    /// Whether this watcher is active
    pub enabled: bool,

    /// Last time this watcher was checked
    pub last_checked: Option<DateTime<Utc>>,

    /// Path to cached content
    pub cache_path: PathBuf,
}

impl Watcher {
    /// Create a new watcher
    pub fn new(url: String, keywords: Vec<String>, check_interval: Duration) -> Self {
        let id = uuid::Uuid::new_v4().to_string();
        // Store just the filename, cache module will resolve full path
        let cache_path = PathBuf::from(format!("{}.html", id));

        Self {
            id,
            url,
            keywords,
            check_interval,
            enabled: true,
            last_checked: None,
            cache_path,
        }
    }

    /// Get the full cache file path
    pub fn full_cache_path(&self) -> anyhow::Result<PathBuf> {
        let cache_dir = crate::config::Config::cache_dir()?;
        Ok(cache_dir.join(&self.cache_path))
    }
}

// Helper module for serializing Duration
mod duration_serde {
    use serde::{Deserialize, Deserializer, Serializer};
    use std::time::Duration;

    pub fn serialize<S>(duration: &Duration, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        serializer.serialize_u64(duration.as_secs())
    }

    pub fn deserialize<'de, D>(deserializer: D) -> Result<Duration, D::Error>
    where
        D: Deserializer<'de>,
    {
        let secs = u64::deserialize(deserializer)?;
        Ok(Duration::from_secs(secs))
    }
}
