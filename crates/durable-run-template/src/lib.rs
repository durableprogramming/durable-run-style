//! # Durable Run Template
//!
//! A powerful template system for dynamic command generation, inspired by tools like GNU Parallel, axe, and rargs.
//!
//! This crate provides:
//! - Template syntax for variable substitution
//! - User-defined macro system
//! - Argument resolution logic
//! - Dynamic command generation
//!
//! ## Features
//!
//! - **Template Syntax**: Supports placeholders like `{name}`, `{0}`, `{name.default}`, `{0..2:sep}`
//! - **Variable Substitution**: Variables from context, environment, or defaults
//! - **Argument Resolution**: Positional arguments with range support
//! - **Macro System**: User-defined macros for reusable templates
//! - **Performance**: Template caching and efficient parsing
//!
//! ## Template Syntax
//!
//! - `{name}` - Variable substitution
//! - `{0}` - Positional argument (0-indexed)
//! - `{name.default}` - Variable with default value
//! - `{0..2}` - Range of arguments
//! - `{0..2:,}` - Range with custom separator
//! - `{macro_name(arg1, arg2)}` - Macro expansion
//!
//! ## Example
//!
//! ```rust
//! use durable_run_template::{TemplateEngine, Context};
//!
//! let mut engine = TemplateEngine::new();
//! let template = engine.parse("echo {name} {0}").unwrap();
//! let mut context = Context::new()
//!     .with_var("name", "hello")
//!     .with_arg(0, "world");
//!
//! let command = template.render(&mut context).unwrap();
//! assert_eq!(command, "echo hello world");
//! ```

pub mod engine;
pub mod lexer;
pub mod parser;
pub mod context;
pub mod macros;
pub mod error;

pub use engine::TemplateEngine;
pub use context::Context;
pub use error::{TemplateError, Result};