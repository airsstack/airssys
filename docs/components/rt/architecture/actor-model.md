# Actor Model Design

The actor model in `airssys-rt` provides a type-safe, performant implementation of the actor pattern based on Erlang/OTP principles.

> **Note**: All code examples are taken from the actual implementation. See [examples directory](../../examples/) for complete working code.

## Actor Trait Architecture

### Core Actor Trait

The foundational trait that all actors implement (from `src/actor/traits.rs`):

```rust
#[async_trait]
pub trait Actor: Send + Sync + 'static {
    /// The type of messages this actor can handle.
    type Message: Message;

    /// The error type returned by actor operations.
    type Error: Error + Send + Sync + 'static;

    /// Handle an incoming message.
    async fn handle_message<B: MessageBroker<Self::Message>>(
        &mut self,
        message: Self::Message,
        context: &mut ActorContext<Self::Message, B>,
    ) -> Result<(), Self::Error>;

    /// Called when the actor is started (optional).
    async fn pre_start<B: MessageBroker<Self::Message>>(
        &mut self,
        context: &mut ActorContext<Self::Message, B>,
    ) -> Result<(), Self::Error> {
        Ok(())
    }

    /// Called when the actor is stopping (optional).
    async fn post_stop<B: MessageBroker<Self::Message>>(
        &mut self,
        context: &mut ActorContext<Self::Message, B>,
    ) -> Result<(), Self::Error> {
        Ok(())
    }

    /// Handle errors and return supervision decision (optional).
    async fn on_error<B: MessageBroker<Self::Message>>(
        &mut self,
        error: Self::Error,
        context: &mut ActorContext<Self::Message, B>,
    ) -> ErrorAction {
        ErrorAction::Restart
    }
}
```

### Design Principles

1. **Generic Constraints** (§6.2): Uses `<B: MessageBroker<Self::Message>>` instead of `dyn` trait objects
2. **Associated Types**: `Message` and `Error` types for compile-time type safety
3. **Lifecycle Hooks**: Optional `pre_start` and `post_stop` for initialization and cleanup
4. **Supervision Integration**: `on_error` returns `ErrorAction` for fault tolerance

## Actor State Encapsulation

Actors maintain private state that can only be modified through message handling. Example from `examples/actor_basic.rs`:

```rust
struct CounterActor {
    value: i32,        // Private state
    max_value: i32,    // Configuration
}

#[async_trait]
impl Actor for CounterActor {
    type Message = CounterMessage;
    type Error = CounterError;

    async fn handle_message<B: MessageBroker<Self::Message>>(
        &mut self,
        message: Self::Message,
        context: &mut ActorContext<Self::Message, B>,
    ) -> Result<(), Self::Error> {
        // Only way to modify state
        self.value += message.delta;

        if self.value > self.max_value {
            return Err(CounterError {
                message: format!("Value {} exceeds maximum {}", 
                    self.value, self.max_value),
            });
        }

        context.record_message();
        Ok(())
    }
}
```

Key principles:
- State fields are private (not `pub`)
- State only modified in `handle_message`
- No direct external access to state
- Thread safety through message passing

## Message Design

### Message Trait

All messages must implement the `Message` trait (from `src/message/mod.rs`):

```rust
pub trait Message: Clone + Send + Sync + 'static 
    + for<'de> serde::Deserialize<'de> + serde::Serialize 
{
    const MESSAGE_TYPE: &'static str;
}
```

### Message Implementation Example

```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
struct CounterMessage {
    delta: i32,
}

impl Message for CounterMessage {
    const MESSAGE_TYPE: &'static str = "counter";
}
```

### Message Envelope

Messages are wrapped for routing (from `src/message/envelope.rs`):

```rust
pub struct MessageEnvelope<M> {
    pub id: MessageId,
    pub message: M,
    pub timestamp: DateTime<Utc>,  // §3.2 chrono DateTime<Utc>
    pub reply_to: Option<ActorAddress>,
}

impl<M: Message> MessageEnvelope<M> {
    pub fn new(message: M) -> Self {
        Self {
            id: MessageId::new(),
            message,
            timestamp: Utc::now(),
            reply_to: None,
        }
    }
}
```


## Actor Lifecycle

### Lifecycle States

Actors transition through defined states (from `src/actor/lifecycle.rs`):

```rust
pub enum ActorState {
    Starting,   // Actor is initializing
    Running,    // Actor is active and processing messages
    Stopping,   // Actor is shutting down
    Stopped,    // Actor has stopped successfully
    Failed,     // Actor has failed (requires supervision)
}
```

### Lifecycle Management

The `ActorLifecycle` struct provides state management:

```rust
#[derive(Debug, Clone)]
pub struct ActorLifecycle {
    state: ActorState,
    last_state_change: DateTime<Utc>,
    restart_count: u32,
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

### Lifecycle Hooks

Actors can override lifecycle hooks:

```rust
#[async_trait]
impl Actor for MyActor {
    type Message = MyMessage;
    type Error = MyError;

