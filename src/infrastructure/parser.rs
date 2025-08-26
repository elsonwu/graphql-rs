use crate::domain::entities::types::*;
use crate::domain::entities::schema::*;
use crate::infrastructure::lexer::{Token, Lexer, LexError};
use nom::{
    IResult,
    branch::alt,
    combinator::{map, opt, value},
    multi::{many0, many1, separated_list0, separated_list1},
    sequence::{delimited, pair, preceded, terminated, tuple},
    error::{Error as NomError, ErrorKind},
};
use std::collections::HashMap;
use thiserror::Error;

/// Errors that can occur during parsing
#[derive(Error, Debug, Clone, PartialEq)]
pub enum ParseError {
    #[error("Lexical error: {0}")]
    LexError(#[from] LexError),
    
    #[error("Unexpected token: expected {expected}, found {found} at position {position}")]
    UnexpectedToken {
        expected: String,
        found: String,
        position: usize,
    },
    
    #[error("Unexpected end of input: expected {expected}")]
    UnexpectedEof {
        expected: String,
    },
    
    #[error("Invalid syntax at position {position}: {message}")]
    InvalidSyntax {
        position: usize,
        message: String,
    },
    
    #[error("Duplicate definition: {name}")]
    DuplicateDefinition {
        name: String,
    },
    
    #[error("Invalid type reference: {type_name}")]
    InvalidTypeReference {
        type_name: String,
    },
}

/// Parser for GraphQL documents
pub struct Parser<'input> {
    lexer: Lexer<'input>,
}

impl<'input> Parser<'input> {
    /// Create a new parser
    pub fn new(input: &'input str) -> Self {
        Self {
            lexer: Lexer::new(input),
        }
    }
    
    /// Parse a complete GraphQL schema document
    pub fn parse_schema_document(&mut self) -> Result<Schema, ParseError> {
        let mut schema_builder = SchemaBuilder::new();
        
        // Parse all type definitions
        while self.lexer.current_token().is_some() {
            let definition = self.parse_type_system_definition()?;
            schema_builder.add_definition(definition)?;
        }
        
        schema_builder.build()
    }
    
    /// Parse a single type system definition
    pub fn parse_type_system_definition(&mut self) -> Result<TypeSystemDefinition, ParseError> {
        match self.lexer.current_token() {
            Some(Token::Schema) => self.parse_schema_definition(),
            Some(Token::Type) => self.parse_object_type_definition(),
            Some(Token::Interface) => self.parse_interface_type_definition(),
            Some(Token::Union) => self.parse_union_type_definition(),
            Some(Token::Scalar) => self.parse_scalar_type_definition(),
            Some(Token::Enum) => self.parse_enum_type_definition(),
            Some(Token::Input) => self.parse_input_object_type_definition(),
            Some(Token::Directive) => self.parse_directive_definition(),
            Some(token) => Err(ParseError::UnexpectedToken {
                expected: "type system definition".to_string(),
                found: format!("{}", token),
                position: self.lexer.position(),
            }),
            None => Err(ParseError::UnexpectedEof {
                expected: "type system definition".to_string(),
            }),
        }
    }
    
