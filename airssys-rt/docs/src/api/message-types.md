# Message Types

This section documents the message system types in `airssys-rt`.

## Message Trait

All messages must implement this trait.

```rust
pub trait Message: Clone + Send + Sync + 'static 
    + serde::Serialize + for<'de> serde::Deserialize<'de> 
{
    const MESSAGE_TYPE: &'static str;
}
```

### Requirements

- **`Clone`**: Messages can be cloned for broadcasting
- **`Send + Sync + 'static`**: Messages can be sent across threads
- **`Serialize + Deserialize`**: Messages can be serialized (for persistence, network, etc.)
- **`MESSAGE_TYPE`**: Unique string identifier for the message type

### Example

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

## MessageEnvelope

Wrapper for messages with metadata.

```rust
pub struct MessageEnvelope<M> {
    pub id: MessageId,
    pub message: M,
    pub timestamp: DateTime<Utc>,
    pub reply_to: Option<ActorAddress>,
}
```

### Methods

```rust
impl<M: Message> MessageEnvelope<M> {
    /// Create a new message envelope
    pub fn new(message: M) -> Self;
}
```

### Fields

- **`id`**: Unique identifier for this message
- **`message`**: The actual message payload
- **`timestamp`**: When the message was created (using chrono DateTime<Utc>)
- **`reply_to`**: Optional address for replies

### Example

```rust
use airssys_rt::message::MessageEnvelope;

let message = MyMessage { data: "hello".to_string() };
let envelope = MessageEnvelope::new(message);

println!("Message ID: {:?}", envelope.id);
println!("Timestamp: {}", envelope.timestamp);
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