# airssys-rt System Patterns

## FINALIZED ARCHITECTURE - October 2, 2025

### Core Design Principles
1. **Zero-Cost Abstractions**: No runtime overhead, maximum compile-time optimization
2. **Type Safety**: Compile-time message type verification, no reflection
3. **Memory Efficiency**: Stack allocation, no unnecessary heap usage
4. **Developer Experience**: Simple, explicit APIs with clear error messages
5. **Modularity**: Clean separation of concerns with embedded unit tests

## Complete Module Architecture

### Core Actor System
```
┌─────────────────────────────────────────┐
│            Actor System                 │
├─────────────────────────────────────────┤
│         Supervisor Tree                 │ ← Fault tolerance
├─────────────────────────────────────────┤
│         Message Broker                  │ ← In-memory message broker
├─────────────────────────────────────────┤
│         Actor Registry                  │ ← Actor addressing
├─────────────────────────────────────────┤
│         Actor Runtime                   │ ← Actor execution
├─────────────────────────────────────────┤
│         airssys-osl Integration         │ ← OS operations
└─────────────────────────────────────────┘
```

### Complete Module Structure
```
airssys-rt/
├── src/
│   ├── lib.rs                          # Public API exports
│   ├── actor/                          # Actor System Core
│   │   ├── mod.rs                      # Public exports
│   │   ├── traits.rs                   # Actor trait + unit tests
│   │   ├── context.rs                  # ActorContext + unit tests
│   │   ├── lifecycle.rs                # Lifecycle management + unit tests
│   │   ├── spawn.rs                    # Actor spawning + unit tests
│   │   └── handle.rs                   # ActorHandle + unit tests
│   ├── message/                        # Message System
│   │   ├── mod.rs                      # Public exports
│   │   ├── traits.rs                   # Message trait + unit tests
│   │   ├── envelope.rs                 # MessageEnvelope + unit tests
│   │   ├── priority.rs                 # MessagePriority + unit tests
│   │   └── router.rs                   # Message routing + unit tests
│   ├── mailbox/                        # Mailbox System
│   │   ├── mod.rs                      # Public exports
│   │   ├── traits.rs                   # Mailbox traits + unit tests
│   │   ├── bounded.rs                  # BoundedMailbox + unit tests
│   │   ├── unbounded.rs                # UnboundedMailbox + unit tests
│   │   └── backpressure.rs             # Backpressure strategies + unit tests
│   ├── broker/                         # Message Broker System
│   │   ├── mod.rs                      # Public exports
│   │   ├── traits.rs                   # MessageBroker trait + unit tests
│   │   ├── in_memory.rs                # InMemoryMessageBroker + unit tests
│   │   ├── registry.rs                 # Actor registry + unit tests
│   │   ├── delivery.rs                 # Message delivery + unit tests
│   │   └── metrics.rs                  # Broker metrics + unit tests
│   ├── supervisor/                     # Supervision System
│   │   ├── mod.rs                      # Public exports
│   │   ├── traits.rs                   # Supervisor traits + unit tests
│   │   ├── tree.rs                     # Supervisor tree + unit tests
│   │   ├── strategy.rs                 # Restart strategies + unit tests
│   │   └── monitor.rs                  # Health monitoring + unit tests
│   ├── system/                         # Actor System Core
│   │   ├── mod.rs                      # Public exports
│   │   ├── actor_system.rs             # Main ActorSystem + unit tests
│   │   ├── config.rs                   # System configuration + unit tests
│   │   ├── builder.rs                  # ActorSpawnBuilder + unit tests
│   │   └── errors.rs                   # System error types + unit tests
│   ├── address/                        # Actor Addressing
│   │   ├── mod.rs                      # Public exports
│   │   ├── types.rs                    # ActorAddress types + unit tests
│   │   ├── resolver.rs                 # Address resolution + unit tests
│   │   └── pool.rs                     # Actor pool management + unit tests
│   ├── integration/                    # External Integrations
│   │   ├── mod.rs                      # Public exports
│   │   ├── osl.rs                      # airssys-osl integration + unit tests
│   │   └── wasm.rs                     # airssys-wasm integration + unit tests
│   └── util/                           # Utilities
│       ├── mod.rs                      # Public exports
│       ├── ids.rs                      # ID generation + unit tests
│       ├── time.rs                     # Time utilities + unit tests
│       └── metrics.rs                  # Metrics collection + unit tests
├── examples/                           # Usage Examples
├── tests/                              # Integration Tests Only
└── benches/                            # Performance Benchmarks
```

