use crate::cli::BatchSubcommands;
use durable_run_config_batch::{BatchConfig, ConfigError};
use std::path::Path;

const DEFAULT_CONFIG_FILE: &str = "batch.yaml";

fn get_config_path(file: &Option<String>) -> &str {
    file.as_deref().unwrap_or(DEFAULT_CONFIG_FILE)
}

pub async fn handle_batch(subcommand: &BatchSubcommands) -> Result<(), Box<dyn std::error::Error>> {
    match subcommand {
        BatchSubcommands::Create { name, file } => {
            handle_create(name, file)?;
        }
        BatchSubcommands::Show { file } => {
            handle_show(file)?;
        }
        BatchSubcommands::Validate { file } => {
            handle_validate(file)?;
        }
        BatchSubcommands::SetLimits { file, max_jobs, max_concurrent, max_retries } => {
            handle_set_limits(file, max_jobs, max_concurrent, max_retries)?;
        }
        BatchSubcommands::SetTimeouts { file, job_timeout, batch_timeout, connection_timeout } => {
            handle_set_timeouts(file, job_timeout, batch_timeout, connection_timeout)?;
        }
        BatchSubcommands::SetResources { file, max_memory_mb, max_cpu_percent, max_disk_mb, priority } => {
            handle_set_resources(file, max_memory_mb, max_cpu_percent, max_disk_mb, priority)?;
        }
    }

    Ok(())
}

fn handle_create(name: &str, file: &Option<String>) -> Result<(), Box<dyn std::error::Error>> {
    let config_path = get_config_path(file);

    if Path::new(config_path).exists() {
        return Err(format!("Configuration file '{}' already exists. Use a different name or remove the existing file.", config_path).into());
    }

    let config = BatchConfig::new(name.to_string());
    config.save_to_file(config_path)?;

    println!("Created batch configuration '{}' at '{}'", name, config_path);
    println!("You can now modify the configuration using the set-* subcommands.");

    Ok(())
}

fn handle_show(file: &Option<String>) -> Result<(), Box<dyn std::error::Error>> {
    let config_path = get_config_path(file);

    let config = BatchConfig::load_from_file(config_path)?;

    println!("Batch Configuration: {}", config.name);
    if let Some(desc) = &config.description {
        println!("Description: {}", desc);
    }
    println!();

    println!("Job Limits:");
    println!("  Max Jobs: {}", config.job_limits.max_jobs);
    println!("  Max Concurrent: {}", config.job_limits.max_concurrent);
    println!("  Max Retries: {}", config.job_limits.max_retries);
    println!();

    println!("Timeouts:");
    println!("  Job Timeout: {}s", config.timeout_settings.job_timeout.as_secs());
    println!("  Batch Timeout: {}s", config.timeout_settings.batch_timeout.as_secs());
    println!("  Connection Timeout: {}s", config.timeout_settings.connection_timeout.as_secs());
    println!();

    println!("Resource Controls:");
    println!("  Max Memory: {} MB", config.resource_control.max_memory_mb);
    println!("  Max CPU: {}%", config.resource_control.max_cpu_percent);
    println!("  Max Disk: {} MB", config.resource_control.max_disk_mb);
    println!("  Priority: {}", config.resource_control.priority);

    Ok(())
}

fn handle_validate(file: &Option<String>) -> Result<(), Box<dyn std::error::Error>> {
    let config_path = get_config_path(file);

    let config = BatchConfig::load_from_file(config_path)?;

    match config.validate() {
        Ok(()) => {
            println!("✓ Batch configuration '{}' is valid", config.name);
            Ok(())
        }
        Err(ConfigError::Invalid(msg)) => {
            println!("✗ Batch configuration validation failed: {}", msg);
            Err(msg.into())
        }
        Err(e) => Err(e.into()),
    }
}

