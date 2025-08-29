use async_trait::async_trait;
use graphql_rs::domain::value_objects::{BatchLoadFn, DataLoader, DataLoaderConfig};
use std::collections::HashMap;
use std::sync::Arc;
use std::time::Duration;

/// Example demonstrating the DataLoader pattern for solving N+1 query problems
///
/// This example simulates a GraphQL server that needs to load user data
/// for multiple posts, demonstrating how DataLoader batches requests
/// and caches results to improve performance.

/// Mock database record for a User
#[derive(Debug, Clone)]
struct User {
    id: u32,
    name: String,
    email: String,
    department: String,
}

/// Mock database record for a Post
#[derive(Debug, Clone)]
struct Post {
    id: u32,
    title: String,
    content: String,
    author_id: u32,
}

/// Simulated database service
struct Database {
    users: HashMap<u32, User>,
    posts: HashMap<u32, Post>,
}

impl Database {
    fn new() -> Self {
        let mut users = HashMap::new();
        let mut posts = HashMap::new();

        // Create mock users
        users.insert(
            1,
            User {
                id: 1,
                name: "Alice Johnson".to_string(),
                email: "alice@example.com".to_string(),
                department: "Engineering".to_string(),
            },
        );
        users.insert(
            2,
            User {
                id: 2,
                name: "Bob Smith".to_string(),
                email: "bob@example.com".to_string(),
                department: "Product".to_string(),
            },
        );
        users.insert(
            3,
            User {
                id: 3,
                name: "Charlie Brown".to_string(),
                email: "charlie@example.com".to_string(),
                department: "Design".to_string(),
            },
        );
        users.insert(
            4,
            User {
                id: 4,
                name: "Diana Prince".to_string(),
                email: "diana@example.com".to_string(),
                department: "Marketing".to_string(),
            },
        );

        // Create mock posts
        posts.insert(
            1,
            Post {
                id: 1,
                title: "Getting Started with GraphQL".to_string(),
                content: "GraphQL is a query language...".to_string(),
                author_id: 1,
            },
        );
        posts.insert(
            2,
            Post {
                id: 2,
                title: "Advanced Rust Patterns".to_string(),
                content: "Rust provides powerful...".to_string(),
                author_id: 2,
            },
        );
        posts.insert(
            3,
            Post {
                id: 3,
                title: "DataLoader Best Practices".to_string(),
                content: "DataLoader solves the N+1...".to_string(),
                author_id: 1,
            },
        );
        posts.insert(
            4,
            Post {
                id: 4,
                title: "Building Scalable APIs".to_string(),
                content: "When building APIs...".to_string(),
                author_id: 3,
            },
        );
        posts.insert(
            5,
            Post {
                id: 5,
                title: "Database Optimization Tips".to_string(),
                content: "Database performance...".to_string(),
                author_id: 1,
            },
        );
        posts.insert(
            6,
            Post {
                id: 6,
                title: "UI/UX Design Principles".to_string(),
                content: "Good design starts...".to_string(),
                author_id: 3,
            },
        );

        Self { users, posts }
    }

    /// Simulate fetching all posts (this would be a database query in real life)
    async fn fetch_posts(&self) -> Vec<Post> {
        println!("🗃️  DATABASE: SELECT * FROM posts");
        // Simulate database delay
        tokio::time::sleep(Duration::from_millis(20)).await;
        self.posts.values().cloned().collect()
    }

    /// Simulate fetching users by IDs in batch (this would be a database query in real life)
    async fn fetch_users_by_ids(&self, user_ids: Vec<u32>) -> Result<HashMap<u32, User>, String> {
        println!(
            "🗃️  DATABASE: SELECT * FROM users WHERE id IN ({:?}) -- BATCHED QUERY",
            user_ids
        );

        // Simulate database delay
        tokio::time::sleep(Duration::from_millis(30)).await;

        let mut results = HashMap::new();
        for id in user_ids {
            if let Some(user) = self.users.get(&id) {
                results.insert(id, user.clone());
            }
        }

        Ok(results)
    }
}

