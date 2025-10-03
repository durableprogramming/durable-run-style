use crate::theme::Theme;
use crate::color::hex_to_rgb;
use durable_color_resolver::{resolve_colors, ColorDefinitions};
use ratatui::style::Color;
use std::collections::HashMap;

pub fn default_theme() -> Theme {
    let mut map = HashMap::new();
    map.insert("base".to_string(), "blue-500".to_string());
    map.insert("primary".to_string(), "base".to_string());
    map.insert("secondary".to_string(), "darken(base, 20%)".to_string());
    map.insert("accent".to_string(), "lighten(base, 10%)".to_string());
    map.insert("border".to_string(), "darken(base, 40%)".to_string());
    map.insert("text".to_string(), "lighten(base, 80%)".to_string());
    map.insert("background".to_string(), "darken(base, 80%)".to_string());
    map.insert("shade1".to_string(), "lighten(base, 60%)".to_string());
    map.insert("shade2".to_string(), "lighten(base, 50%)".to_string());
    map.insert("shade3".to_string(), "lighten(base, 40%)".to_string());
    map.insert("shade4".to_string(), "lighten(base, 30%)".to_string());
    map.insert("shade5".to_string(), "lighten(base, 20%)".to_string());
    map.insert("shade6".to_string(), "lighten(base, 10%)".to_string());
    map.insert("shade7".to_string(), "base".to_string());
    map.insert("shade8".to_string(), "darken(base, 10%)".to_string());

    let definitions = ColorDefinitions(map);
    let resolved = resolve_colors(definitions);

    Theme {
        primary: hex_to_rgb(&resolved.0["primary"]).unwrap_or(Color::Rgb(59, 130, 246)),
        secondary: hex_to_rgb(&resolved.0["secondary"]).unwrap_or(Color::Rgb(30, 64, 175)),
        accent: hex_to_rgb(&resolved.0["accent"]).unwrap_or(Color::Rgb(96, 165, 250)),
        border: hex_to_rgb(&resolved.0["border"]).unwrap_or(Color::Rgb(17, 24, 39)),
        text: hex_to_rgb(&resolved.0["text"]).unwrap_or(Color::Rgb(255, 255, 255)),
        background: hex_to_rgb(&resolved.0["background"]).unwrap_or(Color::Rgb(0, 0, 0)),
        shades: vec![
            hex_to_rgb(&resolved.0["shade1"]).unwrap_or(Color::Rgb(120, 180, 255)),
            hex_to_rgb(&resolved.0["shade2"]).unwrap_or(Color::Rgb(100, 160, 255)),
            hex_to_rgb(&resolved.0["shade3"]).unwrap_or(Color::Rgb(80, 140, 255)),
            hex_to_rgb(&resolved.0["shade4"]).unwrap_or(Color::Rgb(60, 120, 255)),
            hex_to_rgb(&resolved.0["shade5"]).unwrap_or(Color::Rgb(40, 100, 255)),
            hex_to_rgb(&resolved.0["shade6"]).unwrap_or(Color::Rgb(20, 80, 255)),
            hex_to_rgb(&resolved.0["shade7"]).unwrap_or(Color::Rgb(59, 130, 246)),
            hex_to_rgb(&resolved.0["shade8"]).unwrap_or(Color::Rgb(30, 64, 175)),
        ],
    }
}