/// Simple test of core GraphQL schema functionality
/// This tests only our newly implemented types, lexer, parser, and schema service
use std::collections::HashMap;
use std::error::Error;

// Import our core modules
use graphql_rs::application::services::schema_service::SchemaService;
use graphql_rs::domain::entities::{
    schema::Schema,
    types::{FieldDefinition, GraphQLType, ObjectType, ScalarType},
};
use graphql_rs::infrastructure::{
    lexer::{Lexer, Token},
    parser::Parser,
};

fn main() -> Result<(), Box<dyn Error>> {
    println!("Testing GraphQL Schema Implementation");

    // Test 1: Basic Lexer
    println!("\n1. Testing Lexer...");
    test_lexer()?;

    // Test 2: Basic Types
    println!("\n2. Testing GraphQL Types...");
    test_graphql_types()?;

    // Test 3: Schema Creation
    println!("\n3. Testing Schema Creation...");
    test_schema_creation()?;

    // Test 4: Schema Service
    println!("\n4. Testing Schema Service...");
    test_schema_service()?;

    // Test 5: Simple SDL Parsing
    println!("\n5. Testing SDL Parsing...");
    test_sdl_parsing()?;

    println!("\n✅ All tests passed! Schema definition and parsing functionality is working.");
    Ok(())
}

fn test_lexer() -> Result<(), Box<dyn Error>> {
    let input = r#"type User { id: ID! name: String }"#;
    let mut lexer = Lexer::new(input);

    let tokens = lexer.remaining_tokens();
    println!("   Tokenized {} tokens from input", tokens.len());

    // Check that we have the expected tokens
    assert!(tokens.contains(&Token::Type));
    assert!(tokens.contains(&Token::Name("User".to_string())));
    assert!(tokens.contains(&Token::LeftBrace));
    assert!(tokens.contains(&Token::Name("id".to_string())));
    assert!(tokens.contains(&Token::Colon));
    assert!(tokens.contains(&Token::Name("ID".to_string())));
    assert!(tokens.contains(&Token::Bang));

    println!("   ✓ Lexer correctly tokenized GraphQL type definition");
    Ok(())
}

fn test_graphql_types() -> Result<(), Box<dyn Error>> {
    // Create a simple User type
    let mut fields = HashMap::new();
    fields.insert(
        "id".to_string(),
        FieldDefinition {
            name: "id".to_string(),
            description: Some("User identifier".to_string()),
            field_type: GraphQLType::NonNull(Box::new(GraphQLType::Scalar(ScalarType::ID))),
            arguments: HashMap::new(),
            deprecation_reason: None,
        },
    );

    fields.insert(
        "name".to_string(),
        FieldDefinition {
            name: "name".to_string(),
            description: Some("User name".to_string()),
            field_type: GraphQLType::Scalar(ScalarType::String),
            arguments: HashMap::new(),
            deprecation_reason: None,
        },
    );

    let user_type = GraphQLType::Object(ObjectType {
        name: "User".to_string(),
        description: Some("A user in the system".to_string()),
        fields,
        interfaces: vec![],
    });

    // Test type introspection methods
    assert!(user_type.is_output_type());
    assert!(!user_type.is_input_type());
    assert!(!user_type.is_leaf());
    assert!(user_type.name().is_some());

    println!("   ✓ GraphQL type system working correctly");
    Ok(())
}

fn test_schema_creation() -> Result<(), Box<dyn Error>> {
    let mut schema = Schema::new("Query".to_string());

    // Add a simple scalar type
    let datetime_scalar = GraphQLType::Scalar(ScalarType::Custom("DateTime".to_string()));
    schema.add_type(datetime_scalar)?;

    // Add a simple object type
    let mut query_fields = HashMap::new();
    query_fields.insert(
        "hello".to_string(),
        FieldDefinition {
            name: "hello".to_string(),
            description: Some("A simple hello field".to_string()),
            field_type: GraphQLType::Scalar(ScalarType::String),
            arguments: HashMap::new(),
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

    // Basic schema validation
    let validation_result = schema.validate();
    if let Err(errors) = validation_result {
        println!("   Validation errors: {:?}", errors);
        return Err("Schema validation failed".into());
    }

    println!("   ✓ Schema creation and validation working");
    Ok(())
}

fn test_schema_service() -> Result<(), Box<dyn Error>> {
    let mut service = SchemaService::new();

    // Build a schema using the service
    let user_type = GraphQLType::Object(ObjectType {
        name: "User".to_string(),
        description: Some("A user".to_string()),
        fields: {
            let mut fields = HashMap::new();
            fields.insert(
                "id".to_string(),
                FieldDefinition {
                    name: "id".to_string(),
                    description: None,
                    field_type: GraphQLType::NonNull(Box::new(GraphQLType::Scalar(ScalarType::ID))),
                    arguments: HashMap::new(),
                    deprecation_reason: None,
                },
            );
            fields
        },
        interfaces: vec![],
    });

    let _schema = service
        .build_schema()
        .query_type("Query")
        .add_type(user_type)?
        .build()?;

    assert!(service.has_schema());

    let stats = service.get_schema_stats()?;
    println!(
        "   Schema stats: {} total types, {} object types",
        stats.total_types, stats.object_types
    );

    println!("   ✓ Schema service working correctly");
    Ok(())
}

fn test_sdl_parsing() -> Result<(), Box<dyn Error>> {
    let sdl = r#"
        scalar DateTime
        
        type User {
            id: ID!
            name: String
            createdAt: DateTime
        }
        
        enum Status {
            ACTIVE
            INACTIVE
        }
    "#;

    let mut parser = Parser::new(sdl);

    // Parse individual type definitions
    let scalar_def = parser.parse_type_system_definition()?;
    let object_def = parser.parse_type_system_definition()?;
    let enum_def = parser.parse_type_system_definition()?;

    println!("   ✓ Successfully parsed scalar, object, and enum definitions");

    // Test full schema parsing via service
    let mut service = SchemaService::new();
    let simple_sdl = r#"
        type Query {
            hello: String
        }
    "#;

    let _schema = service.load_schema_from_sdl(simple_sdl)?;
    println!("   ✓ Schema service successfully parsed SDL");

    Ok(())
}
