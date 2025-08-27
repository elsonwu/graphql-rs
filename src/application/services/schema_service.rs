use crate::domain::entities::schema::{Schema, SchemaError};
use crate::infrastructure::parser::{ParseError, Parser};
use std::sync::Arc;

/// Error type for schema service operations
#[derive(Debug)]
pub enum SchemaServiceError {
    /// Parse error from the GraphQL schema
    ParseError(ParseError),

    /// Validation error containing multiple schema errors  
    ValidationError {
        /// List of schema validation errors
        errors: Vec<SchemaError>,
    },

    /// Schema not found error
    SchemaNotFound,

    /// I/O error
    IoError(std::io::Error),
}

impl std::fmt::Display for SchemaServiceError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            SchemaServiceError::ParseError(e) => write!(f, "Parse error: {e}"),
            SchemaServiceError::ValidationError { errors } => {
                write!(f, "Validation error: {} errors found", errors.len())
            },
            SchemaServiceError::SchemaNotFound => write!(f, "Schema not found"),
            SchemaServiceError::IoError(e) => write!(f, "I/O error: {e}"),
        }
    }
}

impl std::error::Error for SchemaServiceError {}

impl From<ParseError> for SchemaServiceError {
    fn from(err: ParseError) -> Self {
        SchemaServiceError::ParseError(err)
    }
}

impl From<std::io::Error> for SchemaServiceError {
    fn from(err: std::io::Error) -> Self {
        SchemaServiceError::IoError(err)
    }
}

/// Service for managing GraphQL schemas
pub struct SchemaService {
    current_schema: Option<Arc<Schema>>,
}

impl SchemaService {
    /// Create a new schema service
    #[must_use]
    pub fn new() -> Self {
        Self {
            current_schema: None,
        }
    }

    /// Parse and load a schema from SDL (Schema Definition Language) string
    pub fn load_schema_from_sdl(&mut self, sdl: &str) -> Result<Arc<Schema>, SchemaServiceError> {
        // Parse the schema document
        let mut parser = Parser::new(sdl);
        let schema = parser.parse_schema_document()?;

        // Validate the schema
        if let Err(errors) = schema.validate() {
            return Err(SchemaServiceError::ValidationError { errors });
        }

        // Store the schema
        let schema_arc = Arc::new(schema);
        self.current_schema = Some(schema_arc.clone());

        Ok(schema_arc)
    }

    /// Load a schema from a file
    pub fn load_schema_from_file(
        &mut self,
        file_path: &str,
    ) -> Result<Arc<Schema>, SchemaServiceError> {
        let content = std::fs::read_to_string(file_path)?;
        self.load_schema_from_sdl(&content)
    }

    /// Get the current schema
    #[must_use]
    pub fn get_schema(&self) -> Option<Arc<Schema>> {
        self.current_schema.clone()
    }

    /// Check if a schema is loaded
    #[must_use]
    pub fn has_schema(&self) -> bool {
        self.current_schema.is_some()
    }

    /// Build a schema programmatically
    pub fn build_schema(&mut self) -> SchemaBuilder<'_> {
        SchemaBuilder::new(self)
    }

    /// Get schema introspection information
    pub fn introspect(&self) -> Result<IntrospectionResult, SchemaServiceError> {
        let schema = self
            .current_schema
            .as_ref()
            .ok_or(SchemaServiceError::SchemaNotFound)?;

        Ok(IntrospectionResult::from_schema(schema))
    }

    /// Validate a schema without loading it
    pub fn validate_schema_sdl(sdl: &str) -> Result<Vec<String>, SchemaServiceError> {
        let mut parser = Parser::new(sdl);
        let schema = parser.parse_schema_document()?;

        match schema.validate() {
            Ok(()) => Ok(vec!["Schema is valid".to_string()]),
            Err(errors) => {
                let error_messages: Vec<String> = errors.iter().map(|e| format!("{e}")).collect();
                Ok(error_messages)
            },
        }
    }

    /// Get schema statistics
    pub fn get_schema_stats(&self) -> Result<SchemaStats, SchemaServiceError> {
        let schema = self
            .current_schema
            .as_ref()
            .ok_or(SchemaServiceError::SchemaNotFound)?;

        Ok(SchemaStats::from_schema(schema))
    }
}

impl Default for SchemaService {
    fn default() -> Self {
        Self::new()
    }
}

/// Builder for programmatically creating schemas
pub struct SchemaBuilder<'a> {
    service: &'a mut SchemaService,
    schema: Schema,
}

impl<'a> SchemaBuilder<'a> {
    fn new(service: &'a mut SchemaService) -> Self {
        let schema = Schema::new("Query".to_string());
        Self { service, schema }
    }

