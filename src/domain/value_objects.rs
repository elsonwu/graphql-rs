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
    #[must_use]
    pub fn innermost_name(&self) -> &str {
        match self {
            TypeReference::Named(name) => name,
            TypeReference::List(inner) | TypeReference::NonNull(inner) => inner.innermost_name(),
        }
    }

    /// Check if this type reference is non-null
    #[must_use]
    pub fn is_non_null(&self) -> bool {
        matches!(self, TypeReference::NonNull(_))
    }

    /// Check if this type reference is a list
    #[must_use]
    pub fn is_list(&self) -> bool {
        match self {
            TypeReference::List(_) => true,
            TypeReference::NonNull(inner) => matches!(inner.as_ref(), TypeReference::List(_)),
            TypeReference::Named(_) => false,
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
    #[must_use]
    pub fn valid() -> Self {
        Self::Valid
    }

    /// Create an invalid result with a single error message
    #[must_use]
    pub fn invalid(error: String) -> Self {
        Self::Invalid(vec![GraphQLError::validation_error(error)])
    }

    /// Create an invalid result with a single GraphQLError
    #[must_use]
    pub fn invalid_with_error(error: GraphQLError) -> Self {
        Self::Invalid(vec![error])
    }

    /// Create an invalid result with multiple errors
    #[must_use]
    pub fn invalid_with_errors(errors: Vec<GraphQLError>) -> Self {
        Self::Invalid(errors)
    }

    /// Create a validation error for a specific field
    #[must_use]
    pub fn field_error(field_name: &str, message: String) -> Self {
        let error = GraphQLError::validation_error(message)
            .with_path(vec![PathSegment::Field(field_name.to_string())]);
        Self::Invalid(vec![error])
    }

    /// Create a validation error for a type
    #[must_use]
    pub fn type_error(type_name: &str, message: String) -> Self {
        let error = GraphQLError::validation_error(message)
            .with_extension("type", serde_json::Value::String(type_name.to_string()));
        Self::Invalid(vec![error])
    }

    /// Combine multiple validation results
    #[must_use]
    pub fn combine(results: Vec<ValidationResult>) -> Self {
        let mut all_errors = Vec::new();

        for result in results {
            match result {
                Self::Invalid(errors) => all_errors.extend(errors),
                Self::Valid => continue,
                Self::Pending => return Self::Pending, // If any is pending, result is pending
            }
        }

        if all_errors.is_empty() {
            Self::Valid
        } else {
            Self::Invalid(all_errors)
        }
    }

    /// Add an error to an existing result
    #[must_use]
    pub fn with_error(self, error: GraphQLError) -> Self {
        match self {
            Self::Valid => Self::Invalid(vec![error]),
            Self::Invalid(mut errors) => {
                errors.push(error);
                Self::Invalid(errors)
            },
            Self::Pending => Self::Pending, // Pending stays pending
        }
    }

    /// Check if the validation result is valid
    #[must_use]
    pub fn is_valid(&self) -> bool {
        matches!(self, Self::Valid)
    }

    /// Check if the validation result is invalid
    #[must_use]
    pub fn is_invalid(&self) -> bool {
        matches!(self, Self::Invalid(_))
    }

    /// Check if the validation result is pending
    #[must_use]
    pub fn is_pending(&self) -> bool {
        matches!(self, Self::Pending)
    }

    /// Get the errors if invalid
    #[must_use]
    pub fn errors(&self) -> Option<&Vec<GraphQLError>> {
        match self {
            Self::Invalid(errors) => Some(errors),
            _ => None,
        }
    }

    /// Convert to ExecutionResult if invalid
    #[must_use]
    pub fn to_execution_result(self) -> Option<ExecutionResult> {
        match self {
            Self::Invalid(errors) => Some(ExecutionResult::error(errors)),
            _ => None,
        }
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
    #[must_use]
    pub fn success(data: serde_json::Value) -> Self {
        Self {
            data: Some(data),
            errors: Vec::new(),
            extensions: None,
        }
    }

    /// Create an execution result with errors
    #[must_use]
    pub fn error(errors: Vec<GraphQLError>) -> Self {
        Self {
            data: None,
            errors,
            extensions: None,
        }
    }

    /// Create an execution result with a single error
    #[must_use]
    pub fn single_error(error: GraphQLError) -> Self {
        Self::error(vec![error])
    }

    /// Create a partial result with both data and errors
    #[must_use]
    pub fn partial(data: serde_json::Value, errors: Vec<GraphQLError>) -> Self {
        Self {
            data: Some(data),
            errors,
            extensions: None,
        }
    }

    /// Add an error to the result
    #[must_use]
    pub fn with_error(mut self, error: GraphQLError) -> Self {
        self.errors.push(error);
        self
    }

    /// Add multiple errors to the result
    #[must_use]
    pub fn with_errors(mut self, mut errors: Vec<GraphQLError>) -> Self {
        self.errors.append(&mut errors);
        self
    }

    /// Add an extension to the result
    #[must_use]
    pub fn with_extension(mut self, key: String, value: serde_json::Value) -> Self {
        if self.extensions.is_none() {
            self.extensions = Some(serde_json::Map::new());
        }
        self.extensions.as_mut().unwrap().insert(key, value);
        self
    }

    /// Add execution timing information
    #[must_use]
    pub fn with_timing(self, duration_ms: u64) -> Self {
        self.with_extension(
            "timing".to_string(),
            serde_json::json!({
                "duration_ms": duration_ms
            }),
        )
    }

    /// Add tracing information
    #[must_use]
    pub fn with_tracing(self, trace_id: &str) -> Self {
        self.with_extension(
            "tracing".to_string(),
            serde_json::json!({
                "trace_id": trace_id
            }),
        )
    }

    /// Check if the result has any errors
    #[must_use]
    pub fn has_errors(&self) -> bool {
        !self.errors.is_empty()
    }

    /// Check if the result is successful (no errors)
    #[must_use]
    pub fn is_success(&self) -> bool {
        self.errors.is_empty()
    }

    /// Get errors by type
    #[must_use]
    pub fn errors_by_code(&self, code: &str) -> Vec<&GraphQLError> {
        self.errors
            .iter()
            .filter(|error| error.error_code() == Some(code))
            .collect()
    }

    /// Check if result has client errors
    #[must_use]
    pub fn has_client_errors(&self) -> bool {
        self.errors.iter().any(|error| error.is_client_error())
    }

    /// Check if result has server errors
    #[must_use]
    pub fn has_server_errors(&self) -> bool {
        self.errors.iter().any(|error| error.is_server_error())
    }

    /// Create a sanitized version for production (mask server errors)
    #[must_use]
    pub fn sanitized(&self, mask_server_errors: bool) -> Self {
        if !mask_server_errors {
            return self.clone();
        }

        let sanitized_errors = self
            .errors
            .iter()
            .map(|error| GraphQLError {
                message: error.user_message(mask_server_errors),
                locations: error.locations.clone(),
                path: error.path.clone(),
                extensions: error.extensions.clone(),
            })
            .collect();

        Self {
            data: self.data.clone(),
            errors: sanitized_errors,
            extensions: self.extensions.clone(),
        }
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
    #[must_use]
    pub fn new(message: String) -> Self {
        Self {
            message,
            locations: Vec::new(),
            path: None,
            extensions: None,
        }
    }

    /// Create a validation error with error code
    #[must_use]
    pub fn validation_error(message: String) -> Self {
        Self::new(message).with_error_code("VALIDATION_ERROR")
    }

    /// Create a parse error
    #[must_use]
    pub fn parse_error(message: String) -> Self {
        Self::new(message).with_error_code("PARSE_ERROR")
    }

    /// Create a field not found error
    #[must_use]
    pub fn field_not_found(field_name: &str, type_name: &str) -> Self {
        Self::new(format!(
            "Field '{field_name}' not found on type '{type_name}'"
        ))
        .with_error_code("FIELD_NOT_FOUND")
    }

    /// Create a type not found error
    #[must_use]
    pub fn type_not_found(type_name: &str) -> Self {
        Self::new(format!("Type '{type_name}' not found")).with_error_code("TYPE_NOT_FOUND")
    }

    /// Create an execution error
    #[must_use]
    pub fn execution_error(message: String) -> Self {
        Self::new(message).with_error_code("EXECUTION_ERROR")
    }

    /// Create an authentication error
    #[must_use]
    pub fn auth_error(message: String) -> Self {
        Self::new(message).with_error_code("AUTH_ERROR")
    }

    /// Create an authorization error
    #[must_use]
    pub fn authorization_error(message: String) -> Self {
        Self::new(message).with_error_code("AUTHORIZATION_ERROR")
    }

    /// Create a rate limit error
    #[must_use]
    pub fn rate_limit_error() -> Self {
        Self::new("Rate limit exceeded".to_string()).with_error_code("RATE_LIMIT_EXCEEDED")
    }

    /// Add a source location to the error
    #[must_use]
    pub fn with_location(mut self, line: u32, column: u32) -> Self {
        self.locations.push(SourceLocation { line, column });
        self
    }

    /// Add multiple source locations to the error
    #[must_use]
    pub fn with_locations(mut self, locations: Vec<SourceLocation>) -> Self {
        self.locations.extend(locations);
        self
    }

    /// Add a path to the error
    #[must_use]
    pub fn with_path(mut self, path: Vec<PathSegment>) -> Self {
        self.path = Some(path);
        self
    }

    /// Add an extension to the error
    #[must_use]
    pub fn with_extension(mut self, key: &str, value: serde_json::Value) -> Self {
        if self.extensions.is_none() {
            self.extensions = Some(serde_json::Map::new());
        }
        self.extensions
            .as_mut()
            .unwrap()
            .insert(key.to_string(), value);
        self
    }

    /// Add an error code extension
    #[must_use]
    pub fn with_error_code(self, code: &str) -> Self {
        self.with_extension("code", serde_json::Value::String(code.to_string()))
    }

    /// Add a timestamp extension
    #[must_use]
    pub fn with_timestamp(self) -> Self {
        use std::time::{SystemTime, UNIX_EPOCH};
        let timestamp = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs();
        self.with_extension("timestamp", serde_json::Value::Number(timestamp.into()))
    }

    /// Add request ID for tracing
    #[must_use]
    pub fn with_request_id(self, request_id: &str) -> Self {
        self.with_extension(
            "requestId",
            serde_json::Value::String(request_id.to_string()),
        )
    }

    /// Get the error code from extensions
    #[must_use]
    pub fn error_code(&self) -> Option<&str> {
        self.extensions.as_ref()?.get("code")?.as_str()
    }

    /// Check if this is a client error (4xx equivalent)
    #[must_use]
    pub fn is_client_error(&self) -> bool {
        matches!(
            self.error_code(),
            Some(
                "VALIDATION_ERROR"
                    | "PARSE_ERROR"
                    | "FIELD_NOT_FOUND"
                    | "TYPE_NOT_FOUND"
                    | "AUTH_ERROR"
                    | "AUTHORIZATION_ERROR"
            )
        )
    }

    /// Check if this is a server error (5xx equivalent)
    #[must_use]
    pub fn is_server_error(&self) -> bool {
        matches!(
            self.error_code(),
            Some("EXECUTION_ERROR" | "INTERNAL_ERROR")
        )
    }

    /// Convert to a user-friendly message (mask sensitive server errors in production)
    #[must_use]
    pub fn user_message(&self, mask_server_errors: bool) -> String {
        if mask_server_errors && self.is_server_error() {
            "Internal server error occurred".to_string()
        } else {
            self.message.clone()
        }
    }
}

/// Error propagation utilities for GraphQL execution
pub struct ErrorPropagation;

impl ErrorPropagation {
    /// Propagate field error up the execution path
    /// If a non-nullable field errors, it causes the parent to become null
    pub fn propagate_field_error(
        error: GraphQLError,
        field_path: &[PathSegment],
        is_nullable: bool,
    ) -> (GraphQLError, bool) {
        let error_with_path = error.with_path(field_path.to_vec());

        // Non-nullable fields cause parent to become null (bubble up)
        let should_bubble = !is_nullable;

        (error_with_path, should_bubble)
    }

    /// Collect multiple field errors and determine if parent should be null
    pub fn collect_field_errors(errors: Vec<(GraphQLError, bool)>) -> (Vec<GraphQLError>, bool) {
        let should_bubble = errors.iter().any(|(_, bubble)| *bubble);
        let collected_errors = errors.into_iter().map(|(error, _)| error).collect();

        (collected_errors, should_bubble)
    }

    /// Create error for missing required field
    pub fn missing_required_field(field_name: &str, parent_path: &[PathSegment]) -> GraphQLError {
        let mut path = parent_path.to_vec();
        path.push(PathSegment::Field(field_name.to_string()));

        GraphQLError::validation_error(format!("Required field '{field_name}' is missing"))
            .with_path(path)
    }

    /// Create error for null value in non-nullable field
    pub fn null_in_non_nullable(field_name: &str, parent_path: &[PathSegment]) -> GraphQLError {
        let mut path = parent_path.to_vec();
        path.push(PathSegment::Field(field_name.to_string()));

        GraphQLError::validation_error(format!(
            "Cannot return null for non-nullable field '{field_name}'"
        ))
        .with_path(path)
    }
}

/// Error formatting utilities
pub struct ErrorFormatter;

impl ErrorFormatter {
    /// Format error for development (includes all details)
    pub fn format_development(error: &GraphQLError) -> serde_json::Value {
        serde_json::json!({
            "message": error.message,
            "locations": error.locations,
            "path": error.path,
            "extensions": error.extensions,
            "stackTrace": error.extensions
                .as_ref()
                .and_then(|ext| ext.get("stackTrace"))
        })
    }

    /// Format error for production (sanitized)
    pub fn format_production(error: &GraphQLError) -> serde_json::Value {
        serde_json::json!({
            "message": error.user_message(true),
            "locations": if error.is_client_error() { Some(&error.locations) } else { None },
            "path": error.path,
            "extensions": {
                "code": error.error_code(),
                "timestamp": error.extensions
                    .as_ref()
                    .and_then(|ext| ext.get("timestamp"))
            }
        })
    }

    /// Format multiple errors with summary
    pub fn format_error_summary(errors: &[GraphQLError]) -> serde_json::Value {
        let client_errors = errors.iter().filter(|e| e.is_client_error()).count();
        let server_errors = errors.iter().filter(|e| e.is_server_error()).count();

        serde_json::json!({
            "total": errors.len(),
            "client_errors": client_errors,
            "server_errors": server_errors,
            "by_code": Self::group_errors_by_code(errors)
        })
    }

    /// Group errors by error code
    fn group_errors_by_code(errors: &[GraphQLError]) -> serde_json::Map<String, serde_json::Value> {
        use std::collections::HashMap;

        let mut grouped: HashMap<String, usize> = HashMap::new();

        for error in errors {
            let code = error.error_code().unwrap_or("UNKNOWN").to_string();
            *grouped.entry(code).or_insert(0) += 1;
        }

        grouped
            .into_iter()
            .map(|(k, v)| (k, serde_json::Value::Number(v.into())))
            .collect()
    }
}

/// Result type for operations that can fail with GraphQL errors
pub type GraphQLResult<T> = Result<T, GraphQLError>;

/// Result type for operations that can return multiple GraphQL errors
pub type GraphQLMultiResult<T> = Result<T, Vec<GraphQLError>>;

// ============================================================================
// Subscription Support
// ============================================================================

use futures::Stream;
use std::pin::Pin;

/// Result of a GraphQL subscription execution, containing a stream of results
pub struct SubscriptionResult {
    /// Stream of execution results for the subscription
    pub stream: Option<Pin<Box<dyn Stream<Item = ExecutionResult> + Send>>>,
    /// Any initial errors that occurred during subscription setup
    pub errors: Vec<GraphQLError>,
}

impl std::fmt::Debug for SubscriptionResult {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("SubscriptionResult")
            .field("has_stream", &self.stream.is_some())
            .field("errors", &self.errors)
            .finish()
    }
}

impl SubscriptionResult {
    /// Create a successful subscription result with a stream
    pub fn with_stream(stream: Pin<Box<dyn Stream<Item = ExecutionResult> + Send>>) -> Self {
        Self {
            stream: Some(stream),
            errors: Vec::new(),
        }
    }

    /// Create a subscription result with errors (no stream)
    pub fn with_errors(errors: Vec<GraphQLError>) -> Self {
        Self {
            stream: None,
            errors,
        }
    }

    /// Create a subscription result with a single error
    pub fn with_error(error: GraphQLError) -> Self {
        Self::with_errors(vec![error])
    }

    /// Check if the subscription has errors
    pub fn has_errors(&self) -> bool {
        !self.errors.is_empty()
    }

    /// Check if the subscription has a valid stream
    pub fn has_stream(&self) -> bool {
        self.stream.is_some()
    }
}

// ================================================================================================
// DataLoader Pattern Implementation
// ================================================================================================

use async_trait::async_trait;
use std::collections::HashMap;
use std::hash::Hash;
use std::sync::Arc;
use std::time::{Duration, Instant};
use tokio::sync::{oneshot, Mutex, RwLock};

/// Configuration for DataLoader behavior
#[derive(Debug, Clone)]
pub struct DataLoaderConfig {
    /// Maximum batch size before forcing execution
    pub max_batch_size: usize,

    /// Batch delay in milliseconds to collect requests
    pub batch_delay_ms: u64,

    /// Enable in-memory caching
    pub cache_enabled: bool,

    /// Cache TTL in seconds (None = no expiration)
    pub cache_ttl_seconds: Option<u64>,

    /// Enable metrics collection
    pub enable_metrics: bool,
}

impl Default for DataLoaderConfig {
    fn default() -> Self {
        Self {
            max_batch_size: 100,
            batch_delay_ms: 10,
            cache_enabled: true,
            cache_ttl_seconds: Some(300), // 5 minutes default TTL
            enable_metrics: true,
        }
    }
}

/// Cache entry with optional TTL
#[derive(Debug, Clone)]
struct CacheEntry<V> {
    value: V,
    expires_at: Option<Instant>,
}

impl<V> CacheEntry<V> {
    fn new(value: V, ttl: Option<Duration>) -> Self {
        let expires_at = ttl.map(|ttl| Instant::now() + ttl);
        Self { value, expires_at }
    }

    fn is_expired(&self) -> bool {
        if let Some(expires_at) = self.expires_at {
            Instant::now() > expires_at
        } else {
            false
        }
    }
}

/// Batch loading function trait for DataLoader
#[async_trait]
pub trait BatchLoadFn<K, V, E>: Send + Sync {
    /// Load multiple values for the given keys
    /// Returns a HashMap mapping keys to their loaded values
    async fn load(&self, keys: Vec<K>) -> Result<HashMap<K, V>, E>;
}

/// Pending batch request
#[derive(Debug)]
struct BatchRequest<K, V, E> {
    key: K,
    sender: oneshot::Sender<Result<V, E>>,
}

/// Batch queue for collecting and managing pending requests
#[derive(Debug)]
struct BatchQueue<K, V, E> {
    pending: Vec<BatchRequest<K, V, E>>,
    timer: Option<tokio::task::JoinHandle<()>>,
}

impl<K, V, E> Default for BatchQueue<K, V, E> {
    fn default() -> Self {
        Self {
            pending: Vec::new(),
            timer: None,
        }
    }
}

/// DataLoader metrics for monitoring and performance analysis
#[derive(Debug, Default, Clone)]
pub struct DataLoaderMetrics {
    /// Total number of load requests
    pub total_requests: u64,

    /// Number of cache hits
    pub cache_hits: u64,

    /// Number of cache misses
    pub cache_misses: u64,

    /// Number of batch operations executed
    pub batches_executed: u64,

    /// Total keys loaded in batches
    pub total_keys_loaded: u64,

    /// Average batch size
    pub average_batch_size: f64,
}

impl DataLoaderMetrics {
    /// Calculate cache hit ratio
    pub fn cache_hit_ratio(&self) -> f64 {
        if self.total_requests == 0 {
            0.0
        } else {
            self.cache_hits as f64 / self.total_requests as f64
        }
    }

    /// Update average batch size
    pub fn update_average_batch_size(&mut self, batch_size: usize) {
        let total_batches = self.batches_executed as f64;
        self.average_batch_size =
            ((self.average_batch_size * (total_batches - 1.0)) + batch_size as f64) / total_batches;
    }
}

/// A DataLoader for efficient batching and caching of data fetching operations
///
/// Solves the N+1 query problem by:
/// 1. Batching multiple individual load requests into single batch operations
/// 2. Caching results to avoid redundant loads
/// 3. Request deduplication within the same execution context
#[derive(Clone)]
pub struct DataLoader<K, V, E>
where
    K: Clone + Hash + Eq + Send + Sync + 'static,
    V: Clone + Send + Sync + 'static,
    E: Clone + Send + Sync + 'static,
{
    /// The batch loading function
    batch_load_fn: Arc<dyn BatchLoadFn<K, V, E>>,

    /// In-memory cache for loaded values
    cache: Arc<RwLock<HashMap<K, CacheEntry<V>>>>,

    /// Batch queue for pending requests
    batch_queue: Arc<Mutex<BatchQueue<K, V, E>>>,

    /// Configuration options
    config: DataLoaderConfig,

    /// Performance metrics
    metrics: Arc<RwLock<DataLoaderMetrics>>,
}

impl<K, V, E> DataLoader<K, V, E>
where
    K: Clone + Hash + Eq + Send + Sync + 'static,
    V: Clone + Send + Sync + 'static,
    E: Clone + Send + Sync + 'static,
{
    /// Create a new DataLoader with the given batch loading function
    pub fn new(batch_load_fn: Arc<dyn BatchLoadFn<K, V, E>>) -> Self {
        Self::with_config(batch_load_fn, DataLoaderConfig::default())
    }

    /// Create a new DataLoader with custom configuration
    pub fn with_config(
        batch_load_fn: Arc<dyn BatchLoadFn<K, V, E>>,
        config: DataLoaderConfig,
    ) -> Self {
        Self {
            batch_load_fn,
            cache: Arc::new(RwLock::new(HashMap::new())),
            batch_queue: Arc::new(Mutex::new(BatchQueue::default())),
            config,
            metrics: Arc::new(RwLock::new(DataLoaderMetrics::default())),
        }
    }

    /// Load a single value by key
    ///
    /// If the key is cached and not expired, returns the cached value immediately.
    /// Otherwise, adds the key to the batch queue and waits for batch execution.
    pub async fn load(&self, key: K) -> Result<V, E> {
        // Update metrics
        if self.config.enable_metrics {
            let mut metrics = self.metrics.write().await;
            metrics.total_requests += 1;
        }

        // Check cache first if caching is enabled
        if self.config.cache_enabled {
            let cache = self.cache.read().await;
            if let Some(entry) = cache.get(&key) {
                if !entry.is_expired() {
                    if self.config.enable_metrics {
                        let mut metrics = self.metrics.write().await;
                        metrics.cache_hits += 1;
                    }
                    return Ok(entry.value.clone());
                }
            }
        }

        // Cache miss - add to batch queue
        if self.config.enable_metrics {
            let mut metrics = self.metrics.write().await;
            metrics.cache_misses += 1;
        }

        let (sender, receiver) = oneshot::channel();
        let batch_request = BatchRequest {
            key: key.clone(),
            sender,
        };

        // Add request to batch queue
        let mut queue = self.batch_queue.lock().await;
        queue.pending.push(batch_request);

        // Check if we should execute immediately
        let should_execute = queue.pending.len() >= self.config.max_batch_size;

        if should_execute {
            // Execute immediately if batch size reached
            let pending = std::mem::take(&mut queue.pending);
            if let Some(timer) = queue.timer.take() {
                timer.abort();
            }
            drop(queue); // Release lock before execution

            self.execute_batch(pending).await;
        } else if queue.timer.is_none() {
            // Start batch delay timer
            let batch_queue_clone = Arc::clone(&self.batch_queue);
            let batch_load_fn_clone = Arc::clone(&self.batch_load_fn);
            let cache_clone = Arc::clone(&self.cache);
            let config_clone = self.config.clone();
            let metrics_clone = Arc::clone(&self.metrics);

            let timer = tokio::spawn(async move {
                tokio::time::sleep(Duration::from_millis(config_clone.batch_delay_ms)).await;

                let mut queue = batch_queue_clone.lock().await;
                let pending = std::mem::take(&mut queue.pending);
                queue.timer = None;
                drop(queue);

                if !pending.is_empty() {
                    Self::execute_batch_static(
                        pending,
                        batch_load_fn_clone,
                        cache_clone,
                        config_clone,
                        metrics_clone,
                    )
                    .await;
                }
            });

            queue.timer = Some(timer);
            drop(queue);
        } else {
            drop(queue);
        }

        // Wait for result
        receiver.await.map_err(|_| {
            // This would need to be properly handled based on your error type E
            // For now, we'll need to implement a way to create a "receiver closed" error
            panic!("DataLoader batch execution failed - receiver closed")
        })?
    }

    /// Load multiple values by keys
    ///
    /// More efficient than calling load() multiple times as it can batch all requests together
    pub async fn load_many(&self, keys: Vec<K>) -> Result<HashMap<K, V>, E> {
        let mut results = HashMap::new();
        let mut uncached_keys = Vec::new();

        // Check cache for each key
        if self.config.cache_enabled {
            let cache = self.cache.read().await;
            for key in &keys {
                if let Some(entry) = cache.get(key) {
                    if !entry.is_expired() {
                        results.insert(key.clone(), entry.value.clone());
                        continue;
                    }
                }
                uncached_keys.push(key.clone());
            }
        } else {
            uncached_keys = keys;
        }

        // Load uncached keys if any
        if !uncached_keys.is_empty() {
            let batch_results = self.batch_load_fn.load(uncached_keys.clone()).await?;

            // Update cache and results
            if self.config.cache_enabled {
                let mut cache = self.cache.write().await;
                let ttl = self.config.cache_ttl_seconds.map(Duration::from_secs);

                for (key, value) in &batch_results {
                    cache.insert(key.clone(), CacheEntry::new(value.clone(), ttl));
                }
            }

            results.extend(batch_results);
        }

        Ok(results)
    }

    /// Clear the cache
    pub async fn clear_cache(&self) {
        if self.config.cache_enabled {
            let mut cache = self.cache.write().await;
            cache.clear();
        }
    }

    /// Clear a specific key from the cache
    pub async fn clear_key(&self, key: &K) {
        if self.config.cache_enabled {
            let mut cache = self.cache.write().await;
            cache.remove(key);
        }
    }

    /// Get current metrics
    pub async fn get_metrics(&self) -> DataLoaderMetrics {
        self.metrics.read().await.clone()
    }

    /// Execute a batch of pending requests
    async fn execute_batch(&self, pending: Vec<BatchRequest<K, V, E>>) {
        Self::execute_batch_static(
            pending,
            Arc::clone(&self.batch_load_fn),
            Arc::clone(&self.cache),
            self.config.clone(),
            Arc::clone(&self.metrics),
        )
        .await;
    }

    /// Static version of execute_batch for use in async closures
    async fn execute_batch_static(
        pending: Vec<BatchRequest<K, V, E>>,
        batch_load_fn: Arc<dyn BatchLoadFn<K, V, E>>,
        cache: Arc<RwLock<HashMap<K, CacheEntry<V>>>>,
        config: DataLoaderConfig,
        metrics: Arc<RwLock<DataLoaderMetrics>>,
    ) {
        if pending.is_empty() {
            return;
        }

        // Deduplicate keys while preserving order and mapping back to requests
        let mut unique_keys = Vec::new();
        let mut seen_keys = HashMap::new(); // Use HashMap instead of HashSet for better compatibility

        for request in &pending {
            if !seen_keys.contains_key(&request.key) {
                seen_keys.insert(request.key.clone(), ());
                unique_keys.push(request.key.clone());
            }
        }

        let batch_size = unique_keys.len(); // Use deduplicated size for metrics

        // Update metrics
        if config.enable_metrics {
            let mut metrics_guard = metrics.write().await;
            metrics_guard.batches_executed += 1;
            metrics_guard.total_keys_loaded += batch_size as u64;
            metrics_guard.update_average_batch_size(batch_size);
        }

        // Execute batch load with deduplicated keys
        match batch_load_fn.load(unique_keys).await {
            Ok(results) => {
                // Update cache if enabled
                if config.cache_enabled {
                    let mut cache_guard = cache.write().await;
                    let ttl = config.cache_ttl_seconds.map(Duration::from_secs);

                    for (key, value) in &results {
                        cache_guard.insert(key.clone(), CacheEntry::new(value.clone(), ttl));
                    }
                }

                // Send results to waiters (including duplicate requests)
                for request in pending {
                    let result = results.get(&request.key).cloned();
                    let _ = request.sender.send(result.ok_or_else(|| {
                        // This would need to be properly handled based on your error type E
                        panic!("Key not found in batch result")
                    }));
                }
            },
            Err(error) => {
                // Send error to all waiters
                for request in pending {
                    let _ = request.sender.send(Err(error.clone()));
                }
            },
        }
    }
}
