use clap::{Parser, Subcommand, ValueEnum};

/// Output buffering mode for xargs
#[derive(Debug, Clone, Copy, PartialEq, ValueEnum)]
pub enum BufferMode {
    /// Line buffered - output each line as it's received
    Line,
    /// Unbuffered - output each character/byte immediately
    None,
    /// Fully buffered - collect all output then print at the end
    Full,
}

#[derive(Parser, Clone)]
#[command(author, version, about, long_about = None)]
pub struct Args {

    /// Log STDOUT and STDERR to a file while displaying on screen
    #[arg(long)]
    pub log: Option<String>,

    /// Configuration file path
    #[arg(long)]
    pub config: Option<String>,

    /// Sidebar width
    #[arg(long, default_value = "30")]
    pub sidebar_width: u16,

    /// Maximum number of output lines to keep in memory
    #[arg(long, default_value = "1000")]
    pub max_output_lines: usize,

    /// Maximum number of lines to display for the command
    #[arg(long, default_value = "3")]
    pub max_command_lines: usize,

    /// Enable/disable animation
    #[arg(long, default_value = "true")]
    pub animation: bool,

    /// Disable animation (equivalent to --animation false)
    #[arg(long)]
    pub no_animate: bool,

    #[command(subcommand)]
    pub subcommand: Subcommands,
}

#[derive(Debug, Subcommand, Clone)]
#[derive(PartialEq)]
pub enum AnalyticsSubcommands {
    Collect,
    Show,
    Stats,
    Chart,
    Report,
    Dashboard,
}

#[derive(Debug, Subcommand, Clone)]
#[derive(PartialEq)]
pub enum BatchSubcommands {
    Create {
        /// Name of the batch configuration
        name: String,
        /// Path to save the configuration file (optional, defaults to batch.yaml)
        #[arg(short, long)]
        file: Option<String>,
    },
    Show {
        /// Path to the batch configuration file (optional, defaults to batch.yaml)
        #[arg(short, long)]
        file: Option<String>,
    },
    Validate {
        /// Path to the batch configuration file (optional, defaults to batch.yaml)
        #[arg(short, long)]
        file: Option<String>,
    },
    SetLimits {
        /// Path to the batch configuration file (optional, defaults to batch.yaml)
        #[arg(short, long)]
        file: Option<String>,
        /// Maximum number of jobs
        #[arg(long)]
        max_jobs: Option<usize>,
        /// Maximum concurrent jobs
        #[arg(long)]
        max_concurrent: Option<usize>,
        /// Maximum retries per job
        #[arg(long)]
        max_retries: Option<usize>,
    },
    SetTimeouts {
        /// Path to the batch configuration file (optional, defaults to batch.yaml)
        #[arg(short, long)]
        file: Option<String>,
        /// Job timeout in seconds
        #[arg(long)]
        job_timeout: Option<u64>,
        /// Batch timeout in seconds
        #[arg(long)]
        batch_timeout: Option<u64>,
        /// Connection timeout in seconds
        #[arg(long)]
        connection_timeout: Option<u64>,
    },
    SetResources {
        /// Path to the batch configuration file (optional, defaults to batch.yaml)
        #[arg(short, long)]
        file: Option<String>,
        /// Maximum memory in MB
        #[arg(long)]
        max_memory_mb: Option<usize>,
        /// Maximum CPU percentage (0-100)
        #[arg(long)]
        max_cpu_percent: Option<f32>,
        /// Maximum disk space in MB
        #[arg(long)]
        max_disk_mb: Option<usize>,
        /// Process priority (-20 to 19)
        #[arg(long)]
        priority: Option<i32>,
    },
}

#[derive(Debug, Subcommand, Clone)]
#[derive(PartialEq)]
pub enum ParallelSubcommands {
    Run,
    FromInputs,
    Queue,
}

#[derive(Debug, Subcommand, Clone)]
#[derive(PartialEq)]
pub enum TemplateSubcommands {
    Render,
    Parse,
    Macros,
}

#[derive(Debug, Subcommand, Clone)]
#[derive(PartialEq)]
pub enum WorkflowSubcommands {
    Run,
    Validate,
    List,
}

#[derive(Debug, Subcommand, Clone)]
#[derive(PartialEq)]
pub enum DiskSubcommands {
    Monitor,
    Stats,
}

#[derive(Debug, Subcommand, Clone)]
#[derive(PartialEq)]
pub enum ConfigSubcommands {
    Show,
    Validate,
    Create,
}

#[derive(Debug, Subcommand, Clone)]
#[derive(PartialEq)]
pub enum ThemeSubcommands {
    Show,
    Create,
    Validate,
}

