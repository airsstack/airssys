# Actors API Reference

This reference documents actor-specific types, lifecycle management, and handler patterns.

## Module: `actor`

Actor implementation and lifecycle types.

### Trait: `Handler<M>`

```rust
pub trait Handler<M>: Actor
where
    M: Message,
{
    async fn handle(&mut self, msg: M, ctx: &mut ActorContext<Self>) -> M::Result;
}
```

Trait for actors that can handle specific message types.

**Type Parameters:**

- `M`: The message type this handler processes

**Required Methods:**

- `handle()`: Processes a message and returns its result

**Trait Bounds:**

- Must implement `Actor`
- `M` must implement `Message`

**Example:**

```rust
use airssys_rt::{Actor, ActorContext, Handler, Message};

struct Counter {
    count: i32,
}

struct GetCount;
impl Message for GetCount {
    type Result = i32;
}

struct Increment {
    amount: i32,
}
impl Message for Increment {
    type Result = ();
}

impl Actor for Counter {
    async fn receive(&mut self, ctx: &mut ActorContext<Self>, msg: Box<dyn Message>) {
        // Generic receive implementation
    }
}

impl Handler<GetCount> for Counter {
    async fn handle(&mut self, _msg: GetCount, _ctx: &mut ActorContext<Self>) -> i32 {
        self.count
    }
}

impl Handler<Increment> for Counter {
    async fn handle(&mut self, msg: Increment, _ctx: &mut ActorContext<Self>) {
        self.count += msg.amount;
    }
}
```

## Lifecycle Management

### Struct: `ActorLifecycle`

```rust
pub struct ActorLifecycle {
    pub state: LifecycleState,
    pub restart_count: u32,
    pub last_error: Option<String>,
}
```

Tracks actor lifecycle state and restart history.

**Fields:**

- `state`: Current lifecycle state
- `restart_count`: Number of times actor has been restarted
- `last_error`: Most recent error message, if any

**See Also:**

- [Architecture: Process Lifecycle](../../architecture/process-lifecycle.md)

### Enum: `LifecycleState`

```rust
pub enum LifecycleState {
    Starting,
    Running,
    Stopping,
    Stopped,
    Failed,
}
```

Represents the current state of an actor.

**Variants:**

- `Starting`: Actor is initializing (in `pre_start()`)
- `Running`: Actor is actively processing messages
- `Stopping`: Actor is shutting down (in `post_stop()`)
- `Stopped`: Actor has completed shutdown
- `Failed`: Actor encountered fatal error

**State Transitions:**

```
Starting -> Running       (pre_start() succeeded)
Starting -> Failed        (pre_start() failed)
Running -> Stopping       (stop() called or system shutdown)
Running -> Failed         (unrecoverable error)
Stopping -> Stopped       (post_stop() completed)
Failed -> Starting        (supervisor restart)
```

**Example:**

```rust
use airssys_rt::actor::{ActorLifecycle, LifecycleState};

fn check_actor_state(lifecycle: &ActorLifecycle) {
    match lifecycle.state {
        LifecycleState::Running => println!("Actor is healthy"),
        LifecycleState::Failed => {
            println!("Actor failed: {:?}", lifecycle.last_error);
            println!("Restart count: {}", lifecycle.restart_count);
        }
        _ => println!("Actor in transition: {:?}", lifecycle.state),
    }
}
```

### Enum: `ErrorAction`

```rust
pub enum ErrorAction {
    Resume,
    Restart,
    Stop,
    Escalate,
}
```

Action to take when an actor encounters an error.

**Variants:**

- `Resume`: Continue processing messages (error was handled)
- `Restart`: Restart the actor (preserves supervision)
- `Stop`: Stop the actor permanently
- `Escalate`: Escalate error to supervisor

**Decision Guide:**

| Error Type | Recommended Action | Rationale |
|-----------|-------------------|-----------|
| Transient failure | `Resume` | Error handled, can continue |
| Corrupted state | `Restart` | Fresh start needed |
| Invalid configuration | `Stop` | Cannot proceed |
| Unknown/critical | `Escalate` | Let supervisor decide |

**Example:**

