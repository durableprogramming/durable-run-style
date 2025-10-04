# druns

![Demo](demo.gif)

**Durable RUNner with Style** - A beautiful terminal user interface for running commands with real-time system monitoring and process management.

[![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg)](https://opensource.org/licenses/MIT)
[![Rust](https://img.shields.io/badge/rust-1.70%2B-orange)](https://www.rust-lang.org/)

## Features

- **Beautiful TUI Interface**: Modern terminal UI built with Ratatui featuring customizable themes and animations
- **Real-time System Monitoring**: Live CPU, memory, disk I/O, and network statistics
- **Process Management**: Advanced process spawning with pseudo-terminal support
- **Output Logging**: Simultaneous display and file logging of command output
- **Configurable Layout**: Customizable sidebar width, output history, and animation settings
- **Graphics Demo Mode**: Built-in demo showcasing UI capabilities
- **Experimental Network Monitoring**: Optional pcap-based network traffic analysis

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

## Credits

Built with:
- [Ratatui](https://github.com/ratatui-org/ratatui) - Terminal UI framework
- [Crossterm](https://github.com/crossterm-rs/crossterm) - Terminal manipulation
- [Tokio](https://tokio.rs/) - Async runtime
- [Sysinfo](https://github.com/GuillaumeGomez/sysinfo) - System information