#[derive(Debug, Subcommand, Clone)]
#[derive(PartialEq)]
pub enum MultiSubcommands {
    Run {
        /// Commands to run in tabs (can be specified multiple times)
        #[arg(short, long)]
        command: Vec<String>,
        /// File containing commands to run (one per line)
        #[arg(short, long)]
        file: Option<String>,
    },
}

#[derive(Debug, Subcommand, Clone)]
#[derive(PartialEq)]
pub enum Subcommands {
    Run {
        #[arg(trailing_var_arg = true, allow_hyphen_values = true)]
        command: Vec<String>,
    },
    GfxDemo,
    Multi {
        #[command(subcommand)]
        multi: MultiSubcommands,
    },
    Xargs {
        /// Command template with {} as placeholder for input (e.g., "echo {}")
        #[arg(required = true, trailing_var_arg = true, allow_hyphen_values = true)]
        command: Vec<String>,

        /// Maximum number of jobs to run in parallel
        #[arg(short = 'P', long)]
        max_procs: Option<usize>,

        /// Use at most max-args arguments per command line
        #[arg(short = 'n', long)]
        max_args: Option<usize>,

        /// Read items from file instead of stdin
        #[arg(short = 'a', long)]
        arg_file: Option<String>,

        /// Input items are terminated by character, not newline
        #[arg(short = 'd', long)]
        delimiter: Option<char>,

        /// Input items are separated by null character
        #[arg(short = '0', long)]
        null: bool,

        /// Replace occurrences of replace-str in command with input
        #[arg(short = 'I', long, default_value = "{}")]
        replace_str: String,

        /// Run without TUI, just pass through command output
        #[arg(long)]
        no_tui: bool,

        /// Output buffering mode: 'line' (line buffered), 'none' (unbuffered), 'full' (fully buffered)
        #[arg(long, value_name = "MODE", default_value = "line")]
        buffer: BufferMode,

        /// Redirect stdin for executed commands from this file
        #[arg(long)]
        stdin_file: Option<String>,

        /// Redirect stdin for executed commands from /dev/null
        #[arg(long)]
        stdin_null: bool,
    },
    Analytics {
        #[command(subcommand)]
        analytics: AnalyticsSubcommands,
    },
    Batch {
        #[command(subcommand)]
        batch: BatchSubcommands,
    },
    Parallel {
        #[command(subcommand)]
        parallel: ParallelSubcommands,
    },
    Template {
        #[command(subcommand)]
        template: TemplateSubcommands,
    },
    Workflow {
        #[command(subcommand)]
        workflow: WorkflowSubcommands,
    },
    Disk {
        #[command(subcommand)]
        disk: DiskSubcommands,
    },
    Config {
        #[command(subcommand)]
        config: ConfigSubcommands,
    },
    Theme {
        #[command(subcommand)]
        theme: ThemeSubcommands,
    },
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_args() {
        let args = Args::parse_from(["test", "run", "echo", "hello"]);
        assert_eq!(args.sidebar_width, 30);
        assert_eq!(args.max_output_lines, 1000);
        assert_eq!(args.max_command_lines, 3);
        assert!(args.animation);
        assert!(!args.no_animate);
        assert!(args.log.is_none());
        assert!(args.config.is_none());
        if let Subcommands::Run { command } = args.subcommand {
            assert_eq!(command, vec!["echo", "hello"]);
        } else {
            panic!("Expected Run subcommand");
        }
    }

    #[test]
    fn test_command_parsing() {
        let args = Args::parse_from(["test", "run", "ls", "-la"]);
        if let Subcommands::Run { command } = args.subcommand {
            assert_eq!(command, vec!["ls", "-la"]);
        } else {
            panic!("Expected Run subcommand");
        }
    }

    #[test]
    fn test_sidebar_width() {
        let args = Args::parse_from(["test", "--sidebar-width", "50", "run", "ls"]);
        assert_eq!(args.sidebar_width, 50);
    }

    #[test]
    fn test_max_output_lines() {
        let args = Args::parse_from(["test", "--max-output-lines", "500", "run", "ls"]);
        assert_eq!(args.max_output_lines, 500);
    }

    #[test]
    fn test_max_command_lines() {
        let args = Args::parse_from(["test", "--max-command-lines", "5", "run", "ls"]);
        assert_eq!(args.max_command_lines, 5);
    }

    #[test]
    fn test_animation_flags() {
        // Test --no-animate flag (sets no_animate to true, but doesn't change animation)
        let args = Args::parse_from(["test", "--no-animate", "run", "ls"]);
        assert!(args.animation); // default is true
        assert!(args.no_animate);

        // Test default animation = true
        let args = Args::parse_from(["test", "run", "ls"]);
        assert!(args.animation);
        assert!(!args.no_animate);
    }

    #[test]
    fn test_log_option() {
        let args = Args::parse_from(["test", "--log", "output.log", "run", "ls"]);
        assert_eq!(args.log, Some("output.log".to_string()));
    }

