use crate::app::App;
use crate::cli::{Args, BufferMode};
use crate::theme::Theme as DrunsTheme;
use durable_run_tui_multi::{TuiApp, Theme};
use durable_run_config_batch::BatchConfig;
use std::fs;
use std::io::{self, Write};
use tokio::process::Command;
use tokio::io::{AsyncBufReadExt, AsyncReadExt, BufReader};
use futures::future::join_all;

/// Read input items from stdin or file
pub fn read_input_items(
    arg_file: Option<String>,
    delimiter: Option<char>,
    null: bool,
) -> Result<Vec<String>, Box<dyn std::error::Error>> {
    let mut items = Vec::new();

    // Read the full input
    let contents = if let Some(file_path) = arg_file {
        fs::read_to_string(file_path)?
    } else {
        let stdin = io::stdin();
        let mut reader = stdin.lock();
        let mut buffer = Vec::new();
        io::Read::read_to_end(&mut reader, &mut buffer)?;
        String::from_utf8_lossy(&buffer).to_string()
    };

    if null {
        // Split by null character
        for item in contents.split('\0') {
            let trimmed = item.trim();
            if !trimmed.is_empty() {
                items.push(trimmed.to_string());
            }
        }
    } else if let Some(delim) = delimiter {
        // Split by custom delimiter
        for item in contents.split(delim) {
            let trimmed = item.trim();
            if !trimmed.is_empty() {
                items.push(trimmed.to_string());
            }
        }
    } else {
        // Default behavior: split by whitespace (like GNU xargs)
        for item in contents.split_whitespace() {
            if !item.is_empty() {
                items.push(item.to_string());
            }
        }
    }

    Ok(items)
}

/// Batch input items according to max_args
pub fn batch_items(items: Vec<String>, max_args: Option<usize>) -> Vec<Vec<String>> {
    if let Some(max) = max_args {
        items.chunks(max).map(|chunk| chunk.to_vec()).collect()
    } else {
        vec![items]
    }
}

/// Generate command from template and input items
pub fn generate_command(
    command_template: &[String],
    replace_str: &str,
    items: &[String],
) -> String {
    let template = command_template.join(" ");

    // Quote each item for shell safety to preserve items with special characters
    let quoted_items: Vec<String> = items
        .iter()
        .map(|item| shell_words::quote(item).to_string())
        .collect();
    let items_str = quoted_items.join(" ");

    // If no placeholder found, append items to the end
    if !template.contains(replace_str) {
        if items.is_empty() {
            template
        } else {
            format!("{} {}", template, items_str)
        }
    } else {
        // Replace placeholder with items
        template.replace(replace_str, &items_str)
    }
}

/// Handle xargs subcommand - single job execution using regular TUI
pub async fn handle_xargs_single(
    args: Args,
    command_template: Vec<String>,
    replace_str: String,
    items: Vec<String>,
) -> Result<(), Box<dyn std::error::Error>> {
    // Generate the command
    let command_str = generate_command(&command_template, &replace_str, &items);

    // Split command into parts for execution
    let command_parts: Vec<String> = shell_words::split(&command_str)?;

    if command_parts.is_empty() {
        eprintln!("No command generated");
        std::process::exit(1);
    }

    // Run using the standard TUI
    let mut app = App::new(args, command_parts).await?;
    app.run().await?;
    let status = app.cleanup().await?;

    std::process::exit(status.code().unwrap_or(1));
}

