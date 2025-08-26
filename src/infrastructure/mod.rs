//! Infrastructure layer for external concerns
//!
//! This layer handles external dependencies like HTTP, persistence, and third-party integrations.

pub mod http;
pub mod persistence;

// Re-export main infrastructure components
// (These will be implemented in later PRs)
