# Core API Reference

This reference documents the core types, traits, and utilities that form the foundation of the AirsSys Runtime system.

## Module: `airssys_rt`

The root module provides the primary API surface.

### Re-exports

```rust
pub use actor::{Actor, ActorContext, ActorRef, ActorSystem};
pub use message::{Message, MessageBroker};
pub use supervisor::{Child, ChildSpec, RestartStrategy, Supervisor};
```

## Module: `message`

Core message passing abstractions.

### Trait: `Message`

```rust
pub trait Message: Send + 'static {
    type Result: Send + 'static;
}
```

Marker trait for messages that can be sent between actors.

**Type Parameters:**

- `Result`: The type returned when processing this message

**Trait Bounds:**

- `Send`: Messages must be sendable across thread boundaries
- `'static`: Messages must not contain non-static references

**Example:**

```rust
use airssys_rt::Message;

struct GetStatus;

impl Message for GetStatus {
    type Result = String;
}

struct Increment {
    amount: i32,
}

impl Message for Increment {
    type Result = ();
}
```

## Module: `actor`

Actor system types and traits.

### Struct: `ActorSystem`

```rust
pub struct ActorSystem {
    // fields omitted
}
```

The actor system runtime that manages actor lifecycle and execution.

#### Methods

##### `new()`

```rust
pub fn new(name: &str) -> Result<Self, SystemError>
```

Creates a new actor system with the given name.

**Parameters:**

- `name`: A unique identifier for this actor system

**Returns:**

- `Ok(ActorSystem)`: Successfully created actor system
- `Err(SystemError)`: System initialization failed

**Example:**

```rust
use airssys_rt::ActorSystem;

let system = ActorSystem::new("my-system")
    .expect("Failed to create actor system");
```

##### `spawn()`

```rust
pub async fn spawn<A>(&self, actor: A) -> Result<ActorRef<A>, SpawnError>
where
    A: Actor,
```

Spawns a new actor in the system.

**Type Parameters:**

- `A`: The actor type implementing the `Actor` trait

**Parameters:**

- `actor`: The actor instance to spawn

**Returns:**

- `Ok(ActorRef<A>)`: Reference to the spawned actor
- `Err(SpawnError)`: Actor spawn failed

**Performance:**

- Average spawn time: ~624ns (see [Performance Reference](../performance.md))
- Memory per actor: ~512 bytes base + mailbox

**Example:**

```rust
use airssys_rt::{Actor, ActorContext, ActorSystem, Message};

struct MyActor {
    count: i32,
}

impl Actor for MyActor {
    async fn receive(&mut self, ctx: &mut ActorContext<Self>, msg: Box<dyn Message>) {
        // Handle messages
    }
}

#[tokio::main]
async fn main() {
    let system = ActorSystem::new("example").unwrap();
    let actor_ref = system.spawn(MyActor { count: 0 }).await.unwrap();
}
```

##### `shutdown()`

```rust
pub async fn shutdown(self) -> Result<(), SystemError>
```

Gracefully shuts down the actor system.

**Returns:**

- `Ok(())`: System shut down successfully
- `Err(SystemError)`: Shutdown encountered errors

**Behavior:**

- Stops all running actors in reverse spawn order
- Waits for actors to complete shutdown hooks
- Releases all system resources

**Example:**

```rust
let system = ActorSystem::new("example").unwrap();
// ... use system ...
system.shutdown().await.unwrap();
```

### Trait: `Actor`

```rust
pub trait Actor: Send + 'static {
    async fn receive(&mut self, ctx: &mut ActorContext<Self>, msg: Box<dyn Message>);
    
    async fn pre_start(&mut self, ctx: &mut ActorContext<Self>) {
        // Default: no-op
    }
    
    async fn post_stop(&mut self, ctx: &mut ActorContext<Self>) {
        // Default: no-op
    }
}
```

Core trait that all actors must implement.

**Required Methods:**

