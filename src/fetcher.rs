/// HTTP content fetching module
///
/// Fetches webpage content with error handling

use anyhow::{Context, Result};
use std::time::Duration;

/// Fetch content from a URL
pub async fn fetch_url(url: &str) -> Result<String> {
    // Create HTTP client with timeout
    let client = reqwest::Client::builder()
        .timeout(Duration::from_secs(30))
        .user_agent("Mozilla/5.0 (Macintosh; Intel Mac OS X 10_15_7) AppleWebKit/537.36")
        .build()
        .context("Failed to create HTTP client")?;

    // Fetch the URL
    let response = client
        .get(url)
        .send()
        .await
        .with_context(|| format!("Failed to fetch URL: {}", url))?;

    // Check if response was successful
    if !response.status().is_success() {
        anyhow::bail!("HTTP error {}: {}", response.status(), url);
    }

    // Get the response text
    let content = response
        .text()
        .await
        .context("Failed to read response body")?;

    Ok(content)
}
