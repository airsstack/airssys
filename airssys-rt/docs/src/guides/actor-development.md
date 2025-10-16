# Actor Development Tutorial

This comprehensive tutorial teaches you how to build robust, production-ready actors. You'll learn lifecycle management, state patterns, message design, error handling, and testing strategies.

**Prerequisites:**
- Completed [Getting Started](../implementation/getting-started.md)
- Understanding of async/await in Rust
- Basic familiarity with actor model concepts

**What You'll Learn:**
- Actor lifecycle in depth (pre_start, handle_message, post_stop)
- State management patterns (immutable, mutable, persistent)
- Message design patterns (commands, queries, events)
- Error handling strategies (recoverable, escalation, supervision)
- Testing actors (unit, integration, property-based)

---

## 1. Actor Lifecycle in Depth

Understanding the complete actor lifecycle is crucial for building reliable systems.

### Lifecycle States

Actors progress through distinct states:

```
Created → Starting → Running → Stopping → Stopped
                       ↓
                   Restarting (on error)
```

**State Transitions:**
- **Created**: Initial state after instantiation
- **Starting**: Executing `pre_start()`, initializing resources
- **Running**: Processing messages via `handle_message()`
- **Stopping**: Executing `post_stop()`, cleaning up resources
- **Stopped**: Terminal state, actor no longer active

### pre_start() - Resource Initialization

The `pre_start()` hook runs **once** before the actor begins processing messages.

**When to use:**
- Open file handles or database connections
- Initialize network connections
- Load configuration or state from disk
- Register with external services
- Allocate expensive resources

**Example:**
```rust
use airssys_rt::prelude::*;
use async_trait::async_trait;
use std::fs::File;
use std::io::{self, BufReader};

struct FileProcessor {
    file: Option<BufReader<File>>,
    path: String,
}

#[async_trait]
impl Actor for FileProcessor {
    type Message = ProcessCommand;
    type Error = io::Error;

    async fn pre_start<B: MessageBroker<Self::Message>>(
        &mut self,
        context: &mut ActorContext<Self::Message, B>,
    ) -> Result<(), Self::Error> {
        // Open file during initialization
        let file = File::open(&self.path)?;
        self.file = Some(BufReader::new(file));
        
        println!("[{}] File opened: {}", 
                 context.address().name().unwrap_or("unknown"), 
                 self.path);
        Ok(())
    }

    async fn handle_message<B: MessageBroker<Self::Message>>(
        &mut self,
        message: Self::Message,
        context: &mut ActorContext<Self::Message, B>,
    ) -> Result<(), Self::Error> {
        // File is guaranteed to be open here
        let file = self.file.as_mut().expect("File not initialized");
        // ... process file
        context.record_message();
        Ok(())
    }

    async fn post_stop<B: MessageBroker<Self::Message>>(
        &mut self,
        _context: &mut ActorContext<Self::Message, B>,
    ) -> Result<(), Self::Error> {
        // File automatically closed when dropped
        self.file = None;
        println!("File processor stopped");
        Ok(())
    }
}
```

**Best practices:**
- Return `Err` if initialization fails (prevents starting broken actor)
- Keep initialization fast (<100ms ideal)
- Log initialization steps for debugging
- Use `?` operator for early error returns

### handle_message() - Message Processing

The core of your actor's behavior. Messages are processed **sequentially** (no concurrent access).

**Patterns:**

#### 1. **State Mutations**
```rust
async fn handle_message<B: MessageBroker<Self::Message>>(
    &mut self,
    message: Self::Message,
    context: &mut ActorContext<Self::Message, B>,
) -> Result<(), Self::Error> {
    match message {
        CounterMsg::Increment => {
            self.count += 1;  // Safe - exclusive access
        }
        CounterMsg::Reset => {
            self.count = 0;
        }
    }
    context.record_message();
    Ok(())
}
```

