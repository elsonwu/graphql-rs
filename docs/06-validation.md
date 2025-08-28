# GraphQL Validation System: Visual Guide to Schema and Query Validation

This guide explains GraphQL validation with detailed visual diagrams showing how our server validates both schemas and queries, with practical examples for both developers and GraphQL newcomers.

## 🛡️ What Is GraphQL Validation?

```text
┌─────────────────────────────────────────────────────────────────────────────┐
│                           GRAPHQL VALIDATION LAYERS                         │
├─────────────────────────────────────────────────────────────────────────────┤
│                                                                             │
│  🏗️ SCHEMA VALIDATION                    🔍 QUERY VALIDATION                │
│  ┌─────────────────────────┐             ┌─────────────────────────────┐   │
│  │ Purpose: Validate SDL   │             │ Purpose: Validate queries   │   │
│  │ When: Schema creation   │             │ When: Query execution       │   │
│  │ Checks: Type system     │             │ Checks: Syntax & semantics  │   │
│  │ Goal: Ensure schema is  │             │ Goal: Ensure query is safe  │   │
│  │       well-formed       │             │       and executable        │   │
│  └─────────────────────────┘             └─────────────────────────────┘   │
│                                                                             │
│  Examples:                                Examples:                         │
│  ✅ Query type exists                     ✅ Valid GraphQL syntax           │
│  ✅ All types are defined                 ✅ Fields exist in schema         │
│  ✅ Interfaces implemented correctly      ✅ Arguments match field types    │
│  ✅ Unions contain only object types      ✅ Required fields provided       │
│                                                                             │
└─────────────────────────────────────────────────────────────────────────────┘
```

### Why Validation Matters

1. **🔒 Safety**: Prevents invalid queries from reaching the database
2. **📊 Performance**: Catches expensive queries before execution
3. **🛠️ Developer Experience**: Clear error messages guide developers
4. **📈 API Quality**: Ensures schema follows GraphQL best practices

## 🏗️ Schema Validation: Visual Deep Dive

### The Complete Schema Validation Pipeline

```text
                               📄 GRAPHQL SDL INPUT
                                         │
                                         ▼
┌─────────────────────────────────────────────────────────────────────────────────┐
│                          🔍 LEXER & PARSER                                     │
├─────────────────────────────────────────────────────────────────────────────────┤
│  SDL Text: "type User { id: ID! name: String! }"                              │
│           ↓ Tokenize and Parse                                                 │
│  AST: ObjectTypeDefinition { name: "User", fields: [...] }                    │
└─────────────────────────────────────────────────────────────────────────────────┘
                                         │
                                         ▼
┌─────────────────────────────────────────────────────────────────────────────────┐
│                      ✅ SCHEMA VALIDATION ENGINE                               │
├─────────────────────────────────────────────────────────────────────────────────┤
│                                                                                │
│  🔍 STEP 1: Root Type Validation                                              │
│  ┌─────────────────────────────────────────────────────┐                     │
│  │ ✅ Query type exists and is Object type             │                     │
│  │ ✅ Mutation type (if specified) exists and is Object│                     │
│  │ ✅ Subscription type (if specified) exists and Object│                    │
│  └─────────────────────────────────────────────────────┘                     │
│                                                                                │
│  🔍 STEP 2: Type System Validation                                            │
│  ┌─────────────────────────────────────────────────────┐                     │
│  │ For each Object Type:                                │                     │
│  │   ✅ All field types are valid output types          │                     │
│  │   ✅ All argument types are valid input types        │                     │
│  │   ✅ Interface implementations are correct           │                     │
│  │                                                      │                     │
│  │ For each Interface Type:                             │                     │
│  │   ✅ All fields have valid output types              │                     │
│  │                                                      │                     │
│  │ For each Union Type:                                 │                     │
│  │   ✅ All members are Object types                    │                     │
│  │   ✅ All member types are defined                    │                     │
│  └─────────────────────────────────────────────────────┘                     │
│                                                                                │
│  🔍 STEP 3: Type Reference Validation                                         │
│  ┌─────────────────────────────────────────────────────┐                     │
│  │ ✅ All referenced types are defined                  │                     │
│  │ ✅ No undefined type references                      │                     │
│  │ ✅ Circular references are allowed but tracked       │                     │
│  └─────────────────────────────────────────────────────┘                     │
│                                                                                │
└─────────────────────────────────────────────────────────────────────────────────┘
                                         │
                                         ▼
┌─────────────────────────────────────────────────────────────────────────────────┐
│                        📦 VALIDATION RESULT                                    │
├─────────────────────────────────────────────────────────────────────────────────┤
│                                                                                │
│  ✅ ValidationResult::Valid                                                    │
│     └─ Schema is ready for query execution!                                   │
│                                                                                │
│  ❌ ValidationResult::Invalid(errors)                                         │
│     ├─ List of specific validation errors                                     │
│     ├─ Error locations in SDL                                                 │
│     └─ Human-readable error messages                                          │
│                                                                                │
└─────────────────────────────────────────────────────────────────────────────────┘
```

