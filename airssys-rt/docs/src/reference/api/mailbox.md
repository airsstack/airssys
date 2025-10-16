# Mailbox API Reference

This reference documents the mailbox system for actor message queuing and delivery.

## Module: `mailbox`

Mailbox implementations and configuration.

### Enum: `Mailbox<A>`

```rust
pub enum Mailbox<A: Actor> {
    Bounded(BoundedMailbox<A>),
    Unbounded(UnboundedMailbox<A>),
}
```

Actor mailbox abstraction supporting bounded and unbounded message queues.

**Type Parameters:**
- `A`: The actor type that owns this mailbox

**Variants:**

- `Bounded(BoundedMailbox<A>)`: Fixed-capacity mailbox with backpressure
- `Unbounded(UnboundedMailbox<A>)`: Unlimited capacity mailbox

**Choosing a Mailbox Type:**

| Criteria | Bounded | Unbounded |
|----------|---------|-----------|
| Memory safety | ✅ Guaranteed | ⚠️ Can grow unbounded |
| Backpressure | ✅ Supported | ❌ None |
| Latency predictability | ✅ High | ⚠️ Variable |
| Throughput | Medium | ✅ High |
| Configuration complexity | Higher | ✅ Simple |

**Recommendation:**
- **Production systems**: Use `Bounded` with appropriate capacity
- **Development/prototyping**: `Unbounded` for simplicity
- **Critical path**: `Bounded` with `BackpressureStrategy::Block`
- **Best-effort delivery**: `Bounded` with `BackpressureStrategy::DropOldest`

#### Constructors

##### `bounded()`

```rust
pub fn bounded(capacity: usize) -> Self
```

Creates a bounded mailbox with specified capacity.

**Parameters:**
- `capacity`: Maximum number of messages (must be > 0)

**Default Configuration:**
- Backpressure: `BackpressureStrategy::Block`
- Overflow behavior: Sender blocks until space available

**Example:**

```rust
use airssys_rt::mailbox::Mailbox;

// Standard capacity for most actors
let mailbox = Mailbox::<MyActor>::bounded(1000);

// High-throughput actor
let high_capacity = Mailbox::<Worker>::bounded(10000);

// Low-latency actor
let low_capacity = Mailbox::<Controller>::bounded(100);
```

##### `unbounded()`

```rust
pub fn unbounded() -> Self
```

Creates an unbounded mailbox with unlimited capacity.

**Returns:**
- `Mailbox<A>`: Unbounded mailbox instance

**Memory Characteristics:**
- Initial allocation: ~128 bytes
- Growth: Dynamic based on message count
- No upper limit (can exhaust memory)

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
- `Err(MailboxError::Full)`: Bounded mailbox at capacity (strategy-dependent)
- `Err(MailboxError::Closed)`: Mailbox has been closed

**Behavior by Mailbox Type:**

| Type | Behavior | Latency |
|------|----------|---------|
| Bounded (Block) | Waits for space | Variable |
| Bounded (Drop) | Drops message per strategy | Constant (~181ns) |
| Unbounded | Always succeeds | Constant (~150ns) |

**Performance:**
- Bounded mailbox: ~181ns average
- Unbounded mailbox: ~150ns average
- Cross-thread overhead: +50-100ns

**Example:**

```rust
use airssys_rt::Message;

struct WorkItem {
    data: Vec<u8>,
}
impl Message for WorkItem {
    type Result = ();
}

let msg = Box::new(WorkItem { data: vec![1, 2, 3] });
mailbox.enqueue(msg).await?;
```

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
- Atomic operation (no lost messages)

**Performance:**
- Average latency: ~150ns
- No allocation (returns existing Box)

**Example:**

```rust
while let Some(msg) = mailbox.dequeue().await {
    // Process message
    println!("Received message");
}
// Mailbox closed and empty
```

##### `try_dequeue()`

```rust
pub fn try_dequeue(&self) -> Option<Box<dyn Message>>
```

Attempts to dequeue a message without blocking.