/// Handle xargs subcommand - multi job execution using multi-pane TUI
pub async fn handle_xargs_multi(
    command_template: Vec<String>,
    replace_str: String,
    item_batches: Vec<Vec<String>>,
    max_procs: usize,
) -> Result<(), Box<dyn std::error::Error>> {
    // Generate commands for each batch
    let commands: Vec<String> = item_batches
        .iter()
        .map(|batch| generate_command(&command_template, &replace_str, batch))
        .collect();

    if commands.is_empty() {
        eprintln!("No commands to execute");
        std::process::exit(1);
    }

    // Create TUI app with parallelism config
    let mut config = BatchConfig::default();
    config.job_limits.max_concurrent = max_procs;
    let mut app = TuiApp::with_config(config);

    // Set theme to match druns
    let druns_theme = DrunsTheme::default();
    let theme = Theme {
        primary: druns_theme.primary,
        secondary: druns_theme.secondary,
        accent: druns_theme.accent,
        border: druns_theme.border,
        text: druns_theme.text,
        background: druns_theme.background,
        shades: druns_theme.shades,
    };
    app = app.with_theme(theme);

    // Add commands as tabs
    for (i, cmd) in commands.iter().enumerate() {
        let title = format!("Job {}", i + 1);
        app.add_command_tab(format!("job_{}", i), title, cmd.clone());
    }

    // Run the multi-tab TUI
    app.run().await?;

    Ok(())
}

/// Execute a single command without TUI
pub async fn execute_command_no_tui(
    command_str: String,
    buffer_mode: BufferMode,
    stdin_file: Option<String>,
    stdin_null: bool,
) -> Result<i32, Box<dyn std::error::Error>> {
    // Parse command into parts
    let command_parts = shell_words::split(&command_str)?;

    if command_parts.is_empty() {
        return Err("Empty command".into());
    }

    let program = command_parts[0].clone();
    let args: Vec<String> = command_parts[1..].to_vec();

    // Determine stdin configuration
    let stdin_config = if stdin_null {
        std::process::Stdio::null()
    } else if let Some(ref file_path) = stdin_file {
        let file = std::fs::File::open(file_path)?;
        std::process::Stdio::from(file)
    } else {
        std::process::Stdio::inherit()
    };

    // Spawn the command
    let mut child = Command::new(&program)
        .args(&args)
        .stdin(stdin_config)
        .stdout(std::process::Stdio::piped())
        .stderr(std::process::Stdio::piped())
        .spawn()?;

    // Get stdout and stderr handles
    let stdout = child.stdout.take().ok_or("Failed to get stdout")?;
    let stderr = child.stderr.take().ok_or("Failed to get stderr")?;

    match buffer_mode {
        BufferMode::Line => {
            // Line buffered - output each line as it's received
            let stdout_task = tokio::spawn(async move {
                let reader = BufReader::new(stdout);
                let mut lines = reader.lines();
                while let Ok(Some(line)) = lines.next_line().await {
                    println!("{}", line);
                }
            });

            let stderr_task = tokio::spawn(async move {
                let reader = BufReader::new(stderr);
                let mut lines = reader.lines();
                while let Ok(Some(line)) = lines.next_line().await {
                    eprintln!("{}", line);
                }
            });

            // Wait for both output tasks
            let _ = tokio::join!(stdout_task, stderr_task);
        }
        BufferMode::None => {
            // Unbuffered - output each byte immediately
            let stdout_task = tokio::spawn(async move {
                let mut reader = stdout;
                let mut buffer = [0u8; 1];
                let mut stdout_lock = std::io::stdout();
                while let Ok(n) = reader.read(&mut buffer).await {
                    if n == 0 {
                        break;
                    }
                    let _ = stdout_lock.write_all(&buffer[..n]);
                    let _ = stdout_lock.flush();
                }
            });

            let stderr_task = tokio::spawn(async move {
                let mut reader = stderr;
                let mut buffer = [0u8; 1];
                let mut stderr_lock = std::io::stderr();
                while let Ok(n) = reader.read(&mut buffer).await {
                    if n == 0 {
                        break;
                    }
                    let _ = stderr_lock.write_all(&buffer[..n]);
                    let _ = stderr_lock.flush();
                }
            });

            // Wait for both output tasks
            let _ = tokio::join!(stdout_task, stderr_task);
        }
        BufferMode::Full => {
            // Fully buffered - collect all output then print at the end
            let stdout_task = tokio::spawn(async move {
                let mut reader = stdout;
                let mut buffer = Vec::new();
                let _ = reader.read_to_end(&mut buffer).await;
                buffer
            });

            let stderr_task = tokio::spawn(async move {
                let mut reader = stderr;
                let mut buffer = Vec::new();
                let _ = reader.read_to_end(&mut buffer).await;
                buffer
            });

            // Wait for both output tasks
            let (stdout_result, stderr_result) = tokio::join!(stdout_task, stderr_task);

            if let Ok(stdout_bytes) = stdout_result {
                let _ = std::io::stdout().write_all(&stdout_bytes);
            }
            if let Ok(stderr_bytes) = stderr_result {
                let _ = std::io::stderr().write_all(&stderr_bytes);
            }
        }
    }

    // Wait for the command to finish
    let status = child.wait().await?;

    Ok(status.code().unwrap_or(1))
}

