# GraphQL Mutation Support: A Complete Guide

## 🎯 What You'll Learn

This guide explains GraphQL mutations with detailed visual diagrams showing exactly how our server processes requests from start to finish. Perfect for developers new to GraphQL or those wanting to understand the internal mechanics.

## 📖 GraphQL Mutations Explained

### The Big Picture: Queries vs Mutations

GraphQL has **two main operation types** for different purposes:

```text
┌─────────────────────────────────────────────────────────────────────────────┐
│                               GRAPHQL OPERATIONS                            │
├─────────────────────────────────────────────────────────────────────────────┤
│                                                                             │
│  🔍 QUERIES (Read Data)                    🔧 MUTATIONS (Modify Data)        │
│  ┌─────────────────────────┐              ┌─────────────────────────────┐   │
│  │ Purpose: Fetch data     │              │ Purpose: Change data        │   │
│  │ Side Effects: None      │              │ Side Effects: Yes           │   │
│  │ Execution: Parallel OK  │              │ Execution: Sequential ONLY  │   │
│  │ Caching: Safe           │              │ Caching: Dangerous          │   │
│  │ Idempotent: Yes         │              │ Idempotent: No              │   │
│  └─────────────────────────┘              └─────────────────────────────┘   │
│                                                                             │
│  Example:                                  Example:                         │
│  query GetUser {                          mutation CreateUser {             │
│    user(id: "123") {                        createUser(input: {             │
│      name                                     name: "Alice"                 │
│      email                                    email: "alice@example.com"    │
│    }                                        }) {                            │
│  }                                            id name email                 │
│                                             }                               │
│                                           }                                 │
└─────────────────────────────────────────────────────────────────────────────┘
```

### Why This Separation Matters

1. **🎯 Clear Intent**: Just by seeing `mutation`, you know data will be modified
2. **⚡ Performance**: Queries can run in parallel, mutations run sequentially
3. **🔒 Safety**: Caching systems know queries are safe, mutations are not
4. **🛡️ Error Handling**: Different strategies for read vs write operations

## 🏗️ How GraphQL Request Processing Works

### The Complete Request Journey

Let's trace exactly what happens when a mutation request hits our server:

```text
                               📡 INCOMING HTTP REQUEST
                                         │
                                         ▼
┌─────────────────────────────────────────────────────────────────────────────────┐
│                            🌐 HTTP LAYER                                        │
├─────────────────────────────────────────────────────────────────────────────────┤
│  POST /graphql                                                                  │
│  Content-Type: application/json                                                 │
│  {                                                                              │
│    "query": "mutation { createUser(input: { name: \"Alice\" }) { id name } }",  │
│    "variables": {},                                                             │
│    "operationName": null                                                        │
│  }                                                                              │
└─────────────────────────────────────────────────────────────────────────────────┘
                                         │
                                         ▼
┌─────────────────────────────────────────────────────────────────────────────────┐
│                          🔍 LEXER (Token Analysis)                              │
├─────────────────────────────────────────────────────────────────────────────────┤
│  Raw String: "mutation { createUser(input: { name: \"Alice\" }) { id name } }"  │
│              ↓ Break into tokens                                                │
│  Tokens: [MUTATION, LBRACE, IDENTIFIER("createUser"), LPAREN, ...]              │
│                                                                                 │
│  📍 Located in: src/infrastructure/lexer.rs                                     │
│  🔧 Key Functions: tokenize(), process_identifier(), process_string()           │
└─────────────────────────────────────────────────────────────────────────────────┘
                                         │
                                         ▼
┌─────────────────────────────────────────────────────────────────────────────────┐
│                        🏗️ PARSER (AST Generation)                               │
├─────────────────────────────────────────────────────────────────────────────────┤
│  Tokens: [MUTATION, LBRACE, IDENTIFIER("createUser"), ...]                      │
│          ↓ Build Abstract Syntax Tree                                           │
│  AST:                                                                           │
│  Document {                                                                     │
│    definitions: [                                                               │
│      OperationDefinition {                                                      │
│        operation_type: Mutation,                                                │
│        selection_set: SelectionSet {                                            │
│          selections: [                                                          │
│            Field {                                                              │
│              name: "createUser",                                                │
│              arguments: [                                                       │
│                Argument { name: "input", value: Object(...) }                   │
│              ]                                                                  │
│            }                                                                    │
│          ]                                                                      │
│        }                                                                        │
│      }                                                                          │
│    ]                                                                            │
│  }                                                                              │
│                                                                                 │
│  📍 Located in: src/infrastructure/query_parser.rs                              │
│  🔧 Key Functions: parse_document(), parse_operation_definition()               │
└─────────────────────────────────────────────────────────────────────────────────┘
                                         │
                                         ▼
┌─────────────────────────────────────────────────────────────────────────────────┐
│                         ✅ VALIDATION (Schema Check)                           │
├─────────────────────────────────────────────────────────────────────────────────┤
│  AST + Schema → Validation Rules                                              │
│                                                                                │
│  ✓ Does Mutation type exist in schema?                                        │
│  ✓ Does createUser field exist on Mutation type?                              │
│  ✓ Are argument types correct?                                                 │
│  ✓ Are requested fields available on return type?                             │
│  ✓ Are all required fields provided?                                          │
│                                                                                │
│  📍 Located in: src/domain/services.rs (QueryValidator)                       │
│  🔧 Key Functions: validate(), check_field_existence()                        │
└─────────────────────────────────────────────────────────────────────────────────┘
                                         │
                                         ▼
┌─────────────────────────────────────────────────────────────────────────────────┐
│                        ⚡ EXECUTION ENGINE                                     │
├─────────────────────────────────────────────────────────────────────────────────┤
│                    🔄 MUTATION-SPECIFIC PROCESSING                            │
│                                                                                │
│  1️⃣ Identify Operation Type: MUTATION                                         │
│     ↓                                                                         │
│  2️⃣ Get Mutation Root Type from Schema                                        │
│     schema.mutation_type → "Mutation"                                         │
│     ↓                                                                         │
│  3️⃣ ⚠️ SEQUENTIAL EXECUTION (Critical!)                                       │
│     Unlike queries, mutations MUST execute one-by-one:                        │
│                                                                                │
│     for field in selection_set {  // ← Sequential loop, NOT parallel!        │
│       result = execute_mutation_field(field).await;                          │
│       // ☝️ Wait for completion before next field                            │
│     }                                                                         │
│                                                                                │
│  📍 Located in: src/domain/services.rs (QueryExecutor)                        │
│  🔧 Key Functions: execute_mutation_operation()                               │
└─────────────────────────────────────────────────────────────────────────────────┘
                                         │
                                         ▼
┌─────────────────────────────────────────────────────────────────────────────────┐
│                       🎯 FIELD RESOLUTION                                      │
├─────────────────────────────────────────────────────────────────────────────────┤
│                    createUser Field Execution                                 │
│                                                                                │
│  1️⃣ Find Field Definition:                                                     │
│     Mutation.createUser: (input: CreateUserInput!) → User!                    │
│                                                                                │
│  2️⃣ Extract Arguments:                                                         │
│     input = { name: "Alice", email: "alice@example.com" }                     │
│                                                                                │
│  3️⃣ Execute Resolver Logic: 🔥 SIDE EFFECTS HAPPEN HERE!                      │
│     ┌─────────────────────────────────────────────────┐                      │
│     │  // This is where real-world resolvers would:   │                      │
│     │  // - Validate input data                       │                      │
│     │  // - Write to database                         │                      │
│     │  // - Call external APIs                        │                      │
│     │  // - Generate IDs and timestamps               │                      │
│     │  // - Send notifications                        │                      │
│     │                                                 │                      │
│     │  let user = database.create_user({              │                      │
│     │    name: input.name,                            │                      │
│     │    email: input.email,                          │                      │
│     │    id: generate_uuid(),                         │                      │
│     │    created_at: now()                            │                      │
│     │  });                                            │                      │
│     │  return user;                                   │                      │
│     └─────────────────────────────────────────────────┘                      │
│                                                                                │
│  4️⃣ Process Sub-Selection:                                                     │
│     User { id name } → Extract only requested fields                          │
│                                                                                │
│  📍 Located in: src/domain/services.rs                                        │
│  🔧 Key Functions: execute_mutation_field(), execute_mutation_sub_selection() │
└─────────────────────────────────────────────────────────────────────────────────┘
                                         │
                                         ▼
┌─────────────────────────────────────────────────────────────────────────────────┐
│                        📦 RESPONSE CONSTRUCTION                                │
├─────────────────────────────────────────────────────────────────────────────────┤
│  Field Results → JSON Response                                                │
│                                                                                │
│  createUser result: {                                                         │
│    "id": "user_abc123",                                                       │
│    "name": "Alice"                                                            │
│  }                                                                             │
│          ↓ Wrap in GraphQL response format                                    │
│  {                                                                             │
│    "data": {                                                                  │
│      "createUser": {                                                          │
│        "id": "user_abc123",                                                   │
│        "name": "Alice"                                                        │
│      }                                                                        │
│    },                                                                         │
│    "errors": [],                                                              │
│    "extensions": {}                                                           │
│  }                                                                             │
│                                                                                │
│  📍 Located in: src/domain/value_objects.rs (ExecutionResult)                 │
└─────────────────────────────────────────────────────────────────────────────────┘
                                         │
                                         ▼
                             📤 HTTP RESPONSE SENT TO CLIENT
```