    /// Parse schema definition
    fn parse_schema_definition(&mut self) -> Result<TypeSystemDefinition, ParseError> {
        self.expect_token(&Token::Schema)?;
        
        let description = None; // TODO: Parse description if present
        let directives = self.parse_directives()?;
        
        self.expect_token(&Token::LeftBrace)?;
        
        let mut query_type = None;
        let mut mutation_type = None;
        let mut subscription_type = None;
        
        while !self.is_current_token(&Token::RightBrace) {
            match self.lexer.current_token() {
                Some(Token::Name(name)) => {
                    let operation_type = name.clone();
                    self.lexer.advance();
                    self.expect_token(&Token::Colon)?;
                    
                    let type_name = self.parse_named_type()?;
                    
                    match operation_type.as_str() {
                        "query" => query_type = Some(type_name),
                        "mutation" => mutation_type = Some(type_name),
                        "subscription" => subscription_type = Some(type_name),
                        _ => return Err(ParseError::InvalidSyntax {
                            position: self.lexer.position(),
                            message: format!("Unknown operation type: {}", operation_type),
                        }),
                    }
                },
                Some(token) => return Err(ParseError::UnexpectedToken {
                    expected: "operation type".to_string(),
                    found: format!("{}", token),
                    position: self.lexer.position(),
                }),
                None => return Err(ParseError::UnexpectedEof {
                    expected: "operation type or '}'".to_string(),
                }),
            }
        }
        
        self.expect_token(&Token::RightBrace)?;
        
        Ok(TypeSystemDefinition::Schema(SchemaDefinition {
            description,
            query_type: query_type.ok_or_else(|| ParseError::InvalidSyntax {
                position: self.lexer.position(),
                message: "Schema must have a query type".to_string(),
            })?,
            mutation_type,
            subscription_type,
            directives,
        }))
    }
    
    /// Parse object type definition
    fn parse_object_type_definition(&mut self) -> Result<TypeSystemDefinition, ParseError> {
        self.expect_token(&Token::Type)?;
        
        let name = self.parse_name()?;
        let interfaces = self.parse_implements_interfaces()?;
        let directives = self.parse_directives()?;
        let fields = self.parse_fields_definition()?;
        
        let object_type = ObjectType {
            name: name.clone(),
            description: None,
            fields,
            interfaces,
        };
        
        Ok(TypeSystemDefinition::Type(TypeDefinition::Object(object_type)))
    }
    
    /// Parse interface type definition
    fn parse_interface_type_definition(&mut self) -> Result<TypeSystemDefinition, ParseError> {
        self.expect_token(&Token::Interface)?;
        
        let name = self.parse_name()?;
        let directives = self.parse_directives()?;
        let fields = self.parse_fields_definition()?;
        
        let interface_type = InterfaceType {
            name: name.clone(),
            description: None,
            fields,
        };
        
        Ok(TypeSystemDefinition::Type(TypeDefinition::Interface(interface_type)))
    }
    
    /// Parse union type definition
    fn parse_union_type_definition(&mut self) -> Result<TypeSystemDefinition, ParseError> {
        self.expect_token(&Token::Union)?;
        
        let name = self.parse_name()?;
        let directives = self.parse_directives()?;
        
        self.expect_token(&Token::Equals)?;
        
        let mut types = vec![self.parse_named_type()?];
        
        while self.is_current_token(&Token::Pipe) {
            self.lexer.advance(); // consume |
            types.push(self.parse_named_type()?);
        }
        
        let union_type = UnionType {
            name: name.clone(),
            description: None,
            types,
        };
        
        Ok(TypeSystemDefinition::Type(TypeDefinition::Union(union_type)))
    }
    
    /// Parse scalar type definition
    fn parse_scalar_type_definition(&mut self) -> Result<TypeSystemDefinition, ParseError> {
        self.expect_token(&Token::Scalar)?;
        
        let name = self.parse_name()?;
        let _directives = self.parse_directives()?;
        
        let scalar_type = ScalarType::Custom(name.clone());
        
        Ok(TypeSystemDefinition::Type(TypeDefinition::Scalar(scalar_type)))
    }
    
    /// Parse enum type definition
    fn parse_enum_type_definition(&mut self) -> Result<TypeSystemDefinition, ParseError> {
        self.expect_token(&Token::Enum)?;
        
        let name = self.parse_name()?;
        let _directives = self.parse_directives()?;
        
        self.expect_token(&Token::LeftBrace)?;
        
        let mut values = HashMap::new();
        
        while !self.is_current_token(&Token::RightBrace) {
            let value_name = self.parse_name()?;
            let _value_directives = self.parse_directives()?;
            
            let enum_value = EnumValue {
                description: None,
                deprecation_reason: None,
            };
            
            values.insert(value_name, enum_value);
        }
        
        self.expect_token(&Token::RightBrace)?;
        
        let enum_type = EnumType {
            name: name.clone(),
            description: None,
            values,
        };
        
        Ok(TypeSystemDefinition::Type(TypeDefinition::Enum(enum_type)))
    }
    