- `receive()`: Handles incoming messages

**Provided Methods:**

- `pre_start()`: Called before actor begins receiving messages (default: no-op)
- `post_stop()`: Called after actor stops (default: no-op)

**Trait Bounds:**

- `Send`: Actors must be sendable across threads
- `'static`: Actors must not contain non-static references

**Example:**

```rust
use airssys_rt::{Actor, ActorContext, Message};

struct Counter {
    count: i32,
}

impl Actor for Counter {
    async fn receive(&mut self, ctx: &mut ActorContext<Self>, msg: Box<dyn Message>) {
        // Message handling logic
    }
    
    async fn pre_start(&mut self, ctx: &mut ActorContext<Self>) {
        println!("Counter actor starting with count: {}", self.count);
    }
    
    async fn post_stop(&mut self, ctx: &mut ActorContext<Self>) {
        println!("Counter actor stopped at count: {}", self.count);
    }
}
```

### Struct: `ActorContext<A>`

```rust
pub struct ActorContext<A: Actor> {
    // fields omitted
}
```

Context provided to actors for interaction with the actor system.

**Type Parameters:**

- `A`: The actor type this context belongs to

#### Methods

##### `actor_ref()`

```rust
pub fn actor_ref(&self) -> &ActorRef<A>
```

Returns a reference to this actor's `ActorRef`.

**Returns:**

- `&ActorRef<A>`: Reference to this actor

**Use Cases:**

- Passing self-reference to spawned children
- Registering with services
- Setting up request-reply patterns

**Example:**

```rust
impl Actor for MyActor {
    async fn receive(&mut self, ctx: &mut ActorContext<Self>, msg: Box<dyn Message>) {
        let self_ref = ctx.actor_ref().clone();
        // Use self_ref for communication
    }
}
```

##### `stop()`

```rust
pub fn stop(&mut self)
```

Stops this actor.

**Behavior:**

- Actor will finish processing current message
- No new messages will be processed
- `post_stop()` hook will be called
- Actor will be removed from system

**Example:**

```rust
impl Actor for MyActor {
    async fn receive(&mut self, ctx: &mut ActorContext<Self>, msg: Box<dyn Message>) {
        if should_stop {
            ctx.stop();
        }
    }
}
```

##### `spawn_child()`

```rust
pub async fn spawn_child<C>(&mut self, child: C) -> Result<ActorRef<C>, SpawnError>
where
    C: Actor,
```

Spawns a child actor supervised by this actor.

**Type Parameters:**

- `C`: The child actor type

**Parameters:**

- `child`: The child actor instance

**Returns:**

- `Ok(ActorRef<C>)`: Reference to spawned child
- `Err(SpawnError)`: Child spawn failed

**Example:**

```rust
impl Actor for ParentActor {
    async fn pre_start(&mut self, ctx: &mut ActorContext<Self>) {
        let child = ctx.spawn_child(ChildActor::new())
            .await
            .expect("Failed to spawn child");
    }
}
```

### Struct: `ActorRef<A>`

```rust
pub struct ActorRef<A: Actor> {
    // fields omitted
}
```

Reference to an actor that can be used to send messages.

**Type Parameters:**

- `A`: The actor type this reference points to

**Traits Implemented:**

- `Clone`: Cheap to clone (uses `Arc` internally)
- `Send`: Can be sent across threads
- `Sync`: Can be shared across threads

#### Methods

##### `send()`

```rust
pub async fn send<M>(&self, msg: M) -> Result<M::Result, SendError>
where
    M: Message,
    A: Handler<M>,
```

Sends a message to the actor and waits for the result.

**Type Parameters:**

- `M`: The message type

**Parameters:**

- `msg`: The message to send

**Returns:**

- `Ok(M::Result)`: The result from processing the message
- `Err(SendError)`: Message delivery or processing failed

**Performance:**

- Average roundtrip: ~737ns (send + process + respond)
- Throughput: ~4.7M messages/second

