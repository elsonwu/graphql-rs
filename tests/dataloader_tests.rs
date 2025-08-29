use graphql_rs::domain::value_objects::{
    DataLoader, DataLoaderConfig, DataLoaderMetrics, BatchLoadFn
};
use std::collections::HashMap;
use std::sync::{Arc, atomic::{AtomicUsize, Ordering}};
use std::time::Duration;
use async_trait::async_trait;

/// Mock batch loader for testing user data
#[derive(Debug)]
struct UserBatchLoader {
    /// Counter to track how many batch operations were executed
    pub batch_count: Arc<AtomicUsize>,
    /// Mock database of users
    pub users: HashMap<u32, String>,
}

impl UserBatchLoader {
    fn new() -> Self {
        let mut users = HashMap::new();
        users.insert(1, "Alice".to_string());
        users.insert(2, "Bob".to_string());
        users.insert(3, "Charlie".to_string());
        users.insert(4, "Diana".to_string());
        users.insert(5, "Eve".to_string());
        
        Self {
            batch_count: Arc::new(AtomicUsize::new(0)),
            users,
        }
    }
}

#[async_trait]
impl BatchLoadFn<u32, String, String> for UserBatchLoader {
    async fn load(&self, keys: Vec<u32>) -> Result<HashMap<u32, String>, String> {
        // Increment batch counter
        self.batch_count.fetch_add(1, Ordering::SeqCst);
        
        // Simulate database delay
        tokio::time::sleep(Duration::from_millis(10)).await;
        
        let mut results = HashMap::new();
        for key in keys {
            if let Some(user) = self.users.get(&key) {
                results.insert(key, user.clone());
            }
        }
        
        Ok(results)
    }
}

/// Mock batch loader that always fails for error testing
#[derive(Debug)]
struct FailingBatchLoader;

#[async_trait]
impl BatchLoadFn<u32, String, String> for FailingBatchLoader {
    async fn load(&self, _keys: Vec<u32>) -> Result<HashMap<u32, String>, String> {
        Err("Database connection failed".to_string())
    }
}

#[tokio::test]
async fn test_basic_dataloader_functionality() {
    let loader = UserBatchLoader::new();
    let batch_count = Arc::clone(&loader.batch_count);
    let dataloader = DataLoader::new(Arc::new(loader));
    
    // Load a single user
    let result = dataloader.load(1).await.unwrap();
    assert_eq!(result, "Alice");
    
    // Should have executed exactly one batch
    assert_eq!(batch_count.load(Ordering::SeqCst), 1);
}

#[tokio::test]
async fn test_dataloader_batching() {
    let loader = UserBatchLoader::new();
    let batch_count = Arc::clone(&loader.batch_count);
    
    // Configure with longer batch delay to ensure batching
    let config = DataLoaderConfig {
        max_batch_size: 100,
        batch_delay_ms: 50,
        cache_enabled: true,
        cache_ttl_seconds: Some(300),
        enable_metrics: true,
    };
    
    let dataloader = DataLoader::with_config(Arc::new(loader), config);
    
    // Start multiple loads concurrently
    let handles = vec![
        tokio::spawn({
            let dl = dataloader.clone();
            async move { dl.load(1).await }
        }),
        tokio::spawn({
            let dl = dataloader.clone();
            async move { dl.load(2).await }
        }),
        tokio::spawn({
            let dl = dataloader.clone();
            async move { dl.load(3).await }
        }),
    ];
    
    // Wait for all loads to complete
    let results: Result<Vec<_>, _> = futures::future::try_join_all(handles).await;
    let values: Result<Vec<_>, _> = results.unwrap().into_iter().collect();
    let user_names = values.unwrap();
    
    // Verify results
    assert_eq!(user_names, vec!["Alice", "Bob", "Charlie"]);
    
    // Should have executed exactly one batch (all requests batched together)
    assert_eq!(batch_count.load(Ordering::SeqCst), 1);
}

#[tokio::test]
async fn test_dataloader_caching() {
    let loader = UserBatchLoader::new();
    let batch_count = Arc::clone(&loader.batch_count);
    let dataloader = DataLoader::new(Arc::new(loader));
    
    // Load the same user twice
    let result1 = dataloader.load(1).await.unwrap();
    let result2 = dataloader.load(1).await.unwrap();
    
    assert_eq!(result1, "Alice");
    assert_eq!(result2, "Alice");
    
    // Should have executed exactly one batch (second load was cached)
    assert_eq!(batch_count.load(Ordering::SeqCst), 1);
    
    // Verify cache metrics
    let metrics = dataloader.get_metrics().await;
    assert_eq!(metrics.total_requests, 2);
    assert_eq!(metrics.cache_hits, 1);
    assert_eq!(metrics.cache_misses, 1);
    assert_eq!(metrics.cache_hit_ratio(), 0.5);
}