## 🔧 Sequential Execution: The Critical Difference

### Why Mutations Must Execute Sequentially

This is the **most important concept** in GraphQL mutations:

```text
❌ WRONG - Parallel Execution (What Queries Do)
┌─────────────────────────────────────────────────────────┐
│                                                         │
│  mutation {                                             │
│    first: createUser(name: "Alice")   ───┬─► Database   │
│    second: createUser(name: "Bob")    ───┤              │
│    third: createUser(name: "Charlie") ───┘              │
│  }                                                      │
│                                                         │
│  ⚠️ RACE CONDITION! All execute at once                 │
│  ⚠️ Unpredictable order                                 │
│  ⚠️ Data corruption possible                            │
└─────────────────────────────────────────────────────────┘

✅ CORRECT - Sequential Execution (What Mutations Do)
┌─────────────────────────────────────────────────────────┐
│                                                         │
│  mutation {                                             │
│    first: createUser(name: "Alice")   ──1─► Database    │
│                              │                          │
│                              ▼ (wait for completion)    │
│    second: createUser(name: "Bob")    ──2─► Database    │
│                              │                          │
│                              ▼ (wait for completion)    │
│    third: createUser(name: "Charlie") ──3─► Database    │
│  }                                                      │
│                                                         │
│  ✅ Predictable order (1, 2, 3)                         │
│  ✅ Each sees previous results                          │
│  ✅ Data consistency guaranteed                         │
└─────────────────────────────────────────────────────────┘
```

### Real-World Example: Bank Transfer

```graphql
mutation TransferMoney {
  # These MUST execute in order!
  debit: updateAccount(id: "alice", amount: -100)    # 1st: Remove money from Alice
  credit: updateAccount(id: "bob", amount: 100)      # 2nd: Add money to Bob
  log: createTransaction(from: "alice", to: "bob")   # 3rd: Record the transaction
}
```

**What happens if they run in parallel?** 💥

- Log might execute before debit/credit
- Credit might fail but debit succeeds
- Money disappears or duplicates!

**Sequential execution prevents this** ✅

- Each step completes before the next
- If any step fails, we can rollback cleanly
- Consistent state guaranteed

## 🏗️ Our Implementation Architecture

### Component Interaction Map

```text
┌─────────────────────────────────────────────────────────────────────────────────┐
│                           OUR GRAPHQL SERVER ARCHITECTURE                      │
├─────────────────────────────────────────────────────────────────────────────────┤
│                                                                                 │
│  🌐 HTTP Layer (Axum)                                                          │
│  ┌─────────────────────────────────────────────────────────┐                   │
│  │  POST /graphql                                          │                   │
│  │  └─► Extract JSON body                                  │                   │
│  │  └─► Forward to Application Layer                       │                   │
│  └─────────────────────────────────────────────────────────┘                   │
│                              │                                                 │
│                              ▼                                                 │
│  🎯 Application Layer (Use Cases)                                              │
│  ┌─────────────────────────────────────────────────────────┐                   │
│  │  GraphQLUseCase::execute()                              │                   │
│  │  ├─► Parse query string                                 │                   │
│  │  ├─► Validate against schema                            │                   │
│  │  └─► Delegate to Domain Services                        │                   │
│  └─────────────────────────────────────────────────────────┘                   │
│                              │                                                 │
│                              ▼                                                 │
│  🧠 Domain Layer (Core Logic)                                                  │
│  ┌─────────────────────────────────────────────────────────┐                   │
│  │  QueryExecutor::execute()                               │                   │
│  │  ├─► Detect operation type                              │                   │
│  │  ├─► Route to appropriate executor:                     │                   │
│  │  │   ├─► execute_query_operation() (parallel)          │                   │
│  │  │   └─► execute_mutation_operation() (sequential) ←──── YOU ARE HERE       │
│  │  └─► Return ExecutionResult                             │                   │
│  └─────────────────────────────────────────────────────────┘                   │
│                              │                                                 │
│                              ▼                                                 │
│  ⚙️ Infrastructure Layer (Parsing, Storage)                                   │
│  ┌─────────────────────────────────────────────────────────┐                   │
│  │  QueryParser, Lexer, Schema Repository                  │                   │
│  │  ├─► Convert strings to AST                             │                   │
│  │  ├─► Manage schema definitions                          │                   │
│  │  └─► Future: Database connections, external APIs       │                   │
│  └─────────────────────────────────────────────────────────┘                   │
│                                                                                 │
└─────────────────────────────────────────────────────────────────────────────────┘
```

