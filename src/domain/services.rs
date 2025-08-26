//! Domain services for GraphQL operations
//!
//! Services contain domain logic that doesn't naturally belong to entities or value objects.

use crate::domain::{
    entities::{schema::Schema, query::Query},
    value_objects::{ValidationResult, ExecutionResult, GraphQLError},
};
use async_trait::async_trait;

/// Service for validating GraphQL schemas
pub struct SchemaValidator;

impl SchemaValidator {
    /// Create a new schema validator
    pub fn new() -> Self {
        Self
    }
    
    /// Validate a GraphQL schema
    pub fn validate(&self, schema: &Schema) -> ValidationResult {
        let mut errors = Vec::new();
        
        // Rule 1: Schema must have a Query type
        if schema.get_type(&schema.query_type).is_none() {
            errors.push(GraphQLError::validation_error(format!("Query type '{}' is not defined", schema.query_type)));
        }
        
        // Rule 2: Mutation type must exist if specified
        if let Some(mutation_type) = &schema.mutation_type {
            if schema.get_type(mutation_type).is_none() {
                errors.push(GraphQLError::validation_error(format!("Mutation type '{}' is not defined", mutation_type)));
            }
        }
        
        // Rule 3: Subscription type must exist if specified
        if let Some(subscription_type) = &schema.subscription_type {
            if schema.get_type(subscription_type).is_none() {
                errors.push(GraphQLError::validation_error(format!("Subscription type '{}' is not defined", subscription_type)));
            }
        }
        
        // Additional validation rules will be added in later iterations
        
        if errors.is_empty() {
            ValidationResult::Valid
        } else {
            ValidationResult::Invalid(errors)
        }
    }
}

impl Default for SchemaValidator {
    fn default() -> Self {
        Self::new()
    }
}

/// Service for validating GraphQL queries against a schema
pub struct QueryValidator;

impl QueryValidator {
    /// Create a new query validator
    pub fn new() -> Self {
        Self
    }
    
    /// Validate a GraphQL query against a schema
    pub fn validate(&self, query: &Query, _schema: &Schema) -> ValidationResult {
        // Basic validation for now - comprehensive validation will be implemented later
        if query.is_empty() {
            ValidationResult::invalid("Query string cannot be empty".to_string())
        } else {
            // TODO: Parse and validate query syntax
            // TODO: Validate query against schema
            ValidationResult::Valid
        }
    }
}

impl Default for QueryValidator {
    fn default() -> Self {
        Self::new()
    }
}

/// Service for executing GraphQL queries
pub struct QueryExecutor;

impl QueryExecutor {
    /// Create a new query executor
    pub fn new() -> Self {
        Self
    }
}

impl Default for QueryExecutor {
    fn default() -> Self {
        Self::new()
    }
}

/// Async trait for query execution
#[async_trait]
pub trait QueryExecution {
    /// Execute a GraphQL query against a schema
    async fn execute(&self, query: &Query, schema: &Schema) -> ExecutionResult;
}

#[async_trait]
impl QueryExecution for QueryExecutor {
    async fn execute(&self, query: &Query, _schema: &Schema) -> ExecutionResult {
        // Basic implementation - real execution engine will be implemented later
        if !query.is_valid() {
            return ExecutionResult::error(vec![
                crate::domain::value_objects::GraphQLError::new(
                    "Query is not valid".to_string()
                )
            ]);
        }
        
        // TODO: Parse query
        // TODO: Execute query against schema
        // TODO: Return actual data
        
        ExecutionResult::success(serde_json::json!({
            "message": "Query execution not yet implemented"
        }))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_schema_validator_missing_query_type() {
        let schema = Schema::new("Query".to_string());
        let validator = SchemaValidator::new();
        
        let result = validator.validate(&schema);
        
        assert!(result.is_invalid());
    }
    
    #[test]
    fn test_query_validator_empty_query() {
        let query = Query::new(String::new());
        let schema = Schema::new("Query".to_string());
        let validator = QueryValidator::new();
        
        let result = validator.validate(&query, &schema);
        
        assert!(result.is_invalid());
    }
    
    #[tokio::test]
    async fn test_query_executor_invalid_query() {
        let mut query = Query::new("{ test }".to_string());
        query.mark_validated(ValidationResult::invalid("Test error".to_string()));
        
        let schema = Schema::new("Query".to_string());
        let executor = QueryExecutor::new();
        
        let result = executor.execute(&query, &schema).await;
        
        assert!(!result.errors.is_empty());
    }
}
