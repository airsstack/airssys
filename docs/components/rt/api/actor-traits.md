# Actor Traits

This section documents the core traits that define actor behavior in `airssys-rt`.

## Actor Trait

The fundamental trait that all actors must implement.

```rust
#[async_trait]
pub trait Actor: Send + Sync + 'static {
    /// The type of messages this actor handles
    type Message: Message;

    /// The error type for actor operations
    type Error: Error + Send + Sync + 'static;

    /// Handle an incoming message (REQUIRED)
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
        ErrorAction::Stop  // Default: stop the actor
    }
}
```

### Required Methods

- **`handle_message`**: Process incoming messages. This is the core method where your actor's business logic lives.

### Optional Methods

- **`pre_start`**: Called once before the actor starts processing messages. Use for initialization (e.g., connecting to databases, loading resources).

- **`post_stop`**: Called when the actor stops. Use for cleanup (e.g., closing connections, releasing resources).

- **`on_error`**: Called when `handle_message` returns an error. Return an `ErrorAction` to control supervision behavior.

### Example

See `examples/actor_basic.rs` for a complete actor implementation.

```rust
use airssys_rt::{Actor, ActorContext, Message, ErrorAction};
use async_trait::async_trait;
use serde::{Serialize, Deserialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
struct MyMessage {
    data: String,
}

impl Message for MyMessage {
    const MESSAGE_TYPE: &'static str = "my_message";
}

struct MyActor {
    state: i32,
}

#[async_trait]
impl Actor for MyActor {
    type Message = MyMessage;
    type Error = std::io::Error;

    async fn pre_start<B: MessageBroker<Self::Message>>(
        &mut self,
        context: &mut ActorContext<Self::Message, B>,
    ) -> Result<(), Self::Error> {
        println!("Actor {} starting", context.address().name().unwrap_or("anonymous"));
        Ok(())
    }

    async fn handle_message<B: MessageBroker<Self::Message>>(
        &mut self,
        message: Self::Message,
        context: &mut ActorContext<Self::Message, B>,
    ) -> Result<(), Self::Error> {
        println!("Processing: {}", message.data);
        self.state += 1;
        context.record_message();
        Ok(())
    }

    async fn on_error<B: MessageBroker<Self::Message>>(
        &mut self,
        error: Self::Error,
        _context: &mut ActorContext<Self::Message, B>,
    ) -> ErrorAction {
        eprintln!("Error: {}", error);
        ErrorAction::Stop  // Stop on error (can customize based on error type)
    }
}
```

## Child Trait

Trait for entities that can be supervised.

```rust
#[async_trait]
pub trait Child: Send + Sync {
    /// Start the child entity
    async fn start(&mut self) -> Result<(), Box<dyn Error + Send + Sync>>;
    
    /// Stop the child entity
    async fn stop(&mut self) -> Result<(), Box<dyn Error + Send + Sync>>;
    
    /// Check the health status
    async fn health_check(&self) -> ChildHealth;
}
```

**Note**: Actors automatically implement `Child` via a blanket implementation, so you don't need to implement this manually for actors.

### Example (Custom Child)

```rust
use airssys_rt::supervisor::{Child, ChildHealth};
use async_trait::async_trait;

struct Worker {
    id: String,
}

#[async_trait]
impl Child for Worker {
    async fn start(&mut self) -> Result<(), Box<dyn Error + Send + Sync>> {
        println!("Worker {} started", self.id);
        Ok(())
    }

    async fn stop(&mut self) -> Result<(), Box<dyn Error + Send + Sync>> {
        println!("Worker {} stopped", self.id);
        Ok(())
    }

    async fn health_check(&self) -> ChildHealth {
        ChildHealth::Healthy
    }
}
```

See `examples/supervisor_basic.rs` for complete usage.

## MessageBroker Trait

Trait for message routing and pub/sub.

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

### Implementations

- **`InMemoryMessageBroker<M>`**: Default in-memory implementation using channels

### Example

```rust
use airssys_rt::broker::InMemoryMessageBroker;

let broker = InMemoryMessageBroker::<MyMessage>::new();
```

## Mailbox Traits

### MailboxReceiver

```rust
#[async_trait]
pub trait MailboxReceiver<M: Message>: Send {
    /// Receive next message (blocking)
    async fn recv(&mut self) -> Option<MessageEnvelope<M>>;
    
    /// Try to receive message (non-blocking)
    fn try_recv(&mut self) -> Result<MessageEnvelope<M>, TryRecvError>;
}
```

### MailboxSender

```rust
#[async_trait]
pub trait MailboxSender<M: Message>: Clone + Send + Sync {
    /// Send a message to the mailbox
    async fn send(&self, envelope: MessageEnvelope<M>) 
        -> Result<(), MailboxError>;
}
```

### Implementations

- **`UnboundedMailbox<M>`**: Unlimited capacity mailbox
- **`BoundedMailbox<M>`**: Fixed capacity mailbox with backpressure

All traits use `async_trait` for async method support. See the generated Rustdoc (`cargo doc --open`) for complete trait documentation.