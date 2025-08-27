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
    #[must_use]
    pub fn new() -> Self {
        Self
    }

    /// Validate a GraphQL schema
    #[must_use]
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
                    "Mutation type '{mutation_type}' is not defined"
                )));
            }
        }

        // Rule 3: Subscription type must exist if specified
        if let Some(subscription_type) = &schema.subscription_type {
            if schema.get_type(subscription_type).is_none() {
                errors.push(GraphQLError::validation_error(format!(
                    "Subscription type '{subscription_type}' is not defined"
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
    #[must_use]
    pub fn new() -> Self {
        Self
    }

    /// Validate a GraphQL query against a schema
    #[must_use]
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
    #[must_use]
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
                        "Query parse error: {parse_error}"
                    )),
                ]);
            },
        };

        // For now, execute a simple field resolution based on the query
        match self
            .execute_document(&document, schema, query.variables())
            .await
        {
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
                self.execute_query_operation(operation, schema, variables)
                    .await
            },
            crate::infrastructure::query_parser::OperationType::Mutation => {
                self.execute_mutation_operation(operation, schema, variables)
                    .await
            },
            crate::infrastructure::query_parser::OperationType::Subscription => {
                Err(crate::domain::value_objects::GraphQLError::new(
                    "Subscriptions are not yet implemented".to_string(),
                ))
            },
        }
    }

    /// Find the operation to execute from the document
    fn find_operation<'a>(
        &self,
        document: &'a crate::infrastructure::query_parser::Document,
        operation_name: Option<&str>,
    ) -> Result<
        &'a crate::infrastructure::query_parser::OperationDefinition,
        crate::domain::value_objects::GraphQLError,
    > {
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
                                "Operation '{name}' not found"
                            ))
                        })
                } else {
                    Err(crate::domain::value_objects::GraphQLError::new(
                        "Must provide operation name when document contains multiple operations"
                            .to_string(),
                    ))
                }
            },
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
            crate::domain::value_objects::GraphQLError::new(format!("Schema error: {e}"))
        })?;

        // Execute the selection set on the query root type
        self.execute_selection_set(
            &operation.selection_set,
            query_root,
            &serde_json::Value::Null,
        )
        .await
    }

    /// Execute a mutation operation
    /// Mutations execute sequentially (unlike queries which can be parallel)
    async fn execute_mutation_operation(
        &self,
        operation: &crate::infrastructure::query_parser::OperationDefinition,
        schema: &Schema,
        variables: &Option<serde_json::Value>,
    ) -> Result<serde_json::Value, crate::domain::value_objects::GraphQLError> {
        // Get the Mutation root type from the schema
        let mutation_type_name = schema.mutation_type.as_ref().ok_or_else(|| {
            crate::domain::value_objects::GraphQLError::new(
                "Schema does not define a Mutation type".to_string(),
            )
        })?;

        let mutation_type = schema.get_type(mutation_type_name).ok_or_else(|| {
            crate::domain::value_objects::GraphQLError::new(format!(
                "Mutation type '{mutation_type_name}' not found in schema"
            ))
        })?;

        // Execute the mutation selection set sequentially
        // Unlike queries, mutations must execute in order to maintain consistency
        self.execute_mutation_selection_set_sequential(
            &operation.selection_set,
            mutation_type,
            variables,
        )
        .await
    }

    /// Execute mutation fields sequentially (one by one, not in parallel)
    async fn execute_mutation_selection_set_sequential(
        &self,
        selection_set: &crate::infrastructure::query_parser::SelectionSet,
        mutation_type: &crate::domain::entities::types::GraphQLType,
        _variables: &Option<serde_json::Value>,
    ) -> Result<serde_json::Value, crate::domain::value_objects::GraphQLError> {
        use crate::domain::entities::types::GraphQLType;
        use crate::infrastructure::query_parser::Selection;
        use serde_json::Map;

        // Ensure we're working with an Object type
        let GraphQLType::Object(object_def) = mutation_type else {
            return Err(crate::domain::value_objects::GraphQLError::new(format!(
                "Mutation type must be an Object type, got: {mutation_type:?}"
            )));
        };

        let mut result_map = Map::new();

        // 🚨 CRITICAL: Execute mutations sequentially, not in parallel!
        // Each mutation must see the effects of the previous ones
        for selection in &selection_set.selections {
            match selection {
                Selection::Field(field) => {
                    // Execute this mutation field and wait for completion before proceeding
                    let field_result = self.execute_mutation_field(field, object_def).await?;

                    // Use alias if provided, otherwise use field name
                    let result_key = field.alias.as_ref().unwrap_or(&field.name);
                    result_map.insert(result_key.clone(), field_result);
                },
                Selection::InlineFragment(_) => {
                    // For now, inline fragments in mutations are not supported
                    return Err(crate::domain::value_objects::GraphQLError::new(
                        "Inline fragments in mutations are not yet supported".to_string(),
                    ));
                },
                Selection::FragmentSpread(_) => {
                    // For now, fragment spreads in mutations are not supported
                    return Err(crate::domain::value_objects::GraphQLError::new(
                        "Fragment spreads in mutations are not yet supported".to_string(),
                    ));
                },
            }
        }

        Ok(serde_json::Value::Object(result_map))
    }

    /// Execute a single mutation field with side effects
    async fn execute_mutation_field(
        &self,
        field: &crate::infrastructure::query_parser::Field,
        object_type: &crate::domain::entities::types::ObjectType,
    ) -> Result<serde_json::Value, crate::domain::value_objects::GraphQLError> {
        // Check if the field exists in the mutation type
        let field_def = object_type.fields.get(&field.name).ok_or_else(|| {
            crate::domain::value_objects::GraphQLError::new(format!(
                "Field '{}' not found on Mutation type",
                field.name
            ))
        })?;

        // For demonstration purposes, we'll create mock mutation results
        // In a real implementation, this would call actual resolver functions
        // that perform the side effects (database operations, API calls, etc.)

        match field.name.as_str() {
            "createUser" => {
                // Mock user creation
                let user_data = serde_json::json!({
                    "id": format!("user_{}", uuid::Uuid::new_v4().to_string().split('-').next().unwrap()),
                    "name": self.get_argument_value(&field.arguments, "input")
                        .and_then(|input| Self::extract_string_from_value(&input, &["name"]))
                        .unwrap_or_else(|| "Unknown User".to_string()),
                    "email": self.get_argument_value(&field.arguments, "input")
                        .and_then(|input| Self::extract_string_from_value(&input, &["email"]))
                        .unwrap_or_else(|| "user@example.com".to_string()),
                    "createdAt": chrono::Utc::now().to_rfc3339(),
                    "isActive": true
                });

                // If there are sub-selections, resolve them
                if let Some(sub_selection_set) = &field.selection_set {
                    self.execute_mutation_sub_selection(sub_selection_set, &user_data, field_def)
                        .await
                } else {
                    Ok(user_data)
                }
            },
            "updateUser" => {
                // Mock user update
                let user_id = self
                    .get_argument_value(&field.arguments, "id")
                    .and_then(|id| id.as_str().map(|s| s.to_string()))
                    .unwrap_or_else(|| "unknown".to_string());

                let updated_user = serde_json::json!({
                    "id": user_id,
                    "name": self.get_argument_value(&field.arguments, "input")
                        .and_then(|input| Self::extract_string_from_value(&input, &["name"]))
                        .unwrap_or_else(|| "Updated User".to_string()),
                    "email": self.get_argument_value(&field.arguments, "input")
                        .and_then(|input| Self::extract_string_from_value(&input, &["email"]))
                        .unwrap_or_else(|| "updated@example.com".to_string()),
                    "updatedAt": chrono::Utc::now().to_rfc3339(),
                    "isActive": true
                });

                // If there are sub-selections, resolve them
                if let Some(sub_selection_set) = &field.selection_set {
                    self.execute_mutation_sub_selection(sub_selection_set, &updated_user, field_def)
                        .await
                } else {
                    Ok(updated_user)
                }
            },
            "deleteUser" => {
                // Mock user deletion - typically returns boolean or deleted object
                let _user_id = self
                    .get_argument_value(&field.arguments, "id")
                    .and_then(|id| id.as_str().map(|s| s.to_string()))
                    .unwrap_or_else(|| "unknown".to_string());

                // Return boolean success for deletion
                Ok(serde_json::json!(true))
            },
            _ => {
                // For unknown mutations, return a generic success response
                // In a real implementation, this would be an error or call a dynamic resolver
                Ok(serde_json::json!({
                    "success": true,
                    "message": format!("Mutation '{}' executed successfully", field.name)
                }))
            },
        }
    }

    /// Execute sub-selections for mutation results
    async fn execute_mutation_sub_selection(
        &self,
        selection_set: &crate::infrastructure::query_parser::SelectionSet,
        parent_value: &serde_json::Value,
        _field_def: &crate::domain::entities::types::FieldDefinition,
    ) -> Result<serde_json::Value, crate::domain::value_objects::GraphQLError> {
        use crate::infrastructure::query_parser::Selection;
        use serde_json::Map;

        let mut result_map = Map::new();

        for selection in &selection_set.selections {
            match selection {
                Selection::Field(field) => {
                    // Extract the requested field from the parent value
                    let field_value = parent_value
                        .get(&field.name)
                        .cloned()
                        .unwrap_or(serde_json::Value::Null);

                    // Use alias if provided, otherwise use field name
                    let result_key = field.alias.as_ref().unwrap_or(&field.name);
                    result_map.insert(result_key.clone(), field_value);
                },
                Selection::InlineFragment(_) => {
                    return Err(crate::domain::value_objects::GraphQLError::new(
                        "Inline fragments in mutation sub-selections are not yet supported"
                            .to_string(),
                    ));
                },
                Selection::FragmentSpread(_) => {
                    return Err(crate::domain::value_objects::GraphQLError::new(
                        "Fragment spreads in mutation sub-selections are not yet supported"
                            .to_string(),
                    ));
                },
            }
        }

        Ok(serde_json::Value::Object(result_map))
    }

    /// Helper function to get argument value by name and convert to `serde_json::Value`
    fn get_argument_value(
        &self,
        arguments: &[crate::infrastructure::query_parser::Argument],
        name: &str,
    ) -> Option<serde_json::Value> {
        arguments
            .iter()
            .find(|arg| arg.name == name)
            .map(|arg| self.convert_query_value_to_json(&arg.value))
    }

    /// Convert query parser Value to `serde_json::Value`
    fn convert_query_value_to_json(
        &self,
        value: &crate::infrastructure::query_parser::Value,
    ) -> serde_json::Value {
        use crate::infrastructure::query_parser::Value;
        match value {
            Value::Variable(_) => {
                // For now, variables are not fully supported in mutations
                serde_json::Value::Null
            },
            Value::Int(i) => serde_json::Value::Number(serde_json::Number::from(*i)),
            Value::Float(f) => serde_json::Number::from_f64(*f)
                .map_or(serde_json::Value::Null, serde_json::Value::Number),
            Value::String(s) => serde_json::Value::String(s.clone()),
            Value::Boolean(b) => serde_json::Value::Bool(*b),
            Value::Null => serde_json::Value::Null,
            Value::Enum(e) => serde_json::Value::String(e.clone()),
            Value::List(list) => {
                let converted_list: Vec<serde_json::Value> = list
                    .iter()
                    .map(|v| self.convert_query_value_to_json(v))
                    .collect();
                serde_json::Value::Array(converted_list)
            },
            Value::Object(obj) => {
                let mut converted_obj = serde_json::Map::new();
                for (key, value) in obj {
                    converted_obj.insert(key.clone(), self.convert_query_value_to_json(value));
                }
                serde_json::Value::Object(converted_obj)
            },
        }
    }

    /// Helper function to extract string from argument value  
    fn extract_string_from_value(value: &serde_json::Value, path: &[&str]) -> Option<String> {
        let mut current = value;
        for key in path {
            current = current.get(key)?;
        }
        current.as_str().map(std::string::ToString::to_string)
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
        let GraphQLType::Object(object_def) = object_type else {
            return Err(crate::domain::value_objects::GraphQLError::new(
                "Can only execute selection sets on Object types".to_string(),
            ));
        };

        let mut result = Map::new();

        for selection in &selection_set.selections {
            match selection {
                Selection::Field(field) => {
                    let field_result = self.execute_field(field, object_def).await?;
                    let result_name = field.alias.as_ref().unwrap_or(&field.name);
                    result.insert(result_name.clone(), field_result);
                },
                Selection::InlineFragment(_) => {
                    // TODO: Implement inline fragments
                    return Err(crate::domain::value_objects::GraphQLError::new(
                        "Inline fragments are not yet implemented".to_string(),
                    ));
                },
                Selection::FragmentSpread(_) => {
                    // TODO: Implement fragment spreads
                    return Err(crate::domain::value_objects::GraphQLError::new(
                        "Fragment spreads are not yet implemented".to_string(),
                    ));
                },
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
        self.resolve_field_value(&field_def.field_type, &field.name)
            .await
    }

    /// Resolve a field value based on its type (mock implementation)  
    fn resolve_field_value<'a>(
        &'a self,
        field_type: &'a crate::domain::entities::types::GraphQLType,
        field_name: &'a str,
    ) -> std::pin::Pin<
        Box<
            dyn std::future::Future<
                    Output = Result<serde_json::Value, crate::domain::value_objects::GraphQLError>,
                > + Send
                + 'a,
        >,
    > {
        Box::pin(async move {
            use crate::domain::entities::types::{GraphQLType, ScalarType};

            match field_type {
                GraphQLType::Scalar(scalar) => match scalar {
                    ScalarType::String => {
                        Ok(serde_json::Value::String(format!("Mock {field_name}")))
                    },
                    ScalarType::Int => Ok(serde_json::Value::Number(serde_json::Number::from(42))),
                    ScalarType::Float => Ok(serde_json::Value::Number(
                        serde_json::Number::from_f64(std::f64::consts::PI).unwrap(),
                    )),
                    ScalarType::Boolean => Ok(serde_json::Value::Bool(true)),
                    ScalarType::ID => Ok(serde_json::Value::String(format!("id_{field_name}"))),
                    ScalarType::Custom(name) => Ok(serde_json::Value::String(format!(
                        "Custom scalar: {name} for field: {field_name}"
                    ))),
                },
                GraphQLType::Object(_) => {
                    // For object types, we would need to recursively execute selection sets
                    // For now, return a placeholder
                    Ok(serde_json::json!({
                        "__typename": "Object",
                        "message": format!("Object field: {}", field_name)
                    }))
                },
                GraphQLType::List(_) => {
                    // Return a mock list
                    Ok(serde_json::Value::Array(vec![
                        serde_json::Value::String(format!("{field_name}_item_1")),
                        serde_json::Value::String(format!("{field_name}_item_2")),
                    ]))
                },
                GraphQLType::NonNull(inner) => {
                    // Unwrap the non-null and resolve the inner type
                    self.resolve_field_value(inner, field_name).await
                },
                _ => Ok(serde_json::Value::String(format!(
                    "Unsupported type for field: {field_name}"
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

    #[tokio::test]
    async fn test_execute_create_user_mutation() {
        let executor = QueryExecutor::new();

        // Create a schema with Mutation type
        let mut schema = Schema::new("Query".to_string());
        schema.mutation_type = Some("Mutation".to_string());

        // Add Mutation type to schema
        use crate::domain::entities::types::{FieldDefinition, GraphQLType, ObjectType};
        use std::collections::HashMap;

        let mut mutation_fields = HashMap::new();
        mutation_fields.insert(
            "createUser".to_string(),
            FieldDefinition {
                name: "createUser".to_string(),
                description: Some("Create a new user".to_string()),
                field_type: GraphQLType::Object(ObjectType {
                    name: "User".to_string(),
                    description: Some("User object".to_string()),
                    fields: HashMap::new(),
                    interfaces: Vec::new(),
                }),
                arguments: HashMap::new(),
                deprecation_reason: None,
            },
        );

        let mutation_type = ObjectType {
            name: "Mutation".to_string(),
            description: Some("Root mutation type".to_string()),
            fields: mutation_fields,
            interfaces: Vec::new(),
        };

        schema.add_type(GraphQLType::Object(mutation_type)).unwrap();

        let mutation_query = "mutation { createUser { id name } }";

        let mut query = Query::new(mutation_query.to_string());
        // For now, mark the query as valid to bypass validation
        query.mark_validated(crate::domain::value_objects::ValidationResult::valid());

        let result = executor.execute(&query, &schema).await;

        assert!(
            result.data.is_some(),
            "Mutation should execute successfully"
        );

        let data = result.data.unwrap();
        let create_user_result = data
            .get("createUser")
            .expect("Should have createUser result");

        // Verify the returned data structure (using default values since arguments aren't parsed yet)
        assert!(create_user_result.get("id").is_some());
        assert_eq!(
            create_user_result.get("name").unwrap().as_str().unwrap(),
            "Unknown User"
        );
    }

    #[tokio::test]
    async fn test_mutation_without_mutation_type_in_schema() {
        let executor = QueryExecutor::new();
        let schema = Schema::new("Query".to_string()); // No mutation type defined

        let mutation_query = "mutation { createUser { id } }";

        let mut query = Query::new(mutation_query.to_string());
        query.mark_validated(crate::domain::value_objects::ValidationResult::valid());

        let result = executor.execute(&query, &schema).await;

        assert!(
            !result.errors.is_empty(),
            "Should fail when no Mutation type is defined"
        );

        let error_message = &result.errors[0].message;
        assert!(error_message.contains("Schema does not define a Mutation type"));
    }
}
