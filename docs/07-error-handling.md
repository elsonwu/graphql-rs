# Error Handling in GraphQL: Visual Guide for Engineers and Newcomers

## 🧭 Overview

This guide explains how error handling works in our GraphQL server, with visual diagrams and practical examples for both engineers and GraphQL newcomers. It covers error propagation, error formatting, and best practices for robust APIs.

---

## 🚦 Error Handling Pipeline (Visual)

```text
┌──────────────────────────────────────────────────────────────┐
│                GraphQL Error Handling Pipeline               │
├──────────────────────────────────────────────────────────────┤
│ 1. Parse/Validation Error? ──┐                                │
│ 2. Execution Error?         ─┼─▶ Collect Error Info           │
│ 3. Resolver Error?          ─┘                                │
│      │                                                    │
│      ▼                                                    │
│  Attach: message, locations, path, extensions             │
│      │                                                    │
│      ▼                                                    │
│  Add to errors[] in response                              │
│      │                                                    │
│      ▼                                                    │
│  Return { data, errors, extensions }                      │
└──────────────────────────────────────────────────────────────┘
```

---

## 📦 Error Object Structure (Standard)

```json
{
  "message": "Field 'foo' not found on type 'Query'",
  "locations": [{ "line": 2, "column": 5 }],
  "path": ["query", "foo"],
  "extensions": { "code": "FIELD_NOT_FOUND" }
}
```

---

## 🏗️ Error Types in Our Server

- **Parse Errors**: Invalid GraphQL syntax
- **Validation Errors**: Schema/type/argument issues
- **Execution Errors**: Resolver panics, runtime failures
- **Field Errors**: Nullability violations, missing data
- **Custom Errors**: Business logic, authorization, etc.

---

## 🔍 Error Propagation (Visual)

```text
┌─────────────┐   ┌─────────────┐   ┌─────────────┐
│  Resolver   │─▶│  Field      │─▶│  Response   │
└─────────────┘   └─────────────┘   └─────────────┘
      │                │                │
      ▼                ▼                ▼
  Error? ─────────────▶ Null? ─────────▶ Add to errors[]
```

- If a non-nullable field errors, parent becomes null (bubble up)
- All errors are collected in the `errors` array

---

## 🛠️ Practical Example: Error Handling in Action

**Query:**
```graphql
{
  user(id: "notfound") {
    name
    email
  }
}
```

**If user not found:**
```json
{
  "data": { "user": null },
  "errors": [
    {
      "message": "User not found",
      "path": ["user"],
      "extensions": { "code": "NOT_FOUND" }
    }
  ]
}
```

---

## 🧑‍💻 Error Handling in Rust (Implementation)

```rust
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct GraphQLError {
    pub message: String,
    pub locations: Vec<SourceLocation>,
    pub path: Option<Vec<PathSegment>>,
    pub extensions: Option<serde_json::Map<String, serde_json::Value>>,
}

impl GraphQLError {
    pub fn new(message: String) -> Self { /* ... */ }
    pub fn with_location(mut self, line: u32, column: u32) -> Self { /* ... */ }
    pub fn with_path(mut self, path: Vec<PathSegment>) -> Self { /* ... */ }
    pub fn with_extension(mut self, key: &str, value: serde_json::Value) -> Self { /* ... */ }
}
```

---

## 🧪 Testing Error Handling

- Parse invalid queries and check for parse errors
- Validate queries with missing fields/types
- Simulate resolver panics and check error propagation
- Test nullability violations and error bubbling

---

## 🚀 Future Enhancements

- Add error codes for all error types
- Support for custom error extensions (e.g., auth, rate limit)
- Improved error path tracking for deeply nested fields
- Configurable error masking for production

---

## 📚 References
- [GraphQL Spec: Errors](https://spec.graphql.org/June2018/#sec-Errors)
- [Apollo Error Handling](https://www.apollographql.com/docs/apollo-server/data/errors/)
- [Rust Error Handling Patterns](https://doc.rust-lang.org/book/ch09-02-recoverable-errors-with-result.html)
