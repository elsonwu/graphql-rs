# DataLoader Pattern: Efficient Data Fetching Guide

## 🎯 Overview

The DataLoader pattern is a critical performance optimization for GraphQL servers that solves the infamous **N+1 query problem**. It provides automatic batching and caching of data fetching operations, dramatically improving query performance and reducing database load.

## 🔥 The N+1 Problem

### Problem Scenario

Consider this GraphQL query:
```graphql
query {
  posts {
    id
    title
    author {
      id
      name
    }
  }
}
```

**Without DataLoader:**
```
1. SELECT * FROM posts                    # 1 query
2. SELECT * FROM users WHERE id = 1       # +N queries (one per post)
3. SELECT * FROM users WHERE id = 2
4. SELECT * FROM users WHERE id = 3
... (N more queries for N posts)
```

**Total: 1 + N queries** 🚨

**With DataLoader:**
```
1. SELECT * FROM posts                    # 1 query  
2. SELECT * FROM users WHERE id IN (1,2,3,4,5)  # 1 batched query
```

**Total: 2 queries** ✅

### Visual Representation

```
┌─────────────────────────────────────────────────────────────────┐
│                    GraphQL Query Execution                      │
└─────────────────────────────────────────────────────────────────┘
                                │
                     ┌──────────▼──────────┐
                     │   Resolve posts     │
                     │   [post1, post2,    │
                     │    post3, post4]    │
                     └──────────┬──────────┘
                                │
                    ┌───────────▼───────────┐
                    │  For each post,       │
                    │  resolve author       │
                    └───────────┬───────────┘
                                │
        ┌───────────────────────▼───────────────────────┐
        │                                               │
┌───────▼──────┐ ┌───────▼──────┐ ┌───────▼──────┐ ┌───▼───┐
│   Load user  │ │   Load user  │ │   Load user  │ │  ...  │
│   ID: 1      │ │   ID: 2      │ │   ID: 3      │ │       │
└──────────────┘ └──────────────┘ └──────────────┘ └───────┘

WITHOUT DATALOADER: N separate database calls (N+1 problem)

        ┌───────────────────────▼───────────────────────┐
        │                                               │
┌───────▼──────┐ ┌───────▼──────┐ ┌───────▼──────┐ ┌───▼───┐
│ DataLoader   │ │ DataLoader   │ │ DataLoader   │ │ Data  │
│ queue: [1]   │ │ queue: [1,2] │ │queue:[1,2,3] │ │Loader │
└──────────────┘ └──────────────┘ └──────────────┘ └───┬───┘
                                                       │
                            ┌──────────────────────────▼─────────┐
                            │ Batch Load: load_users([1,2,3,4]) │
                            │ Single database query              │
                            └────────────────────────────────────┘

WITH DATALOADER: Single batched database call + caching
```

## 🏗️ DataLoader Architecture

### Core Components

```rust
┌─────────────────────────────────────────────────────────────────┐
│                        DataLoader                               │
├─────────────────────────────────────────────────────────────────┤
│  🔧 Core Components:                                            │
│                                                                 │
│  ┌─────────────────┐  ┌─────────────────┐  ┌─────────────────┐ │
│  │   BatchQueue    │  │     Cache       │  │   BatchLoader   │ │
│  │                 │  │                 │  │                 │ │
│  │  • Keys to load │  │  • Key -> Value │  │  • Batch func   │ │
│  │  • Pending reqs │  │  • TTL support  │  │  • Load logic   │ │
│  │  • Debouncing   │  │  • Invalidation │  │  • Error handle │ │
│  └─────────────────┘  └─────────────────┘  └─────────────────┘ │
├─────────────────────────────────────────────────────────────────┤
│  🚀 Features:                                                   │
│  • Automatic batching with configurable delay                  │
│  • In-memory caching with TTL                                  │
│  • Request deduplication                                       │
│  • Error handling and partial failures                        │
│  • Metrics and observability                                  │
│  • Async/await support                                        │
└─────────────────────────────────────────────────────────────────┘
```

### Data Flow

