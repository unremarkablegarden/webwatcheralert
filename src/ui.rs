/// Terminal User Interface module
///
/// This module handles all the interactive TUI screens:
/// - Main menu
/// - Add watcher form
/// - List/edit watchers
/// - Monitoring status view

use anyhow::Result;
use crossterm::{
    event::{self, DisableMouseCapture, EnableMouseCapture, Event, KeyCode},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use ratatui::{
    backend::CrosstermBackend,
    layout::{Alignment, Constraint, Direction, Layout},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, List, ListItem, ListState, Paragraph, Wrap},
    Frame, Terminal,
};
use std::io;
use std::time::Duration;

use crate::{config::Config, monitor::Monitor, watcher::Watcher};

#[derive(Debug, PartialEq)]
enum Screen {
    MainMenu,
    AddWatcher,
    ListWatchers,
    EditWatcher(usize), // Index of watcher being edited
    ServiceControl,
}

#[derive(Debug, PartialEq)]
enum FormField {
    Url,
    Keywords,
    Interval,
}

pub struct UI {
    config: Config,
    screen: Screen,
    menu_state: ListState,
    watcher_list_state: ListState,

    // Form state for adding/editing watchers
    form_field: FormField,
    url_input: String,
    keywords_input: String,
    interval_input: String,

    // Service control state
    service_status_message: String,
    service_is_running: bool,
}

impl UI {
    pub fn new() -> Result<Self> {
        let config = Config::load()?;
        let mut menu_state = ListState::default();
        menu_state.select(Some(0));

        Ok(Self {
            config,
            screen: Screen::MainMenu,
            menu_state,
            watcher_list_state: ListState::default(),
            form_field: FormField::Url,
            url_input: String::new(),
            keywords_input: String::new(),
            interval_input: String::from("30"),
            service_status_message: String::new(),
            service_is_running: false,
        })
    }

    pub fn run(&mut self) -> Result<()> {
        // Setup terminal
        enable_raw_mode()?;
        let mut stdout = io::stdout();
        execute!(stdout, EnterAlternateScreen, EnableMouseCapture)?;
        let backend = CrosstermBackend::new(stdout);
        let mut terminal = Terminal::new(backend)?;

        // Run the UI loop
        let result = self.run_loop(&mut terminal);

        // Restore terminal
        disable_raw_mode()?;
        execute!(
            terminal.backend_mut(),
            LeaveAlternateScreen,
            DisableMouseCapture
        )?;
        terminal.show_cursor()?;

        result
    }

    fn run_loop(&mut self, terminal: &mut Terminal<CrosstermBackend<io::Stdout>>) -> Result<()> {
        loop {
            terminal.draw(|f| self.draw(f))?;

            // Handle input with timeout
            if event::poll(Duration::from_millis(100))? {
                if let Event::Key(key) = event::read()? {
                    if self.handle_input(key.code)? {
                        break; // Exit requested
                    }
                }
            }
        }

        Ok(())
    }

    fn draw(&mut self, f: &mut Frame) {
        match &self.screen {
            Screen::MainMenu => self.draw_main_menu(f),
            Screen::AddWatcher => self.draw_add_watcher(f),
            Screen::ListWatchers => self.draw_list_watchers(f),
            Screen::EditWatcher(idx) => self.draw_edit_watcher(f, *idx),
            Screen::ServiceControl => self.draw_service_control(f),
        }
    }

