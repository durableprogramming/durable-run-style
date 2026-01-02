//! Lexer for template syntax using logos

use logos::Logos;
use crate::error::{TemplateError, Result};

/// Tokens for template parsing
#[derive(Debug, Logos, PartialEq)]
#[logos(error = LexingError)]
pub enum TemplateToken {
    /// Variable placeholder like {name} or {name.default}
    #[regex(r"\{[^{}]*\}", priority = 2)]
    Placeholder,

    /// Free text (anything not a placeholder)
    #[regex(r"[^{}\\]+", priority = 0)]
    FreeText,

    /// Escaped braces or backslashes
    #[regex(r"\\\{*[^{}]*}*", priority = 1)]
    EscapedText,
}

/// Tokens inside a placeholder
#[derive(Debug, Logos, PartialEq)]
#[logos(error = LexingError)]
pub enum PlaceholderToken<'a> {
    #[token("{")]
    BraceOpen,
    #[token("}")]
    BraceClose,
    #[token(".")]
    Dot,
    #[token(":")]
    Colon,
    #[token("-")]
    Dash,
    #[token("..")]
    Range,
    #[regex(r"[0-9]+", |lex| lex.slice().parse())]
    Number(usize),
    #[regex(r"[^\W\d]\w*", |lex| lex.slice())]
    Identifier(&'a str),
}

/// Lexing errors
#[derive(Debug, Clone, PartialEq)]
pub enum LexingError {
    /// Invalid token encountered
    InvalidToken(String),
    /// Invalid number format
    InvalidNumber(String),
}

impl Default for LexingError {
    fn default() -> Self {
        LexingError::InvalidToken("Unknown token".to_string())
    }
}

impl From<LexingError> for TemplateError {
    fn from(err: LexingError) -> Self {
        match err {
            LexingError::InvalidToken(msg) => TemplateError::Lexing {
                message: format!("Invalid token: {}", msg),
            },
            LexingError::InvalidNumber(msg) => TemplateError::Lexing {
                message: format!("Invalid number: {}", msg),
            },
        }
    }
}

impl From<std::num::ParseIntError> for LexingError {
    fn from(err: std::num::ParseIntError) -> Self {
        LexingError::InvalidNumber(err.to_string())
    }
}

/// Lexer for templates
pub struct TemplateLexer<'a> {
    pub input: &'a str,
}

impl<'a> TemplateLexer<'a> {
    pub fn new(input: &'a str) -> Self {
        Self { input }
    }

    pub fn tokenize(&self) -> Result<Vec<TemplateToken>> {
        let lexer = TemplateToken::lexer(self.input);
        let mut tokens = Vec::new();

        for token in lexer {
            match token {
                Ok(tok) => tokens.push(tok),
                Err(err) => return Err(err.into()),
            }
        }

        Ok(tokens)
    }

