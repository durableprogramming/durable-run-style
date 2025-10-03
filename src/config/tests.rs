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