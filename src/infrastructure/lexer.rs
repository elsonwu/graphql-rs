use logos::Logos;
use std::fmt;

/// Token types in GraphQL language
#[derive(Logos, Debug, PartialEq, Clone)]
pub enum Token {
    // Punctuation
    /// Exclamation mark token '!'
    #[token("!")]
    Bang,

    /// Dollar sign token '$'
    #[token("$")]
    Dollar,

    /// Left parenthesis token '('
    #[token("(")]
    LeftParen,

    /// Right parenthesis token ')'
    #[token(")")]
    RightParen,

    /// Spread operator token '...'
    #[token("...")]
    Spread,

    /// Colon token ':'
    #[token(":")]
    Colon,

    /// Equals token '='
    #[token("=")]
    Equals,

    /// At symbol token '@'
    #[token("@")]
    At,

    /// Left square bracket token '['
    #[token("[")]
    LeftBracket,

    /// Right square bracket token ']'
    #[token("]")]
    RightBracket,

    /// Left curly brace token '{'
    #[token("{")]
    LeftBrace,

    /// Pipe token '|'
    #[token("|")]
    Pipe,

    /// Right curly brace token '}'
    #[token("}")]
    RightBrace,

    // Keywords
    /// Query keyword token 'query'
    #[token("query")]
    Query,

    /// Mutation keyword token 'mutation'
    #[token("mutation")]
    Mutation,

    /// Subscription keyword token 'subscription'
    #[token("subscription")]
    Subscription,

    /// Fragment keyword token 'fragment'
    #[token("fragment")]
    Fragment,

    /// Type keyword token 'type'
    #[token("type")]
    Type,

    /// Implements keyword token 'implements'
    #[token("implements")]
    Implements,

    /// Interface keyword token 'interface'
    #[token("interface")]
    Interface,

    /// Union keyword token 'union'
    #[token("union")]
    Union,

    /// Scalar keyword token 'scalar'
    #[token("scalar")]
    Scalar,

    /// Enum keyword token 'enum'
    #[token("enum")]
    Enum,

    /// Input keyword token 'input'
    #[token("input")]
    Input,

    /// Extend keyword token 'extend'
    #[token("extend")]
    Extend,

    /// Schema keyword token 'schema'
    #[token("schema")]
    Schema,

    /// Directive keyword token 'directive'
    #[token("directive")]
    Directive,

    /// Repeatable keyword token 'repeatable'
    #[token("repeatable")]
    Repeatable,

    /// On keyword token 'on'
    #[token("on")]
    On,

    /// Null literal token 'null'
    #[token("null")]
    Null,

    /// True literal token 'true'
    #[token("true")]
    True,

    /// False literal token 'false'
    #[token("false")]
    False,

    // Literals
    /// String literal token
    #[regex(r#""([^"\\]|\\.)*""#, string_literal)]
    String(String),

    /// Block string literal token
    #[regex(r#"```([^`]|`[^`]|``[^`])*```"#, block_string_literal)]
    BlockString(String),

    /// Integer literal token
    #[regex(r"-?(?:0|[1-9]\d*)", integer_literal)]
    Integer(i64),

    /// Float literal token
    #[regex(r"-?(?:0|[1-9]\d*)\.(?:\d+)(?:[eE][+-]?\d+)?", float_literal)]
    #[regex(r"-?(?:0|[1-9]\d*)(?:[eE][+-]?\d+)", float_literal)]
    Float(f64),

    // Names (must come after keywords)
    /// Name token (identifier)
    #[regex(r"[_A-Za-z][_0-9A-Za-z]*", |lex| lex.slice().to_string())]
    Name(String),

    // Comments
    /// Comment token
    #[regex(r"#[^\r\n]*", comment)]
    Comment(String),

    // Whitespace (ignored)
    /// Whitespace token (automatically skipped)
    #[regex(r"[ \t\r\n,]+", logos::skip)]
    Whitespace,
    // Error token for invalid input
}

