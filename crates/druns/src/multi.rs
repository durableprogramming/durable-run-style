use crate::cli::MultiSubcommands;
use crate::theme::Theme as DrunsTheme;
use durable_run_tui_multi::{TuiApp, Theme};
use durable_run_config_batch::BatchConfig;
use std::fs;
use std::io::{self, BufRead};

pub async fn handle_multi(multi: &MultiSubcommands) -> Result<(), Box<dyn std::error::Error>> {
    match multi {
        MultiSubcommands::Run { command, file } => {
            let commands = collect_commands(command.clone(), file.clone())?;

            if commands.is_empty() {
                eprintln!("No commands provided. Use --command to specify commands or --file to load from a file.");
                std::process::exit(1);
            }

            // Create TUI app with default config
            let config = BatchConfig::default();
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
                let title = format!("Command {}", i + 1);
                app.add_command_tab(format!("tab_{}", i), title, cmd.clone());
            }

            // Run the multi-tab TUI
            app.run().await?;
        }
    }

    Ok(())
}

fn collect_commands(command_args: Vec<String>, file_path: Option<String>) -> Result<Vec<String>, Box<dyn std::error::Error>> {
    let mut commands = Vec::new();

    // Add commands from --command flags
    commands.extend(command_args);

    // Add commands from file if specified
    if let Some(path) = file_path {
        let file = fs::File::open(&path)?;
        let reader = io::BufReader::new(file);

        for line in reader.lines() {
            let line = line?;
            let trimmed = line.trim();
            if !trimmed.is_empty() && !trimmed.starts_with('#') {
                commands.push(trimmed.to_string());
            }
        }
    }

    Ok(commands)
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::NamedTempFile;
    use std::io::Write;

    #[test]
    fn test_collect_commands_from_args() {
        let commands = vec!["ls".to_string(), "ps".to_string()];
        let result = collect_commands(commands, None).unwrap();
        assert_eq!(result, vec!["ls", "ps"]);
    }

    #[test]
    fn test_collect_commands_from_file() {
        let mut temp_file = NamedTempFile::new().unwrap();
        writeln!(temp_file, "ls -la").unwrap();
        writeln!(temp_file, "# This is a comment").unwrap();
        writeln!(temp_file, "ps aux").unwrap();
        writeln!(temp_file, "").unwrap(); // Empty line
        writeln!(temp_file, "echo hello").unwrap();

        let file_path = temp_file.path().to_str().unwrap().to_string();
        let result = collect_commands(vec![], Some(file_path)).unwrap();
        assert_eq!(result, vec!["ls -la", "ps aux", "echo hello"]);
    }

    #[test]
    fn test_collect_commands_combined() {
        let mut temp_file = NamedTempFile::new().unwrap();
        writeln!(temp_file, "ls -la").unwrap();
        writeln!(temp_file, "ps aux").unwrap();

        let file_path = temp_file.path().to_str().unwrap().to_string();
        let commands = vec!["echo hello".to_string()];
        let result = collect_commands(commands, Some(file_path)).unwrap();
        assert_eq!(result, vec!["echo hello", "ls -la", "ps aux"]);
    }
}