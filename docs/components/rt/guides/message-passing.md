# Message Passing Best Practices

> **Target Audience**: Developers optimizing message patterns and communication in actor systems
>
> **Guide Type**: HOW-TO (task-oriented) - Solutions for message design and performance optimization

## Overview

This guide covers best practices for designing, implementing, and optimizing message passing in AirsSys RT. Message passing is the fundamental communication mechanism in the actor model, and proper message design directly impacts system performance, maintainability, and reliability.

**What you'll learn:**

- How to design efficient message types
- Communication patterns for different scenarios
- Performance optimization techniques
- Error handling strategies in messaging
- Type safety and message evolution

**Prerequisites:**

- Completed [Getting Started](../implementation/getting-started.md)
- Understanding of [Actor Development](actor-development.md) basics
- Familiarity with Rust async/await and channels

---

## 1. Message Design Principles

Effective message design balances performance, clarity, and maintainability. Follow these principles for optimal results.

### 1.1 Keep Messages Small

**Principle**: Small messages improve throughput and reduce memory pressure.

```rust
use airssys_rt::message::Message;

// ✅ GOOD - Small, focused message
#[derive(Debug, Clone)]
pub enum CounterMsg {
    Increment,
    Decrement,
    GetValue(tokio::sync::oneshot::Sender<i32>),
}

impl Message for CounterMsg {
    type Result = ();
}

// ❌ BAD - Large message with unnecessary data
#[derive(Debug, Clone)]
pub struct BloatedMsg {
    pub large_vec: Vec<u8>,           // Large allocation
    pub metadata: HashMap<String, String>, // Cloned on every send
    pub timestamp: String,             // Could be computed
}
```

**Why it matters:**

- Smaller messages = faster serialization (if needed)
- Reduced memory allocations
- Better cache locality
- Lower clone costs

**Performance data** (from BENCHMARKING.md §6.2):
- Small messages (<64 bytes): ~50-100ns send latency
- Medium messages (64-512 bytes): ~100-200ns send latency
- Large messages (>512 bytes): ~200ns+ send latency

### 1.2 Immutable Message Data

**Principle**: Messages should be immutable to prevent data races and simplify reasoning.

```rust
// ✅ GOOD - Immutable message
#[derive(Debug, Clone)]
pub struct UpdateConfig {
    pub new_value: String,  // Owned, immutable
    pub version: u32,
}

// ❌ BAD - Mutable shared state
pub struct MutableMsg {
    pub shared_data: Arc<Mutex<Vec<String>>>, // Invites race conditions
}
```

**Benefits:**

- Thread-safe by design
- No accidental mutations
- Easier to test and debug
- Aligns with actor model isolation

### 1.3 Avoid Large Clones with `Arc<T>`

**Principle**: For large data, use `Arc<T>` to avoid expensive clones.

```rust
use std::sync::Arc;

// ✅ GOOD - Share large data efficiently
#[derive(Debug, Clone)]
pub struct ProcessData {
    pub data: Arc<Vec<u8>>,  // Cheap clone (ref count increment)
    pub metadata: String,     // Small, owned
}

impl Message for ProcessData {
    type Result = ();
}

// Example usage
let large_data = vec![0u8; 1_000_000]; // 1MB
let msg = ProcessData {
    data: Arc::new(large_data),
    metadata: "sensor_data".to_string(),
};

// Clone is cheap - only increments reference count
let msg_clone = msg.clone(); // ~10ns, not 1MB copy
```

**Guidelines:**

- Use `Arc<T>` when data > 1KB
- Use `Arc<T>` when sending to multiple actors
- Use owned data for small types (<64 bytes)
- Consider `Arc<[T]>` for immutable slices

**Performance comparison:**
```text
Clone 1KB Vec<u8>:        ~1,000ns (copy)
Clone Arc<1KB Vec<u8>>:   ~10ns (ref count)
Clone 1MB Vec<u8>:        ~1,000,000ns (copy)
Clone Arc<1MB Vec<u8>>:   ~10ns (ref count)
```

