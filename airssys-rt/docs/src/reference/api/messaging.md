# Messaging API Reference

This reference documents the message broker, mailbox system, and message delivery infrastructure.

## Module: `message`

Message trait and broker types.

### Trait: `Message`

```rust
pub trait Message: Send + 'static {
    type Result: Send + 'static;
}
```

Marker trait for types that can be sent as messages between actors.

**Type Parameters:**
- `Result`: The type returned when this message is processed

**Trait Bounds:**
- `Send`: Must be safe to send across thread boundaries
- `'static`: Must not contain non-static references

**Design Rationale:**

The `Message` trait is intentionally minimal to allow maximum flexibility. Any type that is `Send + 'static` can be a message by simply declaring its result type.

**Example:**

```rust
use airssys_rt::Message;

// Simple notification (no result)
struct Ping;
impl Message for Ping {
    type Result = ();
}

// Query message (returns data)
struct GetUser {
    user_id: u64,
}
impl Message for GetUser {
    type Result = Option<User>;
}

// Command message (returns success/error)
struct UpdateUser {
    user_id: u64,
    name: String,
}
impl Message for UpdateUser {
    type Result = Result<(), UpdateError>;
}
```

## Module: `broker`

Message broker implementations.

### Trait: `MessageBroker`

```rust
pub trait MessageBroker: Send + Sync {
    async fn send<M>(&self, actor_id: ActorId, msg: M) -> Result<M::Result, SendError>
    where
        M: Message;
    
    async fn broadcast<M>(&self, msg: M) -> Vec<Result<M::Result, SendError>>
    where
        M: Message + Clone;
}
```

Trait for message routing and delivery.

**Required Methods:**

- `send()`: Sends a message to a specific actor
- `broadcast()`: Sends a message to all registered actors

**Trait Bounds:**
- `Send + Sync`: Can be safely shared across threads

**Implementations:**
- `InMemoryMessageBroker`: Default in-memory broker

### Struct: `InMemoryMessageBroker`

```rust
pub struct InMemoryMessageBroker {
    // fields omitted
}
```

In-memory message broker using channels for delivery.

**Characteristics:**

- Lock-free message routing using `DashMap`
- Per-actor mailbox isolation
- Configurable backpressure strategies
- Zero-copy message passing (where possible)

#### Methods

##### `new()`

```rust
pub fn new() -> Self
```

Creates a new in-memory message broker.

**Returns:**
- `InMemoryMessageBroker`: New broker instance

**Example:**

```rust
use airssys_rt::broker::InMemoryMessageBroker;

let broker = InMemoryMessageBroker::new();
```

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

**Example:**

```rust
let mailbox = Mailbox::bounded(1000);
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
- Remaining messages in mailbox are dropped
- In-flight sends will fail with `SendError::ActorNotFound`

##### `send()`

```rust
pub async fn send<M>(&self, actor_id: ActorId, msg: M) -> Result<M::Result, SendError>
where
    M: Message,
```

Sends a message to an actor and waits for the result.

**Type Parameters:**
- `M`: The message type

**Parameters:**
- `actor_id`: Target actor identifier
- `msg`: The message to send

**Returns:**
- `Ok(M::Result)`: Message processed successfully
- `Err(SendError)`: Delivery or processing failed

**Performance:**
- Average latency: ~737ns (including actor processing)
- Throughput: ~4.7M messages/second

**Example:**

```rust
use airssys_rt::Message;

struct GetStatus;
impl Message for GetStatus {
    type Result = String;
}

let status = broker.send(actor_id, GetStatus).await?;
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
- `msg`: The message to broadcast

**Returns:**
- `Vec<Result<M::Result, SendError>>`: Results from all actors

**Performance:**
- Parallel delivery to all actors
- Individual failures don't affect other deliveries

**Example:**

```rust
struct Shutdown;
impl Message for Shutdown {
    type Result = ();
}

let results = broker.broadcast(Shutdown).await;
for result in results {
    if let Err(e) = result {
        eprintln!("Shutdown failed: {:?}", e);
    }
}
```

## Module: `mailbox`

Mailbox implementations for actor message queues.

### Enum: `Mailbox<A>`

```rust
pub enum Mailbox<A: Actor> {
    Bounded(BoundedMailbox<A>),
    Unbounded(UnboundedMailbox<A>),
}
```

Actor mailbox abstraction supporting bounded and unbounded queues.

