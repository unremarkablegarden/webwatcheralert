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

        println!("Starting monitoring for {} watchers...", enabled_watchers.len());
        println!("Press Ctrl+C to stop.\n");

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

        println!("[{}] Checking {}...", Utc::now().format("%H:%M:%S"), watcher.url);

        // Perform the check
        match check_watcher(&mut watcher).await {
            Ok(found_matches) => {
                if found_matches {
                    println!("  ✓ Keywords found! Notification sent.");
                } else {
                    println!("  - No changes or keywords found.");
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
                eprintln!("  ✗ Error: {}", e);
            }
        }
    }
}

/// Check a single watcher once
/// Returns Ok(true) if keywords were found, Ok(false) otherwise
async fn check_watcher(watcher: &Watcher) -> Result<bool> {
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
        return Ok(false);
    }

    // 4. Content has changed, search for keywords
    let matches = matcher::find_keywords(&new_content, &watcher.keywords);

    // 5. Send notification if keywords found
    if !matches.is_empty() {
        notify::send_notification(&watcher.url, &matches)?;

        // Update cache since we found matches
        cache::write_cache(&cache_path, &new_content)?;

        return Ok(true);
    }

    // 6. No keywords found, but still update cache
    cache::write_cache(&cache_path, &new_content)?;

    Ok(false)
}