### 1.4 Message Type Organization

**Principle**: Choose between enum messages and separate types based on actor complexity.

**Pattern A: Enum Messages (Simple Actors)**

```rust
// ✅ GOOD - Single enum for related operations
#[derive(Debug, Clone)]
pub enum WorkerMsg {
    Start,
    Stop,
    Process(Arc<Vec<u8>>),
    Status(tokio::sync::oneshot::Sender<String>),
}

impl Message for WorkerMsg {
    type Result = ();
}
```

**Benefits:**

- Clear API surface
- Easy pattern matching
- Single message type per actor

**Pattern B: Separate Types (Complex Actors)**

```rust
// ✅ GOOD - Separate types for distinct domains
#[derive(Debug, Clone)]
pub struct StartProcessing {
    pub config: Arc<Config>,
}

impl Message for StartProcessing {
    type Result = Result<(), String>;
}

#[derive(Debug, Clone)]
pub struct GetMetrics {
    pub reply: tokio::sync::oneshot::Sender<Metrics>,
}

impl Message for GetMetrics {
    type Result = ();
}
```

**Benefits:**

- Explicit type signatures
- Better for actors with many operations
- Easier to evolve independently

**Choose enum when:**

- Actor has <10 message types
- Messages are closely related
- Simple request/response patterns

**Choose separate types when:**

- Actor has >10 message types
- Messages have different result types
- Complex domain logic

---

## 2. Communication Patterns

Different communication needs require different patterns. Choose the right pattern for your use case.

### 2.1 Fire-and-Forget (Async Send)

**Use case**: Send message without waiting for result.

```rust
use airssys_rt::actor::{Actor, ActorContext, ActorError};
use airssys_rt::message::Message;

#[derive(Debug, Clone)]
pub enum LogMsg {
    Info(String),
    Error(String),
}

impl Message for LogMsg {
    type Result = ();
}

pub struct Logger;

#[async_trait::async_trait]
impl Actor for Logger {
    type Message = LogMsg;

    async fn handle(&mut self, msg: Self::Message, _ctx: &ActorContext) -> Result<(), ActorError> {
        match msg {
            LogMsg::Info(msg) => println!("[INFO] {}", msg),
            LogMsg::Error(msg) => eprintln!("[ERROR] {}", msg),
        }
        Ok(())
    }
}

// Usage - fire and forget
async fn example(logger_ref: ActorRef<LogMsg>) {
    // Send without awaiting - fire and forget
    logger_ref.send(LogMsg::Info("Processing started".to_string())).await.ok();
    
    // Continue immediately without waiting for log to be processed
    do_work().await;
}
```

**When to use:**

- Logging, metrics, notifications
- One-way commands
- When sender doesn't need confirmation
- High-throughput scenarios

**Performance**: ~50-100ns send latency (BENCHMARKING.md §6.2)

### 2.2 Request/Reply (Oneshot Channels)

**Use case**: Send message and wait for response.

```rust
use tokio::sync::oneshot;

#[derive(Debug, Clone)]
pub enum QueryMsg {
    GetCount(oneshot::Sender<i32>),
    GetStatus(oneshot::Sender<String>),
}

impl Message for QueryMsg {
    type Result = ();
}

pub struct Counter {
    count: i32,
}

#[async_trait::async_trait]
impl Actor for Counter {
    type Message = QueryMsg;

    async fn handle(&mut self, msg: Self::Message, _ctx: &ActorContext) -> Result<(), ActorError> {
        match msg {
            QueryMsg::GetCount(reply) => {
                let _ = reply.send(self.count);
            }
            QueryMsg::GetStatus(reply) => {
                let status = format!("Counter at {}", self.count);
                let _ = reply.send(status);
            }
        }
        Ok(())
    }
}

// Usage - request/reply
async fn example(counter_ref: ActorRef<QueryMsg>) -> Result<i32, String> {
    let (tx, rx) = oneshot::channel();
    
    counter_ref.send(QueryMsg::GetCount(tx))
        .await
        .map_err(|e| format!("Send failed: {}", e))?;
    
    rx.await
        .map_err(|e| format!("Receive failed: {}", e))
}
```

