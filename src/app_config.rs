use serde::{Deserialize, Serialize};

pub use crate::ui_layout::LayoutConfig;
pub use crate::ui_output::OutputConfig;
pub use crate::ui_animation::AnimationConfig;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AppConfig {
    #[serde(flatten)]
    pub layout: LayoutConfig,
    #[serde(flatten)]
    pub output: OutputConfig,
    #[serde(flatten)]
    pub animation: AnimationConfig,
}

impl Default for AppConfig {
    fn default() -> Self {
        AppConfig {
            layout: LayoutConfig::default(),
            output: OutputConfig::default(),
            animation: AnimationConfig::default(),
        }
    }
}

impl AppConfig {
    // Convenience methods
    pub fn sidebar_width(&self) -> u16 {
        self.layout.sidebar_width
    }

    pub fn max_output_lines(&self) -> usize {
        self.output.max_output_lines
    }

    pub fn animation_enabled(&self) -> bool {
        self.animation.animation_enabled
    }

    pub fn no_animate(&self) -> bool {
        self.animation.no_animate
    }

    pub fn max_command_lines(&self) -> usize {
        self.layout.max_command_lines
    }

    pub fn shine_amplitude(&self) -> f32 {
        self.animation.shine_amplitude
    }

    pub fn shine_frequency(&self) -> f32 {
        self.animation.shine_frequency
    }

    pub fn shine_base_intensity(&self) -> f32 {
        self.animation.shine_base_intensity
    }

    pub fn shine_angle_start(&self) -> f32 {
        self.animation.shine_angle_start
    }

    pub fn shine_angle_end(&self) -> f32 {
        self.animation.shine_angle_end
    }

    pub fn shine_width_start(&self) -> f32 {
        self.animation.shine_width_start
    }

    pub fn shine_width_end(&self) -> f32 {
        self.animation.shine_width_end
    }

    pub fn shine_width_quarterpoint(&self) -> f32 {
        self.animation.shine_width_quarterpoint
    }

    pub fn shine_width_midpoint(&self) -> f32 {
        self.animation.shine_width_midpoint
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_app_config_defaults() {
        let config = AppConfig::default();
        assert_eq!(config.sidebar_width(), 30);
        assert_eq!(config.max_output_lines(), 1000);
        assert!(config.animation_enabled());
        assert!(!config.no_animate());
    }

    #[test]
    fn test_app_config_composition() {
        let config = AppConfig::default();
        assert_eq!(config.layout.sidebar_width, 30);
        assert_eq!(config.output.max_output_lines, 1000);
        assert!(config.animation.animation_enabled);
        assert!(!config.animation.no_animate);
    }
}