//! Domain events for GraphQL operations
//!
//! Events represent things that have happened in the domain and can trigger
//! side effects or cross-cutting concerns.

use crate::domain::entities::{SchemaId, QueryId};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::time::Duration;
use uuid::Uuid;

/// Unique identifier for domain events
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct EventId(pub Uuid);

impl EventId {
    /// Generate a new event ID
    pub fn new() -> Self {
        Self(Uuid::new_v4())
    }
}

impl Default for EventId {
    fn default() -> Self {
        Self::new()
    }
}

/// Base trait for all domain events
pub trait DomainEvent {
    /// Get the unique identifier for this event
    fn event_id(&self) -> &EventId;
    
    /// Get the timestamp when this event occurred
    fn timestamp(&self) -> DateTime<Utc>;
    
    /// Get the type name of this event
    fn event_type(&self) -> &'static str;
}

/// Events related to GraphQL queries
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum QueryEvent {
    /// A query was received and is about to be processed
    QueryReceived {
        event_id: EventId,
        timestamp: DateTime<Utc>,
        query_id: QueryId,
        query_string: String,
        operation_name: Option<String>,
    },
    
    /// A query passed validation
    QueryValidated {
        event_id: EventId,
        timestamp: DateTime<Utc>,
        query_id: QueryId,
    },
    
    /// A query failed validation
    QueryValidationFailed {
        event_id: EventId,
        timestamp: DateTime<Utc>,
        query_id: QueryId,
        errors: Vec<String>,
    },
    
    /// A query started execution
    QueryExecutionStarted {
        event_id: EventId,
        timestamp: DateTime<Utc>,
        query_id: QueryId,
        schema_id: SchemaId,
    },
    
    /// A query completed execution successfully
    QueryExecutionCompleted {
        event_id: EventId,
        timestamp: DateTime<Utc>,
        query_id: QueryId,
        execution_time: Duration,
        field_count: usize,
        result_size_bytes: usize,
    },
    
    /// A query execution failed with an error
    QueryExecutionFailed {
        event_id: EventId,
        timestamp: DateTime<Utc>,
        query_id: QueryId,
        execution_time: Duration,
        error: String,
    },
}

impl DomainEvent for QueryEvent {
    fn event_id(&self) -> &EventId {
        match self {
            QueryEvent::QueryReceived { event_id, .. }
            | QueryEvent::QueryValidated { event_id, .. }
            | QueryEvent::QueryValidationFailed { event_id, .. }
            | QueryEvent::QueryExecutionStarted { event_id, .. }
            | QueryEvent::QueryExecutionCompleted { event_id, .. }
            | QueryEvent::QueryExecutionFailed { event_id, .. } => event_id,
        }
    }
    
    fn timestamp(&self) -> DateTime<Utc> {
        match self {
            QueryEvent::QueryReceived { timestamp, .. }
            | QueryEvent::QueryValidated { timestamp, .. }
            | QueryEvent::QueryValidationFailed { timestamp, .. }
            | QueryEvent::QueryExecutionStarted { timestamp, .. }
            | QueryEvent::QueryExecutionCompleted { timestamp, .. }
            | QueryEvent::QueryExecutionFailed { timestamp, .. } => *timestamp,
        }
    }
    
    fn event_type(&self) -> &'static str {
        match self {
            QueryEvent::QueryReceived { .. } => "QueryReceived",
            QueryEvent::QueryValidated { .. } => "QueryValidated",
            QueryEvent::QueryValidationFailed { .. } => "QueryValidationFailed",
            QueryEvent::QueryExecutionStarted { .. } => "QueryExecutionStarted",
            QueryEvent::QueryExecutionCompleted { .. } => "QueryExecutionCompleted",
            QueryEvent::QueryExecutionFailed { .. } => "QueryExecutionFailed",
        }
    }
}

/// Events related to GraphQL schemas
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum SchemaEvent {
    /// A new schema was created
    SchemaCreated {
        event_id: EventId,
        timestamp: DateTime<Utc>,
        schema_id: SchemaId,
        version: String,
    },
    
    /// A schema was updated
    SchemaUpdated {
        event_id: EventId,
        timestamp: DateTime<Utc>,
        schema_id: SchemaId,
        old_version: String,
        new_version: String,
        changes_summary: String,
    },
    
    /// A schema was validated
    SchemaValidated {
        event_id: EventId,
        timestamp: DateTime<Utc>,
        schema_id: SchemaId,
    },
    
    /// A schema failed validation
    SchemaValidationFailed {
        event_id: EventId,
        timestamp: DateTime<Utc>,
        schema_id: SchemaId,
        errors: Vec<String>,
    },
    
    /// A schema was deleted
    SchemaDeleted {
        event_id: EventId,
        timestamp: DateTime<Utc>,
        schema_id: SchemaId,
    },
}