    /// Parse input object type definition
    fn parse_input_object_type_definition(&mut self) -> Result<TypeSystemDefinition, ParseError> {
        self.expect_token(&Token::Input)?;
        
        let name = self.parse_name()?;
        let _directives = self.parse_directives()?;
        
        let fields = self.parse_input_fields_definition()?;
        
        let input_object_type = InputObjectType {
            name: name.clone(),
            description: None,
            fields,
        };
        
        Ok(TypeSystemDefinition::Type(TypeDefinition::InputObject(input_object_type)))
    }
    
    /// Parse directive definition
    fn parse_directive_definition(&mut self) -> Result<TypeSystemDefinition, ParseError> {
        self.expect_token(&Token::Directive)?;
        self.expect_token(&Token::At)?;
        
        let name = self.parse_name()?;
        let arguments = self.parse_arguments_definition()?;
        
        let is_repeatable = if self.is_current_token(&Token::Repeatable) {
            self.lexer.advance();
            true
        } else {
            false
        };
        
        self.expect_token(&Token::On)?;
        
        let locations = self.parse_directive_locations()?;
        
        let directive = DirectiveDefinition {
            name: name.clone(),
            description: None,
            locations,
            arguments,
            is_repeatable,
        };
        
        Ok(TypeSystemDefinition::Directive(directive))
    }
    
    /// Parse implements interfaces clause
    fn parse_implements_interfaces(&mut self) -> Result<Vec<String>, ParseError> {
        if !self.is_current_token(&Token::Implements) {
            return Ok(vec![]);
        }
        
        self.lexer.advance(); // consume 'implements'
        
        let mut interfaces = vec![self.parse_named_type()?];
        
        while self.is_current_token(&Token::Name("&".to_string())) {
            self.lexer.advance(); // consume &
            interfaces.push(self.parse_named_type()?);
        }
        
        Ok(interfaces)
    }
    
    /// Parse fields definition
    fn parse_fields_definition(&mut self) -> Result<HashMap<String, FieldDefinition>, ParseError> {
        self.expect_token(&Token::LeftBrace)?;
        
        let mut fields = HashMap::new();
        
        while !self.is_current_token(&Token::RightBrace) {
            let field = self.parse_field_definition()?;
            fields.insert(field.name.clone(), field);
        }
        
        self.expect_token(&Token::RightBrace)?;
        
        Ok(fields)
    }
    
    /// Parse input fields definition
    fn parse_input_fields_definition(&mut self) -> Result<HashMap<String, InputFieldDefinition>, ParseError> {
        self.expect_token(&Token::LeftBrace)?;
        
        let mut fields = HashMap::new();
        
        while !self.is_current_token(&Token::RightBrace) {
            let field = self.parse_input_field_definition()?;
            fields.insert(field.name.clone(), field);
        }
        
        self.expect_token(&Token::RightBrace)?;
        
        Ok(fields)
    }
    
    /// Parse field definition
    fn parse_field_definition(&mut self) -> Result<FieldDefinition, ParseError> {
        let name = self.parse_name()?;
        let arguments = self.parse_arguments_definition()?;
        
        self.expect_token(&Token::Colon)?;
        
        let field_type = self.parse_type()?;
        let _directives = self.parse_directives()?;
        
        Ok(FieldDefinition {
            name,
            description: None,
            field_type,
            arguments,
            deprecation_reason: None,
        })
    }
    
    /// Parse input field definition
    fn parse_input_field_definition(&mut self) -> Result<InputFieldDefinition, ParseError> {
        let name = self.parse_name()?;
        
        self.expect_token(&Token::Colon)?;
        
        let field_type = self.parse_type()?;
        
        let default_value = if self.is_current_token(&Token::Equals) {
            self.lexer.advance(); // consume =
            Some(self.parse_value()?)
        } else {
            None
        };
        
        let _directives = self.parse_directives()?;
        
        Ok(InputFieldDefinition {
            name,
            description: None,
            field_type,
            default_value,
        })
    }
    