## Core Implementation Patterns

### Message System - Zero-Cost Abstractions
```rust
/// Core message trait - zero reflection, maximum performance
pub trait Message: Send + Sync + Clone + Debug + 'static {
    /// Explicit message type identifier - no reflection
    const MESSAGE_TYPE: &'static str;
    
    /// Message routing priority
    fn priority(&self) -> MessagePriority {
        MessagePriority::Normal
    }
    
    /// Message version for schema evolution
    fn version(&self) -> u32 {
        1
    }
    
    /// Optional: custom routing key for sharding
    fn routing_key(&self) -> Option<&str> {
        None
    }
}

/// Generic message envelope - no type erasure, zero-cost
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MessageEnvelope<M: Message> {
    pub id: MessageId,
    pub sender: Option<ActorId>,
    pub recipient: ActorAddress,
    pub payload: M,  // Direct generic type - no Box, no dyn
    pub timestamp: DateTime<Utc>,
    pub reply_to: Option<MessageId>,
    pub ttl: Option<Duration>,
    pub priority: MessagePriority,
}
```

### Actor System - Generic Constraints
```rust
/// Core Actor trait - generic constraints, no trait objects
#[async_trait]
pub trait Actor: Send + Sync + 'static {
    type Message: Message;
    type Error: Error + Send + Sync + 'static;
    
    /// Handle incoming message
    async fn handle_message(
        &mut self,
        message: Self::Message,
        context: &mut ActorContext<Self::Message>,
    ) -> Result<(), Self::Error>;
    
    /// Actor lifecycle methods
    async fn pre_start(&mut self, context: &mut ActorContext<Self::Message>) -> Result<(), Self::Error>;
    async fn post_stop(&mut self, context: &ActorContext<Self::Message>) -> Result<(), Self::Error>;
    async fn on_error(&mut self, error: Self::Error, context: &ActorContext<Self::Message>) -> ErrorAction;
}

/// Generic actor context - no trait objects, compile-time generic constraints
pub struct ActorContext<M: Message> {
    actor_id: ActorId,
    address: ActorAddress,
    broker: InMemoryMessageBroker<M>,  // Generic constraints instead of trait objects
    system_info: SystemInfo,
}
```

### Message Broker - Generic Based
```rust
/// Generic message broker trait - no trait objects
#[async_trait]
pub trait MessageBroker<M: Message>: Send + Sync + Clone {
    type Error: Error + Send + Sync + 'static;
    
    /// Send message with compile-time type safety
    async fn send(&self, envelope: MessageEnvelope<M>) -> Result<(), Self::Error>;
    
    /// Request-reply pattern with generic types
    async fn request<R: Message>(
        &self,
        envelope: MessageEnvelope<M>,
        timeout: Duration,
    ) -> Result<Option<MessageEnvelope<R>>, Self::Error>;
    
    /// Reply to a specific message ID
    async fn reply(&self, reply_to: MessageId, message: M) -> Result<(), Self::Error>;
}
```

### Actor Addressing System
```rust
/// Actor addressing scheme with zero-cost abstractions
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum ActorAddress {
    /// Direct actor ID - fastest routing
    Id(ActorId),
    /// Named address for service discovery
    Named(String),
    /// Service-based address with optional routing key
    Service { service: String, key: Option<String> },
    /// Pool-based address for load balancing
    Pool { pool: String, strategy: PoolStrategy },
}

/// Actor pool load balancing strategies
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum PoolStrategy {
    RoundRobin,
    LeastConnections,
    Random,
    ConsistentHash(String),
}
```

