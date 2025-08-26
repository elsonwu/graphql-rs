# GraphQL Fundamentals

## What is GraphQL?

GraphQL is a query language and runtime for APIs that was developed by Facebook in 2012 and open-sourced in 2015. It provides a complete and understandable description of the data in your API, gives clients the power to ask for exactly what they need, and enables powerful developer tools.

## Core Concepts

### 1. Schema Definition Language (SDL)

The GraphQL SDL is a human-readable syntax for defining GraphQL schemas. It describes the shape of your API and the types of data available.

```graphql
type User {
  id: ID!
  name: String!
  email: String
  posts: [Post!]!
}

type Post {
  id: ID!
  title: String!
  content: String!
  author: User!
}

type Query {
  user(id: ID!): User
  posts: [Post!]!
}
```

### 2. Type System

GraphQL has a strong type system that defines the capabilities of an API. All types are explicitly defined:

#### Scalar Types

- `Int`: 32-bit signed integer
- `Float`: Double-precision floating-point
- `String`: UTF-8 character sequence
- `Boolean`: true or false
- `ID`: Unique identifier (serialized as String)

#### Object Types

Define the structure of data with fields that can be other objects or scalars.

#### Special Types

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