**When to use:**

- Queries that need results
- Synchronous-style APIs
- When caller needs confirmation
- RPC-style interactions

**Pattern variations:**
```rust
// Variation 1: Return Result
#[derive(Debug, Clone)]
pub struct Compute {
    pub value: i32,
    pub reply: oneshot::Sender<Result<i32, String>>,
}

// Variation 2: Multiple response types
pub enum Response {
    Success(Data),
    NotFound,
    Error(String),
}

#[derive(Debug, Clone)]
pub struct Query {
    pub key: String,
    pub reply: oneshot::Sender<Response>,
}
```

### 2.3 Pub/Sub (Via MessageBroker)

**Use case**: Broadcast messages to multiple subscribers.

> **Note**: MessageBroker is implemented in RT-TASK-007. See [broker module](../../src/broker/) for details.

```rust
use airssys_rt::broker::{MessageBroker, Topic};
use std::sync::Arc;

#[derive(Debug, Clone)]
pub struct SensorReading {
    pub sensor_id: String,
    pub value: f64,
    pub timestamp: i64,
}

impl Message for SensorReading {
    type Result = ();
}

// Publisher
pub struct SensorPublisher {
    broker: Arc<MessageBroker<SensorReading>>,
}

impl SensorPublisher {
    pub async fn publish_reading(&self, reading: SensorReading) {
        self.broker.publish("sensor.readings".into(), reading).await;
    }
}

// Subscriber
pub struct MetricsCollector {
    broker: Arc<MessageBroker<SensorReading>>,
}

#[async_trait::async_trait]
impl Actor for MetricsCollector {
    type Message = SensorReading;

    async fn handle(&mut self, msg: Self::Message, _ctx: &ActorContext) -> Result<(), ActorError> {
        // Process sensor reading
        println!("Received reading: {} = {}", msg.sensor_id, msg.value);
        Ok(())
    }
}

async fn setup_pubsub() {
    let broker = Arc::new(MessageBroker::new());
    
    // Subscribe to topic
    let topic: Topic = "sensor.readings".into();
    let mut subscriber = broker.subscribe(topic.clone()).await;
    
    // Spawn subscriber task
    tokio::spawn(async move {
        while let Some(reading) = subscriber.recv().await {
            // Process reading
            println!("Subscriber received: {:?}", reading);
        }
    });
    
    // Publish messages
    broker.publish(topic, SensorReading {
        sensor_id: "temp-001".to_string(),
        value: 23.5,
        timestamp: 1234567890,
    }).await;
}
```

**When to use:**

- Event notifications
- Broadcasting state changes
- Decoupling publishers from subscribers
- Fan-out patterns

**Performance**: ~100-200ns per subscriber (BENCHMARKING.md §6.2.3)

### 2.4 Broadcast Patterns

**Use case**: Send same message to multiple actors.

```rust
use airssys_rt::actor::ActorRef;

async fn broadcast_shutdown(workers: Vec<ActorRef<WorkerMsg>>) {
    for worker in workers {
        worker.send(WorkerMsg::Stop).await.ok();
    }
}

// Parallel broadcast with timeout
async fn parallel_broadcast(
    workers: Vec<ActorRef<WorkerMsg>>,
    timeout: Duration,
) -> Result<(), String> {
    use tokio::time::timeout as tokio_timeout;
    
    let futures: Vec<_> = workers.iter()
        .map(|w| w.send(WorkerMsg::Stop))
        .collect();
    
    tokio_timeout(timeout, futures::future::join_all(futures))
        .await
        .map_err(|_| "Broadcast timeout")?;
    
    Ok(())
}
```

**When to use:**

- Shutdown/restart commands
- Configuration updates
- Coordinated state changes

### 2.5 Scatter/Gather Patterns

**Use case**: Send to multiple actors and collect responses.

