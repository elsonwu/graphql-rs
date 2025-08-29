use async_trait::async_trait;
use graphql_rs::domain::{
    entities::{
        ids::{SchemaId, SchemaVersion},
        query::Query,
        schema::Schema,
        types::{FieldDefinition, GraphQLType, ObjectType, ScalarType},
    },
    services::{DataLoaderContext, DataLoaderContextBuilder, QueryExecution, QueryExecutor},
    value_objects::{BatchLoadFn, DataLoader, DataLoaderConfig, ValidationResult},
};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::time::Duration;

/// Example demonstrating GraphQL DataLoader integration
///
/// This shows how to use DataLoaders within a GraphQL server context
/// to efficiently resolve fields that require database access,
/// solving the N+1 query problem in GraphQL field resolution.

/// Mock User entity
#[derive(Debug, Clone)]
struct User {
    id: u32,
    name: String,
    email: String,
}

/// Mock Post entity  
#[derive(Debug, Clone)]
struct Post {
    id: u32,
    title: String,
    author_id: u32,
    category_id: u32,
}

/// Mock Category entity
#[derive(Debug, Clone)]
struct Category {
    id: u32,
    name: String,
    description: String,
}

/// Mock database service
struct Database {
    users: HashMap<u32, User>,
    posts: HashMap<u32, Post>,
    categories: HashMap<u32, Category>,
}

impl Database {
    fn new() -> Self {
        let mut users = HashMap::new();
        let mut posts = HashMap::new();
        let mut categories = HashMap::new();

        // Create mock data
        users.insert(
            1,
            User {
                id: 1,
                name: "Alice".to_string(),
                email: "alice@example.com".to_string(),
            },
        );
        users.insert(
            2,
            User {
                id: 2,
                name: "Bob".to_string(),
                email: "bob@example.com".to_string(),
            },
        );
        users.insert(
            3,
            User {
                id: 3,
                name: "Charlie".to_string(),
                email: "charlie@example.com".to_string(),
            },
        );

        categories.insert(
            1,
            Category {
                id: 1,
                name: "Technology".to_string(),
                description: "Tech posts".to_string(),
            },
        );
        categories.insert(
            2,
            Category {
                id: 2,
                name: "Science".to_string(),
                description: "Science posts".to_string(),
            },
        );

        posts.insert(
            1,
            Post {
                id: 1,
                title: "GraphQL Basics".to_string(),
                author_id: 1,
                category_id: 1,
            },
        );
        posts.insert(
            2,
            Post {
                id: 2,
                title: "Advanced DataLoaders".to_string(),
                author_id: 1,
                category_id: 1,
            },
        );
        posts.insert(
            3,
            Post {
                id: 3,
                title: "Rust Performance".to_string(),
                author_id: 2,
                category_id: 1,
            },
        );
        posts.insert(
            4,
            Post {
                id: 4,
                title: "Quantum Computing".to_string(),
                author_id: 3,
                category_id: 2,
            },
        );

        Self {
            users,
            posts,
            categories,
        }
    }

    /// Batch load users by IDs
    async fn load_users(&self, user_ids: Vec<u32>) -> Result<HashMap<u32, User>, String> {
        println!(
            "🗃️  DATABASE: Batch loading {} users: {:?}",
            user_ids.len(),
            user_ids
        );
        tokio::time::sleep(Duration::from_millis(20)).await;

        let mut results = HashMap::new();
        for id in user_ids {
            if let Some(user) = self.users.get(&id) {
                results.insert(id, user.clone());
            }
        }
        Ok(results)
    }

    /// Batch load categories by IDs
    async fn load_categories(
        &self,
        category_ids: Vec<u32>,
    ) -> Result<HashMap<u32, Category>, String> {
        println!(
            "🗃️  DATABASE: Batch loading {} categories: {:?}",
            category_ids.len(),
            category_ids
        );
        tokio::time::sleep(Duration::from_millis(15)).await;

        let mut results = HashMap::new();
        for id in category_ids {
            if let Some(category) = self.categories.get(&id) {
                results.insert(id, category.clone());
            }
        }
        Ok(results)
    }

    /// Load all posts (simulates GraphQL posts query)
    async fn load_all_posts(&self) -> Result<Vec<Post>, String> {
        println!("🗃️  DATABASE: Loading all posts");
        tokio::time::sleep(Duration::from_millis(25)).await;
        Ok(self.posts.values().cloned().collect())
    }
}

/// User DataLoader implementation
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
        self.database.load_users(keys).await
    }
}

/// Category DataLoader implementation
struct CategoryLoader {
    database: Arc<Database>,
}

impl CategoryLoader {
    fn new(database: Arc<Database>) -> Self {
        Self { database }
    }
}

