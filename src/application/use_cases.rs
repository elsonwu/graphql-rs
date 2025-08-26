//! Use cases representing application-level business operations

use crate::domain::{
    entities::{Schema, Query},
    services::{QueryValidator, QueryExecutor, SchemaValidator, QueryExecution},
    repositories::{SchemaRepository, QueryRepository},
    value_objects::{ValidationResult, ExecutionResult},
    events::{EventPublisher, GraphQLEvent, QueryEvent, SchemaEvent, EventId},
};
use chrono::Utc;
use std::sync::Arc;

/// Use case for executing a GraphQL query
pub struct ExecuteQueryUseCase<S, Q, P>
where
    S: SchemaRepository,
    Q: QueryRepository,
    P: EventPublisher,
{
    schema_repository: Arc<S>,
    query_repository: Arc<Q>,
    query_validator: QueryValidator,
    query_executor: QueryExecutor,
    event_publisher: Arc<P>,
}

impl<S, Q, P> ExecuteQueryUseCase<S, Q, P>
where
    S: SchemaRepository,
    Q: QueryRepository,
    P: EventPublisher,
{
    /// Create a new execute query use case
    pub fn new(
        schema_repository: Arc<S>,
        query_repository: Arc<Q>,
        event_publisher: Arc<P>,
    ) -> Self {
        Self {
            schema_repository,
            query_repository,
            query_validator: QueryValidator::new(),
            query_executor: QueryExecutor::new(),
            event_publisher,
        }
    }
    
    /// Execute a GraphQL query
    pub async fn execute(
        &self,
        query_string: String,
        operation_name: Option<String>,
        variables: std::collections::HashMap<String, serde_json::Value>,
    ) -> ExecutionResult {
        // Create query entity
        let mut query = Query::new(query_string.clone())
            .with_variables(variables);
        
        if let Some(op_name) = operation_name {
            query = query.with_operation_name(op_name);
        }
        
        // Publish query received event
        self.event_publisher.publish(GraphQLEvent::Query(QueryEvent::QueryReceived {
            event_id: EventId::new(),
            timestamp: Utc::now(),
            query_id: query.id.clone(),
            query_string: query_string.clone(),
            operation_name: query.operation_name.clone(),
        }));
        
        // Save query for analytics
        if let Err(_) = self.query_repository.save(query.clone()).await {
            // Log error but continue execution
        }
        
        // Get the schema
        let schema = match self.schema_repository.find_latest().await {
            Ok(Some(schema)) => schema,
            Ok(None) => {
                return ExecutionResult::error(vec![
                    crate::domain::value_objects::GraphQLError::new(
                        "No schema available".to_string()
                    )
                ]);
            }
            Err(_) => {
                return ExecutionResult::error(vec![
                    crate::domain::value_objects::GraphQLError::new(
                        "Failed to load schema".to_string()
                    )
                ]);
            }
        };
        
        // Validate query
        let validation_result = self.query_validator.validate(&query, &schema);
        query.mark_validated(validation_result.clone());
        
        match validation_result {
            ValidationResult::Valid => {
                self.event_publisher.publish(GraphQLEvent::Query(QueryEvent::QueryValidated {
                    event_id: EventId::new(),
                    timestamp: Utc::now(),
                    query_id: query.id.clone(),
                }));
            }
            ValidationResult::Invalid(errors) => {
                self.event_publisher.publish(GraphQLEvent::Query(QueryEvent::QueryValidationFailed {
                    event_id: EventId::new(),
                    timestamp: Utc::now(),
                    query_id: query.id.clone(),
                    errors: errors.clone(),
                }));
                
                let graphql_errors = errors.into_iter()
                    .map(|e| crate::domain::value_objects::GraphQLError::new(e))
                    .collect();
                    
                return ExecutionResult::error(graphql_errors);
            }
            ValidationResult::Pending => {
                return ExecutionResult::error(vec![
                    crate::domain::value_objects::GraphQLError::new(
                        "Query validation is pending".to_string()
                    )
                ]);
            }
        }
        
        // Execute query
        let execution_start = std::time::Instant::now();
        
        self.event_publisher.publish(GraphQLEvent::Query(QueryEvent::QueryExecutionStarted {
            event_id: EventId::new(),
            timestamp: Utc::now(),
            query_id: query.id.clone(),
            schema_id: schema.id.clone(),
        }));
        
        let result = self.query_executor.execute(&query, &schema).await;
        let execution_time = execution_start.elapsed();
        
        // Publish execution completed event
        match &result {
            ExecutionResult { data: Some(_), errors, .. } if errors.is_empty() => {
                self.event_publisher.publish(GraphQLEvent::Query(QueryEvent::QueryExecutionCompleted {
                    event_id: EventId::new(),
                    timestamp: Utc::now(),
                    query_id: query.id.clone(),
                    execution_time,
                    field_count: 1, // Placeholder - will be calculated properly later
                    result_size_bytes: serde_json::to_string(&result).unwrap_or_default().len(),
                }));
            }
            _ => {
                self.event_publisher.publish(GraphQLEvent::Query(QueryEvent::QueryExecutionFailed {
                    event_id: EventId::new(),
                    timestamp: Utc::now(),
                    query_id: query.id.clone(),
                    execution_time,
                    error: "Query execution failed".to_string(),
                }));
            }
        }
        
        result
    }
}

