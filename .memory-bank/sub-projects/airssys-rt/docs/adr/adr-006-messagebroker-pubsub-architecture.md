# ADR-006: MessageBroker Pub-Sub Architecture

**Status**: Accepted  
**Date**: 2025-10-06  
**Decision Makers**: Architecture Team  
**Related**: RT-TASK-006, DEBT-RT-005, KNOWLEDGE-RT-011

---

## Context and Problem Statement

During RT-TASK-006 Phase 2 implementation (ActorSystem core), we discovered a fundamental architectural flaw: **MessageBroker was designed as a direct routing system instead of a proper pub-sub message bus**.

The original design had:
- `MessageBroker.send(recipient, message)` - direct delivery semantics
- ActorSystem directly calling broker methods for message delivery
- No clear separation between transport layer and routing logic
- Limited extensibility for monitoring, persistence, or distributed messaging

This created tight coupling and prevented essential features like:
- Dead letter queues
- Message monitoring and observability
- Circuit breakers
- Distributed broker implementations (Redis, NATS, etc.)
- Multiple subscribers to message streams

## Decision Drivers

- **Extensibility**: Need hooks for logging, metrics, persistence at the transport layer
- **Separation of Concerns**: Transport (broker) vs. Routing (registry) vs. Orchestration (system)
- **Future Distributed Support**: Enable Redis/NATS broker implementations without changing actors
- **Observability**: Multiple subscribers can monitor message flows independently
- **Testability**: Easier to mock pub-sub interfaces than direct routing
- **Standards Compliance**: §6.1 (YAGNI) - build pub-sub now because we need it for monitoring

## Considered Options

### Option 1: Keep Direct Routing in MessageBroker ❌
```rust
async fn send(&self, recipient: ActorAddress, message: M) -> Result<()>;
```

**Pros**:
- Simple implementation
- No additional complexity

**Cons**:
- Tight coupling between broker and routing logic
- No extensibility hooks
- Cannot support multiple subscribers
- Blocks distributed broker implementations
- No monitoring/observability support

### Option 2: Pub-Sub Architecture with Subscribe/Publish ✅ **SELECTED**
```rust
async fn publish(&self, envelope: MessageEnvelope<M>) -> Result<()>;
async fn subscribe(&self) -> Result<MessageStream<M>>;
```

**Pros**:
- Clean separation: broker = transport, registry = routing, system = orchestration
- Natural extensibility hooks in `publish()` method
- Supports multiple subscribers (system, monitor, audit, etc.)
- Enables distributed brokers (Redis pub-sub, NATS, etc.)
- Decouples message producers from consumers
- Better testability with stream-based APIs

**Cons**:
- Slightly more complex implementation (~100 lines for pub-sub channels)
- Requires background task in ActorSystem for routing
- Additional type: MessageStream<M>

### Option 3: Hybrid Approach (Direct + Pub-Sub) ❌
```rust
async fn send(&self, recipient: ActorAddress, message: M) -> Result<()>;
async fn broadcast(&self, message: M) -> Result<()>;
```

**Pros**:
- Supports both patterns

**Cons**:
- API confusion - when to use send vs broadcast?
- Still doesn't solve monitoring/extensibility problem
- Complexity without clear benefit
- Violates single responsibility principle

## Decision Outcome

**Chosen Option: Option 2 - Pub-Sub Architecture**

### Architecture Overview

```
┌─────────────────────────────────────────────────────────────┐
│                      MESSAGE BUS                             │
│                    (MessageBroker)                           │
│                                                              │
│  Publishers                Topics              Subscribers   │
│     │                        │                      │        │
│  Actor─────publish──────────>│                      │        │
│  Context                     │<─────subscribe───────System   │
│                              │                      │        │
│                              │──notify────────────> │        │
│                              │  (message arrives)   │        │
└─────────────────────────────────────────────────────────────┘
                                                       │
                                                       ▼
                                              ActorRegistry.resolve()
                                                       │
                                                       ▼
                                                Actor Mailbox
```

### Component Responsibilities

1. **MessageBroker (Transport Layer)**
   - Publish messages to the bus
   - Manage subscribers
   - Provide extensibility hooks (logging, metrics, persistence)
   - **Does NOT** route messages to actors

2. **ActorRegistry (Routing Table)**
   - Map ActorAddress → MailboxSender
   - Register/unregister actors
   - Resolve addresses to mailboxes
   - **Does NOT** transport messages

3. **ActorSystem (Orchestration)**
   - Subscribe to broker's message stream
   - Route messages using ActorRegistry
   - Handle dead letters
   - Spawn/stop actors
   - **Does NOT** directly deliver messages