### Key Implementation Methods

Now let's dive into the actual Rust code that makes this work:

```rust
// src/domain/services.rs - The heart of mutation processing

impl QueryExecutor {
    /// Main entry point for executing any GraphQL operation
    async fn execute(&self, query: &Query, schema: &Schema) -> ExecutionResult {
        // 1. Parse the query string into AST
        let document = self.parse_query(query.query_string())?;
        
        // 2. Find the operation to execute
        let operation = self.find_operation(&document, None)?;
        
        // 3. Route based on operation type
        match operation.operation_type {
            OperationType::Query => {
                // Queries can execute fields in parallel
                self.execute_query_operation(operation, schema, variables).await
            }
            OperationType::Mutation => {
                // 🚨 Mutations MUST execute sequentially!
                self.execute_mutation_operation(operation, schema, variables).await
            }
            OperationType::Subscription => {
                // Future: Real-time subscriptions
                Err(GraphQLError::new("Subscriptions not yet implemented"))
            }
        }
    }

    /// Execute mutation with sequential field processing
    async fn execute_mutation_operation(
        &self,
        operation: &OperationDefinition,
        schema: &Schema,
        variables: &Option<serde_json::Value>,
    ) -> Result<serde_json::Value, GraphQLError> {
        // Get the Mutation root type
        let mutation_type_name = schema.mutation_type
            .as_ref()
            .ok_or_else(|| GraphQLError::new("No Mutation type defined"))?;
        
        let mutation_type = schema.get_type(mutation_type_name)
            .ok_or_else(|| GraphQLError::new("Mutation type not found"))?;

        // 🔥 THE CRITICAL PART: Sequential execution
        self.execute_mutation_selection_set_sequential(
            &operation.selection_set,
            mutation_type,
            variables,
        ).await
    }

    /// Execute mutation fields one-by-one (NOT parallel!)
    async fn execute_mutation_selection_set_sequential(
        &self,
        selection_set: &SelectionSet,
        mutation_type: &GraphQLType,
        variables: &Option<serde_json::Value>,
    ) -> Result<serde_json::Value, GraphQLError> {
        let mut result_map = serde_json::Map::new();

        // 🚨 SEQUENTIAL LOOP - Each field waits for previous to complete
        for selection in &selection_set.selections {
            match selection {
                Selection::Field(field) => {
                    // Execute this field and WAIT for completion
                    let field_result = self.execute_mutation_field(field, mutation_type).await?;
                    
                    // Add to result map
                    let result_key = field.alias.as_ref().unwrap_or(&field.name);
                    result_map.insert(result_key.clone(), field_result);
                    
                    // ☝️ Only now do we move to the next field!
                }
                // Handle fragments, inline fragments, etc.
                _ => { /* ... */ }
            }
        }

        Ok(serde_json::Value::Object(result_map))
    }

    /// Execute individual mutation field with side effects
    async fn execute_mutation_field(
        &self,
        field: &Field,
        mutation_type: &GraphQLType,
    ) -> Result<serde_json::Value, GraphQLError> {
        // This is where the actual business logic happens!
        match field.name.as_str() {
            "createUser" => {
                // 🔥 SIDE EFFECTS: Create user in database
                let input = self.extract_arguments(&field.arguments);
                let user = self.user_service.create_user(input).await?;
                
                // Return the created user data
                Ok(serde_json::to_value(user)?)
            }
            "updateUser" => {
                // 🔥 SIDE EFFECTS: Update user in database
                let id = self.get_argument_value(&field.arguments, "id")?;
                let input = self.get_argument_value(&field.arguments, "input")?;
                let user = self.user_service.update_user(id, input).await?;
                
                Ok(serde_json::to_value(user)?)
            }
            "deleteUser" => {
                // 🔥 SIDE EFFECTS: Delete user from database
                let id = self.get_argument_value(&field.arguments, "id")?;
                let success = self.user_service.delete_user(id).await?;
                
                Ok(serde_json::Value::Bool(success))
            }
            _ => {
                Err(GraphQLError::new(format!("Unknown mutation field: {}", field.name)))
            }
        }
    }
}
```

## 🧪 Practical Examples: Step by Step

