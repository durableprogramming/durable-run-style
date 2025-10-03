#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct AnimationConfig {
    pub animation_enabled: bool,
    #[serde(default)]
    pub no_animate: bool,
}

impl Default for AnimationConfig {
    fn default() -> Self {
        AnimationConfig {
            animation_enabled: true,
            no_animate: false,
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
    }

    #[test]
    fn test_no_animate_config() {
        let mut config = AnimationConfig::default();
        config.no_animate = true;
        assert!(config.animation_enabled); // animation_enabled is still true in the struct, logic is in main.rs
        assert!(config.no_animate);
    }
}