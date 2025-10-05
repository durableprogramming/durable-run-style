use clap::Parser;

#[derive(Parser)]
#[command(author, version, about, long_about = None)]
pub struct Args {
    /// The command to run (if starts with 'run' or 'gfx-demo', special handling)
    #[arg(trailing_var_arg = true, allow_hyphen_values = true)]
    pub command: Vec<String>,

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
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_args() {
        let args = Args::parse_from(["test"]);
        assert_eq!(args.sidebar_width, 30);
        assert_eq!(args.max_output_lines, 1000);
        assert_eq!(args.max_command_lines, 3);
        assert!(args.animation);
        assert!(!args.no_animate);
        assert!(args.log.is_none());
        assert!(args.config.is_none());
        assert!(args.command.is_empty());
    }

    #[test]
    fn test_command_parsing() {
        let args = Args::parse_from(["test", "ls", "-la"]);
        assert_eq!(args.command, vec!["ls", "-la"]);
    }

    #[test]
    fn test_sidebar_width() {
        let args = Args::parse_from(["test", "--sidebar-width", "50"]);
        assert_eq!(args.sidebar_width, 50);
    }

    #[test]
    fn test_max_output_lines() {
        let args = Args::parse_from(["test", "--max-output-lines", "500"]);
        assert_eq!(args.max_output_lines, 500);
    }

    #[test]
    fn test_max_command_lines() {
        let args = Args::parse_from(["test", "--max-command-lines", "5"]);
        assert_eq!(args.max_command_lines, 5);
    }

    #[test]
    fn test_animation_flags() {
        // Test --no-animate flag (sets no_animate to true, but doesn't change animation)
        let args = Args::parse_from(["test", "--no-animate"]);
        assert!(args.animation); // default is true
        assert!(args.no_animate);

        // Test default animation = true
        let args = Args::parse_from(["test"]);
        assert!(args.animation);
        assert!(!args.no_animate);
    }

    #[test]
    fn test_log_option() {
        let args = Args::parse_from(["test", "--log", "output.log"]);
        assert_eq!(args.log, Some("output.log".to_string()));
    }

    #[test]
    fn test_config_option() {
        let args = Args::parse_from(["test", "--config", "config.toml"]);
        assert_eq!(args.config, Some("config.toml".to_string()));
    }

    #[test]
    fn test_combined_options() {
        let args = Args::parse_from([
            "test",
            "--config", "myconfig.toml",
            "--sidebar-width", "40",
            "--max-output-lines", "2000",
            "--log", "debug.log",
            "--no-animate",
            "echo", "hello"
        ]);
        assert_eq!(args.config, Some("myconfig.toml".to_string()));
        assert_eq!(args.sidebar_width, 40);
        assert_eq!(args.max_output_lines, 2000);
        assert_eq!(args.log, Some("debug.log".to_string()));
        assert!(args.animation); // --animation defaults to true
        assert!(args.no_animate); // --no-animate sets this flag
        assert_eq!(args.command, vec!["echo", "hello"]);
    }

    #[test]
    fn test_hyphen_values_in_command() {
        let args = Args::parse_from(["test", "grep", "-r", "--include=*.rs", "pattern"]);
        assert_eq!(args.command, vec!["grep", "-r", "--include=*.rs", "pattern"]);
    }
}