# Domain Model Design: Visual GraphQL Architecture

This guide shows how we use Domain-Driven Design (DDD) to build a clean, maintainable GraphQL server architecture with visual diagrams and practical examples.

## 🏗️ How Our GraphQL Server is Organized (High-Level View)

```text
┌────────────────────────────────────────────────────────────────┐
│                    GraphQL Request Flow                        │
├────────────────────────────────────────────────────────────────┤
│ 1. HTTP Request → Presentation Layer (GraphQL endpoint)        │
│ 2. Parse & Validate → Application Layer (Use cases)            │
│ 3. Execute Query → Domain Layer (Business logic)               │
│ 4. Fetch Data → Infrastructure Layer (Resolvers, DB)           │
│ 5. Return Response ← All layers collaborate                     │
└────────────────────────────────────────────────────────────────┘
```

### 🧱 DDD Architecture Layers (Visual)

```text
┌─────────────────────┐  HTTP requests, GraphQL over HTTP
│ Presentation Layer  │  ← Controllers, GraphQL endpoints
├─────────────────────┤
│ Application Layer   │  ← Use cases, orchestration
├─────────────────────┤
│ Domain Layer        │  ← Business logic, entities, rules
├─────────────────────┤
│ Infrastructure      │  ← Data access, external services
└─────────────────────┘
```

### 🎯 Practical Example: How a Query Flows Through the System

**Query:** `{ user(id: "123") { name, posts { title } } }`

```text
1. Presentation Layer
   ┌─────────────────┐
   │ GraphQL Handler │ ← Receives HTTP POST with query
   └─────────────────┘
           │
2. Application Layer
   ┌─────────────────┐
   │ ExecuteQuery    │ ← Parses, validates, orchestrates
   │ UseCase         │
   └─────────────────┘
           │
3. Domain Layer
   ┌─────────────────┐
   │ QueryExecutor   │ ← Applies business rules, validation
   │ SchemaValidator │
   └─────────────────┘
           │
4. Infrastructure Layer
   ┌─────────────────┐
   │ UserResolver    │ ← Fetches data from database
   │ PostResolver    │
   └─────────────────┘
```

## Domain-Driven Design Overview

DDD focuses on modeling the business domain and organizing code around domain concepts rather than technical concerns. This approach helps create more maintainable and understandable software.

### 🔧 Core DDD Concepts (With GraphQL Examples)

1. **Domain**: GraphQL execution and schema management
2. **Ubiquitous Language**: "Query", "Schema", "Resolver", "Field" (shared by all team members)
3. **Bounded Context**: GraphQL Engine (separate from Auth, User Management, etc.)
4. **Entities**: Schema, Query (have identity and lifecycle)
5. **Value Objects**: Field, TypeDefinition (no identity, just data)
6. **Aggregates**: SchemaAggregate (consistency boundary around Schema + Resolvers)
7. **Domain Services**: QueryValidator, SchemaValidator (business operations)
8. **Repository**: SchemaRepository (data access abstraction)

## GraphQL Domain Model

### 🎨 Domain Model Visual Overview

```text
┌─────────────────────────────────────────────────────────────────┐
│                     GraphQL Domain Model                        │
├─────────────────────────────────────────────────────────────────┤
│                                                                 │
│  ┌─────────────┐      contains      ┌─────────────┐            │
│  │   Schema    │ ─────────────────▶ │ TypeDef     │            │
│  │ (Entity)    │                    │ (Value Obj) │            │
│  └─────────────┘                    └─────────────┘            │
│         │                                   │                  │
│         │ validates                         │ describes        │
│         ▼                                   ▼                  │
│  ┌─────────────┐      executes      ┌─────────────┐            │
│  │    Query    │ ─────────────────▶ │ Selection   │            │
│  │ (Entity)    │                    │ (Value Obj) │            │
│  └─────────────┘                    └─────────────┘            │
│         │                                                      │
│         │ managed by                                           │
│         ▼                                                      │
│  ┌─────────────┐                                              │
│  │Schema       │  ← Aggregate Root                            │
│  │Aggregate    │    (Consistency boundary)                    │
│  └─────────────┘                                              │
│                                                               │
└─────────────────────────────────────────────────────────────────┘
```

