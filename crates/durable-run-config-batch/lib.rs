use serde::{Deserialize, Serialize};
use std::path::Path;
use std::time::Duration;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum ConfigError {
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),
    #[error("YAML serialization error: {0}")]
    Serde(#[from] serde_yaml::Error),
    #[error("JSON serialization error: {0}")]
    Json(#[from] serde_json::Error),
    #[error("Invalid configuration: {0}")]
    Invalid(String),
}

pub type Result<T> = std::result::Result<T, ConfigError>;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct JobLimits {
    pub max_jobs: usize,
    pub max_concurrent: usize,
    pub max_retries: usize,
}

impl Default for JobLimits {
    fn default() -> Self {
        Self {
            max_jobs: 1000,
            max_concurrent: 10,
            max_retries: 3,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TimeoutSettings {
    pub job_timeout: Duration,
    pub batch_timeout: Duration,
    pub connection_timeout: Duration,
}

impl Default for TimeoutSettings {
    fn default() -> Self {
        Self {
            job_timeout: Duration::from_secs(300), // 5 minutes
            batch_timeout: Duration::from_secs(3600), // 1 hour
            connection_timeout: Duration::from_secs(30),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResourceControl {
    pub max_memory_mb: usize,
    pub max_cpu_percent: f32,
    pub max_disk_mb: usize,
    pub priority: i32,
}

impl Default for ResourceControl {
    fn default() -> Self {
        Self {
            max_memory_mb: 1024,
            max_cpu_percent: 50.0,
            max_disk_mb: 10240,
            priority: 0,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BatchConfig {
    pub job_limits: JobLimits,
    pub timeout_settings: TimeoutSettings,
    pub resource_control: ResourceControl,
    pub name: String,
    pub description: Option<String>,
}

impl BatchConfig {
    pub fn new(name: String) -> Self {
        Self {
            name,
            description: None,
            job_limits: JobLimits::default(),
            timeout_settings: TimeoutSettings::default(),
            resource_control: ResourceControl::default(),
        }
    }

    pub fn from_yaml(yaml: &str) -> Result<Self> {
        Ok(serde_yaml::from_str(yaml)?)
    }

    pub fn from_json(json: &str) -> Result<Self> {
        Ok(serde_json::from_str(json)?)
    }

    pub fn to_yaml(&self) -> Result<String> {
        Ok(serde_yaml::to_string(self)?)
    }

    pub fn to_json(&self) -> Result<String> {
        Ok(serde_json::to_string(self)?)
    }

    pub fn load_from_file<P: AsRef<Path>>(path: P) -> Result<Self> {
        let contents = std::fs::read_to_string(path)?;
        let config: BatchConfig = serde_yaml::from_str(&contents)?;
        Ok(config)
    }

    pub fn save_to_file<P: AsRef<Path>>(&self, path: P) -> Result<()> {
        let yaml = serde_yaml::to_string(self)?;
        std::fs::write(path, yaml)?;
        Ok(())
    }

    /// Load configuration from environment variables with the given prefix.
    /// Environment variables should be in the format PREFIX_FIELD_NAME.
    /// For example, with prefix "BATCH_", BATCH_MAX_JOBS=100 would set max_jobs.
    pub fn from_env(prefix: &str) -> Result<Self> {
        let mut config = Self::default();

        // Job limits
        if let Ok(val) = std::env::var(format!("{}MAX_JOBS", prefix)) {
            config.job_limits.max_jobs = val.parse().map_err(|_| ConfigError::Invalid("Invalid MAX_JOBS value".to_string()))?;
        }
        if let Ok(val) = std::env::var(format!("{}MAX_CONCURRENT", prefix)) {
            config.job_limits.max_concurrent = val.parse().map_err(|_| ConfigError::Invalid("Invalid MAX_CONCURRENT value".to_string()))?;
        }
        if let Ok(val) = std::env::var(format!("{}MAX_RETRIES", prefix)) {
            config.job_limits.max_retries = val.parse().map_err(|_| ConfigError::Invalid("Invalid MAX_RETRIES value".to_string()))?;
        }

        // Timeout settings
        if let Ok(val) = std::env::var(format!("{}JOB_TIMEOUT", prefix)) {
            config.timeout_settings.job_timeout = std::time::Duration::from_secs(val.parse().map_err(|_| ConfigError::Invalid("Invalid JOB_TIMEOUT value".to_string()))?);
        }
        if let Ok(val) = std::env::var(format!("{}BATCH_TIMEOUT", prefix)) {
            config.timeout_settings.batch_timeout = std::time::Duration::from_secs(val.parse().map_err(|_| ConfigError::Invalid("Invalid BATCH_TIMEOUT value".to_string()))?);
        }
        if let Ok(val) = std::env::var(format!("{}CONNECTION_TIMEOUT", prefix)) {
            config.timeout_settings.connection_timeout = std::time::Duration::from_secs(val.parse().map_err(|_| ConfigError::Invalid("Invalid CONNECTION_TIMEOUT value".to_string()))?);
        }

        // Resource control
        if let Ok(val) = std::env::var(format!("{}MAX_MEMORY_MB", prefix)) {
            config.resource_control.max_memory_mb = val.parse().map_err(|_| ConfigError::Invalid("Invalid MAX_MEMORY_MB value".to_string()))?;
        }
        if let Ok(val) = std::env::var(format!("{}MAX_CPU_PERCENT", prefix)) {
            config.resource_control.max_cpu_percent = val.parse().map_err(|_| ConfigError::Invalid("Invalid MAX_CPU_PERCENT value".to_string()))?;
        }
        if let Ok(val) = std::env::var(format!("{}MAX_DISK_MB", prefix)) {
            config.resource_control.max_disk_mb = val.parse().map_err(|_| ConfigError::Invalid("Invalid MAX_DISK_MB value".to_string()))?;
        }
        if let Ok(val) = std::env::var(format!("{}PRIORITY", prefix)) {
            config.resource_control.priority = val.parse().map_err(|_| ConfigError::Invalid("Invalid PRIORITY value".to_string()))?;
        }

        // General settings
        if let Ok(val) = std::env::var(format!("{}NAME", prefix)) {
            config.name = val;
        }
        if let Ok(val) = std::env::var(format!("{}DESCRIPTION", prefix)) {
            config.description = Some(val);
        }

        Ok(config)
    }

    /// Merge another configuration into this one, with `other` taking precedence.
    /// All fields from `other` will override `self`.
    pub fn merge(&mut self, other: &BatchConfig) {
        *self = other.clone();
    }

    /// Create a new configuration by merging `base` with `override_config`.
    pub fn merge_configs(base: &BatchConfig, override_config: &BatchConfig) -> BatchConfig {
        let mut result = base.clone();
        result.merge(override_config);
        result
    }

    /// Load configuration from multiple sources with precedence:
    /// 1. Start with defaults
    /// 2. Override with file config (if file exists)
    /// 3. Override with environment variables (if env_prefix is provided)
    ///
    /// Returns the merged configuration.
    pub fn load_layered<P: AsRef<Path>>(file_path: Option<P>, env_prefix: Option<&str>) -> Result<Self> {
        let mut config = Self::default();

        // Load from file if provided and exists
        if let Some(path) = file_path {
            if path.as_ref().exists() {
                let file_config = Self::load_from_file(path)?;
                config.merge(&file_config);
            }
        }

        // Load from environment if prefix provided
        if let Some(prefix) = env_prefix {
            let env_config = Self::from_env(prefix)?;
            config.merge(&env_config);
        }

        Ok(config)
    }

    pub fn validate(&self) -> Result<()> {
        if self.job_limits.max_jobs == 0 {
            return Err(ConfigError::Invalid("max_jobs must be greater than 0".to_string()));
        }
        if self.job_limits.max_concurrent == 0 {
            return Err(ConfigError::Invalid("max_concurrent must be greater than 0".to_string()));
        }
        if self.job_limits.max_concurrent > self.job_limits.max_jobs {
            return Err(ConfigError::Invalid("max_concurrent cannot be greater than max_jobs".to_string()));
        }
        if self.job_limits.max_retries == 0 {
            return Err(ConfigError::Invalid("max_retries must be greater than 0".to_string()));
        }
        if self.resource_control.max_memory_mb == 0 {
            return Err(ConfigError::Invalid("max_memory_mb must be greater than 0".to_string()));
        }
        if self.resource_control.max_cpu_percent <= 0.0 || self.resource_control.max_cpu_percent > 100.0 {
            return Err(ConfigError::Invalid("max_cpu_percent must be between 0 and 100".to_string()));
        }
        if self.resource_control.max_disk_mb == 0 {
            return Err(ConfigError::Invalid("max_disk_mb must be greater than 0".to_string()));
        }
        if self.timeout_settings.job_timeout.as_secs() == 0 {
            return Err(ConfigError::Invalid("job_timeout must be greater than 0".to_string()));
        }
        if self.timeout_settings.batch_timeout.as_secs() == 0 {
            return Err(ConfigError::Invalid("batch_timeout must be greater than 0".to_string()));
        }
        if self.timeout_settings.connection_timeout.as_secs() == 0 {
            return Err(ConfigError::Invalid("connection_timeout must be greater than 0".to_string()));
        }
        if self.name.trim().is_empty() {
            return Err(ConfigError::Invalid("name cannot be empty".to_string()));
        }
        Ok(())
    }
}

impl Default for BatchConfig {
    fn default() -> Self {
        Self::new("default".to_string())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::time::Duration;
    use tempfile::tempdir;

    #[test]
    fn test_job_limits_default() {
        let limits = JobLimits::default();
        assert_eq!(limits.max_jobs, 1000);
        assert_eq!(limits.max_concurrent, 10);
        assert_eq!(limits.max_retries, 3);
    }

    #[test]
    fn test_timeout_settings_default() {
        let timeouts = TimeoutSettings::default();
        assert_eq!(timeouts.job_timeout, Duration::from_secs(300));
        assert_eq!(timeouts.batch_timeout, Duration::from_secs(3600));
        assert_eq!(timeouts.connection_timeout, Duration::from_secs(30));
    }

    #[test]
    fn test_resource_control_default() {
        let resources = ResourceControl::default();
        assert_eq!(resources.max_memory_mb, 1024);
        assert_eq!(resources.max_cpu_percent, 50.0);
        assert_eq!(resources.max_disk_mb, 10240);
        assert_eq!(resources.priority, 0);
    }

    #[test]
    fn test_batch_config_new() {
        let config = BatchConfig::new("test".to_string());
        assert_eq!(config.name, "test");
        assert!(config.description.is_none());
        assert_eq!(config.job_limits.max_jobs, 1000);
        assert_eq!(config.timeout_settings.job_timeout, Duration::from_secs(300));
        assert_eq!(config.resource_control.max_memory_mb, 1024);
    }

    #[test]
    fn test_batch_config_default() {
        let config = BatchConfig::default();
        assert_eq!(config.name, "default");
        assert!(config.description.is_none());
    }

    #[test]
    fn test_yaml_serialization() {
        let config = BatchConfig::default();
        let yaml = config.to_yaml().unwrap();
        let deserialized = BatchConfig::from_yaml(&yaml).unwrap();
        assert_eq!(deserialized.name, config.name);
        assert_eq!(deserialized.job_limits.max_jobs, config.job_limits.max_jobs);
    }

    #[test]
    fn test_json_serialization() {
        let config = BatchConfig::default();
        let json = config.to_json().unwrap();
        let deserialized = BatchConfig::from_json(&json).unwrap();
        assert_eq!(deserialized.name, config.name);
        assert_eq!(deserialized.job_limits.max_jobs, config.job_limits.max_jobs);
    }

    #[test]
    fn test_validation_valid_config() {
        let config = BatchConfig::default();
        assert!(config.validate().is_ok());
    }

    #[test]
    fn test_validation_invalid_max_jobs() {
        let mut config = BatchConfig::default();
        config.job_limits.max_jobs = 0;
        assert!(config.validate().is_err());
    }

    #[test]
    fn test_validation_invalid_max_concurrent() {
        let mut config = BatchConfig::default();
        config.job_limits.max_concurrent = 0;
        assert!(config.validate().is_err());
    }

    #[test]
    fn test_validation_concurrent_greater_than_jobs() {
        let mut config = BatchConfig::default();
        config.job_limits.max_concurrent = 2000;
        assert!(config.validate().is_err());
    }

    #[test]
    fn test_validation_invalid_max_retries() {
        let mut config = BatchConfig::default();
        config.job_limits.max_retries = 0;
        assert!(config.validate().is_err());
    }

    #[test]
    fn test_validation_invalid_memory() {
        let mut config = BatchConfig::default();
        config.resource_control.max_memory_mb = 0;
        assert!(config.validate().is_err());
    }

    #[test]
    fn test_validation_invalid_cpu_percent() {
        let mut config = BatchConfig::default();
        config.resource_control.max_cpu_percent = 150.0;
        assert!(config.validate().is_err());

        let mut config2 = BatchConfig::default();
        config2.resource_control.max_cpu_percent = 0.0;
        assert!(config2.validate().is_err());
    }

    #[test]
    fn test_validation_invalid_disk() {
        let mut config = BatchConfig::default();
        config.resource_control.max_disk_mb = 0;
        assert!(config.validate().is_err());
    }

    #[test]
    fn test_validation_invalid_timeouts() {
        let mut config = BatchConfig::default();
        config.timeout_settings.job_timeout = Duration::from_secs(0);
        assert!(config.validate().is_err());

        let mut config2 = BatchConfig::default();
        config2.timeout_settings.batch_timeout = Duration::from_secs(0);
        assert!(config2.validate().is_err());

        let mut config3 = BatchConfig::default();
        config3.timeout_settings.connection_timeout = Duration::from_secs(0);
        assert!(config3.validate().is_err());
    }

    #[test]
    fn test_validation_empty_name() {
        let mut config = BatchConfig::default();
        config.name = "".to_string();
        assert!(config.validate().is_err());

        let mut config2 = BatchConfig::default();
        config2.name = "   ".to_string();
        assert!(config2.validate().is_err());
    }

    #[test]
    fn test_file_operations() {
        let dir = tempdir().unwrap();
        let file_path = dir.path().join("config.yaml");

        let config = BatchConfig::default();
        config.save_to_file(&file_path).unwrap();

        let loaded_config = BatchConfig::load_from_file(&file_path).unwrap();
        assert_eq!(loaded_config.name, config.name);
        assert_eq!(loaded_config.job_limits.max_jobs, config.job_limits.max_jobs);
    }

    #[test]
    fn test_load_from_nonexistent_file() {
        let result = BatchConfig::load_from_file("nonexistent.yaml");
        assert!(result.is_err());
    }

    #[test]
    fn test_save_to_invalid_path() {
        let config = BatchConfig::default();
        let result = config.save_to_file("/invalid/path/config.yaml");
        assert!(result.is_err());
    }

    #[test]
    fn test_load_invalid_yaml() {
        let dir = tempdir().unwrap();
        let file_path = dir.path().join("invalid.yaml");

        std::fs::write(&file_path, "invalid: yaml: content: [unclosed").unwrap();

        let result = BatchConfig::load_from_file(&file_path);
        assert!(result.is_err());
    }

    #[test]
    fn test_round_trip_serialization() {
        let dir = tempdir().unwrap();
        let file_path = dir.path().join("round_trip.yaml");

        let original = BatchConfig::default();
        original.save_to_file(&file_path).unwrap();

        let loaded = BatchConfig::load_from_file(&file_path).unwrap();
        loaded.save_to_file(&file_path).unwrap();

        let reloaded = BatchConfig::load_from_file(&file_path).unwrap();
        assert_eq!(reloaded.name, original.name);
        assert_eq!(reloaded.job_limits.max_jobs, original.job_limits.max_jobs);
    }

    #[test]
    fn test_from_env() {
        // Set environment variables
        std::env::set_var("TEST_MAX_JOBS", "200");
        std::env::set_var("TEST_MAX_CONCURRENT", "15");
        std::env::set_var("TEST_MAX_RETRIES", "7");
        std::env::set_var("TEST_JOB_TIMEOUT", "400");
        std::env::set_var("TEST_BATCH_TIMEOUT", "4000");
        std::env::set_var("TEST_CONNECTION_TIMEOUT", "40");
        std::env::set_var("TEST_MAX_MEMORY_MB", "1500");
        std::env::set_var("TEST_MAX_CPU_PERCENT", "80.5");
        std::env::set_var("TEST_MAX_DISK_MB", "25000");
        std::env::set_var("TEST_PRIORITY", "5");
        std::env::set_var("TEST_NAME", "env_config");
        std::env::set_var("TEST_DESCRIPTION", "Config from environment");

        let config = BatchConfig::from_env("TEST_").unwrap();

        assert_eq!(config.job_limits.max_jobs, 200);
        assert_eq!(config.job_limits.max_concurrent, 15);
        assert_eq!(config.job_limits.max_retries, 7);
        assert_eq!(config.timeout_settings.job_timeout, std::time::Duration::from_secs(400));
        assert_eq!(config.timeout_settings.batch_timeout, std::time::Duration::from_secs(4000));
        assert_eq!(config.timeout_settings.connection_timeout, std::time::Duration::from_secs(40));
        assert_eq!(config.resource_control.max_memory_mb, 1500);
        assert_eq!(config.resource_control.max_cpu_percent, 80.5);
        assert_eq!(config.resource_control.max_disk_mb, 25000);
        assert_eq!(config.resource_control.priority, 5);
        assert_eq!(config.name, "env_config");
        assert_eq!(config.description, Some("Config from environment".to_string()));

        // Clean up environment variables
        std::env::remove_var("TEST_MAX_JOBS");
        std::env::remove_var("TEST_MAX_CONCURRENT");
        std::env::remove_var("TEST_MAX_RETRIES");
        std::env::remove_var("TEST_JOB_TIMEOUT");
        std::env::remove_var("TEST_BATCH_TIMEOUT");
        std::env::remove_var("TEST_CONNECTION_TIMEOUT");
        std::env::remove_var("TEST_MAX_MEMORY_MB");
        std::env::remove_var("TEST_MAX_CPU_PERCENT");
        std::env::remove_var("TEST_MAX_DISK_MB");
        std::env::remove_var("TEST_PRIORITY");
        std::env::remove_var("TEST_NAME");
        std::env::remove_var("TEST_DESCRIPTION");
    }

    #[test]
    fn test_from_env_partial() {
        // Set only some environment variables
        std::env::set_var("PARTIAL_MAX_JOBS", "300");
        std::env::set_var("PARTIAL_NAME", "partial_config");

        let config = BatchConfig::from_env("PARTIAL_").unwrap();

        // Should have custom values where set
        assert_eq!(config.job_limits.max_jobs, 300);
        assert_eq!(config.name, "partial_config");

        // Should have defaults for unset values
        assert_eq!(config.job_limits.max_concurrent, 10); // default
        assert_eq!(config.timeout_settings.job_timeout, std::time::Duration::from_secs(300)); // default

        // Clean up
        std::env::remove_var("PARTIAL_MAX_JOBS");
        std::env::remove_var("PARTIAL_NAME");
    }

    #[test]
    fn test_from_env_invalid_values() {
        std::env::set_var("INVALID_MAX_JOBS", "not_a_number");

        let result = BatchConfig::from_env("INVALID_");
        assert!(result.is_err());

        std::env::remove_var("INVALID_MAX_JOBS");
    }

    #[test]
    fn test_merge_configs() {
        let mut base_config = BatchConfig::new("base".to_string());
        base_config.job_limits.max_jobs = 100;
        base_config.timeout_settings.job_timeout = std::time::Duration::from_secs(200);

        let mut override_config = BatchConfig::new("override".to_string());
        override_config.job_limits.max_concurrent = 20;
        override_config.resource_control.max_memory_mb = 2000;
        override_config.description = Some("Override description".to_string());

        base_config.merge(&override_config);

        // Merge fully replaces, so base becomes identical to override_config
        assert_eq!(base_config.name, "override");
        assert_eq!(base_config.job_limits.max_jobs, 1000); // default
        assert_eq!(base_config.job_limits.max_concurrent, 20);
        assert_eq!(base_config.timeout_settings.job_timeout, std::time::Duration::from_secs(300)); // default
        assert_eq!(base_config.resource_control.max_memory_mb, 2000);
        assert_eq!(base_config.description, Some("Override description".to_string()));
    }

    #[test]
    fn test_merge_configs_static() {
        let mut base = BatchConfig::default();
        base.job_limits.max_jobs = 50;

        let mut override_config = BatchConfig::default();
        override_config.job_limits.max_concurrent = 25;

        let merged = BatchConfig::merge_configs(&base, &override_config);

        // Merge fully replaces with override_config
        assert_eq!(merged.job_limits.max_jobs, 1000); // default
        assert_eq!(merged.job_limits.max_concurrent, 25); // from override
    }

    #[test]
    fn test_merge_full_replacement() {
        let mut base = BatchConfig::default();
        base.job_limits.max_jobs = 100;

        let override_config = BatchConfig::default(); // All defaults

        base.merge(&override_config);

        // Merge fully replaces, so base becomes override_config
        assert_eq!(base.job_limits.max_jobs, 1000);
        assert_eq!(base.job_limits.max_concurrent, 10);
    }

    #[test]
    fn test_custom_config_values() {
        let mut config = BatchConfig::new("custom".to_string());
        config.description = Some("Custom batch config".to_string());
        config.job_limits.max_jobs = 500;
        config.job_limits.max_concurrent = 5;
        config.job_limits.max_retries = 5;
        config.timeout_settings.job_timeout = Duration::from_secs(600);
        config.timeout_settings.batch_timeout = Duration::from_secs(7200);
        config.timeout_settings.connection_timeout = Duration::from_secs(60);
        config.resource_control.max_memory_mb = 2048;
        config.resource_control.max_cpu_percent = 75.0;
        config.resource_control.max_disk_mb = 20480;
        config.resource_control.priority = 10;

        assert!(config.validate().is_ok());
        assert_eq!(config.name, "custom");
        assert_eq!(config.description.as_ref().unwrap(), "Custom batch config");
        assert_eq!(config.job_limits.max_jobs, 500);
        assert_eq!(config.job_limits.max_concurrent, 5);
        assert_eq!(config.job_limits.max_retries, 5);
        assert_eq!(config.timeout_settings.job_timeout, Duration::from_secs(600));
        assert_eq!(config.timeout_settings.batch_timeout, Duration::from_secs(7200));
        assert_eq!(config.timeout_settings.connection_timeout, Duration::from_secs(60));
        assert_eq!(config.resource_control.max_memory_mb, 2048);
        assert_eq!(config.resource_control.max_cpu_percent, 75.0);
        assert_eq!(config.resource_control.max_disk_mb, 20480);
        assert_eq!(config.resource_control.priority, 10);
    }

    #[test]
    fn test_load_layered_defaults_only() {
        let config = BatchConfig::load_layered::<&str>(None, None).unwrap();
        assert_eq!(config.name, "default");
        assert_eq!(config.job_limits.max_jobs, 1000);
    }

    #[test]
    fn test_load_layered_with_file() {
        let dir = tempdir().unwrap();
        let file_path = dir.path().join("test_config.yaml");

        let mut file_config = BatchConfig::new("file_config".to_string());
        file_config.job_limits.max_jobs = 500;
        file_config.save_to_file(&file_path).unwrap();

        let config = BatchConfig::load_layered(Some(&file_path), None).unwrap();

        assert_eq!(config.name, "file_config");
        assert_eq!(config.job_limits.max_jobs, 500);
        // Other fields should have defaults
        assert_eq!(config.job_limits.max_concurrent, 10);
    }

    #[test]
    fn test_load_layered_with_env() {
        std::env::set_var("LAYER_MAX_JOBS", "300");
        std::env::set_var("LAYER_NAME", "env_config");

        let config = BatchConfig::load_layered::<&str>(None, Some("LAYER_")).unwrap();

        assert_eq!(config.name, "env_config");
        assert_eq!(config.job_limits.max_jobs, 300);

        std::env::remove_var("LAYER_MAX_JOBS");
        std::env::remove_var("LAYER_NAME");
    }

    #[test]
    fn test_load_layered_file_and_env() {
        let dir = tempdir().unwrap();
        let file_path = dir.path().join("layered_config.yaml");

        let mut file_config = BatchConfig::new("file_config".to_string());
        file_config.job_limits.max_jobs = 200;
        file_config.job_limits.max_concurrent = 5;
        file_config.save_to_file(&file_path).unwrap();

        // Env should override file
        std::env::set_var("FULL_MAX_JOBS", "400");
        std::env::set_var("FULL_NAME", "env_override");

        let config = BatchConfig::load_layered(Some(&file_path), Some("FULL_")).unwrap();

        // Env should take precedence
        assert_eq!(config.name, "env_override");
        assert_eq!(config.job_limits.max_jobs, 400);
        // File values for non-overridden fields
        assert_eq!(config.job_limits.max_concurrent, 5);

        std::env::remove_var("FULL_MAX_JOBS");
        std::env::remove_var("FULL_NAME");
    }
}
