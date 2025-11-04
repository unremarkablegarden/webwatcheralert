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
    widgets::{Block, Borders, List, ListItem, ListState, Paragraph},
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
    #[allow(dead_code)]
    EditWatcher(usize), // Index of watcher being edited (for future use)
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
            ListItem::new("4. Exit"),
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
        let help = Paragraph::new("↑↓: Navigate | t: Toggle | d: Delete | Esc: Back")
            .style(Style::default().fg(Color::Gray))
            .alignment(Alignment::Center)
            .block(Block::default().borders(Borders::ALL));
        f.render_widget(help, chunks[2]);
    }

    fn draw_edit_watcher(&mut self, f: &mut Frame, _idx: usize) {
        // Similar to add_watcher but with existing values
        // For now, just redirect to add_watcher UI
        self.draw_add_watcher(f);
    }

    fn handle_input(&mut self, key: KeyCode) -> Result<bool> {
        match &self.screen {
            Screen::MainMenu => self.handle_main_menu_input(key),
            Screen::AddWatcher => self.handle_add_watcher_input(key),
            Screen::ListWatchers => self.handle_list_watchers_input(key),
            Screen::EditWatcher(_) => self.handle_add_watcher_input(key),
        }
    }

    fn handle_main_menu_input(&mut self, key: KeyCode) -> Result<bool> {
        match key {
            KeyCode::Char('q') | KeyCode::Esc => return Ok(true),
            KeyCode::Down | KeyCode::Char('j') => {
                let i = match self.menu_state.selected() {
                    Some(i) => (i + 1) % 4,
                    None => 0,
                };
                self.menu_state.select(Some(i));
            }
            KeyCode::Up | KeyCode::Char('k') => {
                let i = match self.menu_state.selected() {
                    Some(i) => {
                        if i == 0 {
                            3
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
                    Some(3) => return Ok(true),
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
            KeyCode::Char('4') => return Ok(true),
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
                self.screen = Screen::AddWatcher;
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
}
