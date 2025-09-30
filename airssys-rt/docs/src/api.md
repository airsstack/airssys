# API Reference Overview

This section provides comprehensive documentation for the `airssys-rt` API, covering all public types, traits, and functions available for building actor-based applications.

## API Design Philosophy

The `airssys-rt` API is designed with several key principles:

- **Type Safety**: Leverage Rust's type system to prevent runtime errors
- **Ergonomic Design**: Minimize boilerplate while maintaining clarity
- **Performance**: Zero-cost abstractions and efficient implementations
- **Consistency**: Predictable patterns across all API components
- **Composability**: APIs that work well together and with the Rust ecosystem

## Core API Structure

### Trait-Based Architecture
The API is organized around key traits that define the behavior of actors and system components:

```rust
// Core actor behavior
pub trait Actor: Send + 'static {
    type Message: Send + 'static;
    async fn handle(&mut self, msg: Self::Message) -> ActorResult<()>;
}

// System integration
pub trait ActorSystem {
    async fn spawn<A: Actor>(&self, actor: A) -> Result<ActorRef<A::Message>, SpawnError>;
}

// Supervision behavior  
pub trait Supervisor {
    type Child: Actor;
    async fn handle_child_exit(&mut self, child: ActorId, reason: ExitReason) -> SupervisionAction;
}
```

### Type-Safe References
Actor communication is handled through strongly-typed references:

```rust
pub struct ActorRef<M: Send + 'static> {
    // Internal implementation hidden
}

impl<M: Send + 'static> ActorRef<M> {
    pub async fn tell(&self, message: M) -> Result<(), SendError>;
    pub async fn ask<R>(&self, message: M) -> Result<R, AskError>
    where
        M: HasResponse<Response = R>;
}
```

## API Reference Sections

The API documentation is organized into comprehensive reference sections:

### [Core Types](./api/core-types.md)
Fundamental types used throughout the system:
- **ProcessId**: Unique identifier for actors
- **ActorRef**: Type-safe actor references
- **ActorSystem**: Main system coordinator
- **Message**: Base message traits and utilities
- **Error Types**: Comprehensive error handling types

### [Actor Traits](./api/actor-traits.md)
Core traits that define actor behavior:
- **Actor**: Main actor trait for message handling
- **ActorContext**: Context provided to actors during execution
- **Lifecycle**: Actor lifecycle hooks and callbacks
- **StateManagement**: State persistence and recovery traits

### [Message Types](./api/message-types.md)
Message system API and utilities:
- **Message**: Core message trait requirements
- **Request/Response**: Synchronous-style communication patterns
- **Event**: Asynchronous event broadcasting
- **Envelope**: Internal message wrapper utilities

### [Supervisor API](./api/supervisor-api.md)
Supervision and fault tolerance API:
- **Supervisor**: Supervisor trait and implementations
- **RestartStrategy**: Configuration for restart behavior
- **SupervisionTree**: Hierarchical supervision utilities
- **HealthMonitoring**: Actor health and monitoring APIs

## Quick API Reference

### Basic Actor Implementation
```rust
use airssys_rt::{Actor, ActorResult, ActorSystem, ActorRef};

// Define your actor
struct MyActor {
    state: MyState,
}

// Define messages
#[derive(Debug)]
enum MyMessage {
    DoWork(String),
    GetStatus(oneshot::Sender<Status>),
}

// Implement actor behavior
impl Actor for MyActor {
    type Message = MyMessage;
    
    async fn handle(&mut self, msg: MyMessage) -> ActorResult<()> {
        match msg {
            MyMessage::DoWork(data) => {
                // Process work
                self.process(data).await?;
                Ok(())
            }
            MyMessage::GetStatus(sender) => {
                let status = self.get_status();
                sender.send(status).map_err(|_| ActorError::ChannelClosed)?;
                Ok(())
            }
        }
    }
}

// Use the actor
async fn example() -> Result<(), Box<dyn std::error::Error>> {
    let system = ActorSystem::new().await?;
    let actor_ref = system.spawn(MyActor::new()).await?;
    
    // Send messages
    actor_ref.tell(MyMessage::DoWork("data".to_string())).await?;
    
    let (tx, rx) = oneshot::channel();
    actor_ref.tell(MyMessage::GetStatus(tx)).await?;
    let status = rx.await?;
    
    Ok(())
}
```

### Supervision Setup
```rust
use airssys_rt::{Supervisor, RestartStrategy, RestartPolicy};

async fn setup_supervision() -> Result<(), SupervisionError> {
    let system = ActorSystem::new().await?;
    
    let supervisor = Supervisor::builder()
        .strategy(RestartStrategy::OneForOne)
        .policy(RestartPolicy::Permanent)
        .max_restarts(5)
        .within_seconds(60)
        .child("worker1", WorkerActor::new())
        .child("worker2", WorkerActor::new())
        .build(&system)
        .await?;
    
    Ok(())
}
```

