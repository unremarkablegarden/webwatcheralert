// Module declarations
mod cache;
mod config;
mod diff;
mod fetcher;
mod matcher;
mod monitor;
mod notify;
mod ui;
mod watcher;

use anyhow::Result;
use std::env;

fn main() -> Result<()> {
    // Check if running in daemon mode
    let args: Vec<String> = env::args().collect();
    let daemon_mode = args.iter().any(|arg| arg == "--daemon");

    if daemon_mode {
        // Run in daemon mode (background service)
        run_daemon()?;
    } else {
        // Run interactive TUI
        let mut ui = ui::UI::new()?;
        ui.run()?;
    }

    Ok(())
}

fn run_daemon() -> Result<()> {
    // Load configuration
    let config = config::Config::load()?;

    // Print startup message
    println!("Web Watcher Alert - Daemon Mode");
    println!("Starting monitoring for {} watchers...", config.watchers.len());

    // Create monitor and start
    let monitor = monitor::Monitor::new(config);

    // Create Tokio runtime and run monitoring
    let runtime = tokio::runtime::Runtime::new()?;
    runtime.block_on(async {
        monitor.start().await
    })?;

    Ok(())
}