### 🔄 How Domain Objects Interact (Request Flow)

```text
1. Client Query Request
   ┌─────────────┐
   │   Query     │ ← "{ user { name } }"
   │  (Entity)   │
   └─────────────┘
           │ passed to
           ▼
2. Schema Validation
   ┌─────────────┐
   │   Schema    │ ← Has TypeDefinitions, validates query
   │  (Entity)   │
   └─────────────┘
           │ if valid
           ▼
3. Execution Planning
   ┌─────────────┐
   │ Selection   │ ← Breaks query into field selections
   │(Value Obj)  │
   └─────────────┘
           │ executed by
           ▼
4. Aggregate Coordination
   ┌─────────────┐
   │Schema       │ ← Orchestrates validation + execution
   │Aggregate    │
   └─────────────┘
```

### Core Bounded Context: GraphQL Execution Engine

Our primary bounded context encompasses the GraphQL specification implementation.

#### Domain Entities

##### Schema

- **Identity**: Schema name/version
- **Invariants**: Must contain valid type definitions, must have Query type
- **Behavior**: Validate queries, provide introspection data

```rust
pub struct Schema {
    id: SchemaId,
    version: SchemaVersion,
    types: HashMap<TypeName, TypeDefinition>,
    query_type: ObjectType,
    mutation_type: Option<ObjectType>,
    subscription_type: Option<ObjectType>,
}
```

##### Query

- **Identity**: Query ID (for tracing/caching)
- **Invariants**: Must be valid GraphQL syntax, must pass validation
- **Behavior**: Execute against schema, return results

```rust
pub struct Query {
    id: QueryId,
    document: Document,
    operation_name: Option<String>,
    variables: Variables,
    validation_result: ValidationResult,
}
```

#### Value Objects

##### TypeDefinition

Represents GraphQL type information without identity:

```rust
#[derive(Clone, PartialEq)]
pub enum TypeDefinition {
    Scalar(ScalarType),
    Object(ObjectType),
    Interface(InterfaceType),
    Union(UnionType),
    Enum(EnumType),
    InputObject(InputObjectType),
}
```

##### Field
Represents a field within an object type:

```rust
#[derive(Clone, PartialEq)]
pub struct Field {
    name: FieldName,
    type_ref: TypeReference,
    arguments: Vec<Argument>,
    description: Option<String>,
    deprecation_reason: Option<String>,
}
```

##### Selection
Represents field selections in a query:

```rust
#[derive(Clone, PartialEq)]
pub enum Selection {
    Field {
        name: String,
        alias: Option<String>,
        arguments: Vec<(String, Value)>,
        selection_set: Vec<Selection>,
    },
    InlineFragment {
        type_condition: Option<String>,
        selection_set: Vec<Selection>,
    },
    FragmentSpread {
        name: String,
    },
}
```

#### Aggregates

##### SchemaAggregate
Manages the complete GraphQL schema with consistency boundaries:

```rust
pub struct SchemaAggregate {
    schema: Schema,
    resolver_registry: ResolverRegistry,
    validation_rules: Vec<ValidationRule>,
}

impl SchemaAggregate {
    pub fn validate_query(&self, query: &Query) -> ValidationResult {
        // Validate query against schema
    }
    
    pub fn execute_query(&self, query: Query) -> ExecutionResult {
        // Execute validated query
    }
}
```

#### Domain Services

##### SchemaValidator
Validates schema definitions for correctness:

```rust
pub struct SchemaValidator;

impl SchemaValidator {
    pub fn validate(&self, schema: &Schema) -> SchemaValidationResult {
        // Validate schema structure and rules
    }
}
```

##### QueryExecutor
Orchestrates query execution:

```rust
pub struct QueryExecutor {
    schema: Arc<Schema>,
    resolver_registry: Arc<ResolverRegistry>,
}

impl QueryExecutor {
    pub async fn execute(&self, query: Query) -> ExecutionResult {
        // Execute query with proper error handling
    }
}
```