    /// Set the query root type
    #[must_use]
    pub fn query_type(mut self, type_name: &str) -> Self {
        self.schema.query_type = type_name.to_string();
        self
    }

    /// Set the mutation root type
    #[must_use]
    pub fn mutation_type(mut self, type_name: &str) -> Self {
        self.schema.mutation_type = Some(type_name.to_string());
        self
    }

    /// Set the subscription root type
    #[must_use]
    pub fn subscription_type(mut self, type_name: &str) -> Self {
        self.schema.subscription_type = Some(type_name.to_string());
        self
    }

    /// Add a type to the schema
    pub fn add_type(
        mut self,
        type_def: crate::domain::entities::types::GraphQLType,
    ) -> Result<Self, SchemaServiceError> {
        self.schema
            .add_type(type_def)
            .map_err(|e| SchemaServiceError::ValidationError { errors: vec![e] })?;
        Ok(self)
    }

    /// Add a directive to the schema
    pub fn add_directive(
        mut self,
        directive: crate::domain::entities::types::DirectiveDefinition,
    ) -> Result<Self, SchemaServiceError> {
        self.schema
            .add_directive(directive)
            .map_err(|e| SchemaServiceError::ValidationError { errors: vec![e] })?;
        Ok(self)
    }

    /// Build and load the schema
    pub fn build(self) -> Result<Arc<Schema>, SchemaServiceError> {
        // Validate the schema
        if let Err(errors) = self.schema.validate() {
            return Err(SchemaServiceError::ValidationError { errors });
        }

        // Store the schema
        let schema_arc = Arc::new(self.schema);
        self.service.current_schema = Some(schema_arc.clone());

        Ok(schema_arc)
    }
}

/// Schema introspection result
#[derive(Debug, Clone)]
pub struct IntrospectionResult {
    /// Schema metadata information
    pub schema_info: SchemaInfo,
    /// List of types in the schema
    pub types: Vec<TypeInfo>,
    /// List of directives in the schema  
    pub directives: Vec<DirectiveInfo>,
}

impl IntrospectionResult {
    fn from_schema(schema: &Schema) -> Self {
        let schema_info = SchemaInfo {
            query_type: schema.query_type.clone(),
            mutation_type: schema.mutation_type.clone(),
            subscription_type: schema.subscription_type.clone(),
            description: schema.description.clone(),
        };

        let types: Vec<TypeInfo> = schema
            .types
            .iter()
            .map(|(name, type_def)| TypeInfo::from_graphql_type(name, type_def))
            .collect();

        let directives: Vec<DirectiveInfo> = schema
            .directives
            .iter()
            .map(|(name, directive)| DirectiveInfo::from_directive(name, directive))
            .collect();

        Self {
            schema_info,
            types,
            directives,
        }
    }
}

/// Schema information for introspection
#[derive(Debug, Clone)]
pub struct SchemaInfo {
    /// The root query type name
    pub query_type: String,
    /// The root mutation type name (optional)
    pub mutation_type: Option<String>,
    /// The root subscription type name (optional)
    pub subscription_type: Option<String>,
    /// Schema description (optional)
    pub description: Option<String>,
}

/// Type information for introspection
#[derive(Debug, Clone)]
pub struct TypeInfo {
    /// The type name
    pub name: String,
    /// The type kind (SCALAR, OBJECT, etc.)
    pub kind: String,
    /// The type description (optional)
    pub description: Option<String>,
    /// Fields of the type (for OBJECT and INTERFACE)
    pub fields: Option<Vec<FieldInfo>>,
    /// Possible types (for INTERFACE and UNION)
    pub possible_types: Option<Vec<String>>,
    /// Enum values (for ENUM types)
    pub enum_values: Option<Vec<String>>,
    /// Input fields (for `INPUT_OBJECT` types)
    pub input_fields: Option<Vec<String>>,
}

