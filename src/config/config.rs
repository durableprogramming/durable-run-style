use crate::app_config::AppConfig;
use crate::theme::Theme;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct Config {
    pub app: AppConfig,
    pub theme: Theme,
}

impl Config {
    pub fn with_defaults() -> Self {
        Self::default()
    }
}