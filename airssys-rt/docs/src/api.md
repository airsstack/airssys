# API Reference

This API reference documents the actual, implemented APIs in `airssys-rt`. All code examples are from real, working implementations.

> **💡 Tip**: For the most up-to-date API documentation, run `cargo doc --open --package airssys-rt` to view the generated Rustdoc.

## Core API Overview

The `airssys-rt` API is organized into several key modules:

- **`actor`** - Core actor trait and context
- **`message`** - Message types and envelopes
- **`broker`** - Message broker for pub/sub
- **`mailbox`** - Message queue implementations
- **`supervisor`** - Supervision and fault tolerance
- **`monitoring`** - Health checks and metrics
- **`util`** - Utilities (addressing, IDs)

## Actor API

### Actor Trait

The foundational trait for all actors (from `src/actor/traits.rs`):

```rust
use airssys_rt::{Actor, ActorContext, Message, ErrorAction};
use async_trait::async_trait;

#[async_trait]
pub trait Actor: Send + Sync + 'static {
    /// The type of messages this actor handles
    type Message: Message;

    /// The error type for actor operations
    type Error: Error + Send + Sync + 'static;

    /// Handle an incoming message
    async fn handle_message<B: MessageBroker<Self::Message>>(
        &mut self,
        message: Self::Message,
        context: &mut ActorContext<Self::Message, B>,
    ) -> Result<(), Self::Error>;

    /// Lifecycle hook: called before actor starts (optional)
    async fn pre_start<B: MessageBroker<Self::Message>>(
        &mut self,
        context: &mut ActorContext<Self::Message, B>,
    ) -> Result<(), Self::Error> {
        Ok(())
    }

    /// Lifecycle hook: called when actor stops (optional)
    async fn post_stop<B: MessageBroker<Self::Message>>(
        &mut self,
        context: &mut ActorContext<Self::Message, B>,
    ) -> Result<(), Self::Error> {
        Ok(())
    }

    /// Error handler: return supervision decision (optional)
    async fn on_error<B: MessageBroker<Self::Message>>(
        &mut self,
        error: Self::Error,
        context: &mut ActorContext<Self::Message, B>,
    ) -> ErrorAction {
        ErrorAction::Restart
    }
}
```

### ActorContext

Context provided to actors during message handling:

```rust
pub struct ActorContext<M: Message, B: MessageBroker<M>> {
    // Fields are private, access via methods
}

impl<M: Message, B: MessageBroker<M>> ActorContext<M, B> {
    /// Get the actor's address
    pub fn address(&self) -> &ActorAddress;
    
    /// Get the actor's unique ID
    pub fn id(&self) -> &ActorId;
    
    /// Get when the actor was created
    pub fn created_at(&self) -> DateTime<Utc>;
    
    /// Get when the last message was processed
    pub fn last_message_at(&self) -> Option<DateTime<Utc>>;
    
    /// Get total number of messages processed
    pub fn message_count(&self) -> u64;
    
    /// Record that a message was processed
    pub fn record_message(&mut self);
    
    /// Send a message to another actor
    pub async fn send(&self, message: M, recipient: ActorAddress) 
        -> Result<(), String>;
}
```

### ActorLifecycle

State management for actor lifecycle:

```rust
pub enum ActorState {
    Starting,   // Actor is initializing
    Running,    // Actor is active
    Stopping,   // Actor is shutting down
    Stopped,    // Actor has stopped
    Failed,     // Actor has failed
}

pub struct ActorLifecycle {
    // Implementation details hidden
}

impl ActorLifecycle {
    pub fn new() -> Self;
    pub fn state(&self) -> ActorState;
    pub fn transition_to(&mut self, new_state: ActorState);
    pub fn restart_count(&self) -> u32;
    pub fn last_state_change(&self) -> DateTime<Utc>;
    pub fn is_terminal(&self) -> bool;
    pub fn is_running(&self) -> bool;
}
```

### ErrorAction

Control supervision behavior on errors:

```rust
pub enum ErrorAction {
    Resume,     // Continue processing (ignore error)
    Restart,    // Restart the actor
    Stop,       // Stop the actor permanently
    Escalate,   // Pass error to supervisor
}
```

