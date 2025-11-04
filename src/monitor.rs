/// Background monitoring engine
///
/// Manages async tasks that periodically check each enabled watcher

use anyhow::{Context, Result};
use chrono::Utc;
use std::sync::Arc;
use tokio::sync::Mutex;
use tokio::time::sleep;

use crate::{cache, config::Config, diff, fetcher, matcher, notify, watcher::Watcher};

pub struct Monitor {
    config: Arc<Mutex<Config>>,
}

impl Monitor {
    pub fn new(config: Config) -> Self {
        Self {
            config: Arc::new(Mutex::new(config)),
        }
    }

    /// Start monitoring all enabled watchers
    /// This will spawn a task for each watcher and run until interrupted
    pub async fn start(&self) -> Result<()> {
        let watchers = {
            let config = self.config.lock().await;
            config.watchers.clone()
        };

        if watchers.is_empty() {
            println!("No watchers configured. Add some watchers first!");
            return Ok(());
        }

        let enabled_watchers: Vec<_> = watchers
            .into_iter()
            .filter(|w| w.enabled)
            .collect();

        if enabled_watchers.is_empty() {
            println!("No enabled watchers. Enable at least one watcher to start monitoring.");
            return Ok(());
        }

        println!("[{}] Starting monitoring for {} watchers...", Utc::now().format("%Y-%m-%d %H:%M:%S"), enabled_watchers.len());
        println!("Press Ctrl+C to stop.\n");

        // Log each watcher being started
        for watcher in &enabled_watchers {
            println!("[{}] Watcher: {} | Keywords: {} | Interval: {}min",
                Utc::now().format("%Y-%m-%d %H:%M:%S"),
                watcher.url,
                watcher.keywords.join(", "),
                watcher.check_interval.as_secs() / 60);
        }
        println!();

        // Spawn a task for each watcher
        let mut handles = Vec::new();
        for watcher in enabled_watchers {
            let config = Arc::clone(&self.config);
            let handle = tokio::spawn(async move {
                monitor_watcher(watcher, config).await
            });
            handles.push(handle);
        }

        // Wait for all tasks (they run indefinitely)
        for handle in handles {
            let _ = handle.await;
        }

        Ok(())
    }
}

/// Monitor a single watcher indefinitely
async fn monitor_watcher(mut watcher: Watcher, config: Arc<Mutex<Config>>) {
    loop {
        // Wait for the check interval
        sleep(watcher.check_interval).await;

        let timestamp = Utc::now().format("%Y-%m-%d %H:%M:%S");
        println!("[{}] Checking {}...", timestamp, watcher.url);

        // Perform the check
        match check_watcher(&mut watcher).await {
            Ok((found_matches, matched_keywords)) => {
                if found_matches {
                    println!("[{}]   ✓ Keywords found: {} | Notification sent",
                        timestamp, matched_keywords.join(", "));
                } else {
                    println!("[{}]   - No changes or keywords found", timestamp);
                }

                // Update last_checked timestamp
                watcher.last_checked = Some(Utc::now());

                // Save updated config
                let mut cfg = config.lock().await;
                if let Some(w) = cfg.watchers.iter_mut().find(|w| w.id == watcher.id) {
                    w.last_checked = watcher.last_checked;
                }
                let _ = cfg.save();
            }
            Err(e) => {
                eprintln!("[{}]   ✗ Error: {}", Utc::now().format("%Y-%m-%d %H:%M:%S"), e);
            }
        }
    }
}

/// Check a single watcher once
/// Returns Ok((found_matches, matched_keywords)) where:
/// - found_matches: true if keywords were found, false otherwise
/// - matched_keywords: list of keywords that were found
async fn check_watcher(watcher: &Watcher) -> Result<(bool, Vec<String>)> {
    // 1. Fetch the URL
    let new_content = fetcher::fetch_url(&watcher.url)
        .await
        .context("Failed to fetch URL")?;

    // 2. Get cached content
    let cache_path = watcher.full_cache_path()?;
    let old_content = cache::read_cache(&cache_path)?;

    // 3. Check if content has changed
    let has_changed = match &old_content {
        Some(old) => diff::has_changed(old, &new_content),
        None => true, // No cache means this is the first check
    };

    if !has_changed {
        return Ok((false, Vec::new()));
    }

    // 4. Content has changed, search for keywords
    let matches = matcher::find_keywords(&new_content, &watcher.keywords);

    // 5. Send notification if keywords found
    if !matches.is_empty() {
        // Get unique keywords that were matched
        let matched_keywords: Vec<String> = matches
            .iter()
            .map(|m| m.keyword.clone())
            .collect::<std::collections::HashSet<_>>()
            .into_iter()
            .collect();

        notify::send_notification(&watcher.url, &matches)?;

        // Update cache since we found matches
        cache::write_cache(&cache_path, &new_content)?;

        return Ok((true, matched_keywords));
    }

    // 6. No keywords found, but still update cache
    cache::write_cache(&cache_path, &new_content)?;

    Ok((false, Vec::new()))
}