## Key Architecture Benefits

### Zero-Cost Abstractions
- **No `Box<dyn Trait>`**: All generics resolved at compile time
- **No `std::any`**: Pure compile-time type checking
- **Stack Allocation**: Messages live on stack, not heap
- **Static Dispatch**: Maximum compiler optimization

### Type Safety
- **Compile-Time Verification**: Wrong message types caught at compile time
- **Explicit Message Types**: `const MESSAGE_TYPE` with no reflection
- **Generic Constraints**: Type safety throughout the system
- **Clear Error Messages**: Rust's type system provides excellent error reporting

### Performance
- **Zero Runtime Overhead**: No dynamic dispatch or type checking
- **Memory Efficient**: No unnecessary allocations
- **CPU Optimized**: Compiler can inline and optimize aggressively
- **Cache Friendly**: Predictable memory access patterns

### Developer Experience
- **Simple APIs**: Clean, intuitive interfaces
- **Embedded Tests**: Unit tests co-located with implementation
- **Excellent IDE Support**: Full autocomplete and type checking
- **Clear Module Organization**: Logical separation of concerns

## Integration Points

### airssys-osl Integration
- **Direct Usage**: Actors use OSL components directly without abstraction layer
- **Process Management**: Use OSL for actual OS process operations
- **Security Context**: Inherit security policies from OSL layer
- **Activity Logging**: Integrate with OSL logging framework

### Cross-Service Messaging
- **Hybrid Approach**: Direct messaging for critical paths, event bus for loose coupling
- **Actor Spawning**: Builder Pattern with flexible configuration
- **Supervision**: Per-service granularity with clear boundaries

## Multi-Message Support
```rust
/// Union message type for actors handling multiple message types
#[derive(Debug, Clone)]
pub enum ActorMessage {
    User(UserMessage),
    System(SystemMessage),
    Network(NetworkMessage),
}

impl Message for ActorMessage {
    const MESSAGE_TYPE: &'static str = "ActorMessage";
}
```
}

pub struct ActorHandle<T: Actor> {
    sender: mpsc::UnboundedSender<T::Message>,
    actor_id: ActorId,
    created_at: DateTime<Utc>,
}
```

### Message Passing Pattern
```rust
use serde::{Deserialize, Serialize};
use uuid::Uuid;

pub trait Message: Send + Sync + Clone + 'static {}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Envelope<T: Message> {
    pub message_id: MessageId,
    pub sender: Option<ActorId>,
    pub recipient: ActorId,
    pub message: T,
    pub timestamp: DateTime<Utc>,
}

pub struct Mailbox<T: Message> {
    queue: VecDeque<Envelope<T>>,
    capacity: usize,
}

impl<T: Message> Mailbox<T> {
    pub fn new(capacity: usize) -> Self {
        Self {
            queue: VecDeque::with_capacity(capacity),
            capacity,
        }
    }
    
    pub fn push(&mut self, envelope: Envelope<T>) -> Result<(), MailboxError> {
        if self.queue.len() >= self.capacity {
            return Err(MailboxError::Full);
        }
        self.queue.push_back(envelope);
        Ok(())
    }
}
```

### Supervisor Pattern
```rust
use std::collections::HashMap;
use std::time::Duration;

#[derive(Debug, Clone)]
pub enum SupervisionStrategy {
    OneForOne,
    OneForAll,
    RestForOne,
}

#[derive(Debug, Clone)]
pub enum RestartStrategy {
    Permanent,
    Temporary,
    Transient,
}

pub struct SupervisorSpec {
    pub strategy: SupervisionStrategy,
    pub max_restarts: u32,
    pub max_restart_period: Duration,
    pub children: Vec<ChildSpec>,
}

pub struct Supervisor {
    spec: SupervisorSpec,
    children: HashMap<ActorId, ChildState>,
    restart_count: u32,
    restart_window_start: DateTime<Utc>,
}

