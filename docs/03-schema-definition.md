# Schema Definition & Type System

This document details the implementation of GraphQL's Schema Definition Language (SDL) and type system in our Rust GraphQL server.

## GraphQL Type System Overview

The GraphQL type system defines the capabilities of a GraphQL API. It describes the complete set of possible data (as a graph of nodes and connections) that a client can access.

### Type Categories

1. **Scalar Types**: Primitive data types (Int, Float, String, Boolean, ID)
2. **Object Types**: Composite types with fields
3. **Interface Types**: Abstract types that define a common set of fields
4. **Union Types**: Types that can be one of several object types
5. **Enum Types**: Special scalar types with limited values
6. **Input Types**: Special object types used as input arguments
7. **List and Non-Null Types**: Type modifiers

## Implementation Strategy

### Core Type Definitions

```rust
// Core type system enums
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum TypeDefinition {
    Scalar(ScalarTypeDefinition),
    Object(ObjectTypeDefinition),
    Interface(InterfaceTypeDefinition),
    Union(UnionTypeDefinition),
    Enum(EnumTypeDefinition),
    InputObject(InputObjectTypeDefinition),
}

// Type references with modifiers
#[derive(Debug, Clone, PartialEq)]
pub enum TypeReference {
    Named(String),
    List(Box<TypeReference>),
    NonNull(Box<TypeReference>),
}

// Built-in scalar types
#[derive(Debug, Clone, PartialEq)]
pub enum BuiltinScalar {
    Int,
    Float,
    String,
    Boolean,
    ID,
}
```

### Object Types

Object types are the most common types in GraphQL schemas:

```rust
#[derive(Debug, Clone, PartialEq)]
pub struct ObjectTypeDefinition {
    pub name: String,
    pub description: Option<String>,
    pub fields: IndexMap<String, FieldDefinition>,
    pub interfaces: Vec<String>,
    pub directives: Vec<DirectiveApplication>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct FieldDefinition {
    pub name: String,
    pub description: Option<String>,
    pub type_reference: TypeReference,
    pub arguments: IndexMap<String, ArgumentDefinition>,
    pub directives: Vec<DirectiveApplication>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct ArgumentDefinition {
    pub name: String,
    pub description: Option<String>,
    pub type_reference: TypeReference,
    pub default_value: Option<Value>,
    pub directives: Vec<DirectiveApplication>,
}
```

### Interface Types

Interfaces define a common set of fields that implementing types must include:

```rust
#[derive(Debug, Clone, PartialEq)]
pub struct InterfaceTypeDefinition {
    pub name: String,
    pub description: Option<String>,
    pub fields: IndexMap<String, FieldDefinition>,
    pub interfaces: Vec<String>, // GraphQL 2018+ supports interface inheritance
    pub directives: Vec<DirectiveApplication>,
}
```

### Union Types

Union types represent objects that could be one of several types:

```rust
#[derive(Debug, Clone, PartialEq)]
pub struct UnionTypeDefinition {
    pub name: String,
    pub description: Option<String>,
    pub types: Vec<String>,
    pub directives: Vec<DirectiveApplication>,
}
```

### Enum Types

Enum types are scalar types with a finite set of possible values:

```rust
#[derive(Debug, Clone, PartialEq)]
pub struct EnumTypeDefinition {
    pub name: String,
    pub description: Option<String>,
    pub values: IndexMap<String, EnumValueDefinition>,
    pub directives: Vec<DirectiveApplication>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct EnumValueDefinition {
    pub name: String,
    pub description: Option<String>,
    pub deprecation_reason: Option<String>,
    pub directives: Vec<DirectiveApplication>,
}
```

### Schema Definition

The schema ties everything together:

```rust
#[derive(Debug, Clone, PartialEq)]
pub struct SchemaDefinition {
    pub description: Option<String>,
    pub query: Option<String>,
    pub mutation: Option<String>,
    pub subscription: Option<String>,
    pub directives: Vec<DirectiveApplication>,
}

#[derive(Debug, Clone)]
pub struct Schema {
    pub schema_definition: Option<SchemaDefinition>,
    pub types: IndexMap<String, TypeDefinition>,
    pub directives: IndexMap<String, DirectiveDefinition>,
}
```

## SDL Parser Implementation

### Parsing Strategy

We'll implement a recursive descent parser that converts GraphQL SDL text into our AST:

```rust
pub struct SchemaParser {
    lexer: Lexer,
    current_token: Token,
    peek_token: Token,
}

impl SchemaParser {
    pub fn new(input: &str) -> Self {
        let mut lexer = Lexer::new(input);
        let current_token = lexer.next_token();
        let peek_token = lexer.next_token();
        
        Self {
            lexer,
            current_token,
            peek_token,
        }
    }

    pub fn parse_schema_document(&mut self) -> Result<Document, ParseError> {
        let mut definitions = Vec::new();
        
        while !self.is_at_end() {
            definitions.push(self.parse_definition()?);
        }
        
        Ok(Document { definitions })
    }

    fn parse_definition(&mut self) -> Result<Definition, ParseError> {
        match &self.current_token {
            Token::Type => self.parse_object_type_definition(),
            Token::Interface => self.parse_interface_type_definition(),
            Token::Union => self.parse_union_type_definition(),
            Token::Enum => self.parse_enum_type_definition(),
            Token::Input => self.parse_input_object_type_definition(),
            Token::Schema => self.parse_schema_definition(),
            Token::Scalar => self.parse_scalar_type_definition(),
            Token::Directive => self.parse_directive_definition(),
            _ => Err(ParseError::unexpected_token(self.current_token.clone())),
        }
    }
}
```

### Type Reference Parsing

