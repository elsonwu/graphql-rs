//! Repository traits for data access
//!
//! Repositories provide abstractions for data access and persistence.

use crate::domain::entities::{
    ids::{QueryId, SchemaId},
    query::Query,
    schema::Schema,
};
use async_trait::async_trait;
use thiserror::Error;

/// Errors that can occur during repository operations
#[derive(Error, Debug)]
pub enum RepositoryError {
    /// Schema with the specified ID was not found
    #[error("Schema not found: {id:?}")]
    SchemaNotFound {
        /// The ID of the schema that was not found
        id: SchemaId,
    },

    /// Query with the specified ID was not found
    #[error("Query not found: {id:?}")]
    QueryNotFound {
        /// The ID of the query that was not found
        id: QueryId,
    },

    /// Error occurred in the storage layer
    #[error("Storage error: {message}")]
    StorageError {
        /// Description of the storage error
        message: String,
    },

    /// Error occurred during data serialization or deserialization
    #[error("Serialization error: {message}")]
    SerializationError {
        /// Description of the serialization error
        message: String,
    },

    /// Error occurred connecting to the storage system
    #[error("Connection error: {message}")]
    ConnectionError {
        /// Description of the connection error
        message: String,
    },
}

/// Repository for managing GraphQL schemas
#[async_trait]
pub trait SchemaRepository: Send + Sync {
    /// Save a schema to the repository
    async fn save(&self, schema: Schema) -> Result<(), RepositoryError>;

    /// Find a schema by its ID
    async fn find_by_id(&self, id: SchemaId) -> Result<Option<Schema>, RepositoryError>;

    /// Find the latest version of a schema
    async fn find_latest(&self) -> Result<Option<Schema>, RepositoryError>;

    /// List all schemas
    async fn list_all(&self) -> Result<Vec<Schema>, RepositoryError>;

    /// Delete a schema by its ID
    async fn delete(&self, id: SchemaId) -> Result<(), RepositoryError>;
}

/// Repository for managing GraphQL queries (useful for caching and analytics)
#[async_trait]
pub trait QueryRepository: Send + Sync {
    /// Save a query to the repository
    async fn save(&self, query: Query) -> Result<(), RepositoryError>;

    /// Find a query by its ID
    async fn find_by_id(&self, id: QueryId) -> Result<Option<Query>, RepositoryError>;

    /// Find queries by their query string (for caching)
    async fn find_by_query_string(&self, query_string: &str)
        -> Result<Vec<Query>, RepositoryError>;

    /// List recent queries (for analytics)
    async fn list_recent(&self, limit: usize) -> Result<Vec<Query>, RepositoryError>;

    /// Delete a query by its ID
    async fn delete(&self, id: QueryId) -> Result<(), RepositoryError>;
}

/// In-memory implementation of SchemaRepository for development and testing
pub struct InMemorySchemaRepository {
    schemas: std::sync::Arc<tokio::sync::RwLock<std::collections::HashMap<SchemaId, Schema>>>,
}

impl InMemorySchemaRepository {
    /// Create a new in-memory schema repository
    pub fn new() -> Self {
        Self {
            schemas: std::sync::Arc::new(
                tokio::sync::RwLock::new(std::collections::HashMap::new()),
            ),
        }
    }
}

impl Default for InMemorySchemaRepository {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl SchemaRepository for InMemorySchemaRepository {
    async fn save(&self, schema: Schema) -> Result<(), RepositoryError> {
        let mut schemas = self.schemas.write().await;
        schemas.insert(schema.id.clone(), schema);
        Ok(())
    }

    async fn find_by_id(&self, id: SchemaId) -> Result<Option<Schema>, RepositoryError> {
        let schemas = self.schemas.read().await;
        Ok(schemas.get(&id).cloned())
    }

    async fn find_latest(&self) -> Result<Option<Schema>, RepositoryError> {
        let schemas = self.schemas.read().await;
        // For now, just return the first schema found
        // In a real implementation, this would be based on creation time or version
        Ok(schemas.values().next().cloned())
    }

