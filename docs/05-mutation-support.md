# GraphQL Mutation Support

## 📖 What are GraphQL Mutations?

GraphQL mutations are operations that modify data on the server. While queries are used to *read* data, mutations are used to *write* data - creating, updating, or deleting resources.

### 🤔 Why Separate Queries and Mutations?

This separation serves several important purposes:

1. **Clear Intent**: The operation name immediately tells you whether data will be modified
2. **Sequential Execution**: Unlike queries (which can execute in parallel), mutations execute sequentially
3. **Caching Behavior**: Clients can safely cache query results but should not cache mutation results
4. **Error Handling**: Mutations often have different error handling requirements than queries

## 🔄 Query vs Mutation: A Visual Comparison

```text
┌─────────────────────────────────────────────────────────────────┐
│                           QUERIES                               │
├─────────────────────────────────────────────────────────────────┤
│  Purpose: Read data                                             │
│  Side Effects: None (idempotent)                                │
│  Execution: Can be parallel                                     │
│  Caching: Safe to cache                                         │
│  Example: { user(id: "1") { name email } }                     │
└─────────────────────────────────────────────────────────────────┘

┌─────────────────────────────────────────────────────────────────┐
│                          MUTATIONS                              │
├─────────────────────────────────────────────────────────────────┤
│  Purpose: Write/modify data                                     │
│  Side Effects: Creates, updates, or deletes data               │
│  Execution: Always sequential (one after another)              │
│  Caching: Should not be cached                                  │
│  Example: mutation { createUser(input: { name: "John" }) }     │
└─────────────────────────────────────────────────────────────────┘
```

## 🏗️ Mutation Architecture in Our Implementation

### 1. Schema Definition

First, you define mutation fields in your schema:

```graphql
type Mutation {
  createUser(input: CreateUserInput!): User!
  updateUser(id: ID!, input: UpdateUserInput!): User!
  deleteUser(id: ID!): Boolean!
}

input CreateUserInput {
  name: String!
  email: String!
}

input UpdateUserInput {
  name: String
  email: String
}
```

### 2. Execution Flow

```text
┌─────────────────┐    ┌─────────────────┐    ┌─────────────────┐
│   1. Parse      │    │   2. Validate   │    │   3. Execute    │
│   Mutation      │ -> │   Against       │ -> │   Sequential    │
│   Document      │    │   Schema        │    │   Operations    │
└─────────────────┘    └─────────────────┘    └─────────────────┘
                                                        │
                                                        v
┌─────────────────┐    ┌─────────────────┐    ┌─────────────────┐
│   4. Return     │    │   5. Handle     │    │   6. Apply      │
│   Results or    │ <- │   Errors        │ <- │   Side Effects  │
│   Errors        │    │                 │    │   (Create/Edit) │
└─────────────────┘    └─────────────────┘    └─────────────────┘
```

### 3. Sequential Execution Guarantee

**🚨 Critical Concept**: Unlike queries, mutations must execute sequentially!

```graphql
# These mutations will execute ONE BY ONE, in order:
mutation {
  createUser(input: { name: "Alice" })   # Executes FIRST
  createUser(input: { name: "Bob" })     # Executes SECOND  
  createUser(input: { name: "Charlie" }) # Executes THIRD
}
```

**Why Sequential?**

- **Data Consistency**: Prevents race conditions
- **Predictable State**: Each mutation sees the results of previous ones
- **Error Recovery**: Clear failure points for rollbacks

## 🛠️ Implementation Details

### Core Components

#### 1. Mutation Parsing (Already Implemented ✅)

Our lexer and parser already support mutation syntax:

```rust
// In infrastructure/lexer.rs
#[token("mutation")]
Mutation,

// In infrastructure/query_parser.rs  
#[derive(Debug, Clone, PartialEq)]
pub enum OperationType {
    Query,
    Mutation,    // ✅ Already supported
    Subscription,
}
```

#### 2. Mutation Execution (🚧 Implementing Now)

The main work is in `domain/services.rs`:

```rust
impl QueryExecutor {
    /// Execute a mutation operation
    fn execute_mutation_operation(
        &self,
        operation: &OperationDefinition,
        schema: &Schema,
        variables: &Option<serde_json::Value>,
    ) -> Result<serde_json::Value, GraphQLError> {
        // 🎯 This is what we're implementing!
        
        // 1. Get the Mutation root type from schema
        // 2. Execute selection set sequentially (not parallel!)
        // 3. Apply side effects for each field
        // 4. Return results
    }
}
```

