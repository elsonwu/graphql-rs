//! Domain layer containing core business logic and entities
//!
//! This module contains the pure domain logic without any external dependencies.
//! It represents the core GraphQL concepts and business rules.

pub mod entities;
pub mod services;
pub mod value_objects;
pub mod repositories;
pub mod events;

// Re-export main domain types for easier access
pub use entities::{Schema, Query};
pub use value_objects::{ExecutionResult, ValidationResult};
pub use services::{QueryValidator, QueryExecutor, SchemaValidator};