## Message API

### Message Trait

All messages must implement this trait:

```rust
pub trait Message: Clone + Send + Sync + 'static 
    + serde::Serialize + for<'de> serde::Deserialize<'de> 
{
    const MESSAGE_TYPE: &'static str;
}
```

Example implementation:

```rust
use serde::{Serialize, Deserialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
struct MyMessage {
    data: String,
}

impl Message for MyMessage {
    const MESSAGE_TYPE: &'static str = "my_message";
}
```

### MessageEnvelope

Messages are wrapped in envelopes for routing:

```rust
pub struct MessageEnvelope<M> {
    pub id: MessageId,
    pub message: M,
    pub timestamp: DateTime<Utc>,
    pub reply_to: Option<ActorAddress>,
}

impl<M: Message> MessageEnvelope<M> {
    pub fn new(message: M) -> Self;
}
```

### MessageId

Unique identifier for messages:

```rust
pub struct MessageId(uuid::Uuid);

impl MessageId {
    pub fn new() -> Self;
}
```

## Broker API

### MessageBroker Trait

Pub/sub system for actor communication:

```rust
#[async_trait]
pub trait MessageBroker<M: Message>: Clone + Send + Sync + 'static {
    type Error: Error + Send + Sync + 'static;

    /// Publish a message to all subscribers
    async fn publish(&self, envelope: MessageEnvelope<M>) 
        -> Result<(), Self::Error>;
    
    /// Subscribe to messages for an actor
    async fn subscribe(&self, subscriber_id: ActorId) 
        -> Result<mpsc::Receiver<MessageEnvelope<M>>, Self::Error>;
}
```

### InMemoryMessageBroker

Current broker implementation:

```rust
use airssys_rt::broker::InMemoryMessageBroker;

let broker = InMemoryMessageBroker::<MyMessage>::new();
```

## Mailbox API

### Mailbox Types

Two mailbox implementations are provided:

**UnboundedMailbox**:
```rust
use airssys_rt::mailbox::UnboundedMailbox;

let mailbox = UnboundedMailbox::<MyMessage>::new();
```

**BoundedMailbox**:
```rust
use airssys_rt::mailbox::{BoundedMailbox, BackpressureStrategy};

let mailbox = BoundedMailbox::<MyMessage>::new(
    100,  // capacity
    BackpressureStrategy::Block,
);
```

### BackpressureStrategy

Control behavior when mailbox is full:

```rust
pub enum BackpressureStrategy {
    Block,      // Block sender when mailbox full
    Drop,       // Drop new messages when full
    DropOldest, // Drop oldest message to make room
}
```

### Mailbox Traits

Generic interfaces for mailboxes:

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

## Supervisor API

### Child Trait

Any entity can be supervised by implementing `Child`:

```rust
#[async_trait]
pub trait Child: Send + Sync {
    async fn start(&mut self) -> Result<(), Box<dyn Error + Send + Sync>>;
    async fn stop(&mut self) -> Result<(), Box<dyn Error + Send + Sync>>;
    async fn health_check(&self) -> ChildHealth;
}
```

Note: Actors automatically implement `Child` via blanket implementation.

### ChildSpec

Configuration for supervised children:

```rust
pub struct ChildSpec {
    pub id: ChildId,
    pub restart_policy: RestartPolicy,
    pub shutdown_policy: ShutdownPolicy,
    pub significant: bool,
}
```

### RestartPolicy

Control when children should be restarted:

```rust
pub enum RestartPolicy {
    Permanent,   // Always restart on failure
    Transient,   // Restart only on abnormal termination
    Temporary,   // Never restart
}
```

### RestartStrategy

Choose how supervisor handles child failures:

```rust
// OneForOne - Restart only the failed child
use airssys_rt::supervisor::OneForOne;
let strategy = OneForOne::new();

// OneForAll - Restart all children when one fails
use airssys_rt::supervisor::OneForAll;
let strategy = OneForAll::new();

// RestForOne - Restart failed child and those started after it
use airssys_rt::supervisor::RestForOne;
let strategy = RestForOne::new();
```

