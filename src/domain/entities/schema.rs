use crate::domain::entities::{
    ids::{SchemaId, SchemaVersion},
    types::*,
};
use std::collections::HashMap;
use thiserror::Error;

/// A complete GraphQL schema definition
#[derive(Debug, Clone)]
pub struct Schema {
    /// Unique identifier for this schema
    pub id: SchemaId,
    /// Version of this schema
    pub version: SchemaVersion,
    /// Query root type (required)
    pub query_type: String,
    /// Mutation root type (optional)
    pub mutation_type: Option<String>,
    /// Subscription root type (optional)
    pub subscription_type: Option<String>,
    /// All types defined in the schema
    pub types: HashMap<String, GraphQLType>,
    /// All directives defined in the schema
    pub directives: HashMap<String, DirectiveDefinition>,
    /// Schema description
    pub description: Option<String>,
}

/// Errors that can occur during schema operations
#[derive(Error, Debug, Clone, PartialEq)]
pub enum SchemaError {
    #[error("Type '{0}' not found in schema")]
    TypeNotFound(String),

    #[error("Query root type is required but not specified")]
    QueryTypeRequired,

    #[error("Root type '{0}' is not an Object type")]
    InvalidRootType(String),

    #[error("Circular reference detected in type '{0}'")]
    CircularReference(String),

    #[error("Interface '{interface}' not implemented correctly by type '{implementor}': {reason}")]
    InvalidInterfaceImplementation {
        interface: String,
        implementor: String,
        reason: String,
    },

    #[error("Union type '{union}' contains invalid member type '{member}': {reason}")]
    InvalidUnionMember {
        union: String,
        member: String,
        reason: String,
    },

    #[error("Field '{field}' in type '{type_name}' has invalid type: {reason}")]
    InvalidFieldType {
        type_name: String,
        field: String,
        reason: String,
    },

    #[error(
        "Argument '{argument}' in field '{field}' of type '{type_name}' has invalid type: {reason}"
    )]
    InvalidArgumentType {
        type_name: String,
        field: String,
        argument: String,
        reason: String,
    },

    #[error("Directive '{0}' not found in schema")]
    DirectiveNotFound(String),

    #[error("Duplicate type definition: '{0}'")]
    DuplicateType(String),

    #[error("Duplicate directive definition: '{0}'")]
    DuplicateDirective(String),
}

impl Schema {
    /// Create a new schema with the minimum required components
    pub fn new(query_type: String) -> Self {
        let mut schema = Self {
            id: SchemaId::new(),
            version: SchemaVersion::new("1.0.0"),
            query_type,
            mutation_type: None,
            subscription_type: None,
            types: HashMap::new(),
            directives: HashMap::new(),
            description: None,
        };

        // Add built-in scalar types
        schema.add_builtin_scalars();
        schema.add_builtin_directives();

        schema
    }

    /// Create a new schema with specific ID and version
    pub fn with_id_and_version(id: SchemaId, version: SchemaVersion, query_type: String) -> Self {
        let mut schema = Self {
            id,
            version,
            query_type,
            mutation_type: None,
            subscription_type: None,
            types: HashMap::new(),
            directives: HashMap::new(),
            description: None,
        };

        // Add built-in scalar types
        schema.add_builtin_scalars();
        schema.add_builtin_directives();

        schema
    }

    /// Add a type to the schema
    pub fn add_type(&mut self, type_def: GraphQLType) -> Result<(), SchemaError> {
        let type_name = type_def
            .name()
            .ok_or_else(|| SchemaError::InvalidFieldType {
                type_name: "Schema".to_string(),
                field: "type".to_string(),
                reason: "Type must have a name".to_string(),
            })?
            .to_string();

        if self.types.contains_key(&type_name) {
            return Err(SchemaError::DuplicateType(type_name));
        }

        self.types.insert(type_name, type_def);
        Ok(())
    }

    /// Add a directive to the schema
    pub fn add_directive(&mut self, directive: DirectiveDefinition) -> Result<(), SchemaError> {
        if self.directives.contains_key(&directive.name) {
            return Err(SchemaError::DuplicateDirective(directive.name));
        }

        let name = directive.name.clone();
        self.directives.insert(name, directive);
        Ok(())
    }