```rust
use airssys_rt::actor::ErrorAction;

async fn handle_error(error: &ActorError) -> ErrorAction {
    match error {
        ActorError::Timeout => ErrorAction::Resume,
        ActorError::InvalidState => ErrorAction::Restart,
        ActorError::ConfigError(_) => ErrorAction::Stop,
        _ => ErrorAction::Escalate,
    }
}
```

## Actor Configuration

### Struct: `ActorConfig`

```rust
pub struct ActorConfig {
    pub mailbox_capacity: usize,
    pub max_restarts: u32,
    pub restart_window: Duration,
}
```

Configuration options for actor behavior.

**Fields:**

- `mailbox_capacity`: Maximum number of messages in mailbox (0 = unbounded)
- `max_restarts`: Maximum restart attempts within restart window
- `restart_window`: Time window for counting restarts

**Default Values:**

```rust
impl Default for ActorConfig {
    fn default() -> Self {
        Self {
            mailbox_capacity: 1000,
            max_restarts: 3,
            restart_window: Duration::from_secs(10),
        }
    }
}
```

**Example:**

```rust
use airssys_rt::actor::ActorConfig;
use std::time::Duration;

let config = ActorConfig {
    mailbox_capacity: 5000,  // Larger buffer for high-throughput
    max_restarts: 5,         // More lenient restart policy
    restart_window: Duration::from_secs(30),
};
```

## Actor Patterns

### Builder Pattern (RT-TASK-013)

The recommended pattern for creating actors with configuration.

**Example:**

```rust
use airssys_rt::{Actor, ActorContext};

struct DatabaseActor {
    connection_string: String,
    pool_size: usize,
    timeout: Duration,
}

impl DatabaseActor {
    pub fn builder() -> DatabaseActorBuilder {
        DatabaseActorBuilder::default()
    }
}

pub struct DatabaseActorBuilder {
    connection_string: Option<String>,
    pool_size: usize,
    timeout: Duration,
}

impl Default for DatabaseActorBuilder {
    fn default() -> Self {
        Self {
            connection_string: None,
            pool_size: 10,
            timeout: Duration::from_secs(5),
        }
    }
}

impl DatabaseActorBuilder {
    pub fn connection_string(mut self, conn: String) -> Self {
        self.connection_string = Some(conn);
        self
    }
    
    pub fn pool_size(mut self, size: usize) -> Self {
        self.pool_size = size;
        self
    }
    
    pub fn timeout(mut self, timeout: Duration) -> Self {
        self.timeout = timeout;
        self
    }
    
    pub fn build(self) -> Result<DatabaseActor, BuildError> {
        let connection_string = self.connection_string
            .ok_or(BuildError::MissingField("connection_string"))?;
        
        Ok(DatabaseActor {
            connection_string,
            pool_size: self.pool_size,
            timeout: self.timeout,
        })
    }
}

// Usage
let actor = DatabaseActor::builder()
    .connection_string("postgres://localhost".to_string())
    .pool_size(20)
    .timeout(Duration::from_secs(10))
    .build()?;
```

### Request-Reply Pattern

Pattern for synchronous-style communication between actors.

**Example:**

```rust
use airssys_rt::{Actor, ActorContext, ActorRef, Handler, Message};

// Request message with reply channel
struct ComputeSum {
    numbers: Vec<i32>,
}

impl Message for ComputeSum {
    type Result = i32;
}

struct Calculator;

impl Actor for Calculator {
    async fn receive(&mut self, ctx: &mut ActorContext<Self>, msg: Box<dyn Message>) {
        // Handle messages
    }
}

impl Handler<ComputeSum> for Calculator {
    async fn handle(&mut self, msg: ComputeSum, _ctx: &mut ActorContext<Self>) -> i32 {
        msg.numbers.iter().sum()
    }
}

// Usage
async fn compute(calculator: &ActorRef<Calculator>) -> i32 {
    calculator.send(ComputeSum {
        numbers: vec![1, 2, 3, 4, 5],
    }).await.unwrap()
}
```

### Actor Pool Pattern

Pattern for load balancing across multiple worker actors.

**Example:**