**Type Parameters:**
- `A`: The actor type

**Variants:**

- `Bounded(BoundedMailbox<A>)`: Fixed-capacity mailbox with backpressure
- `Unbounded(UnboundedMailbox<A>)`: Unlimited capacity mailbox

**Choosing a Mailbox:**

| Use Case | Recommended | Rationale |
|----------|-------------|-----------|
| High-throughput actors | Bounded | Prevents memory exhaustion |
| Low-volume actors | Unbounded | Simpler, no backpressure |
| Critical path | Bounded | Predictable latency |
| Background tasks | Unbounded | Flexibility over control |

#### Constructors

##### `bounded()`

```rust
pub fn bounded(capacity: usize) -> Self
```

Creates a bounded mailbox with specified capacity.

**Parameters:**
- `capacity`: Maximum number of messages

**Default Backpressure:**
- Strategy: `BackpressureStrategy::Block`
- Behavior: Senders block when mailbox is full

**Example:**

```rust
use airssys_rt::mailbox::Mailbox;

let mailbox = Mailbox::<MyActor>::bounded(1000);
```

##### `unbounded()`

```rust
pub fn unbounded() -> Self
```

Creates an unbounded mailbox with unlimited capacity.

**Returns:**
- `Mailbox<A>`: Unbounded mailbox instance

**Example:**

```rust
let mailbox = Mailbox::<MyActor>::unbounded();
```

#### Methods

##### `enqueue()`

```rust
pub async fn enqueue(&self, msg: Box<dyn Message>) -> Result<(), MailboxError>
```

Adds a message to the mailbox.

**Parameters:**
- `msg`: Boxed message to enqueue

**Returns:**
- `Ok(())`: Message enqueued successfully
- `Err(MailboxError::Full)`: Bounded mailbox at capacity
- `Err(MailboxError::Closed)`: Mailbox has been closed

**Performance:**
- Bounded mailbox: ~181ns average
- Unbounded mailbox: ~150ns average

##### `dequeue()`

```rust
pub async fn dequeue(&self) -> Option<Box<dyn Message>>
```

Removes and returns the next message from the mailbox.

**Returns:**
- `Some(msg)`: Next message available
- `None`: Mailbox is empty and closed

**Behavior:**
- Blocks until message available or mailbox closed
- FIFO ordering (first-in, first-out)

##### `close()`

```rust
pub fn close(&self)
```

Closes the mailbox, preventing new messages.

**Behavior:**
- Pending messages can still be dequeued
- New `enqueue()` calls will fail
- Dequeue returns `None` when empty

### Struct: `BoundedMailbox<A>`

```rust
pub struct BoundedMailbox<A: Actor> {
    // fields omitted
}
```

Fixed-capacity mailbox with backpressure support.

**Implementation:**
- Uses `tokio::sync::mpsc::channel` internally
- Configurable backpressure strategies
- Memory-bounded operation

#### Methods

##### `with_backpressure()`

```rust
pub fn with_backpressure(capacity: usize, strategy: BackpressureStrategy) -> Self
```

Creates a bounded mailbox with custom backpressure strategy.

**Parameters:**
- `capacity`: Maximum messages
- `strategy`: Backpressure behavior

**Example:**

```rust
use airssys_rt::mailbox::{BoundedMailbox, BackpressureStrategy};

let mailbox = BoundedMailbox::with_backpressure(
    1000,
    BackpressureStrategy::DropOldest,
);
```

##### `len()`

```rust
pub fn len(&self) -> usize
```

Returns the current number of messages in the mailbox.

**Returns:**
- `usize`: Message count

**Use Cases:**
- Monitoring mailbox pressure
- Load balancing decisions
- Health checks

##### `is_full()`

```rust
pub fn is_full(&self) -> bool
```

Checks if the mailbox is at capacity.

**Returns:**
- `true`: Mailbox is full
- `false`: Mailbox has available capacity

### Struct: `UnboundedMailbox<A>`

```rust
pub struct UnboundedMailbox<A: Actor> {
    // fields omitted
}
```

Unlimited-capacity mailbox.

**Implementation:**
- Uses `tokio::sync::mpsc::unbounded_channel` internally
- No backpressure (will grow unbounded)
- Faster enqueue than bounded (no capacity check)

**Warning:**
- Can consume unbounded memory under load
- Recommend monitoring message queue length
- Consider bounded mailbox for production systems