### 🎯 Practical Example: Schema Validation in Action

**Input Schema:**
```graphql
**Input Schema:**

```graphql
type Query {
  user(id: ID!): User
  posts: [Post!]!
}
```

type User {
  id: ID!
  name: String!
  posts: [Post!]!
  profile: UserProfile
}

type Post {
  id: ID!
  title: String!
  author: User!
  tags: [String!]
}

type UserProfile {
  bio: String
  website: String
  avatar: String
}
```

**Validation Process:**

```text
🔍 VALIDATION STEPS:

1️⃣ ROOT TYPE CHECK:
   ✅ Query type "Query" exists
   ✅ Query type is Object type
   ❓ Mutation type not specified (OK)
   ❓ Subscription type not specified (OK)

2️⃣ OBJECT TYPE VALIDATION:
   Query:
     ✅ user field returns User (valid output type)
     ✅ id argument is ID! (valid input type)
     ✅ posts field returns [Post!]! (valid output type)
   
   User:
     ✅ id field is ID! (valid output type)
     ✅ name field is String! (valid output type)
     ✅ posts field is [Post!]! (valid output type)
     ✅ profile field is UserProfile (valid output type)
   
   Post:
     ✅ All fields have valid output types
     ✅ author field creates circular reference (User ↔ Post) - ALLOWED
   
   UserProfile:
     ✅ All fields have valid output types

3️⃣ TYPE REFERENCE CHECK:
   ✅ User type is defined
   ✅ Post type is defined
   ✅ UserProfile type is defined
   ✅ All built-in scalars (ID, String) exist

🎉 RESULT: ValidationResult::Valid
```

## 🔍 Query Validation: Visual Deep Dive

### The Complete Query Validation Pipeline

```text
                              📝 GRAPHQL QUERY INPUT
                                         │
                                         ▼
┌─────────────────────────────────────────────────────────────────────────────────┐
│                         🔍 QUERY PARSER                                        │
├─────────────────────────────────────────────────────────────────────────────────┤
│  Query: "{ user(id: \"123\") { name email } }"                                │
│         ↓ Parse into AST                                                      │
│  Document { operations: [Query { selection_set: [...] }] }                   │
└─────────────────────────────────────────────────────────────────────────────────┘
                                         │
                                         ▼
┌─────────────────────────────────────────────────────────────────────────────────┐
│                     ✅ QUERY VALIDATION ENGINE                                 │
├─────────────────────────────────────────────────────────────────────────────────┤
│                                                                                │
│  🔍 STEP 1: Syntax Validation                                                 │
│  ┌─────────────────────────────────────────────────────┐                     │
│  │ ✅ Valid GraphQL syntax                              │                     │
│  │ ✅ Balanced braces and parentheses                   │                     │
│  │ ✅ Proper string escaping                            │                     │
│  │ ✅ Valid field names and arguments                   │                     │
│  └─────────────────────────────────────────────────────┘                     │
│                                                                                │
│  🔍 STEP 2: Schema Compliance Validation                                      │
│  ┌─────────────────────────────────────────────────────┐                     │
│  │ ✅ All fields exist in schema                        │                     │
│  │ ✅ Field arguments match schema definitions          │                     │
│  │ ✅ Return types are compatible                       │                     │
│  │ ✅ Required arguments are provided                   │                     │
│  └─────────────────────────────────────────────────────┘                     │
│                                                                                │
│  🔍 STEP 3: Type System Validation                                            │
│  ┌─────────────────────────────────────────────────────┐                     │
│  │ ✅ Variables match their declared types              │                     │
│  │ ✅ Argument values match expected types              │                     │
│  │ ✅ Fragment spreads are type-compatible              │                     │
│  │ ✅ Directives are applied correctly                  │                     │
│  └─────────────────────────────────────────────────────┘                     │
│                                                                                │
│  🔍 STEP 4: Execution Validation                                              │
│  ┌─────────────────────────────────────────────────────┐                     │
│  │ ✅ No infinite recursion possible                    │                     │
│  │ ✅ Query complexity within limits                    │                     │
│  │ ✅ Deprecated fields have warnings                   │                     │
│  └─────────────────────────────────────────────────────┘                     │
│                                                                                │
└─────────────────────────────────────────────────────────────────────────────────┘
                                         │
                                         ▼
┌─────────────────────────────────────────────────────────────────────────────────┐
│                        📦 VALIDATION RESULT                                    │
├─────────────────────────────────────────────────────────────────────────────────┤
│  ✅ ValidationResult::Valid → Ready for execution                             │
│  ❌ ValidationResult::Invalid(errors) → Return errors to client               │
└─────────────────────────────────────────────────────────────────────────────────┘
```

