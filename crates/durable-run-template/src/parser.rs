//! Parser for template syntax

use crate::error::{TemplateError, Result};

/// A parsed template fragment
#[derive(Debug, Clone, PartialEq)]
pub enum TemplateFragment {
    /// Literal text
    Literal(String),
    /// Variable or argument reference
    Variable(VariableRef),
}

/// A variable or argument reference
#[derive(Debug, Clone, PartialEq)]
pub struct VariableRef {
    /// The name or index
    pub name: String,
    /// Default value if not found
    pub default: Option<String>,
    /// Separator for ranges
    pub separator: Option<String>,
    /// Range specification
    pub range: Option<Range>,
}

/// Range specification for arguments
#[derive(Debug, Clone, PartialEq)]
pub enum Range {
    /// Single index
    Single(isize),
    /// Range from start to end (inclusive)
    Range(isize, isize),
    /// All remaining arguments
    All,
}

/// Parsed template
#[derive(Debug, Clone, PartialEq)]
pub struct Template {
    pub fragments: Vec<TemplateFragment>,
}

impl Template {
    /// Parse a template string
    pub fn parse(input: &str) -> Result<Self> {
        use crate::lexer::TemplateToken;

        let mut lexer = logos::Logos::lexer(input);
        let mut fragments = Vec::new();
        let mut current_literal = String::new();

        while let Some(token) = lexer.next() {
            match token {
                Ok(tok) => match tok {
                    TemplateToken::FreeText => {
                        current_literal.push_str(lexer.slice());
                    }
                    TemplateToken::Placeholder => {
                        // Flush any pending literal
                        if !current_literal.is_empty() {
                            fragments.push(TemplateFragment::Literal(current_literal.clone()));
                            current_literal.clear();
                        }

                        // Parse the placeholder content (without braces)
                        let slice = lexer.slice();
                        let placeholder_content = &slice[1..slice.len() - 1];
                        let var_ref = VariableRef::parse(placeholder_content)?;
                        fragments.push(TemplateFragment::Variable(var_ref));
                    }
                    TemplateToken::EscapedText => {
                        // Handle escaped text
                        let escaped = lexer.slice();
                        if let Some(stripped) = escaped.strip_prefix('\\') {
                            // Remove one level of escaping
                            current_literal.push_str(stripped);
                        } else {
                            current_literal.push_str(escaped);
                        }
                    }
                },
                Err(err) => return Err(err.into()),
            }
        }

        // Flush final literal
        if !current_literal.is_empty() {
            fragments.push(TemplateFragment::Literal(current_literal));
        }

        Ok(Template { fragments })
    }

    /// Render the template with a context
    pub fn render(&self, context: &mut crate::context::Context) -> Result<String> {
        let mut result = String::new();

        for fragment in &self.fragments {
            match fragment {
                TemplateFragment::Literal(text) => result.push_str(text),
                TemplateFragment::Variable(var_ref) => {
                    let value = var_ref.resolve(context)?;
                    result.push_str(&value);
                }
            }
        }

        Ok(result)
    }
}