    #[test]
    fn test_config_option() {
        let args = Args::parse_from(["test", "--config", "config.toml", "run", "ls"]);
        assert_eq!(args.config, Some("config.toml".to_string()));
    }

    #[test]
    fn test_combined_options() {
        let args = Args::parse_from([
            "test",
            "--config",
            "myconfig.toml",
            "--sidebar-width",
            "40",
            "--max-output-lines",
            "2000",
            "--log",
            "debug.log",
            "--no-animate",
            "run",
            "echo",
            "hello",
        ]);
        assert_eq!(args.config, Some("myconfig.toml".to_string()));
        assert_eq!(args.sidebar_width, 40);
        assert_eq!(args.max_output_lines, 2000);
        assert_eq!(args.log, Some("debug.log".to_string()));
        assert!(args.animation); // --animation defaults to true
        assert!(args.no_animate); // --no-animate sets this flag
        if let Subcommands::Run { command } = args.subcommand {
            assert_eq!(command, vec!["echo", "hello"]);
        } else {
            panic!("Expected Run subcommand");
        }
    }

    #[test]
    fn test_hyphen_values_in_command() {
        let args = Args::parse_from(["test", "run", "grep", "-r", "--include=*.rs", "pattern"]);
        if let Subcommands::Run { command } = args.subcommand {
            assert_eq!(
                command,
                vec!["grep", "-r", "--include=*.rs", "pattern"]
            );
        } else {
            panic!("Expected Run subcommand");
        }
    }

    #[test]
    fn test_gfx_demo_subcommand() {
        let args = Args::parse_from(["test", "gfx-demo"]);
        assert!(matches!(args.subcommand, Subcommands::GfxDemo));
    }

    #[test]
    fn test_analytics_subcommand() {
        let args = Args::parse_from(["test", "analytics", "collect"]);
        if let Subcommands::Analytics { analytics } = args.subcommand {
            assert!(matches!(analytics, AnalyticsSubcommands::Collect));
        } else {
            panic!("Expected Analytics subcommand");
        }
    }

    #[test]
    fn test_batch_subcommand() {
        let args = Args::parse_from(["test", "batch", "create", "mybatch"]);
        if let Subcommands::Batch { batch } = args.subcommand {
            assert!(matches!(batch, BatchSubcommands::Create { .. }));
        } else {
            panic!("Expected Batch subcommand");
        }
    }

    #[test]
    fn test_parallel_subcommand() {
        let args = Args::parse_from(["test", "parallel", "run"]);
        if let Subcommands::Parallel { parallel } = args.subcommand {
            assert!(matches!(parallel, ParallelSubcommands::Run));
        } else {
            panic!("Expected Parallel subcommand");
        }
    }

    #[test]
    fn test_template_subcommand() {
        let args = Args::parse_from(["test", "template", "render"]);
        if let Subcommands::Template { template } = args.subcommand {
            assert!(matches!(template, TemplateSubcommands::Render));
        } else {
            panic!("Expected Template subcommand");
        }
    }

    #[test]
    fn test_workflow_subcommand() {
        let args = Args::parse_from(["test", "workflow", "run"]);
        if let Subcommands::Workflow { workflow } = args.subcommand {
            assert!(matches!(workflow, WorkflowSubcommands::Run));
        } else {
            panic!("Expected Workflow subcommand");
        }
    }

    #[test]
    fn test_disk_subcommand() {
        let args = Args::parse_from(["test", "disk", "monitor"]);
        if let Subcommands::Disk { disk } = args.subcommand {
            assert!(matches!(disk, DiskSubcommands::Monitor));
        } else {
            panic!("Expected Disk subcommand");
        }
    }

    #[test]
    fn test_config_subcommand() {
        let args = Args::parse_from(["test", "config", "show"]);
        if let Subcommands::Config { config } = args.subcommand {
            assert!(matches!(config, ConfigSubcommands::Show));
        } else {
            panic!("Expected Config subcommand");
        }
    }

    #[test]
    fn test_theme_subcommand() {
        let args = Args::parse_from(["test", "theme", "show"]);
        if let Subcommands::Theme { theme } = args.subcommand {
            assert!(matches!(theme, ThemeSubcommands::Show));
        } else {
            panic!("Expected Theme subcommand");
        }
    }

    #[test]
    fn test_multi_subcommand() {
        let args = Args::parse_from(["test", "multi", "run", "--command", "ls", "--command", "ps"]);
        if let Subcommands::Multi { multi } = args.subcommand {
            if let MultiSubcommands::Run { command, file } = multi {
                assert_eq!(command, vec!["ls", "ps"]);
                assert!(file.is_none());
            } else {
                panic!("Expected Run subcommand");
            }
        } else {
            panic!("Expected Multi subcommand");
        }
    }
}
