# druns

![Demo](demo.gif)

**Durable RUNner with Style** - A beautiful terminal user interface for running commands with real-time system monitoring and process management.

[![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg)](https://opensource.org/licenses/MIT)
[![Rust](https://img.shields.io/badge/rust-1.70%2B-orange)](https://www.rust-lang.org/)

## Features

### Core Features
- **Beautiful TUI Interface**: Modern terminal UI built with Ratatui featuring customizable themes and animations
- **Real-time System Monitoring**: Live CPU, memory, disk I/O, and network statistics with sparkline visualizations
- **Process Management**: Advanced process spawning with pseudo-terminal support
- **Output Logging**: Simultaneous display and file logging of command output
- **Smart Output Display**: Word wrapping for long commands, PPID and PWD indicators
- **Enhanced Scrolling**: Improved scroll/follow behavior with PageDown and Escape navigation
- **Visual Enhancements**: Animated shine effects, top and bottom footer art, customizable sidebar
- **Configurable Layout**: Customizable sidebar width, output history, and animation settings
- **Graphics Demo Mode**: Built-in demo showcasing UI capabilities

### Advanced Features
- **Analytics Dashboard**: Comprehensive analytics with data collection, trend analysis, anomaly detection, and reporting
- **Batch Configuration**: Create and manage batch job configurations with resource limits, timeouts, and validation
- **Parallel Execution**: Run commands in parallel with queue management and control flow
- **Template Engine**: Powerful template rendering with macros and context support
- **Workflow Management**: Define and run complex workflows with validation
- **Theme Management**: Create, validate, and apply custom color themes
- **Config Management**: Show, validate, and create configuration files

### Advanced Features
- **Analytics Dashboard**: Comprehensive analytics with data collection, trend analysis, anomaly detection, and reporting
- **Batch Configuration**: Create and manage batch job configurations with resource limits, timeouts, and validation
- **Parallel Execution**: Run commands in parallel with queue management and control flow
- **Template Engine**: Powerful template rendering with macros and context support
- **Workflow Management**: Define and run complex workflows with validation
- **Theme Management**: Create, validate, and apply custom color themes
- **Config Management**: Show, validate, and create configuration files
- **Multi-tab TUI**: Run multiple commands concurrently in separate tabs with real-time monitoring

### Experimental Features
- **Network Monitoring**: Optional pcap-based network traffic analysis
- **Plugin System**: Extensible architecture for custom integrations
- **Performance Profiling**: Built-in performance monitoring and analysis tools

## Installation

### From Source

```bash
git clone https://github.com/durableprogramming/durable-run-style.git
cd durable-run-style
cargo build --release
# Binary will be available at target/release/druns
```

### Using Cargo

```bash
cargo install --git https://github.com/durableprogramming/durable-run-style.git
```

### Pre-built Binaries

Download the latest release from the [releases page](https://github.com/durableprogramming/durable-run-style/releases).

## Quick Start

### Basic Usage

Run any command with a beautiful interface:

```bash
druns ls -la
druns docker compose up
druns npm run dev
```

### Graphics Demo

Experience the UI capabilities:

```bash
druns gfx-demo
```

### Advanced Options

```bash
# Log output to file while displaying
druns --log output.log cargo build

# Custom sidebar width
druns --sidebar-width 40 docker compose logs -f

# Disable animations
druns --no-animate ./my-long-running-script.sh

# Custom configuration
druns --config my-config.toml ./my-command

# Limit command display lines
druns --max-command-lines 5 ./very-long-command.sh

# Adjust output buffer
druns --max-output-lines 2000 ./command
```

### Analytics Commands

Track and analyze system metrics from your command executions:

```bash
# View analytics dashboard
druns analytics dashboard

# Show analytics data
druns analytics show

# Generate statistics report
druns analytics stats

# Display charts and visualizations
druns analytics chart

# Generate comprehensive report
druns analytics report

# Collect analytics data
druns analytics collect
```

### Batch Configuration

Manage batch job configurations:

```bash
# Create a new batch configuration
druns batch create my-batch

# Show current batch configuration
druns batch show

# Validate batch configuration
druns batch validate

# Set job limits
druns batch set-limits --max-jobs 100 --max-concurrent 10 --max-retries 3

# Configure timeouts
druns batch set-timeouts --job-timeout 3600 --batch-timeout 7200

# Set resource controls
druns batch set-resources --max-memory-mb 4096 --max-cpu-percent 80
```

### Parallel Execution

Run commands in parallel:

```bash
# Run commands in parallel
druns parallel run

# Execute from input sources
druns parallel from-inputs

# Manage execution queue
druns parallel queue
```

### Template Operations

Work with command templates:

```bash
# Render a template
druns template render

# Parse template syntax
druns template parse

# List available macros
druns template macros
```

### Workflow Management

Define and execute workflows:

```bash
# Run a workflow
druns workflow run

# Validate workflow definition
druns workflow validate

# List available workflows
druns workflow list
```

### Configuration & Theme Management

```bash
# Show current configuration
druns config show

# Validate configuration
druns config validate

# Create new configuration
druns config create

# Show current theme
druns theme show

# Create custom theme
druns theme create

# Validate theme
druns theme validate
```

### Multi-tab TUI

Run multiple commands concurrently in separate tabs:

```bash
# Run multiple commands in tabs
druns multi run --command "ls -la" --command "ps aux" --command "df -h"

# Load commands from a file (one per line)
druns multi run --file commands.txt
```

Key bindings for multi-tab mode:
- `Tab` / `Right` / `Left`: Switch between tabs
- `BackTab`: Switch to previous tab
- `1-9`: Jump to specific tab
- `r`: Reset and re-run all commands
- `q` / `Esc`: Quit

### Disk Monitoring

```bash
# Monitor disk I/O in real-time
druns disk monitor

# Show disk statistics
druns disk stats
```

## Configuration

Create a `config.toml` file or use `--config` to specify a custom configuration:

```toml
[app.layout]
sidebar_width = 35

[app.output]
max_output_lines = 2000

[app.animation]
animation_enabled = true
no_animate = false

[theme.colors]
# Custom color scheme
```

## Demo

The included demo showcases druns running a multi-service Docker Compose setup with real-time monitoring:

```bash
cd demo
vhs demo.tape  # Requires VHS for recording
```

Or run it manually:

```bash
druns docker compose up
```

This starts multiple services (nginx, redis, postgres, mysql, mongo) and displays their combined output with system statistics.

## Key Bindings

- `Ctrl+C`: Gracefully terminate the running process
- `PageDown`: Scroll down one page (exits follow mode)
- `Escape`: Return to end of log and resume following
- `q`: Quit (when process has finished)

## Project Structure

The project is organized into multiple specialized crates:

### Core Crates
- **druns**: Main CLI application and TUI implementation
- **durable-run-parallel**: Parallel command execution with queue management
- **durable-run-template**: Template engine with macro support

### Configuration & Management
- **durable-run-config-batch**: Batch job configuration and validation
- **durable-run-workflow**: Workflow definition and execution engine
- **durable-run-input**: Input source handling and processing

### Monitoring & Analytics
- **durable-run-analytics**: Data collection, trend analysis, and reporting
- **durable-run-monitor**: System resource monitoring
- **durable-run-perf**: Performance profiling and optimization tools

### UI & Integration
- **durable-run-tui-multi**: Multi-tab terminal UI for concurrent command execution
- **durable-run-plugin**: Plugin system for extensibility
- **durable-run-integrations**: Third-party service integrations

### Development
- **durable-run-testing**: Testing utilities and frameworks
- **durable-run-docs**: Documentation generation tools


## Requirements

- Rust 1.70+
- Linux/macOS/Windows (with terminal support)
- For experimental features: libpcap development headers

## Contributing

1. Fork the repository
2. Create a feature branch
3. Make your changes
4. Run tests: `cargo test`
5. Format code: `cargo fmt`
6. Check linting: `cargo clippy`
7. Submit a pull request

## License

MIT License - see [LICENSE](LICENSE) file for details.

## Recent Improvements

- ✨ Added shine animation variations for enhanced visual appeal
- 🎨 New bottom footer art for improved aesthetics
- 📝 Word wrapping for long commands with visual indicators
- 🔍 PPID (Parent Process ID) indicator
- 📂 PWD (Present Working Directory) indicator
- 📜 Improved scroll/follow behavior with better user control
- 🧪 Expanded test coverage with additional test cases
- 🔧 Code quality improvements to satisfy clippy lints

## Credits

Built with:
- [Ratatui](https://github.com/ratatui-org/ratatui) - Terminal UI framework
- [Crossterm](https://github.com/crossterm-rs/crossterm) - Terminal manipulation
- [Tokio](https://tokio.rs/) - Async runtime
- [Sysinfo](https://github.com/GuillaumeGomez/sysinfo) - System information
- [Clap](https://github.com/clap-rs/clap) - Command-line argument parser
- [Logos](https://github.com/maciejhirsz/logos) - Lexical scanner for template engine
- [Serde](https://serde.rs/) - Serialization framework

