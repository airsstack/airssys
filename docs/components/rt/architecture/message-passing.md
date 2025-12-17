# Message Passing System

The message passing system in `airssys-rt` provides high-performance pub-sub communication between actors following Erlang/OTP principles.

> **Note**: All code examples are from actual implementation. See [examples directory](../../examples/) for complete working code.

## Architecture Overview

### Design Principles

The message passing system is built on three core abstractions:

1. **Message Trait** - Type-safe message contracts
2. **MessageBroker** - Pub/sub routing system  
3. **Mailbox** - Message queue management

**Performance Characteristics** (from BENCHMARKING.md):
- **Point-to-point latency**: 737 ns per roundtrip
- **Sustained throughput**: 4.7M messages/second
- **Broadcast efficiency**: 395 ns to 10 actors (~40 ns per subscriber)
- **Message processing**: 31.55 ns/message (direct), 211.88 ns/message (via broker)

## Message Trait

### Definition

All messages must implement the `Message` trait (from `src/message/mod.rs`):

```rust
pub trait Message: Clone + Send + Sync + 'static 
    + for<'de> serde::Deserialize<'de> + serde::Serialize 
{
    const MESSAGE_TYPE: &'static str;
}
```

**Design Rationale:**

- `Clone`: Messages can be sent to multiple subscribers
- `Send + Sync + 'static`: Thread-safe cross-actor messaging
- `Serialize + Deserialize`: Future network/persistence support
- `MESSAGE_TYPE`: Runtime type identification for routing

### Implementation Example

```rust
use serde::{Deserialize, Serialize};
use airssys_rt::message::Message;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CounterMessage {
    pub delta: i32,
}

impl Message for CounterMessage {
    const MESSAGE_TYPE: &'static str = "counter";
}
```

### Message Best Practices

**DO:**

- ✅ Keep messages small and focused
- ✅ Use strongly-typed enums for variants
- ✅ Make fields `pub` for builder patterns
- ✅ Derive `Debug` for logging

**DON'T:**