```rust
async fn scatter_gather_query(
    workers: Vec<ActorRef<QueryMsg>>,
) -> Vec<i32> {
    let mut replies = Vec::new();
    
    for worker in workers {
        let (tx, rx) = oneshot::channel();
        if worker.send(QueryMsg::GetCount(tx)).await.is_ok() {
            if let Ok(count) = rx.await {
                replies.push(count);
            }
        }
    }
    
    replies
}

// Parallel scatter/gather with timeout
async fn parallel_scatter_gather(
    workers: Vec<ActorRef<QueryMsg>>,
    timeout: Duration,
) -> Result<Vec<i32>, String> {
    use tokio::time::timeout as tokio_timeout;
    
    let futures: Vec<_> = workers.iter().map(|worker| async {
        let (tx, rx) = oneshot::channel();
        worker.send(QueryMsg::GetCount(tx)).await.ok()?;
        rx.await.ok()
    }).collect();
    
    let results = tokio_timeout(timeout, futures::future::join_all(futures))
        .await
        .map_err(|_| "Gather timeout")?;
    
    Ok(results.into_iter().flatten().collect())
}
```

**When to use:**

- Aggregating data from multiple sources
- Parallel query processing
- Consensus algorithms
- Load distribution

---

## 3. Performance Optimization

Optimize message passing for high-throughput scenarios.

### 3.1 Zero-Copy Patterns with Arc

**Strategy**: Share data without copying using `Arc<T>`.

```rust
use std::sync::Arc;

// Efficient data sharing
pub struct DataProcessor;

#[async_trait::async_trait]
impl Actor for DataProcessor {
    type Message = ProcessData;

    async fn handle(&mut self, msg: Self::Message, ctx: &ActorContext) -> Result<(), ActorError> {
        // Data is shared, not copied
        let data_ref = &msg.data; // Arc<Vec<u8>>
        
        // Send to another actor - no copy!
        let downstream_msg = ProcessData {
            data: Arc::clone(&msg.data), // Just ref count increment
            metadata: "processed".to_string(),
        };
        
        // downstream_actor.send(downstream_msg).await?;
        Ok(())
    }
}
```

**Performance gain:**

- 1MB message copy: ~1ms
- 1MB Arc clone: ~10ns
- **100x faster** for large data

### 3.2 Batching Messages

**Strategy**: Process multiple messages together to reduce per-message overhead.

```rust
#[derive(Debug, Clone)]
pub enum BatchedMsg {
    Single(String),
    Batch(Vec<String>),
}

impl Message for BatchedMsg {
    type Result = ();
}

pub struct BatchProcessor {
    batch: Vec<String>,
    batch_size: usize,
}

impl BatchProcessor {
    fn new(batch_size: usize) -> Self {
        Self {
            batch: Vec::with_capacity(batch_size),
            batch_size,
        }
    }
    
    async fn process_batch(&mut self) {
        if !self.batch.is_empty() {
            // Process entire batch at once
            println!("Processing batch of {} items", self.batch.len());
            // ... bulk processing logic ...
            self.batch.clear();
        }
    }
}

#[async_trait::async_trait]
impl Actor for BatchProcessor {
    type Message = BatchedMsg;

    async fn handle(&mut self, msg: Self::Message, _ctx: &ActorContext) -> Result<(), ActorError> {
        match msg {
            BatchedMsg::Single(item) => {
                self.batch.push(item);
                if self.batch.len() >= self.batch_size {
                    self.process_batch().await;
                }
            }
            BatchedMsg::Batch(items) => {
                self.batch.extend(items);
                if self.batch.len() >= self.batch_size {
                    self.process_batch().await;
                }
            }
        }
        Ok(())
    }
}
```

**Performance gain:**

- Individual processing: ~1,000ns per message
- Batched processing: ~100ns per message (10x improvement)
- Trade-off: Increased latency for batch accumulation

**When to use:**

- Database writes
- Network requests
- File I/O operations
- High-volume data processing

### 3.3 Priority Messages

**Strategy**: Process critical messages before normal messages.

> **Note**: Priority messaging is available via custom mailbox implementations.