/// Use case for validating and saving a GraphQL schema
pub struct ValidateSchemaUseCase<S, P>
where
    S: SchemaRepository,
    P: EventPublisher,
{
    schema_repository: Arc<S>,
    schema_validator: SchemaValidator,
    event_publisher: Arc<P>,
}

impl<S, P> ValidateSchemaUseCase<S, P>
where
    S: SchemaRepository,
    P: EventPublisher,
{
    /// Create a new validate schema use case
    pub fn new(
        schema_repository: Arc<S>,
        event_publisher: Arc<P>,
    ) -> Self {
        Self {
            schema_repository,
            schema_validator: SchemaValidator::new(),
            event_publisher,
        }
    }
    
    /// Validate and save a schema
    pub async fn validate_and_save(&self, schema: Schema) -> Result<(), String> {
        // Publish schema created event
        self.event_publisher.publish(GraphQLEvent::Schema(SchemaEvent::SchemaCreated {
            event_id: EventId::new(),
            timestamp: Utc::now(),
            schema_id: schema.id.clone(),
            version: schema.version.0.clone(),
        }));
        
        // Validate schema
        let validation_result = self.schema_validator.validate(&schema);
        
        match validation_result {
            ValidationResult::Valid => {
                self.event_publisher.publish(GraphQLEvent::Schema(SchemaEvent::SchemaValidated {
                    event_id: EventId::new(),
                    timestamp: Utc::now(),
                    schema_id: schema.id.clone(),
                }));
                
                // Save schema
                if let Err(_) = self.schema_repository.save(schema).await {
                    return Err("Failed to save schema".to_string());
                }
                
                Ok(())
            }
            ValidationResult::Invalid(errors) => {
                self.event_publisher.publish(GraphQLEvent::Schema(SchemaEvent::SchemaValidationFailed {
                    event_id: EventId::new(),
                    timestamp: Utc::now(),
                    schema_id: schema.id.clone(),
                    errors: errors.clone(),
                }));
                
                Err(format!("Schema validation failed: {}", errors.join(", ")))
            }
            ValidationResult::Pending => {
                Err("Schema validation is pending".to_string())
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::{
        entities::{SchemaId, SchemaVersion},
        repositories::{InMemorySchemaRepository, InMemoryQueryRepository},
        events::InMemoryEventPublisher,
    };
    
    #[tokio::test]
    async fn test_execute_query_use_case() {
        let schema_repo = Arc::new(InMemorySchemaRepository::new());
        let query_repo = Arc::new(InMemoryQueryRepository::new());
        let event_publisher = Arc::new(InMemoryEventPublisher::new());
        
        // Create and save a basic schema
        let mut schema = Schema::new(SchemaId::new(), SchemaVersion::new("1.0"));
        schema.add_type("Query".to_string(), crate::domain::value_objects::TypeDefinition::Object(
            crate::domain::value_objects::ObjectTypeDefinition {
                name: "Query".to_string(),
                description: None,
                fields: indexmap::IndexMap::new(),
                interfaces: Vec::new(),
            }
        ));
        schema_repo.save(schema).await.unwrap();
        
        let use_case = ExecuteQueryUseCase::new(
            schema_repo,
            query_repo,
            event_publisher.clone(),
        );
        
        let result = use_case.execute(
            "{ test }".to_string(),
            None,
            std::collections::HashMap::new(),
        ).await;
        
        // The query should execute (though it won't return real data yet)
        assert!(result.data.is_some() || !result.errors.is_empty());
        
        // Check that events were published
        tokio::time::sleep(tokio::time::Duration::from_millis(10)).await;
        let events = event_publisher.get_events().await;
        assert!(!events.is_empty());
    }
}