/// UserLoader implements BatchLoadFn to load users in batches
struct UserLoader {
    database: Arc<Database>,
}

impl UserLoader {
    fn new(database: Arc<Database>) -> Self {
        Self { database }
    }
}

#[async_trait]
impl BatchLoadFn<u32, User, String> for UserLoader {
    async fn load(&self, keys: Vec<u32>) -> Result<HashMap<u32, User>, String> {
        self.database.fetch_users_by_ids(keys).await
    }
}

/// Simulate GraphQL field resolution without DataLoader (demonstrating N+1 problem)
async fn resolve_posts_without_dataloader(database: Arc<Database>) {
    println!("\n🚨 === RESOLVING POSTS WITHOUT DATALOADER (N+1 Problem) ===");

    let posts = database.fetch_posts().await;
    println!("📄 Loaded {} posts", posts.len());

    println!("\n📋 Resolving author for each post:");
    for post in &posts {
        println!("  📝 Post: {}", post.title);
        // This creates the N+1 problem - one query per post's author
        let users = database
            .fetch_users_by_ids(vec![post.author_id])
            .await
            .unwrap();
        if let Some(author) = users.get(&post.author_id) {
            println!("    👤 Author: {} ({})", author.name, author.department);
        }
    }

    println!(
        "\n🔥 PROBLEM: {} database queries executed (1 for posts + {} for authors)",
        1 + posts.len(),
        posts.len()
    );
}

/// Simulate GraphQL field resolution with DataLoader (solving N+1 problem)
async fn resolve_posts_with_dataloader(database: Arc<Database>) {
    println!("\n✅ === RESOLVING POSTS WITH DATALOADER (Problem Solved) ===");

    // Create DataLoader with custom configuration
    let user_loader = UserLoader::new(Arc::clone(&database));
    let config = DataLoaderConfig {
        max_batch_size: 50,
        batch_delay_ms: 10,
        cache_enabled: true,
        cache_ttl_seconds: Some(600), // 10 minutes cache
        enable_metrics: true,
    };
    let dataloader = Arc::new(DataLoader::with_config(Arc::new(user_loader), config));

    let posts = database.fetch_posts().await;
    println!("📄 Loaded {} posts", posts.len());

    println!("\n📋 Resolving author for each post:");

    // Simulate concurrent field resolution (like what happens in GraphQL)
    let author_futures: Vec<_> = posts
        .iter()
        .map(|post| {
            let dl = Arc::clone(&dataloader);
            let post_title = post.title.clone();
            let author_id = post.author_id;

            tokio::spawn(async move {
                let author = dl.load(author_id).await.unwrap();
                (post_title, author)
            })
        })
        .collect();

    // Wait for all author resolutions
    let results = futures::future::try_join_all(author_futures).await.unwrap();

    // Display results
    for (post_title, author) in results {
        println!("  📝 Post: {}", post_title);
        println!("    👤 Author: {} ({})", author.name, author.department);
    }

    // Show performance metrics
    let metrics = dataloader.get_metrics().await;
    println!("\n📊 === DATALOADER PERFORMANCE METRICS ===");
    println!("  📈 Total requests: {}", metrics.total_requests);
    println!("  🎯 Cache hits: {}", metrics.cache_hits);
    println!("  ❌ Cache misses: {}", metrics.cache_misses);
    println!("  📦 Batches executed: {}", metrics.batches_executed);
    println!("  🔑 Total keys loaded: {}", metrics.total_keys_loaded);
    println!(
        "  📊 Cache hit ratio: {:.2}%",
        metrics.cache_hit_ratio() * 100.0
    );
    println!("  📏 Average batch size: {:.1}", metrics.average_batch_size);

    println!(
        "\n✨ SOLUTION: Only {} database queries executed (1 for posts + {} batched for authors)",
        1 + metrics.batches_executed,
        metrics.batches_executed
    );
}