##### QueryValidator
Validates queries against schema:

```rust
pub struct QueryValidator {
    schema: Arc<Schema>,
    rules: Vec<ValidationRule>,
}

impl QueryValidator {
    pub fn validate(&self, query: &Query) -> ValidationResult {
        // Apply all validation rules
    }
}
```

#### Repositories

##### SchemaRepository
Manages schema persistence and retrieval:

```rust
pub trait SchemaRepository {
    async fn save(&self, schema: Schema) -> Result<(), SchemaError>;
    async fn find_by_id(&self, id: SchemaId) -> Result<Option<Schema>, SchemaError>;
    async fn find_latest(&self) -> Result<Option<Schema>, SchemaError>;
}
```

### Supporting Bounded Contexts

#### Resolver Context
Manages field resolution and data fetching:

```rust
pub struct ResolverContext {
    pub data_loaders: HashMap<String, Box<dyn DataLoader>>,
    pub user_context: Option<UserContext>,
    pub request_context: RequestContext,
}
```

#### Error Context
Handles error reporting and formatting:

```rust
pub struct GraphQLError {
    message: String,
    locations: Vec<SourceLocation>,
    path: Option<Vec<PathSegment>>,
    extensions: Option<ErrorExtensions>,
}
```

## Architecture Layers

### Domain Layer (src/domain)
- Pure domain logic
- No external dependencies
- Business rules and invariants

```text
domain/
├── entities/
│   ├── schema.rs
│   ├── query.rs
│   └── mod.rs
├── value_objects/
│   ├── types.rs
│   ├── selections.rs
│   └── mod.rs
├── services/
│   ├── validator.rs
│   ├── executor.rs
│   └── mod.rs
└── repositories/
    ├── schema_repository.rs
    └── mod.rs
```

### Application Layer (src/application)
- Use cases and application services
- Orchestrates domain operations
- Handles cross-cutting concerns

```text
application/
├── use_cases/
│   ├── execute_query.rs
│   ├── validate_schema.rs
│   └── mod.rs
├── services/
│   ├── schema_service.rs
│   ├── query_service.rs
│   └── mod.rs
└── dto/
    ├── query_request.rs
    ├── execution_result.rs
    └── mod.rs
```

### Infrastructure Layer (src/infrastructure)
- External concerns (HTTP, persistence)
- Framework-specific code
- Third-party integrations

```text
infrastructure/
├── http/
│   ├── handlers/
│   ├── middleware/
│   └── mod.rs
├── persistence/
│   ├── memory/
│   ├── redis/
│   └── mod.rs
└── resolvers/
    ├── user_resolver.rs
    ├── post_resolver.rs
    └── mod.rs
```

### Presentation Layer (src/presentation)
- API endpoints and serialization
- Request/response handling
- GraphQL over HTTP

```text
presentation/
├── graphql/
│   ├── schema.rs
│   ├── resolvers.rs
│   └── mod.rs
├── rest/
│   ├── health.rs
│   └── mod.rs
└── websocket/
    ├── subscriptions.rs
    └── mod.rs
```

## Domain Events

GraphQL operations can trigger domain events for cross-cutting concerns:

```rust
pub enum GraphQLEvent {
    QueryExecuted {
        query_id: QueryId,
        execution_time: Duration,
        field_count: usize,
    },
    SchemaUpdated {
        schema_id: SchemaId,
        version: SchemaVersion,
    },
    ValidationFailed {
        query_id: QueryId,
        errors: Vec<ValidationError>,
    },
}
```

## Design Patterns

### Repository Pattern
Abstract data access behind domain interfaces.

### Command Query Separation
Separate read (Query) and write (Mutation) operations.

### Specification Pattern
Encapsulate complex validation rules.

### Observer Pattern
Handle domain events and side effects.

This domain model provides a solid foundation for implementing GraphQL server features while maintaining clean separation of concerns and domain-focused design.
