//! Tests for GraphQL subscription functionality

use graphql_rs::domain::{
    entities::{
        schema::Schema,
        query::Query,
        types::{GraphQLType, ObjectType, FieldDefinition, ScalarType},
        ids::{SchemaId, SchemaVersion}
    },
    services::{QueryExecutor, SchemaValidator, QueryExecution},
};
use std::collections::HashMap;

/// Helper function to create a test schema with subscription support
fn create_subscription_schema() -> Schema {
    let mut types = HashMap::new();
    
    // Built-in scalar types
    types.insert("String".to_string(), GraphQLType::Scalar(ScalarType::String));
    
    // Subscription root type
    let mut subscription_fields = HashMap::new();
    subscription_fields.insert("messageAdded".to_string(), FieldDefinition {
        name: "messageAdded".to_string(),
        description: Some("Subscribe to new messages".to_string()),
        field_type: GraphQLType::NonNull(Box::new(GraphQLType::Scalar(ScalarType::String))),
        arguments: HashMap::new(),
        deprecation_reason: None,
    });
    
    let subscription_type = GraphQLType::Object(ObjectType {
        name: "Subscription".to_string(),
        description: Some("Root subscription type".to_string()),
        fields: subscription_fields,
        interfaces: vec![],
    });
    types.insert("Subscription".to_string(), subscription_type);
    
    // Query root type (required by GraphQL spec)
    let mut query_fields = HashMap::new();
    query_fields.insert("hello".to_string(), FieldDefinition {
        name: "hello".to_string(),
        description: Some("A simple greeting".to_string()),
        field_type: GraphQLType::NonNull(Box::new(GraphQLType::Scalar(ScalarType::String))),
        arguments: HashMap::new(),
        deprecation_reason: None,
    });
    
    let query_type = GraphQLType::Object(ObjectType {
        name: "Query".to_string(),
        description: Some("Root query type".to_string()),
        fields: query_fields,
        interfaces: vec![],
    });
    types.insert("Query".to_string(), query_type);
    
    Schema {
        id: SchemaId::new(),
        version: SchemaVersion::new("1.0.0"),
        query_type: "Query".to_string(),
        mutation_type: None,
        subscription_type: Some("Subscription".to_string()),
        types,
        directives: HashMap::new(),
        description: Some("Test schema with subscription support".to_string()),
    }
}

#[test]
fn test_subscription_schema_validation() {
    let schema = create_subscription_schema();
    let validator = SchemaValidator::new();
    
    let validation_result = validator.validate(&schema);
    assert!(validation_result.is_valid(), "Subscription schema should be valid");
}

#[tokio::test]
async fn test_subscription_execution_transport_error() {
    let schema = create_subscription_schema();
    let executor = QueryExecutor::new();
    
    let subscription = "subscription { messageAdded }";
    let mut query = Query::new(subscription.to_string());
    
    // Manually mark as valid to test our subscription logic
    use graphql_rs::domain::value_objects::ValidationResult;
    query.mark_validated(ValidationResult::Valid);
    
    let result = executor.execute(&query, &schema).await;
    assert!(!result.errors.is_empty(), "Should have errors");
    
    // Debug the actual error message
    println!("Error message: {}", result.errors[0].message);
    println!("Extensions: {:?}", result.errors[0].extensions);
    
    // Check if it's a subscription-related error
    let error_msg = &result.errors[0].message;
    let is_subscription_error = error_msg.contains("Subscription") || 
                                error_msg.contains("subscription") ||
                                error_msg.contains("WebSocket");
    assert!(is_subscription_error, "Should be a subscription-related error, got: {}", error_msg);
}

#[tokio::test]
async fn test_subscription_error_handling() {
    let schema = create_subscription_schema();
    let executor = QueryExecutor::new();
    
    let subscription = "subscription { messageAdded }";
    let query = Query::new(subscription.to_string());
    
    let result = executor.execute(&query, &schema).await;
    
    // Should have errors
    assert!(!result.errors.is_empty());
    
    // Debug what we got
    println!("Error message: {}", result.errors[0].message);
    println!("Extensions: {:?}", result.errors[0].extensions);
}

#[test]
fn test_subscription_result_methods() {
    use graphql_rs::domain::value_objects::{GraphQLError, SubscriptionResult};
    
    // Test error constructor
    let error = GraphQLError::new("Test error".to_string());
    let error_result = SubscriptionResult::with_error(error);
    assert!(error_result.has_errors());
    assert!(!error_result.has_stream());
}