    /// Get a type by name
    pub fn get_type(&self, name: &str) -> Option<&GraphQLType> {
        self.types.get(name)
    }

    /// Get a directive by name
    pub fn get_directive(&self, name: &str) -> Option<&DirectiveDefinition> {
        self.directives.get(name)
    }

    /// Validate the schema for correctness
    pub fn validate(&self) -> Result<(), Vec<SchemaError>> {
        let mut errors = Vec::new();

        // Validate root types exist and are Object types
        if let Err(e) = self.validate_root_types() {
            errors.push(e);
        }

        // Validate all types are consistent
        for (name, type_def) in &self.types {
            if let Err(mut type_errors) = self.validate_type(name, type_def) {
                errors.append(&mut type_errors);
            }
        }

        // Check for circular references
        if let Err(mut circular_errors) = self.check_circular_references() {
            errors.append(&mut circular_errors);
        }

        if errors.is_empty() {
            Ok(())
        } else {
            Err(errors)
        }
    }

    /// Get the Query root type
    pub fn query_type(&self) -> Result<&GraphQLType, SchemaError> {
        self.get_type(&self.query_type)
            .ok_or_else(|| SchemaError::TypeNotFound(self.query_type.clone()))
    }

    /// Get the Mutation root type if it exists
    pub fn mutation_type(&self) -> Option<&GraphQLType> {
        self.mutation_type
            .as_ref()
            .and_then(|name| self.get_type(name))
    }

    /// Get the Subscription root type if it exists
    pub fn subscription_type(&self) -> Option<&GraphQLType> {
        self.subscription_type
            .as_ref()
            .and_then(|name| self.get_type(name))
    }

    /// Get all Object types that implement a given interface
    pub fn get_implementations(&self, interface_name: &str) -> Vec<&ObjectType> {
        self.types
            .values()
            .filter_map(|t| match t {
                GraphQLType::Object(obj)
                    if obj.interfaces.contains(&interface_name.to_string()) =>
                {
                    Some(obj)
                },
                _ => None,
            })
            .collect()
    }

    /// Get all types that are part of a union
    pub fn get_union_members(&self, union_name: &str) -> Option<Vec<&GraphQLType>> {
        if let Some(GraphQLType::Union(union_type)) = self.get_type(union_name) {
            let members: Vec<_> = union_type
                .types
                .iter()
                .filter_map(|type_name| self.get_type(type_name))
                .collect();
            Some(members)
        } else {
            None
        }
    }

    /// Add built-in scalar types
    fn add_builtin_scalars(&mut self) {
        let scalars = [
            ScalarType::Int,
            ScalarType::Float,
            ScalarType::String,
            ScalarType::Boolean,
            ScalarType::ID,
        ];

        for scalar in scalars {
            let type_def = GraphQLType::Scalar(scalar.clone());
            self.types.insert(scalar.name().to_string(), type_def);
        }
    }

