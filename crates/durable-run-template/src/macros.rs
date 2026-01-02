//! User-defined macro system

use std::collections::HashMap;
use crate::error::{TemplateError, Result};
use crate::parser::Template;

/// A user-defined macro
#[derive(Debug, Clone)]
pub struct Macro {
    /// Macro name
    pub name: String,
    /// Parameter names
    pub parameters: Vec<String>,
    /// Macro body (template string)
    pub body_str: String,
}

/// Macro registry
#[derive(Debug, Clone)]
pub struct MacroRegistry {
    macros: HashMap<String, Macro>,
}

impl MacroRegistry {
    /// Create a new empty registry
    pub fn new() -> Self {
        Self {
            macros: HashMap::new(),
        }
    }

    /// Register a macro
    pub fn register(&mut self, macro_def: Macro) -> Result<()> {
        if self.macros.contains_key(&macro_def.name) {
            return Err(TemplateError::MacroAlreadyExists {
                name: macro_def.name,
            });
        }
        self.macros.insert(macro_def.name.clone(), macro_def);
        Ok(())
    }

    /// Get a macro by name
    pub fn get(&self, name: &str) -> Option<&Macro> {
        self.macros.get(name)
    }

    /// Check if a macro exists
    pub fn contains(&self, name: &str) -> bool {
        self.macros.contains_key(name)
    }

    /// List all macro names
    pub fn list(&self) -> Vec<&str> {
        self.macros.keys().map(|s| s.as_str()).collect()
    }

    /// Remove a macro
    pub fn remove(&mut self, name: &str) -> bool {
        self.macros.remove(name).is_some()
    }

    /// Clear all macros
    pub fn clear(&mut self) {
        self.macros.clear();
    }
}

impl Macro {
    /// Create a new macro
    pub fn new(name: String, parameters: Vec<String>, body_str: String) -> Self {
        Self {
            name,
            parameters,
            body_str,
        }
    }

    /// Parse a macro from string definition
    /// Format: name(param1, param2) = body or name = body
    pub fn parse(definition: &str) -> Result<Self> {
        let definition = definition.trim();

        // Find the = separator
        let eq_pos = definition.find('=').ok_or_else(|| {
            TemplateError::Parse {
                message: "Macro definition must contain '='".to_string(),
            }
        })?;

        let (signature, body_str) = definition.split_at(eq_pos);
        let body_str = &body_str[1..].trim(); // Skip =

        // Parse signature: name or name(param1, param2)
        let signature = signature.trim();

        if let Some(paren_start) = signature.find('(') {
            // Has parameters
            let paren_end = signature.rfind(')').ok_or_else(|| {
                TemplateError::Parse {
                    message: "Macro signature must contain ')'".to_string(),
                }
            })?;

            let name = signature[..paren_start].trim().to_string();
            let params_str = &signature[paren_start + 1..paren_end];

            let parameters: Vec<String> = if params_str.trim().is_empty() {
                Vec::new()
            } else {
                params_str
                    .split(',')
                    .map(|p| p.trim().to_string())
                    .collect()
            };

            Ok(Self::new(name, parameters, body_str.to_string()))
        } else {
            // No parameters
            let name = signature.to_string();
            Ok(Self::new(name, Vec::new(), body_str.to_string()))
        }
    }

    /// Execute the macro with given arguments
    pub fn execute(&self, args: &[String], context: &mut crate::context::Context, engine: &crate::engine::TemplateEngine) -> Result<String> {
        if args.len() != self.parameters.len() {
            return Err(TemplateError::MacroExecution {
                message: format!(
                    "Macro '{}' expects {} arguments, got {}",
                    self.name,
                    self.parameters.len(),
                    args.len()
                ),
            });
        }

        // Create a temporary context with parameters
        let mut macro_context = context.clone();
        for (param, arg) in self.parameters.iter().zip(args.iter()) {
            macro_context.variables.insert(param.clone(), arg.clone());
        }

        // Expand macros in the body string first, then render
        let expanded_body = engine.expand_macros_in_string(&self.body_str, &mut macro_context)?;
        let body = Template::parse(&expanded_body)?;
        body.render(&mut macro_context)
    }
}

impl Default for MacroRegistry {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::context::Context;

    #[test]
    fn test_macro_parse() {
        let macro_def = Macro::parse("greet(name) = Hello {name}!").unwrap();
        assert_eq!(macro_def.name, "greet");
        assert_eq!(macro_def.parameters, vec!["name"]);
    }

    #[test]
    fn test_macro_execute() {
        use crate::engine::TemplateEngine;
        let engine = TemplateEngine::new();
        let macro_def = Macro::parse("greet(name) = Hello {name}!").unwrap();
        let mut context = Context::new();
        let result = macro_def.execute(&["World".to_string()], &mut context, &engine).unwrap();
        assert_eq!(result, "Hello World!");
    }

    #[test]
    fn test_macro_registry() {
        let mut registry = MacroRegistry::new();
        let macro_def = Macro::parse("test() = success").unwrap();
        registry.register(macro_def).unwrap();
        assert!(registry.contains("test"));
        assert_eq!(registry.list(), vec!["test"]);
    }

    #[test]
    fn test_macro_with_outer_variables() {
        use crate::engine::TemplateEngine;
        let engine = TemplateEngine::new();
        let macro_def = Macro::parse("greet(name) = Hello {name}, today is {day}!").unwrap();
        let mut context = Context::new()
            .with_var("day", "Monday");
        let result = macro_def.execute(&["World".to_string()], &mut context, &engine).unwrap();
        assert_eq!(result, "Hello World, today is Monday!");
    }

    #[test]
    fn test_macro_nested_calls() {
        use crate::engine::TemplateEngine;
        let mut engine = TemplateEngine::new();
        engine.register_macro_str("inner() = inner result").unwrap();
        engine.register_macro_str("outer() = outer {inner()}").unwrap();
        let mut context = Context::new();
        let result = engine.execute_macro("outer", &[], &mut context).unwrap();
        assert_eq!(result, "outer inner result");
    }

    #[test]
    fn test_macro_parse_edge_cases() {
        // Test macro with no parameters
        let macro_def = Macro::parse("simple = hello world").unwrap();
        assert_eq!(macro_def.name, "simple");
        assert_eq!(macro_def.parameters, Vec::<String>::new());
        assert_eq!(macro_def.body_str, "hello world");

        // Test macro with spaces around equals
        let macro_def = Macro::parse("spaced  =  body").unwrap();
        assert_eq!(macro_def.name, "spaced");
        assert_eq!(macro_def.body_str, "body");

        // Test invalid macros
        assert!(Macro::parse("no equals").is_err());
        assert!(Macro::parse("missing_paren(").is_err());
        assert!(Macro::parse("missing_close)").is_err());
    }

    #[test]
    fn test_macro_deep_nesting() {
        use crate::engine::TemplateEngine;
        let mut engine = TemplateEngine::new();

        // Create a chain of macros
        engine.register_macro_str("level1 = level1").unwrap();
        engine.register_macro_str("level2 = {level1()}").unwrap();
        engine.register_macro_str("level3 = {level2()}").unwrap();
        engine.register_macro_str("level4 = {level3()}").unwrap();
        engine.register_macro_str("level5 = {level4()}").unwrap();

        let mut context = Context::new();
        let result = engine.execute_macro("level5", &[], &mut context).unwrap();
        assert_eq!(result, "level1");
    }
}