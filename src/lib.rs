//! # GraphQL Server in Rust
//!
//! A comprehensive GraphQL server implementation built from scratch to understand
//! the core concepts and architecture of GraphQL.
//!
//! This implementation follows Domain-Driven Design (DDD) principles and implements
//! all major GraphQL features step by step.
//!
//! ## Architecture
//!
//! The project is structured using DDD principles:
//!
//! - **Domain**: Core business logic and entities (`domain` module)
//! - **Application**: Use cases and application services (`application` module)  
//! - **Infrastructure**: External concerns like HTTP and persistence (`infrastructure` module)
//! - **Presentation**: API layer and request handling (`presentation` module)
//!
//! ## Usage
//!
//! ```rust,no_run
//! use graphql_rs::application::server::GraphQLServer;
//!
//! fn main() {
//!     let server = GraphQLServer::new().build();
//!     println!("GraphQL server created: {:?}", std::ptr::addr_of!(server));
//! }
//! ```

#![warn(missing_docs)]
#![warn(clippy::all, clippy::pedantic)]
#![allow(clippy::module_name_repetitions)]

pub mod application;
pub mod domain;
pub mod infrastructure;
pub mod presentation;

// Re-export main components for easier access
// pub use application::GraphQLServer; // Will be implemented in later PRs
// pub use domain::{Schema, Query, ExecutionResult}; // Will be implemented in later PRs

/// Current version of the GraphQL server
pub const VERSION: &str = env!("CARGO_PKG_VERSION");

/// GraphQL specification version implemented by this server
pub const GRAPHQL_SPEC_VERSION: &str = "October 2021";

#[cfg(test)]
mod integration_tests {
    use super::*;

    #[tokio::test]
    async fn test_basic_server_creation() {
        // Integration tests will be added as we implement features
        // This is a placeholder to ensure the module structure compiles
    }
}
