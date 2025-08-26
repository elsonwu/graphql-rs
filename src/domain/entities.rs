//! Domain entities representing core GraphQL concepts
//!
//! Entities have identity and represent the core business objects in our domain.

use std::collections::HashMap;
use uuid::Uuid;
use crate::domain::value_objects::{TypeDefinition, ValidationResult};

/// Unique identifier for a GraphQL schema
#[derive(Debug, Clone, PartialEq, Eq, Hash, serde::Serialize, serde::Deserialize)]
pub struct SchemaId(pub Uuid);

impl SchemaId {
    /// Generate a new schema ID
    pub fn new() -> Self {
        Self(Uuid::new_v4())
    }
}

impl Default for SchemaId {
    fn default() -> Self {
        Self::new()
    }
}

/// Schema version for tracking schema evolution
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SchemaVersion(pub String);

impl SchemaVersion {
    /// Create a new schema version
    pub fn new(version: impl Into<String>) -> Self {
        Self(version.into())
    }
}

/// GraphQL Schema entity
///
/// Represents a complete GraphQL schema with all type definitions.
/// This is the root entity that aggregates all schema information.
#[derive(Debug, Clone)]
pub struct Schema {
    /// Unique identifier for this schema
    pub id: SchemaId,
    
    /// Version of this schema
    pub version: SchemaVersion,
    
    /// All type definitions in this schema
    pub types: HashMap<String, TypeDefinition>,
    
    /// Name of the root query type (defaults to "Query")
    pub query_type: String,
    
    /// Name of the root mutation type (optional)
    pub mutation_type: Option<String>,
    
    /// Name of the root subscription type (optional)
    pub subscription_type: Option<String>,
}

impl Schema {
    /// Create a new schema with the given ID and version
    pub fn new(id: SchemaId, version: SchemaVersion) -> Self {
        Self {
            id,
            version,
            types: HashMap::new(),
            query_type: "Query".to_string(),
            mutation_type: None,
            subscription_type: None,
        }
    }
    
    /// Add a type definition to this schema
    pub fn add_type(&mut self, name: String, type_def: TypeDefinition) {
        self.types.insert(name, type_def);
    }
    
    /// Get a type definition by name
    pub fn get_type(&self, name: &str) -> Option<&TypeDefinition> {
        self.types.get(name)
    }
    
    /// Check if this schema is valid
    pub fn validate(&self) -> ValidationResult {
        // Basic validation - more comprehensive validation will be implemented
        // in the SchemaValidator service
        if self.types.get(&self.query_type).is_none() {
            ValidationResult::invalid(format!("Query type '{}' is not defined", self.query_type))
        } else {
            ValidationResult::valid()
        }
    }
}

/// Unique identifier for a GraphQL query
#[derive(Debug, Clone, PartialEq, Eq, Hash, serde::Serialize, serde::Deserialize)]
pub struct QueryId(pub Uuid);

impl QueryId {
    /// Generate a new query ID
    pub fn new() -> Self {
        Self(Uuid::new_v4())
    }
}

impl Default for QueryId {
    fn default() -> Self {
        Self::new()
    }
}

/// GraphQL Query entity
///
/// Represents a parsed and validated GraphQL query that can be executed.
#[derive(Debug, Clone)]
pub struct Query {
    /// Unique identifier for this query
    pub id: QueryId,
    
    /// The query string
    pub query_string: String,
    
    /// Optional operation name if specified in the query
    pub operation_name: Option<String>,
    
    /// Variables provided with the query
    pub variables: HashMap<String, serde_json::Value>,
    
    /// Validation status of this query
    pub validation_result: ValidationResult,
}

impl Query {
    /// Create a new query with the given query string
    pub fn new(query_string: String) -> Self {
        Self {
            id: QueryId::new(),
            query_string,
            operation_name: None,
            variables: HashMap::new(),
            validation_result: ValidationResult::Pending,
        }
    }
    
    /// Set the operation name for this query
    pub fn with_operation_name(mut self, operation_name: String) -> Self {
        self.operation_name = Some(operation_name);
        self
    }
    
    /// Set variables for this query
    pub fn with_variables(mut self, variables: HashMap<String, serde_json::Value>) -> Self {
        self.variables = variables;
        self
    }
    
    /// Mark this query as validated
    pub fn mark_validated(&mut self, result: ValidationResult) {
        self.validation_result = result;
    }
    
    /// Check if this query is valid and ready for execution
    pub fn is_valid(&self) -> bool {
        matches!(self.validation_result, ValidationResult::Valid)
    }
}