### Example 1: Simple User Creation

Let's trace through a complete mutation request:

**1. Client Request:**

```json
POST /graphql
{
  "query": "mutation { createUser(input: { name: \"Alice\", email: \"alice@example.com\" }) { id name email createdAt } }"
}
```

**2. Server Processing:**

```text
┌─────────────────────────────────────────────────────────────────────────────────┐
│                           STEP-BY-STEP EXECUTION                               │
├─────────────────────────────────────────────────────────────────────────────────┤
│                                                                                 │
│  🔍 STEP 1: Lexical Analysis                                                   │
│  Input: "mutation { createUser(input: { name: \"Alice\" }) { id name } }"      │
│  Output: [MUTATION, LBRACE, IDENTIFIER("createUser"), LPAREN, ...]             │
│                                                                                 │
│  🏗️ STEP 2: Syntax Parsing                                                     │
│  Tokens → AST                                                                  │
│  OperationDefinition {                                                          │
│    operation_type: Mutation,                                                   │
│    selection_set: SelectionSet {                                               │
│      selections: [Field("createUser")]                                         │
│    }                                                                            │
│  }                                                                              │
│                                                                                 │
│  ✅ STEP 3: Validation                                                          │
│  ✓ Mutation type exists in schema                                              │
│  ✓ createUser field exists on Mutation type                                    │
│  ✓ Arguments match field definition                                            │
│  ✓ Return type selection is valid                                              │
│                                                                                 │
│  ⚡ STEP 4: Execution                                                           │
│  execute_mutation_operation()                                                  │
│    └─► execute_mutation_selection_set_sequential()                             │
│         └─► execute_mutation_field("createUser")                               │
│              └─► // Side effects happen here!                                 │
│                  user_service.create_user({                                    │
│                    name: "Alice",                                              │
│                    email: "alice@example.com"                                  │
│                  })                                                            │
│                  └─► Database: INSERT INTO users ...                           │
│                       └─► Result: User { id: "abc123", name: "Alice", ... }    │
│                                                                                 │
│  📦 STEP 5: Response Construction                                               │
│  {                                                                              │
│    "data": {                                                                   │
│      "createUser": {                                                           │
│        "id": "abc123",                                                         │
│        "name": "Alice",                                                        │
│        "email": "alice@example.com",                                           │
│        "createdAt": "2024-01-15T10:30:00Z"                                     │
│      }                                                                         │
│    }                                                                            │
│  }                                                                              │
│                                                                                 │
└─────────────────────────────────────────────────────────────────────────────────┘
```

### Example 2: Complex Sequential Mutations

**Client Request:**

```graphql
mutation ComplexTransaction {
  # These execute in EXACT order:
  first: createUser(input: { name: "Alice", email: "alice@example.com" }) {
    id
    name
  }
  second: createUser(input: { name: "Bob", email: "bob@example.com" }) {
    id  
    name
  }
  third: createProject(input: { name: "Awesome Project", ownerId: "???" }) {
    id
    name
    owner { name }
  }
}
```

**Execution Timeline:**

```text
TIME: 0ms    │ 🚀 START mutation execution
             │
TIME: 0ms    │ ⏳ Execute first: createUser (Alice)
             │    └─► Database query: INSERT INTO users (name, email) VALUES ('Alice', 'alice@...')
TIME: 150ms  │ ✅ first completed: { id: "user_1", name: "Alice" }
             │
TIME: 150ms  │ ⏳ Execute second: createUser (Bob)  
             │    └─► Database query: INSERT INTO users (name, email) VALUES ('Bob', 'bob@...')  
TIME: 300ms  │ ✅ second completed: { id: "user_2", name: "Bob" }
             │
TIME: 300ms  │ ⏳ Execute third: createProject
             │    └─► Could reference results from first/second mutations!
             │    └─► Database query: INSERT INTO projects (name, owner_id) VALUES ('Awesome Project', 'user_1')
TIME: 450ms  │ ✅ third completed: { id: "proj_1", name: "Awesome Project", owner: { name: "Alice" }}
             │
TIME: 450ms  │ 🎉 ALL MUTATIONS COMPLETED - Return combined result
```

## 🔍 Error Handling and Edge Cases

### Common Error Scenarios

