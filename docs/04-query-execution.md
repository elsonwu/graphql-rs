# GraphQL Query Execution Engine

This document describes the implementation of the GraphQL query execution engine in our Rust GraphQL server.

## Overview

The query execution engine is responsible for:

1. Parsing GraphQL query documents into Abstract Syntax Trees (AST)
2. Validating queries against the schema
3. Executing queries by resolving fields
4. Returning structured results or errors

## Architecture

### Components

#### 1. Query Parser (`infrastructure/query_parser.rs`)

Parses GraphQL query strings into structured AST:

- **Document**: Top-level container for operations and fragments
- **Operation**: Query, mutation, or subscription definitions
- **Selection Set**: Groups of fields to be resolved
- **Field**: Individual data selections with optional arguments and sub-selections

```rust
use graphql_rs::infrastructure::query_parser::QueryParser;

let mut parser = QueryParser::new("{ user { id name } }");
let document = parser.parse_document()?;
```

#### 2. Query Executor (`domain/services.rs`)

Executes parsed queries against a schema:

- Resolves fields based on schema definitions
- Handles nested object selections
- Returns mock data for demonstration (real resolvers to be added later)

```rust
use graphql_rs::domain::services::{QueryExecution, QueryExecutor};

let executor = QueryExecutor::new();
let result = executor.execute(&query, &schema).await;
```

## Supported Features

### ✅ Currently Implemented

- **Query Parsing**: Full GraphQL query syntax support
  - Operations: `query`, `mutation`, `subscription`
  - Variables with types and default values
  - Field aliases
  - Arguments
  - Directives
  - Fragments (parsing only)

- **Basic Execution**:
  - Field resolution for scalar types
  - Object field selection
  - Error handling for missing fields
  - Mock data generation

- **Type System Integration**:
  - Schema validation during execution
  - Type-based field resolution
  - Built-in scalar type support

### 🚧 Partial Implementation

- **Fragment Support**: Parsed but not yet executed
- **Variable Substitution**: Variables parsed but not yet resolved
- **Directive Processing**: Directives parsed but not yet applied

### 📋 Planned Features

- **Real Resolvers**: Replace mock data with actual resolver functions
- **Variable Resolution**: Substitute variables in field arguments
- **Fragment Execution**: Support inline fragments and fragment spreads
- **Directive Execution**: Apply directives like `@skip` and `@include`
- **Mutations**: Full mutation support
- **Subscriptions**: Real-time subscription implementation

## Usage Example

```rust
use graphql_rs::{
    application::services::schema_service::SchemaService,
    domain::{
        entities::query::Query,
        services::{QueryExecution, QueryExecutor},
        value_objects::ValidationResult,
    },
};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Load schema
    let mut schema_service = SchemaService::new();
    let schema = schema_service.load_schema_from_sdl(r#"
        type Query {
            hello: String
            user: User
        }
        
        type User {
            id: ID!
            name: String
        }
    "#)?;

    // Create and execute query
    let mut query = Query::new(r#"
        {
            hello
            user {
                id
                name
            }
        }
    "#.to_string());
    
    query.mark_validated(ValidationResult::Valid);
    
    let executor = QueryExecutor::new();
    let result = executor.execute(&query, &schema).await;
    
    println!("Result: {}", serde_json::to_string_pretty(&result.data)?);
    Ok(())
}
```

## Query AST Structure

The parser generates an AST with the following key types:

```rust
pub struct Document {
    pub definitions: Vec<Definition>,
}

pub enum Definition {
    Operation(OperationDefinition),
    Fragment(FragmentDefinition),
}

pub struct OperationDefinition {
    pub operation_type: OperationType,
    pub name: Option<String>,
    pub variable_definitions: Vec<VariableDefinition>,
    pub directives: Vec<Directive>,
    pub selection_set: SelectionSet,
}

pub struct SelectionSet {
    pub selections: Vec<Selection>,
}

pub enum Selection {
    Field(Field),
    InlineFragment(InlineFragment),
    FragmentSpread(FragmentSpread),
}
```

## Error Handling

The execution engine provides comprehensive error handling:

- **Parse Errors**: Syntax errors in query strings
- **Validation Errors**: Schema validation failures
- **Field Errors**: Missing fields, type mismatches
- **Execution Errors**: Runtime failures during field resolution

```rust
// Errors are returned in the standard GraphQL format
pub struct ExecutionResult {
    pub data: Option<serde_json::Value>,
    pub errors: Vec<GraphQLError>,
    pub extensions: Option<serde_json::Map<String, serde_json::Value>>,
}
```

## Testing

Comprehensive tests cover:

- Query parsing for various GraphQL constructs
- Schema-based query execution
- Error scenarios and edge cases
- Integration with the schema system

Run tests with:

```bash
cargo test query_executor
cargo run --example test_query_execution
```

## Performance Considerations

Current implementation focuses on correctness over performance:

- Simple recursive field resolution
- Mock data generation (no database queries)
- Basic error handling without optimization

Future optimizations will include:

- Field resolution batching
- DataLoader pattern for N+1 prevention
- Query complexity analysis
- Caching strategies

## Integration Points

The query executor integrates with:

- **Schema System**: Uses schema definitions for validation and field resolution
- **Type System**: Leverages GraphQL type information for proper field handling
- **Validation System**: Coordinates with query validation before execution
- **Event System**: Publishes execution events for monitoring and analytics

## Next Steps

1. **Resolver System**: Implement pluggable resolver functions
2. **Variable Resolution**: Add variable substitution during execution
3. **Fragment Support**: Complete fragment spread and inline fragment execution
4. **Directive Processing**: Add support for built-in and custom directives
5. **Performance**: Add DataLoader pattern and query optimization
6. **Mutations**: Extend execution engine to handle mutations
7. **Subscriptions**: Implement real-time subscription execution

This query execution engine provides the foundation for a complete GraphQL server implementation, with room for future enhancements and optimizations.
