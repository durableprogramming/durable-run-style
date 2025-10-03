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