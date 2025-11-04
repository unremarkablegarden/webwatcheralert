# Web Watcher Alert

A Rust CLI application that monitors websites for content changes and sends notifications when specified keywords appear in updated content.

## Features

- 📊 **Website Monitoring**: Track multiple URLs for content changes
- 🔍 **Case-Insensitive Keyword Matching**: Get alerted only when specific keywords appear (matches "Sale", "SALE", "sale", etc.)
- 🔔 **macOS Notifications**: Receive native notifications when keywords are found
- ⚙️ **Configurable Intervals**: Set custom check frequencies for each site (minutes)
- 🎨 **Interactive TUI**: User-friendly terminal interface for managing watchers
- 💾 **Smart Caching**: Efficient local storage with intelligent diff detection (ignores whitespace changes)
- ⚡ **Async Monitoring**: Concurrent background tasks check multiple sites simultaneously
- 🎯 **Context Snippets**: Notifications show surrounding text where keywords were found

## Prerequisites

- Rust 1.83.0 or higher
- macOS (for notifications)

## Installation

```bash
# Clone the repository
cd web-watcher-alert

# Build the project
cargo build --release

# Run the application
cargo run --release
```

## Usage

Start the application:

```bash
cargo run
```

The interactive TUI will guide you through:

1. **Add Watcher**: Enter URL, keywords (comma-separated), and check interval
2. **List Watchers**: View all watchers, enable/disable, or delete them
3. **Start Monitoring**: Run the background monitoring process
4. **Exit**: Close the application

### Example

Monitor a product page for sales:
- **URL**: `https://example.com/products`
- **Keywords**: `sale, discount, 50% off` (case-insensitive - will match "Sale", "DISCOUNT", etc.)
- **Check Interval**: `30` (minutes)

When any of these keywords appear in new content, you'll receive a macOS notification with a snippet showing the matched text in context!

## Project Structure

```
src/
├── main.rs       # Entry point and TUI coordinator
├── ui.rs         # Interactive terminal interface
├── config.rs     # Configuration management
├── watcher.rs    # Watcher data structure
├── monitor.rs    # Background monitoring engine
├── fetcher.rs    # HTTP content fetching
├── diff.rs       # Content diffing
├── matcher.rs    # Keyword matching
├── cache.rs      # Local cache management
└── notify.rs     # Notification system
```

## Configuration

- **Config file**: `~/.config/web-watcher-alert/config.json` (JSON format)
- **Cache directory**: `~/.cache/web-watcher-alert/` (HTML files named by UUID)

You can manually edit the config file if needed, but the TUI provides a friendly interface.

## Development

See [PROJECT_PLAN.md](PROJECT_PLAN.md) for detailed implementation plan.

```bash
# Check for errors
cargo check

# Run tests
cargo test

# Run with verbose logging
RUST_LOG=debug cargo run
```

## Status

**v0.1.0 - Fully Functional! ✅**

All core features are implemented and working:

- [x] Phase 1: Project foundation
- [x] Phase 2: Core data structures (Config, Watcher, cache management)
- [x] Phase 3: Content fetching (HTTP client with error handling)
- [x] Phase 4: Diff & matching (Smart diffing, case-insensitive keyword search)
- [x] Phase 5: Notifications (macOS notification center integration)
- [x] Phase 6: Monitoring engine (Async background tasks)
- [x] Phase 7: Interactive TUI (Full menu system with keyboard navigation)
- [x] Phase 8: Build & deployment (Compiled binary ready to use)

## Troubleshooting

### Application won't start or crashes
- Make sure you have Rust 1.83.0 or higher: `rustc --version`
- Try rebuilding: `cargo clean && cargo build --release`
- Check that macOS notification permissions are enabled

### No notifications appearing
- Open System Preferences → Notifications
- Ensure notifications are enabled for Terminal (or your terminal app)
- Test with a short interval (5 minutes) first

### "Cannot start a runtime from within a runtime" error
- This has been fixed in the current version (removed nested tokio runtime)
- Make sure you're using the latest build: `cargo build --release`

## Future Enhancements

- Email notifications (SMTP)
- Webhook support (Discord, Slack, etc.)
- Regular expression patterns for advanced keyword matching
- HTML element selectors (monitor specific page sections)
- Export/import watcher configurations
- Statistics dashboard (check history, match frequency)
- Linux and Windows support

## License

MIT

## Contributing

Contributions welcome! Please open an issue or PR.