**Returns:**
- `Some(msg)`: Message available
- `None`: Mailbox is empty (or closed)

**Use Cases:**
- Non-blocking message processing loops
- Polling-based designs
- Integration with custom event loops

**Example:**

```rust
// Process available messages, don't wait
while let Some(msg) = mailbox.try_dequeue() {
    // Process immediately available messages
}
// No messages available, continue with other work
```

##### `close()`

```rust
pub fn close(&self)
```

Closes the mailbox, preventing new messages.

**Behavior:**
- Pending messages can still be dequeued
- New `enqueue()` calls will return `Err(MailboxError::Closed)`
- `dequeue()` returns `None` when empty
- Idempotent (safe to call multiple times)

**Example:**

```rust
// Actor shutdown
mailbox.close();

// Process remaining messages
while let Some(msg) = mailbox.dequeue().await {
    // Handle final messages
}
```

##### `is_closed()`

```rust
pub fn is_closed(&self) -> bool
```

Checks if the mailbox is closed.

**Returns:**
- `true`: Mailbox is closed
- `false`: Mailbox is open

**Example:**

```rust
if mailbox.is_closed() {
    println!("Mailbox has been shut down");
}
```

## Bounded Mailbox

### Struct: `BoundedMailbox<A>`

```rust
pub struct BoundedMailbox<A: Actor> {
    // fields omitted
}
```

Fixed-capacity mailbox with configurable backpressure.

**Implementation Details:**
- Uses `tokio::sync::mpsc::channel` internally
- Lock-free send/receive operations
- Memory-bounded operation

**Type Parameters:**
- `A`: The actor type

#### Constructors

##### `new()`

```rust
pub fn new(capacity: usize) -> Self
```

Creates a bounded mailbox with default backpressure (Block).

**Parameters:**
- `capacity`: Maximum messages (must be > 0)

**Example:**

```rust
use airssys_rt::mailbox::BoundedMailbox;

let mailbox = BoundedMailbox::<MyActor>::new(1000);
```

##### `with_backpressure()`

```rust
pub fn with_backpressure(capacity: usize, strategy: BackpressureStrategy) -> Self
```

Creates a bounded mailbox with custom backpressure strategy.

**Parameters:**
- `capacity`: Maximum messages
- `strategy`: Backpressure behavior on overflow

**Example:**

```rust
use airssys_rt::mailbox::{BoundedMailbox, BackpressureStrategy};

// Drop oldest messages when full (ring buffer behavior)
let mailbox = BoundedMailbox::with_backpressure(
    100,
    BackpressureStrategy::DropOldest,
);

// Fail fast when full
let mailbox = BoundedMailbox::with_backpressure(
    1000,
    BackpressureStrategy::Fail,
);
```

#### Methods

##### `capacity()`

```rust
pub fn capacity(&self) -> usize
```

Returns the maximum capacity of the mailbox.

**Returns:**
- `usize`: Maximum message count

**Example:**

```rust
let cap = mailbox.capacity();
println!("Mailbox can hold {} messages", cap);
```

##### `len()`

```rust
pub fn len(&self) -> usize
```

Returns the current number of messages in the mailbox.

**Returns:**
- `usize`: Current message count

**Use Cases:**
- Monitoring mailbox pressure
- Load balancing decisions
- Health checks and metrics

**Example:**

```rust
let current = mailbox.len();
let capacity = mailbox.capacity();
let utilization = (current as f64 / capacity as f64) * 100.0;
println!("Mailbox utilization: {:.1}%", utilization);
```

##### `is_empty()`

```rust
pub fn is_empty(&self) -> bool
```

Checks if the mailbox contains no messages.

**Returns:**
- `true`: No messages
- `false`: Has messages

##### `is_full()`

```rust
pub fn is_full(&self) -> bool
```

Checks if the mailbox is at capacity.

**Returns:**
- `true`: At maximum capacity
- `false`: Has available space

**Example:**