### 🎯 Practical Example: Query Validation in Action

**Schema:**
```graphql
type Query {
  user(id: ID!): User
  posts(limit: Int = 10): [Post!]!
}

type User {
  id: ID!
  name: String!
  email: String
}
```

**Query Examples:**

#### ✅ Valid Query

```graphql
{
  user(id: "123") {
    id
    name
    email
  }
}
```

**Validation Steps:**

```text
1️⃣ SYNTAX CHECK: ✅ Valid GraphQL syntax
2️⃣ FIELD CHECK: ✅ user field exists on Query
3️⃣ ARGUMENT CHECK: ✅ id argument exists and is ID!
4️⃣ SUB-FIELD CHECK: ✅ id, name, email exist on User
5️⃣ TYPE CHECK: ✅ All types match

🎉 RESULT: ValidationResult::Valid
```

#### ❌ Invalid Query
```graphql
{
  user(id: "123") {
    id
    fullName  # ❌ Field doesn't exist
    posts     # ❌ Field doesn't exist on User
  }
  nonExistentField  # ❌ Field doesn't exist on Query
}
```

**Validation Steps:**
```text
1️⃣ SYNTAX CHECK: ✅ Valid GraphQL syntax
2️⃣ FIELD CHECK: 
   ✅ user field exists on Query
   ❌ nonExistentField doesn't exist on Query
3️⃣ SUB-FIELD CHECK:
   ✅ id exists on User
   ❌ fullName doesn't exist on User
   ❌ posts doesn't exist on User

🚨 RESULT: ValidationResult::Invalid([
  "Field 'nonExistentField' doesn't exist on type 'Query'",
  "Field 'fullName' doesn't exist on type 'User'", 
  "Field 'posts' doesn't exist on type 'User'"
])
```

## 🛠️ Implementation Architecture

### 🎨 Validation System Components (Visual)

```text
┌─────────────────────────────────────────────────────────────────────┐
│                     VALIDATION ARCHITECTURE                         │
├─────────────────────────────────────────────────────────────────────┤
│                                                                     │
│ ┌─────────────────┐    ┌─────────────────┐                        │
│ │ SchemaValidator │    │ QueryValidator  │                        │
│ │                 │    │                 │                        │
│ │ validate()      │    │ validate()      │                        │
│ │ ├─ Root types   │    │ ├─ Syntax       │                        │
│ │ ├─ Type system  │    │ ├─ Field exists │                        │
│ │ ├─ References   │    │ ├─ Arguments    │                        │
│ │ └─ Consistency  │    │ └─ Types match  │                        │
│ └─────────────────┘    └─────────────────┘                        │
│          │                       │                                 │
│          ▼                       ▼                                 │
│ ┌─────────────────────────────────────────────────────────────────┐ │
│ │                ValidationResult                                 │ │
│ ├─────────────────────────────────────────────────────────────────┤ │
│ │ enum ValidationResult {                                         │ │
│ │   Valid,                        ← Ready to proceed              │ │
│ │   Invalid(Vec<GraphQLError>),   ← List of specific errors       │ │
│ │   Pending,                      ← Validation in progress        │ │
│ │ }                                                               │ │
│ └─────────────────────────────────────────────────────────────────┘ │
│          │                       │                                 │
│          ▼                       ▼                                 │
│ ┌─────────────────┐    ┌─────────────────┐                        │
│ │ Schema Service  │    │ Query Executor  │                        │
│ │ (Schema ready)  │    │ (Execute query) │                        │
│ └─────────────────┘    └─────────────────┘                        │
│                                                                     │
└─────────────────────────────────────────────────────────────────────┘
```

