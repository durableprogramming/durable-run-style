//! Error types for the durable-run-parallel crate

use std::fmt;

/// Errors that can occur during parallel command execution
#[derive(Debug)]
pub enum ParallelError {
    /// Command execution failed
    CommandFailed {
        command: String,
        exit_code: Option<i32>,
        stderr: String,
    },
    /// IO error during command execution
    Io(std::io::Error),
    /// UTF-8 decoding error
    Utf8(std::string::FromUtf8Error),
    /// Template parsing/rendering error
    Template(Box<dyn std::error::Error + Send + Sync>),
    /// Input processing error
    Input(Box<dyn std::error::Error + Send + Sync>),
}

impl fmt::Display for ParallelError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ParallelError::CommandFailed { command, exit_code, stderr } => {
                write!(f, "Command '{}' failed", command)?;
                if let Some(code) = exit_code {
                    write!(f, " with exit code {}", code)?;
                }
                if !stderr.is_empty() {
                    write!(f, ": {}", stderr.trim())?;
                }
                Ok(())
            }
            ParallelError::Io(err) => write!(f, "IO error: {}", err),
            ParallelError::Utf8(err) => write!(f, "UTF-8 decoding error: {}", err),
            ParallelError::Template(err) => write!(f, "Template error: {}", err),
            ParallelError::Input(err) => write!(f, "Input processing error: {}", err),
        }
    }
}

impl std::error::Error for ParallelError {}

impl From<std::io::Error> for ParallelError {
    fn from(err: std::io::Error) -> Self {
        ParallelError::Io(err)
    }
}

impl From<std::string::FromUtf8Error> for ParallelError {
    fn from(err: std::string::FromUtf8Error) -> Self {
        ParallelError::Utf8(err)
    }
}

/// Result type alias for parallel operations
pub type Result<T> = std::result::Result<T, ParallelError>;