### SupervisorNode

Create and manage supervision trees:

```rust
use airssys_rt::supervisor::{SupervisorNode, SupervisorId};

let mut supervisor = SupervisorNode::new(
    SupervisorId::new(),
    OneForOne::new(),
);

supervisor.add_child(child_spec, child_instance).await?;
supervisor.start_all_children().await?;
```

### ChildHealth

Health status for monitored actors:

```rust
pub enum ChildHealth {
    Healthy,
    Unhealthy(String),  // Contains error message
}
```

## Addressing API

### ActorAddress

Identify actors by address:

```rust
pub struct ActorAddress {
    // Implementation details hidden
}

impl ActorAddress {
    /// Create anonymous address (UUID-based)
    pub fn anonymous() -> Self;
    
    /// Create named address
    pub fn named(name: impl Into<String>) -> Self;
    
    /// Get actor ID
    pub fn id(&self) -> &ActorId;
    
    /// Get actor name (if any)
    pub fn name(&self) -> Option<&str>;
}
```

### ActorId

Unique identifier using UUIDs:

```rust
pub struct ActorId(uuid::Uuid);

impl ActorId {
    pub fn new() -> Self;
}
```

## Complete API Example

Here's a complete example using the real APIs:

```rust
use airssys_rt::{
    Actor, ActorContext, Message, ErrorAction,
    broker::{InMemoryMessageBroker, MessageBroker},
    util::ActorAddress,
};
use async_trait::async_trait;
use serde::{Serialize, Deserialize};
use std::fmt;

// 1. Define message
#[derive(Debug, Clone, Serialize, Deserialize)]
struct WorkMessage {
    task: String,
}

impl Message for WorkMessage {
    const MESSAGE_TYPE: &'static str = "work";
}

// 2. Define actor
struct WorkerActor {
    completed: usize,
}

// 3. Define error
#[derive(Debug)]
struct WorkerError(String);

impl fmt::Display for WorkerError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "WorkerError: {}", self.0)
    }
}

impl std::error::Error for WorkerError {}

// 4. Implement Actor trait
#[async_trait]
impl Actor for WorkerActor {
    type Message = WorkMessage;
    type Error = WorkerError;

    async fn pre_start<B: MessageBroker<Self::Message>>(
        &mut self,
        context: &mut ActorContext<Self::Message, B>,
    ) -> Result<(), Self::Error> {
        println!("Worker {} starting", 
            context.address().name().unwrap_or("anonymous"));
        Ok(())
    }

    async fn handle_message<B: MessageBroker<Self::Message>>(
        &mut self,
        message: Self::Message,
        context: &mut ActorContext<Self::Message, B>,
    ) -> Result<(), Self::Error> {
        println!("Processing: {}", message.task);
        self.completed += 1;
        context.record_message();
        Ok(())
    }

    async fn on_error<B: MessageBroker<Self::Message>>(
        &mut self,
        error: Self::Error,
        _context: &mut ActorContext<Self::Message, B>,
    ) -> ErrorAction {
        eprintln!("Error occurred: {}", error);
        ErrorAction::Restart
    }
}

// 5. Use the actor
async fn run_actor() {
    let address = ActorAddress::named("worker-1");
    let broker = InMemoryMessageBroker::new();
    let mut context = ActorContext::new(address, broker);
    
    let mut actor = WorkerActor { completed: 0 };
    
    // Start actor
    actor.pre_start(&mut context).await.unwrap();
    
    // Handle message
    let message = WorkMessage { task: "job-1".to_string() };
    actor.handle_message(message, &mut context).await.unwrap();
    
    println!("Messages processed: {}", context.message_count());
}
```

## Further Reading

- **Generated Rustdoc**: Run `cargo doc --open --package airssys-rt` for complete API documentation
- **Examples**: See `examples/` directory for real, working code
- **Architecture**: Read [Core Concepts](./architecture/core-concepts.md) for design principles
- **Implementation Guide**: See [Implementation](./implementation.md) for practical usage patterns

All APIs documented here are implemented and tested. See the working examples for complete usage patterns.