#[async_trait]
impl BatchLoadFn<u32, Category, String> for CategoryLoader {
    async fn load(&self, keys: Vec<u32>) -> Result<HashMap<u32, Category>, String> {
        self.database.load_categories(keys).await
    }
}

/// Create a GraphQL schema for our blog example
fn create_blog_schema() -> Schema {
    let mut types = HashMap::new();

    // Add scalar types
    types.insert(
        "String".to_string(),
        GraphQLType::Scalar(ScalarType::String),
    );
    types.insert("Int".to_string(), GraphQLType::Scalar(ScalarType::Int));
    types.insert("ID".to_string(), GraphQLType::Scalar(ScalarType::ID));

    // User type
    let mut user_fields = HashMap::new();
    user_fields.insert(
        "id".to_string(),
        FieldDefinition {
            name: "id".to_string(),
            description: Some("User ID".to_string()),
            field_type: GraphQLType::NonNull(Box::new(GraphQLType::Scalar(ScalarType::ID))),
            arguments: HashMap::new(),
            deprecation_reason: None,
        },
    );
    user_fields.insert(
        "name".to_string(),
        FieldDefinition {
            name: "name".to_string(),
            description: Some("User name".to_string()),
            field_type: GraphQLType::NonNull(Box::new(GraphQLType::Scalar(ScalarType::String))),
            arguments: HashMap::new(),
            deprecation_reason: None,
        },
    );
    user_fields.insert(
        "email".to_string(),
        FieldDefinition {
            name: "email".to_string(),
            description: Some("User email".to_string()),
            field_type: GraphQLType::NonNull(Box::new(GraphQLType::Scalar(ScalarType::String))),
            arguments: HashMap::new(),
            deprecation_reason: None,
        },
    );

    let user_type = GraphQLType::Object(ObjectType {
        name: "User".to_string(),
        description: Some("A blog user".to_string()),
        fields: user_fields,
        interfaces: vec![],
    });
    types.insert("User".to_string(), user_type);

    // Category type
    let mut category_fields = HashMap::new();
    category_fields.insert(
        "id".to_string(),
        FieldDefinition {
            name: "id".to_string(),
            description: Some("Category ID".to_string()),
            field_type: GraphQLType::NonNull(Box::new(GraphQLType::Scalar(ScalarType::ID))),
            arguments: HashMap::new(),
            deprecation_reason: None,
        },
    );
    category_fields.insert(
        "name".to_string(),
        FieldDefinition {
            name: "name".to_string(),
            description: Some("Category name".to_string()),
            field_type: GraphQLType::NonNull(Box::new(GraphQLType::Scalar(ScalarType::String))),
            arguments: HashMap::new(),
            deprecation_reason: None,
        },
    );
    category_fields.insert(
        "description".to_string(),
        FieldDefinition {
            name: "description".to_string(),
            description: Some("Category description".to_string()),
            field_type: GraphQLType::Scalar(ScalarType::String),
            arguments: HashMap::new(),
            deprecation_reason: None,
        },
    );

    let category_type = GraphQLType::Object(ObjectType {
        name: "Category".to_string(),
        description: Some("A blog category".to_string()),
        fields: category_fields,
        interfaces: vec![],
    });
    types.insert("Category".to_string(), category_type);

    // Post type
    let mut post_fields = HashMap::new();
    post_fields.insert(
        "id".to_string(),
        FieldDefinition {
            name: "id".to_string(),
            description: Some("Post ID".to_string()),
            field_type: GraphQLType::NonNull(Box::new(GraphQLType::Scalar(ScalarType::ID))),
            arguments: HashMap::new(),
            deprecation_reason: None,
        },
    );
    post_fields.insert(
        "title".to_string(),
        FieldDefinition {
            name: "title".to_string(),
            description: Some("Post title".to_string()),
            field_type: GraphQLType::NonNull(Box::new(GraphQLType::Scalar(ScalarType::String))),
            arguments: HashMap::new(),
            deprecation_reason: None,
        },
    );
    post_fields.insert(
        "author".to_string(),
        FieldDefinition {
            name: "author".to_string(),
            description: Some("Post author (resolved via DataLoader)".to_string()),
            field_type: GraphQLType::NonNull(Box::new(GraphQLType::Scalar(ScalarType::String))),
            arguments: HashMap::new(),
            deprecation_reason: None,
        },
    );
    post_fields.insert(
        "category".to_string(),
        FieldDefinition {
            name: "category".to_string(),
            description: Some("Post category (resolved via DataLoader)".to_string()),
            field_type: GraphQLType::NonNull(Box::new(GraphQLType::Scalar(ScalarType::String))),
            arguments: HashMap::new(),
            deprecation_reason: None,
        },
    );

    let post_type = GraphQLType::Object(ObjectType {
        name: "Post".to_string(),
        description: Some("A blog post".to_string()),
        fields: post_fields,
        interfaces: vec![],
    });
    types.insert("Post".to_string(), post_type);

    // Query type
    let mut query_fields = HashMap::new();
    query_fields.insert(
        "posts".to_string(),
        FieldDefinition {
            name: "posts".to_string(),
            description: Some("Get all blog posts".to_string()),
            field_type: GraphQLType::NonNull(Box::new(GraphQLType::List(Box::new(
                GraphQLType::NonNull(Box::new(GraphQLType::Scalar(ScalarType::String))),
            )))),
            arguments: HashMap::new(),
            deprecation_reason: None,
        },
    );

    let query_type = GraphQLType::Object(ObjectType {
        name: "Query".to_string(),
        description: Some("Root query type".to_string()),
        fields: query_fields,
        interfaces: vec![],
    });
    types.insert("Query".to_string(), query_type);

    Schema {
        id: SchemaId::new(),
        version: SchemaVersion::new("1.0.0"),
        query_type: "Query".to_string(),
        mutation_type: None,
        subscription_type: None,
        types,
        directives: HashMap::new(),
        description: Some("Blog GraphQL schema with DataLoader integration".to_string()),
    }
}