#[tokio::test]
async fn test_dataloader_load_many() {
    let loader = UserBatchLoader::new();
    let batch_count = Arc::clone(&loader.batch_count);
    let dataloader = DataLoader::new(Arc::new(loader));
    
    // Load multiple users at once
    let keys = vec![1, 2, 3, 4];
    let results = dataloader.load_many(keys).await.unwrap();
    
    assert_eq!(results.len(), 4);
    assert_eq!(results.get(&1), Some(&"Alice".to_string()));
    assert_eq!(results.get(&2), Some(&"Bob".to_string()));
    assert_eq!(results.get(&3), Some(&"Charlie".to_string()));
    assert_eq!(results.get(&4), Some(&"Diana".to_string()));
    
    // Should have executed exactly one batch
    assert_eq!(batch_count.load(Ordering::SeqCst), 1);
}

#[tokio::test]
async fn test_dataloader_error_handling() {
    let loader = FailingBatchLoader;
    let dataloader = DataLoader::new(Arc::new(loader));
    
    // Load should propagate the error
    let result = dataloader.load(1).await;
    assert!(result.is_err());
    assert_eq!(result.unwrap_err(), "Database connection failed");
}

#[tokio::test]
async fn test_dataloader_cache_operations() {
    let loader = UserBatchLoader::new();
    let batch_count = Arc::clone(&loader.batch_count);
    let dataloader = DataLoader::new(Arc::new(loader));
    
    // Load and cache a user
    let result1 = dataloader.load(1).await.unwrap();
    assert_eq!(result1, "Alice");
    assert_eq!(batch_count.load(Ordering::SeqCst), 1);
    
    // Load again - should be cached
    let result2 = dataloader.load(1).await.unwrap();
    assert_eq!(result2, "Alice");
    assert_eq!(batch_count.load(Ordering::SeqCst), 1); // No additional batch
    
    // Clear specific key
    dataloader.clear_key(&1).await;
    
    // Load again - should execute new batch
    let result3 = dataloader.load(1).await.unwrap();
    assert_eq!(result3, "Alice");
    assert_eq!(batch_count.load(Ordering::SeqCst), 2); // New batch executed
    
    // Clear entire cache
    dataloader.clear_cache().await;
    
    // Load again - should execute another batch
    let result4 = dataloader.load(1).await.unwrap();
    assert_eq!(result4, "Alice");
    assert_eq!(batch_count.load(Ordering::SeqCst), 3); // Another new batch
}

#[tokio::test]
async fn test_dataloader_max_batch_size() {
    let loader = UserBatchLoader::new();
    let batch_count = Arc::clone(&loader.batch_count);
    
    // Configure with small max batch size
    let config = DataLoaderConfig {
        max_batch_size: 2,
        batch_delay_ms: 1000, // Long delay to test max batch size trigger
        cache_enabled: true,
        cache_ttl_seconds: Some(300),
        enable_metrics: true,
    };
    
    let dataloader = DataLoader::with_config(Arc::new(loader), config);
    
    // Start 3 loads - should trigger batch execution when reaching max size
    let handles = vec![
        tokio::spawn({
            let dl = dataloader.clone();
            async move { dl.load(1).await }
        }),
        tokio::spawn({
            let dl = dataloader.clone();
            async move { dl.load(2).await }
        }),
        tokio::spawn({
            let dl = dataloader.clone();
            async move { dl.load(3).await }
        }),
    ];
    
    let results: Result<Vec<_>, _> = futures::future::try_join_all(handles).await;
    let values: Result<Vec<_>, _> = results.unwrap().into_iter().collect();
    let user_names = values.unwrap();
    
    assert_eq!(user_names.len(), 3);
    
    // Should have executed 2 batches (first 2 users in one batch due to max size, third in another)
    assert_eq!(batch_count.load(Ordering::SeqCst), 2);
}

#[tokio::test]
async fn test_dataloader_metrics() {
    let loader = UserBatchLoader::new();
    let dataloader = DataLoader::new(Arc::new(loader));
    
    // Perform various operations
    let _result1 = dataloader.load(1).await.unwrap(); // Cache miss
    let _result2 = dataloader.load(1).await.unwrap(); // Cache hit
    let _result3 = dataloader.load(2).await.unwrap(); // Cache miss
    
    let metrics = dataloader.get_metrics().await;
    
    assert_eq!(metrics.total_requests, 3);
    assert_eq!(metrics.cache_hits, 1);
    assert_eq!(metrics.cache_misses, 2);
    assert_eq!(metrics.batches_executed, 2);
    assert_eq!(metrics.cache_hit_ratio(), 1.0 / 3.0);
    assert!(metrics.average_batch_size > 0.0);
}

