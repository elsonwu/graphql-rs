//! Data Transfer Objects (DTOs) for application layer
//!
//! DTOs define the structure of data exchanged between layers and external interfaces.

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Request DTO for GraphQL query execution
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GraphQLRequest {
    /// The GraphQL query string
    pub query: String,

    /// Optional operation name if the query contains multiple operations
    #[serde(rename = "operationName")]
    pub operation_name: Option<String>,

    /// Variables to be used in the query
    pub variables: Option<HashMap<String, serde_json::Value>>,
}

impl GraphQLRequest {
    /// Create a new GraphQL request with just a query string
    #[must_use]
    pub fn new(query: String) -> Self {
        Self {
            query,
            operation_name: None,
            variables: None,
        }
    }

    /// Add an operation name to the request
    #[must_use]
    pub fn with_operation_name(mut self, operation_name: String) -> Self {
        self.operation_name = Some(operation_name);
        self
    }

    /// Add variables to the request
    #[must_use]
    pub fn with_variables(mut self, variables: HashMap<String, serde_json::Value>) -> Self {
        self.variables = Some(variables);
        self
    }

    /// Get the variables, returning an empty `HashMap` if none are provided
    #[must_use]
    pub fn variables(&self) -> HashMap<String, serde_json::Value> {
        self.variables.clone().unwrap_or_default()
    }
}

/// Response DTO for GraphQL query execution
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GraphQLResponse {
    /// The data returned by the query
    pub data: Option<serde_json::Value>,

    /// Any errors that occurred during execution
    pub errors: Option<Vec<GraphQLErrorDto>>,

    /// Extensions to the response (for debugging, metrics, etc.)
    pub extensions: Option<serde_json::Map<String, serde_json::Value>>,
}

impl GraphQLResponse {
    /// Create a successful response with data
    #[must_use]
    pub fn success(data: serde_json::Value) -> Self {
        Self {
            data: Some(data),
            errors: None,
            extensions: None,
        }
    }

    /// Create an error response
    #[must_use]
    pub fn error(errors: Vec<GraphQLErrorDto>) -> Self {
        Self {
            data: None,
            errors: Some(errors),
            extensions: None,
        }
    }

    /// Add extensions to the response
    #[must_use]
    pub fn with_extensions(
        mut self,
        extensions: serde_json::Map<String, serde_json::Value>,
    ) -> Self {
        self.extensions = Some(extensions);
        self
    }
}

/// Error DTO for GraphQL errors
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GraphQLErrorDto {
    /// Error message
    pub message: String,

    /// Source locations where the error occurred
    pub locations: Option<Vec<SourceLocation>>,

    /// Path to the field that caused the error
    pub path: Option<Vec<PathSegment>>,

    /// Additional error information
    pub extensions: Option<serde_json::Map<String, serde_json::Value>>,
}

impl GraphQLErrorDto {
    /// Create a new GraphQL error with a message
    #[must_use]
    pub fn new(message: String) -> Self {
        Self {
            message,
            locations: None,
            path: None,
            extensions: None,
        }
    }
}

/// Source location information for errors
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct SourceLocation {
    /// Line number in the source
    pub line: u32,
    /// Column number in the source  
    pub column: u32,
}

/// Path segment in a GraphQL response path
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum PathSegment {
    /// Field name in the path
    Field(String),
    /// Array index in the path
    Index(u32),
}

/// Schema definition request DTO
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SchemaDefinitionRequest {
    /// The GraphQL schema in SDL format
    pub schema: String,

    /// Optional schema version
    pub version: Option<String>,

    /// Optional schema name
    pub name: Option<String>,
}

/// Schema definition response DTO
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SchemaDefinitionResponse {
    /// Whether the schema was successfully saved
    pub success: bool,

    /// Error message if the schema was invalid
    pub error: Option<String>,

    /// Schema ID if successfully saved
    pub schema_id: Option<String>,

    /// Schema version
    pub version: Option<String>,
}

/// Health check response DTO
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HealthCheckResponse {
    /// Service status
    pub status: String,

    /// Service version
    pub version: String,

    /// GraphQL spec version
    pub graphql_version: String,

    /// Current timestamp
    pub timestamp: chrono::DateTime<chrono::Utc>,
}

impl HealthCheckResponse {
    /// Create a healthy response
    #[must_use]
    pub fn healthy() -> Self {
        Self {
            status: "healthy".to_string(),
            version: crate::VERSION.to_string(),
            graphql_version: crate::GRAPHQL_SPEC_VERSION.to_string(),
            timestamp: chrono::Utc::now(),
        }
    }
}

/// Convert from domain `ExecutionResult` to DTO `GraphQLResponse`
impl From<crate::domain::value_objects::ExecutionResult> for GraphQLResponse {
    fn from(result: crate::domain::value_objects::ExecutionResult) -> Self {
        let errors = if result.errors.is_empty() {
            None
        } else {
            Some(result.errors.into_iter().map(Into::into).collect())
        };

        Self {
            data: result.data,
            errors,
            extensions: result.extensions,
        }
    }
}

/// Convert from domain `GraphQLError` to DTO
impl From<crate::domain::value_objects::GraphQLError> for GraphQLErrorDto {
    fn from(error: crate::domain::value_objects::GraphQLError) -> Self {
        let locations = if error.locations.is_empty() {
            None
        } else {
            Some(
                error
                    .locations
                    .into_iter()
                    .map(|loc| SourceLocation {
                        line: loc.line,
                        column: loc.column,
                    })
                    .collect(),
            )
        };

        let path = error.path.map(|path| {
            path.into_iter()
                .map(|segment| match segment {
                    crate::domain::value_objects::PathSegment::Field(field) => {
                        PathSegment::Field(field)
                    },
                    crate::domain::value_objects::PathSegment::Index(index) => {
                        PathSegment::Index(index)
                    },
                })
                .collect()
        });

        Self {
            message: error.message,
            locations,
            path,
            extensions: error.extensions,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_graphql_request_creation() {
        let request = GraphQLRequest::new("{ test }".to_string())
            .with_operation_name("TestQuery".to_string())
            .with_variables({
                let mut vars = HashMap::new();
                vars.insert("id".to_string(), serde_json::json!("123"));
                vars
            });

        assert_eq!(request.query, "{ test }");
        assert_eq!(request.operation_name, Some("TestQuery".to_string()));
        assert_eq!(
            request.variables().get("id"),
            Some(&serde_json::json!("123"))
        );
    }

    #[test]
    fn test_graphql_response_creation() {
        let response = GraphQLResponse::success(serde_json::json!({"test": "value"}));

        assert!(response.data.is_some());
        assert!(response.errors.is_none());
    }

    #[test]
    fn test_health_check_response() {
        let response = HealthCheckResponse::healthy();

        assert_eq!(response.status, "healthy");
        assert_eq!(response.version, crate::VERSION);
        assert_eq!(response.graphql_version, crate::GRAPHQL_SPEC_VERSION);
    }

    #[test]
    fn test_serialization() {
        let request = GraphQLRequest::new("{ test }".to_string());
        let serialized = serde_json::to_string(&request).unwrap();
        let deserialized: GraphQLRequest = serde_json::from_str(&serialized).unwrap();

        assert_eq!(request.query, deserialized.query);
    }
}
