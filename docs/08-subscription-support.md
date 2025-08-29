# GraphQL Subscriptions: Real-time Data with Visual Guide

## 🧭 Overview

This guide explains GraphQL subscriptions with visual diagrams and practical examples for both engineers and GraphQL newcomers. Subscriptions enable real-time data updates in GraphQL applications, allowing clients to receive updates when data changes on the server.

---

## 🔄 Subscription Pipeline (Visual)

```text
┌──────────────────────────────────────────────────────────────┐
│                GraphQL Subscription Pipeline                 │
├──────────────────────────────────────────────────────────────┤
│ 1. Client subscribes to field ──┐                            │
│ 2. Server establishes connection ─┼─▶ Create async stream     │
│ 3. Data changes on server       ─┘                           │
│      │                                                    │
│      ▼                                                    │
│  Trigger subscription resolvers                           │
│      │                                                    │
│      ▼                                                    │
│  Filter & transform data                                  │
│      │                                                    │
│      ▼                                                    │
│  Send update to client via WebSocket/SSE                 │
└──────────────────────────────────────────────────────────────┘
```

---

## 🌊 Subscription Flow (Step-by-Step)

```text
CLIENT                    NETWORK                     SERVER
  │                         │                          │
  │ 1. WebSocket Connect    │                          │
  ├────────────────────────▶│                          │
  │                         ├─────────────────────────▶│
  │                         │         Accept           │
  │                         │◄─────────────────────────┤
  │◄────────────────────────┤                          │
  │                         │                          │
  │ 2. Send Subscription    │                          │
  ├────────────────────────▶│                          │
  │                         ├─────────────────────────▶│
  │                         │      Parse & Validate    │
  │                         │◄─────────────────────────┤
  │◄────────────────────────┤      Ack/Error           │
  │                         │                          │
  │                         │   3. Data Change Event   │
  │                         │◄─────────────────────────┤
  │                         │      Execute Resolver    │
  │                         │◄─────────────────────────┤
  │ 4. Receive Update       │                          │
  │◄────────────────────────┤                          │
  │                         │                          │
  │ 5. Connection Active    │      Keep-Alive          │
  │◄──────────────────────────────────────────────────▶│
```

---

## 📡 Subscription Types & Use Cases

### Common Subscription Patterns

```graphql
# 1. Entity Updates - Get notified when specific records change
subscription UserUpdated($userId: ID!) {
  userUpdated(id: $userId) {
    id
    name
    email
    updatedAt
  }
}

# 2. List Changes - Get notified of additions/removals
subscription MessageAdded($chatId: ID!) {
  messageAdded(chatId: $chatId) {
    id
    content
    author {
      name
    }
    createdAt
  }
}

# 3. System Events - Monitor application events
subscription SystemNotifications {
  systemNotifications {
    type
    message
    severity
    timestamp
  }
}
```

---

## 🏗️ Subscription Architecture in Our Server

```text
┌─────────────────────────────────────────────────────────────────────────┐
│                        SUBSCRIPTION ARCHITECTURE                        │
├─────────────────────────────────────────────────────────────────────────┤
│                                                                         │
│  📱 PRESENTATION LAYER                                                  │
│  ┌─────────────────────────┐    ┌─────────────────────────────────────┐ │
│  │   WebSocket Handler     │    │        HTTP Handler                 │ │
│  │   - Connection mgmt     │    │        - Regular queries            │ │
│  │   - Message routing     │    │        - Mutations                  │ │
│  │   - Client lifecycle    │    │        - Schema introspection      │ │
│  └─────────────────────────┘    └─────────────────────────────────────┘ │
│              │                                      │                   │
│              ▼                                      ▼                   │
│  🎯 APPLICATION LAYER                                                   │
│  ┌─────────────────────────────────────────────────────────────────────┐ │
│  │                  Subscription Manager                               │ │
│  │  - Client subscription registry                                     │ │
│  │  - Event routing and filtering                                      │ │
│  │  - Connection lifecycle management                                  │ │
│  └─────────────────────────────────────────────────────────────────────┘ │
│              │                                                         │
│              ▼                                                         │
│  🏛️ DOMAIN LAYER                                                       │
│  ┌─────────────────┐    ┌──────────────────┐    ┌─────────────────────┐ │
│  │  Event Bus      │    │  Subscription    │    │   Data Resolvers    │ │
│  │  - Pub/Sub      │    │  Executors       │    │   - Field resolution│ │
│  │  - Event types  │    │  - Stream mgmt   │    │   - Data fetching   │ │
│  │  - Filtering    │    │  - Async iteration│   │   - Transformation  │ │
│  └─────────────────┘    └──────────────────┘    └─────────────────────┘ │
│              │                    │                         │           │
│              ▼                    ▼                         ▼           │
│  🔧 INFRASTRUCTURE LAYER                                                │
│  ┌─────────────────┐    ┌──────────────────┐    ┌─────────────────────┐ │
│  │   Data Store    │    │    Event Store   │    │   External APIs     │ │
│  │   - Database    │    │    - Event log   │    │   - Third-party     │ │
│  │   - Cache       │    │    - Persistence │    │   - Microservices   │ │
│  └─────────────────┘    └──────────────────┘    └─────────────────────┘ │
└─────────────────────────────────────────────────────────────────────────┘
```