impl TypeInfo {
    fn from_graphql_type(
        name: &str,
        type_def: &crate::domain::entities::types::GraphQLType,
    ) -> Self {
        use crate::domain::entities::types::GraphQLType;

        match type_def {
            GraphQLType::Scalar(_) => Self {
                name: name.to_string(),
                kind: "SCALAR".to_string(),
                description: None,
                fields: None,
                possible_types: None,
                enum_values: None,
                input_fields: None,
            },
            GraphQLType::Object(obj) => Self {
                name: name.to_string(),
                kind: "OBJECT".to_string(),
                description: obj.description.clone(),
                fields: Some(
                    obj.fields
                        .iter()
                        .map(|(field_name, field)| {
                            FieldInfo::from_field_definition(field_name, field)
                        })
                        .collect(),
                ),
                possible_types: None,
                enum_values: None,
                input_fields: None,
            },
            GraphQLType::Interface(interface) => Self {
                name: name.to_string(),
                kind: "INTERFACE".to_string(),
                description: interface.description.clone(),
                fields: Some(
                    interface
                        .fields
                        .iter()
                        .map(|(field_name, field)| {
                            FieldInfo::from_field_definition(field_name, field)
                        })
                        .collect(),
                ),
                possible_types: None,
                enum_values: None,
                input_fields: None,
            },
            GraphQLType::Union(union) => Self {
                name: name.to_string(),
                kind: "UNION".to_string(),
                description: union.description.clone(),
                fields: None,
                possible_types: Some(union.types.clone()),
                enum_values: None,
                input_fields: None,
            },
            GraphQLType::Enum(enum_type) => Self {
                name: name.to_string(),
                kind: "ENUM".to_string(),
                description: enum_type.description.clone(),
                fields: None,
                possible_types: None,
                enum_values: Some(enum_type.values.keys().cloned().collect()),
                input_fields: None,
            },
            GraphQLType::InputObject(input_obj) => Self {
                name: name.to_string(),
                kind: "INPUT_OBJECT".to_string(),
                description: input_obj.description.clone(),
                fields: None,
                possible_types: None,
                enum_values: None,
                input_fields: Some(input_obj.fields.keys().cloned().collect()),
            },
            GraphQLType::List(_) => Self {
                name: name.to_string(),
                kind: "LIST".to_string(),
                description: None,
                fields: None,
                possible_types: None,
                enum_values: None,
                input_fields: None,
            },
            GraphQLType::NonNull(_) => Self {
                name: name.to_string(),
                kind: "NON_NULL".to_string(),
                description: None,
                fields: None,
                possible_types: None,
                enum_values: None,
                input_fields: None,
            },
        }
    }
}

/// Field information for introspection
#[derive(Debug, Clone)]
pub struct FieldInfo {
    /// The field name
    pub name: String,
    /// The field type name
    pub type_name: String,
    /// The field description (optional)
    pub description: Option<String>,
    /// Whether the field is deprecated
    pub is_deprecated: bool,
    /// The deprecation reason (optional)
    pub deprecation_reason: Option<String>,
    /// List of field arguments
    pub arguments: Vec<String>,
}

impl FieldInfo {
    fn from_field_definition(
        name: &str,
        field: &crate::domain::entities::types::FieldDefinition,
    ) -> Self {
        Self {
            name: name.to_string(),
            type_name: format!("{}", field.field_type),
            description: field.description.clone(),
            is_deprecated: field.deprecation_reason.is_some(),
            deprecation_reason: field.deprecation_reason.clone(),
            arguments: field.arguments.keys().cloned().collect(),
        }
    }
}

/// Directive information for introspection
#[derive(Debug, Clone)]
pub struct DirectiveInfo {
    /// The directive name
    pub name: String,
    /// The directive description (optional)
    pub description: Option<String>,
    /// Valid locations for the directive
    pub locations: Vec<String>,
    /// List of directive arguments
    pub arguments: Vec<String>,
    /// Whether the directive is repeatable
    pub is_repeatable: bool,
}

impl DirectiveInfo {
    fn from_directive(
        name: &str,
        directive: &crate::domain::entities::types::DirectiveDefinition,
    ) -> Self {
        Self {
            name: name.to_string(),
            description: directive.description.clone(),
            locations: directive
                .locations
                .iter()
                .map(|loc| format!("{loc:?}"))
                .collect(),
            arguments: directive.arguments.keys().cloned().collect(),
            is_repeatable: directive.is_repeatable,
        }
    }
}

/// Schema statistics
#[derive(Debug, Clone)]
pub struct SchemaStats {
    /// Total number of types in the schema
    pub total_types: usize,
    /// Number of scalar types
    pub scalar_types: usize,
    /// Number of object types
    pub object_types: usize,
    /// Number of interface types
    pub interface_types: usize,
    /// Number of union types  
    pub union_types: usize,
    /// Number of enum types
    pub enum_types: usize,
    /// Number of input object types
    pub input_types: usize,
    /// Total number of fields across all types
    pub total_fields: usize,
    /// Total number of directives
    pub total_directives: usize,
}