    fn draw_main_menu(&mut self, f: &mut Frame) {
        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Length(3),
                Constraint::Min(0),
                Constraint::Length(3),
            ])
            .split(f.size());

        // Title
        let title = Paragraph::new("Web Watcher Alert")
            .style(Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD))
            .alignment(Alignment::Center)
            .block(Block::default().borders(Borders::ALL));
        f.render_widget(title, chunks[0]);

        // Menu items
        let menu_items = vec![
            ListItem::new("1. Add Watcher"),
            ListItem::new("2. List Watchers"),
            ListItem::new("3. Start Monitoring"),
            ListItem::new("4. Service Control"),
            ListItem::new("5. Exit"),
        ];

        let menu = List::new(menu_items)
            .block(Block::default().title("Main Menu").borders(Borders::ALL))
            .highlight_style(
                Style::default()
                    .bg(Color::DarkGray)
                    .add_modifier(Modifier::BOLD),
            )
            .highlight_symbol(">> ");

        f.render_stateful_widget(menu, chunks[1], &mut self.menu_state);

        // Help text
        let help = Paragraph::new("↑↓: Navigate | Enter: Select | q: Quit")
            .style(Style::default().fg(Color::Gray))
            .alignment(Alignment::Center)
            .block(Block::default().borders(Borders::ALL));
        f.render_widget(help, chunks[2]);
    }

    fn draw_add_watcher(&mut self, f: &mut Frame) {
        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .margin(2)
            .constraints([
                Constraint::Length(3),
                Constraint::Length(3),
                Constraint::Length(3),
                Constraint::Length(3),
                Constraint::Min(0),
                Constraint::Length(3),
            ])
            .split(f.size());

        // Title
        let title = Paragraph::new("Add New Watcher")
            .style(Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD))
            .block(Block::default().borders(Borders::ALL));
        f.render_widget(title, chunks[0]);

        // URL field
        let url_style = if self.form_field == FormField::Url {
            Style::default().fg(Color::Yellow)
        } else {
            Style::default()
        };
        let url = Paragraph::new(self.url_input.as_str())
            .style(url_style)
            .block(Block::default().title("URL").borders(Borders::ALL));
        f.render_widget(url, chunks[1]);

        // Keywords field
        let keywords_style = if self.form_field == FormField::Keywords {
            Style::default().fg(Color::Yellow)
        } else {
            Style::default()
        };
        let keywords = Paragraph::new(self.keywords_input.as_str())
            .style(keywords_style)
            .block(Block::default().title("Keywords (comma-separated)").borders(Borders::ALL));
        f.render_widget(keywords, chunks[2]);

        // Interval field
        let interval_style = if self.form_field == FormField::Interval {
            Style::default().fg(Color::Yellow)
        } else {
            Style::default()
        };
        let interval = Paragraph::new(self.interval_input.as_str())
            .style(interval_style)
            .block(Block::default().title("Check Interval (minutes)").borders(Borders::ALL));
        f.render_widget(interval, chunks[3]);

        // Help
        let help = Paragraph::new("Tab: Next field | Enter: Save | Esc: Cancel")
            .style(Style::default().fg(Color::Gray))
            .alignment(Alignment::Center)
            .block(Block::default().borders(Borders::ALL));
        f.render_widget(help, chunks[5]);
    }

    fn draw_list_watchers(&mut self, f: &mut Frame) {
        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Length(3),
                Constraint::Min(0),
                Constraint::Length(3),
            ])
            .split(f.size());

        // Title
        let title = Paragraph::new(format!("Watchers ({})", self.config.watchers.len()))
            .style(Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD))
            .alignment(Alignment::Center)
            .block(Block::default().borders(Borders::ALL));
        f.render_widget(title, chunks[0]);

        // Watcher list
        if self.config.watchers.is_empty() {
            let empty = Paragraph::new("No watchers configured.\nPress 'a' to add one.")
                .style(Style::default().fg(Color::Gray))
                .alignment(Alignment::Center)
                .block(Block::default().borders(Borders::ALL));
            f.render_widget(empty, chunks[1]);
        } else {
            let items: Vec<ListItem> = self
                .config
                .watchers
                .iter()
                .enumerate()
                .map(|(i, w)| {
                    let status = if w.enabled { "✓" } else { "✗" };
                    let keywords = w.keywords.join(", ");
                    let interval_mins = w.check_interval.as_secs() / 60;
                    let text = format!(
                        "{} [{}] {} | Keywords: {} | Every {} min",
                        status, i + 1, w.url, keywords, interval_mins
                    );
                    ListItem::new(text)
                })
                .collect();

            let list = List::new(items)
                .block(Block::default().borders(Borders::ALL))
                .highlight_style(
                    Style::default()
                        .bg(Color::DarkGray)
                        .add_modifier(Modifier::BOLD),
                )
                .highlight_symbol(">> ");

            f.render_stateful_widget(list, chunks[1], &mut self.watcher_list_state);
        }

        // Help
        let help = Paragraph::new("↑↓: Navigate | t: Toggle | e: Edit | d: Delete | a: Add | Esc: Back")
            .style(Style::default().fg(Color::Gray))
            .alignment(Alignment::Center)
            .block(Block::default().borders(Borders::ALL));
        f.render_widget(help, chunks[2]);
    }

    fn draw_edit_watcher(&mut self, f: &mut Frame, idx: usize) {
        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .margin(2)
            .constraints([
                Constraint::Length(3),
                Constraint::Length(3),
                Constraint::Length(3),
                Constraint::Length(3),
                Constraint::Min(0),
                Constraint::Length(3),
            ])
            .split(f.size());

        // Title
        let title = Paragraph::new(format!("Edit Watcher #{}", idx + 1))
            .style(Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD))
            .block(Block::default().borders(Borders::ALL));
        f.render_widget(title, chunks[0]);

        // URL field
        let url_style = if self.form_field == FormField::Url {
            Style::default().fg(Color::Yellow)
        } else {
            Style::default()
        };
        let url = Paragraph::new(self.url_input.as_str())
            .style(url_style)
            .block(Block::default().title("URL").borders(Borders::ALL));
        f.render_widget(url, chunks[1]);

        // Keywords field
        let keywords_style = if self.form_field == FormField::Keywords {
            Style::default().fg(Color::Yellow)
        } else {
            Style::default()
        };
        let keywords = Paragraph::new(self.keywords_input.as_str())
            .style(keywords_style)
            .block(Block::default().title("Keywords (comma-separated)").borders(Borders::ALL));
        f.render_widget(keywords, chunks[2]);

        // Interval field
        let interval_style = if self.form_field == FormField::Interval {
            Style::default().fg(Color::Yellow)
        } else {
            Style::default()
        };
        let interval = Paragraph::new(self.interval_input.as_str())
            .style(interval_style)
            .block(Block::default().title("Check Interval (minutes)").borders(Borders::ALL));
        f.render_widget(interval, chunks[3]);

        // Help
        let help = Paragraph::new("Tab: Next field | Enter: Save | Esc: Cancel")
            .style(Style::default().fg(Color::Gray))
            .alignment(Alignment::Center)
            .block(Block::default().borders(Borders::ALL));
        f.render_widget(help, chunks[5]);
    }

    fn handle_input(&mut self, key: KeyCode) -> Result<bool> {
        match &self.screen {
            Screen::MainMenu => self.handle_main_menu_input(key),
            Screen::AddWatcher => self.handle_add_watcher_input(key),
            Screen::ListWatchers => self.handle_list_watchers_input(key),
            Screen::EditWatcher(idx) => {
                let idx = *idx; // Copy the index
                self.handle_edit_watcher_input(key, idx)
            }
            Screen::ServiceControl => self.handle_service_control_input(key),
        }
    }

    fn handle_main_menu_input(&mut self, key: KeyCode) -> Result<bool> {
        match key {
            KeyCode::Char('q') | KeyCode::Esc => return Ok(true),
            KeyCode::Down | KeyCode::Char('j') => {
                let i = match self.menu_state.selected() {
                    Some(i) => (i + 1) % 5,
                    None => 0,
                };
                self.menu_state.select(Some(i));
            }
            KeyCode::Up | KeyCode::Char('k') => {
                let i = match self.menu_state.selected() {
                    Some(i) => {
                        if i == 0 {
                            4
                        } else {
                            i - 1
                        }
                    }
                    None => 0,
                };
                self.menu_state.select(Some(i));
            }
            KeyCode::Enter | KeyCode::Char(' ') => {
                match self.menu_state.selected() {
                    Some(0) => self.screen = Screen::AddWatcher,
                    Some(1) => {
                        self.screen = Screen::ListWatchers;
                        if !self.config.watchers.is_empty() {
                            self.watcher_list_state.select(Some(0));
                        }
                    }
                    Some(2) => {
                        // Start monitoring - exit TUI and run monitor
                        return self.start_monitoring();
                    }
                    Some(3) => {
                        // Service Control
                        self.check_service_status();
                        self.screen = Screen::ServiceControl;
                    }
                    Some(4) => return Ok(true),
                    _ => {}
                }
            }
            KeyCode::Char('1') => self.screen = Screen::AddWatcher,
            KeyCode::Char('2') => {
                self.screen = Screen::ListWatchers;
                if !self.config.watchers.is_empty() {
                    self.watcher_list_state.select(Some(0));
                }
            }
            KeyCode::Char('3') => return self.start_monitoring(),
            KeyCode::Char('4') => {
                self.check_service_status();
                self.screen = Screen::ServiceControl;
            }
            KeyCode::Char('5') => return Ok(true),
            _ => {}
        }
        Ok(false)
    }

    fn handle_add_watcher_input(&mut self, key: KeyCode) -> Result<bool> {
        match key {
            KeyCode::Esc => {
                self.screen = Screen::MainMenu;
                self.clear_form();
            }
            KeyCode::Tab => {
                self.form_field = match self.form_field {
                    FormField::Url => FormField::Keywords,
                    FormField::Keywords => FormField::Interval,
                    FormField::Interval => FormField::Url,
                };
            }
            KeyCode::Enter => {
                // Save watcher
                if !self.url_input.is_empty() && !self.keywords_input.is_empty() {
                    let interval_mins: u64 = self.interval_input.parse().unwrap_or(30);
                    let interval = Duration::from_secs(interval_mins * 60);

                    let keywords: Vec<String> = self.keywords_input
                        .split(',')
                        .map(|s| s.trim().to_string())
                        .filter(|s| !s.is_empty())
                        .collect();

                    let watcher = Watcher::new(
                        self.url_input.clone(),
                        keywords,
                        interval,
                    );

                    self.config.watchers.push(watcher);
                    self.config.save()?;

                    self.screen = Screen::MainMenu;
                    self.clear_form();
                }
            }
            KeyCode::Backspace => {
                match self.form_field {
                    FormField::Url => {
                        self.url_input.pop();
                    }
                    FormField::Keywords => {
                        self.keywords_input.pop();
                    }
                    FormField::Interval => {
                        self.interval_input.pop();
                    }
                }
            }
            KeyCode::Char(c) => {
                match self.form_field {
                    FormField::Url => self.url_input.push(c),
                    FormField::Keywords => self.keywords_input.push(c),
                    FormField::Interval => {
                        if c.is_ascii_digit() {
                            self.interval_input.push(c);
                        }
                    }
                }
            }
            _ => {}
        }
        Ok(false)
    }

    fn handle_list_watchers_input(&mut self, key: KeyCode) -> Result<bool> {
        match key {
            KeyCode::Esc => {
                self.screen = Screen::MainMenu;
            }
            KeyCode::Down | KeyCode::Char('j') => {
                if !self.config.watchers.is_empty() {
                    let i = match self.watcher_list_state.selected() {
                        Some(i) => (i + 1) % self.config.watchers.len(),
                        None => 0,
                    };
                    self.watcher_list_state.select(Some(i));
                }
            }
            KeyCode::Up | KeyCode::Char('k') => {
                if !self.config.watchers.is_empty() {
                    let i = match self.watcher_list_state.selected() {
                        Some(i) => {
                            if i == 0 {
                                self.config.watchers.len() - 1
                            } else {
                                i - 1
                            }
                        }
                        None => 0,
                    };
                    self.watcher_list_state.select(Some(i));
                }
            }
            KeyCode::Char('t') => {
                // Toggle enabled/disabled
                if let Some(i) = self.watcher_list_state.selected() {
                    if i < self.config.watchers.len() {
                        self.config.watchers[i].enabled = !self.config.watchers[i].enabled;
                        self.config.save()?;
                    }
                }
            }
            KeyCode::Char('d') => {
                // Delete watcher
                if let Some(i) = self.watcher_list_state.selected() {
                    if i < self.config.watchers.len() {
                        self.config.watchers.remove(i);
                        self.config.save()?;

                        // Adjust selection
                        if self.config.watchers.is_empty() {
                            self.watcher_list_state.select(None);
                        } else if i >= self.config.watchers.len() {
                            self.watcher_list_state.select(Some(self.config.watchers.len() - 1));
                        }
                    }
                }
            }
            KeyCode::Char('a') => {
                self.clear_form();
                self.screen = Screen::AddWatcher;
            }
            KeyCode::Char('e') => {
                // Edit watcher
                if let Some(i) = self.watcher_list_state.selected() {
                    if i < self.config.watchers.len() {
                        self.populate_form_from_watcher(i);
                        self.screen = Screen::EditWatcher(i);
                    }
                }
            }
            _ => {}
        }
        Ok(false)
    }

    fn clear_form(&mut self) {
        self.url_input.clear();
        self.keywords_input.clear();
        self.interval_input = String::from("30");
        self.form_field = FormField::Url;
    }

    fn populate_form_from_watcher(&mut self, index: usize) {
        if let Some(watcher) = self.config.watchers.get(index) {
            self.url_input = watcher.url.clone();
            self.keywords_input = watcher.keywords.join(", ");
            self.interval_input = (watcher.check_interval.as_secs() / 60).to_string();
            self.form_field = FormField::Url;
        }
    }

    fn handle_edit_watcher_input(&mut self, key: KeyCode, index: usize) -> Result<bool> {
        match key {
            KeyCode::Esc => {
                self.screen = Screen::ListWatchers;
                self.clear_form();
            }
            KeyCode::Tab => {
                self.form_field = match self.form_field {
                    FormField::Url => FormField::Keywords,
                    FormField::Keywords => FormField::Interval,
                    FormField::Interval => FormField::Url,
                };
            }
            KeyCode::Enter => {
                // Save edited watcher
                if !self.url_input.is_empty() && !self.keywords_input.is_empty() {
                    let interval_mins: u64 = self.interval_input.parse().unwrap_or(30);
                    let interval = Duration::from_secs(interval_mins * 60);

                    let keywords: Vec<String> = self.keywords_input
                        .split(',')
                        .map(|s| s.trim().to_string())
                        .filter(|s| !s.is_empty())
                        .collect();

                    // Update the existing watcher
                    if let Some(watcher) = self.config.watchers.get_mut(index) {
                        watcher.url = self.url_input.clone();
                        watcher.keywords = keywords;
                        watcher.check_interval = interval;
                    }

                    self.config.save()?;

                    self.screen = Screen::ListWatchers;
                    self.clear_form();
                }
            }
            KeyCode::Backspace => {
                match self.form_field {
                    FormField::Url => {
                        self.url_input.pop();
                    }
                    FormField::Keywords => {
                        self.keywords_input.pop();
                    }
                    FormField::Interval => {
                        self.interval_input.pop();
                    }
                }
            }
            KeyCode::Char(c) => {
                match self.form_field {
                    FormField::Url => self.url_input.push(c),
                    FormField::Keywords => self.keywords_input.push(c),
                    FormField::Interval => {
                        if c.is_ascii_digit() {
                            self.interval_input.push(c);
                        }
                    }
                }
            }
            _ => {}
        }
        Ok(false)
    }

    fn start_monitoring(&mut self) -> Result<bool> {
        // Save any pending changes
        self.config.save()?;

        // Exit TUI and start monitoring in blocking mode
        // This returns true to exit the TUI loop
        disable_raw_mode()?;
        execute!(
            io::stdout(),
            LeaveAlternateScreen,
            DisableMouseCapture
        )?;

        // Create monitor and start
        let monitor = Monitor::new(self.config.clone());

        // Run monitoring in the async runtime
        let runtime = tokio::runtime::Runtime::new()?;
        runtime.block_on(async {
            monitor.start().await
        })?;

        // After monitoring ends, exit the application
        Ok(true)
    }

    fn draw_service_control(&mut self, f: &mut Frame) {
        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .margin(2)
            .constraints([
                Constraint::Length(3),
                Constraint::Length(5),
                Constraint::Min(0),
                Constraint::Length(3),
            ])
            .split(f.size());

        // Title
        let title = Paragraph::new("Background Service Control")
            .style(Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD))
            .block(Block::default().borders(Borders::ALL));
        f.render_widget(title, chunks[0]);

        // Status
        let status_text = if self.service_is_running {
            vec![
                Line::from(vec![
                    Span::styled("Status: ", Style::default()),
                    Span::styled("● Running", Style::default().fg(Color::Green)),
                ]),
                Line::from("The background service is actively monitoring watchers."),
            ]
        } else {
            vec![
                Line::from(vec![
                    Span::styled("Status: ", Style::default()),
                    Span::styled("○ Stopped", Style::default().fg(Color::Red)),
                ]),
                Line::from("The background service is not running."),
            ]
        };

        let status = Paragraph::new(status_text)
            .block(Block::default().title("Current Status").borders(Borders::ALL));
        f.render_widget(status, chunks[1]);

        // Message / Actions
        let message_text = if !self.service_status_message.is_empty() {
            self.service_status_message.clone()
        } else {
            format!(
                "Controls:\n\n\
                s - Start service\n\
                x - Stop service\n\
                r - Refresh status\n\
                Esc - Back to main menu\n\n\
                Note: Service runs independently after starting.\n\
                Close this app and it will keep monitoring!"
            )
        };

        let message = Paragraph::new(message_text)
            .wrap(Wrap { trim: true })
            .block(Block::default().title("Actions & Info").borders(Borders::ALL));
        f.render_widget(message, chunks[2]);

        // Help
        let help = Paragraph::new("s: Start | x: Stop | r: Refresh | Esc: Back")
            .style(Style::default().fg(Color::Gray))
            .alignment(Alignment::Center)
            .block(Block::default().borders(Borders::ALL));
        f.render_widget(help, chunks[3]);
    }

    fn handle_service_control_input(&mut self, key: KeyCode) -> Result<bool> {
        match key {
            KeyCode::Esc => {
                self.service_status_message.clear();
                self.screen = Screen::MainMenu;
            }
            KeyCode::Char('s') => {
                self.start_service();
            }
            KeyCode::Char('x') => {
                self.stop_service();
            }
            KeyCode::Char('r') => {
                self.check_service_status();
                self.service_status_message = String::from("Status refreshed.");
            }
            _ => {}
        }
        Ok(false)
    }

    fn check_service_status(&mut self) {
        use std::process::Command;

        let output = Command::new("launchctl")
            .args(&["list", "com.webwatcheralert"])
            .output();

        match output {
            Ok(result) => {
                if result.status.success() {
                    // Parse output to check if service has a PID
                    // Output format: "PID    Status    Label"
                    // If PID is "-", the service is loaded but not running
                    let output_str = String::from_utf8_lossy(&result.stdout);

                    // Look for the PID in the first column
                    // If it's a number, service is running; if it's "-", it's not
                    self.service_is_running = output_str
                        .lines()
                        .any(|line| {
                            let parts: Vec<&str> = line.split_whitespace().collect();
                            if let Some(first) = parts.first() {
                                // Check if first column is a number (PID) rather than "-"
                                first.parse::<i32>().is_ok()
                            } else {
                                false
                            }
                        });
                } else {
                    // Service not even loaded
                    self.service_is_running = false;
                }
            }
            Err(_) => {
                self.service_is_running = false;
            }
        }
    }

    fn start_service(&mut self) {
        use std::process::Command;
        use std::path::Path;

        self.service_status_message.clear();

        // Check if service is installed first
        let plist_path = dirs::home_dir()
            .map(|h| h.join("Library/LaunchAgents/com.webwatcheralert.plist"));

        if let Some(path) = plist_path {
            if !Path::new(&path).exists() {
                self.service_status_message = String::from(
                    "Service not installed!\n\n\
                    Run this command first:\n\
                    ./scripts/install-service.sh\n\n\
                    Then return to this screen and press 'r' to refresh."
                );
                return;
            }
        }

        // First check if already running
        self.check_service_status();
        if self.service_is_running {
            self.service_status_message = String::from("Service is already running.");
            return;
        }

        let output = Command::new("launchctl")
            .args(&["start", "com.webwatcheralert"])
            .output();

        match output {
            Ok(result) => {
                if result.status.success() {
                    // Wait a moment for service to start
                    std::thread::sleep(std::time::Duration::from_millis(500));
                    self.check_service_status();

                    if self.service_is_running {
                        self.service_status_message = String::from(
                            "✓ Service started successfully!\n\n\
                            The background monitor is now running.\n\
                            You can close this app and monitoring will continue.\n\n\
                            Logs: ~/.local/share/web-watcher-alert/logs/"
                        );
                    } else {
                        // Check logs for more info
                        let log_path = dirs::home_dir()
                            .map(|h| h.join(".local/share/web-watcher-alert/logs/stderr.log"));

                        let log_hint = if let Some(path) = log_path {
                            format!("\n\nCheck logs for details:\n{}", path.display())
                        } else {
                            String::new()
                        };

                        self.service_status_message = format!(
                            "Failed to start service.\n\n\
                            The service is installed but didn't start properly.{}\n\n\
                            Make sure:\n\
                            - Binary is built: cargo build --release\n\
                            - At least one watcher is configured",
                            log_hint
                        );
                    }
                } else {
                    let stdout = String::from_utf8_lossy(&result.stdout);
                    let stderr = String::from_utf8_lossy(&result.stderr);
                    let error = if !stderr.is_empty() {
                        stderr.to_string()
                    } else if !stdout.is_empty() {
                        stdout.to_string()
                    } else {
                        "Unknown error".to_string()
                    };

                    self.service_status_message = format!(
                        "Failed to start service.\n\nError: {}",
                        error
                    );
                }
            }
            Err(e) => {
                self.service_status_message = format!(
                    "Failed to execute launchctl.\n\nError: {}\n\n\
                    Make sure the service is installed:\n\
                    ./scripts/install-service.sh",
                    e
                );
            }
        }
    }

    fn stop_service(&mut self) {
        use std::process::Command;

        self.service_status_message.clear();

        // First check if running
        self.check_service_status();
        if !self.service_is_running {
            self.service_status_message = String::from("Service is not running.");
            return;
        }

        // Use kill with SIGTERM instead of stop (works better for non-KeepAlive services)
        // Get the UID for the target format: gui/<uid>/<service-name>
        let uid_output = Command::new("id")
            .arg("-u")
            .output();

        let uid = match uid_output {
            Ok(output) => String::from_utf8_lossy(&output.stdout).trim().to_string(),
            Err(_) => {
                self.service_status_message = String::from("Failed to get user ID.");
                return;
            }
        };

        let target = format!("gui/{}/com.webwatcheralert", uid);
        let output = Command::new("launchctl")
            .args(&["kill", "SIGTERM", &target])
            .output();

        match output {
            Ok(result) => {
                if result.status.success() {
                    // Wait a moment for service to stop
                    std::thread::sleep(std::time::Duration::from_millis(500));
                    self.check_service_status();

                    if !self.service_is_running {
                        self.service_status_message = String::from(
                            "✓ Service stopped successfully.\n\n\
                            Background monitoring has been stopped."
                        );
                    } else {
                        self.service_status_message = String::from(
                            "Service may still be running.\n\
                            Try running: ./scripts/service.sh stop"
                        );
                    }
                } else {
                    let error = String::from_utf8_lossy(&result.stderr);
                    self.service_status_message = format!(
                        "Failed to stop service.\n\nError: {}",
                        error
                    );
                }
            }
            Err(e) => {
                self.service_status_message = format!(
                    "Failed to execute launchctl.\n\nError: {}",
                    e
                );
            }
        }
    }
}
