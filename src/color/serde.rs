use super::SerializableColor;
use ratatui::style::Color;
use serde::{Deserialize, Deserializer, Serialize, Serializer};

pub mod color_serde {
    use super::*;

    pub fn serialize<S>(color: &Color, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let serializable = SerializableColor::from(*color);
        serializable.serialize(serializer)
    }

    pub fn deserialize<'de, D>(deserializer: D) -> Result<Color, D::Error>
    where
        D: Deserializer<'de>,
    {
        let serializable = SerializableColor::deserialize(deserializer)?;
        Ok(Color::from(serializable))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use ratatui::style::Color;

    #[test]
    fn test_color_serde_round_trip() {
        let original = Color::Rgb(123, 200, 150);
        let serializable = SerializableColor::from(original);
        let back_to_color = Color::from(serializable);

        assert_eq!(back_to_color, original);
    }

    #[test]
    fn test_named_color_serde() {
        let original = Color::Red;
        let serializable = SerializableColor::from(original);
        let back_to_color = Color::from(serializable.clone());

        assert_eq!(back_to_color, Color::Rgb(255, 0, 0));
        assert_eq!(serializable.r, 255);
        assert_eq!(serializable.g, 0);
        assert_eq!(serializable.b, 0);
    }

    #[test]
    fn test_all_named_colors() {
        let test_cases = vec![
            (Color::Red, (255, 0, 0)),
            (Color::Green, (0, 255, 0)),
            (Color::Blue, (0, 0, 255)),
            (Color::Yellow, (255, 255, 0)),
            (Color::Magenta, (255, 0, 255)),
            (Color::Cyan, (0, 255, 255)),
            (Color::White, (255, 255, 255)),
            (Color::Black, (0, 0, 0)),
            (Color::Gray, (128, 128, 128)),
        ];

        for (color, expected_rgb) in test_cases {
            let serializable = SerializableColor::from(color);
            assert_eq!(serializable.r, expected_rgb.0);
            assert_eq!(serializable.g, expected_rgb.1);
            assert_eq!(serializable.b, expected_rgb.2);

            let back_to_color = Color::from(serializable);
            assert_eq!(back_to_color, Color::Rgb(expected_rgb.0, expected_rgb.1, expected_rgb.2));
        }
    }
}