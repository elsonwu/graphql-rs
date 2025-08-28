# GraphQL Fundamentals: Visual Guide

## 🎯 What is GraphQL?

GraphQL is a **query language for APIs** and a **runtime for executing queries** by using a type system you define for your data. Unlike REST APIs that expose multiple endpoints for different resources, GraphQL provides a **single endpoint** that can return exactly the data you request.

### 🔄 REST vs GraphQL: Visual Comparison

```text
┌─────────────────────────────────────────────────────────────────┐
│                           REST API                              │
├─────────────────────────────────────────────────────────────────┤
│  GET /users/1        → { id: 1, name: "John", email: "..." }   │
│  GET /users/1/posts  → [ { id: 1, title: "...", body: "..." }] │
│  GET /posts/1        → { id: 1, title: "...", author_id: 1 }   │
│                                                                 │
│  🔴 Problems:                                                   │
│  • Multiple round trips                                         │
│  • Over-fetching (getting unused fields)                       │
│  • Under-fetching (need another request)                       │
│  • API versioning challenges                                   │
└─────────────────────────────────────────────────────────────────┘

┌─────────────────────────────────────────────────────────────────┐
│                          GraphQL                               │
├─────────────────────────────────────────────────────────────────┤
│  POST /graphql                                                  │
│  {                                                              │
│    user(id: 1) {                                                │
│      name                                                       │
│      posts { title }                                            │
│    }                                                            │
│  }                                                              │
│                                                                 │
│  ✅ Benefits:                                                   │
│  • Single request                                               │
│  • Get exactly what you need                                    │
│  • Strongly typed                                               │
│  • Self-documenting                                             │
│  • No versioning needed                                         │
└─────────────────────────────────────────────────────────────────┘
```

## 🏗️ GraphQL Request Lifecycle: Complete Visual Flow

```text
┌─────────────┐    ┌─────────────┐    ┌─────────────┐    ┌─────────────┐
│   Client    │    │    HTTP     │    │  GraphQL    │    │   Schema    │
│ Application │    │   Server    │    │   Runtime   │    │  + Types    │
└─────────────┘    └─────────────┘    └─────────────┘    └─────────────┘
        │                   │                   │                   │
        │ 1. GraphQL Query  │                   │                   │
        │ ──────────────────▶                   │                   │
        │                   │ 2. Parse Query    │                   │
        │                   │ ──────────────────▶                   │
        │                   │                   │ 3. Validate       │
        │                   │                   │ ──────────────────▶
        │                   │                   │                   │
        │                   │                   │ 4. Execute        │
        │                   │ ◀─────────────────── (Field Resolution)
        │                   │                   │                   │
        │ 5. JSON Response  │                   │                   │
        │ ◀──────────────────                   │                   │
```

### Step-by-Step Breakdown

#### 1. Client sends GraphQL Query

```graphql
query GetUserProfile {
  user(id: "123") {
    name
    email
    posts {
      title
      publishedAt
    }
  }
}
```

#### 2. Parse Query into AST (Abstract Syntax Tree)

```text
Document
├── Operation (Query)
    ├── Field: user
    │   ├── Argument: id = "123"
    │   └── SelectionSet
    │       ├── Field: name
    │       ├── Field: email
    │       └── Field: posts
    │           └── SelectionSet
    │               ├── Field: title
    │               └── Field: publishedAt
```

#### 3. Validate Against Schema

```text
✅ Field 'user' exists on Query type
✅ Argument 'id' is valid for user field
✅ Fields 'name', 'email' exist on User type
✅ Field 'posts' returns [Post] type
✅ Fields 'title', 'publishedAt' exist on Post type
```

#### 4. Execute & Resolve Fields

```text
Execution Tree:
├── user(id: "123")           → Call user resolver
    ├── name                  → Return user.name
    ├── email                 → Return user.email
    └── posts                 → Call posts resolver
        ├── [0].title         → Return post1.title
        ├── [0].publishedAt   → Return post1.publishedAt
        ├── [1].title         → Return post2.title
        └── [1].publishedAt   → Return post2.publishedAt
```

