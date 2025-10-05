use super::*;
use tempfile::tempdir;

#[test]
fn test_config_defaults() {
    let config = Config::default();
    assert_eq!(config.app.layout.sidebar_width, 30);
    assert_eq!(config.app.output.max_output_lines, 1000);
    assert!(config.app.animation.animation_enabled);
    assert!(!config.app.animation.no_animate);
    assert!(matches!(config.theme.primary, ratatui::style::Color::Rgb(_, _, _)));
}

#[test]
fn test_config_serialization() {
    let config = Config::default();
    let toml = toml::to_string(&config).unwrap();
    let deserialized: Config = toml::from_str(&toml).unwrap();
    assert_eq!(deserialized.app.layout.sidebar_width, config.app.layout.sidebar_width);
    assert_eq!(deserialized.app.output.max_output_lines, config.app.output.max_output_lines);
    assert_eq!(deserialized.app.animation.animation_enabled, config.app.animation.animation_enabled);
    assert_eq!(deserialized.app.animation.no_animate, config.app.animation.no_animate);
}

#[test]
fn test_config_file_operations() {
    let dir = tempdir().unwrap();
    let file_path = dir.path().join("config.toml");

    let config = Config::default();
    config.save_to_file(&file_path).unwrap();

    let loaded_config = Config::load_from_file(&file_path).unwrap();
    assert_eq!(loaded_config.app.layout.sidebar_width, config.app.layout.sidebar_width);
    assert_eq!(loaded_config.app.output.max_output_lines, config.app.output.max_output_lines);
    assert_eq!(loaded_config.app.animation.animation_enabled, config.app.animation.animation_enabled);
    assert_eq!(loaded_config.app.animation.no_animate, config.app.animation.no_animate);
}

#[test]
fn test_config_with_custom_values() {
    let mut config = Config::default();
    config.app.layout.sidebar_width = 50;
    config.app.output.max_output_lines = 2000;
    config.app.animation.animation_enabled = false;
    config.app.animation.no_animate = true;

    assert_eq!(config.app.layout.sidebar_width, 50);
    assert_eq!(config.app.output.max_output_lines, 2000);
    assert!(!config.app.animation.animation_enabled);
    assert!(config.app.animation.no_animate);
}

#[test]
fn test_config_serialization_with_custom_values() {
    let mut config = Config::default();
    config.app.layout.sidebar_width = 40;
    config.app.output.max_output_lines = 1500;

    let toml = toml::to_string(&config).unwrap();
    let deserialized: Config = toml::from_str(&toml).unwrap();

    assert_eq!(deserialized.app.layout.sidebar_width, 40);
    assert_eq!(deserialized.app.output.max_output_lines, 1500);
}

#[test]
fn test_config_load_from_nonexistent_file() {
    let result = Config::load_from_file("nonexistent_config.toml");
    assert!(result.is_err());
}

#[test]
fn test_config_load_invalid_toml() {
    let dir = tempdir().unwrap();
    let file_path = dir.path().join("invalid.toml");

    std::fs::write(&file_path, "invalid [toml content").unwrap();

    let result = Config::load_from_file(&file_path);
    assert!(result.is_err());
}

#[test]
fn test_config_save_to_invalid_path() {
    let config = Config::default();
    let result = config.save_to_file("/invalid/path/config.toml");
    assert!(result.is_err());
}

#[test]
fn test_config_round_trip_with_modifications() {
    let dir = tempdir().unwrap();
    let file_path = dir.path().join("modified_config.toml");

    let mut original = Config::default();
    original.app.layout.sidebar_width = 35;
    original.app.animation.animation_enabled = false;

    original.save_to_file(&file_path).unwrap();
    let loaded = Config::load_from_file(&file_path).unwrap();

    assert_eq!(loaded.app.layout.sidebar_width, 35);
    assert!(!loaded.app.animation.animation_enabled);
}

#[test]
fn test_config_clone_equality() {
    let config = Config::default();
    let cloned = config.clone();

    assert_eq!(config.app.layout.sidebar_width, cloned.app.layout.sidebar_width);
    assert_eq!(config.app.output.max_output_lines, cloned.app.output.max_output_lines);
    assert_eq!(config.app.animation.animation_enabled, cloned.app.animation.animation_enabled);
}

#[test]
fn test_config_with_defaults_method() {
    let config1 = Config::with_defaults();
    let config2 = Config::default();

    assert_eq!(config1.app.layout.sidebar_width, config2.app.layout.sidebar_width);
    assert_eq!(config1.app.output.max_output_lines, config2.app.output.max_output_lines);
    }

    #[test]
    fn test_save_and_load_config() {
        let dir = tempdir().unwrap();
        let file_path = dir.path().join("test_config.toml");

        let config = Config::default();
        config.save_to_file(&file_path).unwrap();

        let loaded_config = Config::load_from_file(&file_path).unwrap();
        assert_eq!(loaded_config.app.layout.sidebar_width, config.app.layout.sidebar_width);
    }

    #[test]
    fn test_load_from_nonexistent_file() {
        let result = Config::load_from_file("nonexistent.toml");
        assert!(result.is_err());
    }

    #[test]
    fn test_load_invalid_toml() {
        let dir = tempdir().unwrap();
        let file_path = dir.path().join("invalid.toml");

        std::fs::write(&file_path, "invalid toml content [unclosed").unwrap();

        let result = Config::load_from_file(&file_path);
        assert!(result.is_err());
    }

    #[test]
    fn test_save_to_invalid_path() {
        let config = Config::default();
        let result = config.save_to_file("/invalid/path/config.toml");
        assert!(result.is_err());
    }

    #[test]
    fn test_round_trip_serialization() {
        let dir = tempdir().unwrap();
        let file_path = dir.path().join("round_trip.toml");

        let original = Config::default();
        original.save_to_file(&file_path).unwrap();

        let loaded = Config::load_from_file(&file_path).unwrap();
        loaded.save_to_file(&file_path).unwrap();

        let reloaded = Config::load_from_file(&file_path).unwrap();
        assert_eq!(reloaded.app.layout.sidebar_width, original.app.layout.sidebar_width);
    }