#### 2. **Request/Reply Pattern**
```rust
use tokio::sync::oneshot;

#[derive(Clone)]
enum QueryMessage {
    GetCount(oneshot::Sender<u64>),
    GetStats(oneshot::Sender<Stats>),
}

async fn handle_message<B: MessageBroker<Self::Message>>(
    &mut self,
    message: Self::Message,
    context: &mut ActorContext<Self::Message, B>,
) -> Result<(), Self::Error> {
    match message {
        QueryMessage::GetCount(reply) => {
            // Send current count back to requester
            let _ = reply.send(self.count);
        }
        QueryMessage::GetStats(reply) => {
            let stats = Stats {
                count: self.count,
                messages: context.message_count(),
            };
            let _ = reply.send(stats);
        }
    }
    context.record_message();
    Ok(())
}
```

#### 3. **Side Effects**
```rust
async fn handle_message<B: MessageBroker<Self::Message>>(
    &mut self,
    message: Self::Message,
    context: &mut ActorContext<Self::Message, B>,
) -> Result<(), Self::Error> {
    match message {
        LogMessage::Write(entry) => {
            // Write to file (side effect)
            writeln!(self.file, "{}", entry)?;
            self.file.flush()?;
            
            // Update metrics
            self.entries_written += 1;
        }
    }
    context.record_message();
    Ok(())
}
```

**Performance Note:**
- Message processing baseline: ~31.5ns per message
- Keep processing fast for high throughput
- Offload expensive I/O to separate actors
- Use async operations for blocking calls

### post_stop() - Resource Cleanup

The `post_stop()` hook runs when the actor is shutting down.

**When to use:**
- Close file handles or connections
- Flush buffers to disk
- Deregister from external services
- Release locks or semaphores
- Save state for persistence

**Example:**
```rust
async fn post_stop<B: MessageBroker<Self::Message>>(
    &mut self,
    context: &mut ActorContext<Self::Message, B>,
) -> Result<(), Self::Error> {
    // Flush pending writes
    if let Some(ref mut file) = self.file {
        file.flush()?;
    }
    
    // Save state to disk
    let state = ActorState {
        count: self.count,
        timestamp: context.start_time(),
    };
    tokio::fs::write(&self.state_path, serde_json::to_string(&state)?).await?;
    
    println!("[{}] State persisted, shutting down", 
             context.address().name().unwrap_or("unknown"));
    Ok(())
}
```