    // Called before actor starts processing messages
    async fn pre_start<B: MessageBroker<Self::Message>>(
        &mut self,
        context: &mut ActorContext<Self::Message, B>,
    ) -> Result<(), Self::Error> {
        println!("Actor {} starting", context.address().name().unwrap_or("anonymous"));
        // Initialize resources, connect to databases, etc.
        Ok(())
    }

    // Called when actor stops
    async fn post_stop<B: MessageBroker<Self::Message>>(
        &mut self,
        context: &mut ActorContext<Self::Message, B>,
    ) -> Result<(), Self::Error> {
        println!("Actor {} stopping", context.address().name().unwrap_or("anonymous"));
        // Cleanup resources, close connections, etc.
        Ok(())
    }

    async fn handle_message<B: MessageBroker<Self::Message>>(
        &mut self,
        message: Self::Message,
        context: &mut ActorContext<Self::Message, B>,
    ) -> Result<(), Self::Error> {
        // Process messages
        Ok(())
    }
}
```

See `examples/actor_lifecycle.rs` for a complete lifecycle example.

## Actor Addressing

### ActorAddress

Actors are identified by addresses (from `src/util/address.rs`):

```rust
pub struct ActorAddress {
    id: ActorId,
    name: Option<String>,
}

impl ActorAddress {
    pub fn anonymous() -> Self;
    pub fn named(name: impl Into<String>) -> Self;
    pub fn id(&self) -> &ActorId;
    pub fn name(&self) -> Option<&str>;
}
```

### ActorId

Unique identifiers using UUIDs:

```rust
pub struct ActorId(uuid::Uuid);

impl ActorId {
    pub fn new() -> Self {
        Self(uuid::Uuid::new_v4())
    }
}
```

## Message Broker System

### MessageBroker Trait

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

### InMemoryMessageBroker

Current implementation using channels (from `src/broker/in_memory.rs`):

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

## Communication via Context

Actors send messages using their context:

```rust
// In actor's handle_message method
async fn handle_message<B: MessageBroker<Self::Message>>(
    &mut self,
    message: Self::Message,
    context: &mut ActorContext<Self::Message, B>,
) -> Result<(), Self::Error> {
    // Send message to another actor
    context.send(message, recipient_address).await?;
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

### Mailbox Traits

Generic mailbox interface:

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

## Error Handling and Supervision

### ErrorAction

Actors return `ErrorAction` from `on_error` to control fault handling:

```rust
pub enum ErrorAction {
    Resume,    // Continue processing (ignore error)
    Restart,   // Restart the actor
    Stop,      // Stop the actor permanently
    Escalate,  // Pass error to supervisor
}
```

### Error Flow

1. Actor's `handle_message` returns `Err(Self::Error)`
2. Supervisor catches error
3. Supervisor calls actor's `on_error` method  
4. Actor returns `ErrorAction`
5. Supervisor applies restart strategy based on action

Example:

```rust
async fn on_error<B: MessageBroker<Self::Message>>(
    &mut self,
    error: Self::Error,
    context: &mut ActorContext<Self::Message, B>,
) -> ErrorAction {
    match error {
        CounterError::OverflowError => {
            eprintln!("Counter overflow, restarting");
            ErrorAction::Restart
        }
        CounterError::UnrecoverableError => {
            eprintln!("Unrecoverable error, stopping");
            ErrorAction::Stop
        }
        _ => ErrorAction::Escalate,
    }
}
```

## Actor Monitoring

### Health Checks

Actors can be monitored via the `Child` trait's health check:

```rust
async fn health_check(&self) -> ChildHealth {
    if self.is_healthy() {
        ChildHealth::Healthy
    } else {
        ChildHealth::Unhealthy("Connection lost".to_string())
    }
}
```

### Monitoring System

The monitoring system (from `src/monitoring/`) provides:

- Health status tracking
- Performance metrics  
- Message queue depth
- Processing latency
- Error rates

See `examples/monitoring_basic.rs` and `examples/monitoring_supervisor.rs` for monitoring examples.

## Working Examples

Explore these examples to understand the actor model:

| Example | Demonstrates | Command |
|---------|--------------|---------|
| `actor_basic.rs` | Core actor implementation | `cargo run --example actor_basic` |
| `actor_lifecycle.rs` | Lifecycle hooks | `cargo run --example actor_lifecycle` |
| `supervisor_basic.rs` | Supervision patterns | `cargo run --example supervisor_basic` |
| `supervisor_strategies.rs` | Restart strategies | `cargo run --example supervisor_strategies` |
| `monitoring_basic.rs` | Actor monitoring | `cargo run --example monitoring_basic` |

All examples are in the `examples/` directory and demonstrate real implementations of these patterns.