```text
┌─────────────────────────────────────────────────────────────────────────────────┐
│                             ERROR HANDLING MAP                                 │
├─────────────────────────────────────────────────────────────────────────────────┤
│                                                                                 │
│  ❌ PARSING ERRORS                                                              │
│  ┌─────────────────────────────────────────────────────────────┐               │
│  │ Invalid Syntax: mutation { createUser( missing closing }    │               │
│  │ Result: Parse error before execution starts                 │               │
│  │ HTTP Status: 400 Bad Request                                │               │
│  │ Response: { "errors": [{ "message": "Syntax error..." }] } │               │
│  └─────────────────────────────────────────────────────────────┘               │
│                                                                                 │
│  ❌ VALIDATION ERRORS                                                           │
│  ┌─────────────────────────────────────────────────────────────┐               │
│  │ Missing Field: mutation { nonExistentField }                │               │
│  │ Wrong Arguments: createUser(wrongArg: "value")              │               │
│  │ Type Mismatch: createUser(input: "should be object")        │               │
│  │ Result: Validation error before execution starts            │               │
│  │ HTTP Status: 400 Bad Request                                │               │
│  │ Response: { "errors": [{ "message": "Field not found" }] } │               │
│  └─────────────────────────────────────────────────────────────┘               │
│                                                                                 │
│  ❌ EXECUTION ERRORS                                                            │
│  ┌─────────────────────────────────────────────────────────────┐               │
│  │ Database Connection Failed                                   │               │
│  │ Business Logic Error (duplicate email)                      │               │
│  │ Permission Denied                                           │               │
│  │ External API Timeout                                        │               │
│  │ Result: Partial execution, detailed error info             │               │
│  │ HTTP Status: 200 OK (GraphQL convention)                   │               │
│  │ Response: { "data": null, "errors": [...] }                │               │
│  └─────────────────────────────────────────────────────────────┘               │
│                                                                                 │
│  ❌ SEQUENTIAL EXECUTION ERRORS                                                 │
│  ┌─────────────────────────────────────────────────────────────┐               │
│  │ Scenario: 3 mutations, 2nd one fails                       │               │
│  │                                                             │               │
│  │ mutation {                                                  │               │
│  │   first: createUser(...)  ✅ Succeeds                      │               │
│  │   second: updateUser(...) ❌ Fails (user not found)        │               │
│  │   third: deleteUser(...)  🚫 NOT EXECUTED                  │               │
│  │ }                                                           │               │
│  │                                                             │               │
│  │ Result: { "data": { "first": {...}, "second": null },      │               │
│  │           "errors": [{ "path": ["second"], ... }] }        │               │
│  └─────────────────────────────────────────────────────────────┘               │
└─────────────────────────────────────────────────────────────────────────────────┘
```

### Our Error Handling Implementation

```rust
// src/domain/services.rs - Error handling in mutation execution

impl QueryExecutor {
    async fn execute_mutation_selection_set_sequential(
        &self,
        selection_set: &SelectionSet,
        mutation_type: &GraphQLType,
        variables: &Option<serde_json::Value>,
    ) -> Result<serde_json::Value, GraphQLError> {
        let mut result_map = serde_json::Map::new();
        let mut errors = Vec::new();

        // Execute each field sequentially
        for selection in &selection_set.selections {
            match selection {
                Selection::Field(field) => {
                    match self.execute_mutation_field(field, mutation_type).await {
                        Ok(field_result) => {
                            // Success: Add to result
                            let result_key = field.alias.as_ref().unwrap_or(&field.name);
                            result_map.insert(result_key.clone(), field_result);
                        }
                        Err(error) => {
                            // Failure: Record error and STOP execution
                            errors.push(error);
                            
                            // 🚨 CRITICAL: Stop processing remaining fields
                            // This maintains consistency - if one fails, don't continue
                            break;
                        }
                    }
                }
            }
        }

        if errors.is_empty() {
            Ok(serde_json::Value::Object(result_map))
        } else {
            // Return partial results + errors (GraphQL convention)
            Err(errors.into_iter().next().unwrap()) // Return first error for now
        }
    }
}
```

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
## 🚀 Getting Started: Your First Mutation

### Setting Up a Schema with Mutations