    /// Add built-in directives
    fn add_builtin_directives(&mut self) {
        // @include directive
        let include_directive = DirectiveDefinition {
            name: "include".to_string(),
            description: Some("Directs the executor to include this field or fragment only when the `if` argument is true.".to_string()),
            locations: vec![
                DirectiveLocation::Field,
                DirectiveLocation::FragmentSpread,
                DirectiveLocation::InlineFragment,
            ],
            arguments: {
                let mut args = HashMap::new();
                args.insert("if".to_string(), InputFieldDefinition {
                    name: "if".to_string(),
                    description: Some("Included when true.".to_string()),
                    field_type: GraphQLType::NonNull(Box::new(GraphQLType::Scalar(ScalarType::Boolean))),
                    default_value: None,
                });
                args
            },
            is_repeatable: false,
        };

        // @skip directive
        let skip_directive = DirectiveDefinition {
            name: "skip".to_string(),
            description: Some("Directs the executor to skip this field or fragment when the `if` argument is true.".to_string()),
            locations: vec![
                DirectiveLocation::Field,
                DirectiveLocation::FragmentSpread,
                DirectiveLocation::InlineFragment,
            ],
            arguments: {
                let mut args = HashMap::new();
                args.insert("if".to_string(), InputFieldDefinition {
                    name: "if".to_string(),
                    description: Some("Skipped when true.".to_string()),
                    field_type: GraphQLType::NonNull(Box::new(GraphQLType::Scalar(ScalarType::Boolean))),
                    default_value: None,
                });
                args
            },
            is_repeatable: false,
        };

        // @deprecated directive
        let deprecated_directive = DirectiveDefinition {
            name: "deprecated".to_string(),
            description: Some(
                "Marks an element of a GraphQL schema as no longer supported.".to_string(),
            ),
            locations: vec![
                DirectiveLocation::FieldDefinition,
                DirectiveLocation::EnumValue,
            ],
            arguments: {
                let mut args = HashMap::new();
                args.insert(
                    "reason".to_string(),
                    InputFieldDefinition {
                        name: "reason".to_string(),
                        description: Some("Explains why this element was deprecated.".to_string()),
                        field_type: GraphQLType::Scalar(ScalarType::String),
                        default_value: Some(Value::String("No longer supported".to_string())),
                    },
                );
                args
            },
            is_repeatable: false,
        };

        self.directives
            .insert("include".to_string(), include_directive);
        self.directives.insert("skip".to_string(), skip_directive);
        self.directives
            .insert("deprecated".to_string(), deprecated_directive);
    }

    /// Validate that root types exist and are Object types
    fn validate_root_types(&self) -> Result<(), SchemaError> {
        // Validate Query root type
        match self.get_type(&self.query_type) {
            Some(GraphQLType::Object(_)) => {},
            Some(_) => return Err(SchemaError::InvalidRootType(self.query_type.clone())),
            None => return Err(SchemaError::TypeNotFound(self.query_type.clone())),
        }

        // Validate Mutation root type if specified
        if let Some(ref mutation_type_name) = self.mutation_type {
            match self.get_type(mutation_type_name) {
                Some(GraphQLType::Object(_)) => {},
                Some(_) => return Err(SchemaError::InvalidRootType(mutation_type_name.clone())),
                None => return Err(SchemaError::TypeNotFound(mutation_type_name.clone())),
            }
        }

        // Validate Subscription root type if specified
        if let Some(ref subscription_type_name) = self.subscription_type {
            match self.get_type(subscription_type_name) {
                Some(GraphQLType::Object(_)) => {},
                Some(_) => {
                    return Err(SchemaError::InvalidRootType(subscription_type_name.clone()))
                },
                None => return Err(SchemaError::TypeNotFound(subscription_type_name.clone())),
            }
        }

        Ok(())
    }

    /// Validate a single type definition
    fn validate_type(&self, name: &str, type_def: &GraphQLType) -> Result<(), Vec<SchemaError>> {
        let mut errors = Vec::new();

        match type_def {
            GraphQLType::Object(obj) => {
                if let Err(mut obj_errors) = self.validate_object_type(obj) {
                    errors.append(&mut obj_errors);
                }
            },
            GraphQLType::Interface(interface) => {
                if let Err(mut interface_errors) = self.validate_interface_type(interface) {
                    errors.append(&mut interface_errors);
                }
            },
            GraphQLType::Union(union) => {
                if let Err(mut union_errors) = self.validate_union_type(union) {
                    errors.append(&mut union_errors);
                }
            },
            GraphQLType::Enum(enum_type) => {
                if let Err(mut enum_errors) = self.validate_enum_type(enum_type) {
                    errors.append(&mut enum_errors);
                }
            },
            GraphQLType::InputObject(input) => {
                if let Err(mut input_errors) = self.validate_input_object_type(input) {
                    errors.append(&mut input_errors);
                }
            },
            _ => {}, // Scalar types are always valid
        }

        if errors.is_empty() {
            Ok(())
        } else {
            Err(errors)
        }
    }

