/// Notification system module
///
/// Sends macOS notifications when keywords are found

use anyhow::{Context, Result};
use crate::matcher::KeywordMatch;
use notify_rust::Notification;

/// Send a macOS notification about keyword matches
pub fn send_notification(url: &str, matches: &[KeywordMatch]) -> Result<()> {
    if matches.is_empty() {
        return Ok(());
    }

    // Get unique keywords that were found
    let keywords: Vec<String> = matches
        .iter()
        .map(|m| m.keyword.clone())
        .collect::<std::collections::HashSet<_>>()
        .into_iter()
        .collect();

    let keyword_list = keywords.join(", ");

    // Create notification title
    let title = format!("Web Watcher Alert: {} found!", keyword_list);

    // Create notification body with context from first match
    let body = if matches.len() == 1 {
        format!("Found on {}\n\n{}", url, matches[0].context)
    } else {
        format!(
            "Found {} matches on {}\n\n{}",
            matches.len(),
            url,
            matches[0].context
        )
    };

    // Send the notification
    Notification::new()
        .summary(&title)
        .body(&body)
        .sound_name("default")
        .show()
        .context("Failed to send notification")?;

    Ok(())
}