impl SchemaStats {
    fn from_schema(schema: &Schema) -> Self {
        use crate::domain::entities::types::GraphQLType;

        let mut stats = SchemaStats {
            total_types: schema.types.len(),
            scalar_types: 0,
            object_types: 0,
            interface_types: 0,
            union_types: 0,
            enum_types: 0,
            input_types: 0,
            total_fields: 0,
            total_directives: schema.directives.len(),
        };

        for type_def in schema.types.values() {
            match type_def {
                GraphQLType::Scalar(_) => stats.scalar_types += 1,
                GraphQLType::Object(obj) => {
                    stats.object_types += 1;
                    stats.total_fields += obj.fields.len();
                },
                GraphQLType::Interface(interface) => {
                    stats.interface_types += 1;
                    stats.total_fields += interface.fields.len();
                },
                GraphQLType::Union(_) => stats.union_types += 1,
                GraphQLType::Enum(_) => stats.enum_types += 1,
                GraphQLType::InputObject(input) => {
                    stats.input_types += 1;
                    stats.total_fields += input.fields.len();
                },
                _ => {},
            }
        }

        stats
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::entities::types::*;
    use std::collections::HashMap;

    #[test]
    fn create_schema_service() {
        let service = SchemaService::new();
        assert!(!service.has_schema());
        assert!(service.get_schema().is_none());
    }

    #[test]
    fn load_simple_schema_from_sdl() {
        let mut service = SchemaService::new();

        let sdl = r"
        type Query {
            hello: String
        }
        
        type Mutation {
            createUser(name: String!): User
        }
        
        type User {
            id: ID!
            name: String!
        }
        ";

        let result = service.load_schema_from_sdl(sdl);
        assert!(result.is_ok());
        assert!(service.has_schema());

        let schema = service.get_schema().unwrap();
        assert_eq!(schema.query_type, "Query");
    }

    #[test]
    fn build_schema_programmatically() {
        let mut service = SchemaService::new();

        // Create a simple User type
        let user_type = GraphQLType::Object(ObjectType {
            name: "User".to_string(),
            description: Some("A user in the system".to_string()),
            fields: {
                let mut fields = HashMap::new();
                fields.insert(
                    "id".to_string(),
                    FieldDefinition {
                        name: "id".to_string(),
                        description: Some("User ID".to_string()),
                        field_type: GraphQLType::NonNull(Box::new(GraphQLType::Scalar(
                            ScalarType::ID,
                        ))),
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
                fields
            },
            interfaces: vec![],
        });

        // Create Query type
        let query_type = GraphQLType::Object(ObjectType {
            name: "Query".to_string(),
            description: Some("Root Query".to_string()),
            fields: {
                let mut fields = HashMap::new();
                fields.insert(
                    "user".to_string(),
                    FieldDefinition {
                        name: "user".to_string(),
                        description: Some("Get a user".to_string()),
                        field_type: GraphQLType::Object(ObjectType {
                            name: "User".to_string(),
                            description: None,
                            fields: HashMap::new(),
                            interfaces: vec![],
                        }),
                        arguments: HashMap::new(),
                        deprecation_reason: None,
                    },
                );
                fields
            },
            interfaces: vec![],
        });

        let result = service
            .build_schema()
            .query_type("Query")
            .add_type(user_type)
            .unwrap()
            .add_type(query_type)
            .unwrap()
            .build();

        if let Err(ref e) = result {
            eprintln!("Schema build failed with error: {e:?}");
        }
        assert!(result.is_ok());
        assert!(service.has_schema());

        let schema = service.get_schema().unwrap();
        assert!(schema.get_type("User").is_some());
    }

    #[test]
    fn get_schema_stats() {
        let mut service = SchemaService::new();

        let sdl = r"
        type Query {
            user: User
            posts: [Post]
        }
        
        type User {
            id: ID!
            name: String!
            posts: [Post]
        }
        
        type Post {
            id: ID!
            title: String!
            content: String
            author: User!
        }
        
        enum Status {
            ACTIVE
            INACTIVE
        }
        
        input UserInput {
            name: String!
            email: String
        }
        ";

        let _result = service.load_schema_from_sdl(sdl).unwrap();
        let stats = service.get_schema_stats().unwrap();

        // Should have built-in scalars + custom types
        assert!(stats.object_types >= 3); // Query, User, Post
        assert!(stats.enum_types >= 1); // Status
        assert!(stats.input_types >= 1); // UserInput
        assert!(stats.total_fields > 0);
    }

    #[test]
    fn validate_invalid_schema() {
        let invalid_sdl = r"
        type Query {
            user: NonExistentType
        }
        ";

        let result = SchemaService::validate_schema_sdl(invalid_sdl);
        // This should parse successfully but might have validation warnings
        assert!(result.is_ok());
    }

    #[test]
    fn introspect_schema() {
        let mut service = SchemaService::new();

        let sdl = r"
        type Query {
            hello: String
        }
        ";

        let _result = service.load_schema_from_sdl(sdl).unwrap();
        let introspection = service.introspect().unwrap();

        assert_eq!(introspection.schema_info.query_type, "Query");
        assert!(!introspection.types.is_empty());
        assert!(!introspection.directives.is_empty());
    }
}
