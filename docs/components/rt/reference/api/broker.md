# Message Broker API Reference

This reference documents the message broker system for routing messages between actors.

## Module: `broker`

Message routing and delivery infrastructure.

### Trait: `MessageBroker`

```rust
pub trait MessageBroker: Send + Sync {
    async fn send<M>(&self, actor_id: ActorId, msg: M) -> Result<M::Result, SendError>
    where
        M: Message;
    
    async fn broadcast<M>(&self, msg: M) -> Vec<Result<M::Result, SendError>>
    where
        M: Message + Clone;
    
    fn register<A>(&self, actor_id: ActorId, mailbox: Mailbox<A>) -> Result<(), BrokerError>
    where
        A: Actor;
    
    fn unregister(&self, actor_id: ActorId) -> Result<(), BrokerError>;
}
```

Core trait for message routing and delivery.

**Required Methods:**

- `send()`: Sends message to specific actor and awaits result
- `broadcast()`: Sends message to all registered actors
- `register()`: Registers an actor's mailbox for message delivery
- `unregister()`: Removes an actor from the broker

**Trait Bounds:**
- `Send + Sync`: Can be safely shared across threads

**Implementations:**
- `InMemoryMessageBroker`: Default in-memory broker (production-ready)
- `DistributedMessageBroker`: Future distributed broker implementation

### Struct: `InMemoryMessageBroker`

```rust
pub struct InMemoryMessageBroker {
    // fields omitted
}
```

High-performance in-memory message broker.

**Architecture:**
- Lock-free routing using `DashMap<ActorId, Mailbox>`
- Per-actor mailbox isolation (no shared state)
- Zero-copy message passing where possible
- Concurrent send/receive operations

**Performance Characteristics:**
- Send latency: ~737ns (includes actor processing)
- Routing overhead: ~50ns
- Broadcast (10 actors): ~12M msgs/sec
- Memory: ~48 bytes per registered actor

#### Constructors

##### `new()`

```rust
pub fn new() -> Self
```

Creates a new in-memory message broker.

**Returns:**
- `InMemoryMessageBroker`: New broker instance ready for use

**Example:**

```rust
use airssys_rt::broker::InMemoryMessageBroker;

let broker = InMemoryMessageBroker::new();
```

##### `with_capacity()`

```rust
pub fn with_capacity(capacity: usize) -> Self
```

Creates a broker with pre-allocated capacity.

**Parameters:**
- `capacity`: Expected number of actors to register

**Performance:**
- Reduces allocations during actor registration
- Useful for systems with known actor counts

**Example:**

```rust
// System with ~1000 actors
let broker = InMemoryMessageBroker::with_capacity(1000);
```

#### Registration Methods

##### `register()`

```rust
pub fn register<A>(&self, actor_id: ActorId, mailbox: Mailbox<A>) -> Result<(), BrokerError>
where
    A: Actor,
```

Registers an actor's mailbox with the broker.

**Type Parameters:**
- `A`: The actor type

**Parameters:**
- `actor_id`: Unique identifier for the actor
- `mailbox`: The actor's mailbox for receiving messages

**Returns:**
- `Ok(())`: Registration successful
- `Err(BrokerError::AlreadyRegistered)`: Actor ID already in use

**Thread Safety:**
- Safe to call from multiple threads concurrently
- Atomic registration operation

**Example:**

```rust
use airssys_rt::mailbox::Mailbox;
use airssys_rt::util::ActorId;

let actor_id = ActorId::new();
let mailbox = Mailbox::<MyActor>::bounded(1000);

broker.register(actor_id, mailbox)?;
```

##### `unregister()`

```rust
pub fn unregister(&self, actor_id: ActorId) -> Result<(), BrokerError>
```

Unregisters an actor from the broker.

**Parameters:**
- `actor_id`: The actor to unregister

**Returns:**
- `Ok(())`: Unregistration successful
- `Err(BrokerError::NotFound)`: Actor not registered

**Behavior:**
- Removes actor from routing table
- Remaining messages in mailbox are dropped
- In-flight sends will fail with `SendError::ActorNotFound`
- Safe to call multiple times (idempotent after first success)

**Example:**

```rust
broker.unregister(actor_id)?;
```

#### Message Delivery Methods

##### `send()`

