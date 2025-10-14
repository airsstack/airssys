# Core Concepts

`airssys-rt` is built around several fundamental concepts adapted from the Erlang/BEAM runtime model. Understanding these concepts is essential for effectively using the actor runtime.

> **Note**: All code examples in this document are taken from the actual implementation. For complete working examples, see the [examples directory](../../examples/).

## Actors and Message Processing

### Actor Trait

The core `Actor` trait is the foundation of the runtime system. Every actor must implement this trait with associated types for messages and errors:

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

    // Optional lifecycle hooks
    async fn pre_start<B: MessageBroker<Self::Message>>(
        &mut self,
        context: &mut ActorContext<Self::Message, B>,
    ) -> Result<(), Self::Error> {
        Ok(())
    }

    async fn post_stop<B: MessageBroker<Self::Message>>(
        &mut self,
        context: &mut ActorContext<Self::Message, B>,
    ) -> Result<(), Self::Error> {
        Ok(())
    }

    async fn on_error<B: MessageBroker<Self::Message>>(
        &mut self,
        error: Self::Error,
        context: &mut ActorContext<Self::Message, B>,
    ) -> ErrorAction {
        ErrorAction::Restart
    }
}
```

### Actor Context

The `ActorContext` provides metadata and messaging capabilities to actors:

```rust
pub struct ActorContext<M: Message, B: MessageBroker<M>> {
    address: ActorAddress,
    id: ActorId,
    created_at: DateTime<Utc>,
    last_message_at: Option<DateTime<Utc>>,
    message_count: u64,
    broker: B, // Dependency injection
    _marker: PhantomData<M>,
}
```

Key methods:
- `address()` - Get the actor's address
- `id()` - Get the actor's unique ID
- `message_count()` - Get total messages processed
- `record_message()` - Track message processing
- `send(message, recipient)` - Send messages to other actors

### Process Lifecycle

Actors go through several lifecycle stages managed by the `ActorLifecycle` struct:

```rust
pub enum ActorState {
    Starting,   // Actor is initializing
    Running,    // Actor is active and processing messages
    Stopping,   // Actor is shutting down
    Stopped,    // Actor has stopped successfully
    Failed,     // Actor has failed (requires supervision)
}
```

The `ActorLifecycle` struct provides state management (from `src/actor/lifecycle.rs`):

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
    pub fn is_terminal(&self) -> bool;
    pub fn is_running(&self) -> bool;
}
```

## Complete Actor Example

Here's a real actor implementation from `examples/actor_basic.rs`:

```rust
// Define a message type
#[derive(Debug, Clone, Serialize, Deserialize)]
struct CounterMessage {
    delta: i32,
}

impl Message for CounterMessage {
    const MESSAGE_TYPE: &'static str = "counter";
}

// Define the actor
struct CounterActor {
    value: i32,
    max_value: i32,
}

// Define error type
#[derive(Debug)]
struct CounterError {
    message: String,
}

impl fmt::Display for CounterError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "CounterError: {}", self.message)
    }
}

impl std::error::Error for CounterError {}

// Implement Actor trait
#[async_trait]
impl Actor for CounterActor {
    type Message = CounterMessage;
    type Error = CounterError;

    async fn handle_message<B: MessageBroker<Self::Message>>(
        &mut self,
        message: Self::Message,
        context: &mut ActorContext<Self::Message, B>,
    ) -> Result<(), Self::Error> {
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

    async fn pre_start<B: MessageBroker<Self::Message>>(
        &mut self,
        context: &mut ActorContext<Self::Message, B>,
    ) -> Result<(), Self::Error> {
        println!("[Actor {}] Starting with value: {}", 
            context.address().name().unwrap_or("anonymous"), 
            self.value);
        Ok(())
    }

    async fn post_stop<B: MessageBroker<Self::Message>>(
        &mut self,
        context: &mut ActorContext<Self::Message, B>,
    ) -> Result<(), Self::Error> {
        println!("[Actor {}] Stopping with value: {}", 
            context.address().name().unwrap_or("anonymous"), 
            self.value);
        Ok(())
    }

    async fn on_error<B: MessageBroker<Self::Message>>(
        &mut self,
        error: Self::Error,
        _context: &mut ActorContext<Self::Message, B>,
    ) -> ErrorAction {
        eprintln!("Error: {}", error);
        ErrorAction::Restart  // Supervisor will restart this actor
    }
}
```

Run this example:
```bash
cargo run --example actor_basic
```

## Message System

### Message Trait

All messages must implement the `Message` trait:

```rust
pub trait Message: Clone + Send + Sync + 'static 
    + for<'de> serde::Deserialize<'de> + serde::Serialize 
{
    const MESSAGE_TYPE: &'static str;
}
```

### Message Envelope

Messages are wrapped in envelopes for routing:

```rust
pub struct MessageEnvelope<M> {
    pub id: MessageId,
    pub message: M,
    pub timestamp: DateTime<Utc>,
    pub reply_to: Option<ActorAddress>,
}
```

### Message Broker

The `MessageBroker` trait defines the pub/sub system:

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

Current implementation: `InMemoryMessageBroker` (see `src/broker/in_memory.rs`)

## Supervision Framework

### Child Trait

Any entity can be supervised by implementing the `Child` trait:

```rust
#[async_trait]
pub trait Child: Send + Sync {
    async fn start(&mut self) -> Result<(), Box<dyn Error + Send + Sync>>;
    async fn stop(&mut self) -> Result<(), Box<dyn Error + Send + Sync>>;
    async fn health_check(&self) -> ChildHealth;
}
```