fn handle_set_limits(
    file: &Option<String>,
    max_jobs: &Option<usize>,
    max_concurrent: &Option<usize>,
    max_retries: &Option<usize>,
) -> Result<(), Box<dyn std::error::Error>> {
    let config_path = get_config_path(file);

    let mut config = BatchConfig::load_from_file(config_path)?;

    let mut changed = false;

    if let Some(max_jobs) = max_jobs {
        config.job_limits.max_jobs = *max_jobs;
        changed = true;
        println!("Set max_jobs to {}", max_jobs);
    }

    if let Some(max_concurrent) = max_concurrent {
        config.job_limits.max_concurrent = *max_concurrent;
        changed = true;
        println!("Set max_concurrent to {}", max_concurrent);
    }

    if let Some(max_retries) = max_retries {
        config.job_limits.max_retries = *max_retries;
        changed = true;
        println!("Set max_retries to {}", max_retries);
    }

    if !changed {
        println!("No limits specified to change. Use --max-jobs, --max-concurrent, or --max-retries.");
        return Ok(());
    }

    // Validate before saving
    config.validate()?;
    config.save_to_file(config_path)?;

    println!("Limits updated successfully in '{}'", config_path);

    Ok(())
}

fn handle_set_timeouts(
    file: &Option<String>,
    job_timeout: &Option<u64>,
    batch_timeout: &Option<u64>,
    connection_timeout: &Option<u64>,
) -> Result<(), Box<dyn std::error::Error>> {
    let config_path = get_config_path(file);

    let mut config = BatchConfig::load_from_file(config_path)?;

    let mut changed = false;

    if let Some(job_timeout) = job_timeout {
        config.timeout_settings.job_timeout = std::time::Duration::from_secs(*job_timeout);
        changed = true;
        println!("Set job_timeout to {}s", job_timeout);
    }

    if let Some(batch_timeout) = batch_timeout {
        config.timeout_settings.batch_timeout = std::time::Duration::from_secs(*batch_timeout);
        changed = true;
        println!("Set batch_timeout to {}s", batch_timeout);
    }

    if let Some(connection_timeout) = connection_timeout {
        config.timeout_settings.connection_timeout = std::time::Duration::from_secs(*connection_timeout);
        changed = true;
        println!("Set connection_timeout to {}s", connection_timeout);
    }

    if !changed {
        println!("No timeouts specified to change. Use --job-timeout, --batch-timeout, or --connection-timeout.");
        return Ok(());
    }

    // Validate before saving
    config.validate()?;
    config.save_to_file(config_path)?;

    println!("Timeouts updated successfully in '{}'", config_path);

    Ok(())
}

fn handle_set_resources(
    file: &Option<String>,
    max_memory_mb: &Option<usize>,
    max_cpu_percent: &Option<f32>,
    max_disk_mb: &Option<usize>,
    priority: &Option<i32>,
) -> Result<(), Box<dyn std::error::Error>> {
    let config_path = get_config_path(file);

    let mut config = BatchConfig::load_from_file(config_path)?;

    let mut changed = false;

    if let Some(max_memory_mb) = max_memory_mb {
        config.resource_control.max_memory_mb = *max_memory_mb;
        changed = true;
        println!("Set max_memory_mb to {} MB", max_memory_mb);
    }

    if let Some(max_cpu_percent) = max_cpu_percent {
        config.resource_control.max_cpu_percent = *max_cpu_percent;
        changed = true;
        println!("Set max_cpu_percent to {}%", max_cpu_percent);
    }

    if let Some(max_disk_mb) = max_disk_mb {
        config.resource_control.max_disk_mb = *max_disk_mb;
        changed = true;
        println!("Set max_disk_mb to {} MB", max_disk_mb);
    }

    if let Some(priority) = priority {
        config.resource_control.priority = *priority;
        changed = true;
        println!("Set priority to {}", priority);
    }

    if !changed {
        println!("No resources specified to change. Use --max-memory-mb, --max-cpu-percent, --max-disk-mb, or --priority.");
        return Ok(());
    }

    // Validate before saving
    config.validate()?;
    config.save_to_file(config_path)?;

    println!("Resources updated successfully in '{}'", config_path);

    Ok(())
}
