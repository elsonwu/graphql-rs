use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Represents a GraphQL type system
#[derive(Debug, Clone, PartialEq)]
pub enum GraphQLType {
    /// Scalar type
    Scalar(ScalarType),
    /// Object type
    Object(ObjectType),
    /// Interface type
    Interface(InterfaceType),
    /// Union type
    Union(UnionType),
    /// Enum type
    Enum(EnumType),
    /// Input object type
    InputObject(InputObjectType),
    /// List wrapper type
    List(Box<GraphQLType>),
    /// Non-null wrapper type
    NonNull(Box<GraphQLType>),
}

/// Built-in scalar types in GraphQL
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum ScalarType {
    /// Integer scalar type
    Int,
    /// Float scalar type  
    Float,
    /// String scalar type
    String,
    /// Boolean scalar type
    Boolean,
    /// ID scalar type
    ID,
    /// Custom scalar type
    Custom(String),
}

/// GraphQL Object type definition
#[derive(Debug, Clone, PartialEq)]
pub struct ObjectType {
    /// The object type name
    pub name: String,
    /// Optional description of the type
    pub description: Option<String>,
    /// Map of field name to field definitions
    pub fields: HashMap<String, FieldDefinition>,
    /// List of interfaces this type implements
    pub interfaces: Vec<String>,
}

/// GraphQL Interface type definition
#[derive(Debug, Clone, PartialEq)]
pub struct InterfaceType {
    /// The interface type name
    pub name: String,
    /// Optional description of the type
    pub description: Option<String>,
    /// Map of field name to field definitions
    pub fields: HashMap<String, FieldDefinition>,
}

/// GraphQL Union type definition
#[derive(Debug, Clone, PartialEq)]
pub struct UnionType {
    /// The union type name
    pub name: String,
    /// Optional description of the type
    pub description: Option<String>,
    /// List of types that are members of this union
    pub types: Vec<String>,
}

/// GraphQL Enum type definition
#[derive(Debug, Clone, PartialEq)]
pub struct EnumType {
    /// The enum type name
    pub name: String,
    /// Optional description of the type
    pub description: Option<String>,
    /// Map of enum value name to value definition
    pub values: HashMap<String, EnumValue>,
}

/// GraphQL Enum value definition
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct EnumValue {
    /// Optional description of the enum value
    pub description: Option<String>,
    /// Optional deprecation reason
    pub deprecation_reason: Option<String>,
}

/// GraphQL Input Object type definition
#[derive(Debug, Clone, PartialEq)]
pub struct InputObjectType {
    /// The input object type name
    pub name: String,
    /// Optional description of the type
    pub description: Option<String>,
    /// Map of field name to input field definitions
    pub fields: HashMap<String, InputFieldDefinition>,
}

/// Field definition in a GraphQL object or interface
#[derive(Debug, Clone, PartialEq)]
pub struct FieldDefinition {
    /// The field name
    pub name: String,
    /// Optional description of the field
    pub description: Option<String>,
    /// The type of the field
    pub field_type: GraphQLType,
    /// Map of argument name to argument definitions
    pub arguments: HashMap<String, InputFieldDefinition>,
    /// Optional deprecation reason
    pub deprecation_reason: Option<String>,
}

/// Input field definition used in arguments and input objects
#[derive(Debug, Clone, PartialEq)]
pub struct InputFieldDefinition {
    /// The field name
    pub name: String,
    /// Optional description of the field
    pub description: Option<String>,
    /// The type of the field
    pub field_type: GraphQLType,
    /// Optional default value
    pub default_value: Option<Value>,
}

/// GraphQL directive definition
#[derive(Debug, Clone, PartialEq)]
pub struct DirectiveDefinition {
    /// The directive name
    pub name: String,
    /// Optional description of the directive
    pub description: Option<String>,
    /// Valid locations where this directive can be applied
    pub locations: Vec<DirectiveLocation>,
    /// Map of argument name to argument definitions
    pub arguments: HashMap<String, InputFieldDefinition>,
    /// Whether the directive is repeatable
    pub is_repeatable: bool,
}

