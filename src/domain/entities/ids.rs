use serde::{Deserialize, Serialize};
use std::fmt;
use uuid::Uuid;

/// Unique identifier for a GraphQL schema
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct SchemaId(Uuid);

impl SchemaId {
    pub fn new() -> Self {
        Self(Uuid::new_v4())
    }
    
    pub fn from_uuid(uuid: Uuid) -> Self {
        Self(uuid)
    }
    
    pub fn as_uuid(&self) -> Uuid {
        self.0
    }
}

impl fmt::Display for SchemaId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl Default for SchemaId {
    fn default() -> Self {
        Self::new()
    }
}

/// Unique identifier for a GraphQL query
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct QueryId(Uuid);

impl QueryId {
    pub fn new() -> Self {
        Self(Uuid::new_v4())
    }
    
    pub fn from_uuid(uuid: Uuid) -> Self {
        Self(uuid)
    }
    
    pub fn as_uuid(&self) -> Uuid {
        self.0
    }
}

impl fmt::Display for QueryId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl Default for QueryId {
    fn default() -> Self {
        Self::new()
    }
}

/// Schema version identifier
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SchemaVersion(String);

impl SchemaVersion {
    pub fn new(version: impl Into<String>) -> Self {
        Self(version.into())
    }
    
    pub fn as_str(&self) -> &str {
        &self.0
    }
    
    pub fn to_string(&self) -> String {
        self.0.clone()
    }
}

impl fmt::Display for SchemaVersion {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_schema_id() {
        let id1 = SchemaId::new();
        let id2 = SchemaId::new();
        assert_ne!(id1, id2);
        
        let uuid = Uuid::new_v4();
        let id3 = SchemaId::from_uuid(uuid);
        assert_eq!(id3.as_uuid(), uuid);
    }

    #[test]
    fn test_query_id() {
        let id1 = QueryId::new();
        let id2 = QueryId::new();
        assert_ne!(id1, id2);
        
        let uuid = Uuid::new_v4();
        let id3 = QueryId::from_uuid(uuid);
        assert_eq!(id3.as_uuid(), uuid);
    }

    #[test]
    fn test_schema_version() {
        let version = SchemaVersion::new("1.0.0");
        assert_eq!(version.as_str(), "1.0.0");
        assert_eq!(version.to_string(), "1.0.0");
    }
}