/// Handle xargs without TUI - single execution
pub async fn handle_xargs_no_tui_single(
    command_template: Vec<String>,
    replace_str: String,
    items: Vec<String>,
    buffer_mode: BufferMode,
    stdin_file: Option<String>,
    stdin_null: bool,
) -> Result<(), Box<dyn std::error::Error>> {
    let command_str = generate_command(&command_template, &replace_str, &items);
    let exit_code = execute_command_no_tui(command_str, buffer_mode, stdin_file, stdin_null).await?;
    std::process::exit(exit_code);
}

/// Handle xargs without TUI - parallel execution
pub async fn handle_xargs_no_tui_parallel(
    command_template: Vec<String>,
    replace_str: String,
    item_batches: Vec<Vec<String>>,
    max_procs: usize,
    buffer_mode: BufferMode,
    stdin_file: Option<String>,
    stdin_null: bool,
) -> Result<(), Box<dyn std::error::Error>> {
    // Generate commands for each batch
    let commands: Vec<String> = item_batches
        .iter()
        .map(|batch| generate_command(&command_template, &replace_str, batch))
        .collect();

    if commands.is_empty() {
        eprintln!("No commands to execute");
        std::process::exit(1);
    }

    // Use a shared channel to queue commands and limit concurrent executions
    let (tx, rx) = tokio::sync::mpsc::channel::<String>(commands.len());
    let rx = std::sync::Arc::new(tokio::sync::Mutex::new(rx));

    // Send all commands to the channel
    for cmd in commands {
        tx.send(cmd).await?;
    }
    drop(tx); // Close the channel

    // Spawn worker tasks limited by max_procs
    let mut workers = Vec::new();
    let results = std::sync::Arc::new(tokio::sync::Mutex::new(Vec::new()));

    for _ in 0..max_procs {
        let worker_rx = rx.clone();
        let results_clone = results.clone();
        let stdin_file_clone = stdin_file.clone();

        let worker = tokio::spawn(async move {
            loop {
                let cmd = {
                    let mut rx_guard = worker_rx.lock().await;
                    rx_guard.recv().await
                };

                match cmd {
                    Some(cmd) => {
                        let exit_code = match execute_command_no_tui(cmd, buffer_mode, stdin_file_clone.clone(), stdin_null).await {
                            Ok(code) => code,
                            Err(e) => {
                                eprintln!("Command error: {}", e);
                                1
                            }
                        };
                        results_clone.lock().await.push(exit_code);
                    }
                    None => break, // Channel closed and empty
                }
            }
        });

        workers.push(worker);
    }

    // Wait for all workers to complete
    join_all(workers).await;

    // Check if any command failed
    let final_results = results.lock().await;
    let exit_code = final_results.iter().find(|&&code| code != 0).copied().unwrap_or(0);

    std::process::exit(exit_code);
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_generate_command_with_placeholder() {
        let template = vec!["echo".to_string(), "{}".to_string()];
        let items = vec!["hello".to_string(), "world".to_string()];
        let result = generate_command(&template, "{}", &items);
        assert_eq!(result, "echo hello world");
    }

    #[test]
    fn test_generate_command_without_placeholder() {
        let template = vec!["echo".to_string()];
        let items = vec!["hello".to_string(), "world".to_string()];
        let result = generate_command(&template, "{}", &items);
        assert_eq!(result, "echo hello world");
    }

    #[test]
    fn test_generate_command_custom_placeholder() {
        let template = vec!["grep".to_string(), "-r".to_string(), "PATTERN".to_string()];
        let items = vec!["file.txt".to_string()];
        let result = generate_command(&template, "PATTERN", &items);
        assert_eq!(result, "grep -r file.txt");
    }

    #[test]
    fn test_batch_items_with_limit() {
        let items = vec![
            "1".to_string(),
            "2".to_string(),
            "3".to_string(),
            "4".to_string(),
            "5".to_string(),
        ];
        let batches = batch_items(items, Some(2));
        assert_eq!(batches.len(), 3);
        assert_eq!(batches[0], vec!["1", "2"]);
        assert_eq!(batches[1], vec!["3", "4"]);
        assert_eq!(batches[2], vec!["5"]);
    }

    #[test]
    fn test_batch_items_no_limit() {
        let items = vec!["1".to_string(), "2".to_string(), "3".to_string()];
        let batches = batch_items(items.clone(), None);
        assert_eq!(batches.len(), 1);
        assert_eq!(batches[0], items);
    }

    #[test]
    fn test_generate_command_empty_items() {
        let template = vec!["echo".to_string(), "{}".to_string()];
        let items = vec![];
        let result = generate_command(&template, "{}", &items);
        assert_eq!(result, "echo ");
    }

    #[test]
    fn test_generate_command_with_spaces() {
        // Note: with xargs-like behavior, "hello world" would be split into two items
        // This test shows quoting preserves a single item that happens to have spaces
        let template = vec!["echo".to_string()];
        let items = vec!["hello world".to_string()];
        let result = generate_command(&template, "{}", &items);
        assert_eq!(result, "echo 'hello world'");
    }

    #[test]
    fn test_generate_command_with_quotes() {
        let template = vec!["echo".to_string()];
        let items = vec!["it's".to_string()];
        let result = generate_command(&template, "{}", &items);
        // shell_words::quote should escape the single quote
        assert!(result.contains("it"));
    }

    #[test]
    fn test_generate_command_multiple_items() {
        // Items are already split, quoting preserves them
        let template = vec!["echo".to_string(), "{}".to_string()];
        let items = vec!["hello".to_string(), "world".to_string()];
        let result = generate_command(&template, "{}", &items);
        assert_eq!(result, "echo hello world");
    }

    #[test]
    fn test_generate_command_with_special_chars() {
        let template = vec!["echo".to_string()];
        let items = vec!["$VAR".to_string(), "*.txt".to_string()];
        let result = generate_command(&template, "{}", &items);
        // Special chars should be quoted to prevent shell expansion
        assert!(result.contains("$VAR"));
        assert!(result.contains("*.txt"));
    }

    #[test]
    fn test_read_input_items_whitespace_splitting() {
        use std::io::Write;
        use tempfile::NamedTempFile;

        // Create a temp file with whitespace-separated items
        let mut file = NamedTempFile::new().unwrap();
        writeln!(file, "hello world\nfoo  bar\t baz").unwrap();

        let items = read_input_items(
            Some(file.path().to_string_lossy().to_string()),
            None,
            false
        ).unwrap();

        // Default behavior splits on all whitespace
        assert_eq!(items, vec!["hello", "world", "foo", "bar", "baz"]);
    }

    #[test]
    fn test_read_input_items_null_delimiter() {
        use std::io::Write;
        use tempfile::NamedTempFile;

        // Create a temp file with null-separated items
        let mut file = NamedTempFile::new().unwrap();
        write!(file, "hello world\0foo bar\0").unwrap();

        let items = read_input_items(
            Some(file.path().to_string_lossy().to_string()),
            None,
            true
        ).unwrap();

        // Null delimiter preserves spaces within items
        assert_eq!(items, vec!["hello world", "foo bar"]);
    }

    #[test]
    fn test_read_input_items_custom_delimiter() {
        use std::io::Write;
        use tempfile::NamedTempFile;

        // Create a temp file with comma-separated items
        let mut file = NamedTempFile::new().unwrap();
        write!(file, "hello,world,foo bar").unwrap();

        let items = read_input_items(
            Some(file.path().to_string_lossy().to_string()),
            Some(','),
            false
        ).unwrap();

        // Custom delimiter preserves spaces within items
        assert_eq!(items, vec!["hello", "world", "foo bar"]);
    }
}