/// Demonstrate cache benefits with repeated requests
async fn demonstrate_caching_benefits(database: Arc<Database>) {
    println!("\n🔄 === DEMONSTRATING CACHING BENEFITS ===");

    let user_loader = UserLoader::new(Arc::clone(&database));
    let dataloader = Arc::new(DataLoader::new(Arc::new(user_loader)));

    println!("Loading user 1 for the first time...");
    let user1_first = dataloader.load(1).await.unwrap();
    println!("  👤 {}", user1_first.name);

    println!("Loading user 1 again (should be cached)...");
    let user1_second = dataloader.load(1).await.unwrap();
    println!("  👤 {} (from cache)", user1_second.name);

    println!("Loading user 2 for the first time...");
    let user2 = dataloader.load(2).await.unwrap();
    println!("  👤 {}", user2.name);

    let metrics = dataloader.get_metrics().await;
    println!("\n📊 Caching Metrics:");
    println!("  📈 Total requests: {}", metrics.total_requests);
    println!("  🎯 Cache hits: {}", metrics.cache_hits);
    println!("  ❌ Cache misses: {}", metrics.cache_misses);
    println!(
        "  🔄 Cache hit ratio: {:.2}%",
        metrics.cache_hit_ratio() * 100.0
    );
}

/// Demonstrate manual cache management
async fn demonstrate_cache_management(database: Arc<Database>) {
    println!("\n🛠️ === DEMONSTRATING CACHE MANAGEMENT ===");

    let user_loader = UserLoader::new(Arc::clone(&database));
    let dataloader = Arc::new(DataLoader::new(Arc::new(user_loader)));

    // Load and cache user
    println!("Loading user 1...");
    let _user = dataloader.load(1).await.unwrap();

    println!("Loading user 1 again (cached)...");
    let _user = dataloader.load(1).await.unwrap();

    // Clear specific key from cache
    println!("Clearing user 1 from cache...");
    dataloader.clear_key(&1).await;

    println!("Loading user 1 again (cache miss after clearing)...");
    let _user = dataloader.load(1).await.unwrap();

    // Show final metrics
    let metrics = dataloader.get_metrics().await;
    println!(
        "📊 Final metrics - Cache hits: {}, misses: {}",
        metrics.cache_hits, metrics.cache_misses
    );
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("🚀 DataLoader Pattern Demonstration");
    println!("===================================");
    println!("This example shows how DataLoader solves the N+1 query problem");
    println!("by batching database requests and caching results.\n");

    let database = Arc::new(Database::new());

    // Demonstrate the N+1 problem
    resolve_posts_without_dataloader(Arc::clone(&database)).await;

    // Demonstrate the solution with DataLoader
    resolve_posts_with_dataloader(Arc::clone(&database)).await;

    // Demonstrate caching benefits
    demonstrate_caching_benefits(Arc::clone(&database)).await;

    // Demonstrate cache management
    demonstrate_cache_management(Arc::clone(&database)).await;

    println!("\n🎯 === KEY TAKEAWAYS ===");
    println!(
        "1. 🔥 Without DataLoader: {} queries (1 + N pattern)",
        6 + 1
    );
    println!("2. ✅ With DataLoader: ~2-3 queries (1 + batched)");
    println!("3. 🚀 Performance improvement: ~70% fewer database queries");
    println!("4. 💾 Caching eliminates redundant requests within execution context");
    println!("5. 🛠️ Configurable batching, caching, and metrics");
    println!("\n✨ DataLoader Pattern Demo Complete!");

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_database_operations() {
        let db = Database::new();

        // Test fetching posts
        let posts = db.fetch_posts().await;
        assert_eq!(posts.len(), 6);

        // Test fetching users
        let users = db.fetch_users_by_ids(vec![1, 2, 3]).await.unwrap();
        assert_eq!(users.len(), 3);
        assert!(users.contains_key(&1));
        assert!(users.contains_key(&2));
        assert!(users.contains_key(&3));
    }

    #[tokio::test]
    async fn test_user_loader() {
        let db = Arc::new(Database::new());
        let loader = UserLoader::new(db);

        let users = loader.load(vec![1, 2]).await.unwrap();
        assert_eq!(users.len(), 2);
        assert_eq!(users.get(&1).unwrap().name, "Alice Johnson");
        assert_eq!(users.get(&2).unwrap().name, "Bob Smith");
    }
}
