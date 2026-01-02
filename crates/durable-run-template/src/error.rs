//! Error types for the template system

use thiserror::Error;

/// Result type for template operations
pub type Result<T> = std::result::Result<T, TemplateError>;

/// Errors that can occur during template processing
#[derive(Error, Debug)]
pub enum TemplateError {
    #[error("Parse error: {message}")]
    Parse { message: String },

    #[error("Lexing error: {message}")]
    Lexing { message: String },

    #[error("Variable not found: {name}")]
    VariableNotFound { name: String },

    #[error("Argument not found: {index}")]
    ArgumentNotFound { index: usize },

    #[error("Macro not found: {name}")]
    MacroNotFound { name: String },

    #[error("Macro execution failed: {message}")]
    MacroExecution { message: String },

    #[error("Circular macro dependency detected involving '{macro_name}'")]
    CircularDependency { macro_name: String },

    #[error("Invalid range syntax: {range}")]
    InvalidRange { range: String },

    #[error("Macro '{name}' already exists")]
    MacroAlreadyExists { name: String },

    #[error("Regex error: {0}")]
    Regex(#[from] regex::Error),

    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),

    #[error("Serde error: {0}")]
    Serde(#[from] serde_json::Error),

    #[error("Anyhow error: {0}")]
    Anyhow(#[from] anyhow::Error),
}