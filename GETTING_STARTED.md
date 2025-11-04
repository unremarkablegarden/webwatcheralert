# Getting Started with Web Watcher Alert

## What We Built

A fully functional Rust CLI application that:
- ✅ Monitors websites for content changes
- ✅ Searches for keywords in updated content
- ✅ Sends macOS notifications when keywords are found
- ✅ Interactive TUI for managing watchers
- ✅ Configurable check intervals per watcher
- ✅ Enable/disable watchers
- ✅ Smart diff detection (ignores whitespace changes)
- ✅ Local caching system

## Quick Start

### 1. Run the Application

```bash
# From the project directory
cargo run --release

# Or use the binary directly
./target/release/web-watcher-alert
```

### 2. Add Your First Watcher

When the TUI loads:

1. Press `1` or `Enter` on "Add Watcher"
2. Fill in the form:
   - **URL**: The website to monitor (e.g., `https://example.com`)
   - **Keywords**: Comma-separated keywords to search for (e.g., `sale, discount, 50% off`)
   - **Interval**: How often to check in minutes (e.g., `30`)
3. Use `Tab` to move between fields
4. Press `Enter` to save
5. Press `Esc` to return to main menu

### 3. Manage Watchers

From the main menu, press `2` or select "List Watchers":

- **↑/↓ or j/k**: Navigate the list
- **t**: Toggle watcher enabled/disabled
- **d**: Delete watcher
- **a**: Add new watcher
- **Esc**: Return to main menu

### 4. Start Monitoring

From the main menu, press `3` or select "Start Monitoring":

- The app will check each enabled watcher at its configured interval
- When content changes and keywords are found, you'll get a macOS notification
- Press `Ctrl+C` to stop monitoring

## Running as a Background Service

Want to keep monitoring even after closing the terminal? Set up the background service:

### Initial Setup (One Time)

```bash
# 1. First, configure your watchers using the TUI
cargo run

# 2. Add your watchers, then exit

# 3. Build the release version
cargo build --release

# 4. Install the service
./scripts/install-service.sh
```

You'll see a success message with available commands.

### Daily Usage

```bash
# Start monitoring in background
./scripts/service.sh start

# Check if it's running
./scripts/service.sh status

# View recent activity
./scripts/service.sh logs

# Stop when you want to make changes
./scripts/service.sh stop
```

### Viewing Logs

```bash
# See last 50 lines of logs
./scripts/service.sh logs

# Follow logs in real-time (like tail -f)
./scripts/service.sh logs-tail

# View full stdout log
./scripts/service.sh logs-stdout

# View full error log
./scripts/service.sh logs-stderr
```

### Log Format

Logs show:
- Timestamp for each check
- Which URL is being checked
- Whether keywords were found
- Any errors that occurred

Example log output:
```
[2025-11-04 16:30:15] Starting monitoring for 2 watchers...
[2025-11-04 16:30:15] Watcher: https://example.com | Keywords: sale, discount | Interval: 30min
[2025-11-04 16:30:15] Checking https://example.com...
[2025-11-04 16:30:16]   ✓ Keywords found: sale, discount | Notification sent
```

### Benefits of Background Service

- ✅ **Runs 24/7** - Keeps monitoring even when you log out of terminal
- ✅ **Persistent** - Terminal can be closed, monitoring continues
- ✅ **Logged** - All activity saved to files for review
- ✅ **Easy Control** - Simple commands to start/stop/check status

## Example Use Cases

### 1. Monitor Product Sales

```
URL: https://store.example.com/products/laptop
Keywords: sale, discount, reduced, clearance
Interval: 60 minutes
```

### 2. Track News

```
URL: https://news.example.com
Keywords: breaking news, urgent, alert
Interval: 15 minutes
```

### 3. Watch for Job Postings

```
URL: https://company.example.com/careers
Keywords: senior developer, remote, rust engineer
Interval: 120 minutes
```

## Configuration Files

- **Config**: `~/.config/web-watcher-alert/config.json`
- **Cache**: `~/.cache/web-watcher-alert/`

You can manually edit the config file if needed (JSON format).

## Tips

1. **Test First**: Start with a shorter interval (5 minutes) to test your watcher
2. **Specific Keywords**: Use specific keywords to avoid too many false positives
3. **Multiple Keywords**: The app alerts if ANY keyword is found
4. **Cache**: On the first check, everything will match (no cached version). Subsequent checks will detect actual changes

## Troubleshooting

### No notifications appearing

- Check System Preferences > Notifications and ensure notifications are enabled
- Make sure at least one watcher is enabled (green checkmark in list)
- Verify the URL is accessible

### Too many notifications

- Increase the check interval
- Use more specific keywords
- Disable watchers you're not actively using

## What's Next?

This is v0.1.0. Future enhancements could include:

- Email notifications
- Webhook support
- Regular expression patterns for keywords
- HTML element selectors (monitor specific page sections)
- Export/import configurations
- Statistics dashboard
- Multiple notification channels

## Development

To modify or extend:

```bash
# Check for errors
cargo check

# Run tests
cargo test

# Build release
cargo build --release

# Run with debug logging
RUST_LOG=debug cargo run
```

See `PROJECT_PLAN.md` for architecture details and `claude.md` for project context.

## Need Help?

Check the README.md for more details or open an issue on GitHub.

Happy monitoring! 🔍