```rust
if mailbox.is_full() {
    println!("Warning: Mailbox at capacity, backpressure active");
}
```

##### `available_capacity()`

```rust
pub fn available_capacity(&self) -> usize
```

Returns the number of messages that can be enqueued without blocking.

**Returns:**
- `usize`: Available slots

**Example:**

```rust
let available = mailbox.available_capacity();
println!("Can send {} more messages without blocking", available);
```

## Unbounded Mailbox

### Struct: `UnboundedMailbox<A>`

```rust
pub struct UnboundedMailbox<A: Actor> {
    // fields omitted
}
```

Unlimited-capacity mailbox.

**Implementation Details:**
- Uses `tokio::sync::mpsc::unbounded_channel` internally
- No capacity checks (faster enqueue)
- Can grow to system memory limits

**Warning:**
- No backpressure protection
- Can consume unbounded memory under sustained load
- Monitor `len()` in production systems

**Type Parameters:**
- `A`: The actor type

#### Constructors

##### `new()`

```rust
pub fn new() -> Self
```

Creates a new unbounded mailbox.

**Example:**

```rust
use airssys_rt::mailbox::UnboundedMailbox;

let mailbox = UnboundedMailbox::<MyActor>::new();
```

#### Methods

##### `len()`

```rust
pub fn len(&self) -> usize
```

Returns the current number of messages in the mailbox.

**Returns:**
- `usize`: Current message count

**Monitoring:**

```rust
let len = mailbox.len();
if len > 10000 {
    eprintln!("Warning: Unbounded mailbox has {} messages", len);
}
```

##### `is_empty()`

```rust
pub fn is_empty(&self) -> bool
```

Checks if the mailbox contains no messages.

## Backpressure Strategies

### Enum: `BackpressureStrategy`

```rust
pub enum BackpressureStrategy {
    Block,
    DropOldest,
    DropNewest,
    Fail,
}
```

Strategy for handling mailbox overflow in bounded mailboxes.

**Variants:**

- `Block`: Block sender until space available (default)
- `DropOldest`: Remove oldest message to make room for new message
- `DropNewest`: Drop the incoming message, keep existing messages
- `Fail`: Return error to sender without enqueueing

**Detailed Comparison:**

| Strategy | Sender Latency | Message Loss | Use Case |
|----------|----------------|--------------|----------|
| `Block` | Variable (0-∞) | None | Critical messages, ordered processing |
| `DropOldest` | Constant (~181ns) | Oldest messages | Latest-value semantics (sensors, status) |
| `DropNewest` | Constant (~181ns) | Newest messages | Preserve history (audit logs) |
| `Fail` | Constant (~181ns) | Newest messages | Explicit error handling required |

**Examples:**

```rust
use airssys_rt::mailbox::{BoundedMailbox, BackpressureStrategy};

// Financial transactions - never drop, wait for processing
let ledger = BoundedMailbox::with_backpressure(
    1000,
    BackpressureStrategy::Block,
);

// Temperature sensor - only latest reading matters
let sensor = BoundedMailbox::with_backpressure(
    10,
    BackpressureStrategy::DropOldest,
);

// Audit log - preserve oldest entries
let audit = BoundedMailbox::with_backpressure(
    5000,
    BackpressureStrategy::DropNewest,
);

// Best-effort notifications
let notifications = BoundedMailbox::with_backpressure(
    100,
    BackpressureStrategy::Fail,
);
```

## Performance Characteristics

### Operation Latency

| Operation | Bounded | Unbounded | Notes |
|-----------|---------|-----------|-------|
| `enqueue()` | 181ns | 150ns | Average, no contention |
| `dequeue()` | 150ns | 150ns | Average, no contention |
| `try_dequeue()` | 50ns | 50ns | Non-blocking check |
| `len()` | 10ns | 10ns | Atomic read |
| `is_empty()` | 10ns | 10ns | Atomic read |

### Throughput