```rust
use airssys_rt::{Actor, ActorContext, ActorRef};
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::Arc;

struct WorkerPool<W: Actor> {
    workers: Vec<ActorRef<W>>,
    next: Arc<AtomicUsize>,
}

impl<W: Actor> WorkerPool<W> {
    pub fn new(workers: Vec<ActorRef<W>>) -> Self {
        Self {
            workers,
            next: Arc::new(AtomicUsize::new(0)),
        }
    }
    
    pub fn get_worker(&self) -> &ActorRef<W> {
        let index = self.next.fetch_add(1, Ordering::Relaxed) % self.workers.len();
        &self.workers[index]
    }
}

// Usage
async fn create_pool(system: &ActorSystem) -> WorkerPool<Worker> {
    let mut workers = Vec::new();
    for _ in 0..4 {
        workers.push(system.spawn(Worker::new()).await.unwrap());
    }
    WorkerPool::new(workers)
}
```

## Performance Characteristics

### Actor Operations

| Operation | Latency | Throughput | Notes |
|-----------|---------|------------|-------|
| Actor spawn | ~624ns | 1.6M/sec | Includes mailbox setup |
| Message send | ~181ns | 5.5M/sec | Mailbox enqueue only |
| Message roundtrip | ~737ns | 4.7M/sec | Send + process + reply |
| Actor stop | ~2-5µs | - | Includes cleanup |

### Memory Usage

| Component | Base Size | Per-Message | Notes |
|-----------|-----------|-------------|-------|
| Actor struct | Varies | - | User-defined state |
| ActorRef | 16 bytes | - | Arc + ID |
| ActorContext | ~256 bytes | - | Per actor instance |
| Mailbox (bounded) | 8KB | 64 bytes | Capacity 1000 |
| Mailbox (unbounded) | 128 bytes | 64 bytes | Grows dynamically |

**See Also:**

- [Performance Reference](../performance.md) - Detailed benchmarks
- [BENCHMARKING.md](../../../BENCHMARKING.md) - Raw benchmark data

## Error Handling

### Enum: `ActorError`

```rust
pub enum ActorError {
    Timeout,
    InvalidState,
    ConfigError(String),
    MessageError(String),
    Custom(Box<dyn Error + Send + Sync>),
}
```

Errors specific to actor operations.

**Variants:**

- `Timeout`: Operation timed out
- `InvalidState`: Actor in invalid state for operation
- `ConfigError(String)`: Configuration error with description
- `MessageError(String)`: Message processing error
- `Custom(Box<dyn Error>)`: User-defined error

**Example:**

```rust
use airssys_rt::actor::ActorError;

impl Actor for MyActor {
    async fn receive(&mut self, ctx: &mut ActorContext<Self>, msg: Box<dyn Message>) {
        // Handle message
        if invalid_state {
            return Err(ActorError::InvalidState);
        }
    }
}
```

## Testing Utilities

### Struct: `TestProbe<A>`

```rust
pub struct TestProbe<A: Actor> {
    // fields omitted
}
```

Testing utility for observing actor behavior.

**Available in:** Test builds only (`#[cfg(test)]`)

#### Methods

##### `expect_msg()`

```rust
pub async fn expect_msg<M>(&mut self, timeout: Duration) -> Option<M>
where
    M: Message,
```

Waits for a specific message type.

**Example:**

```rust
#[cfg(test)]
mod tests {
    use super::*;
    use airssys_rt::actor::TestProbe;
    
    #[tokio::test]
    async fn test_actor_sends_response() {
        let mut probe = TestProbe::<MyActor>::new();
        // ... send message to actor ...
        let response = probe.expect_msg::<ResponseMsg>(Duration::from_secs(1))
            .await
            .expect("Expected response");
    }
}
```

## See Also

- [Core API Reference](core.md) - Core types and system API
- [Messaging API Reference](messaging.md) - Message broker and delivery
- [Supervisors API Reference](supervisors.md) - Supervision patterns
- [How-To: Actor Development](../../guides/actor-development.md) - Actor development guide
- [Architecture: Actor Model](../../architecture/actor-model.md) - Conceptual overview