```
Request Timeline:
Time: 0ms    5ms    10ms   15ms   20ms
│        │      │      │      │      │
▼        ▼      ▼      ▼      ▼      ▼

load(1) ──┐
          │
load(2) ──┼──► Batch Queue
          │    [1, 2, 3, 4]
load(3) ──┤         │
          │         │ (batch delay: 10ms)
load(4) ──┘         │
                    ▼
               Execute Batch
               load_many([1,2,3,4])
                    │
                    ▼
               ┌─────────────┐
               │  Database   │
               │   Results   │
               │ [u1,u2,u3,u4] │
               └─────────────┘
                    │
                    ▼
               Cache & Return
               Results to waiters

Request Deduplication:
load(1) ──┐
          ├──► Single request, multiple waiters
load(1) ──┘
```

## 🎯 Implementation Strategy

### 1. Core DataLoader Struct

```rust
use std::collections::HashMap;
use std::hash::Hash;
use std::sync::Arc;
use tokio::sync::{Mutex, oneshot};
use futures::future::BoxFuture;

/// A DataLoader for efficient batching and caching of data fetching operations
pub struct DataLoader<K, V, E> 
where 
    K: Clone + Hash + Eq + Send + sync + 'static,
    V: Clone + Send + Sync + 'static,
    E: Send + Sync + 'static,
{
    /// The batch loading function
    batch_load_fn: Arc<dyn BatchLoadFn<K, V, E>>,
    
    /// In-memory cache
    cache: Arc<Mutex<HashMap<K, CacheEntry<V>>>>,
    
    /// Batch queue and scheduling
    batch_queue: Arc<Mutex<BatchQueue<K, V, E>>>,
    
    /// Configuration options
    config: DataLoaderConfig,
}

/// Configuration for DataLoader behavior
#[derive(Debug, Clone)]
pub struct DataLoaderConfig {
    /// Maximum batch size
    pub max_batch_size: usize,
    
    /// Batch delay in milliseconds
    pub batch_delay_ms: u64,
    
    /// Enable caching
    pub cache_enabled: bool,
    
    /// Cache TTL in seconds
    pub cache_ttl_seconds: Option<u64>,
}

/// Batch loading function trait
pub trait BatchLoadFn<K, V, E>: Send + Sync {
    fn load<'a>(&'a self, keys: Vec<K>) -> BoxFuture<'a, Result<HashMap<K, V>, E>>;
}
```

### 2. Batch Queue Management

```rust
/// Manages batching and queuing of load requests
struct BatchQueue<K, V, E> {
    /// Pending requests
    pending: HashMap<K, Vec<oneshot::Sender<Result<V, LoadError<E>>>>>,
    
    /// Timer for batch execution
    batch_timer: Option<tokio::time::Instant>,
    
    /// Whether a batch is currently being processed
    processing: bool,
}

/// Load operation errors
#[derive(Debug)]
pub enum LoadError<E> {
    /// Batch function returned an error
    BatchError(E),
    
    /// Key was not found in batch result
    KeyNotFound,
    
    /// Request was cancelled
    Cancelled,
    
    /// Timeout exceeded
    Timeout,
}
```

### 3. Cache Implementation

```rust
/// Cache entry with TTL support
#[derive(Debug, Clone)]
struct CacheEntry<V> {
    /// The cached value
    value: V,
    
    /// When the entry was created
    created_at: std::time::Instant,
    
    /// Optional TTL
    ttl: Option<std::time::Duration>,
}

impl<V> CacheEntry<V> {
    /// Check if the cache entry is expired
    fn is_expired(&self) -> bool {
        if let Some(ttl) = self.ttl {
            self.created_at.elapsed() > ttl
        } else {
            false
        }
    }
}
```

## 🔧 GraphQL Integration

### Field Resolver with DataLoader

```rust
/// User data loader for GraphQL resolvers
pub struct UserDataLoader {
    db_pool: DatabasePool,
}

impl BatchLoadFn<UserId, User, DatabaseError> for UserDataLoader {
    fn load<'a>(&'a self, user_ids: Vec<UserId>) -> BoxFuture<'a, Result<HashMap<UserId, User>, DatabaseError>> {
        Box::pin(async move {
            // Single database query with IN clause
            let users = sqlx::query_as!(
                User,
                "SELECT id, name, email FROM users WHERE id = ANY($1)",
                &user_ids[..]
            )
            .fetch_all(&self.db_pool)
            .await?;
            
            // Convert to HashMap keyed by user ID
            let user_map = users.into_iter()
                .map(|user| (user.id, user))
                .collect();
                
            Ok(user_map)
        })
    }
}

/// GraphQL context with DataLoaders
pub struct GraphQLContext {
    pub user_loader: DataLoader<UserId, User, DatabaseError>,
    pub post_loader: DataLoader<PostId, Post, DatabaseError>, 
    // ... other loaders
}

/// Field resolver using DataLoader
impl PostResolvers {
    async fn author(&self, ctx: &GraphQLContext, post: &Post) -> Result<User, FieldError> {
        // This will be automatically batched!
        let user = ctx.user_loader.load(post.author_id).await?;
        Ok(user)
    }
}
```