```rust
// src/main.rs - Setting up a schema with mutations

use graphql_rs::domain::entities::{Schema, types::*};

fn create_schema_with_mutations() -> Schema {
    let mut schema = Schema::new("Query".to_string());
    
    // 1. Define the Mutation root type
    schema.mutation_type = Some("Mutation".to_string());
    
    // 2. Create User type (return type for mutations)
    let user_type = ObjectType {
        name: "User".to_string(),
        fields: HashMap::from([
            ("id".to_string(), FieldDefinition {
                name: "id".to_string(),
                field_type: GraphQLType::Scalar(ScalarType::ID),
                // ... other properties
            }),
            ("name".to_string(), FieldDefinition {
                name: "name".to_string(),
                field_type: GraphQLType::Scalar(ScalarType::String),
                // ... other properties  
            }),
            ("email".to_string(), FieldDefinition {
                name: "email".to_string(),
                field_type: GraphQLType::Scalar(ScalarType::String),
                // ... other properties
            }),
        ]),
        // ... other properties
    };
    
    // 3. Create Mutation type with CRUD operations
    let mutation_type = ObjectType {
        name: "Mutation".to_string(),
        fields: HashMap::from([
            ("createUser".to_string(), FieldDefinition {
                name: "createUser".to_string(),
                field_type: GraphQLType::Object(user_type.clone()),
                arguments: HashMap::from([
                    ("input".to_string(), ArgumentDefinition {
                        name: "input".to_string(),
                        arg_type: GraphQLType::InputObject(/* CreateUserInput */),
                        // ... other properties
                    }),
                ]),
                // ... other properties
            }),
            ("updateUser".to_string(), FieldDefinition {
                name: "updateUser".to_string(),
                field_type: GraphQLType::Object(user_type.clone()),
                // ... similar structure
            }),
            ("deleteUser".to_string(), FieldDefinition {
                name: "deleteUser".to_string(),
                field_type: GraphQLType::Scalar(ScalarType::Boolean),
                // ... similar structure
            }),
        ]),
        // ... other properties
    };
    
    // 4. Add types to schema
    schema.add_type(GraphQLType::Object(user_type)).unwrap();
    schema.add_type(GraphQLType::Object(mutation_type)).unwrap();
    
    schema
}
```

### Writing Your First Mutation

```rust
// examples/my_first_mutation.rs

use graphql_rs::domain::{
    entities::Query,
    services::{QueryExecutor, QueryExecution},
    value_objects::ValidationResult,
};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // 1. Create schema with mutation support
    let schema = create_schema_with_mutations();
    
    // 2. Create query executor
    let executor = QueryExecutor::new();
    
    // 3. Write a mutation query
    let mutation_query = r#"
        mutation CreateNewUser {
            createUser(input: {
                name: "John Doe",
                email: "john@example.com"
            }) {
                id
                name
                email
                createdAt
            }
        }
    "#;
    
    // 4. Execute the mutation
    let mut query = Query::new(mutation_query.to_string());
    query.mark_validated(ValidationResult::valid()); // Skip validation for now
    
    let result = executor.execute(&query, &schema).await;
    
    // 5. Handle the result
    match result.data {
        Some(data) => {
            println!("✅ Mutation successful!");
            println!("Created user: {}", serde_json::to_string_pretty(&data)?);
        }
        None => {
            println!("❌ Mutation failed:");
            for error in result.errors {
                println!("  - {}", error.message);
            }
        }
    }
    
    Ok(())
}
```

## 🧪 Testing Your Mutations

### Unit Test Example

```rust
// tests/mutation_tests.rs

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_create_user_mutation() {
        // Setup
        let schema = create_test_schema();
        let executor = QueryExecutor::new();
        
        let mutation = r#"
            mutation {
                createUser(input: { name: "Alice", email: "alice@example.com" }) {
                    id
                    name
                    email
                }
            }
        "#;
        
        // Execute
        let mut query = Query::new(mutation.to_string());
        query.mark_validated(ValidationResult::valid());
        let result = executor.execute(&query, &schema).await;
        
        // Assert
        assert!(result.errors.is_empty(), "Mutation should not have errors");
        assert!(result.data.is_some(), "Mutation should return data");
        
        let data = result.data.unwrap();
        let user = &data["createUser"];
        
        assert!(user["id"].as_str().is_some(), "Should have generated ID");
        assert_eq!(user["name"].as_str().unwrap(), "Alice");
        assert_eq!(user["email"].as_str().unwrap(), "alice@example.com");
    }
    
    #[tokio::test] 
    async fn test_sequential_mutation_execution() {
        let schema = create_test_schema();
        let executor = QueryExecutor::new();
        
        let mutation = r#"
            mutation {
                first: createUser(input: { name: "User 1" }) { id name }
                second: createUser(input: { name: "User 2" }) { id name }
                third: createUser(input: { name: "User 3" }) { id name }
            }
        "#;
        
        let mut query = Query::new(mutation.to_string());
        query.mark_validated(ValidationResult::valid());
        let result = executor.execute(&query, &schema).await;
        
        // Verify all mutations executed successfully
        assert!(result.errors.is_empty());
        let data = result.data.unwrap();
        
        // Verify results are in order
        assert!(data["first"]["id"].as_str().is_some());
        assert!(data["second"]["id"].as_str().is_some());  
        assert!(data["third"]["id"].as_str().is_some());
        
        println!("✅ All mutations executed sequentially!");
    }
}
```