**Example:**

```rust
use airssys_rt::{ActorRef, Message};

struct GetCount;
impl Message for GetCount {
    type Result = i32;
}

async fn example(actor: &ActorRef<Counter>) {
    let count = actor.send(GetCount)
        .await
        .expect("Failed to get count");
    println!("Count: {}", count);
}
```

##### `tell()`

```rust
pub fn tell<M>(&self, msg: M) -> Result<(), SendError>
where
    M: Message<Result = ()>,
    A: Handler<M>,
```

Sends a fire-and-forget message (no response expected).

**Type Parameters:**

- `M`: The message type (must have `Result = ()`)

**Parameters:**

- `msg`: The message to send

**Returns:**

- `Ok(())`: Message queued successfully
- `Err(SendError)`: Message delivery failed

**Performance:**

- Non-blocking send operation
- Lower latency than `send()` (no wait for response)

**Example:**

```rust
struct Increment { amount: i32 }
impl Message for Increment {
    type Result = ();
}

actor_ref.tell(Increment { amount: 5 })
    .expect("Failed to send increment");
```

## Module: `util`

Utility types and functions.

### Struct: `ActorId`

```rust
pub struct ActorId(/* private fields */);
```

Unique identifier for an actor.

**Traits Implemented:**

- `Copy`, `Clone`: Lightweight value type
- `Eq`, `PartialEq`: Equality comparison
- `Hash`: Can be used in hash maps
- `Display`: Human-readable format

#### Methods

##### `new()`

```rust
pub fn new() -> Self
```

Creates a new unique actor ID.

**Returns:**

- `ActorId`: Globally unique identifier

**Thread Safety:**

- Uses atomic counter for uniqueness
- Safe to call from multiple threads concurrently

**Example:**

```rust
use airssys_rt::util::ActorId;

let id = ActorId::new();
println!("Actor ID: {}", id);
```

## Error Types

### Enum: `SystemError`

```rust
pub enum SystemError {
    AlreadyRunning,
    InitializationFailed(String),
    ShutdownFailed(String),
}
```

Errors that can occur during system operations.

**Variants:**

- `AlreadyRunning`: Attempted to start a system that's already running
- `InitializationFailed(String)`: System initialization failed with reason
- `ShutdownFailed(String)`: System shutdown encountered errors

**Traits Implemented:**

- `Error`, `Display`, `Debug`: Standard error traits
- `Send`, `Sync`: Thread-safe error type

### Enum: `SpawnError`

```rust
pub enum SpawnError {
    SystemNotRunning,
    ActorInitFailed(String),
    ResourceExhausted,
}
```

Errors that can occur when spawning actors.

**Variants:**

- `SystemNotRunning`: System is not active
- `ActorInitFailed(String)`: Actor initialization failed
- `ResourceExhausted`: Insufficient resources to spawn actor

### Enum: `SendError`

```rust
pub enum SendError {
    ActorNotFound,
    ActorStopped,
    MailboxFull,
    Timeout,
}
```

Errors that can occur when sending messages.

**Variants:**

- `ActorNotFound`: Target actor doesn't exist
- `ActorStopped`: Target actor has stopped
- `MailboxFull`: Actor's mailbox is at capacity
- `Timeout`: Message send/receive timed out

## Type Aliases

### `ActorResult<T>`

```rust
pub type ActorResult<T> = Result<T, ActorError>;
```

Convenience alias for actor operation results.

### `SystemResult<T>`

```rust
pub type SystemResult<T> = Result<T, SystemError>;
```

Convenience alias for system operation results.

## See Also

- [Actors API Reference](actors.md) - Actor-specific types and patterns
- [Messaging API Reference](messaging.md) - Message broker and delivery
- [Supervisors API Reference](supervisors.md) - Supervision and fault tolerance
- [Architecture: Actor Model](../../architecture/actor-model.md) - Conceptual overview
