use crate::cli::Args;
use crate::config::Config;
use crate::ui::{draw_gfx_demo_ui, GfxDemoDrawContext};
use crossterm::{
    event::{self, Event, KeyCode, KeyModifiers},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use rand::Rng;
use ratatui::{backend::CrosstermBackend, Terminal};
use std::io::{stdout, IsTerminal};
use tokio::signal;
use tokio::time::{sleep, Duration, Instant};

fn generate_status_update() -> String {
    let systems = [
        "Warp Core", "Photon Torpedoes", "Dilithium Crystals", "Plasma Conduits",
        "Quantum Flux Capacitor", "Subspace Communications", "Antimatter Containment",
        "Transporter Buffer", "Deflector Shields", "Inertial Dampeners",
        "Navigational Deflector", "Life Support Systems", "Tachyon Pulse Emitters",
        "Gravimetric Sensors", "Chroniton Detectors", "Neutrino Shielding",
        "Hyperdrive Engines", "Cloaking Device", "Tractor Beams", "Sensor Arrays",
    ];
    let actions = [
        "stabilized", "optimized", "calibrated", "recharged", "synchronized",
        "harmonized", "polarized", "modulated", "amplified", "degaussed",
        "phased", "aligned", "balanced", "compensated", "normalized",
    ];
    let statuses = [
        "nominal", "optimal", "critical", "fluctuating", "stable",
        "unstable", "overloaded", "underpowered", "resonating", "dampened",
    ];

    let mut rng = rand::thread_rng();
    let system = systems[rng.gen_range(0..systems.len())];
    let action = actions[rng.gen_range(0..actions.len())];
    let status = statuses[rng.gen_range(0..statuses.len())];
    let value = rng.gen_range(0..100);

    format!("[{:.2}s] {} {} - {} at {}%", 
        rand::random::<f64>() * 100.0, 
        system, 
        action, 
        status, 
        value)
}


pub struct GfxDemoApp {
    config: Config,
    terminal: Option<Terminal<CrosstermBackend<std::io::Stdout>>>,
    animation_frame: u32,
    start_time: Instant,
    // Demo state
    iteration_count: u64,
    display_states: Vec<(f64, f64)>, // (value, velocity) for each technobabble item
    status_updates: Vec<String>, // Scrolling list of status updates
    is_tty: bool,
}

impl GfxDemoApp {
    pub async fn new(args: Args) -> Result<Self, Box<dyn std::error::Error>> {
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

        // Override config with command line arguments
        if args.no_animate {
            config.app.animation.animation_enabled = false;
        } else {
            config.app.animation.animation_enabled = args.animation;
        }

        let is_tty = std::io::stdout().is_terminal();

        // Setup terminal only if TTY
        let terminal = if is_tty {
            let mut stdout = stdout();
            enable_raw_mode()?;
            execute!(stdout, EnterAlternateScreen)?;
            let backend = CrosstermBackend::new(stdout);
            Some(Terminal::new(backend)?)
        } else {
            None
        };

        let animation_frame = 0;
        let start_time = Instant::now();
        let iteration_count = 0;
        let mut rng = rand::thread_rng();
        let display_states = (0..20).map(|_| (rng.gen::<f64>() * 100.0, 0.0)).collect();
        let status_updates = Vec::new();

        Ok(GfxDemoApp {
            config,
            terminal,
            animation_frame,
            start_time,
            iteration_count,
            display_states,
            status_updates,
            is_tty,
        })
    }

    pub async fn run(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        loop {
            tokio::select! {
                _ = signal::ctrl_c() => {
                    break;
                }
                _ = sleep(Duration::from_millis(20)) => {
                    // Check for key events only if TTY
                    if self.is_tty && event::poll(Duration::from_millis(10))? {
                        if let Event::Key(key) = event::read()? {
                            if key.code == KeyCode::Char('c') && key.modifiers.contains(KeyModifiers::CONTROL) {
                                break;
                            }
                            if key.code == KeyCode::Char('q') {
                                break;
                            }
                        }
                    }

                    // Update demo state every 50ms
                    self.iteration_count += 1;

                    // Add new status update every 10 iterations (500ms)
                    if self.iteration_count % 10 == 0 {
                        let update = generate_status_update();
                        self.status_updates.push(update);
                        // Keep only last 50 updates for scrolling
                        if self.status_updates.len() > 50 {
                            self.status_updates.remove(0);
                        }
                    }

                    // Update display states with chaotic velocity
                    let mut rng = rand::thread_rng();
                    for (value, velocity) in &mut self.display_states {
                        // Add random acceleration (-1 to 1)
                        let acceleration = rng.gen::<f64>() * 2.0 - 1.0;
                        *velocity += acceleration * 0.1; // Scale acceleration
                        *velocity *= 0.98; // Damping to slow down over time
                        *value += *velocity;
                        // Clamp value between 0 and 100
                        *value = value.clamp(0.0, 100.0);
                        // Bounce if hitting bounds
                        if *value == 0.0 || *value == 100.0 {
                            *velocity = -*velocity * 0.5; // Reverse and dampen
                        }
                    }

                    if self.is_tty {
                        // Draw UI
                         let current_animation_frame = if self.config.app.animation.animation_enabled { self.animation_frame } else { 0 };
                         let theme = self.config.theme.clone();
                          self.terminal.as_mut().unwrap().draw(|f| {
                              draw_gfx_demo_ui(f, GfxDemoDrawContext {
                                  animation_frame: current_animation_frame,
                                  theme: &theme,
                                  crunch_result: 0.0, // Not used anymore
                                  memory_allocated: 0, // Not used anymore
                                  memory_fluctuation: &[], // Not used anymore
                                  cpu_load: &[], // Not used anymore
                                  elapsed: self.start_time.elapsed(),
                                  display_states: &self.display_states,
                                  status_updates: &self.status_updates,
                              });
                          })?;

                        // Update animation
                        self.animation_frame = (self.animation_frame + 1) % 110;
                    } else {
                        // Print text output every second
                        if self.iteration_count % 50 == 0 {
                            println!("GFX Demo - Elapsed: {:.2}s, States: {:.1}, {:.1}, {:.1}, ...",
                                self.start_time.elapsed().as_secs_f64(),
                                self.display_states[0].0,
                                self.display_states[1].0,
                                self.display_states[2].0);
                        }
                    }
                }
            }
        }

        Ok(())
    }



    pub async fn cleanup(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        // Cleanup
        if self.is_tty {
            disable_raw_mode()?;
            execute!(self.terminal.as_mut().unwrap().backend_mut(), LeaveAlternateScreen)?;
            self.terminal.as_mut().unwrap().show_cursor()?;
        }

        println!("Starship systems monitor completed!");
        println!("Mission duration: {:.2} seconds", self.start_time.elapsed().as_secs_f64());

        Ok(())
    }
}

impl Drop for GfxDemoApp {
    fn drop(&mut self) {
        // Ensure cleanup happens even if cleanup() isn't called
        if self.is_tty {
            let _ = disable_raw_mode();
            let _ = execute!(std::io::stdout(), LeaveAlternateScreen);
            if let Some(ref mut terminal) = self.terminal {
                let _ = terminal.show_cursor();
            }
        }
    }
}