impl VariableRef {
    /// Parse a placeholder content into a VariableRef
    pub fn parse(content: &str) -> Result<Self> {
        if content.is_empty() {
            return Err(TemplateError::Parse {
                message: "Empty placeholder content".to_string(),
            });
        }

        let content = content.trim();

        // Check for range syntax first
        if let Some(range_pos) = content.find("..") {
            let (before_range, after_range) = content.split_at(range_pos);
            let after_range = &after_range[2..]; // Skip ..

            // Check for separator
            let (range_part, separator) = if let Some(colon_pos) = after_range.find(':') {
                let (range, sep) = after_range.split_at(colon_pos);
                (range, Some(sep[1..].trim().to_string()))
            } else {
                (after_range, None)
            };

            // Parse range
            let range = if before_range.is_empty() && range_part.is_empty() {
                Range::All
            } else if before_range.is_empty() {
                // {..end} -> Range(0, end)
                let end = range_part.parse::<isize>().map_err(|_| {
                    TemplateError::InvalidRange {
                        range: format!("..{}", range_part),
                    }
                })?;
                Range::Range(0, end)
            } else if range_part.is_empty() {
                // {start..} -> Range(start, -1) meaning to end
                let start = before_range.parse::<isize>().map_err(|_| {
                    TemplateError::InvalidRange {
                        range: format!("{}..", before_range),
                    }
                })?;
                Range::Range(start, -1)
            } else {
                // {start..end}
                let start = before_range.parse::<isize>().map_err(|_| {
                    TemplateError::InvalidRange {
                        range: format!("{}..{}", before_range, range_part),
                    }
                })?;
                let end = range_part.parse::<isize>().map_err(|_| {
                    TemplateError::InvalidRange {
                        range: format!("{}..{}", before_range, range_part),
                    }
                })?;
                Range::Range(start, end)
            };

            return Ok(VariableRef {
                name: before_range.to_string(),
                default: None,
                separator,
                range: Some(range),
            });
        }

        // Check for default value (colon syntax)
        if let Some(colon_pos) = content.find(':') {
            let (name, default) = content.split_at(colon_pos);
            let default = &default[1..]; // Skip colon
            return Ok(VariableRef {
                name: name.to_string(),
                default: Some(default.to_string()),
                separator: None,
                range: None,
            });
        }

        // Check for default value (dot syntax)
        if let Some(dot_pos) = content.find('.') {
            let (name, default) = content.split_at(dot_pos);
            let default = &default[1..]; // Skip dot
            return Ok(VariableRef {
                name: name.to_string(),
                default: Some(default.to_string()),
                separator: None,
                range: None,
            });
        }

        // Simple name or single index
        // If it's just a number, treat as single positional argument
        if let Ok(index) = content.parse::<isize>() {
            Ok(VariableRef {
                name: content.to_string(),
                default: None,
                separator: None,
                range: Some(Range::Single(index)),
            })
        } else {
            Ok(VariableRef {
                name: content.to_string(),
                default: None,
                separator: None,
                range: None,
            })
        }
    }

