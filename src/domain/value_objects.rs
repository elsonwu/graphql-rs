//! Value objects representing GraphQL type system and execution results
//!
//! Value objects are immutable objects without identity, defined by their attributes.

use indexmap::IndexMap;
use serde::{Deserialize, Serialize};

/// Represents a GraphQL type definition
#[derive(Debug, Clone, PartialEq)]
pub enum TypeDefinition {
    /// Scalar type (Int, Float, String, Boolean, ID, or custom scalars)
    Scalar(ScalarTypeDefinition),
    /// Object type with fields
    Object(ObjectTypeDefinition),
    /// Interface type defining common fields
    Interface(InterfaceTypeDefinition),
    /// Union type representing one of several object types
    Union(UnionTypeDefinition),
    /// Enum type with finite set of values
    Enum(EnumTypeDefinition),
    /// Input object type for mutations and field arguments
    InputObject(InputObjectTypeDefinition),
}

/// Scalar type definition
#[derive(Debug, Clone, PartialEq)]
pub struct ScalarTypeDefinition {
    /// Name of the scalar type
    pub name: String,
    /// Optional description of the scalar type
    pub description: Option<String>,
    /// Whether this is a built-in GraphQL scalar
    pub is_builtin: bool,
}

/// Object type definition
#[derive(Debug, Clone, PartialEq)]
pub struct ObjectTypeDefinition {
    /// Name of the object type
    pub name: String,
    /// Optional description of the object type
    pub description: Option<String>,
    /// Fields defined on this object type
    pub fields: IndexMap<String, FieldDefinition>,
    /// Interfaces implemented by this object type
    pub interfaces: Vec<String>,
}

/// Interface type definition
#[derive(Debug, Clone, PartialEq)]
pub struct InterfaceTypeDefinition {
    /// Name of the interface type
    pub name: String,
    /// Optional description of the interface type
    pub description: Option<String>,
    /// Fields defined on this interface type
    pub fields: IndexMap<String, FieldDefinition>,
}

/// Union type definition
#[derive(Debug, Clone, PartialEq)]
pub struct UnionTypeDefinition {
    /// Name of the union type
    pub name: String,
    /// Optional description of the union type
    pub description: Option<String>,
    /// Object types that are part of this union
    pub types: Vec<String>,
}

/// Enum type definition
#[derive(Debug, Clone, PartialEq)]
pub struct EnumTypeDefinition {
    /// Name of the enum type
    pub name: String,
    /// Optional description of the enum type
    pub description: Option<String>,
    /// Values defined in this enum
    pub values: IndexMap<String, EnumValueDefinition>,
}

/// Input object type definition
#[derive(Debug, Clone, PartialEq)]
pub struct InputObjectTypeDefinition {
    /// Name of the input object type
    pub name: String,
    /// Optional description of the input object type
    pub description: Option<String>,
    /// Fields defined in this input object
    pub fields: IndexMap<String, InputValueDefinition>,
}

/// Field definition within object or interface types
#[derive(Debug, Clone, PartialEq)]
pub struct FieldDefinition {
    /// Name of the field
    pub name: String,
    /// Optional description of the field
    pub description: Option<String>,
    /// Type of the field
    pub type_reference: TypeReference,
    /// Arguments accepted by the field
    pub arguments: IndexMap<String, InputValueDefinition>,
}

/// Enum value definition
#[derive(Debug, Clone, PartialEq)]
pub struct EnumValueDefinition {
    /// Name of the enum value
    pub name: String,
    /// Optional description of the enum value
    pub description: Option<String>,
    /// Optional deprecation reason
    pub deprecation_reason: Option<String>,
}

/// Input value definition for arguments and input object fields
#[derive(Debug, Clone, PartialEq)]
pub struct InputValueDefinition {
    /// Name of the input value
    pub name: String,
    /// Optional description of the input value
    pub description: Option<String>,
    /// Type of the input value
    pub type_reference: TypeReference,
    /// Default value if not provided
    pub default_value: Option<serde_json::Value>,
}

/// Type reference with list and non-null modifiers
#[derive(Debug, Clone, PartialEq)]
pub enum TypeReference {
    /// Named type reference
    Named(String),
    /// List type: [Type]
    List(Box<TypeReference>),
    /// Non-null type: Type!
    NonNull(Box<TypeReference>),
}

