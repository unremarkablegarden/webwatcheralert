/// Cache management module
///
/// Handles reading and writing cached webpage content to disk

use anyhow::{Context, Result};
use std::fs;
use std::path::Path;

/// Read cached content from file
pub fn read_cache(path: &Path) -> Result<Option<String>> {
    // If file doesn't exist, return None
    if !path.exists() {
        return Ok(None);
    }

    // Read and return the content
    let content = fs::read_to_string(path)
        .with_context(|| format!("Failed to read cache file: {}", path.display()))?;

    Ok(Some(content))
}

/// Write content to cache file
pub fn write_cache(path: &Path, content: &str) -> Result<()> {
    // Create parent directories if they don't exist
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)
            .with_context(|| format!("Failed to create cache directory: {}", parent.display()))?;
    }

    // Write content to file
    fs::write(path, content)
        .with_context(|| format!("Failed to write cache file: {}", path.display()))?;

    Ok(())
}