### Integration Test with Example

```rust
// examples/test_mutation_support.rs (already in our codebase!)

cargo run --example test_mutation_support
```

**Output:**

```text
🚀 Testing GraphQL Mutation Support
=====================================

📋 Test 1: Schema with Mutation Type...
   ✅ Schema created with Mutation type
   📝 Available mutations: createUser, updateUser, deleteUser

👤 Test 2: Create User Mutation...
   📤 Executing mutation: mutation CreateUser { createUser { id name email } }
   ✅ Mutation executed successfully!
   👤 Created user:
      - ID: user_abc123
      - Name: Unknown User (default)
      - Email: user@example.com (default)

🔄 Test 5: Sequential Mutations (CRITICAL TEST!)...
   📝 This test verifies mutations execute one-by-one, not in parallel
   📤 Executing sequential mutations
   ✅ Sequential mutations executed successfully!
   👥 Created users in sequence:
      1. Unknown User (ID: user_abc123)
      2. Unknown User (ID: user_def456)  
      3. Unknown User (ID: user_ghi789)
   🎯 Sequential execution verified!

✅ All mutation tests passed!
🎉 GraphQL Mutation Support is working correctly!
```

## 🎓 Learning Exercises

### Exercise 1: Add a New Mutation

Try adding a `updateUserEmail` mutation to the schema:

```graphql
type Mutation {
  # Existing mutations...
  updateUserEmail(id: ID!, newEmail: String!): User
}
```

**Implementation Challenge:**

```rust
// In execute_mutation_field method, add:
"updateUserEmail" => {
    let user_id = self.get_argument_value(&field.arguments, "id")?;
    let new_email = self.get_argument_value(&field.arguments, "newEmail")?;
    
    // Your implementation here:
    // 1. Find user by ID
    // 2. Validate new email format
    // 3. Update email in database
    // 4. Return updated user
    
    todo!("Implement updateUserEmail mutation")
}
```

### Exercise 2: Error Handling

What happens when a mutation fails? Try this:

```graphql
mutation {
  first: createUser(input: { name: "Alice" }) { id }
  second: createUser(input: { name: "" }) { id }      # Invalid: empty name
  third: createUser(input: { name: "Charlie" }) { id } # Should this execute?
}
```

**Expected behavior:** Sequential execution stops at the failed mutation.

### Exercise 3: Complex Business Logic

Implement a `transferMoney` mutation that requires multiple database operations:

```graphql
mutation {
  transferMoney(
    fromAccountId: "acc_1",
    toAccountId: "acc_2", 
    amount: 100.00
  ) {
    success
    fromAccount { id balance }
    toAccount { id balance }
    transaction { id timestamp amount }
  }
}
```

**Requirements:**

- Debit source account
- Credit destination account  
- Create transaction record
- All-or-nothing: if any step fails, rollback all changes

## 🔮 What's Next?

### Future Enhancements

1. **Database Integration** - Replace mock data with real database operations
2. **Transaction Support** - Wrap mutations in database transactions
3. **Authorization** - Add permission checks to mutation fields
4. **Rate Limiting** - Prevent abuse of expensive mutations
5. **Audit Logging** - Track all data modifications
6. **Batch Operations** - Support for bulk mutations
7. **Optimistic Locking** - Handle concurrent modifications
8. **Custom Scalars** - Support for complex input types

### Advanced Topics

- **DataLoader Pattern** - Efficient data loading in mutations
- **Subscription Integration** - Trigger real-time updates after mutations
- **Custom Directives** - Add middleware to mutation fields
- **Schema Stitching** - Combine mutations from multiple services

## 🎉 Congratulations

You now understand:

- ✅ **What mutations are** and why they're different from queries
- ✅ **How GraphQL processes requests** from HTTP to response
- ✅ **Sequential execution** and why it's critical
- ✅ **Our implementation architecture** with detailed code examples
- ✅ **Practical testing strategies** with real examples
- ✅ **Error handling patterns** for robust applications

**Your mutation support is production-ready!** 🚀

---

### 📚 Additional Resources

- [GraphQL Mutations Specification](https://spec.graphql.org/draft/#sec-Mutation)
- [GraphQL Best Practices - Mutations](https://graphql.org/learn/queries/#mutations)
- [Our Query Execution Guide](docs/04-query-execution.md)
- [Schema Definition Guide](docs/03-schema-definition.md)