impl Supervisor {
    pub async fn handle_child_failure(
        &mut self,
        failed_child: ActorId,
        error: ActorError,
    ) -> Result<(), SupervisorError> {
        match self.spec.strategy {
            SupervisionStrategy::OneForOne => {
                self.restart_child(failed_child).await?;
            }
            SupervisionStrategy::OneForAll => {
                self.restart_all_children().await?;
            }
            SupervisionStrategy::RestForOne => {
                self.restart_child_and_after(failed_child).await?;
            }
        }
        Ok(())
    }
}
```

## Integration Patterns

### airssys-osl Integration
```rust
use airssys_osl::process::ProcessManager;
use airssys_osl::security::SecurityContext;

pub struct OSLIntegratedActor {
    process_mgr: ProcessManager,
    security_context: SecurityContext,
}

#[async_trait::async_trait]
impl Actor for OSLIntegratedActor {
    type Message = OSLMessage;
    type State = OSLState;
    
    async fn handle_message(
        &mut self,
        message: Self::Message,
        context: &mut ActorContext,
    ) -> Result<(), ActorError> {
        match message {
            OSLMessage::SpawnProcess(cmd) => {
                let handle = self.process_mgr.spawn_secure(cmd).await?;
                context.reply(OSLMessage::ProcessSpawned(handle)).await?;
            }
            OSLMessage::FileOperation(op) => {
                let result = self.process_mgr.execute_file_operation(op).await?;
                context.reply(OSLMessage::FileResult(result)).await?;
            }
        }
        Ok(())
    }
}
```

### Performance Optimization Patterns
```rust
use std::sync::Arc;
use parking_lot::RwLock;

pub struct ActorPool<T: Actor> {
    actors: Vec<ActorHandle<T>>,
    round_robin_counter: Arc<RwLock<usize>>,
}

impl<T: Actor> ActorPool<T> {
    pub fn route_message(&self, message: T::Message) -> Result<(), ActorError> {
        let mut counter = self.round_robin_counter.write();
        let index = *counter % self.actors.len();
        *counter = (*counter + 1) % self.actors.len();
        
        self.actors[index].send(message)?;
        Ok(())
    }
}

// Zero-copy message passing for large data
pub struct ZeroCopyMessage {
    data: Arc<[u8]>,
    metadata: MessageMetadata,
}

impl Message for ZeroCopyMessage {}

impl Clone for ZeroCopyMessage {
    fn clone(&self) -> Self {
        Self {
            data: Arc::clone(&self.data), // Zero-copy clone
            metadata: self.metadata.clone(),
        }
    }
}
```

## Fault Tolerance Patterns

### Circuit Breaker Pattern
```rust
use std::sync::atomic::{AtomicU32, AtomicBool, Ordering};

pub struct CircuitBreaker {
    failure_threshold: u32,
    failure_count: AtomicU32,
    is_open: AtomicBool,
    last_failure_time: Arc<RwLock<DateTime<Utc>>>,
    timeout: Duration,
}

impl CircuitBreaker {
    pub async fn call<F, T, E>(&self, f: F) -> Result<T, CircuitBreakerError>
    where
        F: Future<Output = Result<T, E>>,
    {
        if self.is_open() {
            return Err(CircuitBreakerError::Open);
        }
        
        match f.await {
            Ok(result) => {
                self.record_success();
                Ok(result)
            }
            Err(_) => {
                self.record_failure();
                Err(CircuitBreakerError::Failure)
            }
        }
    }
}
```

### Health Monitoring Pattern
```rust
pub struct ActorSystemHealth {
    actor_count: AtomicUsize,
    message_throughput: Arc<RwLock<MessageThroughput>>,
    supervisor_stats: Arc<RwLock<SupervisorStats>>,
}

impl ActorSystemHealth {
    pub async fn health_check(&self) -> HealthStatus {
        let actor_count = self.actor_count.load(Ordering::Relaxed);
        let throughput = self.message_throughput.read().clone();
        let supervisor_stats = self.supervisor_stats.read().clone();
        
        HealthStatus {
            actor_count,
            throughput,
            supervisor_stats,
            timestamp: Utc::now(),
        }
    }
}