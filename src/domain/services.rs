//! Domain services for GraphQL operations
//!
//! Services contain domain logic that doesn't naturally belong to entities or value objects.

use crate::domain::{
    entities::{query::Query, schema::Schema},
    value_objects::{ExecutionResult, GraphQLError, ValidationResult},
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
            errors.push(GraphQLError::validation_error(format!(
                "Query type '{}' is not defined",
                schema.query_type
            )));
        }

        // Rule 2: Mutation type must exist if specified
        if let Some(mutation_type) = &schema.mutation_type {
            if schema.get_type(mutation_type).is_none() {
                errors.push(GraphQLError::validation_error(format!(
                    "Mutation type '{}' is not defined",
                    mutation_type
                )));
            }
        }

        // Rule 3: Subscription type must exist if specified
        if let Some(subscription_type) = &schema.subscription_type {
            if schema.get_type(subscription_type).is_none() {
                errors.push(GraphQLError::validation_error(format!(
                    "Subscription type '{}' is not defined",
                    subscription_type
                )));
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
    async fn execute(&self, query: &Query, schema: &Schema) -> ExecutionResult {
        // Basic implementation - real execution engine will be implemented later
        if !query.is_valid() {
            return ExecutionResult::error(vec![crate::domain::value_objects::GraphQLError::new(
                "Query is not valid".to_string(),
            )]);
        }

        // Parse the query string into AST
        use crate::infrastructure::query_parser::QueryParser;
        let mut parser = QueryParser::new(query.query_string());
        let document = match parser.parse_document() {
            Ok(doc) => doc,
            Err(parse_error) => {
                return ExecutionResult::error(vec![
                    crate::domain::value_objects::GraphQLError::new(format!(
                        "Query parse error: {}", 
                        parse_error
                    ))
                ]);
            }
        };

        // For now, execute a simple field resolution based on the query
        match self.execute_document(&document, schema, query.variables()).await {
            Ok(data) => ExecutionResult::success(data),
            Err(error) => ExecutionResult::error(vec![error]),
        }
    }
}

impl QueryExecutor {
    /// Execute a parsed GraphQL document against a schema
    async fn execute_document(
        &self,
        document: &crate::infrastructure::query_parser::Document,
        schema: &Schema,
        variables: &Option<serde_json::Value>,
    ) -> Result<serde_json::Value, crate::domain::value_objects::GraphQLError> {
        // Find the operation to execute
        let operation = self.find_operation(document, None)?;

        // Execute based on operation type
        match operation.operation_type {
            crate::infrastructure::query_parser::OperationType::Query => {
                self.execute_query_operation(operation, schema, variables).await
            }
            crate::infrastructure::query_parser::OperationType::Mutation => {
                self.execute_mutation_operation(operation, schema, variables).await
            }
            crate::infrastructure::query_parser::OperationType::Subscription => {
                Err(crate::domain::value_objects::GraphQLError::new(
                    "Subscriptions are not yet implemented".to_string(),
                ))
            }
        }
    }

    /// Find the operation to execute from the document
    fn find_operation<'a>(
        &self,
        document: &'a crate::infrastructure::query_parser::Document,
        operation_name: Option<&str>,
    ) -> Result<&'a crate::infrastructure::query_parser::OperationDefinition, crate::domain::value_objects::GraphQLError> {
        use crate::infrastructure::query_parser::Definition;

        let operations: Vec<_> = document
            .definitions
            .iter()
            .filter_map(|def| match def {
                Definition::Operation(op) => Some(op),
                Definition::Fragment(_) => None,
            })
            .collect();

        match operations.len() {
            0 => Err(crate::domain::value_objects::GraphQLError::new(
                "No operations found in document".to_string(),
            )),
            1 => Ok(operations[0]),
            _ => {
                // Multiple operations require operation name
                if let Some(name) = operation_name {
                    operations
                        .iter()
                        .find(|op| op.name.as_ref() == Some(&name.to_string()))
                        .copied()
                        .ok_or_else(|| {
                            crate::domain::value_objects::GraphQLError::new(format!(
                                "Operation '{}' not found",
                                name
                            ))
                        })
                } else {
                    Err(crate::domain::value_objects::GraphQLError::new(
                        "Must provide operation name when document contains multiple operations"
                            .to_string(),
                    ))
                }
            }
        }
    }

    /// Execute a query operation
    async fn execute_query_operation(
        &self,
        operation: &crate::infrastructure::query_parser::OperationDefinition,
        schema: &Schema,
        _variables: &Option<serde_json::Value>,
    ) -> Result<serde_json::Value, crate::domain::value_objects::GraphQLError> {
        // Get the Query root type from schema
        let query_root = schema.query_type().map_err(|e| {
            crate::domain::value_objects::GraphQLError::new(format!(
                "Schema error: {}",
                e
            ))
        })?;

        // Execute the selection set on the query root type
        self.execute_selection_set(&operation.selection_set, query_root, &serde_json::Value::Null).await
    }

    /// Execute a mutation operation (stub for now)
    async fn execute_mutation_operation(
        &self,
        _operation: &crate::infrastructure::query_parser::OperationDefinition,
        _schema: &Schema,
        _variables: &Option<serde_json::Value>,
    ) -> Result<serde_json::Value, crate::domain::value_objects::GraphQLError> {
        Err(crate::domain::value_objects::GraphQLError::new(
            "Mutations are not yet implemented".to_string(),
        ))
    }

    /// Execute a selection set against a GraphQL type
    async fn execute_selection_set(
        &self,
        selection_set: &crate::infrastructure::query_parser::SelectionSet,
        object_type: &crate::domain::entities::types::GraphQLType,
        _parent_value: &serde_json::Value,
    ) -> Result<serde_json::Value, crate::domain::value_objects::GraphQLError> {
        use crate::domain::entities::types::GraphQLType;
        use crate::infrastructure::query_parser::Selection;
        use serde_json::Map;

        // Ensure we're working with an Object type
        let object_def = match object_type {
            GraphQLType::Object(obj) => obj,
            _ => {
                return Err(crate::domain::value_objects::GraphQLError::new(
                    "Can only execute selection sets on Object types".to_string(),
                ))
            }
        };

        let mut result = Map::new();

        for selection in &selection_set.selections {
            match selection {
                Selection::Field(field) => {
                    let field_result = self.execute_field(field, object_def).await?;
                    let result_name = field.alias.as_ref().unwrap_or(&field.name);
                    result.insert(result_name.clone(), field_result);
                }
                Selection::InlineFragment(_) => {
                    // TODO: Implement inline fragments
                    return Err(crate::domain::value_objects::GraphQLError::new(
                        "Inline fragments are not yet implemented".to_string(),
                    ));
                }
                Selection::FragmentSpread(_) => {
                    // TODO: Implement fragment spreads
                    return Err(crate::domain::value_objects::GraphQLError::new(
                        "Fragment spreads are not yet implemented".to_string(),
                    ));
                }
            }
        }

        Ok(serde_json::Value::Object(result))
    }

    /// Execute a field selection
    async fn execute_field(
        &self,
        field: &crate::infrastructure::query_parser::Field,
        object_def: &crate::domain::entities::types::ObjectType,
    ) -> Result<serde_json::Value, crate::domain::value_objects::GraphQLError> {
        // Find the field definition in the object type
        let field_def = object_def.fields.get(&field.name).ok_or_else(|| {
            crate::domain::value_objects::GraphQLError::new(format!(
                "Field '{}' not found on type '{}'",
                field.name, object_def.name
            ))
        })?;

        // For now, return mock data based on the field type
        self.resolve_field_value(&field_def.field_type, &field.name).await
    }

    /// Resolve a field value based on its type (mock implementation)  
    fn resolve_field_value<'a>(
        &'a self,
        field_type: &'a crate::domain::entities::types::GraphQLType,
        field_name: &'a str,
    ) -> std::pin::Pin<Box<dyn std::future::Future<Output = Result<serde_json::Value, crate::domain::value_objects::GraphQLError>> + Send + 'a>> {
        Box::pin(async move {
            use crate::domain::entities::types::{GraphQLType, ScalarType};

            match field_type {
                GraphQLType::Scalar(scalar) => match scalar {
                    ScalarType::String => Ok(serde_json::Value::String(format!("Mock {}", field_name))),
                    ScalarType::Int => Ok(serde_json::Value::Number(serde_json::Number::from(42))),
                    ScalarType::Float => Ok(serde_json::Value::Number(
                        serde_json::Number::from_f64(3.14).unwrap()
                    )),
                    ScalarType::Boolean => Ok(serde_json::Value::Bool(true)),
                    ScalarType::ID => Ok(serde_json::Value::String(format!("id_{}", field_name))),
                    ScalarType::Custom(name) => Ok(serde_json::Value::String(format!(
                        "Custom scalar: {} for field: {}",
                        name, field_name
                    ))),
                },
                GraphQLType::Object(_) => {
                    // For object types, we would need to recursively execute selection sets
                    // For now, return a placeholder
                    Ok(serde_json::json!({
                        "__typename": "Object",
                        "message": format!("Object field: {}", field_name)
                    }))
                }
                GraphQLType::List(_) => {
                    // Return a mock list
                    Ok(serde_json::Value::Array(vec![
                        serde_json::Value::String(format!("{}_item_1", field_name)),
                        serde_json::Value::String(format!("{}_item_2", field_name)),
                    ]))
                }
                GraphQLType::NonNull(inner) => {
                    // Unwrap the non-null and resolve the inner type
                    self.resolve_field_value(inner, field_name).await
                }
                _ => Ok(serde_json::Value::String(format!(
                    "Unsupported type for field: {}",
                    field_name
                ))),
            }
        })
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