- ❌ Include large data structures (use references/IDs)
- ❌ Add non-serializable types
- ❌ Mutate messages (they're cloned)
- ❌ Use `Box<dyn Trait>` in messages

## Message Envelope

### Structure

Messages are wrapped in envelopes for routing (from `src/message/envelope.rs`):

```rust
pub struct MessageEnvelope<M> {
    pub id: MessageId,
    pub message: M,
    pub timestamp: DateTime<Utc>,  // §3.2 chrono DateTime<Utc>
    pub reply_to: Option<ActorAddress>,
}
```

**Fields:**

- `id`: Unique message identifier (UUID-based)
- `message`: Actual message payload
- `timestamp`: When envelope was created (UTC)
- `reply_to`: Optional sender address for request/reply pattern

### Creation

```rust
use airssys_rt::message::MessageEnvelope;
use chrono::Utc;

let envelope = MessageEnvelope::new(CounterMessage { delta: 1 });
// Sets id, timestamp automatically

// With reply address
let envelope = MessageEnvelope {
    id: MessageId::new(),
    message: CounterMessage { delta: 1 },
    timestamp: Utc::now(),
    reply_to: Some(sender_address),
};
```

## MessageBroker Trait

### Definition

The pub/sub system for actor communication (from `src/broker/traits.rs`):

```rust
#[async_trait]
pub trait MessageBroker<M: Message>: Clone + Send + Sync + 'static {
    type Error: Error + Send + Sync + 'static;

    async fn publish(&self, envelope: MessageEnvelope<M>) 
        -> Result<(), Self::Error>;
    
    async fn subscribe(&self, subscriber_id: ActorId) 
        -> Result<mpsc::Receiver<MessageEnvelope<M>>, Self::Error>;
}
```

**Design Rationale:**

- `Clone`: Brokers can be shared across actors (Arc internally)
- Generic `<M: Message>`: Type-safe message routing
- `async`: Non-blocking operations
- Associated `Error`: Broker-specific error handling

### Publish-Subscribe Pattern

```rust
// Publisher side
let broker = InMemoryMessageBroker::new();
let envelope = MessageEnvelope::new(my_message);
broker.publish(envelope).await?;

// Subscriber side
let mut receiver = broker.subscribe(actor_id).await?;
while let Some(envelope) = receiver.recv().await {
    // Process envelope.message
}
```

## InMemoryMessageBroker

### Implementation

Current production broker using Tokio channels (from `src/broker/in_memory.rs`):

```rust
#[derive(Clone)]
pub struct InMemoryMessageBroker<M: Message> {
    subscribers: Arc<Mutex<HashMap<ActorId, mpsc::Sender<MessageEnvelope<M>>>>>,
}

impl<M: Message> InMemoryMessageBroker<M> {
    pub fn new() -> Self {
        Self {
            subscribers: Arc::new(Mutex::new(HashMap::new())),
        }
    }
}
```

**Characteristics:**

- **Thread-safe**: Arc + Mutex for multi-threaded access
- **Cheap Clone**: Arc-based, no deep copy
- **Dynamic subscribers**: Add/remove at runtime
- **Unbounded channels**: No backpressure (see Mailbox for bounded queues)

### Performance Profile

Based on `benches/message_benchmarks.rs`:

| Operation | Latency | Throughput |
|-----------|---------|------------|
| Point-to-point | 737 ns | 1.36M messages/sec |
| Sustained throughput (100 msgs) | 211 ns/msg | 4.7M messages/sec |
| Broadcast to 10 actors | 395 ns total | ~40 ns/subscriber |

**Broker Overhead:**

- Direct actor processing: 31.55 ns/message
- Via broker routing: 211.88 ns/message
- **6.7x overhead** - acceptable for pub-sub semantics

## Actor Context Messaging

### Sending Messages

Actors send messages via their context (from `ActorContext`):

```rust
#[async_trait]
impl Actor for MyActor {
    type Message = MyMessage;
    type Error = MyError;

    async fn handle_message<B: MessageBroker<Self::Message>>(
        &mut self,
        message: Self::Message,
        context: &mut ActorContext<Self::Message, B>,
    ) -> Result<(), Self::Error> {
        // Send to another actor
        let recipient = ActorAddress::named("counter");
        context.send(
            CounterMessage { delta: 1 },
            recipient
        ).await?;

        Ok(())
    }
}
```

### Request-Reply Pattern

For synchronous-style communication (async underneath):

```rust
// Request side
let response = context.request(
    QueryMessage { id: 42 },
    target_address,
    Duration::from_secs(5)  // timeout
).await?;

// Reply side (in target actor)
async fn handle_message<B: MessageBroker<Self::Message>>(
    &mut self,
    envelope: MessageEnvelope<Self::Message>,
    context: &mut ActorContext<Self::Message, B>,
) -> Result<(), Self::Error> {
    if let Some(reply_to) = envelope.reply_to {
        let response = ResponseMessage { result: "ok" };
        context.send(response, reply_to).await?;
    }
    Ok(())
}
```

## Mailbox System

### Mailbox Types

The runtime provides two mailbox implementations (from `src/mailbox/`):

**UnboundedMailbox** - Unlimited capacity:
```rust
pub struct UnboundedMailbox<M: Message> {
    receiver: mpsc::UnboundedReceiver<MessageEnvelope<M>>,
    metrics: Arc<AtomicMetrics>,
}
```

**BoundedMailbox** - Limited capacity with backpressure:
```rust
pub struct BoundedMailbox<M: Message> {
    receiver: mpsc::Receiver<MessageEnvelope<M>>,
    capacity: usize,
    backpressure: BackpressureStrategy,
    metrics: Arc<AtomicMetrics>,
}
```

### Backpressure Strategies

```rust
pub enum BackpressureStrategy {
    Block,      // Block sender when mailbox full
    Drop,       // Drop new messages when full
    DropOldest, // Drop oldest message to make room
}
```

**Usage Guidelines:**

- **Block**: Critical messages that must be delivered
- **Drop**: Optional updates (metrics, status) where latest is enough
- **DropOldest**: Event streams where recent data matters most

### Performance Characteristics

From `benches/message_benchmarks.rs`:

| Mailbox Operation | Latency |
|-------------------|---------|
| Enqueue + Dequeue (100 ops) | 181.60 ns/operation |
| Bounded mailbox (capacity 100) | 244.18 ns/mailbox overhead |

**Mailbox operations are ~6x faster than broker routing** (181 ns vs 211 ns), confirming Tokio channel efficiency.

### Mailbox Traits

Generic mailbox interface for testing and future backends:

```rust
#[async_trait]
pub trait MailboxReceiver<M: Message>: Send {
    async fn recv(&mut self) -> Option<MessageEnvelope<M>>;
    fn try_recv(&mut self) -> Result<MessageEnvelope<M>, TryRecvError>;
}

#[async_trait]
pub trait MailboxSender<M: Message>: Clone + Send + Sync {
    async fn send(&self, envelope: MessageEnvelope<M>) 
        -> Result<(), MailboxError>;
}
```

## Message Flow Architecture

### Complete Message Path

```
┌─────────────┐
│ Sender      │
│ Actor       │
└──────┬──────┘
       │ 1. context.send(message, recipient)
       ▼
┌─────────────────┐
│ ActorContext    │
│ - Wraps message │
│ - Creates       │
│   envelope      │
└──────┬──────────┘
       │ 2. broker.publish(envelope)
       ▼
┌──────────────────┐
│ MessageBroker    │
│ - Routes to      │
│   subscribers    │
│ - Clones for     │
│   broadcast      │
└──────┬───────────┘
       │ 3. mpsc::Sender → receivers
       ▼
┌──────────────────┐
│ Mailbox (queue)  │
│ - Buffers        │
│ - Backpressure   │
│ - Metrics        │
└──────┬───────────┘
       │ 4. Receiver.recv()
       ▼
┌──────────────────┐
│ Recipient Actor  │
│ - handle_message │
│ - Process logic  │
└──────────────────┘
```

### Latency Breakdown

Based on benchmark measurements:

1. **Message wrapping**: ~10 ns (allocation + timestamp)
2. **Broker routing**: ~180 ns (mutex + channel send)
3. **Mailbox buffering**: ~20 ns (queue operation)
4. **Actor processing**: 31-200 ns (depends on logic)

**Total roundtrip**: 737 ns (sub-microsecond)

## Communication Patterns

### Fire-and-Forget

```rust
// No response expected
context.send(
    NotificationMessage { event: "started" },
    monitor_address
).await?;
```

### Request-Reply

```rust
// Wait for response with timeout
let response = context.request(
    QueryMessage { id: 42 },
    database_actor,
    Duration::from_secs(5)
).await?;
```

### Broadcast

```rust
// MessageBroker automatically broadcasts to all subscribers
broker.publish(
    MessageEnvelope::new(BroadcastMessage { alert: "shutdown" })
).await?;
```

### Actor Pools

From `examples/worker_pool.rs`:

```rust
// Round-robin distribution to worker pool
let worker_id = self.next_worker;
self.next_worker = (self.next_worker + 1) % self.workers.len();

context.send(
    WorkMessage { task_id },
    self.workers[worker_id].clone()
).await?;
```

## Error Handling

### Broker Errors

```rust
#[derive(Debug)]
pub enum BrokerError {
    SubscriberNotFound(ActorId),
    ChannelClosed,
    SendError(String),
}
```

**Recovery Strategies:**

- `SubscriberNotFound`: Retry with discovery or fail gracefully
- `ChannelClosed`: Cleanup subscriber, log issue
- `SendError`: Escalate to supervisor

### Mailbox Errors

```rust
#[derive(Debug)]
pub enum MailboxError {
    Full,           // Bounded mailbox at capacity
    Closed,         // Receiver dropped
    Timeout,        // Receive timeout exceeded
}
```

**Handling Guidelines:**

- **Full + Block**: Automatic backpressure (sender waits)
- **Full + Drop**: Log dropped message, continue
- **Closed**: Stop sending, cleanup references
- **Timeout**: Retry or escalate based on criticality

## Performance Optimization

### Message Design

**Optimize for:**

- Small message size (<100 bytes ideal)
- Cheap cloning (primitives, small vecs)
- Serialization efficiency (serde derives)

**Avoid:**

- Large vecs/strings (use Arc or IDs)
- Boxed trait objects (static dispatch preferred)
- Deep nesting (flattens better)

### Broker Selection

**InMemoryMessageBroker:**

- ✅ Low latency (737 ns roundtrip)
- ✅ High throughput (4.7M msgs/sec)
- ✅ Simple, correct, fast
- ❌ Single-process only
- ❌ No persistence

**Future brokers** (planned):
- Distributed broker (network routing)
- Persistent broker (durability)
- Sharded broker (horizontal scaling)

### Mailbox Tuning

**Unbounded:**

- Use for: Control messages, low-volume actors
- Avoids: Backpressure complexity
- Risk: Unbounded memory growth

**Bounded:**

- Use for: High-volume data streams
- Capacity: 100-1000 typical (balance latency vs memory)
- Strategy: Match to use case (Block/Drop/DropOldest)

## Testing Patterns

### Unit Testing Messages

```rust
#[test]
fn test_message_serialization() {
    let msg = CounterMessage { delta: 42 };
    let json = serde_json::to_string(&msg).unwrap();
    let deserialized: CounterMessage = serde_json::from_str(&json).unwrap();
    assert_eq!(msg.delta, deserialized.delta);
}
```

### Integration Testing Broker

```rust
#[tokio::test]
async fn test_pub_sub_roundtrip() {
    let broker = InMemoryMessageBroker::new();
    let actor_id = ActorId::new();
    
    let mut receiver = broker.subscribe(actor_id).await.unwrap();
    
    let message = TestMessage { value: 42 };
    broker.publish(MessageEnvelope::new(message.clone())).await.unwrap();
    
    let envelope = receiver.recv().await.unwrap();
    assert_eq!(envelope.message.value, 42);
}
```

## Working Examples

Explore message passing in these examples:

| Example | Demonstrates | Command |
|---------|--------------|---------|
| `actor_basic.rs` | Simple message handling | `cargo run --example actor_basic` |
| `worker_pool.rs` | Round-robin message routing | `cargo run --example worker_pool` |
| `event_pipeline.rs` | Message-driven pipeline | `cargo run --example event_pipeline` |

All examples are in the `examples/` directory with complete, runnable implementations.
