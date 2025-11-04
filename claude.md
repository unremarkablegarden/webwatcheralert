# Web Watcher Alert

## Project Context
This is a Rust CLI application that monitors websites for content changes and sends notifications when specified keywords appear in new content.

## Tech Stack
- **Language**: Rust (1.83.0)
- **Async Runtime**: Tokio
- **HTTP Client**: reqwest
- **TUI**: ratatui + crossterm
- **Diffing**: similar crate
- **Notifications**: notify-rust (macOS)
- **Config**: JSON with serde

## Project Structure
```
src/
├── main.rs          # Entry point, TUI coordinator
├── ui.rs            # TUI screens and rendering
├── config.rs        # Configuration management (JSON load/save)
├── watcher.rs       # Watcher data structure
├── monitor.rs       # Background monitoring engine
├── fetcher.rs       # HTTP content fetching
├── diff.rs          # Content diffing logic
├── matcher.rs       # Keyword matching
├── cache.rs         # Local cache management
└── notify.rs        # macOS notification system
```

## Key Features
1. **Interactive TUI**: Menu-driven interface for managing watchers
2. **Background Monitoring**: Async tasks check URLs at configurable intervals
3. **Smart Diffing**: Detects meaningful content changes
4. **Keyword Search**: Matches keywords in new content only
5. **macOS Notifications**: Native notification center alerts

## Data Flow
1. User adds watcher via TUI (URL + keywords + interval)
2. Monitoring engine schedules periodic checks
3. On each check: fetch content → compare diff → search keywords → notify if match
4. Cache updated after successful check

## Configuration Storage
- Config file: `~/.config/web-watcher-alert/config.json`
- Cache directory: `~/.cache/web-watcher-alert/`

## Watcher Model
```rust
struct Watcher {
    id: String,              // Unique identifier
    url: String,             // URL to monitor
    keywords: Vec<String>,   // Keywords to search for
    check_interval: Duration,// How often to check
    enabled: bool,           // Active/inactive status
    last_checked: Option<DateTime>, // Last check timestamp
    cache_path: PathBuf,     // Path to cached content
}
```

## Development Notes
- First Rust project for the user - keep code clear and well-commented
- Prioritize error handling (network failures, invalid URLs)
- Use anyhow/thiserror for ergonomic error handling
- Keep TUI responsive - run monitoring in background tasks
- Test with real websites during development

## Next Steps (See PROJECT_PLAN.md)
Currently implementing Phase 1-2: project setup and core data structures