impl DomainEvent for SchemaEvent {
    fn event_id(&self) -> &EventId {
        match self {
            SchemaEvent::SchemaCreated { event_id, .. }
            | SchemaEvent::SchemaUpdated { event_id, .. }
            | SchemaEvent::SchemaValidated { event_id, .. }
            | SchemaEvent::SchemaValidationFailed { event_id, .. }
            | SchemaEvent::SchemaDeleted { event_id, .. } => event_id,
        }
    }
    
    fn timestamp(&self) -> DateTime<Utc> {
        match self {
            SchemaEvent::SchemaCreated { timestamp, .. }
            | SchemaEvent::SchemaUpdated { timestamp, .. }
            | SchemaEvent::SchemaValidated { timestamp, .. }
            | SchemaEvent::SchemaValidationFailed { timestamp, .. }
            | SchemaEvent::SchemaDeleted { timestamp, .. } => *timestamp,
        }
    }
    
    fn event_type(&self) -> &'static str {
        match self {
            SchemaEvent::SchemaCreated { .. } => "SchemaCreated",
            SchemaEvent::SchemaUpdated { .. } => "SchemaUpdated",
            SchemaEvent::SchemaValidated { .. } => "SchemaValidated",
            SchemaEvent::SchemaValidationFailed { .. } => "SchemaValidationFailed",
            SchemaEvent::SchemaDeleted { .. } => "SchemaDeleted",
        }
    }
}

/// All possible domain events in the GraphQL system
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum GraphQLEvent {
    Query(QueryEvent),
    Schema(SchemaEvent),
}

impl DomainEvent for GraphQLEvent {
    fn event_id(&self) -> &EventId {
        match self {
            GraphQLEvent::Query(event) => event.event_id(),
            GraphQLEvent::Schema(event) => event.event_id(),
        }
    }
    
    fn timestamp(&self) -> DateTime<Utc> {
        match self {
            GraphQLEvent::Query(event) => event.timestamp(),
            GraphQLEvent::Schema(event) => event.timestamp(),
        }
    }
    
    fn event_type(&self) -> &'static str {
        match self {
            GraphQLEvent::Query(event) => event.event_type(),
            GraphQLEvent::Schema(event) => event.event_type(),
        }
    }
}

/// Event publisher trait for publishing domain events
pub trait EventPublisher: Send + Sync {
    /// Publish a domain event
    fn publish(&self, event: GraphQLEvent);
}

/// Simple in-memory event publisher for development
pub struct InMemoryEventPublisher {
    events: std::sync::Arc<tokio::sync::RwLock<Vec<GraphQLEvent>>>,
}

impl InMemoryEventPublisher {
    /// Create a new in-memory event publisher
    pub fn new() -> Self {
        Self {
            events: std::sync::Arc::new(tokio::sync::RwLock::new(Vec::new())),
        }
    }
    
    /// Get all published events (for testing)
    pub async fn get_events(&self) -> Vec<GraphQLEvent> {
        let events = self.events.read().await;
        events.clone()
    }
    
    /// Clear all events (for testing)
    pub async fn clear_events(&self) {
        let mut events = self.events.write().await;
        events.clear();
    }
}

impl Default for InMemoryEventPublisher {
    fn default() -> Self {
        Self::new()
    }
}

impl EventPublisher for InMemoryEventPublisher {
    fn publish(&self, event: GraphQLEvent) {
        let events = self.events.clone();
        tokio::spawn(async move {
            let mut events = events.write().await;
            events.push(event);
        });
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::entities::{SchemaId, QueryId};
    
    #[test]
    fn test_query_event_creation() {
        let query_id = QueryId::new();
        let event = QueryEvent::QueryReceived {
            event_id: EventId::new(),
            timestamp: Utc::now(),
            query_id: query_id.clone(),
            query_string: "{ test }".to_string(),
            operation_name: None,
        };
        
        assert_eq!(event.event_type(), "QueryReceived");
        assert!(event.timestamp() <= Utc::now());
    }
    
    #[test]
    fn test_schema_event_creation() {
        let schema_id = SchemaId::new();
        let event = SchemaEvent::SchemaCreated {
            event_id: EventId::new(),
            timestamp: Utc::now(),
            schema_id: schema_id.clone(),
            version: "1.0".to_string(),
        };
        
        assert_eq!(event.event_type(), "SchemaCreated");
        assert!(event.timestamp() <= Utc::now());
    }
    
    #[tokio::test]
    async fn test_in_memory_event_publisher() {
        let publisher = InMemoryEventPublisher::new();
        let query_id = QueryId::new();
        
        let event = GraphQLEvent::Query(QueryEvent::QueryReceived {
            event_id: EventId::new(),
            timestamp: Utc::now(),
            query_id,
            query_string: "{ test }".to_string(),
            operation_name: None,
        });
        
        publisher.publish(event.clone());
        
        // Give some time for the async task to complete
        tokio::time::sleep(tokio::time::Duration::from_millis(10)).await;
        
        let events = publisher.get_events().await;
        assert_eq!(events.len(), 1);
    }
}