/// Simulate GraphQL field resolution with DataLoaders
async fn simulate_graphql_execution_with_dataloaders(
    database: Arc<Database>,
    dataloader_context: Arc<DataLoaderContext>,
) -> Result<(), Box<dyn std::error::Error>> {
    println!("🚀 Simulating GraphQL query execution with DataLoaders...");

    // Simulate loading posts (like GraphQL posts field resolution)
    let posts = database.load_all_posts().await?;
    println!("📄 Loaded {} posts", posts.len());

    println!("\n📋 Resolving nested fields using DataLoaders:");

    // Get DataLoaders from context
    let user_loader: &DataLoader<u32, User, String> = dataloader_context
        .get_dataloader("User")
        .expect("User DataLoader should be registered");

    let category_loader: &DataLoader<u32, Category, String> = dataloader_context
        .get_dataloader("Category")
        .expect("Category DataLoader should be registered");

    // Simulate concurrent field resolution (like GraphQL would do)
    for post in &posts {
        let user_loader_clone = user_loader.clone();
        let category_loader_clone = category_loader.clone();
        let post_title = post.title.clone();
        let author_id = post.author_id;
        let category_id = post.category_id;

        // In a real GraphQL resolver, these would be separate field resolvers
        // running concurrently. Here we simulate that behavior.
        let author_future = tokio::spawn(async move { user_loader_clone.load(author_id).await });

        let category_future =
            tokio::spawn(async move { category_loader_clone.load(category_id).await });

        let (author_result, category_result) = tokio::join!(author_future, category_future);
        let author = author_result??;
        let category = category_result??;

        println!("  📝 Post: {}", post_title);
        println!("    👤 Author: {} ({})", author.name, author.email);
        println!(
            "    📂 Category: {} - {}",
            category.name, category.description
        );
    }

    // Display DataLoader metrics
    let user_metrics = user_loader.get_metrics().await;
    let category_metrics = category_loader.get_metrics().await;

    println!("\n📊 === DATALOADER PERFORMANCE METRICS ===");
    println!("👤 User DataLoader:");
    println!("  📈 Total requests: {}", user_metrics.total_requests);
    println!("  🎯 Cache hits: {}", user_metrics.cache_hits);
    println!("  ❌ Cache misses: {}", user_metrics.cache_misses);
    println!("  📦 Batches executed: {}", user_metrics.batches_executed);
    println!(
        "  📊 Cache hit ratio: {:.2}%",
        user_metrics.cache_hit_ratio() * 100.0
    );

    println!("\n📂 Category DataLoader:");
    println!("  📈 Total requests: {}", category_metrics.total_requests);
    println!("  🎯 Cache hits: {}", category_metrics.cache_hits);
    println!("  ❌ Cache misses: {}", category_metrics.cache_misses);
    println!(
        "  📦 Batches executed: {}",
        category_metrics.batches_executed
    );
    println!(
        "  📊 Cache hit ratio: {:.2}%",
        category_metrics.cache_hit_ratio() * 100.0
    );

    Ok(())
}