impl fmt::Display for Token {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Token::Bang => write!(f, "!"),
            Token::Dollar => write!(f, "$"),
            Token::LeftParen => write!(f, "("),
            Token::RightParen => write!(f, ")"),
            Token::Spread => write!(f, "..."),
            Token::Colon => write!(f, ":"),
            Token::Equals => write!(f, "="),
            Token::At => write!(f, "@"),
            Token::LeftBracket => write!(f, "["),
            Token::RightBracket => write!(f, "]"),
            Token::LeftBrace => write!(f, "{{"),
            Token::Pipe => write!(f, "|"),
            Token::RightBrace => write!(f, "}}"),
            Token::Query => write!(f, "query"),
            Token::Mutation => write!(f, "mutation"),
            Token::Subscription => write!(f, "subscription"),
            Token::Fragment => write!(f, "fragment"),
            Token::Type => write!(f, "type"),
            Token::Implements => write!(f, "implements"),
            Token::Interface => write!(f, "interface"),
            Token::Union => write!(f, "union"),
            Token::Scalar => write!(f, "scalar"),
            Token::Enum => write!(f, "enum"),
            Token::Input => write!(f, "input"),
            Token::Extend => write!(f, "extend"),
            Token::Schema => write!(f, "schema"),
            Token::Directive => write!(f, "directive"),
            Token::Repeatable => write!(f, "repeatable"),
            Token::On => write!(f, "on"),
            Token::Null => write!(f, "null"),
            Token::True => write!(f, "true"),
            Token::False => write!(f, "false"),
            Token::String(s) => write!(f, r#""{}""#, s),
            Token::BlockString(s) => write!(f, "```{}```", s),
            Token::Integer(i) => write!(f, "{}", i),
            Token::Float(fl) => write!(f, "{}", fl),
            Token::Name(name) => write!(f, "{}", name),
            Token::Comment(c) => write!(f, "#{}", c),
            Token::Whitespace => write!(f, " "),
        }
    }
}

/// Lexer for tokenizing GraphQL documents
pub struct Lexer<'input> {
    inner: logos::Lexer<'input, Token>,
    current_token: Option<Token>,
    position: usize,
}

impl<'input> Lexer<'input> {
    /// Create a new lexer for the given input
    pub fn new(input: &'input str) -> Self {
        let mut lexer = Self {
            inner: Token::lexer(input),
            current_token: None,
            position: 0,
        };
        lexer.advance(); // Load the first token
        lexer
    }

    /// Get the current token without advancing
    pub fn current_token(&self) -> Option<&Token> {
        self.current_token.as_ref()
    }

    /// Get the current position in the input
    pub fn position(&self) -> usize {
        self.position
    }

    /// Get the span of the current token
    pub fn span(&self) -> std::ops::Range<usize> {
        self.inner.span()
    }

    /// Get the slice of text for the current token
    pub fn slice(&self) -> &str {
        self.inner.slice()
    }

    /// Advance to the next token
    pub fn advance(&mut self) -> Option<Token> {
        if let Some(Ok(token)) = self.inner.next() {
            self.current_token = Some(token.clone());
            self.position = self.inner.span().start;
            Some(token)
        } else {
            self.current_token = None;
            None
        }
    }

    /// Peek at the current token and advance if it matches the expected token
    pub fn expect(&mut self, expected: &Token) -> Result<(), LexError> {
        match &self.current_token {
            Some(token) if std::mem::discriminant(token) == std::mem::discriminant(expected) => {
                self.advance();
                Ok(())
            },
            Some(token) => Err(LexError::UnexpectedToken {
                expected: format!("{}", expected),
                found: format!("{}", token),
                position: self.position,
            }),
            None => Err(LexError::UnexpectedEof {
                expected: format!("{}", expected),
            }),
        }
    }

    /// Check if the current token matches the expected token without advancing
    pub fn is_current(&self, expected: &Token) -> bool {
        match &self.current_token {
            Some(token) => std::mem::discriminant(token) == std::mem::discriminant(expected),
            None => false,
        }
    }

    /// Consume all remaining tokens
    pub fn remaining_tokens(&mut self) -> Vec<Token> {
        let mut tokens = Vec::new();

        // Add the current token if it exists
        if let Some(token) = &self.current_token {
            tokens.push(token.clone());
        }

        // Add all remaining tokens
        while let Some(token) = self.advance() {
            tokens.push(token);
        }
        tokens
    }
}