    /// Validate an Object type
    fn validate_object_type(&self, obj: &ObjectType) -> Result<(), Vec<SchemaError>> {
        let mut errors = Vec::new();

        // Validate each field
        for (field_name, field) in &obj.fields {
            if !self.is_valid_output_type(&field.field_type) {
                errors.push(SchemaError::InvalidFieldType {
                    type_name: obj.name.clone(),
                    field: field_name.clone(),
                    reason: "Field type is not a valid output type".to_string(),
                });
            }

            // Validate field arguments
            for (arg_name, arg) in &field.arguments {
                if !self.is_valid_input_type(&arg.field_type) {
                    errors.push(SchemaError::InvalidArgumentType {
                        type_name: obj.name.clone(),
                        field: field_name.clone(),
                        argument: arg_name.clone(),
                        reason: "Argument type is not a valid input type".to_string(),
                    });
                }
            }
        }

        // Validate interface implementations
        for interface_name in &obj.interfaces {
            if let Some(GraphQLType::Interface(interface)) = self.get_type(interface_name) {
                if let Err(error) = self.validate_interface_implementation(obj, interface) {
                    errors.push(error);
                }
            } else {
                errors.push(SchemaError::TypeNotFound(interface_name.clone()));
            }
        }

        if errors.is_empty() {
            Ok(())
        } else {
            Err(errors)
        }
    }

    /// Validate an Interface type
    fn validate_interface_type(&self, interface: &InterfaceType) -> Result<(), Vec<SchemaError>> {
        let mut errors = Vec::new();

        // Validate each field
        for (field_name, field) in &interface.fields {
            if !self.is_valid_output_type(&field.field_type) {
                errors.push(SchemaError::InvalidFieldType {
                    type_name: interface.name.clone(),
                    field: field_name.clone(),
                    reason: "Field type is not a valid output type".to_string(),
                });
            }
        }

        if errors.is_empty() {
            Ok(())
        } else {
            Err(errors)
        }
    }

    /// Validate a Union type
    fn validate_union_type(&self, union: &UnionType) -> Result<(), Vec<SchemaError>> {
        let mut errors = Vec::new();

        for member_name in &union.types {
            match self.get_type(member_name) {
                Some(GraphQLType::Object(_)) => {}, // Valid
                Some(_) => {
                    errors.push(SchemaError::InvalidUnionMember {
                        union: union.name.clone(),
                        member: member_name.clone(),
                        reason: "Union members must be Object types".to_string(),
                    });
                },
                None => {
                    errors.push(SchemaError::TypeNotFound(member_name.clone()));
                },
            }
        }

        if errors.is_empty() {
            Ok(())
        } else {
            Err(errors)
        }
    }

    /// Validate an Enum type
    fn validate_enum_type(&self, _enum_type: &EnumType) -> Result<(), Vec<SchemaError>> {
        // Enum types are generally valid if they have at least one value
        // Additional validation could be added here
        Ok(())
    }

    /// Validate an Input Object type
    fn validate_input_object_type(&self, input: &InputObjectType) -> Result<(), Vec<SchemaError>> {
        let mut errors = Vec::new();

        // Validate each field
        for (field_name, field) in &input.fields {
            if !self.is_valid_input_type(&field.field_type) {
                errors.push(SchemaError::InvalidFieldType {
                    type_name: input.name.clone(),
                    field: field_name.clone(),
                    reason: "Field type is not a valid input type".to_string(),
                });
            }
        }

        if errors.is_empty() {
            Ok(())
        } else {
            Err(errors)
        }
    }

    /// Validate that an object correctly implements an interface
    fn validate_interface_implementation(
        &self,
        obj: &ObjectType,
        interface: &InterfaceType,
    ) -> Result<(), SchemaError> {
        for (field_name, interface_field) in &interface.fields {
            match obj.fields.get(field_name) {
                Some(obj_field) => {
                    // Check if field types are compatible
                    if !self.is_type_compatible(&obj_field.field_type, &interface_field.field_type)
                    {
                        return Err(SchemaError::InvalidInterfaceImplementation {
                            interface: interface.name.clone(),
                            implementor: obj.name.clone(),
                            reason: format!(
                                "Field '{}' type '{}' is not compatible with interface field type '{}'",
                                field_name, obj_field.field_type, interface_field.field_type
                            ),
                        });
                    }
                },
                None => {
                    return Err(SchemaError::InvalidInterfaceImplementation {
                        interface: interface.name.clone(),
                        implementor: obj.name.clone(),
                        reason: format!("Missing field '{}'", field_name),
                    });
                },
            }
        }

        Ok(())
    }