4. **ActorContext (Publisher API)**
   - Publish messages via broker
   - Simple actor-facing API
   - **Does NOT** know about routing

### Type Signatures

```rust
#[async_trait]
pub trait MessageBroker<M: Message>: Send + Sync + Clone {
    type Error: Error + Send + Sync + 'static;
    
    /// Publish a message to the broker bus
    async fn publish(&self, envelope: MessageEnvelope<M>) -> Result<(), Self::Error>;
    
    /// Subscribe to message events on the broker
    async fn subscribe(&self) -> Result<MessageStream<M>, Self::Error>;
    
    /// Publish a request and track correlation for reply
    async fn publish_request<R: Message>(
        &self,
        envelope: MessageEnvelope<M>,
        timeout: Duration,
    ) -> Result<Option<MessageEnvelope<R>>, Self::Error>;
}

pub struct MessageStream<M: Message> {
    receiver: mpsc::UnboundedReceiver<MessageEnvelope<M>>,
}

pub struct ActorSystem<M, S, B>
where
    M: Message,
    S: MailboxSender<M> + Clone,
    B: MessageBroker<M>,
{
    inner: Arc<ActorSystemInner<M, S, B>>,
}

struct ActorSystemInner<M, S, B> {
    config: SystemConfig,
    broker: B,                      // ← Dependency Injection
    registry: ActorRegistry<M, S>,  // ← Created internally
    // ...
}
```

### Implementation Plan

#### Phase 0: Extend MessageBroker Trait (RT-TASK-004 Modification)
- **File**: `src/broker/traits.rs`
- **Changes**:
  - Add `publish()` method (rename from `send()`)
  - Add `subscribe()` method
  - Add `publish_request()` method
  - Add MessageStream type
- **Estimated**: 2-3 hours
- **Tests**: Update all broker tests (~30 tests)

#### Phase 1: Update InMemoryMessageBroker
- **File**: `src/broker/in_memory.rs`
- **Changes**:
  - Add `subscribers: Arc<RwLock<Vec<UnboundedSender<MessageEnvelope<M>>>>>`
  - Implement `subscribe()` - register new subscriber
  - Implement `publish()` - broadcast to all subscribers
  - Add extensibility hooks (logging, metrics placeholders)
- **Estimated**: 2-3 hours
- **Tests**: ~15 new tests

#### Phase 2: Update ActorSystem (RT-TASK-006 Phase 2)
- **File**: `src/system/actor_system.rs`
- **Changes**:
  - Subscribe to broker in `new()`
  - Spawn message router background task
  - Route via ActorRegistry.resolve()
  - Implement dead letter queue
- **Estimated**: 3-4 hours
- **Tests**: ~20-25 tests

#### Phase 3: Update ActorContext (Future RT-TASK-007)
- **File**: `src/actor/context.rs`
- **Changes**:
  - Use `broker.publish()` instead of direct routing
  - Keep simple actor-facing API
- **Estimated**: 1-2 hours
- **Tests**: Update existing tests

**Total Estimated Time**: 8-12 hours

## Consequences

### Positive

✅ **Clean Architecture**: Clear separation between transport, routing, and orchestration  
✅ **Extensibility**: Natural hooks for logging, metrics, persistence, circuit breakers  
✅ **Multiple Subscribers**: System, monitor, audit can all subscribe independently  
✅ **Distributed Ready**: Redis/NATS brokers can implement same trait  
✅ **Testability**: Stream-based APIs easier to mock and test  
✅ **Dead Letter Support**: Undeliverable messages naturally handled by router  
✅ **Observability**: Message flows visible to monitoring subscribers  
✅ **Future-Proof**: Enables advanced patterns (event sourcing, CQRS, etc.)

### Negative

⚠️ **Complexity**: ~100 additional lines for pub-sub channel management  
⚠️ **Background Task**: ActorSystem spawns routing task (standard pattern)  
⚠️ **Memory**: Each subscriber holds message channel (minimal overhead)  
⚠️ **Refactoring**: Need to update RT-TASK-004 and RT-TASK-006 implementations

### Neutral

➡️ **Performance**: Similar to direct routing (one extra hop through channel)  
➡️ **Learning Curve**: Pub-sub is well-understood pattern in actor systems

## Compliance Check

- **§6.1 (YAGNI)**: ✅ We NEED pub-sub for monitoring/observability (immediate requirement)
- **§6.2 (Avoid dyn)**: ✅ Using generic `B: MessageBroker<M>` not `dyn MessageBroker`
- **§6.3 (M-DI-HIERARCHY)**: ✅ Generic constraints over trait objects
- **§6.3 (M-SERVICES-CLONE)**: ✅ MessageBroker is Clone via Arc<Inner> pattern
- **§6.3 (M-DESIGN-FOR-AI)**: ✅ Idiomatic pub-sub pattern, well-documented

