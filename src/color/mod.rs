pub mod serializable;
pub mod serde;
pub mod utils;

pub use serializable::SerializableColor;
pub use serde::color_serde;
pub use utils::hex_to_rgb;

#[cfg(test)]
mod tests {
    use super::*;
    use ratatui::style::Color;

    #[test]
    fn test_hex_to_rgb() {
        assert_eq!(hex_to_rgb("#000000"), Some(Color::Rgb(0, 0, 0)));
        assert_eq!(hex_to_rgb("#ffffff"), Some(Color::Rgb(255, 255, 255)));
        assert_eq!(hex_to_rgb("#3b82f6"), Some(Color::Rgb(59, 130, 246)));
        assert_eq!(hex_to_rgb("invalid"), None);
    }

    #[test]
    fn test_color_conversion() {
        let color = Color::Rgb(255, 128, 64);
        let serializable = SerializableColor::from(color);
        assert_eq!(serializable.r, 255);
        assert_eq!(serializable.g, 128);
        assert_eq!(serializable.b, 64);

        let back_to_color = Color::from(serializable);
        assert_eq!(back_to_color, color);
    }

    #[test]
    fn test_named_color_conversion() {
        let color = Color::Red;
        let serializable = SerializableColor::from(color);
        assert_eq!(serializable.r, 255);
        assert_eq!(serializable.g, 0);
        assert_eq!(serializable.b, 0);
    }
}