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

    /// Enable/disable animation
    #[arg(long, default_value = "true")]
    pub animation: bool,

    /// Disable animation (equivalent to --animation false)
    #[arg(long)]
    pub no_animate: bool,
}