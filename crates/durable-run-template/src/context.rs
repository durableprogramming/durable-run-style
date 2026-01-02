//! Context for template rendering

use std::collections::HashMap;
use serde::{Deserialize, Serialize};

/// Context containing variables and arguments for template rendering
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Context {
    /// Named variables
    pub variables: HashMap<String, String>,
    /// Positional arguments (0-indexed)
    pub arguments: Vec<String>,
    /// Environment variables (optional, loaded on demand)
    #[serde(skip)]
    pub env_vars: Option<HashMap<String, String>>,
    /// Macro execution stack (for cycle detection)
    #[serde(skip)]
    pub macro_stack: Vec<String>,
}

impl Context {
    /// Create a new empty context
    pub fn new() -> Self {
        Self {
            variables: HashMap::new(),
            arguments: Vec::new(),
            env_vars: None,
            macro_stack: Vec::new(),
        }
    }

    /// Add a named variable
    pub fn with_var<S: Into<String>>(mut self, name: S, value: S) -> Self {
        self.variables.insert(name.into(), value.into());
        self
    }

    /// Add a positional argument
    pub fn with_arg<S: Into<String>>(mut self, index: usize, value: S) -> Self {
        if index >= self.arguments.len() {
            self.arguments.resize(index + 1, String::new());
        }
        self.arguments[index] = value.into();
        self
    }

    /// Add multiple arguments
    pub fn with_args<I, S>(mut self, args: I) -> Self
    where
        I: IntoIterator<Item = S>,
        S: Into<String>,
    {
        self.arguments = args.into_iter().map(|s| s.into()).collect();
        self
    }

    /// Get a variable by name
    pub fn get_var(&self, name: &str) -> Option<&str> {
        self.variables.get(name).map(|s| s.as_str())
    }

    /// Get an argument by index
    pub fn get_arg(&self, index: usize) -> Option<&str> {
        self.arguments.get(index).map(|s| s.as_str())
    }

    /// Get an environment variable
    pub fn get_env(&mut self, name: &str) -> Option<&str> {
        if self.env_vars.is_none() {
            self.env_vars = Some(std::env::vars().collect());
        }
        self.env_vars.as_ref().unwrap().get(name).map(|s| s.as_str())
    }

    /// Get a value with fallback: var -> arg -> env
    pub fn get(&mut self, key: &str) -> Option<String> {
        // Try as variable name
        if let Some(value) = self.variables.get(key) {
            return Some(value.clone());
        }

        // Try as argument index
        if let Ok(index) = key.parse::<usize>() {
            if let Some(value) = self.arguments.get(index) {
                return Some(value.clone());
            }
        }

        // Try as environment variable
        if self.env_vars.is_none() {
            self.env_vars = Some(std::env::vars().collect());
        }
        self.env_vars.as_ref().unwrap().get(key).cloned()
    }

    /// Get a range of arguments
    pub fn get_arg_range(&self, start: usize, end: Option<usize>) -> Vec<&str> {
        let end = end.unwrap_or(self.arguments.len());
        self.arguments
            .iter()
            .skip(start)
            .take(end - start)
            .map(|s| s.as_str())
            .collect()
    }

    /// Get an argument by index (supports negative indices)
    pub fn get_arg_isize(&self, index: isize) -> Option<&str> {
        let len = self.arguments.len() as isize;
        let actual_index = if index < 0 { len + index } else { index };
        if actual_index >= 0 && actual_index < len {
            self.arguments.get(actual_index as usize).map(|s| s.as_str())
        } else {
            None
        }
    }

    /// Push a macro name onto the stack
    pub fn push_macro(&mut self, name: &str) {
        self.macro_stack.push(name.to_string());
    }

    /// Pop a macro name from the stack
    pub fn pop_macro(&mut self) -> Option<String> {
        self.macro_stack.pop()
    }

    /// Get the current macro stack depth
    pub fn macro_depth(&self) -> usize {
        self.macro_stack.len()
    }

    /// Check if a macro is currently being executed (to detect cycles)
    pub fn is_macro_active(&self, name: &str) -> bool {
        self.macro_stack.contains(&name.to_string())
    }

    /// Check if context has any data
    pub fn is_empty(&self) -> bool {
        self.variables.is_empty() && self.arguments.is_empty()
    }

    /// Get all variable names
    pub fn variable_names(&self) -> Vec<&str> {
        self.variables.keys().map(|s| s.as_str()).collect()
    }

    /// Check if a variable exists
    pub fn has_var(&self, name: &str) -> bool {
        self.variables.contains_key(name)
    }

    /// Check if an argument exists at the given index
    pub fn has_arg(&self, index: usize) -> bool {
        index < self.arguments.len()
    }

    /// Merge another context into this one (variables and arguments)
    /// Variables from the other context take precedence
    pub fn merge(&mut self, other: &Context) {
        for (key, value) in &other.variables {
            self.variables.insert(key.clone(), value.clone());
        }
        // Extend arguments
        self.arguments.extend(other.arguments.iter().cloned());
    }

    /// Create a new context by merging this one with another
    pub fn merged(&self, other: &Context) -> Context {
        let mut new_ctx = self.clone();
        new_ctx.merge(other);
        new_ctx
    }
}

