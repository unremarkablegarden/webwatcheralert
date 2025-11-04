# Background Service Guide

This guide explains how to run Web Watcher Alert as a background service on macOS.

## Overview

The background service allows Web Watcher Alert to run independently of any terminal session, monitoring websites 24/7 even after you close your terminal or log out.

## Architecture

- **LaunchAgent**: macOS native service management
- **Log Location**: `~/.local/share/web-watcher-alert/logs/`
- **Startup Mode**: Manual (not auto-start on login)
- **Control**: Simple bash scripts for management

## Quick Start

### 1. Configure Watchers

First, add your watchers using the interactive TUI:

```bash
cargo run
```

Add watchers, configure keywords and intervals, then exit.

### 2. Install Service

```bash
# Build release binary
cargo build --release

# Install the service
./scripts/install-service.sh
```

### 3. Start Monitoring

```bash
./scripts/service.sh start
```

That's it! Monitoring is now running in the background.

## Commands

### Start/Stop

```bash
./scripts/service.sh start    # Start monitoring
./scripts/service.sh stop     # Stop monitoring
./scripts/service.sh restart  # Restart service
```

### Status & Logs

```bash
./scripts/service.sh status       # Check if running
./scripts/service.sh logs         # View last 50 lines
./scripts/service.sh logs-tail    # Follow logs (Ctrl+C to exit)
./scripts/service.sh logs-stdout  # View full stdout log
./scripts/service.sh logs-stderr  # View full error log
```

### Uninstall

```bash
./scripts/service.sh uninstall  # Remove service (keeps logs)
```

## Log Format

The service writes detailed logs with timestamps:

```
[2025-11-04 16:30:15] Starting monitoring for 2 watchers...
[2025-11-04 16:30:15] Watcher: https://example.com | Keywords: sale, discount | Interval: 30min
[2025-11-04 16:30:15] Watcher: https://news.example.com | Keywords: breaking | Interval: 15min

[2025-11-04 16:30:15] Checking https://example.com...
[2025-11-04 16:30:16]   ✓ Keywords found: sale, discount | Notification sent

[2025-11-04 16:45:15] Checking https://news.example.com...
[2025-11-04 16:45:16]   - No changes or keywords found
```

## Files Created

When you install the service, these files are created:

- `~/Library/LaunchAgents/com.webwatcheralert.plist` - LaunchAgent configuration
- `~/.local/share/web-watcher-alert/logs/stdout.log` - Standard output log
- `~/.local/share/web-watcher-alert/logs/stderr.log` - Error log

## How It Works

1. **LaunchAgent** registers the service with macOS
2. When you run `./scripts/service.sh start`, macOS launches the binary with `--daemon` flag
3. The app runs `Monitor::start()` in daemon mode (no TUI)
4. All output goes to log files
5. Service runs until you stop it or system reboots

## Modifying Watchers

To add/edit/remove watchers while service is running:

```bash
# 1. Stop the service
./scripts/service.sh stop

# 2. Run the TUI to make changes
cargo run

# 3. Start the service again
./scripts/service.sh start
```

## Troubleshooting

### Service won't start

Check error logs:
```bash
./scripts/service.sh logs-stderr
```

Common issues:
- Binary not built: Run `cargo build --release`
- No watchers configured: Add watchers via TUI first
- Port/permission issues: Check stderr log for details

### Can't see notifications

- Ensure System Preferences → Notifications is enabled for Terminal
- Check logs to verify keywords are being found
- Test with a short interval (5 minutes) first

### Service keeps stopping

- Check if there are any errors in stderr log
- Verify URLs are accessible
- Check network connectivity

## Comparison: TUI vs Service

| Feature | TUI Mode | Service Mode |
|---------|----------|--------------|
| Runs in foreground | ✓ | ✗ |
| Persists after closing terminal | ✗ | ✓ |
| Interactive management | ✓ | ✗ |
| Log files | ✗ | ✓ |
| Ideal for | Testing, configuration | Production monitoring |

## Best Practices

1. **Test First**: Use TUI mode to verify watchers work before enabling service
2. **Check Logs**: Regularly review logs to ensure monitoring is working
3. **Reasonable Intervals**: Don't set intervals too short (< 5 minutes) to avoid hammering servers
4. **Monitor Disk Space**: Logs grow over time; consider rotating them periodically

## Advanced: Auto-Start on Login

If you want the service to start automatically when you log in, edit the plist file:

```bash
# Edit the plist
nano ~/Library/LaunchAgents/com.webwatcheralert.plist

# Change this line:
<key>RunAtLoad</key>
<false/>

# To:
<key>RunAtLoad</key>
<true/>

# Reload the service
launchctl unload ~/Library/LaunchAgents/com.webwatcheralert.plist
launchctl load ~/Library/LaunchAgents/com.webwatcheralert.plist
```

Now the service will start automatically on login.

## Logs Rotation

To prevent logs from growing too large, you can set up log rotation:

```bash
# Create a simple log rotation script
cat > ~/.local/bin/rotate-watcher-logs.sh << 'EOF'
#!/bin/bash
LOG_DIR="$HOME/.local/share/web-watcher-alert/logs"
MAX_SIZE=10485760  # 10MB

for log in "$LOG_DIR"/*.log; do
  if [ -f "$log" ] && [ $(stat -f%z "$log") -gt $MAX_SIZE ]; then
    mv "$log" "$log.old"
    touch "$log"
  fi
done
EOF

chmod +x ~/.local/bin/rotate-watcher-logs.sh

# Run weekly via cron
echo "0 0 * * 0 ~/.local/bin/rotate-watcher-logs.sh" | crontab -
```

## Summary

The background service provides a robust way to run Web Watcher Alert 24/7 without keeping a terminal open. It's the recommended approach for production monitoring use cases.

For quick tests or configuration changes, use the TUI mode. For continuous monitoring, use the background service.