## Examples

### Example 1: Basic Pub-Sub Flow

```rust
// Setup
let broker = InMemoryMessageBroker::<MyMessage, BoundedMailbox<_, _>>::new();
let config = SystemConfig::default();
let system = ActorSystem::new(config, broker.clone()).await?;

// System subscribes on initialization (automatic)
// Background router task runs:
//   while let Some(envelope) = stream.next().await {
//       let sender = registry.resolve(&envelope.reply_to)?;
//       sender.send(envelope).await?;
//   }

// Actor publishes
let context = ActorContext::new(address, broker.clone());
context.send(recipient, message).await?;  // ← Calls broker.publish()
```

### Example 2: Multiple Subscribers

```rust
// Routing subscriber (ActorSystem)
let routing_stream = broker.subscribe().await?;
tokio::spawn(route_messages(routing_stream, registry));

// Monitoring subscriber
let monitor_stream = broker.subscribe().await?;
tokio::spawn(async move {
    while let Some(envelope) = monitor_stream.next().await {
        metrics.record_message(&envelope);
        log::trace!("Message: {:?}", envelope);
    }
});

// Audit subscriber
let audit_stream = broker.subscribe().await?;
tokio::spawn(async move {
    while let Some(envelope) = audit_stream.next().await {
        audit_log.persist(&envelope).await;
    }
});
```

### Example 3: Extensibility Hooks

```rust
impl MessageBroker<M> for InMemoryMessageBroker<M, S> {
    async fn publish(&self, envelope: MessageEnvelope<M>) -> Result<()> {
        // Hook 1: Logging
        log::trace!("Publishing: {:?}", envelope);
        
        // Hook 2: Metrics
        self.metrics.messages_published.increment();
        
        // Hook 3: Persistence (optional)
        if self.config.persist_messages {
            self.storage.persist(&envelope).await?;
        }
        
        // Hook 4: Circuit breaker
        self.circuit_breaker.check(&envelope.reply_to)?;
        
        // Broadcast to all subscribers
        let subscribers = self.subscribers.read().await;
        for sender in subscribers.iter() {
            let _ = sender.send(envelope.clone());
        }
        
        Ok(())
    }
}
```

### Example 4: Future Distributed Broker

```rust
pub struct RedisMessageBroker {
    client: RedisClient,
    channel: String,
}

#[async_trait]
impl<M: Message> MessageBroker<M> for RedisMessageBroker {
    async fn publish(&self, envelope: MessageEnvelope<M>) -> Result<()> {
        let serialized = serde_json::to_vec(&envelope)?;
        self.client.publish(&self.channel, serialized).await?;
        Ok(())
    }
    
    async fn subscribe(&self) -> Result<MessageStream<M>> {
        let mut pubsub = self.client.pubsub().await?;
        pubsub.subscribe(&self.channel).await?;
        
        let (tx, rx) = mpsc::unbounded_channel();
        
        tokio::spawn(async move {
            while let Some(msg) = pubsub.next().await {
                let envelope: MessageEnvelope<M> = serde_json::from_slice(&msg.payload)?;
                let _ = tx.send(envelope);
            }
        });
        
        Ok(MessageStream { receiver: rx })
    }
}

// Usage: ZERO CHANGES to actors!
let broker = RedisMessageBroker::new("redis://localhost", "actor-messages");
let system = ActorSystem::new(config, broker).await?;
// All actors work identically with Redis broker
```

## Related Decisions

- **ADR-002**: ActorRegistry Lock-Free Design (routing table)
- **ADR-003**: Message Envelope Design (message format)
- **RT-TASK-004**: Message Broker Implementation (trait definition)
- **RT-TASK-006**: Actor System Framework (orchestration)
- **DEBT-RT-005**: Original issue that uncovered this architecture gap

## References

- Erlang OTP Message Passing: Pub-sub via process groups
- Akka EventBus: Pub-sub pattern for message distribution
- Redis Pub-Sub: Distributed message bus example
- NATS Messaging: Cloud-native pub-sub architecture
- Microsoft Rust Guidelines: M-DI-HIERARCHY (generic constraints over dyn)

---

**Decision**: Use pub-sub architecture with publish/subscribe pattern  
**Impact**: High - Affects RT-TASK-004 and RT-TASK-006  
**Benefit**: Massive - Enables extensibility, monitoring, and distributed support  
**Status**: Accepted, implementation in progress
