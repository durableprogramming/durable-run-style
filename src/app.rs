use crate::cli::Args;
use crate::config::Config;
use crate::disk::monitor_disk;
#[cfg(feature = "experimental-pcap")]
use crate::network::monitor_network;
use crate::process::ProcessManager;
use crate::ui::{draw_ui, DrawContext};
use crossterm::{
    event::{self, Event, KeyCode, KeyModifiers},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use ratatui::{backend::CrosstermBackend, Terminal};
use std::fs::OpenOptions;
use std::io::stdout;
use std::sync::Arc;
use sysinfo::{Pid, System};
use tokio::signal;
use tokio::sync::mpsc;
use tokio::time::{sleep, Duration, Instant};

pub struct App {
    args: Args,
    config: Config,
    process_manager: ProcessManager,
    log_file: Option<Arc<std::sync::Mutex<std::fs::File>>>,
    output_lines: Vec<String>,
    output_rx: mpsc::UnboundedReceiver<String>,
    terminal: Terminal<CrosstermBackend<std::io::Stdout>>,
    current_width: u16,
    current_height: u16,
    animation_frame: u32,
    system: System,
    network_stats_rx: Option<mpsc::UnboundedReceiver<(u64, u64)>>,
    disk_stats_rx: mpsc::UnboundedReceiver<(u64, u64)>,
    start_time: Instant,
    // History for averaging
    cpu_history: Vec<f32>,
    memory_history: Vec<u64>,
    disk_read_history: Vec<u64>,
    disk_write_history: Vec<u64>,
    // Display values (averaged)
    display_cpu: f32,
    display_memory: u64,
    display_disk_read: u64,
    display_disk_write: u64,
    iteration_count: u32,
    command: Vec<String>,
    pwd: String,
    // Output scrolling state
    follow_mode: bool,
    scroll_offset: usize,
}

impl App {
    pub async fn new(args: Args, command: Vec<String>) -> Result<Self, Box<dyn std::error::Error>> {
        // Load configuration
        let mut config = if let Some(config_path) = &args.config {
            Config::load_from_file(config_path)?
        } else {
            Config::with_defaults()
        };

        // Handle no_animate from config
        if config.app.animation.no_animate {
            config.app.animation.animation_enabled = false;
        }

        // Calculate min sidebar width based on header
        let header_content = include_str!("../ascii_art.txt");
        let header_lines: Vec<&str> = header_content.lines().filter(|line| !line.trim().is_empty()).collect();
        let max_len = header_lines.iter().map(|l| l.chars().count()).max().unwrap_or(0);
        let min_content_width = max_len + 2;
        let min_sidebar_width = min_content_width + 2;
        let effective_sidebar_width = args.sidebar_width.max(min_sidebar_width as u16);

        // Override config with command line arguments
        config.app.layout.sidebar_width = effective_sidebar_width;
        config.app.layout.max_command_lines = args.max_command_lines;
        config.app.output.max_output_lines = args.max_output_lines;
        if args.no_animate {
            config.app.animation.animation_enabled = false;
        } else {
            config.app.animation.animation_enabled = args.animation;
        }

        // Open log file if specified
        let log_file = if let Some(log_path) = &args.log {
            Some(Arc::new(std::sync::Mutex::new(OpenOptions::new().create(true).append(true).open(log_path)?)))
        } else {
            None
        };

        // Get terminal size
        let (width, height) = crossterm::terminal::size()?;

        // Get current working directory
        let pwd = std::env::current_dir()?.to_string_lossy().to_string();
        let sidebar_width = config.app.layout.sidebar_width;
        let app_width = width.saturating_sub(sidebar_width);

        // Override COLUMNS
        std::env::set_var("COLUMNS", app_width.to_string());

        // Create process manager
        let mut process_manager = ProcessManager::new(&command)?;

        // Start output reading
        let output_rx = process_manager.start_output_reading(log_file.clone());

        // Channel for network stats
        #[cfg(feature = "experimental-pcap")]
        let network_stats_rx = {
            let (tx, rx) = mpsc::unbounded_channel::<(u64, u64)>();
            let pid = process_manager.pid;
            tokio::spawn(async move {
                monitor_network(pid, tx).await;
            });
            Some(rx)
        };
        #[cfg(not(feature = "experimental-pcap"))]
        let network_stats_rx = None;

        // Channel for disk stats
        let (disk_stats_tx, disk_stats_rx) = mpsc::unbounded_channel::<(u64, u64)>();
        let pid = process_manager.pid;
        tokio::spawn(async move {
            monitor_disk(pid, disk_stats_tx).await;
        });

        // Setup terminal
        let mut stdout = stdout();
        enable_raw_mode()?;
        execute!(stdout, EnterAlternateScreen)?;
        let backend = CrosstermBackend::new(stdout);
        let terminal = Terminal::new(backend)?;

        let current_width = width;
        let current_height = height;
        let animation_frame = 0;
        let system = System::new_all();
        let start_time = Instant::now();
        let output_lines = Vec::new();
        let cpu_history = Vec::new();
        let memory_history = Vec::new();
        let disk_read_history = Vec::new();
        let disk_write_history = Vec::new();
        let display_cpu = 0.0;
        let display_memory = 0;
        let display_disk_read = 0;
        let display_disk_write = 0;
        let iteration_count = 0;
        let follow_mode = true; // Start in follow mode
        let scroll_offset = 0;

        Ok(App {
            args,
            config,
            process_manager,
            log_file,
            output_lines,
            output_rx,
            terminal,
            current_width,
            current_height,
            animation_frame,
            system,
            network_stats_rx,
            disk_stats_rx,
            start_time,
            cpu_history,
            memory_history,
            disk_read_history,
            disk_write_history,
            display_cpu,
            display_memory,
            display_disk_read,
            display_disk_write,
            iteration_count,
            command,
            pwd,
            follow_mode,
            scroll_offset,
        })
    }

    pub async fn run(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        loop {
            tokio::select! {
                _ = signal::ctrl_c() => {
                    let _ = self.process_manager.kill().await;
                    break;
                }
                Some(line) = self.output_rx.recv() => {
                    self.output_lines.push(line);
                    if self.output_lines.len() > self.config.app.output.max_output_lines {
                        self.output_lines.remove(0);
                    }
                }
                _ = sleep(Duration::from_millis(20)) => {
                    // Check if child is still running
                    if let Ok(Some(_)) = self.process_manager.try_wait() {
                        break;
                    }

                    // Check for resize and key events
                    if event::poll(Duration::from_millis(10))? {
                        match event::read()? {
                            Event::Resize(new_width, new_height) => {
                                self.current_width = new_width;
                                self.current_height = new_height;
                                self.terminal.resize(ratatui::layout::Rect::new(0, 0, self.current_width, new_height))?;
                            }
                             Event::Key(key) => {
                                 let page_height = (self.current_height.saturating_sub(4) as usize).min(20);
                                 let total_lines = self.output_lines.len();
                                 let visible_height = page_height;
                                 let max_scroll_up = total_lines.saturating_sub(visible_height);
                                 match key.code {
                                     KeyCode::Char('c') if key.modifiers.contains(KeyModifiers::CONTROL) => {
                                         let _ = self.process_manager.kill().await;
                                         break;
                                     }
                                     KeyCode::Char('f') => {
                                         // Toggle follow mode
                                         self.follow_mode = !self.follow_mode;
                                         if self.follow_mode {
                                             self.scroll_offset = 0;
                                         }
                                     }
                                     KeyCode::Up => {
                                         // Scroll up by one line
                                         if self.follow_mode {
                                             self.follow_mode = false;
                                         }
                                         self.scroll_offset += 1;
                                     }
                                     KeyCode::Down => {
                                         // Scroll down by one line
                                         self.scroll_offset = self.scroll_offset.saturating_sub(1);
                                         if self.scroll_offset == 0 {
                                             self.follow_mode = true;
                                         }
                                     }
                                     KeyCode::PageUp => {
                                         // Scroll up by page height
                                         if self.follow_mode {
                                             self.follow_mode = false;
                                         }
                                         self.scroll_offset += page_height;
                                     }
                                     KeyCode::PageDown => {
                                         // Scroll down by page height
                                         self.scroll_offset = self.scroll_offset.saturating_sub(page_height);
                                         if self.scroll_offset == 0 {
                                             self.follow_mode = true;
                                         }
                                     }
                                     KeyCode::Home => {
                                         // Scroll to top
                                         self.follow_mode = false;
                                         self.scroll_offset = max_scroll_up;
                                     }
                                     KeyCode::End => {
                                         // Scroll to bottom and enable follow mode
                                         self.follow_mode = true;
                                         self.scroll_offset = 0;
                                     }
                                     KeyCode::Esc => {
                                         // Return to end of log and enable follow mode
                                         self.follow_mode = true;
                                         self.scroll_offset = 0;
                                     }
                                     _ => {}
                                 }
                             }
                            _ => {}
                        }
                    }

                    // Update COLUMNS
                    let app_width = self.current_width.saturating_sub(self.config.app.layout.sidebar_width);
                    std::env::set_var("COLUMNS", app_width.to_string());

                    // Refresh system info
                    self.system.refresh_all();

                    // Calculate stats
                    let cpu_percent = if let Some(process) = self.system.process(Pid::from(self.process_manager.pid as usize)) {
                        process.cpu_usage()
                    } else {
                        0.0
                    };
                    let memory_used = if let Some(process) = self.system.process(Pid::from(self.process_manager.pid as usize)) {
                        process.memory()
                    } else {
                        0
                    };
                    let memory_total = self.system.total_memory();

                    // Get latest network stats
                    let (child_network_rx, child_network_tx) = {
                        #[cfg(feature = "experimental-pcap")]
                        { self.network_stats_rx.as_mut().unwrap().try_recv().unwrap_or((0, 0)) }
                        #[cfg(not(feature = "experimental-pcap"))]
                        { (0, 0) }
                    };

                    // Get latest disk stats
                    let mut disk_read = 0u64;
                    let mut disk_write = 0u64;
                    if let Ok((r, w)) = self.disk_stats_rx.try_recv() {
                        disk_read = r;
                        disk_write = w;
                    }

                    // Push to history every second (50 iterations)
                    if self.iteration_count % 50 == 0 {
                        self.cpu_history.push(cpu_percent);
                        self.memory_history.push(memory_used);
                        self.disk_read_history.push(disk_read);
                        self.disk_write_history.push(disk_write);

                        // Limit history to last 5 minutes (300 samples at 1s intervals)
                        const MAX_HISTORY: usize = 300;
                        if self.cpu_history.len() > MAX_HISTORY {
                            self.cpu_history.remove(0);
                            self.memory_history.remove(0);
                            self.disk_read_history.remove(0);
                            self.disk_write_history.remove(0);
                        }
                    }

                    // Update display values every second (50 iterations)
                    self.iteration_count += 1;
                    if self.iteration_count % 50 == 0 {
                        self.display_cpu = if !self.cpu_history.is_empty() {
                            self.cpu_history.iter().sum::<f32>() / self.cpu_history.len() as f32
                        } else {
                            0.0
                        };
                        self.display_memory = if !self.memory_history.is_empty() {
                            (self.memory_history.iter().sum::<u64>() as f64 / self.memory_history.len() as f64).round() as u64
                        } else {
                            0
                        };
                        self.display_disk_read = if !self.disk_read_history.is_empty() {
                            (self.disk_read_history.iter().sum::<u64>() as f64 / self.disk_read_history.len() as f64).round() as u64
                        } else {
                            0
                        };
                        self.display_disk_write = if !self.disk_write_history.is_empty() {
                            (self.disk_write_history.iter().sum::<u64>() as f64 / self.disk_write_history.len() as f64).round() as u64
                        } else {
                            0
                        };
                    }

                    // Draw UI
                    let current_animation_frame = if self.config.app.animation.animation_enabled { self.animation_frame } else { 0 };
                    let theme = self.config.theme.clone();
                    self.terminal.draw(|f| {
                        draw_ui(f, DrawContext {
                            command: &self.command,
                            pwd: &self.pwd,
                            elapsed: self.start_time.elapsed(),
                            pid: self.process_manager.pid,
                            ppid: self.process_manager.ppid,
                            animation_frame: current_animation_frame,
                            theme: &theme,
                            output_lines: &self.output_lines,
                            sidebar_width: self.config.app.layout.sidebar_width,
                            max_command_lines: self.config.app.max_command_lines(),
                            cpu_percent: self.display_cpu,
                            memory_used: self.display_memory,
                            memory_total,
                            disk_read: self.display_disk_read,
                            disk_write: self.display_disk_write,
                            child_network_rx,
                            child_network_tx,
                            cpu_history: &self.cpu_history,
                            memory_history: &self.memory_history,
                            disk_read_history: &self.disk_read_history,
                            disk_write_history: &self.disk_write_history,
                            follow_mode: self.follow_mode,
                            scroll_offset: self.scroll_offset,
                        });
                    })?;

                    // Update animation
                    self.animation_frame = (self.animation_frame + 1) % 110;
                }
            }
        }

        Ok(())
    }

    pub async fn cleanup(&mut self) -> Result<std::process::ExitStatus, Box<dyn std::error::Error>> {
        // Cleanup
        disable_raw_mode()?;
        execute!(self.terminal.backend_mut(), LeaveAlternateScreen)?;
        self.terminal.show_cursor()?;
        let status = self.process_manager.wait().await?;

        // Print last few lines of output
        let last_lines = self.output_lines.iter().rev().take(10).collect::<Vec<_>>().into_iter().rev();
        for line in last_lines {
            println!("{}", line);
        }
        println!("Thank you for using druns!");

        Ok(status)
    }
}
