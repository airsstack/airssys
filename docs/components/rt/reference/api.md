# API Reference

Complete API documentation for AirsSys RT - authoritative technical specifications for all public types, traits, and functions.

---

## Overview

This section provides detailed API documentation for all AirsSys RT components. Each page follows Reference documentation principles from the Diátaxis framework: neutral, authoritative, and comprehensive.

**Organization:** API documentation is organized by functional area (core, actors, messaging, supervisors, mailbox, broker, monitoring) rather than by module structure.

---

## Core API

**[Core Types](./api/core.md)**

Fundamental types and traits that form the foundation of the actor system:
- `ActorAddress` - Unique actor identification
- `ActorContext` - Actor execution environment and metadata
- `Message` trait - Message type definition
- `ActorState` - Actor lifecycle states
- `ActorLifecycle` - Lifecycle state machine

**When to use:** Every actor implementation requires understanding these core types.

---

## Actor API

**[Actors](./api/actors.md)**

Actor trait and actor-related functionality:
- `Actor` trait - Core actor behavior
- Lifecycle hooks: `pre_start`, `post_stop`, `post_restart`
- Error handling: `on_error`, `ErrorAction`
- Message handling: `handle_message`
- State management patterns

**When to use:** Implementing custom actors or understanding actor behavior.

---

## Messaging API

**[Messaging](./api/messaging.md)**

Message passing, routing, and communication:
- Message types and protocols
- Request-reply pattern (`ask`)
- Fire-and-forget pattern (`send`)
- Message routing mechanisms
- Performance characteristics

**When to use:** Implementing inter-actor communication or designing message protocols.

---

## Supervision API

**[Supervisors](./api/supervisors.md)**

Supervision trees and fault tolerance:
- `SupervisorBuilder` - Builder pattern for supervisors
- `RestartStrategy` - OneForOne, OneForAll, RestForOne
- `ChildSpec` - Child actor specifications
- Health monitoring and supervision events
- Restart rate limiting

**When to use:** Building fault-tolerant systems with supervision trees.

---

## Mailbox API

**[Mailbox](./api/mailbox.md)**

Actor message queues and backpressure:
- `Mailbox` trait - Message queue abstraction
- Bounded mailboxes - Fixed capacity with backpressure
- Unbounded mailboxes - Dynamic growth
- Capacity configuration and tuning
- Performance characteristics

**When to use:** Configuring actor mailboxes or handling backpressure.

---

## Message Broker API

**[Message Broker](./api/broker.md)**

Publish-subscribe messaging infrastructure:
- `MessageBroker` trait - Broker abstraction
- `InMemoryMessageBroker` - In-process pub-sub
- Topic-based routing
- Subscription management
- Broadcast performance

**When to use:** Implementing event-driven architectures or pub-sub patterns.

---

## Monitoring API

**[Monitoring](./api/monitoring.md)**

Observability, metrics, and health monitoring:
- Health check API
- Metrics collection
- Supervision events
- Actor statistics
- Performance monitoring

**When to use:** Adding observability to actor systems or monitoring system health.

---

## API Design Principles

All AirsSys RT APIs follow these design principles:

### Type Safety First
```rust
// ✅ Strong typing - compile-time safety
impl Actor for MyActor {
    type Message = MyMessage;  // Type-safe messages
    type Error = MyError;      // Type-safe errors
}
```

### Zero-Cost Abstractions
```rust
// ✅ Generic compilation - no runtime overhead
async fn handle_message<B: MessageBroker<Self::Message>>(...) {
    // Monomorphized at compile-time
}
```

### Builder Pattern for Complex Construction
```rust
// ✅ Fluent API - self-documenting configuration
let supervisor = SupervisorBuilder::new()
    .with_strategy(RestartStrategy::OneForOne)
    .with_max_restarts(3)
    .build()
    .await?;
```

### Trait-Based Abstractions
```rust
// ✅ Trait objects for flexibility (where needed)
pub trait Actor {
    type Message: Message;
    type Error: std::error::Error;
    async fn handle_message(...) -> Result<Self::Message::Result, Self::Error>;
}
```

---

## Common Patterns

### Actor Implementation
```rust
use airssys_rt::prelude::*;
use async_trait::async_trait;

#[async_trait]
impl Actor for MyActor {
    type Message = MyMessage;
    type Error = MyError;

    async fn handle_message<B: MessageBroker<Self::Message>>(
        &mut self,
        message: Self::Message,
        context: &mut ActorContext<Self::Message, B>,
    ) -> Result<MyMessage::Result, Self::Error> {
        // Handle message
        context.record_message();
        Ok(result)
    }
}
```

### Supervisor Setup
```rust
let supervisor = SupervisorBuilder::new()
    .with_name("my_supervisor")
    .with_strategy(RestartStrategy::OneForOne)
    .with_max_restarts(3)
    .with_restart_window(Duration::from_secs(60))
    .build()
    .await?;

let actor_ref = supervisor.spawn_child(MyActor::new()).await?;
```

### Message Handling
```rust
// Request-Reply
let result = actor_ref.ask(GetData { id }).await?;

// Fire-and-Forget
actor_ref.send(LogMessage { text }).await?;

// Pub-Sub
broker.publish("topic", Event::DataChanged { id }).await?;
```

---

## Performance Characteristics

All API operations have documented performance characteristics from benchmark data (October 2025 baseline):

| Operation | Latency | Throughput |
|-----------|---------|------------|
| Actor spawn | 624ns | 1.6M/sec |
| Message send (direct) | 737ns | 1.36M/sec |
| Message send (broker) | 917ns | 1.09M/sec |
| Supervision overhead | +656ns | - |
| Mailbox enqueue | 181ns | - |

See [Performance Reference](./performance.md) for complete benchmarking data.

---

## Further Reading

- **[Architecture Reference](./architecture.md)** - System architecture and design
- **[Performance Reference](./performance.md)** - Benchmarking and performance data
- **[Troubleshooting Guide](./troubleshooting.md)** - Common issues and solutions
- **[How-To Guides](../guides/actor-development.md)** - Task-oriented guidance
- **[Explanation](../explanation/actor-model.md)** - Understanding the actor model