| Scenario | Messages/sec | Notes |
|----------|--------------|-------|
| Single sender, single receiver | 5.5M | Optimal case |
| 4 senders, single receiver | 4.2M | Contention overhead |
| Single sender, 4 receivers (broadcast) | 4.8M | Parallel dequeue |

### Memory Usage

| Component | Bounded (cap=1000) | Unbounded | Notes |
|-----------|-------------------|-----------|-------|
| Empty mailbox | ~8KB | ~128 bytes | Pre-allocated vs dynamic |
| Per message overhead | 64 bytes | 64 bytes | Box + metadata |
| Full mailbox (1000 msgs) | ~72KB | ~64KB | Plus message data |

### Backpressure Strategy Performance

| Strategy | Latency (empty) | Latency (full) | Message Loss |
|----------|----------------|----------------|--------------|
| Block | 181ns | Variable | 0% |
| DropOldest | 181ns | 181ns | Bounded |
| DropNewest | 181ns | 181ns | Bounded |
| Fail | 181ns | 181ns | Bounded |

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

- `Full`: Bounded mailbox at capacity (only with `BackpressureStrategy::Fail`)
- `Closed`: Mailbox has been closed
- `Timeout`: Operation timed out (when timeout specified)

**Example:**

```rust
use airssys_rt::mailbox::MailboxError;

match mailbox.enqueue(msg).await {
    Ok(()) => println!("Message sent"),
    Err(MailboxError::Full) => {
        eprintln!("Mailbox full, message rejected");
        // Handle rejection (retry, log, etc.)
    }
    Err(MailboxError::Closed) => {
        eprintln!("Actor has shut down");
        // Clean up sender
    }
    Err(MailboxError::Timeout) => {
        eprintln!("Send timed out");
        // Handle timeout
    }
}
```

## Monitoring and Metrics

### Mailbox Health Metrics

```rust
pub struct MailboxMetrics {
    pub capacity: usize,
    pub current_size: usize,
    pub total_enqueued: u64,
    pub total_dequeued: u64,
    pub messages_dropped: u64,
}

impl BoundedMailbox<A> {
    pub fn metrics(&self) -> MailboxMetrics {
        // Implementation
    }
}
```

**Usage:**

```rust
let metrics = mailbox.metrics();
println!("Mailbox utilization: {}/{}", metrics.current_size, metrics.capacity);
println!("Throughput: {} msgs/sec", 
    (metrics.total_dequeued as f64 / uptime.as_secs() as f64));
println!("Drop rate: {:.2}%", 
    (metrics.messages_dropped as f64 / metrics.total_enqueued as f64) * 100.0);
```

## Testing Utilities

### Struct: `MailboxTestProbe`

```rust
pub struct MailboxTestProbe<A: Actor> {
    // fields omitted
}
```

Testing utility for mailbox behavior.

**Available in:** Test builds only (`#[cfg(test)]`)

#### Methods

##### `new()`

```rust
pub fn new(mailbox: Mailbox<A>) -> Self
```

Creates a test probe for a mailbox.

##### `assert_enqueued()`

```rust
pub fn assert_enqueued(&self, expected: usize) -> bool
```

Asserts that expected number of messages were enqueued.

**Example:**

```rust
#[cfg(test)]
mod tests {
    use super::*;
    use airssys_rt::mailbox::MailboxTestProbe;
    
    #[tokio::test]
    async fn test_mailbox_capacity() {
        let mailbox = Mailbox::<MyActor>::bounded(10);
        let probe = MailboxTestProbe::new(mailbox.clone());
        
        // Enqueue messages
        for i in 0..10 {
            mailbox.enqueue(Box::new(TestMsg { id: i })).await.unwrap();
        }
        
        assert!(probe.assert_enqueued(10));
    }
}
```

## See Also

- [Core API Reference](core.md) - Core types and system
- [Messaging API Reference](messaging.md) - Message broker and patterns
- [Broker API Reference](broker.md) - Message routing
- [Architecture: Message Passing](../../architecture/message-passing.md) - System design
- [BENCHMARKING.md](../../../BENCHMARKING.md) - Performance data