#### 5. Return JSON Response

```json
{
  "data": {
    "user": {
      "name": "John Doe",
      "email": "john@example.com",
      "posts": [
        {
          "title": "GraphQL is Amazing",
          "publishedAt": "2024-01-15"
        },
        {
          "title": "Building APIs with Rust",
          "publishedAt": "2024-01-20"
        }
      ]
    }
  }
}
```

## 📋 Core GraphQL Concepts: Visual Guide

### 1. Schema Definition Language (SDL)

The schema is the **contract** between client and server:

```graphql
# Object Type - describes data structure
type User {
  id: ID!           # Non-null ID scalar
  name: String!     # Non-null String scalar  
  email: String     # Nullable String scalar
  age: Int          # Nullable Integer scalar
  posts: [Post!]!   # Non-null list of non-null Posts
}

type Post {
  id: ID!
  title: String!
  content: String!
  author: User!     # Connection back to User
  tags: [String!]   # List of non-null strings
}

# Root Query Type - entry points
type Query {
  user(id: ID!): User           # Get user by ID
  users: [User!]!               # Get all users
  post(id: ID!): Post           # Get post by ID
  searchPosts(query: String!): [Post!]!  # Search posts
}
```

### 2. Type System Hierarchy

```text
┌─────────────────────────────────────────────────────────────────┐
│                        GraphQL Types                           │
├─────────────────────────────────────────────────────────────────┤
│                                                                 │
│  Scalar Types (Leaf nodes)                                      │
│  ├── Built-in: String, Int, Float, Boolean, ID                 │
│  └── Custom: DateTime, Email, URL, JSON                        │
│                                                                 │
│  Object Types (Complex structures)                             │
│  ├── Fields with other types                                   │
│  ├── Can implement interfaces                                  │
│  └── Can be part of unions                                     │
│                                                                 │
│  Interface Types (Abstract contracts)                          │
│  ├── Define common fields                                      │
│  └── Implemented by objects                                    │
│                                                                 │
│  Union Types (One of many types)                               │
│  ├── SearchResult = User | Post | Comment                      │
│  └── Used for heterogeneous collections                       │
│                                                                 │
│  Enum Types (Limited values)                                   │
│  ├── Status = DRAFT | PUBLISHED | ARCHIVED                     │
│  └── Provides type safety                                      │
│                                                                 │
│  Input Types (For arguments)                                   │
│  ├── CreateUserInput { name: String!, email: String! }        │
│  └── Used in mutations and query arguments                     │
│                                                                 │
│  Type Modifiers                                                │
│  ├── Non-Null: String! (required)                             │
│  ├── List: [String] (array of strings)                        │
│  └── Non-Null List: [String!]! (required array of required)   │
│                                                                 │
└─────────────────────────────────────────────────────────────────┘
```

### 3. Field Resolution: How Data Flows

```text
Query: { user(id: "123") { name, posts { title } } }

Resolution Process:
┌─────────────────────────────────────────────────────────────────┐
│  Step 1: Resolve Root Field                                     │
│  ┌─────────────────────────────────────────────────────────────┐ │
│  │ user(id: "123")                                             │ │
│  │ ├── Call: UserResolver.user(id: "123")                     │ │
│  │ ├── Returns: User { id: "123", name: "John", ... }         │ │
│  │ └── Type: User                                              │ │
│  └─────────────────────────────────────────────────────────────┘ │
│                                │                                │
│  Step 2: Resolve Object Fields                                 │
│  ┌─────────────────────────────────────────────────────────────┐ │
│  │ user.name                                                   │ │
│  │ ├── Direct property access                                  │ │
│  │ ├── Returns: "John"                                         │ │
│  │ └── Type: String!                                           │ │
│  └─────────────────────────────────────────────────────────────┘ │
│                                │                                │
│  Step 3: Resolve Nested Objects                                │
│  ┌─────────────────────────────────────────────────────────────┐ │
│  │ user.posts                                                  │ │
│  │ ├── Call: PostResolver.posts(user: User)                   │ │
│  │ ├── Returns: [Post{title: "..."}, Post{title: "..."}]      │ │
│  │ └── Type: [Post!]!                                          │ │
│  └─────────────────────────────────────────────────────────────┘ │
│                                │                                │
│  Step 4: Resolve List Items                                    │
│  ┌─────────────────────────────────────────────────────────────┐ │
│  │ posts[0].title, posts[1].title                              │ │
│  │ ├── Direct property access for each post                    │ │
│  │ ├── Returns: ["Post 1 Title", "Post 2 Title"]              │ │
│  │ └── Type: String! (for each)                                │ │
│  └─────────────────────────────────────────────────────────────┘ │
└─────────────────────────────────────────────────────────────────┘

Final Result:
{
  "data": {
    "user": {
      "name": "John",
      "posts": [
        { "title": "Post 1 Title" },
        { "title": "Post 2 Title" }
      ]
    }
  }
}
```