Actors automatically implement `Child` via blanket implementation.

### Supervision Strategies

Three BEAM-inspired restart strategies:

```rust
pub enum RestartStrategy {
    OneForOne,   // Restart only the failed child
    OneForAll,   // Restart all children when one fails
    RestForOne,  // Restart failed child and those started after it
}
```

### Restart Policies

Control when children should be restarted:

```rust
pub enum RestartPolicy {
    Permanent,   // Always restart on failure
    Transient,   // Restart only on abnormal termination
    Temporary,   // Never restart
}
```

### Child Specification

Configure supervised children:

```rust
pub struct ChildSpec {
    pub id: ChildId,
    pub restart_policy: RestartPolicy,
    pub shutdown_policy: ShutdownPolicy,
    pub significant: bool,  // Does failure affect supervisor?
}
```

### Complete Supervisor Example

From `examples/supervisor_basic.rs`:

```rust
use airssys_rt::supervisor::{Child, ChildHealth, ChildSpec, RestartPolicy};

// Define a worker that implements Child
struct SimpleWorker {
    id: String,
    fail_on_start: bool,
}

#[async_trait]
impl Child for SimpleWorker {
    async fn start(&mut self) -> Result<(), Box<dyn Error + Send + Sync>> {
        if self.fail_on_start {
            return Err(format!("Worker {} failed to start", self.id).into());
        }
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

// Create supervisor with OneForOne strategy
let mut supervisor = SupervisorNode::new(
    SupervisorId::new(),
    OneForOne::new(),
);

// Add children
supervisor.add_child(
    ChildSpec {
        id: ChildId::new(),
        restart_policy: RestartPolicy::Permanent,
        shutdown_policy: ShutdownPolicy::default(),
        significant: true,
    },
    Box::new(SimpleWorker {
        id: "worker-1".to_string(),
        fail_on_start: false,
    }),
).await?;
```

Run this example:
```bash
cargo run --example supervisor_basic
```

## Actor Addressing

### ActorAddress

Actors are identified by addresses:

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

Unique identifiers for actors:

```rust
pub struct ActorId(uuid::Uuid);

impl ActorId {
    pub fn new() -> Self {
        Self(uuid::Uuid::new_v4())
    }
}
```

## Error Handling

### ErrorAction

Actors return `ErrorAction` from `on_error` to control supervision:

```rust
pub enum ErrorAction {
    Resume,       // Continue processing (ignore error)
    Restart,      // Restart the actor
    Stop,         // Stop the actor permanently
    Escalate,     // Pass error to supervisor
}
```

### Actor Error Flow

1. Actor's `handle_message` returns `Err(Self::Error)`
2. Supervisor calls actor's `on_error` method
3. Actor returns `ErrorAction` to supervisor
4. Supervisor applies restart strategy based on action

## Integration Patterns

### OSL Integration

Actors can use `airssys-osl` for secure system operations. From `examples/osl_integration_example.rs`:

```rust
use airssys_osl::prelude::*;
use airssys_rt::{Actor, ActorContext};

struct FileActor {
    executor: OslExecutor,
}

#[async_trait]
impl Actor for FileActor {
    type Message = FileMessage;
    type Error = FileError;

    async fn handle_message<B: MessageBroker<Self::Message>>(
        &mut self,
        message: Self::Message,
        context: &mut ActorContext<Self::Message, B>,
    ) -> Result<(), Self::Error> {
        match message {
            FileMessage::Read { path } => {
                let result = self.executor.read_file(&path)?;
                // Process file content
                Ok(())
            }
        }
    }
}
```

Run this example:
```bash
cargo run --example osl_integration_example
```

## Architecture Layers

The runtime is organized in layers:

1. **Message Layer** (`src/message/`) - Message types and envelopes
2. **Broker Layer** (`src/broker/`) - Pub/sub message routing
3. **Actor Layer** (`src/actor/`) - Actor trait and context
4. **Mailbox Layer** (`src/mailbox/`) - Message queue management
5. **Supervisor Layer** (`src/supervisor/`) - Fault tolerance
6. **Monitoring Layer** (`src/monitoring/`) - Health checks and metrics
7. **System Layer** (`src/system/`) - Runtime coordination (planned)

Each layer builds on the previous, following Microsoft Rust Guidelines (M-SIMPLE-ABSTRACTIONS).

## Working Examples

Explore these examples to understand the runtime:

| Example | Description | Command |
|---------|-------------|---------|
| `actor_basic.rs` | Basic actor implementation | `cargo run --example actor_basic` |
| `actor_lifecycle.rs` | Lifecycle hooks | `cargo run --example actor_lifecycle` |
| `supervisor_basic.rs` | Basic supervision | `cargo run --example supervisor_basic` |
| `supervisor_strategies.rs` | Restart strategies | `cargo run --example supervisor_strategies` |
| `supervisor_automatic_health.rs` | Health monitoring | `cargo run --example supervisor_automatic_health` |
| `monitoring_basic.rs` | Actor monitoring | `cargo run --example monitoring_basic` |
| `monitoring_supervisor.rs` | Supervisor monitoring | `cargo run --example monitoring_supervisor` |
| `osl_integration_example.rs` | OSL integration | `cargo run --example osl_integration_example` |

All examples are located in the `examples/` directory and demonstrate real, working implementations of the concepts described in this document.