    /// Parse arguments definition
    fn parse_arguments_definition(&mut self) -> Result<HashMap<String, InputFieldDefinition>, ParseError> {
        if !self.is_current_token(&Token::LeftParen) {
            return Ok(HashMap::new());
        }
        
        self.lexer.advance(); // consume (
        
        let mut arguments = HashMap::new();
        
        while !self.is_current_token(&Token::RightParen) {
            let arg = self.parse_input_field_definition()?;
            arguments.insert(arg.name.clone(), arg);
        }
        
        self.expect_token(&Token::RightParen)?;
        
        Ok(arguments)
    }
    
    /// Parse directive locations
    fn parse_directive_locations(&mut self) -> Result<Vec<DirectiveLocation>, ParseError> {
        let mut locations = vec![self.parse_directive_location()?];
        
        while self.is_current_token(&Token::Pipe) {
            self.lexer.advance(); // consume |
            locations.push(self.parse_directive_location()?);
        }
        
        Ok(locations)
    }
    
    /// Parse directive location
    fn parse_directive_location(&mut self) -> Result<DirectiveLocation, ParseError> {
        match self.lexer.current_token() {
            Some(Token::Name(name)) => {
                let location = match name.as_str() {
                    "QUERY" => DirectiveLocation::Query,
                    "MUTATION" => DirectiveLocation::Mutation,
                    "SUBSCRIPTION" => DirectiveLocation::Subscription,
                    "FIELD" => DirectiveLocation::Field,
                    "FRAGMENT_DEFINITION" => DirectiveLocation::FragmentDefinition,
                    "FRAGMENT_SPREAD" => DirectiveLocation::FragmentSpread,
                    "INLINE_FRAGMENT" => DirectiveLocation::InlineFragment,
                    "VARIABLE_DEFINITION" => DirectiveLocation::VariableDefinition,
                    "SCHEMA" => DirectiveLocation::Schema,
                    "SCALAR" => DirectiveLocation::Scalar,
                    "OBJECT" => DirectiveLocation::Object,
                    "FIELD_DEFINITION" => DirectiveLocation::FieldDefinition,
                    "ARGUMENT_DEFINITION" => DirectiveLocation::ArgumentDefinition,
                    "INTERFACE" => DirectiveLocation::Interface,
                    "UNION" => DirectiveLocation::Union,
                    "ENUM" => DirectiveLocation::Enum,
                    "ENUM_VALUE" => DirectiveLocation::EnumValue,
                    "INPUT_OBJECT" => DirectiveLocation::InputObject,
                    "INPUT_FIELD_DEFINITION" => DirectiveLocation::InputFieldDefinition,
                    _ => return Err(ParseError::InvalidSyntax {
                        position: self.lexer.position(),
                        message: format!("Unknown directive location: {}", name),
                    }),
                };
                self.lexer.advance();
                Ok(location)
            },
            Some(token) => Err(ParseError::UnexpectedToken {
                expected: "directive location".to_string(),
                found: format!("{}", token),
                position: self.lexer.position(),
            }),
            None => Err(ParseError::UnexpectedEof {
                expected: "directive location".to_string(),
            }),
        }
    }
    
    /// Parse directives
    fn parse_directives(&mut self) -> Result<Vec<String>, ParseError> {
        let mut directives = Vec::new();
        
        while self.is_current_token(&Token::At) {
            self.lexer.advance(); // consume @
            let name = self.parse_name()?;
            directives.push(name);
            
            // Skip arguments for now (TODO: implement proper directive arguments parsing)
            if self.is_current_token(&Token::LeftParen) {
                self.skip_arguments()?;
            }
        }
        
        Ok(directives)
    }
    
