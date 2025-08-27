//! Infrastructure layer providing external services and adapters

/// GraphQL lexer module
pub mod lexer;
/// GraphQL parser module (for schema definition language)
pub mod parser;
/// GraphQL query parser module (for query documents)
pub mod query_parser;
pub mod persistence;

// Re-export main infrastructure components
// (These will be implemented in later PRs)