    /// Resolve the variable reference in a context
    pub fn resolve(&self, context: &mut crate::context::Context) -> Result<String> {
        match &self.range {
            Some(Range::Single(idx)) => {
                let index = if *idx < 0 {
                    (context.arguments.len() as isize + idx) as usize
                } else {
                    *idx as usize
                };
                context.get_arg(index)
                    .or(self.default.as_deref())
                    .map(|s| s.to_string())
                    .ok_or_else(|| TemplateError::ArgumentNotFound { index })
            }
            Some(Range::Range(start, end)) => {
                let start = if *start < 0 {
                    (context.arguments.len() as isize + start) as usize
                } else {
                    *start as usize
                };
                let end = if *end < 0 {
                    context.arguments.len()
                } else {
                    (*end + 1) as usize
                };

                let args: Vec<&str> = context.arguments
                    .iter()
                    .skip(start)
                    .take(end.saturating_sub(start))
                    .map(|s| s.as_str())
                    .collect();

                let sep = self.separator.as_deref().unwrap_or(" ");
                Ok(args.join(sep))
            }
            Some(Range::All) => {
                let args: Vec<&str> = context.arguments.iter().map(|s| s.as_str()).collect();
                let sep = self.separator.as_deref().unwrap_or(" ");
                Ok(args.join(sep))
            }
            None => {
                // Simple variable or argument
                if let Some(value) = context.get(&self.name) {
                    Ok(value)
                } else if let Some(default) = &self.default {
                    Ok(default.clone())
                } else if let Ok(idx) = self.name.parse::<usize>() {
                    Err(TemplateError::ArgumentNotFound { index: idx })
                } else {
                    Err(TemplateError::VariableNotFound {
                        name: self.name.clone(),
                    })
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::Context;

    #[test]
    fn test_parse_simple_variable() {
        let template = Template::parse("Hello {name}").unwrap();
        assert_eq!(template.fragments.len(), 2);
        assert!(matches!(template.fragments[0], TemplateFragment::Literal(_)));
        assert!(matches!(template.fragments[1], TemplateFragment::Variable(_)));
    }

    #[test]
    fn test_parse_variable_with_default() {
        let var_ref = VariableRef::parse("name.default").unwrap();
        assert_eq!(var_ref.name, "name");
        assert_eq!(var_ref.default, Some("default".to_string()));
    }

    #[test]
    fn test_parse_argument_range() {
        let var_ref = VariableRef::parse("0..2").unwrap();
        assert_eq!(var_ref.name, "0");
        assert!(matches!(var_ref.range, Some(Range::Range(0, 2))));
    }

    #[test]
    fn test_render_simple() {
        let template = Template::parse("Hello {name}").unwrap();
        let mut context = crate::context::Context::new().with_var("name", "World");
        let result = template.render(&mut context).unwrap();
        assert_eq!(result, "Hello World");
    }

    #[test]
    fn test_render_with_default() {
        let template = Template::parse("Hello {nonexistent.unknown}").unwrap();
        let mut context = crate::context::Context::new();
        let result = template.render(&mut context).unwrap();
        assert_eq!(result, "Hello unknown");
    }

    #[test]
    fn test_parse_single_argument() {
        let var_ref = VariableRef::parse("0").unwrap();
        assert_eq!(var_ref.name, "0");
        assert!(matches!(var_ref.range, Some(Range::Single(0))));
    }

    #[test]
    fn test_parse_range_all() {
        let var_ref = VariableRef::parse("..").unwrap();
        assert_eq!(var_ref.name, "");
        assert!(matches!(var_ref.range, Some(Range::All)));
    }

    #[test]
    fn test_parse_range_from_start() {
        let var_ref = VariableRef::parse("1..").unwrap();
        assert_eq!(var_ref.name, "1");
        assert!(matches!(var_ref.range, Some(Range::Range(1, -1))));
    }

    #[test]
    fn test_parse_range_to_end() {
        let var_ref = VariableRef::parse("..3").unwrap();
        assert_eq!(var_ref.name, "");
        assert!(matches!(var_ref.range, Some(Range::Range(0, 3))));
    }

    #[test]
    fn test_parse_range_with_separator() {
        let var_ref = VariableRef::parse("0..2:, ").unwrap();
        assert_eq!(var_ref.name, "0");
        assert!(matches!(var_ref.range, Some(Range::Range(0, 2))));
        assert_eq!(var_ref.separator, Some(",".to_string()));
    }

    #[test]
    fn test_render_single_argument() {
        let template = Template::parse("Arg: {0}").unwrap();
        let mut context = Context::new().with_arg(0, "first");
        let result = template.render(&mut context).unwrap();
        assert_eq!(result, "Arg: first");
    }

    #[test]
    fn test_render_range() {
        let template = Template::parse("Args: {0..2}").unwrap();
        let mut context = Context::new()
            .with_arg(0, "a")
            .with_arg(1, "b")
            .with_arg(2, "c")
            .with_arg(3, "d");
        let result = template.render(&mut context).unwrap();
        assert_eq!(result, "Args: a b c");
    }

    #[test]
    fn test_render_range_with_separator() {
        let template = Template::parse("Args: {0..2:,}").unwrap();
        let mut context = Context::new()
            .with_arg(0, "a")
            .with_arg(1, "b")
            .with_arg(2, "c");
        let result = template.render(&mut context).unwrap();
        assert_eq!(result, "Args: a,b,c");
    }

    #[test]
    fn test_render_range_all() {
        let template = Template::parse("All: {..}").unwrap();
        let mut context = Context::new()
            .with_arg(0, "x")
            .with_arg(1, "y");
        let result = template.render(&mut context).unwrap();
        assert_eq!(result, "All: x y");
    }

    #[test]
    fn test_render_negative_index() {
        let template = Template::parse("Last: {-1}").unwrap();
        let mut context = Context::new()
            .with_arg(0, "first")
            .with_arg(1, "last");
        let result = template.render(&mut context).unwrap();
        assert_eq!(result, "Last: last");
    }
}