### Enum: `BackpressureStrategy`

```rust
pub enum BackpressureStrategy {
    Block,
    DropOldest,
    DropNewest,
    Fail,
}
```

Strategy for handling mailbox overflow.

**Variants:**

- `Block`: Block sender until space available (default)
- `DropOldest`: Remove oldest message to make room
- `DropNewest`: Drop the incoming message
- `Fail`: Return error to sender

**Tradeoffs:**

| Strategy | Latency | Throughput | Data Loss | Use Case |
|----------|---------|------------|-----------|----------|
| Block | Variable | Lower | None | Critical messages |
| DropOldest | Constant | Higher | Oldest | Latest-value semantics |
| DropNewest | Constant | Highest | Newest | Best-effort delivery |
| Fail | Constant | Highest | Newest | Explicit error handling |

**Example:**

```rust
use airssys_rt::mailbox::{BoundedMailbox, BackpressureStrategy};

// Critical financial transactions - never drop
let ledger_mailbox = BoundedMailbox::with_backpressure(
    1000,
    BackpressureStrategy::Block,
);

// Sensor readings - only care about latest
let sensor_mailbox = BoundedMailbox::with_backpressure(
    100,
    BackpressureStrategy::DropOldest,
);

// Best-effort notifications
let notification_mailbox = BoundedMailbox::with_backpressure(
    500,
    BackpressureStrategy::Fail,
);
```

## Communication Patterns

### Fire-and-Forget

Send a message without waiting for response.

```rust
// Message with no result
struct Notify {
    event: String,
}
impl Message for Notify {
    type Result = ();
}

// Send without waiting
actor_ref.tell(Notify {
    event: "user_logged_in".to_string(),
})?;
```

### Request-Reply

Synchronous-style communication.

```rust
struct GetBalance {
    account_id: u64,
}
impl Message for GetBalance {
    type Result = f64;
}

let balance = actor_ref.send(GetBalance { account_id: 123 }).await?;
```

### Broadcast

Send to multiple actors.

```rust
struct HealthCheck;
impl Message for HealthCheck {
    type Result = bool;
}

let results = broker.broadcast(HealthCheck).await;
```

### Actor Pool

Load balance across workers.

```rust
struct WorkItem {
    data: Vec<u8>,
}
impl Message for WorkItem {
    type Result = Vec<u8>;
}

// Round-robin distribution
let worker = pool.get_worker();
let result = worker.send(WorkItem { data }).await?;
```

## Performance Characteristics

### Message Passing Latency

| Metric | Latency | Measurement |
|--------|---------|-------------|
| Message enqueue | 181ns | Mailbox send operation |
| Message dequeue | 150ns | Mailbox receive operation |
| Roundtrip (send+receive) | 737ns | Full message cycle |
| Cross-thread message | 850ns | With thread context switch |

### Throughput

| Configuration | Messages/sec | Notes |
|---------------|--------------|-------|
| Single actor | 4.7M | Single-threaded |
| 4 actors (no contention) | 18M | Linear scaling |
| 16 actors (high contention) | 45M | Sublinear scaling |
| Broadcast (10 actors) | 12M | Parallel delivery |

### Memory Usage

| Component | Size | Notes |
|-----------|------|-------|
| Message (avg) | 64 bytes | Varies by type |
| BoundedMailbox | 8KB | Capacity 1000 |
| UnboundedMailbox | 128 bytes | Plus message storage |
| Broker entry | 48 bytes | Per registered actor |

## Error Types

### Enum: `MailboxError`

```rust
pub enum MailboxError {
    Full,
    Closed,
    Timeout,
}
```

Errors specific to mailbox operations.

**Variants:**

- `Full`: Bounded mailbox at capacity
- `Closed`: Mailbox has been closed
- `Timeout`: Operation timed out

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

- `AlreadyRegistered`: Actor ID already registered
- `NotFound`: Actor not found in broker
- `RoutingFailed(String)`: Message routing failed

## See Also

- [Core API Reference](core.md) - Core types and system API
- [Actors API Reference](actors.md) - Actor types and patterns
- [Mailbox API Reference](mailbox.md) - Detailed mailbox API
- [Broker API Reference](broker.md) - Detailed broker API
- [Architecture: Message Passing](../../architecture/message-passing.md) - Design overview
- [How-To: Message Passing](../../guides/message-passing.md) - Usage guide
