//! Template engine that combines all components

use std::collections::HashMap;

use crate::error::{TemplateError, Result};
use crate::context::Context;
use crate::parser::Template;
use crate::macros::{MacroRegistry, Macro};

/// The main template engine
#[derive(Debug, Clone)]
pub struct TemplateEngine {
    /// Registered macros
    pub macros: MacroRegistry,
    /// Cached parsed templates
    cache: HashMap<String, Template>,
}

impl TemplateEngine {
    /// Create a new template engine
    pub fn new() -> Self {
        Self {
            macros: MacroRegistry::new(),
            cache: HashMap::new(),
        }
    }

    /// Parse a template string
    pub fn parse(&mut self, template_str: &str) -> Result<Template> {
        if let Some(template) = self.cache.get(template_str) {
            Ok(template.clone())
        } else {
            let template = Template::parse(template_str)?;
            self.cache.insert(template_str.to_string(), template.clone());
            Ok(template)
        }
    }

    /// Render a template string with context
    pub fn render(&mut self, template_str: &str, context: &mut Context) -> Result<String> {
        let template = self.parse(template_str)?;
        template.render(context)
    }

    /// Register a macro
    pub fn register_macro(&mut self, macro_def: Macro) -> Result<()> {
        self.macros.register(macro_def)
    }

    /// Register a macro from string definition
    pub fn register_macro_str(&mut self, definition: &str) -> Result<()> {
        let macro_def = Macro::parse(definition)?;
        self.register_macro(macro_def)
    }

    /// Execute a macro
    pub fn execute_macro(&self, name: &str, args: &[String], context: &mut Context) -> Result<String> {
        // Check for circular dependency
        if context.is_macro_active(name) {
            return Err(TemplateError::CircularDependency {
                macro_name: name.to_string(),
            });
        }

        context.push_macro(name);
        let result = self.macros.get(name).ok_or_else(|| {
            TemplateError::MacroNotFound { name: name.to_string() }
        }).and_then(|macro_def| macro_def.execute(args, context, self));
        context.pop_macro();
        result
    }

    /// Check if a macro exists
    pub fn has_macro(&self, name: &str) -> bool {
        self.macros.contains(name)
    }

    /// Get all macro names
    pub fn list_macros(&self) -> Vec<&str> {
        self.macros.list()
    }

    /// Clear the template cache
    pub fn clear_cache(&mut self) {
        self.cache.clear();
    }

    /// Get the number of cached templates
    pub fn cache_size(&self) -> usize {
        self.cache.len()
    }

    /// Load macros from a JSON string
    pub fn load_macros_json(&mut self, json: &str) -> Result<()> {
        let macro_defs: Vec<(String, Vec<String>, String)> = serde_json::from_str(json)?;
        for (name, params, body) in macro_defs {
            let macro_def = Macro::new(name, params, body);
            self.register_macro(macro_def)?;
        }
        Ok(())
    }

    /// Export macros to JSON
    pub fn export_macros_json(&self) -> Result<String> {
        let macros: Vec<(String, Vec<String>, String)> = self.macros.list().iter().filter_map(|name| self.macros.get(name)).map(|m| (m.name.clone(), m.parameters.clone(), m.body_str.clone())).collect();
        serde_json::to_string(&macros).map_err(Into::into)
    }

    /// Create a context from command line arguments
    pub fn context_from_args(args: &[String]) -> Context {
        let mut context = Context::new();
        for (i, arg) in args.iter().enumerate() {
            context = context.with_arg(i, arg.clone());
        }
        context
    }

    /// Create a context from environment variables
    pub fn context_from_env() -> Context {
        let mut context = Context::new();
        for (key, value) in std::env::vars() {
            context.variables.insert(key, value);
        }
        context
    }

    /// Create a context from both args and env
    pub fn context_from_args_and_env(args: &[String]) -> Context {
        let mut context = Self::context_from_args(args);
        for (key, value) in std::env::vars() {
            context.variables.entry(key).or_insert(value);
        }
        context
    }

    /// Advanced rendering with macro expansion
    pub fn render_with_macros(&mut self, template_str: &str, context: &mut Context) -> Result<String> {
        // First pass: expand macros in the template string
        let expanded_template = self.expand_macros_in_string(template_str, context)?;
        // Second pass: render the expanded template
        self.render(&expanded_template, context)
    }

    /// Expand macros in a template string
    pub fn expand_macros_in_string(&self, template_str: &str, context: &mut Context) -> Result<String> {
        // Simple macro expansion: look for {macro_name(arg1, arg2)} patterns
        use regex::Regex;
        lazy_static::lazy_static! {
            static ref MACRO_REGEX: Regex = Regex::new(r"\{(\w+)\(([^}]*)\)\}").unwrap();
        }

        let mut result = template_str.to_string();
        let mut changed = true;

        // Expand macros iteratively until no more expansions
        while changed {
            changed = false;
            let mut new_result = String::new();
            let mut last_end = 0;

            for cap in MACRO_REGEX.captures_iter(&result) {
                let full_match = cap.get(0).unwrap();
                let macro_name = cap.get(1).unwrap().as_str();
                let args_str = cap.get(2).unwrap().as_str();

                new_result.push_str(&result[last_end..full_match.start()]);

                if self.has_macro(macro_name) {
                    let args: Vec<String> = if args_str.trim().is_empty() {
                        Vec::new()
                    } else {
                        args_str.split(',').map(|s| s.trim().to_string()).collect()
                    };

                    let expansion = self.execute_macro(macro_name, &args, context)?;

                    new_result.push_str(&expansion);
                    changed = true;
                } else {
                    // Keep the original if macro not found
                    new_result.push_str(full_match.as_str());
                }

                last_end = full_match.end();
            }

            new_result.push_str(&result[last_end..]);
            result = new_result;
        }

        Ok(result)
    }
}

impl Default for TemplateEngine {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_engine_basic_render() {
        let mut engine = TemplateEngine::new();
        let mut context = Context::new().with_var("name", "World");
        let result = engine.render("Hello {name}!", &mut context).unwrap();
        assert_eq!(result, "Hello World!");
    }

    #[test]
    fn test_engine_macro() {
        let mut engine = TemplateEngine::new();
        engine.register_macro_str("greet(name) = Hello {name}!").unwrap();
        let mut context = Context::new();
        let result = engine.execute_macro("greet", &["World".to_string()], &mut context).unwrap();
        assert_eq!(result, "Hello World!");
    }

    #[test]
    fn test_engine_macro_expansion() {
        let mut engine = TemplateEngine::new();
        engine.register_macro_str("bold(text) = **{text}**").unwrap();
        let mut context = Context::new();
        let result = engine.render_with_macros("This is {bold(important)} text", &mut context).unwrap();
        assert_eq!(result, "This is **important** text");
    }

    #[test]
    fn test_context_from_args() {
        let args = vec!["arg1".to_string(), "arg2".to_string()];
        let context = TemplateEngine::context_from_args(&args);
        assert_eq!(context.get_arg(0), Some("arg1"));
        assert_eq!(context.get_arg(1), Some("arg2"));
    }

    #[test]
    fn test_macro_expansion_cycle_detection() {
        let mut engine = TemplateEngine::new();
        // Create circular dependency: a calls b, b calls a
        engine.register_macro_str("a() = {b()}").unwrap();
        engine.register_macro_str("b() = {a()}").unwrap();
        let mut context = Context::new();
        let result = engine.render_with_macros("Start {a()}", &mut context);
        assert!(result.is_err());
        assert!(matches!(result.unwrap_err(), TemplateError::CircularDependency { .. }));
    }
}