    /// Parse type
    fn parse_type(&mut self) -> Result<GraphQLType, ParseError> {
        let mut base_type = self.parse_named_type_or_list_type()?;
        
        if self.is_current_token(&Token::Bang) {
            self.lexer.advance(); // consume !
            base_type = GraphQLType::NonNull(Box::new(base_type));
        }
        
        Ok(base_type)
    }
    
    /// Parse named type or list type
    fn parse_named_type_or_list_type(&mut self) -> Result<GraphQLType, ParseError> {
        if self.is_current_token(&Token::LeftBracket) {
            self.lexer.advance(); // consume [
            let inner_type = self.parse_type()?;
            self.expect_token(&Token::RightBracket)?;
            Ok(GraphQLType::List(Box::new(inner_type)))
        } else {
            let name = self.parse_named_type()?;
            // Convert named type to appropriate GraphQLType
            // This is simplified - in a real implementation, you'd need to resolve the type
            match name.as_str() {
                "String" => Ok(GraphQLType::Scalar(ScalarType::String)),
                "Int" => Ok(GraphQLType::Scalar(ScalarType::Int)),
                "Float" => Ok(GraphQLType::Scalar(ScalarType::Float)),
                "Boolean" => Ok(GraphQLType::Scalar(ScalarType::Boolean)),
                "ID" => Ok(GraphQLType::Scalar(ScalarType::ID)),
                _ => Ok(GraphQLType::Scalar(ScalarType::Custom(name))),
            }
        }
    }
    
    /// Parse named type
    fn parse_named_type(&mut self) -> Result<String, ParseError> {
        self.parse_name()
    }
    
    /// Parse name
    fn parse_name(&mut self) -> Result<String, ParseError> {
        match self.lexer.current_token() {
            Some(Token::Name(name)) => {
                let result = name.clone();
                self.lexer.advance();
                Ok(result)
            },
            Some(token) => Err(ParseError::UnexpectedToken {
                expected: "name".to_string(),
                found: format!("{}", token),
                position: self.lexer.position(),
            }),
            None => Err(ParseError::UnexpectedEof {
                expected: "name".to_string(),
            }),
        }
    }
    
    /// Parse value
    fn parse_value(&mut self) -> Result<Value, ParseError> {
        match self.lexer.current_token() {
            Some(Token::String(s)) => {
                let result = Value::String(s.clone());
                self.lexer.advance();
                Ok(result)
            },
            Some(Token::Integer(i)) => {
                let result = Value::Int(*i);
                self.lexer.advance();
                Ok(result)
            },
            Some(Token::Float(f)) => {
                let result = Value::Float(*f);
                self.lexer.advance();
                Ok(result)
            },
            Some(Token::True) => {
                self.lexer.advance();
                Ok(Value::Boolean(true))
            },
            Some(Token::False) => {
                self.lexer.advance();
                Ok(Value::Boolean(false))
            },
            Some(Token::Null) => {
                self.lexer.advance();
                Ok(Value::Null)
            },
            Some(Token::Name(name)) => {
                let result = Value::Enum(name.clone());
                self.lexer.advance();
                Ok(result)
            },
            Some(Token::LeftBracket) => self.parse_list_value(),
            Some(Token::LeftBrace) => self.parse_object_value(),
            Some(token) => Err(ParseError::UnexpectedToken {
                expected: "value".to_string(),
                found: format!("{}", token),
                position: self.lexer.position(),
            }),
            None => Err(ParseError::UnexpectedEof {
                expected: "value".to_string(),
            }),
        }
    }
    
    /// Parse list value
    fn parse_list_value(&mut self) -> Result<Value, ParseError> {
        self.expect_token(&Token::LeftBracket)?;
        
        let mut values = Vec::new();
        
        while !self.is_current_token(&Token::RightBracket) {
            values.push(self.parse_value()?);
        }
        
        self.expect_token(&Token::RightBracket)?;
        
        Ok(Value::List(values))
    }
    