```rust
pub async fn send<M>(&self, actor_id: ActorId, msg: M) -> Result<M::Result, SendError>
where
    M: Message,
```

Sends a message to a specific actor and awaits the result.

**Type Parameters:**
- `M`: The message type

**Parameters:**
- `actor_id`: Target actor identifier
- `msg`: The message to send

**Returns:**
- `Ok(M::Result)`: Message processed successfully, contains result
- `Err(SendError::ActorNotFound)`: Actor not registered
- `Err(SendError::ActorStopped)`: Actor has stopped
- `Err(SendError::MailboxFull)`: Actor's mailbox at capacity
- `Err(SendError::Timeout)`: Operation timed out

**Performance:**
- Average latency: ~737ns (routing + delivery + processing)
- Routing overhead: ~50ns
- Throughput: ~4.7M messages/second (single actor)

**Example:**

```rust
use airssys_rt::Message;

struct GetBalance { account_id: u64 }
impl Message for GetBalance {
    type Result = f64;
}

let balance = broker.send(actor_id, GetBalance { account_id: 123 }).await?;
println!("Account balance: ${:.2}", balance);
```

##### `try_send()`

```rust
pub fn try_send<M>(&self, actor_id: ActorId, msg: M) -> Result<(), SendError>
where
    M: Message<Result = ()>,
```

Attempts to send a fire-and-forget message without blocking.

**Type Parameters:**
- `M`: The message type (must have `Result = ()`)

**Parameters:**
- `actor_id`: Target actor
- `msg`: Message to send

**Returns:**
- `Ok(())`: Message enqueued successfully
- `Err(SendError)`: Delivery failed

**Performance:**
- Non-blocking operation
- Lower latency than `send()` (~181ns)
- No result returned

**Example:**

```rust
struct LogEvent { message: String }
impl Message for LogEvent {
    type Result = ();
}

broker.try_send(logger_id, LogEvent {
    message: "User logged in".to_string(),
})?;
```

##### `broadcast()`

```rust
pub async fn broadcast<M>(&self, msg: M) -> Vec<Result<M::Result, SendError>>
where
    M: Message + Clone,
```

Broadcasts a message to all registered actors.

**Type Parameters:**
- `M`: The message type (must implement `Clone`)

**Parameters:**
- `msg`: The message to broadcast (will be cloned for each actor)

**Returns:**
- `Vec<Result<M::Result, SendError>>`: Results from all actors (one per registered actor)

**Behavior:**
- Parallel delivery to all actors
- Individual failures don't affect other deliveries
- Order of results matches registration order

**Performance:**
- Parallel delivery via `tokio::join_all`
- Throughput: ~12M msgs/sec (10 actors)
- Scales with actor count

**Example:**

```rust
struct Shutdown;
impl Message for Shutdown {
    type Result = ();
}

let results = broker.broadcast(Shutdown).await;

let mut failed = 0;
for result in results {
    if let Err(e) = result {
        eprintln!("Shutdown failed: {:?}", e);
        failed += 1;
    }
}

println!("Shutdown complete: {} failures", failed);
```

##### `broadcast_filter()`

```rust
pub async fn broadcast_filter<M, F>(&self, msg: M, filter: F) -> Vec<Result<M::Result, SendError>>
where
    M: Message + Clone,
    F: Fn(ActorId) -> bool,
```

Broadcasts a message to actors matching a filter predicate.

**Type Parameters:**
- `M`: The message type
- `F`: Filter function type

**Parameters:**
- `msg`: Message to broadcast
- `filter`: Predicate to select target actors

**Returns:**
- `Vec<Result<M::Result, SendError>>`: Results from matching actors

**Example:**

```rust
// Send only to worker actors (IDs 100-199)
let results = broker.broadcast_filter(
    WorkItem { data: vec![] },
    |id| {
        let id_num = id.as_u64();
        id_num >= 100 && id_num < 200
    }
).await;
```

#### Query Methods

##### `is_registered()`

```rust
pub fn is_registered(&self, actor_id: ActorId) -> bool
```

Checks if an actor is registered with the broker.

**Parameters:**
- `actor_id`: Actor to check

**Returns:**
- `true`: Actor is registered
- `false`: Actor not found

**Example:**