### 3. Mutation Resolvers

Unlike query resolvers (which just return data), mutation resolvers have side effects:

```rust
pub trait MutationResolver {
    /// Apply side effects and return result
    async fn resolve(
        &self,
        field: &str,
        args: &HashMap<String, serde_json::Value>,
    ) -> Result<serde_json::Value, GraphQLError>;
}

// Example implementation
impl MutationResolver for UserMutationResolver {
    async fn resolve(&self, field: &str, args: &HashMap<String, serde_json::Value>) 
        -> Result<serde_json::Value, GraphQLError> {
        match field {
            "createUser" => {
                // 1. Extract input from args
                let input = args.get("input").unwrap();
                
                // 2. Apply side effect (create user in database)
                let user = self.user_service.create_user(input).await?;
                
                // 3. Return created user data
                Ok(serde_json::to_value(user)?)
            }
            _ => Err(GraphQLError::new(format!("Unknown field: {}", field)))
        }
    }
}
```

## 🧪 Testing Strategy

### 1. Unit Tests

Test individual mutation components:

```rust
#[tokio::test]
async fn test_execute_create_user_mutation() {
    let executor = QueryExecutor::new();
    let schema = create_test_schema_with_mutations();
    
    let query = r#"
        mutation CreateUser {
            createUser(input: { name: "Alice", email: "alice@example.com" }) {
                id
                name
                email
            }
        }
    "#;
    
    let result = executor.execute(query, &schema).await;
    assert!(result.is_ok());
    
    let data = result.unwrap();
    assert_eq!(data["createUser"]["name"], "Alice");
}
```

### 2. Sequential Execution Tests

Verify mutations execute in order:

```rust
#[tokio::test] 
async fn test_sequential_mutation_execution() {
    // Test that mutations execute one by one, not in parallel
    let query = r#"
        mutation {
            first: createUser(input: { name: "First" }) { id }
            second: createUser(input: { name: "Second" }) { id }  
            third: createUser(input: { name: "Third" }) { id }
        }
    "#;
    
    // Each mutation should see the effects of the previous ones
}
```

### 3. Error Handling Tests

Test partial failure scenarios:

```rust
#[tokio::test]
async fn test_mutation_error_handling() {
    let query = r#"
        mutation {
            success: createUser(input: { name: "Valid" }) { id }
            failure: createUser(input: { name: "" }) { id }     # Invalid input
            afterError: createUser(input: { name: "After" }) { id }
        }
    "#;
    
    // Should execute 'success', fail on 'failure', and NOT execute 'afterError'
}
```

## 🎯 Common Mutation Patterns

### 1. Create Operations

```graphql
mutation CreateUser($input: CreateUserInput!) {
    createUser(input: $input) {
        id
        name
        email
        createdAt
    }
}
```

**Implementation Notes:**

- Validate required fields
- Generate new IDs  
- Set timestamps
- Return created object with all fields

### 2. Update Operations

```graphql
mutation UpdateUser($id: ID!, $input: UpdateUserInput!) {
    updateUser(id: $id, input: $input) {
        id
        name
        email
        updatedAt
    }
}
```

**Implementation Notes:**

- Check if resource exists
- Validate permissions
- Apply partial updates (only provided fields)
- Update timestamps
- Return updated object

### 3. Delete Operations

```graphql
mutation DeleteUser($id: ID!) {
    deleteUser(id: $id)  # Often returns Boolean or deleted object
}
```

**Implementation Notes:**

- Check if resource exists
- Validate permissions
- Handle cascade deletes
- Return confirmation

### 4. Batch Operations

```graphql
mutation BatchCreateUsers($users: [CreateUserInput!]!) {
    batchCreateUsers(input: $users) {
        count
        users {
            id
            name
        }
        errors {
            index
            message
        }
    }
}
```

## 🚨 Error Handling in Mutations

### 1. Validation Errors

```json
{
  "errors": [
    {
      "message": "Invalid email format",
      "path": ["createUser", "input", "email"],
      "extensions": {
        "code": "VALIDATION_ERROR"
      }
    }
  ]
}
```

### 2. Business Logic Errors

```json
{
  "errors": [
    {
      "message": "User with this email already exists",
      "path": ["createUser"],
      "extensions": {
        "code": "DUPLICATE_EMAIL"
      }
    }
  ]
}
```

