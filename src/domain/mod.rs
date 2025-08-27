//! Domain layer containing core business logic and entities
//!
//! This module contains the pure domain logic without any external dependencies.
//! It represents the core GraphQL concepts and business rules.

/// Entity module containing domain entities
pub mod entities;
pub mod events;
pub mod repositories;
pub mod services;
pub mod value_objects;

// Re-export main domain types for easier access
// pub use entities::{Schema, Query}; // Will be implemented in later PRs
// pub use value_objects::{ExecutionResult, ValidationResult}; // Will be implemented in later PRs
// pub use services::{QueryValidator, QueryExecutor, SchemaValidator}; // Will be implemented in later PRs