#[tokio::test]
async fn test_dataloader_config_options() {
    let loader = UserBatchLoader::new();
    
    // Test with caching disabled
    let config = DataLoaderConfig {
        max_batch_size: 100,
        batch_delay_ms: 10,
        cache_enabled: false,
        cache_ttl_seconds: None,
        enable_metrics: false,
    };
    
    let dataloader = DataLoader::with_config(Arc::new(loader), config);
    
    // Load the same key twice - should execute two batches since caching is disabled
    let _result1 = dataloader.load(1).await.unwrap();
    let _result2 = dataloader.load(1).await.unwrap();
    
    let metrics = dataloader.get_metrics().await;
    // With metrics disabled, counters should be zero
    assert_eq!(metrics.total_requests, 0);
    assert_eq!(metrics.cache_hits, 0);
    assert_eq!(metrics.cache_misses, 0);
}

#[test]
fn test_dataloader_config_defaults() {
    let config = DataLoaderConfig::default();
    
    assert_eq!(config.max_batch_size, 100);
    assert_eq!(config.batch_delay_ms, 10);
    assert!(config.cache_enabled);
    assert_eq!(config.cache_ttl_seconds, Some(300));
    assert!(config.enable_metrics);
}

#[test]
fn test_dataloader_metrics_calculations() {
    let mut metrics = DataLoaderMetrics::default();
    
    // Test cache hit ratio with no requests
    assert_eq!(metrics.cache_hit_ratio(), 0.0);
    
    // Add some data
    metrics.total_requests = 10;
    metrics.cache_hits = 7;
    metrics.cache_misses = 3;
    
    assert_eq!(metrics.cache_hit_ratio(), 0.7);
    
    // Test average batch size calculation
    metrics.batches_executed = 1;
    metrics.update_average_batch_size(5);
    assert_eq!(metrics.average_batch_size, 5.0);
    
    metrics.batches_executed = 2;
    metrics.update_average_batch_size(3);
    assert_eq!(metrics.average_batch_size, 4.0); // (5 + 3) / 2
}

/// Integration test demonstrating real-world usage pattern
#[tokio::test]
async fn test_dataloader_graphql_scenario() {
    // Simulate a GraphQL query that loads multiple posts, each needing its author
    let loader = UserBatchLoader::new();
    let batch_count = Arc::clone(&loader.batch_count);
    let dataloader = Arc::new(DataLoader::new(Arc::new(loader)));
    
    // Simulate resolving a list of posts, each requesting its author
    struct Post {
        id: u32,
        title: String,
        author_id: u32,
    }
    
    let posts = vec![
        Post { id: 1, title: "GraphQL Basics".to_string(), author_id: 1 },
        Post { id: 2, title: "Advanced Rust".to_string(), author_id: 2 },
        Post { id: 3, title: "DataLoader Pattern".to_string(), author_id: 1 }, // Same author as post 1
        Post { id: 4, title: "Performance Tips".to_string(), author_id: 3 },
    ];
    
    // Simulate resolving the author field for each post (this would happen in GraphQL field resolvers)
    let author_handles: Vec<_> = posts.iter().map(|post| {
        let dl = Arc::clone(&dataloader);
        let author_id = post.author_id;
        tokio::spawn(async move { 
            dl.load(author_id).await 
        })
    }).collect();
    
    let results: Result<Vec<_>, _> = futures::future::try_join_all(author_handles).await;
    let authors: Result<Vec<_>, _> = results.unwrap().into_iter().collect();
    let author_names = authors.unwrap();
    
    // Verify all authors were loaded
    assert_eq!(author_names, vec!["Alice", "Bob", "Alice", "Charlie"]);
    
    // Despite 4 author loads (including duplicate for Alice), only 1 batch should have executed
    // This demonstrates the N+1 problem solution: 4 individual loads → 1 batched load
    let final_batch_count = batch_count.load(Ordering::SeqCst);
    println!("Expected batch count: 1, Actual: {}", final_batch_count);
    
    // Get metrics for debugging
    let metrics = dataloader.get_metrics().await;
    println!("Metrics - Total requests: {}, Cache hits: {}, Cache misses: {}, Batches executed: {}", 
        metrics.total_requests, metrics.cache_hits, metrics.cache_misses, metrics.batches_executed);
    
    assert_eq!(final_batch_count, 1);
    
    // Verify caching worked (Alice was requested twice but loaded once)
    let metrics = dataloader.get_metrics().await;
    assert_eq!(metrics.total_requests, 4);
    println!("Debug: cache_hits = {}, expected = 0", metrics.cache_hits);
    // The concurrent nature means all requests are batched together, so all are cache misses initially
    assert_eq!(metrics.cache_misses, 4); // All were cache misses due to concurrent execution
    assert_eq!(metrics.batches_executed, 1);
    // With deduplication, we should only load 3 unique keys (Alice, Bob, Charlie)
    assert_eq!(metrics.total_keys_loaded, 3); // Only 3 unique users loaded: Alice, Bob, Charlie
}
