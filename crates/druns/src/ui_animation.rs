fn default_shine_amplitude() -> f32 {
    0.2
}

fn default_shine_frequency() -> f32 {
    3.0
}

fn default_shine_base_intensity() -> f32 {
    0.4
}

fn default_shine_angle_start() -> f32 {
    25.0
}

fn default_shine_angle_end() -> f32 {
    45.0
}

fn default_shine_width_start() -> f32 {
    5.0
}

fn default_shine_width_end() -> f32 {
    5.0
}

fn default_shine_width_midpoint() -> f32 {
    3.0
}

fn default_shine_width_quarterpoint() -> f32 {
    2.0
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct AnimationConfig {
    pub animation_enabled: bool,
    #[serde(default)]
    pub no_animate: bool,
    #[serde(default = "default_shine_amplitude")]
    pub shine_amplitude: f32,
    #[serde(default = "default_shine_frequency")]
    pub shine_frequency: f32,
    #[serde(default = "default_shine_base_intensity")]
    pub shine_base_intensity: f32,
    #[serde(default = "default_shine_angle_start")]
    pub shine_angle_start: f32,
    #[serde(default = "default_shine_angle_end")]
    pub shine_angle_end: f32,
    #[serde(default = "default_shine_width_start")]
    pub shine_width_start: f32,
    #[serde(default = "default_shine_width_end")]
    pub shine_width_end: f32,
    #[serde(default = "default_shine_width_quarterpoint")]
    pub shine_width_quarterpoint: f32,
    #[serde(default = "default_shine_width_midpoint")]
    pub shine_width_midpoint: f32,
}

impl Default for AnimationConfig {
    fn default() -> Self {
        AnimationConfig {
            animation_enabled: true,
            no_animate: false,
            shine_amplitude: default_shine_amplitude(),
            shine_frequency: default_shine_frequency(),
            shine_base_intensity: default_shine_base_intensity(),
            shine_angle_start: default_shine_angle_start(),
            shine_angle_end: default_shine_angle_end(),
            shine_width_start: default_shine_width_start(),
            shine_width_end: default_shine_width_end(),
            shine_width_quarterpoint: default_shine_width_quarterpoint(),
            shine_width_midpoint: default_shine_width_midpoint(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_animation_config_default() {
        let config = AnimationConfig::default();
        assert!(config.animation_enabled);
        assert!(!config.no_animate);
        assert_eq!(config.shine_angle_start, 25.0);
        assert_eq!(config.shine_angle_end, 45.0);
    }

    #[test]
    fn test_no_animate_config() {
        let mut config = AnimationConfig::default();
        config.no_animate = true;
        assert!(config.animation_enabled); // animation_enabled is still true in the struct, logic is in main.rs
        assert!(config.no_animate);
    }
}