    /// Parse object value
    fn parse_object_value(&mut self) -> Result<Value, ParseError> {
        self.expect_token(&Token::LeftBrace)?;
        
        let mut fields = HashMap::new();
        
        while !self.is_current_token(&Token::RightBrace) {
            let name = self.parse_name()?;
            self.expect_token(&Token::Colon)?;
            let value = self.parse_value()?;
            fields.insert(name, value);
        }
        
        self.expect_token(&Token::RightBrace)?;
        
        Ok(Value::Object(fields))
    }
    
    /// Skip arguments (temporary implementation)
    fn skip_arguments(&mut self) -> Result<(), ParseError> {
        self.expect_token(&Token::LeftParen)?;
        
        let mut depth = 1;
        while depth > 0 {
            match self.lexer.current_token() {
                Some(Token::LeftParen) => depth += 1,
                Some(Token::RightParen) => depth -= 1,
                Some(_) => {},
                None => return Err(ParseError::UnexpectedEof {
                    expected: "')' to close arguments".to_string(),
                }),
            }
            self.lexer.advance();
        }
        
        Ok(())
    }
    
    /// Expect a specific token
    fn expect_token(&mut self, expected: &Token) -> Result<(), ParseError> {
        self.lexer.expect(expected).map_err(ParseError::from)
    }
    
    /// Check if current token matches expected
    fn is_current_token(&self, expected: &Token) -> bool {
        self.lexer.is_current(expected)
    }
}

/// Helper struct for building schemas
struct SchemaBuilder {
    query_type: Option<String>,
    mutation_type: Option<String>,
    subscription_type: Option<String>,
    types: HashMap<String, GraphQLType>,
    directives: HashMap<String, DirectiveDefinition>,
}

impl SchemaBuilder {
    fn new() -> Self {
        Self {
            query_type: None,
            mutation_type: None,
            subscription_type: None,
            types: HashMap::new(),
            directives: HashMap::new(),
        }
    }
    
    fn add_definition(&mut self, definition: TypeSystemDefinition) -> Result<(), ParseError> {
        match definition {
            TypeSystemDefinition::Schema(schema_def) => {
                self.query_type = Some(schema_def.query_type);
                self.mutation_type = schema_def.mutation_type;
                self.subscription_type = schema_def.subscription_type;
            },
            TypeSystemDefinition::Type(type_def) => {
                let graphql_type = match type_def {
                    TypeDefinition::Scalar(scalar) => GraphQLType::Scalar(scalar),
                    TypeDefinition::Object(object) => GraphQLType::Object(object),
                    TypeDefinition::Interface(interface) => GraphQLType::Interface(interface),
                    TypeDefinition::Union(union) => GraphQLType::Union(union),
                    TypeDefinition::Enum(enum_type) => GraphQLType::Enum(enum_type),
                    TypeDefinition::InputObject(input_object) => GraphQLType::InputObject(input_object),
                };
                
                if let Some(name) = graphql_type.name() {
                    self.types.insert(name.to_string(), graphql_type);
                }
            },
            TypeSystemDefinition::Directive(directive) => {
                let name = directive.name.clone();
                self.directives.insert(name, directive);
            },
        }
        Ok(())
    }
    
    fn build(self) -> Result<Schema, ParseError> {
        let query_type = self.query_type.unwrap_or_else(|| "Query".to_string());
        
        let mut schema = Schema::new(query_type);
        schema.mutation_type = self.mutation_type;
        schema.subscription_type = self.subscription_type;
        
        for (name, type_def) in self.types {
            schema.add_type(type_def).map_err(|e| ParseError::InvalidSyntax {
                position: 0,
                message: format!("Failed to add type {}: {}", name, e),
            })?;
        }
        
        for (name, directive) in self.directives {
            schema.add_directive(directive).map_err(|e| ParseError::InvalidSyntax {
                position: 0,
                message: format!("Failed to add directive {}: {}", name, e),
            })?;
        }
        
        Ok(schema)
    }
}