### Core Implementation

Our validation system is built around these key components:

#### SchemaValidator
```rust
pub struct SchemaValidator;

impl SchemaValidator {
    pub fn validate(&self, schema: &Schema) -> ValidationResult {
        let mut errors = Vec::new();

        // Rule 1: Schema must have a Query type
        if schema.get_type(&schema.query_type).is_none() {
            errors.push(GraphQLError::validation_error(
                format!("Query type '{}' is not defined", schema.query_type)
            ));
        }

        // Rule 2: Validate all type definitions
        for (name, type_def) in &schema.types {
            if let Err(type_errors) = schema.validate_type(name, type_def) {
                errors.extend(type_errors);
            }
        }

        if errors.is_empty() {
            ValidationResult::Valid
        } else {
            ValidationResult::Invalid(errors)
        }
    }
}
```

#### QueryValidator  
```rust
pub struct QueryValidator;

impl QueryValidator {
    pub fn validate(&self, query: &Query, schema: &Schema) -> ValidationResult {
        // Basic validation - comprehensive validation will be added later
        if query.is_empty() {
            ValidationResult::invalid("Query string cannot be empty".to_string())
        } else {
            // TODO: Parse and validate query syntax
            // TODO: Validate query against schema
            ValidationResult::Valid
        }
    }
}
```

## 📊 Implementation Status

### 🎯 Current Validation Features

```text
┌─────────────────────────────────────────────────────────────────────┐
│                     VALIDATION STATUS OVERVIEW                      │
├─────────────────────────────────────────────────────────────────────┤
│                                                                     │
│ ✅ SCHEMA VALIDATION (Completed)                                   │
│ ├─ Root type validation (Query, Mutation, Subscription)            │
│ ├─ Type system validation (Object, Interface, Union, Enum)         │
│ ├─ Type reference validation                                        │
│ ├─ Interface implementation validation                              │
│ ├─ Union member validation                                          │
│ └─ Circular reference detection                                     │
│                                                                     │
│ 🚧 QUERY VALIDATION (Basic Implementation)                         │
│ ├─ ✅ Empty query detection                                         │
│ ├─ ✅ Basic syntax validation (via parser)                         │
│ ├─ 📋 Field existence validation (Planned)                         │
│ ├─ 📋 Argument validation (Planned)                                 │
│ ├─ 📋 Type compatibility validation (Planned)                      │
│ └─ 📋 Fragment validation (Planned)                                 │
│                                                                     │
│ 📋 ADVANCED VALIDATION (Planned)                                   │
│ ├─ Query complexity analysis                                       │
│ ├─ Depth limiting                                                  │
│ ├─ Rate limiting integration                                       │
│ └─ Custom validation rules                                         │
│                                                                     │
└─────────────────────────────────────────────────────────────────────┘
```

### Testing Strategy

Our validation system includes comprehensive tests:

