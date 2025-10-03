use ratatui::style::Color;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SerializableColor {
    pub r: u8,
    pub g: u8,
    pub b: u8,
}

impl From<Color> for SerializableColor {
    fn from(color: Color) -> Self {
        match color {
            Color::Rgb(r, g, b) => SerializableColor { r, g, b },
            Color::Red => SerializableColor { r: 255, g: 0, b: 0 },
            Color::Green => SerializableColor { r: 0, g: 255, b: 0 },
            Color::Yellow => SerializableColor { r: 255, g: 255, b: 0 },
            Color::Blue => SerializableColor { r: 0, g: 0, b: 255 },
            Color::Magenta => SerializableColor { r: 255, g: 0, b: 255 },
            Color::Cyan => SerializableColor { r: 0, g: 255, b: 255 },
            Color::Gray => SerializableColor { r: 128, g: 128, b: 128 },
            Color::DarkGray => SerializableColor { r: 64, g: 64, b: 64 },
            Color::White => SerializableColor { r: 255, g: 255, b: 255 },
            Color::Black => SerializableColor { r: 0, g: 0, b: 0 },
            Color::LightRed => SerializableColor { r: 255, g: 128, b: 128 },
            Color::LightGreen => SerializableColor { r: 128, g: 255, b: 128 },
            Color::LightYellow => SerializableColor { r: 255, g: 255, b: 128 },
            Color::LightBlue => SerializableColor { r: 128, g: 128, b: 255 },
            Color::LightMagenta => SerializableColor { r: 255, g: 128, b: 255 },
            Color::LightCyan => SerializableColor { r: 128, g: 255, b: 255 },
            Color::Indexed(_) => SerializableColor { r: 128, g: 128, b: 128 }, // Default to gray
            Color::Reset => SerializableColor { r: 0, g: 0, b: 0 },
        }
    }
}

impl From<SerializableColor> for Color {
    fn from(color: SerializableColor) -> Self {
        Color::Rgb(color.r, color.g, color.b)
    }
}