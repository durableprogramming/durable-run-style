//! # Durable Run TUI Multi
//!
//! A terminal user interface for running multiple commands concurrently in tabs.
//! This crate provides a multi-tabbed TUI that can execute commands in parallel,
//! track their progress, and display results with analytics.
//!
//! ## Features
//!
//! - **Multi-tab Interface**: Run multiple commands in separate tabs
//! - **Concurrent Execution**: Execute commands in parallel using durable-run-parallel
//! - **Progress Tracking**: Visual progress indicators for each command
//! - **Analytics Integration**: Track execution metrics using durable-run-analytics
//! - **Theme Support**: Consistent theming with the druns ecosystem
//! - **Template Support**: Dynamic command generation using durable-run-template
//! - **Async Updates**: Real-time updates of command output and status
//!
//! ## Example
//!
//! ```rust,no_run
//! use durable_run_tui_multi::TuiApp;
//!
//! #[tokio::main]
//! async fn main() -> Result<(), Box<dyn std::error::Error>> {
//!     let mut app = TuiApp::new();
//!
//!     // Add some commands to execute
//!     app.add_command_tab("tab1".to_string(), "List files".to_string(), "ls -la".to_string());
//!     app.add_command_tab("tab2".to_string(), "Show processes".to_string(), "ps aux".to_string());
//!
//!     // Add a template-based command
//!     app.add_template_tab(
//!         "tab3".to_string(),
//!         "Greet user".to_string(),
//!         "echo 'Hello {0}!'".to_string(),
//!         vec!["World".to_string()]
//!     );
//!
//!     // Run the TUI (this will block until user quits)
//!     app.run().await?;
//!
//!     Ok(())
//! }
//! ```
//!
//! ## Key Bindings
//!
//! - `Tab` / `Right` / `Left`: Switch between tabs
//! - `BackTab`: Switch to previous tab
//! - `1-9`: Jump to specific tab (1-based indexing)
//! - `r`: Reset and re-run all commands
//! - `q` / `Esc`: Quit the application
//! - `Ctrl+C`: Force quit (terminal interrupt)
//!
//! ## Configuration
//!
//! The TUI can be configured using `BatchConfig` for job limits, timeouts, and resource controls:
//!
//! ```rust,no_run
//! use durable_run_tui_multi::TuiApp;
//! use durable_run_config_batch::BatchConfig;
//!
//! #[tokio::main]
//! async fn main() -> Result<(), Box<dyn std::error::Error>> {
//!     let config = BatchConfig::new("my_batch".to_string());
//!     let mut app = TuiApp::with_config(config);
//!
//!     // Add tabs...
//!
//!     app.run().await?;
//!     Ok(())
//! }
//! ```
//!
//! ## Template Usage
//!
//! Use templates for dynamic command generation:
//!
//! ```rust,no_run
//! use durable_run_tui_multi::TuiApp;
//!
//! #[tokio::main]
//! async fn main() -> Result<(), Box<dyn std::error::Error>> {
//!     let mut app = TuiApp::new();
//!
//!     // Template with positional arguments
//!     app.add_template_tab(
//!         "greet".to_string(),
//!         "Greet User".to_string(),
//!         "echo 'Hello {0}, welcome to {1}!'".to_string(),
//!         vec!["Alice".to_string(), "Rust".to_string()]
//!     );
//!
//!     // Template with variables
//!     app.add_template_tab(
//!         "build".to_string(),
//!         "Build Project".to_string(),
//!         "cd {project_dir} && cargo build --{build_type}".to_string(),
//!         vec!["/path/to/project".to_string(), "release".to_string()]
//!     );
//!
//!     app.run().await?;
//!     Ok(())
//! }
//! ```