```rust
if broker.is_registered(actor_id) {
    broker.send(actor_id, msg).await?;
} else {
    eprintln!("Actor {} not found", actor_id);
}
```

##### `registered_count()`

```rust
pub fn registered_count(&self) -> usize
```

Returns the number of registered actors.

**Returns:**
- `usize`: Count of registered actors

**Use Cases:**
- Monitoring system health
- Capacity planning
- Load balancing decisions

**Example:**

```rust
let count = broker.registered_count();
println!("Active actors: {}", count);
```

##### `actor_ids()`

```rust
pub fn actor_ids(&self) -> Vec<ActorId>
```

Returns a snapshot of all registered actor IDs.

**Returns:**
- `Vec<ActorId>`: List of registered actors

**Note:**
- Snapshot at time of call
- May become stale immediately
- Use for monitoring/debugging, not synchronization

**Example:**

```rust
let actors = broker.actor_ids();
for actor_id in actors {
    println!("Registered: {}", actor_id);
}
```

## Routing Patterns

### Point-to-Point

Direct message delivery to specific actor.

```rust
// One sender, one receiver
let result = broker.send(worker_id, ProcessTask { id: 42 }).await?;
```

### Publish-Subscribe

Broadcast message to all interested actors.

```rust
// Publish event to all subscribers
struct UserCreated { user_id: u64 }
impl Message for UserCreated {
    type Result = ();
}

broker.broadcast(UserCreated { user_id: 123 }).await;
```

### Request-Response

Synchronous-style communication.

```rust
struct GetUserName { user_id: u64 }
impl Message for GetUserName {
    type Result = String;
}

let name = broker.send(db_actor_id, GetUserName { user_id: 42 }).await?;
```

### Fire-and-Forget

Asynchronous notification without response.

```rust
broker.try_send(logger_id, LogMessage {
    level: Level::Info,
    message: "Task completed".to_string(),
})?;
```

## Performance Characteristics

### Latency Breakdown

| Component | Latency | Percentage |
|-----------|---------|------------|
| Routing lookup | ~50ns | 6.8% |
| Mailbox enqueue | ~181ns | 24.6% |
| Actor processing | ~400ns | 54.3% |
| Result return | ~106ns | 14.4% |
| **Total roundtrip** | **~737ns** | **100%** |

### Throughput

| Scenario | Messages/sec | Notes |
|----------|--------------|-------|
| Single actor | 4.7M | Baseline |
| 4 actors (independent) | 18M | Linear scaling |
| 16 actors (independent) | 45M | Sublinear scaling |
| Broadcast (10 actors) | 12M | Parallel delivery |
| Broadcast (100 actors) | 8M | Overhead increases |

### Scalability

| Actor Count | Send Latency | Memory Overhead |
|-------------|-------------|-----------------|
| 10 | 737ns | 480 bytes |
| 100 | 745ns | 4.8 KB |
| 1,000 | 780ns | 48 KB |
| 10,000 | 850ns | 480 KB |
| 100,000 | 1.2µs | 4.8 MB |

### Memory Usage

| Component | Size | Per Actor | Notes |
|-----------|------|-----------|-------|
| Broker base | ~256 bytes | - | DashMap structure |
| Routing entry | ~48 bytes | Yes | ActorId + Mailbox ref |
| Mailbox ref | ~16 bytes | Yes | Arc pointer |

## Error Types

### Enum: `BrokerError`

```rust
pub enum BrokerError {
    AlreadyRegistered,
    NotFound,
    RoutingFailed(String),
}
```

Errors specific to broker operations.

**Variants:**

- `AlreadyRegistered`: Attempted to register duplicate actor ID
- `NotFound`: Actor ID not registered in broker
- `RoutingFailed(String)`: Message routing failed with reason

**Example:**

```rust
use airssys_rt::broker::BrokerError;

match broker.register(actor_id, mailbox) {
    Ok(()) => println!("Actor registered"),
    Err(BrokerError::AlreadyRegistered) => {
        eprintln!("Actor ID {} already in use", actor_id);
        // Generate new ID or handle conflict
    }
    Err(e) => eprintln!("Registration failed: {:?}", e),
}
```

### Enum: `SendError`

```rust
pub enum SendError {
    ActorNotFound,
    ActorStopped,
    MailboxFull,
    Timeout,
}
```