    pub fn tokenize_placeholder(content: &'a str) -> Result<Vec<PlaceholderToken<'a>>> {
        let lexer = PlaceholderToken::lexer(content);
        let mut tokens = Vec::new();

        for token in lexer {
            match token {
                Ok(tok) => tokens.push(tok),
                Err(err) => return Err(TemplateError::Lexing {
                    message: format!("Invalid placeholder token: {:?}", err),
                }),
            }
        }

        Ok(tokens)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_tokenize_simple() {
        let lexer = TemplateLexer::new("echo {name}");
        let tokens = lexer.tokenize().unwrap();
        assert_eq!(tokens.len(), 2);
        assert!(matches!(tokens[0], TemplateToken::FreeText));
        assert!(matches!(tokens[1], TemplateToken::Placeholder));
    }

    #[test]
    fn test_tokenize_placeholder() {
        let tokens = TemplateLexer::tokenize_placeholder("name").unwrap();
        assert_eq!(tokens.len(), 1);
        assert!(matches!(tokens[0], PlaceholderToken::Identifier("name")));
    }

    #[test]
    fn test_tokenize_range() {
        let tokens = TemplateLexer::tokenize_placeholder("0..2").unwrap();
        assert_eq!(tokens.len(), 3);
        assert!(matches!(tokens[0], PlaceholderToken::Number(0)));
        assert!(matches!(tokens[1], PlaceholderToken::Range));
        assert!(matches!(tokens[2], PlaceholderToken::Number(2)));
    }

    #[test]
    fn test_tokenize_escaped_brace() {
        let lexer = TemplateLexer::new(r"echo \{name\}");
        let tokens = lexer.tokenize().unwrap();
        assert_eq!(tokens.len(), 2);
        assert!(matches!(tokens[0], TemplateToken::FreeText));
        assert!(matches!(tokens[1], TemplateToken::EscapedText));
    }

    #[test]
    fn test_tokenize_double_escaped() {
        let lexer = TemplateLexer::new(r"echo \\{name}");
        let tokens = lexer.tokenize().unwrap();
        assert_eq!(tokens.len(), 3);
        assert!(matches!(tokens[0], TemplateToken::FreeText));
        assert!(matches!(tokens[1], TemplateToken::EscapedText));
        assert!(matches!(tokens[2], TemplateToken::Placeholder));
    }

    #[test]
    fn test_tokenize_empty_placeholder() {
        let lexer = TemplateLexer::new("echo {}");
        let tokens = lexer.tokenize().unwrap();
        assert_eq!(tokens.len(), 2);
        assert!(matches!(tokens[0], TemplateToken::FreeText));
        assert!(matches!(tokens[1], TemplateToken::Placeholder));
    }

    #[test]
    fn test_tokenize_placeholder_with_special_chars() {
        let tokens = TemplateLexer::tokenize_placeholder("var_name_123").unwrap();
        assert_eq!(tokens.len(), 1);
        assert!(matches!(tokens[0], PlaceholderToken::Identifier("var_name_123")));
    }

    #[test]
    fn test_tokenize_placeholder_with_dash() {
        let tokens = TemplateLexer::tokenize_placeholder("0-2").unwrap();
        assert_eq!(tokens.len(), 3);
        assert!(matches!(tokens[0], PlaceholderToken::Number(0)));
        assert!(matches!(tokens[1], PlaceholderToken::Dash));
        assert!(matches!(tokens[2], PlaceholderToken::Number(2)));
    }

    #[test]
    fn test_tokenize_unclosed_brace() {
        let lexer = TemplateLexer::new("echo {name");
        assert!(lexer.tokenize().is_err());
    }

    #[test]
    fn test_tokenize_nested_braces() {
        let lexer = TemplateLexer::new("echo {{name}}");
        // Nested braces should be an error since placeholders can't contain braces
        assert!(lexer.tokenize().is_err());
    }

    #[test]
    fn test_tokenize_complex_escaped() {
        let lexer = TemplateLexer::new(r"\{ {name} \}");
        let tokens = lexer.tokenize().unwrap();
        assert_eq!(tokens.len(), 4);
        assert!(matches!(tokens[0], TemplateToken::EscapedText));
        assert!(matches!(tokens[1], TemplateToken::Placeholder));
        assert!(matches!(tokens[2], TemplateToken::FreeText));
        assert!(matches!(tokens[3], TemplateToken::EscapedText));
    }

    #[test]
    fn test_tokenize_escaped_with_content() {
        let lexer = TemplateLexer::new(r"\{name\}");
        let tokens = lexer.tokenize().unwrap();
        assert_eq!(tokens.len(), 1);
        assert!(matches!(tokens[0], TemplateToken::EscapedText));
    }

    #[test]
    fn test_tokenize_mixed_escaped_and_placeholders() {
        let lexer = TemplateLexer::new(r"start \{escaped\} {name} end");
        let tokens = lexer.tokenize().unwrap();
        assert_eq!(tokens.len(), 5);
        assert!(matches!(tokens[0], TemplateToken::FreeText));
        assert!(matches!(tokens[1], TemplateToken::EscapedText));
        assert!(matches!(tokens[2], TemplateToken::FreeText));
        assert!(matches!(tokens[3], TemplateToken::Placeholder));
        assert!(matches!(tokens[4], TemplateToken::FreeText));
    }

    #[test]
    fn test_tokenize_unicode() {
        let lexer = TemplateLexer::new("hello {变量} world");
        let tokens = lexer.tokenize().unwrap();
        assert_eq!(tokens.len(), 3);
        assert!(matches!(tokens[0], TemplateToken::FreeText));
        assert!(matches!(tokens[1], TemplateToken::Placeholder));
        assert!(matches!(tokens[2], TemplateToken::FreeText));
    }

    #[test]
    fn test_tokenize_placeholder_unicode() {
        let tokens = TemplateLexer::tokenize_placeholder("变量名").unwrap();
        assert_eq!(tokens.len(), 1);
        assert!(matches!(tokens[0], PlaceholderToken::Identifier("变量名")));
    }
}