## 🚀 GraphQL Operations: Visual Guide

### Queries vs Mutations vs Subscriptions

```text
┌─────────────────────────────────────────────────────────────────┐
│                            QUERIES                              │
├─────────────────────────────────────────────────────────────────┤
│  Purpose: Read data (like HTTP GET)                            │
│  Side Effects: None (idempotent)                                │
│  Execution: Can run in parallel                                 │
│  Caching: Safe to cache results                                │
│                                                                 │
│  query GetUser($id: ID!) {                                     │
│    user(id: $id) {                                              │
│      name                                                       │
│      email                                                      │
│    }                                                            │
│  }                                                              │
└─────────────────────────────────────────────────────────────────┘

┌─────────────────────────────────────────────────────────────────┐
│                           MUTATIONS                             │
├─────────────────────────────────────────────────────────────────┤
│  Purpose: Modify data (like HTTP POST/PUT/DELETE)              │
│  Side Effects: Creates, updates, or deletes data               │
│  Execution: Always sequential (one by one)                     │
│  Caching: Should not be cached                                 │
│                                                                 │
│  mutation CreateUser($input: CreateUserInput!) {               │
│    createUser(input: $input) {                                 │
│      id                                                         │
│      name                                                       │
│      email                                                      │
│    }                                                            │
│  }                                                              │
└─────────────────────────────────────────────────────────────────┘

┌─────────────────────────────────────────────────────────────────┐
│                         SUBSCRIPTIONS                          │
├─────────────────────────────────────────────────────────────────┤
│  Purpose: Real-time updates (like WebSocket)                   │
│  Side Effects: Establishes persistent connection               │
│  Execution: Long-lived connection                              │
│  Caching: Not applicable                                       │
│                                                                 │
│  subscription MessageAdded($chatId: ID!) {                     │
│    messageAdded(chatId: $chatId) {                             │
│      id                                                         │
│      text                                                       │
│      user { name }                                              │
│    }                                                            │
│  }                                                              │
└─────────────────────────────────────────────────────────────────┘
```

### Variables and Arguments Flow

```text
GraphQL Request with Variables:
┌─────────────────────────────────────────────────────────────────┐
│  {                                                              │
│    "query": "query GetUser($userId: ID!, $includeEmail: Boolean!) { │
│                user(id: $userId) {                              │
│                  name                                            │
│                  email @include(if: $includeEmail)              │
│                }                                                │
│              }",                                                │
│    "variables": {                                               │
│      "userId": "123",                                           │
│      "includeEmail": true                                       │
│    }                                                            │
│  }                                                              │
└─────────────────────────────────────────────────────────────────┘
                              │
                              ▼
┌─────────────────────────────────────────────────────────────────┐
│  Variable Substitution:                                         │
│  ┌─────────────────────────────────────────────────────────────┐ │
│  │ $userId → "123"                                             │ │
│  │ $includeEmail → true                                        │ │
│  └─────────────────────────────────────────────────────────────┘ │
│                              │                                 │
│  Effective Query:                                               │
│  ┌─────────────────────────────────────────────────────────────┐ │
│  │ user(id: "123") {                                           │ │
│  │   name                                                      │ │
│  │   email @include(if: true)  // Will be included            │ │
│  │ }                                                           │ │
│  └─────────────────────────────────────────────────────────────┘ │
└─────────────────────────────────────────────────────────────────┘
```