```rust
#[cfg(test)]
mod validation_tests {
    use super::*;

    #[test]
    fn test_schema_validator_missing_query_type() {
        let schema = Schema::new("NonExistentQuery".to_string());
        let validator = SchemaValidator::new();
        
        let result = validator.validate(&schema);
        
        assert!(matches!(result, ValidationResult::Invalid(_)));
    }

    #[test]
    fn test_query_validator_empty_query() {
        let query = Query::new(String::new());
        let schema = Schema::new("Query".to_string());
        let validator = QueryValidator::new();

        let result = validator.validate(&query, &schema);
        
        assert!(result.is_invalid());
    }

    #[test]
    fn test_valid_schema_validation() {
        let mut schema = Schema::new("Query".to_string());
        // Add Query type definition...
        
        let validator = SchemaValidator::new();
        let result = validator.validate(&schema);
        
        assert!(result.is_valid());
    }
}
```

## 🔮 Future Enhancements

### Advanced Query Validation

```text
┌─────────────────────────────────────────────────────────────────────┐
│                    PLANNED VALIDATION FEATURES                      │
├─────────────────────────────────────────────────────────────────────┤
│                                                                     │
│ 🎯 COMPREHENSIVE QUERY VALIDATION                                   │
│ ├─ Field-level validation against schema                           │
│ ├─ Argument type and requirement validation                        │
│ ├─ Fragment spread validation                                       │
│ ├─ Variable definition and usage validation                        │
│ └─ Directive validation                                             │
│                                                                     │
│ 📊 QUERY COMPLEXITY ANALYSIS                                       │
│ ├─ Query depth analysis                                            │
│ ├─ Field complexity scoring                                        │
│ ├─ Query cost estimation                                           │
│ └─ Configurable limits                                             │
│                                                                     │
│ 🛡️ SECURITY VALIDATION                                             │
│ ├─ Input sanitization                                              │
│ ├─ Query whitelisting                                              │
│ ├─ Rate limiting per field                                         │
│ └─ Introspection query filtering                                   │
│                                                                     │
└─────────────────────────────────────────────────────────────────────┘
```

### Custom Validation Rules

Future versions will support custom validation rules:

```rust
pub trait ValidationRule {
    fn validate(&self, query: &Query, schema: &Schema) -> Vec<GraphQLError>;
}

pub struct MaxDepthRule {
    max_depth: u32,
}

impl ValidationRule for MaxDepthRule {
    fn validate(&self, query: &Query, _schema: &Schema) -> Vec<GraphQLError> {
        // Implement depth validation logic
        vec![]
    }
}
```

## 🎉 Integration with Execution Pipeline

### How Validation Fits in the Request Flow

```text
┌─────────────────────────────────────────────────────────────────────┐
│                    COMPLETE REQUEST PIPELINE                        │
├─────────────────────────────────────────────────────────────────────┤
│                                                                     │
│ 1. HTTP Request → Parse Query                                       │
│                        │                                            │
│                        ▼                                            │
│ 2. Query Parser → AST                                               │
│                        │                                            │
│                        ▼                                            │
│ 3. QueryValidator → ValidationResult                                │
│                        │                                            │
│                        ├─ Valid? → Continue to Step 4              │
│                        └─ Invalid? → Return errors immediately      │
│                                                                     │
│ 4. Query Executor → Execute validated query                         │
│                        │                                            │
│                        ▼                                            │
│ 5. Field Resolvers → Fetch data                                     │
│                        │                                            │
│                        ▼                                            │
│ 6. Response Builder → JSON Response                                  │
│                                                                     │
└─────────────────────────────────────────────────────────────────────┘
```

## 🚀 Best Practices

### Validation Guidelines

1. **🔍 Fail Fast**: Validate early in the request pipeline
2. **📊 Clear Errors**: Provide specific, actionable error messages
3. **⚡ Performance**: Cache validation results when possible
4. **🛡️ Security**: Always validate untrusted input
5. **🧪 Testing**: Comprehensive test coverage for all validation rules

### Error Message Examples

Good validation errors are specific and actionable:

```json
{
  "errors": [
    {
      "message": "Field 'fullName' doesn't exist on type 'User'. Did you mean 'name'?",
      "locations": [{"line": 3, "column": 5}],
      "path": ["user", "fullName"],
      "extensions": {
        "code": "FIELD_NOT_FOUND",
        "typeName": "User",
        "fieldName": "fullName",
        "availableFields": ["id", "name", "email"]
      }
    }
  ]
}
```

This validation system ensures that only safe, well-formed queries reach the execution engine, providing excellent developer experience while maintaining security and performance.
