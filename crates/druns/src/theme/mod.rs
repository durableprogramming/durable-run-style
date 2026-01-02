use crate::color::color_serde;
use ratatui::style::Color;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Theme {
    #[serde(with = "color_serde")]
    pub primary: Color,
    #[serde(with = "color_serde")]
    pub secondary: Color,
    #[serde(with = "color_serde")]
    pub accent: Color,
    #[serde(with = "color_serde")]
    pub border: Color,
    #[serde(with = "color_serde")]
    pub text: Color,
    #[serde(with = "color_serde")]
    pub background: Color,
    #[serde(skip)]
    pub shades: Vec<Color>,
}

impl Default for Theme {
    fn default() -> Self {
        crate::theme::default::default_theme()
    }
}

pub mod default;