**Best practices:**
- **Always** clean up resources (prevent leaks)
- Handle cleanup errors gracefully (log, don't panic)
- Keep shutdown fast (<1 second ideal)
- Persist critical state before shutdown

---

## 2. State Management Patterns

Choosing the right state management pattern affects correctness, performance, and maintainability.

### Pattern 1: Immutable State

**When to use:**
- State changes are infrequent
- State is small (< 1KB)
- Functional programming style preferred

**Example:**
```rust
#[derive(Clone, Debug)]
struct ImmutableState {
    count: u64,
    name: String,
    config: Config,
}

struct ImmutableActor {
    state: ImmutableState,
}

async fn handle_message<B: MessageBroker<Self::Message>>(
    &mut self,
    message: Self::Message,
    context: &mut ActorContext<Self::Message, B>,
) -> Result<(), Self::Error> {
    // Create new state instead of mutating
    self.state = ImmutableState {
        count: self.state.count + 1,
        ..self.state.clone()
    };
    context.record_message();
    Ok(())
}
```

**Pros:**
- Simple reasoning (no hidden mutations)
- Easy to test (pure functions)
- Works well with undo/redo patterns

**Cons:**
- Memory allocation on every change
- Slower for large state (cloning overhead)

### Pattern 2: Mutable State

**When to use:**
- Frequent state updates
- Large state objects (> 1KB)
- Performance-critical paths

**Example:**
```rust
struct MutableActor {
    count: u64,
    cache: HashMap<String, CachedValue>,
    buffer: Vec<u8>,
}

async fn handle_message<B: MessageBroker<Self::Message>>(
    &mut self,
    message: Self::Message,
    context: &mut ActorContext<Self::Message, B>,
) -> Result<(), Self::Error> {
    match message {
        Msg::Increment => {
            self.count += 1;  // Direct mutation - zero allocation
        }
        Msg::UpdateCache(key, value) => {
            self.cache.insert(key, value);  // HashMap mutation
        }
        Msg::Append(data) => {
            self.buffer.extend_from_slice(&data);  // Vec mutation
        }
    }
    context.record_message();
    Ok(())
}
```

**Pros:**
- Zero allocation for updates
- Best performance
- Natural for imperative style

**Cons:**
- Must track mutations carefully
- More complex testing

### Pattern 3: Interior Mutability (Advanced)

**When to use:**
- Shared state across async boundaries
- Complex borrowing scenarios
- Performance-critical with thread safety

**Example:**
```rust
use std::sync::Arc;
use tokio::sync::RwLock;

struct SharedActor {
    // Shared mutable state
    state: Arc<RwLock<SharedState>>,
}

async fn handle_message<B: MessageBroker<Self::Message>>(
    &mut self,
    message: Self::Message,
    context: &mut ActorContext<Self::Message, B>,
) -> Result<(), Self::Error> {
    match message {
        Msg::Read => {
            let state = self.state.read().await;
            println!("Count: {}", state.count);
        }
        Msg::Write(value) => {
            let mut state = self.state.write().await;
            state.count = value;
        }
    }
    context.record_message();
    Ok(())
}
```

**Pros:**
- Share state across async tasks
- Fine-grained locking
- Concurrent reads

**Cons:**
- Complexity (deadlock risk)
- Slower than direct mutation
- Requires careful lock management

**⚠️ Warning:** Use only when necessary. Most actors don't need interior mutability.

### Pattern 4: State Persistence

**When to use:**
- Actors must survive restarts
- Audit trail required
- Recovery from crashes needed

**Example:**
```rust
use serde::{Serialize, Deserialize};
use tokio::fs;

#[derive(Serialize, Deserialize, Clone)]
struct PersistentState {
    count: u64,
    last_updated: chrono::DateTime<chrono::Utc>,
}

struct PersistentActor {
    state: PersistentState,
    state_path: String,
}

impl PersistentActor {
    async fn save_state(&self) -> io::Result<()> {
        let json = serde_json::to_string(&self.state)?;
        fs::write(&self.state_path, json).await?;
        Ok(())
    }

    async fn load_state(path: &str) -> io::Result<PersistentState> {
        let json = fs::read_to_string(path).await?;
        Ok(serde_json::from_str(&json)?)
    }
}

async fn pre_start<B: MessageBroker<Self::Message>>(
    &mut self,
    _context: &mut ActorContext<Self::Message, B>,
) -> Result<(), Self::Error> {
    // Load state on startup
    if let Ok(state) = Self::load_state(&self.state_path).await {
        self.state = state;
        println!("State restored from disk");
    }
    Ok(())
}

async fn handle_message<B: MessageBroker<Self::Message>>(
    &mut self,
    message: Self::Message,
    context: &mut ActorContext<Self::Message, B>,
) -> Result<(), Self::Error> {
    // Update state
    self.state.count += 1;
    self.state.last_updated = chrono::Utc::now();
    
    // Persist every N messages
    if context.message_count() % 100 == 0 {
        self.save_state().await?;
    }
    
    context.record_message();
    Ok(())
}
```

**Best practices:**
- Batch writes (don't persist every message)
- Use write-ahead logging for durability
- Handle corruption gracefully
- Version your state format

---

## 3. Message Design Patterns

Well-designed messages improve clarity, type safety, and maintainability.

### Pattern 1: Command Messages

Commands instruct the actor to **do something**.

**Characteristics:**
- Imperative naming (Verb + Object)
- Fire-and-forget semantics
- May have side effects

**Example:**
```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
enum FileCommand {
    Write(String),           // Write data
    Flush,                   // Flush buffers
    Close,                   // Close file
    Rotate,                  // Rotate log file
}

impl Message for FileCommand {
    const MESSAGE_TYPE: &'static str = "file_command";
}
```

### Pattern 2: Query Messages

Queries request information **without** modifying state.

**Characteristics:**
- Request/reply pattern (oneshot channel)
- No side effects (read-only)
- Returns data to caller

**Example:**
```rust
use tokio::sync::oneshot;

#[derive(Clone)]
enum FileQuery {
    GetSize(oneshot::Sender<u64>),
    GetPath(oneshot::Sender<String>),
    GetStats(oneshot::Sender<FileStats>),
}

impl Message for FileQuery {
    const MESSAGE_TYPE: &'static str = "file_query";
}

// Usage:
async fn query_file_size(file_actor: &ActorAddress) -> Result<u64, Error> {
    let (tx, rx) = oneshot::channel();
    file_actor.send(FileQuery::GetSize(tx)).await?;
    let size = rx.await?;
    Ok(size)
}
```

### Pattern 3: Event Messages

Events notify observers that **something happened**.

**Characteristics:**
- Past tense naming (Subject + Past Verb)
- Pub/sub semantics (via MessageBroker)
- Immutable facts

**Example:**
```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
enum FileEvent {
    FileOpened { path: String, size: u64 },
    BytesWritten { count: u64 },
    FileClosed { final_size: u64 },
    ErrorOccurred { error: String },
}

impl Message for FileEvent {
    const MESSAGE_TYPE: &'static str = "file_event";
}
```

### Pattern 4: Message Versioning

Support evolution of message types over time.

**Example:**
```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "version")]
enum VersionedMessage {
    #[serde(rename = "v1")]
    V1(MessageV1),
    
    #[serde(rename = "v2")]
    V2(MessageV2),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct MessageV1 {
    data: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct MessageV2 {
    data: String,
    timestamp: chrono::DateTime<chrono::Utc>,  // New field
    priority: u8,                               // New field
}

async fn handle_message<B: MessageBroker<Self::Message>>(
    &mut self,
    message: VersionedMessage,
    context: &mut ActorContext<Self::Message, B>,
) -> Result<(), Self::Error> {
    match message {
        VersionedMessage::V1(msg) => {
            // Handle old format
            self.process_legacy(msg)?;
        }
        VersionedMessage::V2(msg) => {
            // Handle new format
            self.process_current(msg)?;
        }
    }
    context.record_message();
    Ok(())
}
```

**Best practices:**
- Use enums for message variants
- Derive `Clone` (messages are cloned when sent)
- Derive `Serialize` for routing/persistence
- Version messages for backward compatibility
- Keep messages small (< 1KB ideal)

---

## 4. Error Handling

Actors use supervision for fault tolerance. Understanding error handling is critical.

### Error Types

**Recoverable Errors:**
- Temporary failures (network timeout, file lock)
- Can be retried
- Don't indicate actor corruption

**Non-Recoverable Errors:**
- Logic errors (invalid state)
- Resource exhaustion
- Indicate actor needs restart

### ErrorAction Strategy

The `on_error()` hook determines supervision behavior:

```rust
#[async_trait]
impl Actor for MyActor {
    async fn on_error<B: MessageBroker<Self::Message>>(
        &mut self,
        error: Self::Error,
        context: &mut ActorContext<Self::Message, B>,
    ) -> ErrorAction {
        match error {
            MyError::Temporary(_) => {
                // Retry-able error - keep running
                ErrorAction::Resume
            }
            MyError::RateLimitExceeded => {
                // Need backoff, but don't restart
                ErrorAction::Resume
            }
            MyError::InvalidState => {
                // Corrupted state - restart to clean slate
                ErrorAction::Restart
            }
            MyError::Fatal(_) => {
                // Can't recover - let supervisor decide
                ErrorAction::Escalate
            }
        }
    }
}
```

**ErrorAction variants:**
- **Resume**: Continue processing (error handled)
- **Stop**: Graceful shutdown
- **Restart**: Reset state and restart
- **Escalate**: Let supervisor decide

### Retry Pattern

Implement exponential backoff for transient failures:

```rust
use tokio::time::{sleep, Duration};

struct RetryableActor {
    retry_count: u32,
    max_retries: u32,
}

async fn handle_message<B: MessageBroker<Self::Message>>(
    &mut self,
    message: Self::Message,
    context: &mut ActorContext<Self::Message, B>,
) -> Result<(), Self::Error> {
    // Try operation with retries
    let mut attempt = 0;
    loop {
        match self.try_operation(&message).await {
            Ok(result) => {
                self.retry_count = 0;  // Reset on success
                context.record_message();
                return Ok(());
            }
            Err(e) if attempt < self.max_retries => {
                attempt += 1;
                let delay = Duration::from_millis(100 * 2_u64.pow(attempt));
                sleep(delay).await;
                continue;
            }
            Err(e) => {
                // Exhausted retries
                return Err(e);
            }
        }
    }
}
```

### Circuit Breaker Pattern

Prevent cascading failures:

```rust
use std::time::Instant;

struct CircuitBreaker {
    state: BreakerState,
    failure_count: u32,
    threshold: u32,
    timeout: Duration,
    last_failure: Option<Instant>,
}

enum BreakerState {
    Closed,   // Normal operation
    Open,     // Failing, reject requests
    HalfOpen, // Testing if service recovered
}

impl CircuitBreaker {
    fn should_attempt(&mut self) -> bool {
        match self.state {
            BreakerState::Closed => true,
            BreakerState::Open => {
                // Check if timeout expired
                if let Some(last) = self.last_failure {
                    if last.elapsed() > self.timeout {
                        self.state = BreakerState::HalfOpen;
                        true
                    } else {
                        false
                    }
                } else {
                    false
                }
            }
            BreakerState::HalfOpen => true,
        }
    }

    fn on_success(&mut self) {
        self.failure_count = 0;
        self.state = BreakerState::Closed;
    }

    fn on_failure(&mut self) {
        self.failure_count += 1;
        self.last_failure = Some(Instant::now());
        
        if self.failure_count >= self.threshold {
            self.state = BreakerState::Open;
        }
    }
}
```

---

## 5. Testing Actors

Comprehensive testing ensures actor correctness and reliability.

### Unit Testing

Test actor logic in isolation:

```rust
#[cfg(test)]
mod tests {
    use super::*;
    
    #[tokio::test]
    async fn test_counter_increment() {
        // Arrange
        let mut actor = CounterActor { count: 0 };
        let address = ActorAddress::named("test-counter");
        let broker = InMemoryMessageBroker::new();
        let mut context = ActorContext::new(address, broker);
        
        // Act
        let result = actor.handle_message(
            CounterMsg::Increment, 
            &mut context
        ).await;
        
        // Assert
        assert!(result.is_ok());
        assert_eq!(actor.count, 1);
        assert_eq!(context.message_count(), 1);
    }
    
    #[tokio::test]
    async fn test_error_handling() {
        let mut actor = CounterActor { count: 100 };
        let address = ActorAddress::named("test-counter");
        let broker = InMemoryMessageBroker::new();
        let mut context = ActorContext::new(address, broker);
        
        // Test error condition
        let result = actor.handle_message(
            CounterMsg::SetValue(1000),  // Over limit
            &mut context
        ).await;
        
        assert!(result.is_err());
        
        // Test error action
        let action = actor.on_error(
            result.unwrap_err(), 
            &mut context
        ).await;
        assert_eq!(action, ErrorAction::Restart);
    }
}
```

### Integration Testing

Test actor interactions:

```rust
#[cfg(test)]
mod integration_tests {
    use super::*;
    use tokio::sync::oneshot;
    
    #[tokio::test]
    async fn test_request_reply() {
        // Setup
        let mut actor = QueryActor { count: 42 };
        let address = ActorAddress::named("query-actor");
        let broker = InMemoryMessageBroker::new();
        let mut context = ActorContext::new(address, broker);
        
        // Create query with reply channel
        let (tx, rx) = oneshot::channel();
        let msg = QueryMsg::GetCount(tx);
        
        // Send query
        actor.handle_message(msg, &mut context).await.unwrap();
        
        // Verify response
        let count = rx.await.unwrap();
        assert_eq!(count, 42);
    }
}
```

### Lifecycle Testing

Test complete actor lifecycle:

```rust
#[tokio::test]
async fn test_full_lifecycle() {
    let mut actor = FileActor::new("test.txt");
    let address = ActorAddress::named("file-actor");
    let broker = InMemoryMessageBroker::new();
    let mut context = ActorContext::new(address, broker);
    let mut lifecycle = ActorLifecycle::new();
    
    // Test pre_start
    actor.pre_start(&mut context).await.unwrap();
    lifecycle.transition_to(ActorState::Running);
    assert_eq!(lifecycle.state(), ActorState::Running);
    
    // Test message processing
    actor.handle_message(
        FileMsg::Write("test data".into()),
        &mut context
    ).await.unwrap();
    
    // Test post_stop
    lifecycle.transition_to(ActorState::Stopping);
    actor.post_stop(&mut context).await.unwrap();
    lifecycle.transition_to(ActorState::Stopped);
    assert_eq!(lifecycle.state(), ActorState::Stopped);
}
```

### Property-Based Testing

Test actor properties with `proptest`:

```rust
#[cfg(test)]
mod property_tests {
    use proptest::prelude::*;
    
    proptest! {
        #[test]
        fn counter_always_increases(increments in prop::collection::vec(1..100u64, 1..100)) {
            // Setup
            let rt = tokio::runtime::Runtime::new().unwrap();
            rt.block_on(async {
                let mut actor = CounterActor { count: 0 };
                let address = ActorAddress::named("prop-counter");
                let broker = InMemoryMessageBroker::new();
                let mut context = ActorContext::new(address, broker);
                
                let mut expected = 0u64;
                
                // Apply increments
                for inc in increments {
                    actor.handle_message(
                        CounterMsg::Add(inc),
                        &mut context
                    ).await.unwrap();
                    expected += inc;
                }
                
                // Property: count equals sum of increments
                assert_eq!(actor.count, expected);
            });
        }
    }
}
```

---

## Next Steps

Congratulations! You now understand actor development deeply. Continue your learning:

### 🎯 **Build Fault-Tolerant Systems**
- [Supervisor Patterns Guide](./supervisor-patterns.md) - Supervision trees and strategies
- [Error Recovery Patterns](./supervisor-patterns.md#error-recovery) - Production-ready fault tolerance

### 📨 **Master Message Patterns**
- [Message Passing Guide](./message-passing.md) - Communication patterns and optimization
- [Performance Optimization](../performance/optimization.md) - Tuning message throughput

### 📊 **Monitor Your Actors**
- [Monitoring Guide](./monitoring.md) - Observability and health checks
- [Debugging Actors](../reference/debugging.md) - Common issues and solutions

### 🏗️ **Architecture Patterns**
- [Actor Hierarchies](./supervisor-patterns.md#hierarchical-supervision) - Multi-level supervision
- [Event Sourcing](../architecture/event-sourcing.md) - State management patterns

---

## Summary

✅ **Lifecycle Management**: pre_start, handle_message, post_stop  
✅ **State Patterns**: Immutable, mutable, interior mutability, persistence  
✅ **Message Design**: Commands, queries, events, versioning  
✅ **Error Handling**: ErrorAction, retries, circuit breakers  
✅ **Testing**: Unit, integration, lifecycle, property-based  

You're now ready to build production-ready actor systems with AirsSys-RT!
