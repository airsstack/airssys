# Core Concepts

`airssys-rt` is built around several fundamental concepts adapted from the Erlang/BEAM runtime model. Understanding these concepts is essential for effectively using the actor runtime.

## Virtual Processes

### Definition
Virtual processes in `airssys-rt` are lightweight units of execution that exist entirely in memory. They are similar to:
- Erlang processes in BEAM
- Green threads in Go
- Actors in Akka/Actor frameworks

**Important**: These are **not** operating system processes. They are managed execution contexts within a single OS process.

### Characteristics
```rust
struct VirtualProcess {
    pid: ProcessId,           // Unique identifier
    state: ActorState,        // Private, encapsulated state
    mailbox: MessageQueue,    // Incoming message queue
    supervisor: Option<ProcessId>, // Parent supervisor reference
    children: HashSet<ProcessId>,  // Child process references
    status: ProcessStatus,    // Current execution status
}
```

### Process Lifecycle
1. **Spawn**: Process is created and registered in the system
2. **Running**: Process actively handles messages
3. **Waiting**: Process is idle, waiting for messages
4. **Stopping**: Process is shutting down gracefully
5. **Stopped**: Process has terminated and been cleaned up

## Actor Model

### Encapsulation Principle
Each actor maintains its own private state that cannot be directly accessed by other actors:

```rust
struct CounterActor {
    count: i64,        // Private - no external access
    name: String,      // Only modified via message handling
}

impl Actor for CounterActor {
    type Message = CounterMessage;
    
    // Only way to modify state is through message handling
    async fn handle(&mut self, msg: CounterMessage) -> ActorResult<()> {
        match msg {
            CounterMessage::Increment => {
                self.count += 1;  // State mutation happens here
                Ok(())
            }
            CounterMessage::GetCount => {
                // Can read state and respond
                Ok(())
            }
        }
    }
}
```

### Actor Identity
Every actor has a unique identity represented by a `ProcessId`:

```rust
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct ProcessId(u64);

// Actors are referenced by their PID
let actor_ref = ActorRef::new(pid);
actor_ref.send(message).await?;
```

## Message Passing System

### Message Immutability
All messages in `airssys-rt` are immutable once sent:

```rust
#[derive(Debug, Clone)]
pub enum UserMessage {
    CreateUser { name: String, email: String },
    UpdateUser { id: UserId, changes: UserUpdate },
    DeleteUser { id: UserId },
}

// Messages are cloned/moved when sent - no shared references
actor.send(UserMessage::CreateUser {
    name: "Alice".to_string(),
    email: "alice@example.com".to_string(),
}).await?;
```

### Mailbox Model
Each actor has a private mailbox (message queue):

```rust
struct Mailbox {
    queue: VecDeque<Message>,
    capacity: Option<usize>,      // Optional bounded capacity
    backpressure: BackpressureStrategy,
}

enum BackpressureStrategy {
    Block,        // Block sender when full
    Drop,         // Drop new messages when full
    DropOldest,   // Drop oldest messages when full
}
```

### Sequential Processing
Messages are processed one at a time, ensuring state consistency:

```rust
// Actor processes messages sequentially
loop {
    let message = mailbox.recv().await?;
    
    // Only one message processed at a time
    match self.handle(message).await {
        Ok(()) => continue,
        Err(e) => {
            // Error handling - may trigger supervision
            self.handle_error(e).await;
        }
    }
}
```

## Supervision Trees

### Hierarchical Structure
Supervision trees create a hierarchy of processes where supervisors monitor children:

```rust
// Parent-child relationships
Supervisor
├── WorkerActor1
├── WorkerActor2
└── ChildSupervisor
    ├── WorkerActor3
    └── WorkerActor4
```

### Supervision Strategies
Different strategies for handling child failures:

```rust
pub enum RestartStrategy {
    OneForOne,    // Restart only the failed child
    OneForAll,    // Restart all children when one fails
    RestForOne,   // Restart failed child and those started after it
}

pub enum RestartPolicy {
    Permanent,    // Always restart on failure
    Temporary,    // Never restart
    Transient,    // Restart only on abnormal termination
}
```

### Fault Isolation
Failures are contained within the supervision hierarchy:

```rust
// Child failure doesn't affect siblings (OneForOne strategy)
Actor1 [RUNNING] ─┐
                  ├── Supervisor [MONITORING]
Actor2 [FAILED]  ─┤  └── Restart Actor2
                  │
Actor3 [RUNNING] ─┘
```

## Process Communication Patterns

### Request-Response
Synchronous-style communication using async/await:

```rust
// Send request and wait for response
let response = actor_ref
    .ask(CalculateRequest { x: 10, y: 20 })
    .await?;

match response {
    CalculateResponse::Result(sum) => println!("Sum: {}", sum),
    CalculateResponse::Error(e) => eprintln!("Error: {}", e),
}
```

### Fire-and-Forget
Asynchronous messaging without waiting for response:

```rust
// Send message without waiting
actor_ref.tell(LogMessage {
    level: LogLevel::Info,
    message: "System started".to_string(),
}).await?;
```

### Publish-Subscribe
Event broadcasting to multiple subscribers:

```rust
// Publisher sends events
event_bus.publish(UserCreated {
    user_id: user.id,
    timestamp: Utc::now(),
}).await?;

// Multiple actors can subscribe
analytics_actor.subscribe::<UserCreated>().await?;
notification_actor.subscribe::<UserCreated>().await?;
```

## System Architecture Layers

### Runtime Layer
Core runtime managing process lifecycle:
- Process registry and addressing
- Message routing and delivery
- Scheduler integration with Tokio
- Resource management and cleanup

### Actor Layer
High-level actor abstractions:
- Actor trait definitions
- Message handling patterns
- Lifecycle hooks and callbacks
- State management utilities

### Supervision Layer
Fault tolerance and monitoring:
- Supervisor implementations
- Restart strategies and policies
- Error handling and escalation
- System health monitoring

### Integration Layer
External system integration:
- airssys-osl integration for OS operations
- Async/await ecosystem compatibility
- Metrics and monitoring integration
- Future airssys-wasm integration

## Performance Considerations

### Memory Efficiency
- Actors have minimal baseline overhead (<1KB)
- Messages are efficiently queued and processed
- Garbage collection is handled by Rust's ownership system
- No global GC pauses like in BEAM

### CPU Efficiency
- Cooperative scheduling reduces context switching
- Zero-copy message passing where possible
- Efficient actor lookup and routing
- Integration with Tokio's work-stealing scheduler

### Scalability Characteristics
- Designed for 10,000+ concurrent actors
- Sub-millisecond message latency for local delivery
- Horizontal scaling through distribution (planned)
- Efficient resource utilization under high load

These core concepts form the foundation of `airssys-rt`'s design and provide the building blocks for creating robust, concurrent applications using the actor model.