## 🔍 Error Handling: Visual Guide

GraphQL has a structured approach to error handling:

```text
┌─────────────────────────────────────────────────────────────────┐
│                      GraphQL Response                          │
├─────────────────────────────────────────────────────────────────┤
│  {                                                              │
│    "data": {        // Partial data (what succeeded)           │
│      "user": {                                                  │
│        "name": "John",                                          │
│        "email": null    // Failed field returns null           │
│      }                                                          │
│    },                                                           │
│    "errors": [      // Detailed error information              │
│      {                                                          │
│        "message": "Email service unavailable",                 │
│        "path": ["user", "email"],  // Location of error        │
│        "locations": [{"line": 4, "column": 5}],               │
│        "extensions": {                                          │
│          "code": "SERVICE_UNAVAILABLE",                        │
│          "timestamp": "2024-01-15T10:30:00Z"                  │
│        }                                                        │
│      }                                                          │
│    ]                                                            │
│  }                                                              │
└─────────────────────────────────────────────────────────────────┘
```

### Error Propagation Rules

```text
Field Resolution Error Propagation:
┌─────────────────────────────────────────────────────────────────┐
│                                                                 │
│  Non-null field error → Propagates up to parent                │
│  ┌─────────────────────────────────────────────────────────────┐ │
│  │ user {                                                      │ │
│  │   name!  ← Error here (non-null)                           │ │
│  │   email                                                     │ │
│  │ }                                                           │ │
│  │                                                             │ │
│  │ Result: user becomes null, error recorded                  │ │
│  └─────────────────────────────────────────────────────────────┘ │
│                                                                 │
│  Nullable field error → Field becomes null                     │
│  ┌─────────────────────────────────────────────────────────────┐ │
│  │ user {                                                      │ │
│  │   name                                                      │ │
│  │   email  ← Error here (nullable)                           │ │
│  │ }                                                           │ │
│  │                                                             │ │
│  │ Result: email becomes null, error recorded, user remains   │ │
│  └─────────────────────────────────────────────────────────────┘ │
└─────────────────────────────────────────────────────────────────┘
```

## 🎯 Our Implementation: Architecture Overview

```text
┌─────────────────────────────────────────────────────────────────┐
│                  GraphQL-RS Architecture                       │
├─────────────────────────────────────────────────────────────────┤
│                                                                 │
│  ┌─────────────────┐    ┌─────────────────┐                    │
│  │   HTTP Layer    │    │   Presentation  │                    │
│  │   (Axum/Warp)   │◄──►│     Layer       │                    │
│  └─────────────────┘    └─────────────────┘                    │
│                                   │                            │
│  ┌─────────────────┐    ┌─────────────────┐                    │
│  │  Infrastructure │    │   Application   │                    │
│  │     Layer       │◄──►│     Layer       │                    │
│  │ • Query Parser  │    │ • Use Cases     │                    │
│  │ • Schema Parser │    │ • Services      │                    │
│  │ • Lexer         │    └─────────────────┘                    │
│  └─────────────────┘              │                            │
│                                   │                            │
│                         ┌─────────────────┐                    │
│                         │  Domain Layer   │                    │
│                         │ • Entities      │                    │
│                         │ • Value Objects │                    │
│                         │ • Services      │                    │
│                         │ • Events        │                    │
│                         └─────────────────┘                    │
│                                                                 │
└─────────────────────────────────────────────────────────────────┘
```

### Request Flow in Our Implementation

