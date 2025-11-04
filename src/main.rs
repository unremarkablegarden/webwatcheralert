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

fn main() -> Result<()> {
    // Initialize the TUI
    let mut ui = ui::UI::new()?;

    // Run the interactive interface
    ui.run()?;

    Ok(())
}
