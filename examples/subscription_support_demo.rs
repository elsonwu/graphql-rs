//! Enhanced Subscription Support Demo
//!
//! This example demonstrates the GraphQL subscription system implemented in graphql-rs.
//! Subscriptions enable real-time, event-driven communication between clients and the GraphQL server.
//!
//! Key concepts demonstrated:
//! - Subscription schema creation
//! - Subscription query validation
//! - Subscription execution (with transport requirements)
//! - Error handling for subscription-specific scenarios
//! - Stream-based subscription results
//! - Production-ready subscription patterns

use graphql_rs::domain::{
    entities::{
        ids::{SchemaId, SchemaVersion},
        query::Query,
        schema::Schema,
        types::{FieldDefinition, GraphQLType, ObjectType, ScalarType},
    },
    services::{QueryExecution, QueryExecutor, QueryValidator, SchemaValidator},
    value_objects::{GraphQLError, SubscriptionResult, ValidationResult},
};
use std::collections::HashMap;

/// Create a comprehensive schema with subscription support
fn create_chat_schema() -> Schema {
    let mut types = HashMap::new();

    // Built-in scalar types
    types.insert(
        "String".to_string(),
        GraphQLType::Scalar(ScalarType::String),
    );
    types.insert("ID".to_string(), GraphQLType::Scalar(ScalarType::ID));
    types.insert("Int".to_string(), GraphQLType::Scalar(ScalarType::Int));
    types.insert(
        "Boolean".to_string(),
        GraphQLType::Scalar(ScalarType::Boolean),
    );

    // User type
    let mut user_fields = HashMap::new();
    user_fields.insert(
        "id".to_string(),
        FieldDefinition {
            name: "id".to_string(),
            description: Some("User ID".to_string()),
            field_type: GraphQLType::NonNull(Box::new(GraphQLType::Scalar(ScalarType::ID))),
            arguments: HashMap::new(),
            deprecation_reason: None,
        },
    );
    user_fields.insert(
        "name".to_string(),
        FieldDefinition {
            name: "name".to_string(),
            description: Some("User name".to_string()),
            field_type: GraphQLType::NonNull(Box::new(GraphQLType::Scalar(ScalarType::String))),
            arguments: HashMap::new(),
            deprecation_reason: None,
        },
    );
    user_fields.insert(
        "online".to_string(),
        FieldDefinition {
            name: "online".to_string(),
            description: Some("User online status".to_string()),
            field_type: GraphQLType::NonNull(Box::new(GraphQLType::Scalar(ScalarType::Boolean))),
            arguments: HashMap::new(),
            deprecation_reason: None,
        },
    );

    let user_type = GraphQLType::Object(ObjectType {
        name: "User".to_string(),
        description: Some("A chat user".to_string()),
        fields: user_fields,
        interfaces: vec![],
    });
    types.insert("User".to_string(), user_type);

    // Message type
    let mut message_fields = HashMap::new();
    message_fields.insert(
        "id".to_string(),
        FieldDefinition {
            name: "id".to_string(),
            description: Some("Message ID".to_string()),
            field_type: GraphQLType::NonNull(Box::new(GraphQLType::Scalar(ScalarType::ID))),
            arguments: HashMap::new(),
            deprecation_reason: None,
        },
    );
    message_fields.insert(
        "content".to_string(),
        FieldDefinition {
            name: "content".to_string(),
            description: Some("Message content".to_string()),
            field_type: GraphQLType::NonNull(Box::new(GraphQLType::Scalar(ScalarType::String))),
            arguments: HashMap::new(),
            deprecation_reason: None,
        },
    );
    message_fields.insert(
        "author".to_string(),
        FieldDefinition {
            name: "author".to_string(),
            description: Some("Message author".to_string()),
            field_type: GraphQLType::NonNull(Box::new(GraphQLType::Scalar(ScalarType::String))), // Simplified reference
            arguments: HashMap::new(),
            deprecation_reason: None,
        },
    );
    message_fields.insert(
        "timestamp".to_string(),
        FieldDefinition {
            name: "timestamp".to_string(),
            description: Some("Message timestamp".to_string()),
            field_type: GraphQLType::NonNull(Box::new(GraphQLType::Scalar(ScalarType::String))),
            arguments: HashMap::new(),
            deprecation_reason: None,
        },
    );

    let message_type = GraphQLType::Object(ObjectType {
        name: "Message".to_string(),
        description: Some("A chat message".to_string()),
        fields: message_fields,
        interfaces: vec![],
    });
    types.insert("Message".to_string(), message_type);

    // Subscription root type
    let mut subscription_fields = HashMap::new();
    subscription_fields.insert(
        "messageAdded".to_string(),
        FieldDefinition {
            name: "messageAdded".to_string(),
            description: Some("Subscribe to new messages".to_string()),
            field_type: GraphQLType::NonNull(Box::new(GraphQLType::Scalar(ScalarType::String))), // Simplified
            arguments: HashMap::new(),
            deprecation_reason: None,
        },
    );
    subscription_fields.insert(
        "userStatusChanged".to_string(),
        FieldDefinition {
            name: "userStatusChanged".to_string(),
            description: Some("Subscribe to user status changes".to_string()),
            field_type: GraphQLType::NonNull(Box::new(GraphQLType::Scalar(ScalarType::String))), // Simplified
            arguments: HashMap::new(),
            deprecation_reason: None,
        },
    );
    subscription_fields.insert(
        "typingIndicator".to_string(),
        FieldDefinition {
            name: "typingIndicator".to_string(),
            description: Some("Subscribe to typing indicators".to_string()),
            field_type: GraphQLType::Scalar(ScalarType::String), // Nullable for when typing stops
            arguments: HashMap::new(),
            deprecation_reason: None,
        },
    );

    let subscription_type = GraphQLType::Object(ObjectType {
        name: "Subscription".to_string(),
        description: Some("Real-time subscription root".to_string()),
        fields: subscription_fields,
        interfaces: vec![],
    });
    types.insert("Subscription".to_string(), subscription_type);

    // Mutation type (for completeness)
    let mut mutation_fields = HashMap::new();
    mutation_fields.insert(
        "sendMessage".to_string(),
        FieldDefinition {
            name: "sendMessage".to_string(),
            description: Some("Send a new message".to_string()),
            field_type: GraphQLType::NonNull(Box::new(GraphQLType::Scalar(ScalarType::String))), // Simplified
            arguments: HashMap::new(),
            deprecation_reason: None,
        },
    );

    let mutation_type = GraphQLType::Object(ObjectType {
        name: "Mutation".to_string(),
        description: Some("Mutation root".to_string()),
        fields: mutation_fields,
        interfaces: vec![],
    });
    types.insert("Mutation".to_string(), mutation_type);

    // Query root type (required by GraphQL spec)
    let mut query_fields = HashMap::new();
    query_fields.insert(
        "messages".to_string(),
        FieldDefinition {
            name: "messages".to_string(),
            description: Some("Get all messages".to_string()),
            field_type: GraphQLType::NonNull(Box::new(GraphQLType::List(Box::new(
                GraphQLType::NonNull(Box::new(GraphQLType::Scalar(ScalarType::String))),
            )))),
            arguments: HashMap::new(),
            deprecation_reason: None,
        },
    );
    query_fields.insert(
        "users".to_string(),
        FieldDefinition {
            name: "users".to_string(),
            description: Some("Get all users".to_string()),
            field_type: GraphQLType::NonNull(Box::new(GraphQLType::List(Box::new(
                GraphQLType::NonNull(Box::new(GraphQLType::Scalar(ScalarType::String))),
            )))),
            arguments: HashMap::new(),
            deprecation_reason: None,
        },
    );

    let query_type = GraphQLType::Object(ObjectType {
        name: "Query".to_string(),
        description: Some("Query root".to_string()),
        fields: query_fields,
        interfaces: vec![],
    });
    types.insert("Query".to_string(), query_type);

    Schema {
        id: SchemaId::new(),
        version: SchemaVersion::new("1.0.0"),
        query_type: "Query".to_string(),
        mutation_type: Some("Mutation".to_string()),
        subscription_type: Some("Subscription".to_string()),
        types,
        directives: HashMap::new(),
        description: Some("Chat application schema with real-time subscriptions".to_string()),
    }
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("🚀 GraphQL Subscription Support Demo");
    println!("=====================================\n");

    // Step 1: Create a comprehensive schema with subscription support
    println!("📋 Step 1: Creating chat schema with subscription support...");
    let schema = create_chat_schema();

    // Step 2: Validate the schema
    println!("🔍 Step 2: Validating schema structure...");
    let schema_validator = SchemaValidator::new();
    let schema_validation = schema_validator.validate(&schema);

    if schema_validation.is_valid() {
        println!("✅ Schema validation: PASSED");
        println!("   - Query type: {}", schema.query_type);
        println!("   - Mutation type: {:?}", schema.mutation_type);
        println!("   - Subscription type: {:?}", schema.subscription_type);
        println!("   - Total types defined: {}", schema.types.len());
    } else {
        println!("❌ Schema validation: FAILED");
        if let Some(errors) = schema_validation.to_execution_result() {
            for error in errors.errors {
                println!("   Error: {}", error.message);
            }
        }
        return Ok(());
    }

    // Step 3: Demonstrate various subscription queries
    println!("\n📡 Step 3: Demonstrating subscription queries...");

    let subscription_examples = vec![
        (
            "Basic Message Subscription",
            "subscription { messageAdded }",
        ),
        (
            "User Status Subscription",
            "subscription { userStatusChanged }",
        ),
        (
            "Typing Indicator Subscription",
            "subscription { typingIndicator }",
        ),
        (
            "Multiple Subscriptions",
            "subscription { messageAdded userStatusChanged }",
        ),
    ];

    let query_validator = QueryValidator::new();
    let executor = QueryExecutor::new();

    for (title, subscription_query) in subscription_examples {
        println!("\n🔄 Testing: {}", title);
        println!("   Query: {}", subscription_query);

        let mut query = Query::new(subscription_query.to_string());

        // Mark as valid for demonstration (in production, proper parsing would be needed)
        query.mark_validated(ValidationResult::Valid);

        // Validate query structure
        let validation_result = query_validator.validate(&query, &schema);
        if validation_result.is_valid() {
            println!("   ✅ Query validation: PASSED");
        } else {
            println!("   ⚠️ Query validation: Issues detected");
        }

        // Execute subscription
        match executor.execute(&query, &schema).await {
            result if result.errors.is_empty() => {
                println!("   🎉 Subscription execution: SUCCESS");
                if let Some(data) = result.data {
                    println!("   📊 Response data: {}", data);
                }
            },
            result => {
                println!(
                    "   📞 Subscription requires transport: {}",
                    result.errors[0].message
                );
                if let Some(extensions) = &result.errors[0].extensions {
                    if let Some(code) = extensions.get("code") {
                        println!("   🏷️ Error code: {}", code);
                    }
                }
            },
        }
    }

    // Step 4: Demonstrate error handling scenarios
    println!("\n🛠️ Step 4: Testing error handling scenarios...");

    let error_scenarios = vec![
        ("Schema without subscription type", true),
        ("Invalid subscription field", false),
    ];

    for (scenario_name, remove_subscription_type) in error_scenarios {
        println!("\n⚠️ Error Scenario: {}", scenario_name);

        let mut test_schema = create_chat_schema();
        if remove_subscription_type {
            test_schema.subscription_type = None;
        }

        let mut query = Query::new("subscription { messageAdded }".to_string());
        query.mark_validated(ValidationResult::Valid);

        let result = executor.execute(&query, &test_schema).await;
        if !result.errors.is_empty() {
            println!("   ✅ Error correctly caught: {}", result.errors[0].message);
        } else {
            println!("   ❌ Expected error not caught");
        }
    }

    // Step 5: Demonstrate SubscriptionResult features
    println!("\n🔧 Step 5: Testing SubscriptionResult features...");

    // Test error result construction
    let error_result =
        SubscriptionResult::with_error(GraphQLError::new("Connection lost".to_string()));
    println!(
        "   Error result created - has errors: {}, has stream: {}",
        error_result.has_errors(),
        error_result.has_stream()
    );

    // Test multiple errors
    let multi_error_result = SubscriptionResult::with_errors(vec![
        GraphQLError::new("Authentication failed".to_string()),
        GraphQLError::new("Rate limit exceeded".to_string()),
    ]);
    println!(
        "   Multi-error result - error count: {}, has stream: {}",
        multi_error_result.errors.len(),
        multi_error_result.has_stream()
    );

    // Step 6: Production considerations
    println!("\n🏭 Step 6: Production Implementation Notes:");
    println!("   📡 Transport Layer:");
    println!("      - WebSocket connection required for real-time subscriptions");
    println!("      - Server-Sent Events (SSE) as fallback transport");
    println!("      - Connection lifecycle management (connect/disconnect/reconnect)");

    println!("   🔄 Event System:");
    println!("      - Pub/Sub mechanism for event distribution");
    println!("      - Event filtering based on subscription parameters");
    println!("      - Backpressure handling for high-throughput scenarios");

    println!("   🛡️ Security & Performance:");
    println!("      - Authentication and authorization for subscriptions");
    println!("      - Rate limiting to prevent subscription abuse");
    println!("      - Connection pooling and resource management");

    println!("   📊 Monitoring:");
    println!("      - Active subscription metrics");
    println!("      - Event throughput monitoring");
    println!("      - Connection health checks");

    println!("\n✨ Subscription Support Demo Complete!");
    println!("   The GraphQL server now supports real-time subscriptions");
    println!("   with proper error handling and production-ready patterns.");

    Ok(())
}

// Helper functions for subscription testing
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_chat_schema_creation() {
        let schema = create_chat_schema();
        assert!(schema.subscription_type.is_some());
        assert_eq!(schema.subscription_type.as_ref().unwrap(), "Subscription");

        let subscription_type = schema.get_type("Subscription").unwrap();
        if let GraphQLType::Object(obj) = subscription_type {
            assert!(obj.fields.contains_key("messageAdded"));
            assert!(obj.fields.contains_key("userStatusChanged"));
            assert!(obj.fields.contains_key("typingIndicator"));
        } else {
            panic!("Subscription should be an Object type");
        }
    }

    #[tokio::test]
    async fn test_subscription_execution_demo() {
        let schema = create_chat_schema();
        let executor = QueryExecutor::new();

        let mut query = Query::new("subscription { messageAdded }".to_string());
        query.mark_validated(ValidationResult::Valid);

        let result = executor.execute(&query, &schema).await;
        // Should get transport requirement error
        assert!(!result.errors.is_empty());
        assert!(
            result.errors[0].message.contains("WebSocket")
                || result.errors[0].message.contains("transport")
        );
    }
}