impl TypeReference {
    /// Get the innermost named type
    pub fn innermost_name(&self) -> &str {
        match self {
            TypeReference::Named(name) => name,
            TypeReference::List(inner) | TypeReference::NonNull(inner) => inner.innermost_name(),
        }
    }

    /// Check if this type reference is non-null
    pub fn is_non_null(&self) -> bool {
        matches!(self, TypeReference::NonNull(_))
    }

    /// Check if this type reference is a list
    pub fn is_list(&self) -> bool {
        match self {
            TypeReference::List(_) => true,
            TypeReference::NonNull(inner) => matches!(inner.as_ref(), TypeReference::List(_)),
            _ => false,
        }
    }
}

/// Result of validating a GraphQL schema or query
#[derive(Debug, Clone, PartialEq)]
pub enum ValidationResult {
    /// Validation is pending
    Pending,
    /// Validation passed
    Valid,
    /// Validation failed with errors
    Invalid(Vec<GraphQLError>),
}

impl ValidationResult {
    /// Create a valid result
    pub fn valid() -> Self {
        Self::Valid
    }

    /// Create an invalid result with a single error
    pub fn invalid(error: String) -> Self {
        Self::Invalid(vec![GraphQLError::new(error)])
    }

    /// Create an invalid result with multiple errors
    pub fn invalid_with_errors(errors: Vec<GraphQLError>) -> Self {
        Self::Invalid(errors)
    }

    /// Check if the validation result is valid
    pub fn is_valid(&self) -> bool {
        matches!(self, Self::Valid)
    }

    /// Check if the validation result is invalid
    pub fn is_invalid(&self) -> bool {
        matches!(self, Self::Invalid(_))
    }
}

/// Result of GraphQL query execution
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExecutionResult {
    /// The data returned by the query
    pub data: Option<serde_json::Value>,

    /// Any errors that occurred during execution
    pub errors: Vec<GraphQLError>,

    /// Extensions to the result (for debugging, metrics, etc.)
    pub extensions: Option<serde_json::Map<String, serde_json::Value>>,
}

impl ExecutionResult {
    /// Create a successful execution result with data
    pub fn success(data: serde_json::Value) -> Self {
        Self {
            data: Some(data),
            errors: Vec::new(),
            extensions: None,
        }
    }

    /// Create an execution result with errors
    pub fn error(errors: Vec<GraphQLError>) -> Self {
        Self {
            data: None,
            errors,
            extensions: None,
        }
    }

    /// Add an extension to the result
    pub fn with_extension(mut self, key: String, value: serde_json::Value) -> Self {
        if self.extensions.is_none() {
            self.extensions = Some(serde_json::Map::new());
        }
        self.extensions.as_mut().unwrap().insert(key, value);
        self
    }
}

/// GraphQL error representation
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct GraphQLError {
    /// Error message
    pub message: String,

    /// Source locations where the error occurred
    pub locations: Vec<SourceLocation>,

    /// Path to the field that caused the error
    pub path: Option<Vec<PathSegment>>,

    /// Additional error information
    pub extensions: Option<serde_json::Map<String, serde_json::Value>>,
}

/// Source location in the GraphQL query
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct SourceLocation {
    /// Line number in the GraphQL query
    pub line: u32,
    /// Column number in the GraphQL query
    pub column: u32,
}

/// Path segment for error paths
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(untagged)]
pub enum PathSegment {
    /// Field name in the path
    Field(String),
    /// Array index in the path
    Index(u32),
}

impl GraphQLError {
    /// Create a new GraphQL error with a message
    pub fn new(message: String) -> Self {
        Self {
            message,
            locations: Vec::new(),
            path: None,
            extensions: None,
        }
    }

    /// Create a validation error
    pub fn validation_error(message: String) -> Self {
        Self::new(message)
    }

    /// Add a source location to the error
    pub fn with_location(mut self, line: u32, column: u32) -> Self {
        self.locations.push(SourceLocation { line, column });
        self
    }

    /// Add a path to the error
    pub fn with_path(mut self, path: Vec<PathSegment>) -> Self {
        self.path = Some(path);
        self
    }
}