## Common API Patterns

### Request-Response Pattern
```rust
// Define request with response channel
#[derive(Debug)]
struct CalculateRequest {
    x: i32,
    y: i32,
    response: oneshot::Sender<i32>,
}

// Helper trait for ergonomic ask() method
impl HasResponse for CalculateRequest {
    type Response = i32;
    
    fn with_response(self, sender: oneshot::Sender<Self::Response>) -> Self {
        CalculateRequest { 
            x: self.x, 
            y: self.y, 
            response: sender 
        }
    }
}

// Usage
let result = actor.ask(CalculateRequest { x: 10, y: 20 }).await?;
```

### Event Broadcasting
```rust
use airssys_rt::{EventBus, Event};

#[derive(Debug, Clone)]
struct UserCreated {
    user_id: UserId,
    timestamp: DateTime<Utc>,
}

impl Event for UserCreated {
    fn event_type(&self) -> &'static str {
        "user.created"
    }
}

// Publishing
let event_bus = EventBus::new();
event_bus.publish(UserCreated {
    user_id: user.id,
    timestamp: Utc::now(),
}).await?;

// Subscribing
actor_ref.subscribe::<UserCreated>().await?;
```

### Actor Pools
```rust
use airssys_rt::{ActorPool, RoundRobinStrategy};

let pool = ActorPool::new()
    .strategy(RoundRobinStrategy::new())
    .size(10)
    .factory(|| WorkerActor::new())
    .build(&system)
    .await?;

// Submit work to pool
pool.submit(WorkMessage::Process(data)).await?;
```

## Error Handling

### Comprehensive Error Types
```rust
#[derive(Debug, thiserror::Error)]
pub enum ActorError {
    #[error("Actor mailbox is full")]
    MailboxFull,
    
    #[error("Actor not found: {id}")]
    ActorNotFound { id: ProcessId },
    
    #[error("Channel closed unexpectedly")]
    ChannelClosed,
    
    #[error("Business logic error: {0}")]
    BusinessLogic(String),
    
    #[error("Actor timeout after {seconds}s")]
    Timeout { seconds: u64 },
}

pub type ActorResult<T> = Result<T, ActorError>;
```

### Error Recovery Patterns
```rust
impl Actor for ResilientActor {
    async fn handle(&mut self, msg: MyMessage) -> ActorResult<()> {
        self.process_message(msg).await
            .or_else(|e| self.handle_recoverable_error(e))
    }
    
    async fn handle_error(&mut self, error: ActorError) -> ErrorAction {
        match error {
            ActorError::Timeout { .. } => ErrorAction::Retry,
            ActorError::BusinessLogic(_) => ErrorAction::Continue,
            _ => ErrorAction::Stop,
        }
    }
}
```

## Performance and Configuration

### System Configuration
```rust
use airssys_rt::{ActorSystemConfig, SchedulerConfig};

let config = ActorSystemConfig::builder()
    .max_actors(10000)
    .message_queue_size(1000)
    .scheduler(SchedulerConfig {
        worker_threads: num_cpus::get(),
        max_blocking_threads: 512,
    })
    .build();

let system = ActorSystem::with_config(config).await?;
```

### Performance Monitoring
```rust
use airssys_rt::{ActorMetrics, SystemMetrics};

// Actor-level metrics
let metrics = actor_ref.metrics().await?;
println!("Messages processed: {}", metrics.messages_processed);
println!("Average processing time: {:?}", metrics.avg_processing_time);

// System-level metrics
let system_metrics = system.metrics().await?;
println!("Active actors: {}", system_metrics.active_actors);
println!("Total memory usage: {} bytes", system_metrics.memory_usage);
```

## Integration APIs

### AirsSys Integration
```rust
use airssys_rt::{AirsOslIntegration, SecurityContext, ActivityLogger};

let system = ActorSystem::builder()
    .integration(AirsOslIntegration::new()
        .security_context(security_ctx)
        .activity_logger(logger)
        .resource_limits(limits))
    .build()
    .await?;
```

### Custom Extensions
```rust
use airssys_rt::{ActorSystemExtension, Extension};

struct CustomExtension {
    config: CustomConfig,
}

impl Extension for CustomExtension {
    async fn initialize(&mut self, system: &ActorSystem) -> Result<(), ExtensionError>;
    async fn actor_spawned(&mut self, actor_id: ProcessId) -> Result<(), ExtensionError>;
    async fn actor_stopped(&mut self, actor_id: ProcessId) -> Result<(), ExtensionError>;
}
```

The API reference provides complete documentation for all public interfaces in `airssys-rt`, enabling you to build robust actor-based applications with full type safety and performance optimization.