```rust
use airssys_rt::mailbox::{Mailbox, MailboxConfig};

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Priority {
    High,
    Normal,
}

#[derive(Debug, Clone)]
pub struct PriorityMsg {
    pub priority: Priority,
    pub data: String,
}

impl Message for PriorityMsg {
    type Result = ();
}

// Custom mailbox with priority handling
pub struct PriorityMailbox {
    high_priority: Vec<PriorityMsg>,
    normal: Vec<PriorityMsg>,
}

impl PriorityMailbox {
    pub fn next_message(&mut self) -> Option<PriorityMsg> {
        // Always process high priority first
        if let Some(msg) = self.high_priority.pop() {
            Some(msg)
        } else {
            self.normal.pop()
        }
    }
}
```

**When to use:**

- Health check messages (high priority)
- Shutdown commands (high priority)
- Control plane vs data plane separation
- Real-time vs batch processing

### 3.4 Message Pooling

**Strategy**: Reuse message allocations for extremely high-throughput scenarios.

> **Note**: Message pooling is an advanced optimization. Only use when profiling shows allocation overhead.

```rust
use std::sync::Arc;
use parking_lot::Mutex;

// Object pool for message data
pub struct MessagePool<T> {
    pool: Arc<Mutex<Vec<Box<T>>>>,
}

impl<T: Default> MessagePool<T> {
    pub fn new(capacity: usize) -> Self {
        let mut pool = Vec::with_capacity(capacity);
        for _ in 0..capacity {
            pool.push(Box::new(T::default()));
        }
        
        Self {
            pool: Arc::new(Mutex::new(pool)),
        }
    }
    
    pub fn acquire(&self) -> Option<Box<T>> {
        self.pool.lock().pop()
    }
    
    pub fn release(&self, mut item: Box<T>) {
        *item = T::default(); // Reset
        self.pool.lock().push(item);
    }
}

// Example usage
pub struct PooledData {
    pub buffer: Vec<u8>,
}

impl Default for PooledData {
    fn default() -> Self {
        Self {
            buffer: Vec::with_capacity(1024),
        }
    }
}
```

**Performance gain:**

- Without pooling: ~1,000ns per allocation
- With pooling: ~100ns per acquisition
- **10x faster** for allocation-heavy workloads

**When to use:**

- Message rates >100,000/sec
- Large message buffers
- Real-time systems with strict latency requirements
- Only after profiling confirms allocation bottleneck

### 3.5 Performance Benchmarks

**Reference data** from BENCHMARKING.md §6.2:

```text
Message Send Latency:
- Small message (<64B):     50-100ns
- Medium message (64-512B): 100-200ns
- Large message (>512B):    200ns+

Message Throughput:
- Single actor:             1M+ messages/sec
- With Arc<T> sharing:      2M+ messages/sec
- Batched (100/batch):      5M+ messages/sec

Channel Performance:
- Unbounded send:           ~50ns
- Bounded send (empty):     ~75ns
- Bounded send (full):      ~1,000ns (backpressure)
```

**Optimization priorities:**
1. Profile first - measure before optimizing
2. Use Arc<T> for large data (>1KB)
3. Batch when latency tolerance allows
4. Consider priority for critical messages
5. Pool only when allocations are proven bottleneck

---

## 4. Error Handling in Messaging

Robust error handling ensures system reliability.

### 4.1 Send Failures

**Scenario**: Actor stopped or mailbox full.

```rust
use airssys_rt::actor::ActorRef;

async fn robust_send(actor_ref: &ActorRef<WorkerMsg>) -> Result<(), String> {
    actor_ref.send(WorkerMsg::Process(Arc::new(vec![1, 2, 3])))
        .await
        .map_err(|e| format!("Send failed: {}", e))?;
    
    Ok(())
}

// Handle specific error types
async fn handle_send_errors(actor_ref: &ActorRef<WorkerMsg>) {
    match actor_ref.send(WorkerMsg::Start).await {
        Ok(_) => println!("Message sent successfully"),
        Err(e) => {
            eprintln!("Send failed: {}", e);
            // Common causes:
            // - Actor stopped
            // - Mailbox full (bounded mailbox)
            // - System shutdown
            
            // Recovery strategies:
            // 1. Retry with backoff
            // 2. Log and continue
            // 3. Escalate to supervisor
            // 4. Use circuit breaker pattern
        }
    }
}
```