/// Valid locations where a directive can be applied
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum DirectiveLocation {
    // Executable Directive Locations
    /// Query operation
    Query,
    /// Mutation operation
    Mutation,
    /// Subscription operation  
    Subscription,
    /// Field selection
    Field,
    /// Fragment definition
    FragmentDefinition,
    /// Fragment spread
    FragmentSpread,
    /// Inline fragment
    InlineFragment,
    /// Variable definition
    VariableDefinition,

    // Type System Directive Locations
    /// Schema definition
    Schema,
    /// Scalar type
    Scalar,
    /// Object type
    Object,
    /// Field definition
    FieldDefinition,
    /// Argument definition
    ArgumentDefinition,
    /// Interface type
    Interface,
    /// Union type
    Union,
    /// Enum type
    Enum,
    /// Enum value
    EnumValue,
    /// Input object type
    InputObject,
    /// Input field definition
    InputFieldDefinition,
}

/// GraphQL value types
#[derive(Debug, Clone, PartialEq)]
pub enum Value {
    /// Null value
    Null,
    /// Integer value
    Int(i64),
    /// Float value
    Float(f64),
    /// String value
    String(String),
    /// Boolean value
    Boolean(bool),
    /// Enum value
    Enum(String),
    /// List of values
    List(Vec<Value>),
    /// Object with string keys and values
    Object(HashMap<String, Value>),
    /// Variable reference
    Variable(String),
}

impl GraphQLType {
    /// Check if this type is nullable
    #[must_use]
    pub fn is_nullable(&self) -> bool {
        !matches!(self, GraphQLType::NonNull(_))
    }

    /// Get the innermost type, unwrapping `NonNull` and List wrappers
    #[must_use]
    pub fn inner_type(&self) -> &GraphQLType {
        match self {
            GraphQLType::List(inner) | GraphQLType::NonNull(inner) => inner.inner_type(),
            _ => self,
        }
    }

    /// Get the name of this type
    #[must_use]
    pub fn name(&self) -> Option<&str> {
        match self.inner_type() {
            GraphQLType::Scalar(scalar) => Some(scalar.name()),
            GraphQLType::Object(obj) => Some(&obj.name),
            GraphQLType::Interface(interface) => Some(&interface.name),
            GraphQLType::Union(union) => Some(&union.name),
            GraphQLType::Enum(enum_type) => Some(&enum_type.name),
            GraphQLType::InputObject(input) => Some(&input.name),
            _ => None,
        }
    }

    /// Check if this is a composite type (Object, Interface, Union)
    #[must_use]
    pub fn is_composite(&self) -> bool {
        matches!(
            self.inner_type(),
            GraphQLType::Object(_) | GraphQLType::Interface(_) | GraphQLType::Union(_)
        )
    }

    /// Check if this is a leaf type (Scalar, Enum)
    #[must_use]
    pub fn is_leaf(&self) -> bool {
        matches!(
            self.inner_type(),
            GraphQLType::Scalar(_) | GraphQLType::Enum(_)
        )
    }

    /// Check if this is an input type
    #[must_use]
    pub fn is_input_type(&self) -> bool {
        match self {
            GraphQLType::List(inner) | GraphQLType::NonNull(inner) => inner.is_input_type(),
            GraphQLType::Scalar(_) | GraphQLType::Enum(_) | GraphQLType::InputObject(_) => true,
            _ => false,
        }
    }

    /// Check if this is an output type
    #[must_use]
    pub fn is_output_type(&self) -> bool {
        match self {
            GraphQLType::List(inner) | GraphQLType::NonNull(inner) => inner.is_output_type(),
            GraphQLType::InputObject(_) => false,
            _ => true,
        }
    }
}

impl ScalarType {
    /// Get the name of this scalar type
    #[must_use]
    pub fn name(&self) -> &str {
        match self {
            ScalarType::Int => "Int",
            ScalarType::Float => "Float",
            ScalarType::String => "String",
            ScalarType::Boolean => "Boolean",
            ScalarType::ID => "ID",
            ScalarType::Custom(name) => name,
        }
    }

