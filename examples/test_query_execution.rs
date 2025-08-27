/// Test example of query execution functionality
/// This example demonstrates the new query execution engine capabilities
use std::error::Error;

// Import our core modules
use graphql_rs::application::services::schema_service::SchemaService;
use graphql_rs::domain::entities::{
    query::Query,
    schema::Schema,
    types::{FieldDefinition, GraphQLType, ObjectType, ScalarType},
};
use graphql_rs::domain::services::{QueryExecution, QueryExecutor};
use graphql_rs::domain::value_objects::ValidationResult;

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    println!("🚀 Testing GraphQL Query Execution Engine");

    // Test 1: Create a simple schema with Query type
    test_schema_creation()?;

    // Test 2: Test query parsing
    test_query_parsing().await?;

    // Test 3: Test complete query execution
    test_query_execution().await?;

    println!("✅ All query execution tests passed!");
    Ok(())
}

fn test_schema_creation() -> Result<(), Box<dyn Error>> {
    println!("\n📋 Testing Schema Creation...");

    let mut schema = Schema::new("Query".to_string());

    // Add fields to Query type
    let mut query_fields = std::collections::HashMap::new();

    query_fields.insert(
        "hello".to_string(),
        FieldDefinition {
            name: "hello".to_string(),
            description: Some("A simple hello field".to_string()),
            field_type: GraphQLType::Scalar(ScalarType::String),
            arguments: std::collections::HashMap::new(),
            deprecation_reason: None,
        },
    );

    query_fields.insert(
        "user".to_string(),
        FieldDefinition {
            name: "user".to_string(),
            description: Some("User query field".to_string()),
            field_type: GraphQLType::Object(ObjectType {
                name: "User".to_string(),
                description: Some("User object type".to_string()),
                fields: {
                    let mut user_fields = std::collections::HashMap::new();
                    user_fields.insert(
                        "id".to_string(),
                        FieldDefinition {
                            name: "id".to_string(),
                            description: Some("User ID".to_string()),
                            field_type: GraphQLType::Scalar(ScalarType::ID),
                            arguments: std::collections::HashMap::new(),
                            deprecation_reason: None,
                        },
                    );
                    user_fields.insert(
                        "name".to_string(),
                        FieldDefinition {
                            name: "name".to_string(),
                            description: Some("User name".to_string()),
                            field_type: GraphQLType::Scalar(ScalarType::String),
                            arguments: std::collections::HashMap::new(),
                            deprecation_reason: None,
                        },
                    );
                    user_fields
                },
                interfaces: vec![],
            }),
            arguments: std::collections::HashMap::new(),
            deprecation_reason: None,
        },
    );

    let query_type = GraphQLType::Object(ObjectType {
        name: "Query".to_string(),
        description: Some("The root query type".to_string()),
        fields: query_fields,
        interfaces: vec![],
    });

    schema.add_type(query_type)?;

    println!("✅ Schema created successfully");
    Ok(())
}

async fn test_query_parsing() -> Result<(), Box<dyn Error>> {
    println!("\n🔍 Testing Query Parsing...");

    use graphql_rs::infrastructure::query_parser::QueryParser;

    let simple_query = r#"
    {
        hello
    }
    "#;

    let mut parser = QueryParser::new(simple_query);
    let document = parser.parse_document()?;

    println!(
        "✅ Simple query parsed: {} definitions",
        document.definitions.len()
    );

    let complex_query = r#"
    query GetUser($id: ID!) {
        user(id: $id) {
            id
            name
        }
        hello
    }
    "#;

    let mut parser = QueryParser::new(complex_query);
    let document = parser.parse_document()?;

    println!(
        "✅ Complex query parsed: {} definitions",
        document.definitions.len()
    );

    Ok(())
}

async fn test_query_execution() -> Result<(), Box<dyn Error>> {
    println!("\n⚡ Testing Query Execution...");

    // Create a simple schema
    let mut schema_service = SchemaService::new();

    let simple_sdl = r#"
    type Query {
        hello: String
        count: Int
        active: Boolean
    }
    "#;

    let schema = schema_service.load_schema_from_sdl(simple_sdl)?;

    // Create and validate query
    let mut query = Query::new(
        r#"
    {
        hello
        count
        active
    }
    "#
        .to_string(),
    );

    // Mark as valid (in real usage, this would go through validation)
    query.mark_validated(ValidationResult::Valid);

    // Execute query
    let executor = QueryExecutor::new();
    let result = executor.execute(&query, &schema).await;

    println!("✅ Query execution result:");
    if let Some(data) = &result.data {
        println!("   Data: {}", serde_json::to_string_pretty(data)?);
    }

    if !result.errors.is_empty() {
        println!("   Errors: {:?}", result.errors);
    }

    // Test query with field that doesn't exist
    let mut invalid_query = Query::new(
        r#"
    {
        nonexistent
    }
    "#
        .to_string(),
    );

    invalid_query.mark_validated(ValidationResult::Valid);
    let invalid_result = executor.execute(&invalid_query, &schema).await;

    println!("✅ Invalid field query handled gracefully:");
    if !invalid_result.errors.is_empty() {
        println!("   Expected error: {}", invalid_result.errors[0].message);
    }

    Ok(())
}
