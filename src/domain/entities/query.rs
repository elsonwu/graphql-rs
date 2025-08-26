use crate::domain::value_objects::{ValidationResult, GraphQLError};
use crate::domain::entities::ids::QueryId;
use std::time::{SystemTime, UNIX_EPOCH};

/// Represents a GraphQL query with its lifecycle state
#[derive(Debug, Clone)]
pub struct Query {
    id: QueryId,
    query_string: String,
    variables: Option<serde_json::Value>,
    operation_name: Option<String>,
    validation_result: Option<ValidationResult>,
    created_at: u64,
    updated_at: u64,
}

impl Query {
    /// Create a new Query with the given query string
    pub fn new(query_string: String) -> Self {
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs();
            
        Self {
            id: QueryId::new(),
            query_string,
            variables: None,
            operation_name: None,
            validation_result: None,
            created_at: now,
            updated_at: now,
        }
    }
    
    /// Create a new Query with all parameters
    pub fn new_with_params(
        query_string: String,
        variables: Option<serde_json::Value>,
        operation_name: Option<String>,
    ) -> Self {
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs();
            
        Self {
            id: QueryId::new(),
            query_string,
            variables,
            operation_name,
            validation_result: None,
            created_at: now,
            updated_at: now,
        }
    }
    
    /// Get the query ID
    pub fn id(&self) -> &QueryId {
        &self.id
    }
    
    /// Get the query string
    pub fn query_string(&self) -> &str {
        &self.query_string
    }
    
    /// Get the variables
    pub fn variables(&self) -> &Option<serde_json::Value> {
        &self.variables
    }
    
    /// Get the operation name
    pub fn operation_name(&self) -> &Option<String> {
        &self.operation_name
    }
    
    /// Get the validation result
    pub fn validation_result(&self) -> &Option<ValidationResult> {
        &self.validation_result
    }
    
    /// Get creation timestamp
    pub fn created_at(&self) -> u64 {
        self.created_at
    }
    
    /// Get update timestamp
    pub fn updated_at(&self) -> u64 {
        self.updated_at
    }
    
    /// Mark the query as validated with the given result
    pub fn mark_validated(&mut self, result: ValidationResult) {
        self.validation_result = Some(result);
        self.update_timestamp();
    }
    
    /// Check if the query is valid
    pub fn is_valid(&self) -> bool {
        matches!(self.validation_result, Some(ValidationResult::Valid))
    }
    
    /// Check if the query has validation errors
    pub fn has_validation_errors(&self) -> bool {
        matches!(self.validation_result, Some(ValidationResult::Invalid(_)))
    }
    
    /// Get validation errors if any
    pub fn validation_errors(&self) -> Vec<GraphQLError> {
        match &self.validation_result {
            Some(ValidationResult::Invalid(errors)) => errors.clone(),
            _ => Vec::new(),
        }
    }
    
    /// Update the query string
    pub fn update_query_string(&mut self, query_string: String) {
        self.query_string = query_string;
        self.validation_result = None; // Reset validation when query changes
        self.update_timestamp();
    }
    
    /// Update variables
    pub fn update_variables(&mut self, variables: Option<serde_json::Value>) {
        self.variables = variables;
        self.update_timestamp();
    }
    
    /// Update operation name
    pub fn update_operation_name(&mut self, operation_name: Option<String>) {
        self.operation_name = operation_name;
        self.update_timestamp();
    }
    
    /// Check if query string is empty
    pub fn is_empty(&self) -> bool {
        self.query_string.trim().is_empty()
    }
    
    /// Get query complexity (simplified metric based on string length for now)
    pub fn complexity(&self) -> usize {
        self.query_string.len()
    }
    
    /// Update the timestamp
    fn update_timestamp(&mut self) {
        self.updated_at = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs();
    }
}

impl PartialEq for Query {
    fn eq(&self, other: &Self) -> bool {
        self.id == other.id
    }
}

impl Eq for Query {}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::value_objects::GraphQLError;

    #[test]
    fn test_new_query() {
        let query = Query::new("{ hello }".to_string());
        
        assert_eq!(query.query_string(), "{ hello }");
        assert_eq!(query.variables(), &None);
        assert_eq!(query.operation_name(), &None);
        assert_eq!(query.validation_result(), &None);
        assert!(!query.is_valid());
        assert!(!query.has_validation_errors());
    }
    
    #[test]
    fn test_new_query_with_params() {
        let variables = Some(serde_json::json!({"name": "test"}));
        let operation_name = Some("TestOperation".to_string());
        
        let query = Query::new_with_params(
            "query TestOperation($name: String) { hello(name: $name) }".to_string(),
            variables.clone(),
            operation_name.clone(),
        );
        
        assert_eq!(query.variables(), &variables);
        assert_eq!(query.operation_name(), &operation_name);
    }
    
    #[test]
    fn test_mark_validated() {
        let mut query = Query::new("{ hello }".to_string());
        
        query.mark_validated(ValidationResult::Valid);
        assert!(query.is_valid());
        assert!(!query.has_validation_errors());
        
        let error = GraphQLError::validation_error("Test error".to_string());
        query.mark_validated(ValidationResult::Invalid(vec![error.clone()]));
        assert!(!query.is_valid());
        assert!(query.has_validation_errors());
        assert_eq!(query.validation_errors(), vec![error]);
    }
    
    #[test]
    fn test_update_query_string() {
        let mut query = Query::new("{ hello }".to_string());
        query.mark_validated(ValidationResult::Valid);
        
        query.update_query_string("{ world }".to_string());
        assert_eq!(query.query_string(), "{ world }");
        assert_eq!(query.validation_result(), &None); // Reset after update
    }
    
    #[test]
    fn test_is_empty() {
        let empty_query = Query::new("".to_string());
        let whitespace_query = Query::new("   \n\t  ".to_string());
        let valid_query = Query::new("{ hello }".to_string());
        
        assert!(empty_query.is_empty());
        assert!(whitespace_query.is_empty());
        assert!(!valid_query.is_empty());
    }
    
    #[test]
    fn test_complexity() {
        let simple_query = Query::new("{ hello }".to_string());
        let complex_query = Query::new("{ user { id name email posts { id title content } } }".to_string());
        
        assert!(complex_query.complexity() > simple_query.complexity());
    }
    
    #[test]
    fn test_equality() {
        let query1 = Query::new("{ hello }".to_string());
        let query2 = Query::new("{ hello }".to_string());
        
        assert_ne!(query1, query2); // Different IDs
        assert_eq!(query1, query1); // Same instance
    }
}