    async fn list_all(&self) -> Result<Vec<Schema>, RepositoryError> {
        let schemas = self.schemas.read().await;
        Ok(schemas.values().cloned().collect())
    }

    async fn delete(&self, id: SchemaId) -> Result<(), RepositoryError> {
        let mut schemas = self.schemas.write().await;
        schemas.remove(&id);
        Ok(())
    }
}

/// In-memory implementation of QueryRepository for development and testing
pub struct InMemoryQueryRepository {
    queries: std::sync::Arc<tokio::sync::RwLock<std::collections::HashMap<QueryId, Query>>>,
}

impl InMemoryQueryRepository {
    /// Create a new in-memory query repository
    pub fn new() -> Self {
        Self {
            queries: std::sync::Arc::new(
                tokio::sync::RwLock::new(std::collections::HashMap::new()),
            ),
        }
    }
}

impl Default for InMemoryQueryRepository {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl QueryRepository for InMemoryQueryRepository {
    async fn save(&self, query: Query) -> Result<(), RepositoryError> {
        let mut queries = self.queries.write().await;
        queries.insert(query.id().clone(), query);
        Ok(())
    }

    async fn find_by_id(&self, id: QueryId) -> Result<Option<Query>, RepositoryError> {
        let queries = self.queries.read().await;
        Ok(queries.get(&id).cloned())
    }

    async fn find_by_query_string(
        &self,
        query_string: &str,
    ) -> Result<Vec<Query>, RepositoryError> {
        let queries = self.queries.read().await;
        let matching_queries = queries
            .values()
            .filter(|query| query.query_string() == query_string)
            .cloned()
            .collect();
        Ok(matching_queries)
    }

    async fn list_recent(&self, limit: usize) -> Result<Vec<Query>, RepositoryError> {
        let queries = self.queries.read().await;
        let recent_queries = queries.values().take(limit).cloned().collect();
        Ok(recent_queries)
    }

    async fn delete(&self, id: QueryId) -> Result<(), RepositoryError> {
        let mut queries = self.queries.write().await;
        queries.remove(&id);
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::entities::ids::SchemaVersion;

    #[tokio::test]
    async fn test_in_memory_schema_repository() {
        let repo = InMemorySchemaRepository::new();
        let schema_id = SchemaId::new();
        let schema_version = SchemaVersion::new("1.0");
        let schema =
            Schema::with_id_and_version(schema_id.clone(), schema_version, "Query".to_string());

        // Save schema
        repo.save(schema).await.unwrap();

        // Find schema
        let found_schema = repo.find_by_id(schema_id.clone()).await.unwrap();
        assert!(found_schema.is_some());

        // List all schemas
        let all_schemas = repo.list_all().await.unwrap();
        assert_eq!(all_schemas.len(), 1);

        // Delete schema
        repo.delete(schema_id.clone()).await.unwrap();
        let deleted_schema = repo.find_by_id(schema_id).await.unwrap();
        assert!(deleted_schema.is_none());
    }

    #[tokio::test]
    async fn test_in_memory_query_repository() {
        let repo = InMemoryQueryRepository::new();
        let query = Query::new("{ test }".to_string());
        let query_id = query.id().clone();
        let query_string = query.query_string().to_string();

        // Save query
        repo.save(query).await.unwrap();

        // Find query by ID
        let found_query = repo.find_by_id(query_id.clone()).await.unwrap();
        assert!(found_query.is_some());

        // Find query by string
        let matching_queries = repo.find_by_query_string(&query_string).await.unwrap();
        assert_eq!(matching_queries.len(), 1);

        // List recent queries
        let recent_queries = repo.list_recent(10).await.unwrap();
        assert_eq!(recent_queries.len(), 1);

        // Delete query
        repo.delete(query_id.clone()).await.unwrap();
        let deleted_query = repo.find_by_id(query_id).await.unwrap();
        assert!(deleted_query.is_none());
    }
}