## 🚀 Advanced Features

### 1. Request Context Integration

```rust
/// DataLoader context for a single GraphQL request
pub struct DataLoaderContext {
    loaders: HashMap<String, Box<dyn DataLoaderTrait>>,
}

impl DataLoaderContext {
    pub fn new() -> Self {
        let mut loaders = HashMap::new();
        
        // Register all loaders
        loaders.insert("user".to_string(), Box::new(user_loader()));
        loaders.insert("post".to_string(), Box::new(post_loader()));
        loaders.insert("comment".to_string(), Box::new(comment_loader()));
        
        Self { loaders }
    }
    
    pub fn get_loader<K, V, E>(&self, name: &str) -> Option<&DataLoader<K, V, E>> {
        self.loaders.get(name)?.downcast_ref()
    }
}
```

### 2. Metrics and Observability  

```rust
/// DataLoader metrics for monitoring
#[derive(Debug, Default)]
pub struct DataLoaderMetrics {
    /// Total number of individual load requests
    pub total_loads: u64,
    
    /// Number of batch operations executed  
    pub total_batches: u64,
    
    /// Cache hit rate
    pub cache_hits: u64,
    pub cache_misses: u64,
    
    /// Average batch size
    pub avg_batch_size: f64,
    
    /// Error counts
    pub batch_errors: u64,
    pub timeout_errors: u64,
}

impl DataLoaderMetrics {
    pub fn cache_hit_rate(&self) -> f64 {
        let total = self.cache_hits + self.cache_misses;
        if total == 0 { 0.0 } else { self.cache_hits as f64 / total as f64 }
    }
    
    pub fn batch_efficiency(&self) -> f64 {
        if self.total_batches == 0 { 0.0 } 
        else { self.total_loads as f64 / self.total_batches as f64 }
    }
}
```

### 3. Error Handling Strategies

```rust
/// Error handling configuration
#[derive(Debug, Clone)]
pub enum ErrorStrategy {
    /// Fail entire batch if any key fails
    FailFast,
    
    /// Return partial results, individual key errors  
    Partial,
    
    /// Retry failed keys with exponential backoff
    RetryWithBackoff { max_retries: u32, base_delay_ms: u64 },
}

/// Partial batch result
pub struct BatchResult<K, V, E> {
    /// Successfully loaded values
    pub values: HashMap<K, V>,
    
    /// Individual key errors
    pub errors: HashMap<K, E>,
}
```

## 📊 Performance Characteristics

### Benchmarks

```
Scenario: Loading 1000 users across 10 concurrent GraphQL requests

Without DataLoader:
├── Database queries: 10,000 (N+1 problem)
├── Query time: 2.5 seconds  
├── Database connections: 100
└── Memory usage: 50MB

With DataLoader:
├── Database queries: 10 (batched)
├── Query time: 0.1 seconds (25x faster!)
├── Database connections: 10
└── Memory usage: 15MB

Cache Benefits (subsequent requests):
├── Cache hit rate: 95%
├── Response time: 5ms (500x faster!)
└── Database queries: 0
```

### Configuration Tuning

```rust
// High-throughput configuration
let config = DataLoaderConfig {
    max_batch_size: 1000,      // Large batches
    batch_delay_ms: 5,         // Low latency
    cache_enabled: true,
    cache_ttl_seconds: Some(300), // 5 minute TTL
};

// Memory-constrained configuration  
let config = DataLoaderConfig {
    max_batch_size: 100,       // Smaller batches
    batch_delay_ms: 10,        // Slight delay for batching
    cache_enabled: true,
    cache_ttl_seconds: Some(60), // 1 minute TTL
};
```

## 🛡️ Best Practices

### 1. Loader Design Patterns

```rust
// ✅ GOOD: Dedicated loader per entity type
pub struct UserLoader;
pub struct PostLoader; 
pub struct CommentLoader;

// ❌ BAD: Generic loader trying to do everything
pub struct GenericLoader;

// ✅ GOOD: Consistent key types
impl BatchLoadFn<UserId, User, DbError> for UserLoader { ... }

// ❌ BAD: Mixed key types
impl BatchLoadFn<String, serde_json::Value, Error> for MixedLoader { ... }
```

