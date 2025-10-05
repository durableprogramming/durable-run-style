use serde::{Deserialize, Serialize};

pub use crate::ui_layout::LayoutConfig;
pub use crate::ui_output::OutputConfig;
pub use crate::ui_animation::AnimationConfig;

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct AppConfig {
    #[serde(flatten)]
    pub layout: LayoutConfig,
    #[serde(flatten)]
    pub output: OutputConfig,
    #[serde(flatten)]
    pub animation: AnimationConfig,
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

    #[test]
    fn test_app_config_serialization() {
        let config = AppConfig::default();
        let serialized = toml::to_string(&config).unwrap();
        let deserialized: AppConfig = toml::from_str(&serialized).unwrap();
        assert_eq!(deserialized.sidebar_width(), config.sidebar_width());
        assert_eq!(deserialized.max_output_lines(), config.max_output_lines());
    }

    #[test]
    fn test_app_config_clone() {
        let config = AppConfig::default();
        let cloned = config.clone();
        assert_eq!(cloned.sidebar_width(), config.sidebar_width());
        assert_eq!(cloned.max_output_lines(), config.max_output_lines());
    }

    #[test]
    fn test_app_config_convenience_methods() {
        let config = AppConfig::default();
        assert_eq!(config.sidebar_width(), config.layout.sidebar_width);
        assert_eq!(config.max_output_lines(), config.output.max_output_lines);
        assert_eq!(config.animation_enabled(), config.animation.animation_enabled);
        assert_eq!(config.no_animate(), config.animation.no_animate);
        assert_eq!(config.max_command_lines(), config.layout.max_command_lines);
    }

    #[test]
    fn test_app_config_animation_properties() {
        let config = AppConfig::default();
        // Test that animation properties are accessible
        let _amplitude = config.shine_amplitude();
        let _frequency = config.shine_frequency();
        let _base_intensity = config.shine_base_intensity();
        let _angle_start = config.shine_angle_start();
        let _angle_end = config.shine_angle_end();
        let _width_start = config.shine_width_start();
        let _width_end = config.shine_width_end();
        let _width_quarter = config.shine_width_quarterpoint();
        let _width_mid = config.shine_width_midpoint();
        // Values depend on AnimationConfig defaults
    }

    #[test]
    fn test_app_config_debug() {
        let config = AppConfig::default();
        let debug_str = format!("{config:?}");
        assert!(debug_str.contains("AppConfig"));
        assert!(debug_str.contains("layout:"));
        assert!(debug_str.contains("output:"));
        assert!(debug_str.contains("animation:"));
    }
}