---

## ⚡ Subscription Execution Flow

```text
SUBSCRIPTION LIFECYCLE
┌─────────────────────────────────────────────────────────────────┐
│                                                                 │
│  1️⃣ SUBSCRIPTION CREATION                                       │
│  ┌─────────────────────────────────────────────────────────────┐ │
│  │ Client sends subscription → Parse → Validate → Execute     │ │
│  │ Create async stream → Register with event system           │ │
│  └─────────────────────────────────────────────────────────────┘ │
│                              │                                 │
│                              ▼                                 │
│  2️⃣ EVENT PROCESSING                                            │
│  ┌─────────────────────────────────────────────────────────────┐ │
│  │ Event occurs → Filter by subscription → Execute resolver   │ │
│  │ Transform data → Apply selection set → Send to client      │ │
│  └─────────────────────────────────────────────────────────────┘ │
│                              │                                 │
│                              ▼                                 │
│  3️⃣ CONNECTION MANAGEMENT                                       │
│  ┌─────────────────────────────────────────────────────────────┐ │
│  │ Handle disconnections → Cleanup subscriptions               │ │
│  │ Manage backpressure → Error handling → Graceful shutdown   │ │
│  └─────────────────────────────────────────────────────────────┘ │
│                                                                 │
└─────────────────────────────────────────────────────────────────┘
```

---

## 🛠️ Implementation Components

### Core Subscription Types

```rust
// Subscription execution result
pub struct SubscriptionResult {
    pub stream: Pin<Box<dyn Stream<Item = ExecutionResult> + Send>>,
    pub errors: Vec<GraphQLError>,
}

// Event system for triggering subscriptions
pub trait EventBus {
    async fn publish(&self, event: Event) -> Result<(), EventError>;
    async fn subscribe(&self, filter: EventFilter) -> Result<EventStream, EventError>;
}

// Subscription manager
pub trait SubscriptionManager {
    async fn create_subscription(
        &self,
        query: Query,
        context: ExecutionContext,
    ) -> Result<SubscriptionResult, GraphQLError>;
    
    async fn terminate_subscription(&self, id: SubscriptionId) -> Result<(), GraphQLError>;
}
```

### Transport Layer Support

```rust
// WebSocket support for real-time communication
pub trait SubscriptionTransport {
    async fn handle_connection(&self, socket: WebSocket) -> Result<(), TransportError>;
    async fn send_message(&self, client_id: ClientId, message: Message) -> Result<(), TransportError>;
}

// Server-Sent Events as alternative transport
pub trait SSETransport {
    async fn create_event_stream(&self, subscription_id: SubscriptionId) -> Result<EventStream, TransportError>;
}
```

---

## 🧪 Subscription Testing Strategy

### Unit Tests

```rust
#[tokio::test]
async fn test_subscription_creation() {
    let subscription = r#"
        subscription {
            messageAdded {
                id
                content
            }
        }
    "#;
    
    let result = executor.create_subscription(subscription).await;
    assert!(result.is_ok());
    assert!(result.unwrap().stream.is_some());
}

#[tokio::test]
async fn test_event_filtering() {
    // Test that subscriptions only receive relevant events
}
```

### Integration Tests

```rust
#[tokio::test]
async fn test_end_to_end_subscription() {
    // 1. Create subscription
    // 2. Trigger data change
    // 3. Verify client receives update
    // 4. Cleanup subscription
}
```

---

## 🚀 Advanced Subscription Features

### Subscription Filtering

```graphql
# Subscribe with filter conditions
subscription MessagesFiltered($chatId: ID!, $authorId: ID) {
  messageAdded(chatId: $chatId, authorId: $authorId) {
    id
    content
    author {
      id
      name
    }
  }
}
```

### Subscription Authentication

```rust
// Authorization context for subscriptions
pub struct SubscriptionContext {
    pub user_id: Option<UserId>,
    pub permissions: Vec<Permission>,
    pub rate_limits: RateLimitConfig,
}
```

### Error Handling

```json
{
  "type": "error",
  "payload": {
    "errors": [
      {
        "message": "Subscription authorization failed",
        "extensions": {
          "code": "UNAUTHORIZED",
          "subscriptionId": "sub_123"
        }
      }
    ]
  }
}
```

---

## 📊 Performance Considerations

### Memory Management

- Connection pooling and cleanup
- Subscription registry optimization
- Event filtering at source

### Scalability

- Horizontal subscription distribution
- Event bus clustering
- Client connection limits

### Monitoring

- Active subscription metrics
- Event throughput monitoring
- Connection health checks

---

## 🔮 Future Enhancements

- **Live Queries**: Automatic query re-execution on data changes
- **Subscription Batching**: Group multiple updates for efficiency
- **Client-side Caching**: Integration with GraphQL cache systems
- **Custom Transports**: Plugin system for different transport mechanisms

---

## 📚 References

- [GraphQL Spec: Subscriptions](https://spec.graphql.org/June2018/#sec-Subscription)
- [Apollo Subscriptions](https://www.apollographql.com/docs/apollo-server/data/subscriptions/)
- [WebSocket Protocol](https://tools.ietf.org/html/rfc6455)
- [Server-Sent Events](https://developer.mozilla.org/en-US/docs/Web/API/Server-sent_events)
