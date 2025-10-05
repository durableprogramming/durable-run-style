#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct LayoutConfig {
    pub sidebar_width: u16,
    pub max_command_lines: usize,
}

impl Default for LayoutConfig {
    fn default() -> Self {
        LayoutConfig {
            sidebar_width: 30,
            max_command_lines: 3,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_layout_config_default() {
        let config = LayoutConfig::default();
        assert_eq!(config.sidebar_width, 30);
        assert_eq!(config.max_command_lines, 3);
    }
}