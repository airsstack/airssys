# Component Architecture

Detailed subsystem documentation for each layer of the `airssys-rt` runtime, including implementation details, interfaces, and integration patterns.

> **Note**: For high-level architecture, see [System Overview](./system-overview.md). This document provides implementation-level details.

## Table of Contents

1. [Message Layer](#message-layer)
2. [Broker Layer](#broker-layer)
3. [Actor Layer](#actor-layer)
4. [Mailbox Layer](#mailbox-layer)
5. [Supervisor Layer](#supervisor-layer)
6. [Monitoring Layer](#monitoring-layer)
7. [System Layer](#system-layer-planned)

---

## Message Layer

### Overview

The foundation layer providing type-safe message contracts and metadata.

**Location:** `src/message/`

**Responsibilities:**

- Define message trait contract
- Provide message envelope wrapper
- Generate unique message identifiers
- Timestamp message creation

### Components

#### Message Trait

```rust
// src/message/mod.rs
pub trait Message: Clone + Send + Sync + 'static 
    + for<'de> serde::Deserialize<'de> + serde::Serialize 
{
    const MESSAGE_TYPE: &'static str;
}
```

**Design Rationale:**

- `Clone`: Messages broadcast to multiple subscribers need cloning
- `Send + Sync + 'static`: Cross-thread messaging requires thread safety
- `Serialize + Deserialize`: Future network/persistence support
- `MESSAGE_TYPE`: Runtime message type identification

**Implementation Example:**

```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CounterMessage {
    pub delta: i32,
}

impl Message for CounterMessage {
    const MESSAGE_TYPE: &'static str = "counter";
}
```

#### MessageId

```rust
// src/message/mod.rs
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct MessageId(uuid::Uuid);

impl MessageId {
    pub fn new() -> Self {
        Self(uuid::Uuid::new_v4())
    }
}
```

**Characteristics:**

- UUID v4 for global uniqueness
- 128-bit identifier
- Copy-able (16 bytes on stack)
- Hash-able for collections

#### MessageEnvelope

```rust
// src/message/envelope.rs
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

    pub fn with_reply_to(message: M, reply_to: ActorAddress) -> Self {
        Self {
            id: MessageId::new(),
            message,
            timestamp: Utc::now(),
            reply_to: Some(reply_to),
        }
    }
}
```

**Features:**

- Automatic ID generation
- Timestamp at creation (UTC)
- Optional reply address for request/reply pattern
- Generic over message type

### Performance

| Operation | Latency | Notes |
|-----------|---------|-------|
| MessageId creation | ~5 ns | UUID v4 generation |
| Envelope wrapping | ~10 ns | ID + timestamp + allocation |
| Message clone | Varies | Depends on message size |

**Memory:**

- `MessageId`: 16 bytes (UUID)
- `MessageEnvelope<M>`: 16 + sizeof(M) + 16 + 24 = 56 + sizeof(M) bytes

---

## Broker Layer

### Overview

Pub/sub message routing system connecting actors through publish/subscribe semantics.

**Location:** `src/broker/`

**Responsibilities:**

- Route messages from publishers to subscribers
- Manage subscriber registration
- Handle message broadcast
- Provide broker abstraction trait

### Components

#### MessageBroker Trait

```rust
// src/broker/traits.rs
#[async_trait]
pub trait MessageBroker<M: Message>: Clone + Send + Sync + 'static {
    type Error: Error + Send + Sync + 'static;

    async fn publish(&self, envelope: MessageEnvelope<M>) 
        -> Result<(), Self::Error>;
    
    async fn subscribe(&self, subscriber_id: ActorId) 
        -> Result<mpsc::Receiver<MessageEnvelope<M>>, Self::Error>;
}
```

**Design Rationale:**

- `Clone`: Brokers shared across actors via cheap Arc cloning
- Generic `<M: Message>`: Type-safe message routing per message type
- `async`: Non-blocking pub/sub operations
- Associated `Error`: Broker-specific error handling

#### InMemoryMessageBroker

```rust
// src/broker/in_memory.rs
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

#[async_trait]
impl<M: Message> MessageBroker<M> for InMemoryMessageBroker<M> {
    type Error = BrokerError;

    async fn publish(&self, envelope: MessageEnvelope<M>) 
        -> Result<(), Self::Error> 
    {
        let subscribers = self.subscribers.lock().unwrap();
        
        for (_, sender) in subscribers.iter() {
            // Clone envelope for each subscriber
            let _ = sender.send(envelope.clone()).await;
        }
        
        Ok(())
    }

    async fn subscribe(&self, subscriber_id: ActorId) 
        -> Result<mpsc::Receiver<MessageEnvelope<M>>, Self::Error> 
    {
        let (sender, receiver) = mpsc::channel(100);  // Buffered channel
        
        self.subscribers.lock().unwrap()
            .insert(subscriber_id, sender);
        
        Ok(receiver)
    }
}
```

**Implementation Details:**

- `Arc<Mutex<HashMap>>`: Thread-safe subscriber map, cheap cloning
- Tokio `mpsc::channel`: Async message channels
- Buffer size: 100 messages per subscriber
- Broadcast clones envelope to all subscribers

#### BrokerError

```rust
// src/broker/in_memory.rs
#[derive(Debug)]
pub enum BrokerError {
    SubscriberNotFound(ActorId),
    ChannelClosed,
    SendError(String),
}
```

### Performance

From `benches/message_benchmarks.rs`:

| Operation | Latency | Throughput | Notes |
|-----------|---------|------------|-------|
| publish + receive | 737 ns | 1.36M msgs/sec | Full roundtrip |
| Sustained throughput | 211 ns/msg | 4.7M msgs/sec | 100 messages |
| Broadcast (10 actors) | 395 ns | ~40 ns/actor | Efficient multi-cast |

**Overhead Analysis:**

- Direct actor processing: 31.55 ns/msg
- Via broker: 211.88 ns/msg
- **Broker overhead: 6.7x** - acceptable for pub/sub semantics

**Bottlenecks:**

- `Mutex<HashMap>` contention with many concurrent publishers
- Message cloning for broadcast (scales with subscriber count)

**Memory:**

- Base broker: ~48 bytes (Arc + Mutex)
- Per subscriber: ~32 bytes (ActorId) + channel overhead

### Future Enhancements

**Planned (not yet implemented):**

- Sharded broker (reduce contention)
- Network broker (distributed actors)
- Persistent broker (message durability)
- Topic-based routing (message filtering)

---

## Actor Layer

### Overview

Business logic execution layer providing actor trait, context, and lifecycle management.

**Location:** `src/actor/`

**Responsibilities:**

- Define actor behavior contract
- Provide actor execution context
- Manage actor lifecycle state
- Handle message processing

### Components

#### Actor Trait

```rust
// src/actor/traits.rs
#[async_trait]
pub trait Actor: Send + Sync + 'static {
    type Message: Message;
    type Error: Error + Send + Sync + 'static;

    async fn handle_message<B: MessageBroker<Self::Message>>(
        &mut self,
        message: Self::Message,
        context: &mut ActorContext<Self::Message, B>,
    ) -> Result<(), Self::Error>;

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

**Design Rationale:**

- Generic constraint `<B: MessageBroker>`: Dependency injection, testability
- Associated types: Type safety without parameter explosion
- Lifecycle hooks: Initialization and cleanup integration points
- Error handling: Supervision decision via `ErrorAction`

#### ActorContext

```rust
// src/actor/context.rs
pub struct ActorContext<M: Message, B: MessageBroker<M>> {
    address: ActorAddress,
    id: ActorId,
    created_at: DateTime<Utc>,
    last_message_at: Option<DateTime<Utc>>,
    message_count: u64,
    broker: B,
    _marker: PhantomData<M>,
}

impl<M: Message, B: MessageBroker<M>> ActorContext<M, B> {
    pub fn new(address: ActorAddress, broker: B) -> Self {
        Self {
            id: address.id().clone(),
            address,
            created_at: Utc::now(),
            last_message_at: None,
            message_count: 0,
            broker,
            _marker: PhantomData,
        }
    }

    pub fn address(&self) -> &ActorAddress { &self.address }
    pub fn id(&self) -> &ActorId { &self.id }
    pub fn message_count(&self) -> u64 { self.message_count }
    pub fn created_at(&self) -> DateTime<Utc> { self.created_at }

    pub fn record_message(&mut self) {
        self.message_count += 1;
        self.last_message_at = Some(Utc::now());
    }

    pub async fn send(
        &self,
        message: M,
        _recipient: ActorAddress,
    ) -> Result<(), B::Error> {
        let envelope = MessageEnvelope::new(message);
        self.broker.publish(envelope).await
    }
}
```

**Features:**

- Actor metadata (address, ID, timestamps)
- Message statistics tracking
- Broker access for messaging
- Type-safe message sending

#### ActorLifecycle

```rust
// src/actor/lifecycle.rs
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ActorState {
    Starting,
    Running,
    Stopping,
    Stopped,
    Failed,
}

#[derive(Debug, Clone)]
pub struct ActorLifecycle {
    state: ActorState,
    last_state_change: DateTime<Utc>,
    restart_count: u32,
}

impl ActorLifecycle {
    pub fn new() -> Self {
        Self {
            state: ActorState::Starting,
            last_state_change: Utc::now(),
            restart_count: 0,
        }
    }

    pub fn state(&self) -> ActorState { self.state }
    
    pub fn transition_to(&mut self, new_state: ActorState) {
        self.state = new_state;
        self.last_state_change = Utc::now();
    }

    pub fn restart_count(&self) -> u32 { self.restart_count }
    
    pub fn record_restart(&mut self) {
        self.restart_count += 1;
    }

    pub fn is_terminal(&self) -> bool {
        matches!(self.state, ActorState::Stopped | ActorState::Failed)
    }

    pub fn is_running(&self) -> bool {
        self.state == ActorState::Running
    }
}
```

**State Machine:**

- Starting → Running (successful init)
- Starting → Failed (init error)
- Running → Stopping (graceful shutdown)
- Running → Failed (runtime error)
- Stopping → Stopped (cleanup complete)
- Stopped → Starting (supervisor restart)

#### ErrorAction

```rust
// src/actor/traits.rs
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ErrorAction {
    Resume,    // Continue processing (ignore error)
    Restart,   // Restart the actor
    Stop,      // Stop the actor permanently
    Escalate,  // Pass error to supervisor
}
```

### Performance

From `benches/actor_benchmarks.rs`:

| Operation | Latency | Notes |
|-----------|---------|-------|
| Actor spawn (single) | 624.74 ns | Context + lifecycle creation |
| Actor spawn (batch 10) | 681.40 ns/actor | Only 9% overhead |
| Message processing | 31.55 ns/msg | Direct handle_message call |

**Memory:**

- `ActorContext<M, B>`: ~200 bytes (address, timestamps, broker clone, stats)
- `ActorLifecycle`: ~32 bytes (state, timestamp, counter)
- Actor implementation: Varies (user-defined state)

---

## Mailbox Layer

### Overview

Message queue management providing buffering and backpressure control.

**Location:** `src/mailbox/`

**Responsibilities:**

- Buffer incoming messages
- Implement backpressure strategies
- Track mailbox metrics
- Provide async receive interface

### Components

#### Mailbox Traits

```rust
// src/mailbox/mod.rs
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

#### UnboundedMailbox

```rust
// src/mailbox/unbounded.rs
pub struct UnboundedMailbox<M: Message> {
    receiver: mpsc::UnboundedReceiver<MessageEnvelope<M>>,
    metrics: Arc<AtomicMetrics>,
}

impl<M: Message> UnboundedMailbox<M> {
    pub fn new() -> (UnboundedMailboxSender<M>, Self) {
        let (sender, receiver) = mpsc::unbounded_channel();
        let metrics = Arc::new(AtomicMetrics::new());
        
        let mailbox = Self { receiver, metrics: metrics.clone() };
        let sender = UnboundedMailboxSender { sender, metrics };
        
        (sender, mailbox)
    }
}

#[async_trait]
impl<M: Message> MailboxReceiver<M> for UnboundedMailbox<M> {
    async fn recv(&mut self) -> Option<MessageEnvelope<M>> {
        self.receiver.recv().await
    }

    fn try_recv(&mut self) -> Result<MessageEnvelope<M>, TryRecvError> {
        self.receiver.try_recv()
    }
}
```

**Characteristics:**

- Unlimited capacity (bounded only by memory)
- No backpressure (sender never blocks)
- Tokio `mpsc::unbounded_channel` backend
- Atomic metrics tracking

**Use Cases:**

- Low-volume control messages
- Actors with predictable load
- Development and prototyping

#### BoundedMailbox

```rust
// src/mailbox/bounded.rs
pub struct BoundedMailbox<M: Message> {
    receiver: mpsc::Receiver<MessageEnvelope<M>>,
    capacity: usize,
    backpressure: BackpressureStrategy,
    metrics: Arc<AtomicMetrics>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BackpressureStrategy {
    Block,      // Block sender when mailbox full
    Drop,       // Drop new messages when full
    DropOldest, // Drop oldest message to make room
}

impl<M: Message> BoundedMailbox<M> {
    pub fn new(
        capacity: usize,
        backpressure: BackpressureStrategy,
    ) -> (BoundedMailboxSender<M>, Self) {
        let (sender, receiver) = mpsc::channel(capacity);
        let metrics = Arc::new(AtomicMetrics::new());
        
        let mailbox = Self {
            receiver,
            capacity,
            backpressure,
            metrics: metrics.clone(),
        };
        
        let sender = BoundedMailboxSender {
            sender,
            backpressure,
            metrics,
        };
        
        (sender, mailbox)
    }
}
```

**Characteristics:**

- Fixed capacity (prevents unbounded growth)
- Configurable backpressure strategy
- Tokio `mpsc::channel(capacity)` backend
- Atomic metrics tracking

**Use Cases:**

- High-volume data streams
- Memory-constrained environments
- Flow control requirements

#### MailboxError

```rust
// src/mailbox/bounded.rs
#[derive(Debug)]
pub enum MailboxError {
    Full,       // Bounded mailbox at capacity
    Closed,     // Receiver dropped
    Timeout,    // Receive timeout exceeded
}
```

### Performance

From `benches/message_benchmarks.rs`:

| Operation | Latency | Notes |
|-----------|---------|-------|
| Mailbox enqueue + dequeue (100 ops) | 181.60 ns/op | Tokio channel efficiency |
| Bounded mailbox (capacity 100) | 244.18 ns/mailbox | Creation overhead |

**Memory:**

- `UnboundedMailbox<M>`: ~100 bytes base + queue size
- `BoundedMailbox<M>`: ~150 bytes base + (capacity × sizeof(envelope))

### Backpressure Strategies

**Block Strategy:**
```rust
// Sender blocks until space available
mailbox_sender.send(envelope).await?;  // Waits if full
```

**Drop Strategy:**
```rust
// New messages dropped when full
if mailbox.is_full() {
    return Err(MailboxError::Full);
}
```

**DropOldest Strategy:**
```rust
// Remove oldest message, add new message
if mailbox.is_full() {
    mailbox.pop_front();
}
mailbox.push_back(envelope);
```

---

## Supervisor Layer

### Overview

Fault tolerance layer implementing Erlang/OTP supervision patterns with builder-based configuration.

**Location:** `src/supervisor/`

**Responsibilities:**

- Supervise child actors/components
- Implement restart strategies
- Handle child lifecycle
- Provide builder API for type-safe configuration

### Components

#### Child Trait

```rust
// src/supervisor/child.rs
#[async_trait]
pub trait Child: Send + Sync {
    async fn start(&mut self) -> Result<(), Box<dyn Error + Send + Sync>>;
    async fn stop(&mut self) -> Result<(), Box<dyn Error + Send + Sync>>;
    async fn health_check(&self) -> ChildHealth;
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ChildHealth {
    Healthy,
    Unhealthy(String),
    Unknown,
}
```

**Blanket Implementation for Actors:**
```rust
#[async_trait]
impl<A: Actor> Child for A {
    async fn start(&mut self) -> Result<(), Box<dyn Error + Send + Sync>> {
        // Calls Actor::pre_start
    }

    async fn stop(&mut self) -> Result<(), Box<dyn Error + Send + Sync>> {
        // Calls Actor::post_stop
    }

    async fn health_check(&self) -> ChildHealth {
        // Default implementation based on lifecycle state
    }
}
```

#### ChildSpec

```rust
// src/supervisor/child.rs
pub struct ChildSpec {
    pub id: ChildId,
    pub restart_policy: RestartPolicy,
    pub shutdown_policy: ShutdownPolicy,
    pub significant: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RestartPolicy {
    Permanent,   // Always restart on failure
    Transient,   // Restart only on abnormal termination
    Temporary,   // Never restart
}

#[derive(Debug, Clone)]
pub struct ShutdownPolicy {
    pub timeout: Duration,
    pub strategy: ShutdownStrategy,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ShutdownStrategy {
    Graceful,  // Call stop(), wait for timeout
    Brutal,    // Immediate termination
}
```

#### RestartStrategy Trait

```rust
// src/supervisor/strategy.rs
#[async_trait]
pub trait RestartStrategy: Send + Sync {
    async fn handle_failure(
        &self,
        failed_child_id: &ChildId,
        children: &mut Vec<(ChildSpec, Box<dyn Child>)>,
    ) -> Result<(), SupervisorError>;
}
```

#### OneForOne Strategy

```rust
// src/supervisor/strategy.rs
pub struct OneForOne;

impl OneForOne {
    pub fn new() -> Self { Self }
}

#[async_trait]
impl RestartStrategy for OneForOne {
    async fn handle_failure(
        &self,
        failed_child_id: &ChildId,
        children: &mut Vec<(ChildSpec, Box<dyn Child>)>,
    ) -> Result<(), SupervisorError> {
        // Find failed child
        let (spec, child) = children.iter_mut()
            .find(|(s, _)| &s.id == failed_child_id)
            .ok_or(SupervisorError::ChildNotFound(failed_child_id.clone()))?;

        // Restart only this child
        child.stop().await.map_err(|e| SupervisorError::ChildStopFailed(spec.id.clone(), e))?;
        child.start().await.map_err(|e| SupervisorError::ChildStartFailed(spec.id.clone(), e))?;

        Ok(())
    }
}
```

#### OneForAll Strategy

```rust
// src/supervisor/strategy.rs
pub struct OneForAll;

#[async_trait]
impl RestartStrategy for OneForAll {
    async fn handle_failure(
        &self,
        _failed_child_id: &ChildId,
        children: &mut Vec<(ChildSpec, Box<dyn Child>)>,
    ) -> Result<(), SupervisorError> {
        // Stop all children
        for (spec, child) in children.iter_mut() {
            child.stop().await.map_err(|e| SupervisorError::ChildStopFailed(spec.id.clone(), e))?;
        }

        // Start all children
        for (spec, child) in children.iter_mut() {
            child.start().await.map_err(|e| SupervisorError::ChildStartFailed(spec.id.clone(), e))?;
        }

        Ok(())
    }
}
```

#### RestForOne Strategy

```rust
// src/supervisor/strategy.rs
pub struct RestForOne;

#[async_trait]
impl RestartStrategy for RestForOne {
    async fn handle_failure(
        &self,
        failed_child_id: &ChildId,
        children: &mut Vec<(ChildSpec, Box<dyn Child>)>,
    ) -> Result<(), SupervisorError> {
        // Find failed child index
        let failed_index = children.iter()
            .position(|(s, _)| &s.id == failed_child_id)
            .ok_or(SupervisorError::ChildNotFound(failed_child_id.clone()))?;

        // Stop failed child and all subsequent children
        for (spec, child) in children[failed_index..].iter_mut() {
            child.stop().await.map_err(|e| SupervisorError::ChildStopFailed(spec.id.clone(), e))?;
        }

        // Restart failed child and all subsequent children
        for (spec, child) in children[failed_index..].iter_mut() {
            child.start().await.map_err(|e| SupervisorError::ChildStartFailed(spec.id.clone(), e))?;
        }

        Ok(())
    }
}
```

#### SupervisorNode

```rust
// src/supervisor/supervisor.rs
pub struct SupervisorNode {
    id: SupervisorId,
    strategy: Box<dyn RestartStrategy>,
    children: Vec<(ChildSpec, Box<dyn Child>)>,
    lifecycle: ActorLifecycle,
}

impl SupervisorNode {
    pub fn builder() -> SupervisorBuilder {
        SupervisorBuilder::new()
    }

    pub fn new<S: RestartStrategy + 'static>(
        id: SupervisorId,
        strategy: S,
    ) -> Self {
        Self {
            id,
            strategy: Box::new(strategy),
            children: Vec::new(),
            lifecycle: ActorLifecycle::new(),
        }
    }

    pub async fn add_child(
        &mut self,
        spec: ChildSpec,
        child: Box<dyn Child>,
    ) -> Result<(), SupervisorError> {
        self.children.push((spec, child));
        Ok(())
    }

    pub async fn start_all(&mut self) -> Result<(), SupervisorError> {
        self.lifecycle.transition_to(ActorState::Starting);
        
        for (spec, child) in &mut self.children {
            child.start().await
                .map_err(|e| SupervisorError::ChildStartFailed(spec.id.clone(), e))?;
        }
        
        self.lifecycle.transition_to(ActorState::Running);
        Ok(())
    }

    pub async fn stop_all(&mut self) -> Result<(), SupervisorError> {
        self.lifecycle.transition_to(ActorState::Stopping);
        
        for (spec, child) in &mut self.children {
            child.stop().await
                .map_err(|e| SupervisorError::ChildStopFailed(spec.id.clone(), e))?;
        }
        
        self.lifecycle.transition_to(ActorState::Stopped);
        Ok(())
    }

    pub async fn handle_child_failure(
        &mut self,
        child_id: &ChildId,
    ) -> Result<(), SupervisorError> {
        self.strategy.handle_failure(child_id, &mut self.children).await
    }
}
```

#### SupervisorBuilder (RT-TASK-013)

```rust
// src/supervisor/builder.rs
pub struct SupervisorBuilder {
    id: Option<SupervisorId>,
    strategy: Option<Box<dyn RestartStrategy>>,
    children: Vec<(ChildSpec, Box<dyn Child>)>,
}

impl SupervisorBuilder {
    pub fn new() -> Self {
        Self {
            id: None,
            strategy: None,
            children: Vec::new(),
        }
    }

    pub fn with_id(mut self, id: SupervisorId) -> Self {
        self.id = Some(id);
        self
    }

    pub fn with_strategy<S: RestartStrategy + 'static>(mut self, strategy: S) -> Self {
        self.strategy = Some(Box::new(strategy));
        self
    }

    pub fn add_child(
        mut self,
        spec: ChildSpec,
        child: Box<dyn Child>,
    ) -> Self {
        self.children.push((spec, child));
        self
    }

    pub fn build(self) -> Result<SupervisorNode, SupervisorError> {
        let id = self.id.unwrap_or_else(SupervisorId::new);
        let strategy = self.strategy
            .ok_or(SupervisorError::StrategyError("No strategy specified".to_string()))?;

        let mut supervisor = SupervisorNode {
            id,
            strategy,
            children: Vec::new(),
            lifecycle: ActorLifecycle::new(),
        };

        supervisor.children = self.children;

        Ok(supervisor)
    }
}
```

### Performance

From `benches/supervisor_benchmarks.rs`:

| Operation | Latency | Notes |
|-----------|---------|-------|
| Child spawn (builder API) | 5-20 µs | Type-safe configuration |
| OneForOne restart | 10-50 µs | Single child lifecycle |
| OneForAll restart (3 children) | 30-150 µs | ~3x OneForOne |
| RestForOne restart (2 children) | 20-100 µs | Between OneForOne and OneForAll |

**Memory:**

- `SupervisorNode`: ~200 bytes base + children vec
- Per child: ~80 bytes (ChildSpec + Box<dyn Child> pointer)

---

## Monitoring Layer

### Overview

Health checks and metrics tracking for actors and supervisors.

**Location:** `src/monitoring/`

**Responsibilities:**

- Monitor child health status
- Track actor performance metrics
- Provide automatic health checks
- Alert on unhealthy children

### Components

#### HealthMonitor

```rust
// src/monitoring/health.rs
pub struct HealthMonitor {
    config: HealthConfig,
    checks: Vec<HealthCheck>,
}

pub struct HealthConfig {
    pub check_interval: Duration,
    pub unhealthy_threshold: u32,
    pub auto_restart: bool,
}

impl HealthMonitor {
    pub fn new(config: HealthConfig) -> Self {
        Self {
            config,
            checks: Vec::new(),
        }
    }

    pub async fn monitor_supervisor(
        &mut self,
        supervisor: &mut SupervisorNode,
    ) -> Result<(), MonitoringError> {
        loop {
            tokio::time::sleep(self.config.check_interval).await;

            for (spec, child) in supervisor.children_mut() {
                let health = child.health_check().await;

                if health == ChildHealth::Unhealthy {
                    if self.config.auto_restart {
                        supervisor.handle_child_failure(&spec.id).await?;
                    }
                }
            }
        }
    }
}
```

#### ActorMetrics

```rust
// src/monitoring/metrics.rs
pub struct ActorMetrics {
    pub message_count: AtomicU64,
    pub error_count: AtomicU64,
    pub last_message_at: AtomicU64,  // Unix timestamp
    pub processing_time_ns: AtomicU64,
}

impl ActorMetrics {
    pub fn new() -> Self {
        Self {
            message_count: AtomicU64::new(0),
            error_count: AtomicU64::new(0),
            last_message_at: AtomicU64::new(0),
            processing_time_ns: AtomicU64::new(0),
        }
    }

    pub fn record_message(&self, processing_time: Duration) {
        self.message_count.fetch_add(1, Ordering::Relaxed);
        self.last_message_at.store(
            Utc::now().timestamp() as u64,
            Ordering::Relaxed,
        );
        self.processing_time_ns.fetch_add(
            processing_time.as_nanos() as u64,
            Ordering::Relaxed,
        );
    }

    pub fn record_error(&self) {
        self.error_count.fetch_add(1, Ordering::Relaxed);
    }

    pub fn message_count(&self) -> u64 {
        self.message_count.load(Ordering::Relaxed)
    }

    pub fn error_count(&self) -> u64 {
        self.error_count.load(Ordering::Relaxed)
    }

    pub fn average_processing_time(&self) -> Duration {
        let total = self.processing_time_ns.load(Ordering::Relaxed);
        let count = self.message_count.load(Ordering::Relaxed);
        
        if count == 0 {
            Duration::from_nanos(0)
        } else {
            Duration::from_nanos(total / count)
        }
    }
}
```

### Performance

- Health check overhead: Depends on `Child::health_check()` implementation
- Metrics update: Lock-free atomic operations (~5-10 ns)
- Monitoring loop: Configurable interval (default: 5 seconds)

**Memory:**

- `HealthMonitor`: ~100 bytes + checks vec
- `ActorMetrics`: 32 bytes (4 × AtomicU64)

---

## System Layer (Planned)

### Overview

Future runtime coordination layer for actor registry and distributed nodes.

**Location:** `src/system/` (planned for Q1 2026)

**Planned Responsibilities:**

- Global actor registry
- Actor address resolution
- System lifecycle management
- Distributed node coordination (future)

### Planned Components

#### ActorSystem (Not Yet Implemented)

```rust
// Planned design
pub struct ActorSystem {
    registry: ActorRegistry,
    root_supervisor: SupervisorNode,
    config: SystemConfig,
}

impl ActorSystem {
    pub async fn new(config: SystemConfig) -> Result<Self, SystemError>;
    pub async fn spawn_actor<A: Actor>(&mut self, actor: A) -> ActorAddress;
    pub async fn lookup(&self, address: &ActorAddress) -> Option<ActorRef>;
    pub async fn shutdown(&mut self) -> Result<(), SystemError>;
}
```

#### ActorRegistry (Not Yet Implemented)

```rust
// Planned design
pub struct ActorRegistry {
    actors: HashMap<ActorAddress, ActorRef>,
}

impl ActorRegistry {
    pub fn register(&mut self, address: ActorAddress, actor_ref: ActorRef);
    pub fn unregister(&mut self, address: &ActorAddress);
    pub fn lookup(&self, address: &ActorAddress) -> Option<&ActorRef>;
}
```

**Status:** Architecture designed, implementation planned for Q1 2026.

---

## Component Integration

### Data Flow Diagram

Complete message flow from send to receive:

```
Actor A                  Broker                   Actor B
   │                        │                        │
   │ 1. send(msg)           │                        │
   ├───────────────────────>│                        │
   │                        │                        │
   │                    2. publish()                 │
   │                    (wrap envelope)              │
   │                        │                        │
   │                    3. route to                  │
   │                    subscribers                  │
   │                        ├───────────────────────>│
   │                        │                  4. recv()
   │                        │                  (mailbox)
   │                        │                        │
   │                        │              5. handle_message()
   │                        │                        │
```

### Supervision Integration

How supervision integrates with other layers:

```
┌────────────────────────────────────────────────────┐
│             SupervisorNode                          │
│  - Manages child lifecycle                          │
│  - Applies restart strategy                         │
└──────────┬────────────────────────┬────────────────┘
           │                        │
           │ supervises             │ monitors
           │                        │
┌──────────▼──────────┐   ┌─────────▼──────────┐
│   Child (Actor)     │   │  HealthMonitor     │
│  - implements Child │   │  - health_check()  │
│  - ActorLifecycle   │   │  - auto-restart    │
└──────────┬──────────┘   └────────────────────┘
           │
           │ uses
           │
┌──────────▼──────────┐
│   ActorContext      │
│  - MessageBroker    │
│  - ActorAddress     │
└──────────┬──────────┘
           │
           │ publishes via
           │
┌──────────▼──────────────┐
│  InMemoryMessageBroker  │
│  - routes messages      │
└─────────────────────────┘
```

## Next Steps

For architecture overviews and cross-cutting concerns, see:

- [System Overview](./system-overview.md) - High-level architecture
- [Core Concepts](./core-concepts.md) - Fundamental concepts
- [Actor Model](./actor-model.md) - Actor design patterns
- [Message Passing](./message-passing.md) - Messaging details
- [Supervision](./supervision.md) - Fault tolerance patterns
- [Process Lifecycle](./process-lifecycle.md) - State management

For implementation guidance:

- [Actor Development Guide](../guides/actor-development.md) - Building actors
- [Supervisor Patterns Guide](../guides/supervisor-patterns.md) - Supervision setup
- [Message Passing Guide](../guides/message-passing.md) - Messaging patterns