/// Demonstrate GraphQL query execution (simplified)
async fn demonstrate_graphql_query_execution() -> Result<(), Box<dyn std::error::Error>> {
    println!("\n🔍 === GRAPHQL QUERY EXECUTION ===");

    let schema = create_blog_schema();
    let executor = QueryExecutor::new();

    // Create a simple GraphQL query
    let mut query = Query::new("query { posts { id title } }".to_string());
    query.mark_validated(ValidationResult::Valid);

    println!("🔄 Executing GraphQL query: {}", query.query_string());
    let result = executor.execute(&query, &schema).await;

    if result.errors.is_empty() {
        println!("✅ Query executed successfully");
        if let Some(data) = result.data {
            println!("📊 Result data: {}", data);
        }
    } else {
        println!("❌ Query execution failed:");
        for error in &result.errors {
            println!("  • {}", error.message);
        }
    }

    Ok(())
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("🚀 GraphQL DataLoader Integration Demo");
    println!("======================================");
    println!("This example shows how DataLoaders integrate with GraphQL services");
    println!("to provide efficient data loading in field resolvers.\n");

    let database = Arc::new(Database::new());

    // Create DataLoaders
    let user_config = DataLoaderConfig {
        max_batch_size: 50,
        batch_delay_ms: 10,
        cache_enabled: true,
        cache_ttl_seconds: Some(300), // 5 minutes
        enable_metrics: true,
    };

    let category_config = DataLoaderConfig {
        max_batch_size: 30,
        batch_delay_ms: 10,
        cache_enabled: true,
        cache_ttl_seconds: Some(600), // 10 minutes (categories change less frequently)
        enable_metrics: true,
    };

    let user_loader = DataLoader::with_config(
        Arc::new(UserLoader::new(Arc::clone(&database))),
        user_config,
    );

    let category_loader = DataLoader::with_config(
        Arc::new(CategoryLoader::new(Arc::clone(&database))),
        category_config,
    );

    // Create DataLoader context
    let dataloader_context = Arc::new(
        DataLoaderContextBuilder::new()
            .with_dataloader("User", user_loader)
            .with_dataloader("Category", category_loader)
            .build(),
    );

    // Demonstrate GraphQL field resolution with DataLoaders
    simulate_graphql_execution_with_dataloaders(
        Arc::clone(&database),
        Arc::clone(&dataloader_context),
    )
    .await?;

    // Demonstrate basic GraphQL query execution
    demonstrate_graphql_query_execution().await?;

    println!("\n🎯 === KEY BENEFITS OF GRAPHQL + DATALOADER ===");
    println!("1. 🔥 Solves N+1 query problem in GraphQL field resolvers");
    println!("2. 🚀 Automatic request batching across concurrent field resolvers");
    println!("3. 💾 Intelligent caching within single GraphQL request execution");
    println!("4. 📊 Performance metrics and monitoring built-in");
    println!("5. 🛠️ Easy integration with existing GraphQL services");
    println!("6. 🔧 Configurable batching and caching per entity type");

    println!("\n✨ GraphQL DataLoader Integration Demo Complete!");

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_database_operations() {
        let db = Database::new();

        let users = db.load_users(vec![1, 2]).await.unwrap();
        assert_eq!(users.len(), 2);

        let categories = db.load_categories(vec![1]).await.unwrap();
        assert_eq!(categories.len(), 1);

        let posts = db.load_all_posts().await.unwrap();
        assert_eq!(posts.len(), 4);
    }

    #[tokio::test]
    async fn test_dataloader_context() {
        let db = Arc::new(Database::new());
        let user_loader = DataLoader::new(Arc::new(UserLoader::new(db)));

        let mut context = DataLoaderContext::new();
        context.register_dataloader("User", user_loader);

        let retrieved_loader: Option<&DataLoader<u32, User, String>> =
            context.get_dataloader("User");
        assert!(retrieved_loader.is_some());
    }

    #[tokio::test]
    async fn test_dataloader_context_builder() {
        let db = Arc::new(Database::new());
        let user_loader = DataLoader::new(Arc::new(UserLoader::new(Arc::clone(&db))));
        let category_loader = DataLoader::new(Arc::new(CategoryLoader::new(db)));

        let context = DataLoaderContextBuilder::new()
            .with_dataloader("User", user_loader)
            .with_dataloader("Category", category_loader)
            .build();

        let user_loader: Option<&DataLoader<u32, User, String>> = context.get_dataloader("User");
        let category_loader: Option<&DataLoader<u32, Category, String>> =
            context.get_dataloader("Category");

        assert!(user_loader.is_some());
        assert!(category_loader.is_some());
    }

    #[test]
    fn test_blog_schema_creation() {
        let schema = create_blog_schema();

        assert_eq!(schema.query_type, "Query");
        assert!(schema.mutation_type.is_none());
        assert!(schema.subscription_type.is_none());
        assert!(schema.get_type("User").is_some());
        assert!(schema.get_type("Post").is_some());
        assert!(schema.get_type("Category").is_some());
        assert!(schema.get_type("Query").is_some());
    }
}