    /// Check for circular references in the type system
    fn check_circular_references(&self) -> Result<(), Vec<SchemaError>> {
        let mut errors = Vec::new();
        let mut visited = std::collections::HashSet::new();
        let mut visiting = std::collections::HashSet::new();

        for type_name in self.types.keys() {
            if !visited.contains(type_name) {
                if let Err(error) =
                    self.visit_type_for_cycles(type_name, &mut visited, &mut visiting)
                {
                    errors.push(error);
                }
            }
        }

        if errors.is_empty() {
            Ok(())
        } else {
            Err(errors)
        }
    }

    /// Helper method for circular reference detection
    fn visit_type_for_cycles(
        &self,
        type_name: &str,
        visited: &mut std::collections::HashSet<String>,
        visiting: &mut std::collections::HashSet<String>,
    ) -> Result<(), SchemaError> {
        if visiting.contains(type_name) {
            return Err(SchemaError::CircularReference(type_name.to_string()));
        }

        if visited.contains(type_name) {
            return Ok(());
        }

        visiting.insert(type_name.to_string());

        if let Some(type_def) = self.get_type(type_name) {
            // Visit referenced types based on the type definition
            // This is a simplified version - a full implementation would traverse all type references
            match type_def {
                GraphQLType::Object(obj) => {
                    for field in obj.fields.values() {
                        if let Some(referenced_type) = field.field_type.name() {
                            if referenced_type != type_name {
                                self.visit_type_for_cycles(referenced_type, visited, visiting)?;
                            }
                        }
                    }
                },
                _ => {}, // Other types handled similarly
            }
        }

        visiting.remove(type_name);
        visited.insert(type_name.to_string());

        Ok(())
    }

    /// Check if a type is a valid input type
    fn is_valid_input_type(&self, type_def: &GraphQLType) -> bool {
        type_def.is_input_type()
    }

    /// Check if a type is a valid output type
    fn is_valid_output_type(&self, type_def: &GraphQLType) -> bool {
        type_def.is_output_type()
    }

    /// Check if two types are compatible (for interface implementation)
    fn is_type_compatible(
        &self,
        implementor_type: &GraphQLType,
        interface_type: &GraphQLType,
    ) -> bool {
        // Simplified compatibility check - a full implementation would be more comprehensive
        format!("{}", implementor_type) == format!("{}", interface_type)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn create_schema_with_query_type() {
        let schema = Schema::new("Query".to_string());
        assert_eq!(schema.query_type, "Query");
        assert!(schema.mutation_type.is_none());
        assert!(schema.subscription_type.is_none());

        // Should have built-in scalars
        assert!(schema.get_type("String").is_some());
        assert!(schema.get_type("Int").is_some());
        assert!(schema.get_type("Boolean").is_some());

        // Should have built-in directives
        assert!(schema.get_directive("include").is_some());
        assert!(schema.get_directive("skip").is_some());
        assert!(schema.get_directive("deprecated").is_some());
    }

    #[test]
    fn add_type_to_schema() {
        let mut schema = Schema::new("Query".to_string());

        let user_type = GraphQLType::Object(ObjectType {
            name: "User".to_string(),
            description: Some("A user in the system".to_string()),
            fields: HashMap::new(),
            interfaces: vec![],
        });

        assert!(schema.add_type(user_type).is_ok());
        assert!(schema.get_type("User").is_some());
    }

    #[test]
    fn prevent_duplicate_type() {
        let mut schema = Schema::new("Query".to_string());

        let user_type1 = GraphQLType::Object(ObjectType {
            name: "User".to_string(),
            description: Some("First user type".to_string()),
            fields: HashMap::new(),
            interfaces: vec![],
        });

        let user_type2 = GraphQLType::Object(ObjectType {
            name: "User".to_string(),
            description: Some("Second user type".to_string()),
            fields: HashMap::new(),
            interfaces: vec![],
        });

        assert!(schema.add_type(user_type1).is_ok());
        assert!(matches!(
            schema.add_type(user_type2),
            Err(SchemaError::DuplicateType(_))
        ));
    }
}