Type references handle the complexity of lists and non-null modifiers:

```rust
impl SchemaParser {
    fn parse_type_reference(&mut self) -> Result<TypeReference, ParseError> {
        let mut type_ref = self.parse_named_type()?;
        
        // Handle list and non-null wrappers
        loop {
            match &self.current_token {
                Token::LeftBracket => {
                    self.advance(); // consume '['
                    type_ref = TypeReference::List(Box::new(type_ref));
                    self.expect_token(Token::RightBracket)?;
                }
                Token::Bang => {
                    self.advance(); // consume '!'
                    type_ref = TypeReference::NonNull(Box::new(type_ref));
                }
                _ => break,
            }
        }
        
        Ok(type_ref)
    }
    
    fn parse_named_type(&mut self) -> Result<TypeReference, ParseError> {
        if let Token::Name(name) = &self.current_token {
            let type_name = name.clone();
            self.advance();
            Ok(TypeReference::Named(type_name))
        } else {
            Err(ParseError::expected_name())
        }
    }
}
```

## Schema Validation

### Validation Rules

Our schema validator will enforce GraphQL specification rules:

```rust
pub struct SchemaValidator;

impl SchemaValidator {
    pub fn validate(&self, schema: &Schema) -> ValidationResult {
        let mut errors = Vec::new();
        
        // Rule: Schema must have Query type
        self.validate_query_type_exists(schema, &mut errors);
        
        // Rule: All types must be defined
        self.validate_type_references(schema, &mut errors);
        
        // Rule: Interface implementations must be valid
        self.validate_interface_implementations(schema, &mut errors);
        
        // Rule: Union types must contain object types
        self.validate_union_members(schema, &mut errors);
        
        // Rule: Field names must be unique within types
        self.validate_field_uniqueness(schema, &mut errors);
        
        // Rule: Argument names must be unique within fields
        self.validate_argument_uniqueness(schema, &mut errors);
        
        if errors.is_empty() {
            ValidationResult::Valid
        } else {
            ValidationResult::Invalid(errors)
        }
    }
    
    fn validate_query_type_exists(&self, schema: &Schema, errors: &mut Vec<ValidationError>) {
        let query_type_name = schema
            .schema_definition
            .as_ref()
            .and_then(|def| def.query.as_ref())
            .unwrap_or(&"Query".to_string());
            
        if !schema.types.contains_key(query_type_name) {
            errors.push(ValidationError::missing_query_type());
        }
    }
    
    fn validate_type_references(&self, schema: &Schema, errors: &mut Vec<ValidationError>) {
        for (type_name, type_def) in &schema.types {
            self.validate_type_definition_references(type_name, type_def, schema, errors);
        }
    }
}
```

## Built-in Types and Introspection

### Introspection Schema

GraphQL servers must provide introspection capabilities:

```rust
pub fn build_introspection_schema() -> Schema {
    let mut types = IndexMap::new();
    
    // Add introspection types
    types.insert("__Schema".to_string(), build_schema_type());
    types.insert("__Type".to_string(), build_type_type());
    types.insert("__Field".to_string(), build_field_type());
    types.insert("__InputValue".to_string(), build_input_value_type());
    types.insert("__EnumValue".to_string(), build_enum_value_type());
    types.insert("__Directive".to_string(), build_directive_type());
    types.insert("__DirectiveLocation".to_string(), build_directive_location_enum());
    types.insert("__TypeKind".to_string(), build_type_kind_enum());
    
    Schema {
        schema_definition: None,
        types,
        directives: build_introspection_directives(),
    }
}

fn build_schema_type() -> TypeDefinition {
    TypeDefinition::Object(ObjectTypeDefinition {
        name: "__Schema".to_string(),
        description: Some("A GraphQL Schema defines the capabilities of a GraphQL server.".to_string()),
        fields: {
            let mut fields = IndexMap::new();
            fields.insert("types".to_string(), FieldDefinition {
                name: "types".to_string(),
                description: Some("A list of all types supported by this server.".to_string()),
                type_reference: TypeReference::NonNull(Box::new(
                    TypeReference::List(Box::new(
                        TypeReference::NonNull(Box::new(TypeReference::Named("__Type".to_string())))
                    ))
                )),
                arguments: IndexMap::new(),
                directives: Vec::new(),
            });
            // ... more introspection fields
            fields
        },
        interfaces: Vec::new(),
        directives: Vec::new(),
    })
}
```

## Testing Strategy

### Unit Tests for Type System

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_simple_object_type() {
        let sdl = r#"
            type User {
                id: ID!
                name: String
                email: String!
            }
        "#;
        
        let mut parser = SchemaParser::new(sdl);
        let document = parser.parse_schema_document().unwrap();
        
        assert_eq!(document.definitions.len(), 1);
        // More detailed assertions...
    }

    #[test]
    fn test_type_reference_with_lists_and_nulls() {
        let sdl = "field: [String!]!";
        // Test parsing complex type references
    }

    #[test]
    fn test_schema_validation_missing_query_type() {
        let schema = Schema {
            schema_definition: None,
            types: IndexMap::new(),
            directives: IndexMap::new(),
        };
        
        let validator = SchemaValidator;
        let result = validator.validate(&schema);
        
        assert!(matches!(result, ValidationResult::Invalid(_)));
    }
}
```

## Integration with Domain Model

The type system integrates with our domain model by:

1. **Schema Entity**: Encapsulates the complete type system
2. **Validation Service**: Ensures schema correctness
3. **Type Registry**: Provides efficient type lookup during execution
4. **Introspection Service**: Generates introspection responses

This type system implementation provides the foundation for query execution, validation, and introspection in our GraphQL server.