Errors that occur during message delivery.

**Variants:**

- `ActorNotFound`: Target actor not registered
- `ActorStopped`: Actor has stopped (mailbox closed)
- `MailboxFull`: Actor's mailbox at capacity (bounded mailbox with Fail strategy)
- `Timeout`: Send operation timed out

**Example:**

```rust
use airssys_rt::broker::SendError;

match broker.send(actor_id, msg).await {
    Ok(result) => println!("Success: {:?}", result),
    Err(SendError::ActorNotFound) => {
        eprintln!("Actor {} not found", actor_id);
    }
    Err(SendError::MailboxFull) => {
        eprintln!("Actor {} mailbox full, retry later", actor_id);
        // Implement retry logic
    }
    Err(e) => eprintln!("Send failed: {:?}", e),
}
```

## Monitoring and Metrics

### Broker Metrics

```rust
pub struct BrokerMetrics {
    pub registered_actors: usize,
    pub total_sent: u64,
    pub total_delivered: u64,
    pub total_failed: u64,
    pub avg_routing_latency_ns: u64,
}

impl InMemoryMessageBroker {
    pub fn metrics(&self) -> BrokerMetrics {
        // Implementation
    }
}
```

**Usage:**

```rust
let metrics = broker.metrics();
println!("Active actors: {}", metrics.registered_actors);
println!("Success rate: {:.2}%", 
    (metrics.total_delivered as f64 / metrics.total_sent as f64) * 100.0);
println!("Avg routing latency: {}ns", metrics.avg_routing_latency_ns);
```

## Testing Utilities

### Struct: `BrokerTestProbe`

```rust
pub struct BrokerTestProbe {
    // fields omitted
}
```

Testing utility for broker behavior.

**Available in:** Test builds only (`#[cfg(test)]`)

#### Methods

##### `new()`

```rust
pub fn new(broker: InMemoryMessageBroker) -> Self
```

Creates a test probe for a broker.

##### `assert_sent()`

```rust
pub fn assert_sent(&self, actor_id: ActorId, count: usize) -> bool
```

Asserts that expected number of messages were sent to an actor.

**Example:**

```rust
#[cfg(test)]
mod tests {
    use super::*;
    use airssys_rt::broker::BrokerTestProbe;
    
    #[tokio::test]
    async fn test_message_routing() {
        let broker = InMemoryMessageBroker::new();
        let probe = BrokerTestProbe::new(broker.clone());
        
        let actor_id = ActorId::new();
        let mailbox = Mailbox::<MyActor>::bounded(10);
        broker.register(actor_id, mailbox)?;
        
        // Send messages
        for i in 0..5 {
            broker.try_send(actor_id, TestMsg { id: i })?;
        }
        
        assert!(probe.assert_sent(actor_id, 5));
    }
}
```

##### `assert_broadcast()`

```rust
pub fn assert_broadcast(&self, expected_actors: usize) -> bool
```

Asserts that a broadcast reached expected number of actors.

## Advanced Features

### Custom Routing

```rust
pub struct RoutingKey {
    pub topic: String,
    pub priority: u8,
}

impl InMemoryMessageBroker {
    pub fn send_with_routing<M>(
        &self,
        routing_key: RoutingKey,
        msg: M,
    ) -> Result<M::Result, SendError>
    where
        M: Message,
    {
        // Custom routing logic
    }
}
```

### Message Priorities

```rust
pub enum Priority {
    Low,
    Normal,
    High,
    Critical,
}

broker.send_with_priority(actor_id, msg, Priority::High).await?;
```

### Dead Letter Queue

```rust
pub struct DeadLetterQueue {
    // Stores messages that couldn't be delivered
}

impl InMemoryMessageBroker {
    pub fn with_dead_letter_queue(self, dlq: DeadLetterQueue) -> Self {
        // Enable DLQ for failed deliveries
    }
}
```

## See Also

- [Core API Reference](core.md) - Core types and system
- [Messaging API Reference](messaging.md) - Message trait and patterns
- [Mailbox API Reference](mailbox.md) - Message queuing
- [Architecture: Message Passing](../../architecture/message-passing.md) - System design
- [BENCHMARKING.md](../../../BENCHMARKING.md) - Performance data