**Error handling strategies:**

**Strategy 1: Retry with Exponential Backoff**
```rust
use tokio::time::{sleep, Duration};

async fn send_with_retry(
    actor_ref: &ActorRef<WorkerMsg>,
    msg: WorkerMsg,
    max_retries: u32,
) -> Result<(), String> {
    let mut delay = Duration::from_millis(10);
    
    for attempt in 0..max_retries {
        match actor_ref.send(msg.clone()).await {
            Ok(_) => return Ok(()),
            Err(e) if attempt < max_retries - 1 => {
                eprintln!("Send failed (attempt {}): {}", attempt + 1, e);
                sleep(delay).await;
                delay *= 2; // Exponential backoff
            }
            Err(e) => return Err(format!("Send failed after {} retries: {}", max_retries, e)),
        }
    }
    
    Err("Max retries exceeded".to_string())
}
```

**Strategy 2: Circuit Breaker**
```rust
pub struct CircuitBreaker {
    failure_count: u32,
    threshold: u32,
    state: CircuitState,
}

pub enum CircuitState {
    Closed,   // Normal operation
    Open,     // Failing, reject requests
    HalfOpen, // Testing recovery
}

impl CircuitBreaker {
    pub async fn send_protected(
        &mut self,
        actor_ref: &ActorRef<WorkerMsg>,
        msg: WorkerMsg,
    ) -> Result<(), String> {
        match self.state {
            CircuitState::Open => {
                Err("Circuit breaker open".to_string())
            }
            CircuitState::Closed | CircuitState::HalfOpen => {
                match actor_ref.send(msg).await {
                    Ok(_) => {
                        self.on_success();
                        Ok(())
                    }
                    Err(e) => {
                        self.on_failure();
                        Err(format!("Send failed: {}", e))
                    }
                }
            }
        }
    }
    
    fn on_success(&mut self) {
        self.failure_count = 0;
        self.state = CircuitState::Closed;
    }
    
    fn on_failure(&mut self) {
        self.failure_count += 1;
        if self.failure_count >= self.threshold {
            self.state = CircuitState::Open;
        }
    }
}
```

### 4.2 Timeout Patterns

**Strategy**: Set time limits on request/reply operations.

```rust
use tokio::time::{timeout, Duration};

async fn query_with_timeout(
    actor_ref: &ActorRef<QueryMsg>,
    timeout_duration: Duration,
) -> Result<i32, String> {
    let (tx, rx) = oneshot::channel();
    
    // Send message
    actor_ref.send(QueryMsg::GetCount(tx))
        .await
        .map_err(|e| format!("Send failed: {}", e))?;
    
    // Wait for response with timeout
    timeout(timeout_duration, rx)
        .await
        .map_err(|_| "Request timeout".to_string())?
        .map_err(|_| "Reply channel closed".to_string())
}

// Usage
async fn example(actor_ref: ActorRef<QueryMsg>) {
    match query_with_timeout(&actor_ref, Duration::from_secs(5)).await {
        Ok(count) => println!("Count: {}", count),
        Err(e) => eprintln!("Query failed: {}", e),
    }
}
```

**Timeout guidelines:**

- Fast queries: 100ms - 1s
- Normal operations: 1s - 5s
- Long-running tasks: 30s - 5min
- Always set timeouts for request/reply patterns

### 4.3 Dead Letter Handling

> **Note**: Dead letter queues are a planned feature (future RT task). Current approach: log and handle manually.

**Current approach:**
```rust
async fn send_with_dead_letter_logging(
    actor_ref: &ActorRef<WorkerMsg>,
    msg: WorkerMsg,
) {
    if let Err(e) = actor_ref.send(msg.clone()).await {
        // Log dead letter
        eprintln!("DEAD LETTER: Failed to send {:?}: {}", msg, e);
        
        // Optional: persist for later replay
        // dead_letter_store.save(msg).await;
    }
}
```

