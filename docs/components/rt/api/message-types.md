# Message Types

This section documents the message system types in `airssys-rt`.

## Message Trait

All messages must implement this trait.

```rust
pub trait Message: Send + Sync + Clone + Debug + 'static {
    const MESSAGE_TYPE: &'static str;
    
    fn priority(&self) -> MessagePriority {
        MessagePriority::Normal
    }
}
```

### Requirements

- **`Clone`**: Messages can be cloned for broadcasting
- **`Send + Sync + 'static`**: Messages can be sent across threads
- **`Debug`**: Messages can be debugged and logged
- **`MESSAGE_TYPE`**: Unique string identifier for the message type
- **`priority()`**: Optional method to override message priority (default: Normal)

**Note:** `Serialize + Deserialize` are **NOT** required by the trait. Add them only when you need serialization (e.g., for network transport or persistence).

### Example Without Serialization

```rust
use airssys_rt::Message;

#[derive(Debug, Clone)]
struct CounterMessage {
    delta: i32,
}

impl Message for CounterMessage {
    const MESSAGE_TYPE: &'static str = "counter";
}
```

### Example With Serialization

```rust
use airssys_rt::Message;
use serde::{Serialize, Deserialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
struct CounterMessage {
    delta: i32,
}

impl Message for CounterMessage {
    const MESSAGE_TYPE: &'static str = "counter";
}
```

### Example With Priority

```rust
use airssys_rt::message::{Message, MessagePriority};

#[derive(Debug, Clone)]
struct ShutdownMessage;

impl Message for ShutdownMessage {
    const MESSAGE_TYPE: &'static str = "shutdown";
    
    fn priority(&self) -> MessagePriority {
        MessagePriority::Critical  // Processed before other messages
    }
}
```

## MessageEnvelope

Wrapper for messages with metadata.

```rust
pub struct MessageEnvelope<M: Message> {
    /// The actual message payload
    pub payload: M,

    /// Optional sender address for reply capability
    pub sender: Option<ActorAddress>,

    /// Optional recipient for reply-to pattern
    pub reply_to: Option<ActorAddress>,

    /// Message creation timestamp (UTC)
    pub timestamp: DateTime<Utc>,

    /// Optional correlation ID for request/response tracking
    pub correlation_id: Option<Uuid>,

    /// Message priority (extracted from payload)
    pub priority: MessagePriority,

    /// Optional time-to-live in seconds
    pub ttl: Option<u64>,
}
```

### Methods

```rust
impl<M: Message> MessageEnvelope<M> {
    /// Create new envelope with current timestamp
    pub fn new(payload: M) -> Self;
    
    /// Builder methods for adding metadata
    pub fn with_sender(self, sender: ActorAddress) -> Self;
    pub fn with_reply_to(self, reply_to: ActorAddress) -> Self;
    pub fn with_correlation_id(self, id: Uuid) -> Self;
    pub fn with_ttl(self, ttl_seconds: u64) -> Self;
    
    /// Check if message has expired based on TTL
    pub fn is_expired(&self) -> bool;
    
    /// Get message type string
    pub fn message_type(&self) -> &'static str;
}
```

### Fields

- **`payload`**: The actual message (generic type M)
- **`sender`**: Optional sender address for replies
- **`reply_to`**: Optional recipient for reply messages
- **`timestamp`**: Message creation time (UTC, using chrono DateTime<Utc>)
- **`correlation_id`**: Optional UUID for request/response tracking
- **`priority`**: Message priority (from payload.priority())
- **`ttl`**: Optional time-to-live in seconds

### Example

```rust
use airssys_rt::message::MessageEnvelope;
use airssys_rt::util::ActorAddress;
use uuid::Uuid;

let message = MyMessage { data: "hello".to_string() };
let sender = ActorAddress::named("sender");

let envelope = MessageEnvelope::new(message)
    .with_sender(sender)
    .with_correlation_id(Uuid::new_v4())
    .with_ttl(60); // 60 seconds TTL

println!("Message type: {}", envelope.message_type());
println!("Timestamp: {}", envelope.timestamp);
println!("Expired: {}", envelope.is_expired());
```

## MessageId

Unique identifier for messages.

```rust
pub struct MessageId(uuid::Uuid);
```

### Methods

```rust
impl MessageId {
    /// Generate a new unique message ID
    pub fn new() -> Self;
}
```

### Traits

Implements: `Clone`, `Copy`, `Debug`, `PartialEq`, `Eq`, `Hash`

## Message Patterns

### Simple Messages

Messages without responses:

```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
struct LogMessage {
    level: String,
    text: String,
}

impl Message for LogMessage {
    const MESSAGE_TYPE: &'static str = "log";
}
```

### Request-Response Pattern

Messages that need responses (future feature):

```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
struct CalculateRequest {
    x: i32,
    y: i32,
}

impl Message for CalculateRequest {
    const MESSAGE_TYPE: &'static str = "calculate";
}

// Response would be sent via separate mechanism
```

### Event Messages

Messages for event broadcasting:

```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
struct UserCreatedEvent {
    user_id: String,
    timestamp: DateTime<Utc>,
}

impl Message for UserCreatedEvent {
    const MESSAGE_TYPE: &'static str = "user.created";
}
```

## Message Design Guidelines

### 1. Keep Messages Small

Messages are cloned, so keep them lightweight:

```rust
// Good: Small, focused message
#[derive(Debug, Clone, Serialize, Deserialize)]
struct FilePathMessage {
    path: String,
}

// Avoid: Large data in messages
#[derive(Debug, Clone, Serialize, Deserialize)]
struct FileDataMessage {
    data: Vec<u8>,  // Could be megabytes!
}
```

For large data, consider using `Arc`:

```rust
use std::sync::Arc;

#[derive(Debug, Clone, Serialize, Deserialize)]
struct SharedDataMessage {
    #[serde(skip)]  // Skip serialization for Arc
    data: Arc<Vec<u8>>,
}
```

### 2. Use Descriptive Names

Message type strings should be clear and unique:

```rust
impl Message for UserMessage {
    const MESSAGE_TYPE: &'static str = "user.action";  // Good: namespaced
}
```

### 3. Version Your Messages

For evolving systems, consider message versioning:

```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
struct UserMessageV2 {
    version: u32,
    data: String,
}

impl Message for UserMessageV2 {
    const MESSAGE_TYPE: &'static str = "user.v2";
}
```

## Complete Example

```rust
use airssys_rt::{Actor, ActorContext, Message, MessageBroker};
use async_trait::async_trait;
use serde::{Serialize, Deserialize};

// Define message
#[derive(Debug, Clone, Serialize, Deserialize)]
struct WorkMessage {
    task_id: String,
    priority: u8,
}

impl Message for WorkMessage {
    const MESSAGE_TYPE: &'static str = "work";
}

// Use in actor
struct WorkerActor;

#[async_trait]
impl Actor for WorkerActor {
    type Message = WorkMessage;
    type Error = std::io::Error;

    async fn handle_message<B: MessageBroker<Self::Message>>(
        &mut self,
        message: Self::Message,
        context: &mut ActorContext<Self::Message, B>,
    ) -> Result<(), Self::Error> {
        println!("Processing task {} (priority: {})", 
            message.task_id, message.priority);
        context.record_message();
        Ok(())
    }
}
```

See `examples/actor_basic.rs` for complete working examples.