### 3. Partial Success Scenarios

Some mutations might partially succeed:

```json
{
  "data": {
    "batchCreateUsers": {
      "count": 2,
      "users": [
        { "id": "1", "name": "Alice" },
        { "id": "2", "name": "Bob" }
      ],
      "errors": [
        {
          "index": 2,
          "message": "Invalid email for user at index 2"
        }
      ]
    }
  }
}
```

## 🔒 Security Considerations

### 1. Input Validation

```rust
fn validate_create_user_input(input: &CreateUserInput) -> Result<(), GraphQLError> {
    if input.name.trim().is_empty() {
        return Err(GraphQLError::new("Name cannot be empty"));
    }
    
    if !is_valid_email(&input.email) {
        return Err(GraphQLError::new("Invalid email format"));
    }
    
    // Add more validations...
    Ok(())
}
```

### 2. Authorization

```rust
async fn check_create_user_permission(context: &Context) -> Result<(), GraphQLError> {
    if !context.current_user.has_permission("CREATE_USER") {
        return Err(GraphQLError::new("Insufficient permissions"));
    }
    Ok(())
}
```

### 3. Rate Limiting

```rust
// Prevent abuse of expensive mutations
async fn check_rate_limit(user_id: &str, operation: &str) -> Result<(), GraphQLError> {
    if rate_limiter.is_exceeded(user_id, operation) {
        return Err(GraphQLError::new("Rate limit exceeded"));
    }
    Ok(())
}
```

## 🚀 Performance Considerations

### 1. Efficient Sequential Execution

While mutations must be sequential, we can optimize within each mutation:

```rust
async fn execute_mutation_field(&self, field: &Field) -> Result<Value, GraphQLError> {
    match field.name.as_str() {
        "batchCreateUsers" => {
            // Process multiple users efficiently in a single database transaction
            self.user_service.batch_create_optimized(users).await
        }
        _ => self.execute_single_mutation_field(field).await
    }
}
```

### 2. Database Transactions

```rust
async fn execute_mutation_operation(&self, operation: &OperationDefinition) -> Result<Value, GraphQLError> {
    // Start database transaction
    let mut tx = self.db.begin_transaction().await?;
    
    let mut results = HashMap::new();
    
    // Execute each field sequentially within the transaction
    for field in &operation.selection_set.selections {
        match self.execute_mutation_field(field, &mut tx).await {
            Ok(result) => {
                results.insert(field.name.clone(), result);
            }
            Err(error) => {
                // Rollback on any error
                tx.rollback().await?;
                return Err(error);
            }
        }
    }
    
    // Commit if all mutations succeeded
    tx.commit().await?;
    Ok(serde_json::to_value(results)?)
}
```

## 📋 Implementation Roadmap

### Phase 1: Basic Mutation Execution ✅ (This PR)

- [x] Implement `execute_mutation_operation` method
- [x] Add sequential execution logic
- [x] Basic mutation field resolution
- [x] Comprehensive tests

### Phase 2: Advanced Mutation Features (Future)

- [ ] Transaction support
- [ ] Batch operations
- [ ] Optimistic concurrency control
- [ ] Mutation result caching

### Phase 3: Production Features (Future)

- [ ] Authorization integration
- [ ] Rate limiting
- [ ] Audit logging
- [ ] Performance monitoring

## 💡 Learning Resources

### GraphQL Specification

- [GraphQL Mutations Spec](https://spec.graphql.org/draft/#sec-Mutation)
- [Execution - Mutations](https://spec.graphql.org/draft/#sec-Mutation)

### Best Practices

- [Mutation Design Best Practices](https://graphql.org/learn/queries/#mutations)
- [Error Handling in GraphQL](https://graphql.org/learn/validation/)

### Real-World Examples

- [GitHub GraphQL API Mutations](https://docs.github.com/en/graphql/guides/forming-calls-with-graphql#working-with-mutations)
- [Shopify GraphQL Mutations](https://shopify.dev/docs/api/admin-graphql#mutations)

---

## 🎯 Next Steps

After completing mutation support, the logical next features are:

1. **Advanced Validation** - Complete query validation against schema
2. **DataLoader Pattern** - Efficient data loading and N+1 prevention  
3. **Subscription Support** - Real-time data updates
4. **Custom Resolvers** - User-defined resolver functions

This mutation implementation provides the foundation for all these advanced features!