use ratatui::{
    layout::{Constraint, Direction, Layout},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Clear, Gauge, Paragraph, Tabs, Wrap},
    Terminal,
};
use crossterm::{
    event::{self, DisableMouseCapture, EnableMouseCapture, Event, KeyCode},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use std::io;
use std::sync::{Arc, Mutex};
use tokio::sync::mpsc::{self, UnboundedReceiver, UnboundedSender};
use thiserror::Error;
use ratatui::backend::CrosstermBackend;
use durable_run_parallel::ConcurrentExecutor;
use durable_run_analytics::{AnalyticsDashboard, DataPoint};
use durable_run_config_batch::BatchConfig;
use durable_run_template::{TemplateEngine, Context};
use chrono::Utc;
use std::collections::HashMap;

#[derive(Debug, Clone)]
pub struct Theme {
    pub primary: Color,
    pub secondary: Color,
    pub accent: Color,
    pub border: Color,
    pub text: Color,
    pub background: Color,
    pub shades: Vec<Color>,
}

impl Default for Theme {
    fn default() -> Self {
        Self {
            primary: Color::Rgb(59, 130, 246), // blue-500
            secondary: Color::Rgb(30, 64, 175), // darker blue
            accent: Color::Rgb(96, 165, 250), // lighter blue
            border: Color::Rgb(17, 24, 39), // very dark
            text: Color::Rgb(255, 255, 255), // white
            background: Color::Rgb(0, 0, 0), // black
            shades: vec![
                Color::Rgb(120, 180, 255),
                Color::Rgb(100, 160, 255),
                Color::Rgb(80, 140, 255),
                Color::Rgb(60, 120, 255),
                Color::Rgb(40, 100, 255),
                Color::Rgb(20, 80, 255),
                Color::Rgb(59, 130, 246),
                Color::Rgb(30, 64, 175),
            ],
        }
    }
}

#[derive(Debug)]
pub enum UpdateMessage {
    UpdateTabContent { tab_id: String, content: String },
    UpdateTabProgress { tab_id: String, progress: f32 },
    UpdateTabState { tab_id: String, state: TabState },
    AddTab(Tab),
    RemoveTab(String),
}

#[derive(Debug, Error)]
pub enum TuiError {
    #[error("IO error: {0}")]
    Io(#[from] io::Error),
    #[error("Terminal error: {0}")]
    Terminal(String),
    #[error("Config error: {0}")]
    Config(#[from] durable_run_config_batch::ConfigError),
}

pub type Result<T> = std::result::Result<T, TuiError>;

#[derive(Debug, Clone)]
pub enum TabState {
    Pending,
    Running,
    Completed(String),
    Failed(String),
}

#[derive(Debug, Clone)]
pub struct Tab {
    pub id: String,
    pub title: String,
    pub command: String,
    pub template: Option<String>,
    pub inputs: Vec<String>,
    pub content: String,
    pub progress: f32, // 0.0 to 1.0
    pub state: TabState,
}

impl Tab {
    pub fn new(id: String, title: String, command: String) -> Self {
        Self {
            id,
            title,
            command,
            template: None,
            inputs: Vec::new(),
            content: String::new(),
            progress: 0.0,
            state: TabState::Pending,
        }
    }

    pub fn with_template(id: String, title: String, template: String, inputs: Vec<String>) -> Self {
        Self {
            id,
            title,
            command: String::new(), // Will be generated from template
            template: Some(template),
            inputs,
            content: String::new(),
            progress: 0.0,
            state: TabState::Pending,
        }
    }

    pub fn update_content(&mut self, content: String) {
        self.content = content;
    }

    pub fn set_progress(&mut self, progress: f32) {
        self.progress = progress.clamp(0.0, 1.0);
    }

    pub fn set_state(&mut self, state: TabState) {
        self.state = state;
    }

    pub fn is_running(&self) -> bool {
        matches!(self.state, TabState::Running)
    }

    pub fn is_completed(&self) -> bool {
        matches!(self.state, TabState::Completed(_))
    }

    pub fn is_failed(&self) -> bool {
        matches!(self.state, TabState::Failed(_))
    }
}

pub struct TabManager {
    tabs: Vec<Tab>,
    active_tab: usize,
}

impl TabManager {
    pub fn new() -> Self {
        Self {
            tabs: vec![],
            active_tab: 0,
        }
    }

    pub fn add_tab(&mut self, tab: Tab) {
        self.tabs.push(tab);
    }

    pub fn remove_tab(&mut self, id: &str) {
        self.tabs.retain(|t| t.id != id);
        if self.active_tab >= self.tabs.len() && self.active_tab > 0 {
            self.active_tab -= 1;
        }
    }

    pub fn switch_tab(&mut self, index: usize) {
        if index < self.tabs.len() {
            self.active_tab = index;
        }
    }

    pub fn active_tab(&self) -> Option<&Tab> {
        self.tabs.get(self.active_tab)
    }

    pub fn active_tab_mut(&mut self) -> Option<&mut Tab> {
        self.tabs.get_mut(self.active_tab)
    }

    pub fn tabs(&self) -> &[Tab] {
        &self.tabs
    }

    pub fn tabs_mut(&mut self) -> &mut [Tab] {
        &mut self.tabs
    }
}

pub struct ProgressTracker {
    progresses: Arc<Mutex<std::collections::HashMap<String, f32>>>,
}

impl ProgressTracker {
    pub fn new() -> Self {
        Self {
            progresses: Arc::new(Mutex::new(std::collections::HashMap::new())),
        }
    }

    pub fn update_progress(&self, process_id: String, progress: f32) {
        let mut progresses = self.progresses.lock().unwrap();
        progresses.insert(process_id, progress.clamp(0.0, 1.0));
    }

    pub fn get_progress(&self, process_id: &str) -> f32 {
        let progresses = self.progresses.lock().unwrap();
        *progresses.get(process_id).unwrap_or(&0.0)
    }

    pub fn all_progresses(&self) -> std::collections::HashMap<String, f32> {
        self.progresses.lock().unwrap().clone()
    }
}

pub struct TuiApp {
    tab_manager: TabManager,
    progress_tracker: ProgressTracker,
    theme: Theme,
    should_quit: bool,
    update_rx: UnboundedReceiver<UpdateMessage>,
    _update_tx: UnboundedSender<UpdateMessage>, // Keep a copy for the app
    executor: ConcurrentExecutor,
    analytics: AnalyticsDashboard,
    config: BatchConfig,
    template_engine: TemplateEngine,
}

impl TuiApp {
    pub fn new() -> Self {
        Self::with_config(BatchConfig::default())
    }

    pub fn with_config(config: BatchConfig) -> Self {
        let (update_tx, update_rx) = mpsc::unbounded_channel();
        Self {
            tab_manager: TabManager::new(),
            progress_tracker: ProgressTracker::new(),
            theme: Theme::default(),
            should_quit: false,
            update_rx,
            _update_tx: update_tx,
            executor: ConcurrentExecutor::new(config.job_limits.max_concurrent),
            analytics: AnalyticsDashboard::new(),
            config,
            template_engine: TemplateEngine::new(),
        }
    }

    pub fn load_config_from_file<P: AsRef<std::path::Path>>(path: P) -> Result<Self> {
        let config = BatchConfig::load_from_file(path)?;
        Ok(Self::with_config(config))
    }

    pub fn update_sender(&self) -> UnboundedSender<UpdateMessage> {
        self._update_tx.clone()
    }

    pub fn with_theme(mut self, theme: Theme) -> Self {
        self.theme = theme;
        self
    }

    pub fn tab_manager(&mut self) -> &mut TabManager {
        &mut self.tab_manager
    }

    pub fn progress_tracker(&self) -> &ProgressTracker {
        &self.progress_tracker
    }

    pub fn analytics(&self) -> &AnalyticsDashboard {
        &self.analytics
    }

    pub fn quit(&mut self) {
        self.should_quit = true;
    }

    pub fn should_quit(&self) -> bool {
        self.should_quit
    }

    pub fn record_command_execution(&mut self, tab_id: &str, command: &str, success: bool, execution_time_ms: f64) {
        // Record execution time
        let time_point = DataPoint {
            timestamp: Utc::now(),
            metric: "command_execution_time".to_string(),
            value: execution_time_ms,
            tags: {
                let mut tags = HashMap::new();
                tags.insert("tab_id".to_string(), tab_id.to_string());
                tags.insert("command".to_string(), command.to_string());
                tags.insert("success".to_string(), success.to_string());
                tags
            },
        };
        self.analytics.collection().record_data_point(time_point);

        // Record command length
        let length_point = DataPoint {
            timestamp: Utc::now(),
            metric: "command_length".to_string(),
            value: command.len() as f64,
            tags: {
                let mut tags = HashMap::new();
                tags.insert("tab_id".to_string(), tab_id.to_string());
                tags
            },
        };
        self.analytics.collection().record_data_point(length_point);
    }

    pub fn add_command_tab(&mut self, id: String, title: String, command: String) {
        let tab = Tab::new(id.clone(), title, command);
        self.tab_manager.add_tab(tab);
    }

    pub fn add_template_tab(&mut self, id: String, title: String, template: String, inputs: Vec<String>) {
        let tab = Tab::with_template(id.clone(), title, template, inputs);
        self.tab_manager.add_tab(tab);
    }

    pub async fn execute_tab_commands(&mut self) -> Result<()> {
        use tokio::task;
        use std::sync::Arc;

        // Collect commands from pending tabs
        let mut commands: Vec<(String, String)> = Vec::new();
        for tab in self.tab_manager.tabs().iter().filter(|tab| matches!(tab.state, TabState::Pending)) {
            let command = if let Some(template) = &tab.template {
                // Generate command from template
                let mut context = Context::new().with_args(tab.inputs.clone());
                match self.template_engine.render(template, &mut context) {
                    Ok(cmd) => cmd,
                    Err(e) => {
                        // If template rendering fails, use error as command
                        format!("echo 'Template error: {}'", e)
                    }
                }
            } else {
                tab.command.clone()
            };
            commands.push((tab.id.clone(), command));
        }

        if commands.is_empty() {
            return Ok(());
        }

        // Mark tabs as running
        for (tab_id, _) in &commands {
            if let Some(tab) = self.tab_manager.tabs_mut().iter_mut().find(|t| t.id == *tab_id) {
                tab.set_state(TabState::Running);
            }
        }

        // Execute commands concurrently with streaming output
        let command_strings: Vec<String> = commands.iter().map(|(_, cmd)| cmd.clone()).collect();
        let update_tx = self._update_tx.clone();
        let state_update_tx = self._update_tx.clone();
        let tab_ids: Vec<String> = commands.iter().map(|(id, _)| id.clone()).collect();
        let tab_ids_for_completion = tab_ids.clone();

        // Create callback for streaming output
        let output_callback: durable_run_parallel::OutputCallback = Arc::new(move |index: usize, output: String| {
            if let Some(tab_id) = tab_ids.get(index) {
                let _ = update_tx.send(UpdateMessage::UpdateTabContent {
                    tab_id: tab_id.clone(),
                    content: output,
                });
            }
        });

        let executor = self.executor.clone();
        task::spawn(async move {
            let _start_time = std::time::Instant::now();
            let results = executor.execute_batch_streaming(command_strings, output_callback).await;

            // Send completion state for each command
            for (index, result) in results.iter().enumerate() {
                if let Some(tab_id) = tab_ids_for_completion.get(index) {
                    let state = match result {
                        Ok(output) => TabState::Completed(output.clone()),
                        Err(e) => TabState::Failed(format!("{}", e)),
                    };
                    let _ = state_update_tx.send(UpdateMessage::UpdateTabState {
                        tab_id: tab_id.clone(),
                        state,
                    });
                }
            }
        });

        Ok(())
    }

    pub async fn reset_tabs_to_pending(&mut self) -> Result<()> {
        for tab in self.tab_manager.tabs_mut() {
            if !matches!(tab.state, TabState::Pending) {
                tab.set_state(TabState::Pending);
                tab.content.clear();
                tab.progress = 0.0;
            }
        }
        Ok(())
    }

    pub async fn run(mut self) -> Result<()> {
        enable_raw_mode()?;
        let mut stdout = io::stdout();
        execute!(stdout, EnterAlternateScreen, EnableMouseCapture)?;
        let backend = CrosstermBackend::new(stdout);
        let mut terminal = Terminal::new(backend)?;

        // Execute initial commands
        self.execute_tab_commands().await?;

        loop {
            if self.should_quit {
                break;
            }

            // Handle async updates
            while let Ok(message) = self.update_rx.try_recv() {
                self.handle_update(message);
            }

            // Draw the UI
            terminal.draw(|f| {
                self.draw_ui(f);
            })?;

            // Handle events
            if event::poll(std::time::Duration::from_millis(100))? {
                if let Event::Key(key) = event::read()? {
                    match key.code {
                        KeyCode::Char('q') | KeyCode::Esc => self.quit(),
                        KeyCode::Char('r') => {
                            // Re-run commands
                            self.reset_tabs_to_pending().await?;
                            self.execute_tab_commands().await?;
                        }
                        KeyCode::Tab => {
                            let next = (self.tab_manager.active_tab + 1) % self.tab_manager.tabs().len().max(1);
                            self.tab_manager.switch_tab(next);
                        }
                        KeyCode::BackTab => {
                            let prev = if self.tab_manager.active_tab == 0 {
                                self.tab_manager.tabs().len().saturating_sub(1)
                            } else {
                                self.tab_manager.active_tab - 1
                            };
                            self.tab_manager.switch_tab(prev);
                        }
                        KeyCode::Right => {
                            let next = (self.tab_manager.active_tab + 1) % self.tab_manager.tabs().len().max(1);
                            self.tab_manager.switch_tab(next);
                        }
                        KeyCode::Left => {
                            let prev = if self.tab_manager.active_tab == 0 {
                                self.tab_manager.tabs().len().saturating_sub(1)
                            } else {
                                self.tab_manager.active_tab - 1
                            };
                            self.tab_manager.switch_tab(prev);
                        }
                        KeyCode::Char(c) if c.is_ascii_digit() => {
                            if let Some(digit) = c.to_digit(10) {
                                let index = (digit as usize).saturating_sub(1); // 1-based to 0-based
                                if index < self.tab_manager.tabs().len() {
                                    self.tab_manager.switch_tab(index);
                                }
                            }
                        }
                        _ => {}
                    }
                }
            }
        }

        disable_raw_mode()?;
        execute!(
            terminal.backend_mut(),
            LeaveAlternateScreen,
            DisableMouseCapture
        )?;
        terminal.show_cursor()?;

        Ok(())
    }

    fn draw_ui(&self, f: &mut ratatui::Frame) {
        let size = f.size();

        // Split the layout: tabs at top, content below
        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Length(3), // Tab bar height
                Constraint::Min(0),    // Content area
            ])
            .split(size);

        // Create tab titles - only show tabs that have started (Running, Completed, Failed)
        let started_tabs: Vec<(usize, &Tab)> = self.tab_manager.tabs()
            .iter()
            .enumerate()
            .filter(|(_, tab)| !matches!(tab.state, TabState::Pending))
            .collect();

        let tab_titles: Vec<Line> = started_tabs
            .iter()
            .enumerate()
            .map(|(_display_idx, (original_idx, tab))| {
                let progress = self.progress_tracker.get_progress(&tab.id);
                let state_indicator = match &tab.state {
                    TabState::Pending => "⏳",
                    TabState::Running => "▶️",
                    TabState::Completed(_) => "✅",
                    TabState::Failed(_) => "❌",
                };
                let title = if progress > 0.0 {
                    format!("{} {} ({:.0}%)", state_indicator, tab.title, progress * 100.0)
                } else {
                    format!("{} {}", state_indicator, tab.title)
                };

                // Color based on state
                let color = match &tab.state {
                    TabState::Pending => self.theme.text,
                    TabState::Running => self.theme.primary,
                    TabState::Completed(_) => Color::Green,
                    TabState::Failed(_) => Color::Red,
                };

                Line::from(Span::styled(
                    title,
                    if *original_idx == self.tab_manager.active_tab {
                        Style::default()
                            .fg(color)
                            .add_modifier(Modifier::BOLD)
                    } else {
                        Style::default().fg(color)
                    },
                ))
            })
            .collect();

        // Create tabs widget
        let tabs = Tabs::new(tab_titles)
            .block(
                Block::default()
                    .borders(Borders::ALL)
                    .border_style(Style::default().fg(self.theme.border))
                    .title("Tabs")
                    .title_style(
                        Style::default()
                            .fg(self.theme.primary)
                            .add_modifier(Modifier::BOLD),
                    ),
            )
            .select(self.tab_manager.active_tab)
            .style(Style::default().fg(self.theme.text))
            .highlight_style(
                Style::default()
                    .fg(self.theme.accent)
                    .add_modifier(Modifier::BOLD),
            );

        f.render_widget(tabs, chunks[0]);

        // Draw active tab content
        if let Some(tab) = self.tab_manager.active_tab() {
            let progress = self.progress_tracker.get_progress(&tab.id);

            // Split content area: progress bar at bottom, content above
            let content_chunks = Layout::default()
                .direction(Direction::Vertical)
                .constraints([
                    Constraint::Min(0),    // Content
                    Constraint::Length(3), // Progress bar
                ])
                .split(chunks[1]);

            // Content area
            let status_info = match &tab.state {
                TabState::Pending => format!("Command: {}\nStatus: Pending", tab.command),
                TabState::Running => format!("Command: {}\nStatus: Running...", tab.command),
                TabState::Completed(_) => format!("Command: {}\nStatus: Completed", tab.command),
                TabState::Failed(err) => format!("Command: {}\nStatus: Failed - {}", tab.command, err),
            };

            let full_content = if tab.content.is_empty() {
                status_info
            } else {
                format!("{}\n\n--- Output ---\n{}", status_info, tab.content)
            };

            let content_block = Block::default()
                .borders(Borders::ALL)
                .border_style(Style::default().fg(self.theme.border))
                .title(format!("{} - {}", tab.title, tab.id))
                .title_style(
                    Style::default()
                        .fg(self.theme.primary)
                        .add_modifier(Modifier::BOLD),
                );

            let content_paragraph = Paragraph::new(full_content)
                .block(content_block)
                .wrap(Wrap { trim: true })
                .style(Style::default().fg(self.theme.text));

            // Clear the area first to prevent artifacts from previous tab
            f.render_widget(Clear, content_chunks[0]);
            f.render_widget(content_paragraph, content_chunks[0]);

            // Progress bar
            let progress_block = Block::default()
                .borders(Borders::ALL)
                .border_style(Style::default().fg(self.theme.border))
                .title("Progress")
                .title_style(
                    Style::default()
                        .fg(self.theme.secondary)
                        .add_modifier(Modifier::BOLD),
                );

            let gauge = Gauge::default()
                .block(progress_block)
                .gauge_style(Style::default().fg(self.theme.accent))
                .percent((progress * 100.0) as u16)
                .label(format!("{:.1}%", progress * 100.0));

            f.render_widget(gauge, content_chunks[1]);
        }
    }

    fn handle_update(&mut self, message: UpdateMessage) {
        match message {
            UpdateMessage::UpdateTabContent { tab_id, content } => {
                // Just update content without changing state - streaming output updates
                if let Some(tab) = self.tab_manager.tabs_mut().iter_mut().find(|t| t.id == tab_id) {
                    tab.update_content(content);
                }
            }
            UpdateMessage::UpdateTabProgress { tab_id, progress } => {
                self.progress_tracker.update_progress(tab_id, progress);
            }
            UpdateMessage::UpdateTabState { tab_id, state } => {
                let success = matches!(state, TabState::Completed(_));
                let should_record = success || matches!(state, TabState::Failed(_));

                // Get command before borrowing self again
                let command = self.tab_manager.tabs().iter()
                    .find(|t| t.id == tab_id)
                    .map(|t| t.command.clone());

                if let Some(command) = command {
                    // Record analytics when tab completes
                    if should_record {
                        self.record_command_execution(&tab_id, &command, success, 0.0); // TODO: track actual timing
                    }
                }

                if let Some(tab) = self.tab_manager.tabs_mut().iter_mut().find(|t| t.id == tab_id) {
                    tab.set_state(state);
                }
            }
            UpdateMessage::AddTab(tab) => {
                self.tab_manager.add_tab(tab);
            }
            UpdateMessage::RemoveTab(tab_id) => {
                self.tab_manager.remove_tab(&tab_id);
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_tab_creation() {
        let tab = Tab::new("test_id".to_string(), "Test Tab".to_string(), "echo test".to_string());
        assert_eq!(tab.id, "test_id");
        assert_eq!(tab.title, "Test Tab");
        assert_eq!(tab.command, "echo test");
        assert_eq!(tab.content, "");
        assert_eq!(tab.progress, 0.0);
        assert!(matches!(tab.state, TabState::Pending));
    }

    #[test]
    fn test_tab_update_content() {
        let mut tab = Tab::new("test_id".to_string(), "Test Tab".to_string(), "echo test".to_string());
        tab.update_content("New content".to_string());
        assert_eq!(tab.content, "New content");
    }

    #[test]
    fn test_tab_set_progress() {
        let mut tab = Tab::new("test_id".to_string(), "Test Tab".to_string(), "echo test".to_string());
        tab.set_progress(0.75);
        assert_eq!(tab.progress, 0.75);

        // Test clamping
        tab.set_progress(1.5);
        assert_eq!(tab.progress, 1.0);

        tab.set_progress(-0.5);
        assert_eq!(tab.progress, 0.0);
    }

    #[test]
    fn test_tab_manager_new() {
        let manager = TabManager::new();
        assert_eq!(manager.tabs.len(), 0);
        assert_eq!(manager.active_tab, 0);
    }

    #[test]
    fn test_tab_manager_add_tab() {
        let mut manager = TabManager::new();
        let tab = Tab::new("tab1".to_string(), "Tab 1".to_string(), "echo hello".to_string());
        manager.add_tab(tab);

        assert_eq!(manager.tabs.len(), 1);
        assert_eq!(manager.tabs[0].id, "tab1");
        assert_eq!(manager.tabs[0].title, "Tab 1");
        assert_eq!(manager.tabs[0].command, "echo hello");
    }

    #[test]
    fn test_tab_manager_remove_tab() {
        let mut manager = TabManager::new();
        manager.add_tab(Tab::new("tab1".to_string(), "Tab 1".to_string(), "echo 1".to_string()));
        manager.add_tab(Tab::new("tab2".to_string(), "Tab 2".to_string(), "echo 2".to_string()));
        manager.add_tab(Tab::new("tab3".to_string(), "Tab 3".to_string(), "echo 3".to_string()));

        // Remove middle tab
        manager.remove_tab("tab2");
        assert_eq!(manager.tabs.len(), 2);
        assert_eq!(manager.tabs[0].id, "tab1");
        assert_eq!(manager.tabs[1].id, "tab3");
        assert_eq!(manager.active_tab, 0);

        // Remove first tab, should adjust active_tab
        manager.switch_tab(0);
        manager.remove_tab("tab1");
        assert_eq!(manager.tabs.len(), 1);
        assert_eq!(manager.tabs[0].id, "tab3");
        assert_eq!(manager.active_tab, 0);
    }

    #[test]
    fn test_tab_manager_switch_tab() {
        let mut manager = TabManager::new();
        manager.add_tab(Tab::new("tab1".to_string(), "Tab 1".to_string(), "echo 1".to_string()));
        manager.add_tab(Tab::new("tab2".to_string(), "Tab 2".to_string(), "echo 2".to_string()));
        manager.add_tab(Tab::new("tab3".to_string(), "Tab 3".to_string(), "echo 3".to_string()));

        manager.switch_tab(1);
        assert_eq!(manager.active_tab, 1);

        // Test out of bounds
        manager.switch_tab(10);
        assert_eq!(manager.active_tab, 1); // Should not change

        manager.switch_tab(2);
        assert_eq!(manager.active_tab, 2);
    }

    #[test]
    fn test_tab_manager_active_tab() {
        let mut manager = TabManager::new();
        assert!(manager.active_tab().is_none());

        manager.add_tab(Tab::new("tab1".to_string(), "Tab 1".to_string(), "echo 1".to_string()));
        assert_eq!(manager.active_tab().unwrap().id, "tab1");

        manager.add_tab(Tab::new("tab2".to_string(), "Tab 2".to_string(), "echo 2".to_string()));
        manager.switch_tab(1);
        assert_eq!(manager.active_tab().unwrap().id, "tab2");
    }

    #[test]
    fn test_progress_tracker_new() {
        let tracker = ProgressTracker::new();
        assert_eq!(tracker.get_progress("nonexistent"), 0.0);
    }

    #[test]
    fn test_progress_tracker_update_and_get() {
        let tracker = ProgressTracker::new();

        tracker.update_progress("task1".to_string(), 0.5);
        assert_eq!(tracker.get_progress("task1"), 0.5);

        // Test clamping
        tracker.update_progress("task1".to_string(), 1.2);
        assert_eq!(tracker.get_progress("task1"), 1.0);

        tracker.update_progress("task1".to_string(), -0.1);
        assert_eq!(tracker.get_progress("task1"), 0.0);
    }

    #[test]
    fn test_progress_tracker_all_progresses() {
        let tracker = ProgressTracker::new();

        tracker.update_progress("task1".to_string(), 0.3);
        tracker.update_progress("task2".to_string(), 0.7);

        let all = tracker.all_progresses();
        assert_eq!(all.len(), 2);
        assert_eq!(all["task1"], 0.3);
        assert_eq!(all["task2"], 0.7);
    }

    #[test]
    fn test_tui_app_new() {
        let app = TuiApp::new();
        assert_eq!(app.tab_manager.tabs().len(), 0);
        assert!(!app.should_quit);
    }

    #[test]
    fn test_tui_app_with_theme() {
        let theme = Theme::default();
        let app = TuiApp::new().with_theme(theme.clone());
        // Test that theme is set correctly
        // Theme fields are Color enums, so we just check they exist
        let _ = app.theme.primary;
        let _ = app.theme.secondary;
    }

    #[test]
    fn test_tui_app_tab_manager() {
        let mut app = TuiApp::new();
        let tab_manager = app.tab_manager();
        assert_eq!(tab_manager.tabs().len(), 0);
    }

    #[test]
    fn test_tui_app_progress_tracker() {
        let app = TuiApp::new();
        let tracker = app.progress_tracker();
        assert_eq!(tracker.get_progress("test"), 0.0);
    }

    #[test]
    fn test_tui_app_quit() {
        let mut app = TuiApp::new();
        assert!(!app.should_quit());
        app.quit();
        assert!(app.should_quit());
    }

    #[test]
    fn test_update_message_handling() {
        let mut app = TuiApp::new();

        // Add a tab first
        let tab = Tab::new("test_tab".to_string(), "Test Tab".to_string(), "echo test".to_string());
        app.tab_manager.add_tab(tab);

        // Test update content
        app.handle_update(UpdateMessage::UpdateTabContent {
            tab_id: "test_tab".to_string(),
            content: "Updated content".to_string(),
        });
        assert_eq!(app.tab_manager.tabs()[0].content, "Updated content");

        // Test update progress
        app.handle_update(UpdateMessage::UpdateTabProgress {
            tab_id: "test_tab".to_string(),
            progress: 0.8,
        });
        assert_eq!(app.progress_tracker.get_progress("test_tab"), 0.8);

        // Test add tab
        app.handle_update(UpdateMessage::AddTab(Tab::new(
            "new_tab".to_string(),
            "New Tab".to_string(),
            "echo new".to_string(),
        )));
        assert_eq!(app.tab_manager.tabs().len(), 2);

        // Test remove tab
        app.handle_update(UpdateMessage::RemoveTab("new_tab".to_string()));
        assert_eq!(app.tab_manager.tabs().len(), 1);
    }

    #[test]
    fn test_theme_default() {
        let theme = Theme::default();
        // Test that default theme has expected structure
        // Theme fields are Color enums, so we just check they can be accessed
        let _ = theme.primary;
        let _ = theme.secondary;
        let _ = theme.accent;
        let _ = theme.border;
        let _ = theme.text;
        let _ = theme.background;
    }

    #[tokio::test]
    async fn test_add_command_tab() {
        let mut app = TuiApp::new();
        app.add_command_tab("test_tab".to_string(), "Test Tab".to_string(), "echo hello".to_string());

        assert_eq!(app.tab_manager.tabs().len(), 1);
        let tab = &app.tab_manager.tabs()[0];
        assert_eq!(tab.id, "test_tab");
        assert_eq!(tab.title, "Test Tab");
        assert_eq!(tab.command, "echo hello");
        assert!(matches!(tab.state, TabState::Pending));
    }

    #[tokio::test]
    async fn test_add_template_tab() {
        let mut app = TuiApp::new();
        app.add_template_tab(
            "template_tab".to_string(),
            "Template Tab".to_string(),
            "echo 'Hello {0}!'".to_string(),
            vec!["World".to_string()]
        );

        assert_eq!(app.tab_manager.tabs().len(), 1);
        let tab = &app.tab_manager.tabs()[0];
        assert_eq!(tab.id, "template_tab");
        assert_eq!(tab.title, "Template Tab");
        assert_eq!(tab.template.as_ref().unwrap(), "echo 'Hello {0}!'");
        assert_eq!(tab.inputs, vec!["World"]);
        assert!(matches!(tab.state, TabState::Pending));
    }

    #[tokio::test]
    async fn test_execute_tab_commands() {
        let mut app = TuiApp::new();
        app.add_command_tab("test_tab".to_string(), "Test Tab".to_string(), "echo hello".to_string());

        // Execute commands
        app.execute_tab_commands().await.unwrap();

        // Wait for execution to complete
        tokio::time::sleep(tokio::time::Duration::from_millis(200)).await;

        // Process any pending updates
        while let Ok(message) = app.update_rx.try_recv() {
            app.handle_update(message);
        }

        // Check final state
        let tab = &app.tab_manager.tabs()[0];
        assert!(matches!(tab.state, TabState::Completed(_)));
        if let TabState::Completed(content) = &tab.state {
            assert!(content.contains("hello"));
        }
    }

    #[tokio::test]
    async fn test_reset_tabs_to_pending() {
        let mut app = TuiApp::new();
        app.add_command_tab("test_tab".to_string(), "Test Tab".to_string(), "echo hello".to_string());

        // Manually set tab to completed state
        {
            let tab = &mut app.tab_manager.tabs_mut()[0];
            tab.set_state(TabState::Completed("output".to_string()));
            tab.update_content("output".to_string());
        }

        // Reset tabs
        app.reset_tabs_to_pending().await.unwrap();

        // Check that tab is back to pending
        let tab = &app.tab_manager.tabs()[0];
        assert!(matches!(tab.state, TabState::Pending));
        assert_eq!(tab.content, "");
        assert_eq!(tab.progress, 0.0);
    }
}