/// Errors that can occur during lexical analysis
#[derive(Debug, Clone, PartialEq)]
pub enum LexError {
    /// Unexpected token found during parsing
    UnexpectedToken {
        /// The expected token type
        expected: String,
        /// The token that was actually found
        found: String,
        /// Position in the input where the error occurred
        position: usize,
    },
    /// Unexpected end of file during parsing
    UnexpectedEof {
        /// The expected token type
        expected: String,
    },
    /// Invalid token that could not be recognized
    InvalidToken {
        /// Position in the input where the invalid token was found
        position: usize,
    },
}

impl fmt::Display for LexError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            LexError::UnexpectedToken {
                expected,
                found,
                position,
            } => {
                write!(
                    f,
                    "Expected {} but found {} at position {}",
                    expected, found, position
                )
            },
            LexError::UnexpectedEof { expected } => {
                write!(f, "Unexpected end of input, expected {}", expected)
            },
            LexError::InvalidToken { position } => {
                write!(f, "Invalid token at position {}", position)
            },
        }
    }
}

impl std::error::Error for LexError {}

// Helper functions for token extraction

/// Extract string literal content
fn string_literal(lex: &mut logos::Lexer<Token>) -> Option<String> {
    let slice = lex.slice();
    // Remove quotes and process escape sequences
    let content = &slice[1..slice.len() - 1];
    Some(process_string_escapes(content))
}

/// Extract block string literal content
fn block_string_literal(lex: &mut logos::Lexer<Token>) -> Option<String> {
    let slice = lex.slice();
    // Remove triple quotes
    let content = &slice[3..slice.len() - 3];
    Some(process_block_string(content))
}

/// Extract integer literal
fn integer_literal(lex: &mut logos::Lexer<Token>) -> Option<i64> {
    lex.slice().parse().ok()
}

/// Extract float literal
fn float_literal(lex: &mut logos::Lexer<Token>) -> Option<f64> {
    lex.slice().parse().ok()
}

/// Extract comment content
fn comment(lex: &mut logos::Lexer<Token>) -> Option<String> {
    let slice = lex.slice();
    // Remove # prefix
    Some(slice[1..].to_string())
}

/// Process escape sequences in string literals
fn process_string_escapes(input: &str) -> String {
    let mut result = String::new();
    let mut chars = input.chars().peekable();

    while let Some(ch) = chars.next() {
        if ch == '\\' {
            match chars.next() {
                Some('"') => result.push('"'),
                Some('\\') => result.push('\\'),
                Some('/') => result.push('/'),
                Some('b') => result.push('\u{0008}'), // backspace
                Some('f') => result.push('\u{000C}'), // form feed
                Some('n') => result.push('\n'),
                Some('r') => result.push('\r'),
                Some('t') => result.push('\t'),
                Some('u') => {
                    // Unicode escape sequence \uXXXX
                    let mut hex = String::new();
                    for _ in 0..4 {
                        if let Some(hex_char) = chars.next() {
                            hex.push(hex_char);
                        }
                    }
                    if let Ok(code_point) = u32::from_str_radix(&hex, 16) {
                        if let Some(unicode_char) = std::char::from_u32(code_point) {
                            result.push(unicode_char);
                        }
                    }
                },
                Some(escaped) => {
                    // Unknown escape, keep as-is
                    result.push('\\');
                    result.push(escaped);
                },
                None => result.push('\\'),
            }
        } else {
            result.push(ch);
        }
    }

    result
}