### 2. Cache Strategy

```rust
// User data - longer TTL (users don't change often)
let user_loader = DataLoader::new(UserLoader, DataLoaderConfig {
    cache_ttl_seconds: Some(3600), // 1 hour
    ..Default::default()
});

// Real-time data - shorter TTL
let notification_loader = DataLoader::new(NotificationLoader, DataLoaderConfig {
    cache_ttl_seconds: Some(30), // 30 seconds
    ..Default::default()
});

// Session data - no caching (sensitive/changing)
let session_loader = DataLoader::new(SessionLoader, DataLoaderConfig {
    cache_enabled: false,
    ..Default::default()
});
```

### 3. Error Resilience

```rust
impl BatchLoadFn<UserId, User, DatabaseError> for UserLoader {
    fn load<'a>(&'a self, user_ids: Vec<UserId>) -> BoxFuture<'a, Result<HashMap<UserId, User>, DatabaseError>> {
        Box::pin(async move {
            // Always handle partial failures gracefully
            match self.load_users_from_db(user_ids.clone()).await {
                Ok(users) => Ok(users),
                Err(db_error) => {
                    // Fallback: load individually for partial failure
                    tracing::warn!("Batch load failed, falling back to individual loads: {}", db_error);
                    self.load_users_individually(user_ids).await
                }
            }
        })
    }
}
```

## 🚀 Production Considerations

### 1. Monitoring & Alerting

```rust
// Key metrics to monitor
let metrics_to_track = vec![
    "dataloader.cache_hit_rate",      // Should be > 80%
    "dataloader.batch_efficiency",    // Should be > 5
    "dataloader.avg_response_time",   // Should be < 50ms
    "dataloader.error_rate",          // Should be < 1%
];

// Set up alerts
if metrics.cache_hit_rate() < 0.8 {
    alert!("DataLoader cache hit rate too low: {}", metrics.cache_hit_rate());
}

if metrics.batch_efficiency() < 2.0 {
    alert!("DataLoader batching not effective: {}", metrics.batch_efficiency());  
}
```

### 2. Resource Management

```rust
// Prevent memory leaks with size limits
let config = DataLoaderConfig {
    max_cache_entries: Some(10_000),
    max_batch_queue_size: Some(1_000),
    batch_timeout_ms: Some(100),
    ..Default::default()
};

// Regular cache cleanup
tokio::spawn(async move {
    let mut interval = tokio::time::interval(Duration::from_secs(60));
    loop {
        interval.tick().await;
        dataloader.cleanup_expired_cache().await;
    }
});
```

### 3. Testing Strategies

```rust
#[tokio::test]
async fn test_dataloader_batching() {
    let loader = create_test_user_loader().await;
    
    // Multiple concurrent loads should batch
    let futures = vec![
        loader.load(UserId(1)),
        loader.load(UserId(2)), 
        loader.load(UserId(3)),
    ];
    
    let start = Instant::now();
    let results = futures::future::join_all(futures).await;
    let duration = start.elapsed();
    
    // All should succeed
    assert!(results.iter().all(|r| r.is_ok()));
    
    // Should be fast (batched, not individual)
    assert!(duration < Duration::from_millis(100));
    
    // Verify only one database call was made
    assert_eq!(loader.get_metrics().total_batches, 1);
}

#[tokio::test] 
async fn test_dataloader_caching() {
    let loader = create_test_user_loader().await;
    
    // Load data
    let user1_first = loader.load(UserId(1)).await.unwrap();
    
    // Second load should hit cache
    let user1_second = loader.load(UserId(1)).await.unwrap();
    
    assert_eq!(user1_first.id, user1_second.id);
    assert_eq!(loader.get_metrics().cache_hits, 1);
}
```

---

## 🎯 Next Steps

1. **Core DataLoader Implementation** - Basic batching and caching
2. **GraphQL Context Integration** - Request-scoped loader instances  
3. **Advanced Features** - Metrics, monitoring, error strategies
4. **Performance Optimization** - Memory management, cleanup
5. **Production Deployment** - Monitoring, alerting, scaling

The DataLoader pattern is essential for production GraphQL servers, providing dramatic performance improvements and better resource utilization. When implemented correctly, it can reduce database load by 10-100x while improving response times significantly.