    /// Check if this is a built-in scalar type
    #[must_use]
    pub fn is_builtin(&self) -> bool {
        matches!(
            self,
            ScalarType::Int
                | ScalarType::Float
                | ScalarType::String
                | ScalarType::Boolean
                | ScalarType::ID
        )
    }
}

impl std::fmt::Display for GraphQLType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            GraphQLType::Scalar(scalar) => write!(f, "{}", scalar.name()),
            GraphQLType::Object(obj) => write!(f, "{}", obj.name),
            GraphQLType::Interface(interface) => write!(f, "{}", interface.name),
            GraphQLType::Union(union) => write!(f, "{}", union.name),
            GraphQLType::Enum(enum_type) => write!(f, "{}", enum_type.name),
            GraphQLType::InputObject(input) => write!(f, "{}", input.name),
            GraphQLType::List(inner) => write!(f, "[{inner}]"),
            GraphQLType::NonNull(inner) => write!(f, "{inner}!"),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn scalar_type_name() {
        assert_eq!(ScalarType::Int.name(), "Int");
        assert_eq!(ScalarType::String.name(), "String");
        assert_eq!(
            ScalarType::Custom("DateTime".to_string()).name(),
            "DateTime"
        );
    }

    #[test]
    fn scalar_type_is_builtin() {
        assert!(ScalarType::Int.is_builtin());
        assert!(ScalarType::String.is_builtin());
        assert!(!ScalarType::Custom("DateTime".to_string()).is_builtin());
    }

    #[test]
    fn graphql_type_nullable() {
        let nullable_int = GraphQLType::Scalar(ScalarType::Int);
        let non_null_int = GraphQLType::NonNull(Box::new(GraphQLType::Scalar(ScalarType::Int)));

        assert!(nullable_int.is_nullable());
        assert!(!non_null_int.is_nullable());
    }

    #[test]
    fn graphql_type_inner_type() {
        let base_type = GraphQLType::Scalar(ScalarType::String);
        let list_type = GraphQLType::List(Box::new(base_type.clone()));
        let non_null_list = GraphQLType::NonNull(Box::new(list_type));

        assert_eq!(non_null_list.inner_type(), &base_type);
    }

    #[test]
    fn graphql_type_is_leaf() {
        let scalar = GraphQLType::Scalar(ScalarType::Int);
        let enum_type = GraphQLType::Enum(EnumType {
            name: "Color".to_string(),
            description: None,
            values: HashMap::new(),
        });
        let object = GraphQLType::Object(ObjectType {
            name: "User".to_string(),
            description: None,
            fields: HashMap::new(),
            interfaces: vec![],
        });

        assert!(scalar.is_leaf());
        assert!(enum_type.is_leaf());
        assert!(!object.is_leaf());
    }

    #[test]
    fn graphql_type_is_input_output() {
        let scalar = GraphQLType::Scalar(ScalarType::String);
        let object = GraphQLType::Object(ObjectType {
            name: "User".to_string(),
            description: None,
            fields: HashMap::new(),
            interfaces: vec![],
        });
        let input_object = GraphQLType::InputObject(InputObjectType {
            name: "UserInput".to_string(),
            description: None,
            fields: HashMap::new(),
        });

        assert!(scalar.is_input_type());
        assert!(scalar.is_output_type());

        assert!(!object.is_input_type());
        assert!(object.is_output_type());

        assert!(input_object.is_input_type());
        assert!(!input_object.is_output_type());
    }

    #[test]
    fn graphql_type_display() {
        let scalar = GraphQLType::Scalar(ScalarType::String);
        let list = GraphQLType::List(Box::new(scalar.clone()));
        let non_null_list = GraphQLType::NonNull(Box::new(list));

        assert_eq!(format!("{scalar}"), "String");
        assert_eq!(format!("{non_null_list}"), "[String]!");
    }
}