/// Process block string formatting (remove common indentation)
fn process_block_string(input: &str) -> String {
    let lines: Vec<&str> = input.lines().collect();
    if lines.is_empty() {
        return String::new();
    }

    // Find common indentation (ignoring first and last lines if they're empty)
    let mut min_indent = usize::MAX;
    let start = if lines[0].trim().is_empty() { 1 } else { 0 };
    let end = if lines.len() > 1 && lines[lines.len() - 1].trim().is_empty() {
        lines.len() - 1
    } else {
        lines.len()
    };

    for line in &lines[start..end] {
        if !line.trim().is_empty() {
            let indent = line.len() - line.trim_start().len();
            min_indent = min_indent.min(indent);
        }
    }

    if min_indent == usize::MAX {
        min_indent = 0;
    }

    // Remove common indentation and join lines
    let processed_lines: Vec<&str> = lines[start..end]
        .iter()
        .map(|line| {
            if line.len() >= min_indent {
                &line[min_indent..]
            } else {
                line
            }
        })
        .collect();

    processed_lines.join("\n")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn tokenize_simple_query() {
        let input = r#"
        query GetUser($id: ID!) {
            user(id: $id) {
                name
                email
            }
        }
        "#;

        let mut lexer = Lexer::new(input);
        let tokens = lexer.remaining_tokens();

        // Should contain Query, Name, LeftParen, etc.
        assert!(tokens.contains(&Token::Query));
        assert!(tokens.contains(&Token::Name("GetUser".to_string())));
        assert!(tokens.contains(&Token::LeftParen));
        assert!(tokens.contains(&Token::Dollar));
        assert!(tokens.contains(&Token::Name("id".to_string())));
    }

    #[test]
    fn tokenize_string_literal() {
        let input = r#""Hello, World!""#;
        let lexer = Lexer::new(input);

        assert_eq!(
            lexer.current_token(),
            Some(&Token::String("Hello, World!".to_string()))
        );
    }

    #[test]
    fn tokenize_block_string_literal() {
        let input = r#"```
        This is a block string
        with multiple lines
        ```"#;
        let lexer = Lexer::new(input);

        if let Some(Token::BlockString(content)) = lexer.current_token() {
            assert!(content.contains("This is a block string"));
            assert!(content.contains("with multiple lines"));
        } else {
            panic!("Expected block string token");
        }
    }

    #[test]
    fn tokenize_numbers() {
        let input = "42 3.141 -7 1.5e10";
        let mut lexer = Lexer::new(input);
        let tokens = lexer.remaining_tokens();

        assert!(tokens.contains(&Token::Integer(42)));
        assert!(tokens.contains(&Token::Float(3.141))); // Changed from 3.14 to avoid approximation warning
        assert!(tokens.contains(&Token::Integer(-7)));
        assert!(tokens.contains(&Token::Float(1.5e10)));
    }

    #[test]
    fn tokenize_keywords_and_punctuation() {
        let input = "type User implements Node { id: ID! }";
        let mut lexer = Lexer::new(input);
        let tokens = lexer.remaining_tokens();

        assert!(tokens.contains(&Token::Type));
        assert!(tokens.contains(&Token::Name("User".to_string())));
        assert!(tokens.contains(&Token::Implements));
        assert!(tokens.contains(&Token::Name("Node".to_string())));
        assert!(tokens.contains(&Token::LeftBrace));
        assert!(tokens.contains(&Token::Name("id".to_string())));
        assert!(tokens.contains(&Token::Colon));
        assert!(tokens.contains(&Token::Name("ID".to_string())));
        assert!(tokens.contains(&Token::Bang));
        assert!(tokens.contains(&Token::RightBrace));
    }

    #[test]
    fn tokenize_comments() {
        let input = r#"
        # This is a comment
        type User {
            # Another comment
            name: String
        }
        "#;
        let mut lexer = Lexer::new(input);
        let tokens = lexer.remaining_tokens();

        assert!(tokens.contains(&Token::Comment(" This is a comment".to_string())));
        assert!(tokens.contains(&Token::Comment(" Another comment".to_string())));
        assert!(tokens.contains(&Token::Type));
        assert!(tokens.contains(&Token::Name("User".to_string())));
    }

    #[test]
    fn lexer_expect_token() {
        let input = "query { user }";
        let mut lexer = Lexer::new(input);

        assert!(lexer.expect(&Token::Query).is_ok());
        assert!(lexer.expect(&Token::LeftBrace).is_ok());
        assert!(lexer.expect(&Token::Name("user".to_string())).is_ok());
        assert!(lexer.expect(&Token::RightBrace).is_ok());
    }

    #[test]
    fn lexer_unexpected_token_error() {
        let input = "query";
        let mut lexer = Lexer::new(input);

        let result = lexer.expect(&Token::Mutation);
        assert!(matches!(result, Err(LexError::UnexpectedToken { .. })));
    }

    #[test]
    fn process_string_escapes_test() {
        assert_eq!(process_string_escapes(r#"Hello\nWorld"#), "Hello\nWorld");
        assert_eq!(process_string_escapes(r#"Quote: \""#), r#"Quote: ""#);
        assert_eq!(process_string_escapes(r#"Unicode: \u0041"#), "Unicode: A");
    }
}