impl Default for Context {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_context_variables() {
        let ctx = Context::new()
            .with_var("name", "world")
            .with_var("cmd", "echo");

        assert_eq!(ctx.get_var("name"), Some("world"));
        assert_eq!(ctx.get_var("cmd"), Some("echo"));
        assert_eq!(ctx.get_var("missing"), None);
    }

    #[test]
    fn test_context_arguments() {
        let ctx = Context::new()
            .with_arg(0, "hello")
            .with_arg(1, "world");

        assert_eq!(ctx.get_arg(0), Some("hello"));
        assert_eq!(ctx.get_arg(1), Some("world"));
        assert_eq!(ctx.get_arg(2), None);
    }

    #[test]
    fn test_context_get_fallback() {
        let mut ctx = Context::new()
            .with_var("name", "var_value")
            .with_arg(0, "arg_value");

        // Variable takes precedence
        assert_eq!(ctx.get("name"), Some("var_value".to_string()));

        // Argument by index
        assert_eq!(ctx.get("0"), Some("arg_value".to_string()));

        // Environment variable (if set)
        // Note: This test assumes TEST_VAR is not set
        assert_eq!(ctx.get("TEST_VAR"), std::env::var("TEST_VAR").ok());
    }

    #[test]
    fn test_context_args() {
        let ctx = Context::new().with_args(vec!["a", "b", "c"]);
        assert_eq!(ctx.arguments, vec!["a", "b", "c"]);
    }

    #[test]
    fn test_get_arg_isize() {
        let ctx = Context::new()
            .with_arg(0, "first")
            .with_arg(1, "second")
            .with_arg(2, "third");

        assert_eq!(ctx.get_arg_isize(0), Some("first"));
        assert_eq!(ctx.get_arg_isize(1), Some("second"));
        assert_eq!(ctx.get_arg_isize(2), Some("third"));
        assert_eq!(ctx.get_arg_isize(-1), Some("third"));
        assert_eq!(ctx.get_arg_isize(-2), Some("second"));
        assert_eq!(ctx.get_arg_isize(-3), Some("first"));
        assert_eq!(ctx.get_arg_isize(3), None);
        assert_eq!(ctx.get_arg_isize(-4), None);
    }

    #[test]
    fn test_macro_stack() {
        let mut ctx = Context::new();

        assert_eq!(ctx.macro_depth(), 0);
        assert!(!ctx.is_macro_active("test"));

        ctx.push_macro("test");
        assert_eq!(ctx.macro_depth(), 1);
        assert!(ctx.is_macro_active("test"));
        assert!(!ctx.is_macro_active("other"));

        ctx.push_macro("nested");
        assert_eq!(ctx.macro_depth(), 2);
        assert!(ctx.is_macro_active("test"));
        assert!(ctx.is_macro_active("nested"));

        assert_eq!(ctx.pop_macro(), Some("nested".to_string()));
        assert_eq!(ctx.macro_depth(), 1);
        assert!(ctx.is_macro_active("test"));
        assert!(!ctx.is_macro_active("nested"));

        assert_eq!(ctx.pop_macro(), Some("test".to_string()));
        assert_eq!(ctx.macro_depth(), 0);
        assert!(!ctx.is_macro_active("test"));

        assert_eq!(ctx.pop_macro(), None);
    }

    #[test]
    fn test_context_utilities() {
        let ctx = Context::new()
            .with_var("name", "world")
            .with_var("cmd", "echo")
            .with_arg(0, "hello");

        let mut var_names = ctx.variable_names();
        var_names.sort();
        assert_eq!(var_names, vec!["cmd", "name"]);
        assert!(ctx.has_var("name"));
        assert!(!ctx.has_var("missing"));
        assert!(ctx.has_arg(0));
        assert!(!ctx.has_arg(1));
    }

    #[test]
    fn test_context_merge() {
        let mut ctx1 = Context::new()
            .with_var("name", "world")
            .with_arg(0, "hello");

        let ctx2 = Context::new()
            .with_var("name", "universe")  // Should override
            .with_var("cmd", "echo")
            .with_arg(0, "hi")  // Should be added as arg 1
            .with_arg(1, "there");

        ctx1.merge(&ctx2);

        assert_eq!(ctx1.get_var("name"), Some("universe"));  // Overridden
        assert_eq!(ctx1.get_var("cmd"), Some("echo"));       // Added
        assert_eq!(ctx1.get_arg(0), Some("hello"));          // Original
        assert_eq!(ctx1.get_arg(1), Some("hi"));             // Added
        assert_eq!(ctx1.get_arg(2), Some("there"));          // Added
    }

    #[test]
    fn test_context_merged() {
        let ctx1 = Context::new().with_var("a", "1");
        let ctx2 = Context::new().with_var("b", "2");

        let merged = ctx1.merged(&ctx2);

        assert_eq!(merged.get_var("a"), Some("1"));
        assert_eq!(merged.get_var("b"), Some("2"));
        // Original contexts unchanged
        assert_eq!(ctx1.get_var("b"), None);
        assert_eq!(ctx2.get_var("a"), None);
    }
}