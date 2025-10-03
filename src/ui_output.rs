#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct OutputConfig {
    pub max_output_lines: usize,
}

impl Default for OutputConfig {
    fn default() -> Self {
        OutputConfig {
            max_output_lines: 1000,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_output_config_default() {
        let config = OutputConfig::default();
        assert_eq!(config.max_output_lines, 1000);
    }
}