```text
HTTP Request → GraphQL Runtime → Response
     │                 │              │
     ▼                 ▼              ▼
┌──────────┐    ┌──────────────┐    ┌──────────┐
│   POST   │    │    Parse     │    │   JSON   │
│ /graphql │───▶│   Validate   │───▶│ Response │
│          │    │   Execute    │    │          │
└──────────┘    └──────────────┘    └──────────┘
                       │
                       ▼
                ┌──────────────────────────┐
                │   Our Rust Components    │
                │                          │
                │ 1. QueryParser          │
                │    • Lexical Analysis    │
                │    • Syntax Parsing      │
                │                          │
                │ 2. SchemaValidator       │
                │    • Type Checking       │
                │    • Rule Validation     │
                │                          │
                │ 3. QueryExecutor         │
                │    • Field Resolution    │
                │    • Data Fetching       │
                │                          │
                │ 4. ResponseBuilder       │
                │    • Error Formatting    │
                │    • JSON Serialization  │
                └──────────────────────────┘
```

## 🚀 What Makes GraphQL Powerful

### 1. Single Source of Truth

The schema serves as the single contract between frontend and backend teams.

### 2. Strong Type Safety

Every field, argument, and return type is explicitly defined and validated.

### 3. Introspection

GraphQL APIs are self-documenting through introspection queries.

### 4. Developer Experience

Tools like GraphQL Playground, Apollo Studio, and IDE extensions provide excellent DX.

### 5. Performance

- Clients request exactly the data they need
- Resolvers can be optimized independently  
- DataLoader pattern prevents N+1 queries

## 🎯 Next Steps

Now that you understand GraphQL fundamentals, explore:

1. **[Domain Model](02-domain-model.md)** - How we structure our Rust implementation
2. **[Schema Definition](03-schema-definition.md)** - Building type-safe GraphQL schemas
3. **[Query Execution](04-query-execution.md)** - How queries are processed and resolved
4. **[Mutation Support](05-mutation-support.md)** - Data modification with sequential execution

This foundation will help you understand how each component works together to create a complete GraphQL server! 🎉

- **Query**: Entry point for read operations
- **Mutation**: Entry point for write operations  
- **Subscription**: Entry point for real-time updates

### 3. Queries

Clients specify exactly what data they need:

```graphql
query GetUser {
  user(id: "123") {
    id
    name
    posts {
      id
      title
    }
  }
}
```

### 4. Mutations

Modify data on the server:

```graphql
mutation CreateUser {
  createUser(input: {
    name: "John Doe"
    email: "john@example.com"
  }) {
    id
    name
  }
}
```

### 5. Subscriptions

Real-time data updates:

```graphql
subscription PostCreated {
  postCreated {
    id
    title
    author {
      name
    }
  }
}
```

## GraphQL Execution Model

### 1. Parsing

- Parse the query string into an Abstract Syntax Tree (AST)
- Validate syntax according to GraphQL grammar

### 2. Validation

- Validate the AST against the schema
- Check field existence, argument types, etc.

### 3. Execution

- Execute the query by calling resolver functions
- Handle field resolution in depth-first manner
- Collect and format results

## Key Benefits

1. **Single Endpoint**: One URL for all data operations
2. **Precise Data Fetching**: No over-fetching or under-fetching
3. **Strong Type System**: Compile-time query validation
4. **Introspection**: Self-documenting APIs
5. **Real-time**: Built-in subscription support
6. **Versionless**: Evolve APIs without breaking changes

## GraphQL vs REST

| Aspect | GraphQL | REST |
|--------|---------|------|
| Endpoints | Single endpoint | Multiple endpoints |
| Data Fetching | Precise, client-controlled | Fixed structure |
| Over-fetching | Eliminated | Common issue |
| Versioning | Not needed | Required |
| Caching | Complex | Simple (HTTP caching) |
| Learning Curve | Steeper | Gentler |

## Implementation Challenges

1. **Query Complexity**: Preventing expensive nested queries
2. **N+1 Problem**: Efficient data loading patterns
3. **Caching**: More complex than HTTP caching
4. **File Uploads**: Not part of core spec
5. **Security**: Query depth and complexity attacks

## Next Steps

Understanding these fundamentals is crucial for implementing a GraphQL server. In the next sections, we'll dive into:

- Domain modeling for GraphQL
- Schema design patterns
- Resolver implementation strategies
- Performance optimization techniques