**Planned feature** (future):
```rust
// Future: Automatic dead letter queue
pub struct DeadLetterQueue<M: Message> {
    queue: Vec<M>,
}

// Messages that fail to send will be automatically queued
// for manual inspection or replay
```

---

## 5. Type Safety and Message Versioning

Maintain type safety and evolve messages over time.

### 5.1 Strong Typing Benefits

**Principle**: Use Rust's type system to prevent message errors at compile time.

```rust
// ✅ GOOD - Strong types prevent errors
#[derive(Debug, Clone)]
pub struct UpdateConfig {
    pub timeout: Duration,      // Type-safe duration
    pub max_retries: u32,       // Unsigned, can't be negative
    pub endpoint: Url,          // Validated URL type
}

// ❌ BAD - Weak types allow invalid states
pub struct WeakConfig {
    pub timeout_ms: i32,        // Could be negative!
    pub max_retries: String,    // Should be a number
    pub endpoint: String,       // Could be invalid URL
}

// ✅ GOOD - Enums prevent invalid states
#[derive(Debug, Clone)]
pub enum ConnectionState {
    Connected { session_id: String },
    Disconnected { reason: String },
    Reconnecting { attempt: u32 },
}

// ❌ BAD - Booleans create invalid combinations
pub struct WeakState {
    pub connected: bool,
    pub disconnected: bool,     // Both could be true!
    pub session_id: Option<String>, // Could be Some when disconnected!
}
```

**Type safety checklist:**

- Use enums for finite states
- Use newtypes for domain-specific values
- Use Result<T, E> for operations that can fail
- Use Option<T> only when absence is meaningful
- Avoid stringly-typed data

### 5.2 Message Evolution Strategies

**Strategy 1: Versioned Messages**

```rust
#[derive(Debug, Clone)]
pub enum ConfigMsg {
    V1(ConfigV1),
    V2(ConfigV2),
}

#[derive(Debug, Clone)]
pub struct ConfigV1 {
    pub timeout: Duration,
}

#[derive(Debug, Clone)]
pub struct ConfigV2 {
    pub timeout: Duration,
    pub max_retries: u32,  // New field
}

impl Message for ConfigMsg {
    type Result = ();
}

// Handler supports both versions
#[async_trait::async_trait]
impl Actor for ConfigurableActor {
    type Message = ConfigMsg;

    async fn handle(&mut self, msg: Self::Message, _ctx: &ActorContext) -> Result<(), ActorError> {
        match msg {
            ConfigMsg::V1(v1) => {
                // Handle old version
                self.timeout = v1.timeout;
                self.max_retries = 3; // Default
            }
            ConfigMsg::V2(v2) => {
                // Handle new version
                self.timeout = v2.timeout;
                self.max_retries = v2.max_retries;
            }
        }
        Ok(())
    }
}
```

**Strategy 2: Non-Breaking Additions**

```rust
#[derive(Debug, Clone)]
pub struct ExtensibleConfig {
    // Required fields (never remove these)
    pub timeout: Duration,
    
    // Optional new fields (non-breaking)
    pub max_retries: Option<u32>,
    pub endpoint: Option<String>,
}

impl ExtensibleConfig {
    pub fn new(timeout: Duration) -> Self {
        Self {
            timeout,
            max_retries: None,
            endpoint: None,
        }
    }
    
    pub fn with_retries(mut self, retries: u32) -> Self {
        self.max_retries = Some(retries);
        self
    }
}
```

**Strategy 3: Builder Pattern for Complex Messages**