/// Type system definitions
#[derive(Debug, Clone)]
pub enum TypeSystemDefinition {
    Schema(SchemaDefinition),
    Type(TypeDefinition),
    Directive(DirectiveDefinition),
}

/// Schema definition
#[derive(Debug, Clone)]
pub struct SchemaDefinition {
    pub description: Option<String>,
    pub query_type: String,
    pub mutation_type: Option<String>,
    pub subscription_type: Option<String>,
    pub directives: Vec<String>,
}

/// Type definitions
#[derive(Debug, Clone)]
pub enum TypeDefinition {
    Scalar(ScalarType),
    Object(ObjectType),
    Interface(InterfaceType),
    Union(UnionType),
    Enum(EnumType),
    InputObject(InputObjectType),
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn parse_simple_scalar_definition() {
        let input = "scalar DateTime";
        let mut parser = Parser::new(input);
        
        let result = parser.parse_type_system_definition();
        assert!(result.is_ok());
        
        match result.unwrap() {
            TypeSystemDefinition::Type(TypeDefinition::Scalar(ScalarType::Custom(name))) => {
                assert_eq!(name, "DateTime");
            },
            _ => panic!("Expected scalar type definition"),
        }
    }
    
    #[test]
    fn parse_simple_object_type() {
        let input = r#"
        type User {
            id: ID!
            name: String
            age: Int
        }
        "#;
        let mut parser = Parser::new(input);
        
        let result = parser.parse_type_system_definition();
        assert!(result.is_ok());
        
        match result.unwrap() {
            TypeSystemDefinition::Type(TypeDefinition::Object(obj)) => {
                assert_eq!(obj.name, "User");
                assert!(obj.fields.contains_key("id"));
                assert!(obj.fields.contains_key("name"));
                assert!(obj.fields.contains_key("age"));
            },
            _ => panic!("Expected object type definition"),
        }
    }
    
    #[test]
    fn parse_enum_type() {
        let input = r#"
        enum Status {
            ACTIVE
            INACTIVE
            PENDING
        }
        "#;
        let mut parser = Parser::new(input);
        
        let result = parser.parse_type_system_definition();
        assert!(result.is_ok());
        
        match result.unwrap() {
            TypeSystemDefinition::Type(TypeDefinition::Enum(enum_type)) => {
                assert_eq!(enum_type.name, "Status");
                assert!(enum_type.values.contains_key("ACTIVE"));
                assert!(enum_type.values.contains_key("INACTIVE"));
                assert!(enum_type.values.contains_key("PENDING"));
            },
            _ => panic!("Expected enum type definition"),
        }
    }
    
    #[test]
    fn parse_union_type() {
        let input = "union SearchResult = User | Post | Comment";
        let mut parser = Parser::new(input);
        
        let result = parser.parse_type_system_definition();
        assert!(result.is_ok());
        
        match result.unwrap() {
            TypeSystemDefinition::Type(TypeDefinition::Union(union_type)) => {
                assert_eq!(union_type.name, "SearchResult");
                assert_eq!(union_type.types.len(), 3);
                assert!(union_type.types.contains(&"User".to_string()));
                assert!(union_type.types.contains(&"Post".to_string()));
                assert!(union_type.types.contains(&"Comment".to_string()));
            },
            _ => panic!("Expected union type definition"),
        }
    }
    
    #[test]
    fn parse_schema_definition() {
        let input = r#"
        schema {
            query: Query
            mutation: Mutation
        }
        "#;
        let mut parser = Parser::new(input);
        
        let result = parser.parse_type_system_definition();
        assert!(result.is_ok());
        
        match result.unwrap() {
            TypeSystemDefinition::Schema(schema_def) => {
                assert_eq!(schema_def.query_type, "Query");
                assert_eq!(schema_def.mutation_type, Some("Mutation".to_string()));
                assert!(schema_def.subscription_type.is_none());
            },
            _ => panic!("Expected schema definition"),
        }
    }
}