```rust
pub struct ComplexMsgBuilder {
    required_field: Option<String>,
    optional_field1: Option<u32>,
    optional_field2: Option<Duration>,
}

impl ComplexMsgBuilder {
    pub fn new(required: String) -> Self {
        Self {
            required_field: Some(required),
            optional_field1: None,
            optional_field2: None,
        }
    }
    
    pub fn with_field1(mut self, value: u32) -> Self {
        self.optional_field1 = Some(value);
        self
    }
    
    pub fn with_field2(mut self, value: Duration) -> Self {
        self.optional_field2 = Some(value);
        self
    }
    
    pub fn build(self) -> ComplexMsg {
        ComplexMsg {
            required: self.required_field.expect("required field missing"),
            optional1: self.optional_field1.unwrap_or(10),
            optional2: self.optional_field2.unwrap_or(Duration::from_secs(30)),
        }
    }
}

#[derive(Debug, Clone)]
pub struct ComplexMsg {
    required: String,
    optional1: u32,
    optional2: Duration,
}

impl Message for ComplexMsg {
    type Result = ();
}

// Usage
let msg = ComplexMsgBuilder::new("value".to_string())
    .with_field1(42)
    .build();
```

### 5.3 Backward Compatibility Patterns

**Pattern 1: Default Values**
```rust
#[derive(Debug, Clone)]
pub struct ConfigV2 {
    pub timeout: Duration,
    
    #[serde(default = "default_max_retries")]
    pub max_retries: u32,
}

fn default_max_retries() -> u32 {
    3
}
```

**Pattern 2: Conversion Traits**
```rust
impl From<ConfigV1> for ConfigV2 {
    fn from(v1: ConfigV1) -> Self {
        Self {
            timeout: v1.timeout,
            max_retries: 3, // Sensible default
        }
    }
}

// Upgrade messages automatically
let v1_msg = ConfigV1 { timeout: Duration::from_secs(10) };
let v2_msg: ConfigV2 = v1_msg.into();
```

**Pattern 3: Feature Flags**
```rust
#[derive(Debug, Clone)]
pub struct FeatureConfig {
    pub timeout: Duration,
    
    #[cfg(feature = "advanced")]
    pub circuit_breaker: CircuitBreakerConfig,
    
    #[cfg(feature = "advanced")]
    pub retry_policy: RetryPolicy,
}
```

---

## Summary

**Message Design Principles:**

- Keep messages small (<64 bytes ideal)
- Use immutable data structures
- Share large data with `Arc<T>`
- Choose enum vs separate types based on complexity

**Communication Patterns:**

- Fire-and-forget: One-way commands, logging
- Request/reply: Queries, RPC-style interactions
- Pub/Sub: Event broadcasting, decoupling
- Broadcast: Multiple recipients, same message
- Scatter/gather: Parallel queries, aggregation

**Performance Optimization:**

- Use `Arc<T>` for zero-copy sharing (100x faster for large data)
- Batch messages for high throughput (10x improvement)
- Implement priority queues for critical messages
- Pool objects only after profiling confirms bottleneck
- Reference BENCHMARKING.md §6.2 for performance targets

**Error Handling:**

- Handle send failures with retry and backoff
- Always use timeouts for request/reply
- Log dead letters for debugging
- Implement circuit breakers for failing actors

**Type Safety:**

- Use strong types to prevent runtime errors
- Version messages for backward compatibility
- Use builder patterns for complex messages
- Leverage Rust's type system for correctness

**Performance Reference (BENCHMARKING.md §6.2):**

- Small message send: 50-100ns
- Arc clone vs copy: 10ns vs 1ms (100x)
- Single actor throughput: 1M+ msg/sec
- Batched throughput: 5M+ msg/sec

**Next Steps:**

- See [message_patterns.rs](../../examples/message_patterns.rs) for working examples
- Review [Actor Development](actor-development.md) for message handling patterns
- Check [Supervisor Patterns](supervisor-patterns.md) for error recovery integration
- Consult BENCHMARKING.md for performance tuning guidance

---

**Related Documentation:**

- [Getting Started](../implementation/getting-started.md) - Basic message passing
- [Actor Development](actor-development.md) - Message handling implementation
- [Supervisor Patterns](supervisor-patterns.md) - Error recovery with messages
- BENCHMARKING.md §6.2 - Message passing performance data
- [MessageBroker API](../reference/broker.md) - Pub/sub implementation details
