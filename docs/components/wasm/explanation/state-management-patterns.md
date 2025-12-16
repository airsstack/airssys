# State Management Patterns

**Category:** Explanation (Understanding-Oriented)  
**Purpose:** Explains why `Arc<RwLock<T>>` pattern is used for ComponentActor state management and when to use alternatives.

## Overview

State management is a critical concern in concurrent component systems. Components need to maintain state that can be accessed safely from multiple threads while minimizing lock contention and maintaining correctness.

This document explains the design rationale behind the `Arc<RwLock<T>>` pattern used in ComponentActor and explores alternative approaches with their tradeoffs.

## The Problem: Shared Mutable State

ComponentActors face several state management challenges:

1. **Concurrency:** Multiple actors may access the same component's state
2. **Lifecycle:** State must persist across actor restarts and lifecycle hooks
3. **Performance:** State access should not become a bottleneck
4. **Safety:** Rust's ownership rules must be satisfied

### Example Scenario

```rust
struct TemperatureSensor {
    current_temp: f64,
    readings: Vec<f64>,
    last_update: DateTime<Utc>,
}

// Multiple questions arise:
// - How do lifecycle hooks access state?
// - How do message handlers mutate state?
// - How do we share state across actor instances?
// - How do we prevent data races?
```

## Why Arc<RwLock<T>>?

The `Arc<RwLock<T>>` pattern is the default approach in ComponentActor because it provides:

1. **Shared Ownership:** `Arc` allows multiple owners (component, lifecycle, handlers)
2. **Interior Mutability:** `RwLock` enables mutation through shared references
3. **Read Optimization:** Multiple concurrent readers with single writer
4. **Rust Safety:** Compile-time guarantees against data races

### Pattern Implementation

```rust
use std::sync::Arc;

use tokio::sync::RwLock;
use chrono::{DateTime, Utc};

#[derive(Clone)]
struct TemperatureSensor {
    state: Arc<RwLock<SensorState>>,
}

struct SensorState {
    current_temp: f64,
    readings: Vec<f64>,
    last_update: DateTime<Utc>,
}

impl TemperatureSensor {
    fn new() -> Self {
        Self {
            state: Arc::new(RwLock::new(SensorState {
                current_temp: 0.0,
                readings: Vec::new(),
                last_update: Utc::now(),
            })),
        }
    }
    
    // Read access (multiple concurrent readers)
    async fn get_temperature(&self) -> f64 {
        let state = self.state.read().await;
        state.current_temp
    }
    
    // Write access (exclusive lock)
    async fn update_temperature(&self, temp: f64) {
        let mut state = self.state.write().await;
        state.current_temp = temp;
        state.readings.push(temp);
        state.last_update = Utc::now();
    }
}
```

### Why Arc?

`Arc` (Atomic Reference Counting) provides shared ownership:

- **Cloneable:** Component can be cloned for lifecycle hooks, handlers, etc.
- **Thread-safe:** Reference count is atomic, safe across threads
- **No GC:** Deterministic cleanup when last reference is dropped

**Alternative Considered:** `Rc` (Reference Counted)
- ❌ Not thread-safe
- ❌ Cannot be sent across threads
- ❌ Unsuitable for actor systems

### Why RwLock?

`RwLock` (Read-Write Lock) enables safe concurrent access:

- **Multiple readers:** Many threads can read simultaneously
- **Exclusive writer:** Only one thread can write at a time
- **Fairness:** Writers are prioritized to prevent starvation (in `tokio::sync::RwLock`)

**Alternative Considered:** `Mutex`
- ❌ Exclusive lock for both reads and writes
- ❌ Higher contention in read-heavy workloads
- ✅ Simpler mental model (always exclusive)

**When to use Mutex instead:**
- State is small and access is infrequent
- Writes are as frequent as reads
- Simpler code is preferred over performance

## Performance Characteristics

### Read Performance

**Scenario:** 10 concurrent readers

```rust
// RwLock: All 10 readers proceed in parallel
for _ in 0..10 {
    tokio::spawn(async {
        let state = component.state.read().await;
        // Read operations...
    });
}
```

**Latency:** 37-39ns per read (measured in Task 6.2 `actor_lifecycle_benchmarks.rs` benchmark `bench_state_read_access`)

### Write Performance

**Scenario:** 1 writer, blocking readers

```rust
// Only one writer at a time
let mut state = component.state.write().await;
state.value = 42;
// All readers blocked until write lock is released
```

**Latency:** 39ns per write (measured in Task 6.2 `actor_lifecycle_benchmarks.rs` benchmark `bench_state_write_access`)

**Key Insight:** Read and write latencies are similar (~37-39ns) because there's no contention in single-threaded benchmarks. In production with high concurrency, reads remain fast while writes may queue.

### Lock Contention

Lock contention occurs when multiple threads compete for the lock:

**Low Contention (Read-heavy):**
- 90% reads, 10% writes
- RwLock excels: readers don't block each other

**High Contention (Write-heavy):**
- 50% reads, 50% writes
- RwLock and Mutex perform similarly
- Consider alternative patterns (see below)

## Alternative Patterns

### 1. Actor-Internal State (No Sharing)

**Approach:** Keep state private to actor, no Arc/RwLock

```rust
struct PrivateStateComponent {
    // State owned by actor
    value: u32,
    data: Vec<u8>,
}

impl Actor for PrivateStateComponent {
    async fn handle_message(&mut self, msg: Message) {
        // Direct mutable access (no locks)
        self.value += 1;
        self.data.push(msg.data);
    }
}
```

**Pros:**
- ✅ Zero lock overhead
- ✅ Simplest mental model
- ✅ Fastest access (direct field access)

**Cons:**
- ❌ State not accessible outside message handler
- ❌ Cannot share state across lifecycle hooks
- ❌ Breaks ComponentActor pattern (lifecycle hooks need state)

**When to Use:**
- State is only accessed in message handlers
- No lifecycle hooks need state access
- Performance is critical

### 2. Message-Based State Updates

**Approach:** Send messages to update state instead of direct access

```rust
enum StateMessage {
    GetValue(oneshot::Sender<u32>),
    SetValue(u32),
}

impl Actor for MessageBasedComponent {
    async fn handle_message(&mut self, msg: StateMessage) {
        match msg {
            StateMessage::GetValue(reply) => {
                reply.send(self.value).ok();
            }
            StateMessage::SetValue(v) => {
                self.value = v;
            }
        }
    }
}

// Usage
let (tx, rx) = oneshot::channel();
actor_ref.send(StateMessage::GetValue(tx)).await?;
let value = rx.await?;
```

**Pros:**
- ✅ No shared locks
- ✅ Actor serializes all access (no races)
- ✅ Explicit state access (clear intent)

**Cons:**
- ❌ Higher latency (message passing overhead)
- ❌ More complex code (request-response for reads)
- ❌ Doesn't work well with lifecycle hooks

**When to Use:**
- Strong guarantees of serialized access are needed
- Message passing overhead is acceptable
- State access is infrequent

### 3. Channel-Based State Access

**Approach:** Use channels for state access from outside actor

```rust
struct ChannelComponent {
    state_tx: mpsc::Sender<StateQuery>,
}

enum StateQuery {
    Read(oneshot::Sender<StateSnapshot>),
    Write(StateUpdate),
}

// Background task manages state
async fn state_manager(mut rx: mpsc::Receiver<StateQuery>) {
    let mut state = State::default();
    
    while let Some(query) = rx.recv().await {
        match query {
            StateQuery::Read(reply) => {
                reply.send(state.snapshot()).ok();
            }
            StateQuery::Write(update) => {
                state.apply(update);
            }
        }
    }
}
```

**Pros:**
- ✅ Serialized access (no locks)
- ✅ Decoupled from actor lifecycle
- ✅ Can use different concurrency model

**Cons:**
- ❌ Highest complexity (requires background task)
- ❌ Message passing overhead for all access
- ❌ Harder to debug

**When to Use:**
- State management is complex (needs dedicated task)
- State access patterns don't fit actor model
- Advanced use cases only

### 4. Atomic Types (Lock-Free)

**Approach:** Use atomic types for primitive state

```rust
use std::sync::Arc;
use std::sync::atomic::{AtomicU64, Ordering};

struct AtomicComponent {
    counter: Arc<AtomicU64>,
}

impl AtomicComponent {
    fn increment(&self) {
        self.counter.fetch_add(1, Ordering::Relaxed);
    }
    
    fn get(&self) -> u64 {
        self.counter.load(Ordering::Relaxed)
    }
}
```

**Pros:**
- ✅ Lock-free (best performance)
- ✅ No deadlocks
- ✅ Minimal overhead

**Cons:**
- ❌ Limited to primitive types (u32, u64, bool, etc.)
- ❌ Complex memory ordering semantics
- ❌ Cannot represent complex state

**When to Use:**
- State is a single primitive value
- Extremely high performance required
- Simple increment/decrement operations

## Tradeoffs Summary

| Pattern | Performance | Complexity | Flexibility | Lifecycle Support |
|---------|-------------|------------|-------------|-------------------|
| Arc<RwLock<T>> | ⭐⭐⭐⭐ | ⭐⭐⭐ | ⭐⭐⭐⭐⭐ | ⭐⭐⭐⭐⭐ |
| Actor-Internal | ⭐⭐⭐⭐⭐ | ⭐⭐⭐⭐⭐ | ⭐⭐ | ⭐ |
| Message-Based | ⭐⭐⭐ | ⭐⭐ | ⭐⭐⭐⭐ | ⭐⭐ |
| Channel-Based | ⭐⭐ | ⭐ | ⭐⭐⭐⭐⭐ | ⭐⭐⭐ |
| Atomic Types | ⭐⭐⭐⭐⭐ | ⭐⭐⭐ | ⭐ | ⭐⭐⭐⭐⭐ |

**Legend:** ⭐⭐⭐⭐⭐ = Excellent, ⭐ = Poor

## Best Practices for Arc<RwLock<T>>

### 1. Minimize Lock Duration

**Bad:** Hold lock across async calls

```rust
// ❌ Don't do this
let mut state = self.state.write().await;
state.value = fetch_from_network().await; // Lock held during I/O!
```

**Good:** Release lock before async operations

```rust
// ✅ Do this
let new_value = fetch_from_network().await;
let mut state = self.state.write().await;
state.value = new_value; // Lock held briefly
```

### 2. Prefer Read Locks

Use read locks when possible to allow concurrency:

```rust
// Read lock for queries
async fn get_status(&self) -> Status {
    let state = self.state.read().await;
    state.status.clone()
}

// Write lock only when mutating
async fn update_status(&self, status: Status) {
    let mut state = self.state.write().await;
    state.status = status;
}
```

### 3. Avoid Nested Locks

**Deadlock Risk:** Nested locks can deadlock

```rust
// ❌ Deadlock risk
let state1 = component1.state.write().await;
let state2 = component2.state.write().await; // If another thread locks in opposite order
```

**Solution:** Lock in consistent order or use try_lock

```rust
// ✅ Consistent order
let (state1, state2) = if component1.id < component2.id {
    (component1.state.write().await, component2.state.write().await)
} else {
    (component2.state.write().await, component1.state.write().await)
};
```

### 4. Clone State for Async Operations

Clone state if you need to hold data across await points:

```rust
// ✅ Clone state before long operation
let current_state = {
    let state = self.state.read().await;
    state.clone() // Clone inside lock scope
}; // Lock released

// Use cloned state in async operations
process_data(current_state).await;
```

### 5. Use Timeout on Lock Acquisition

Prevent indefinite blocking with timeouts:

```rust
use tokio::time::{timeout, Duration};

// Try to acquire lock with timeout
match timeout(Duration::from_secs(1), self.state.write()).await {
    Ok(state) => {
        // Lock acquired
    }
    Err(_) => {
        // Timeout: lock contention too high
        return Err("Lock acquisition timeout");
    }
}
```

## Anti-Patterns to Avoid

### 1. Long-Held Locks

**Problem:** Holding locks across async operations blocks all other accessors

**Example:**
```rust
// ❌ ANTI-PATTERN
let mut state = self.state.write().await;
for item in items {
    state.process(item); // Long operation
    tokio::time::sleep(Duration::from_millis(10)).await; // Lock held!
}
```

**Solution:** Batch operations or release lock between iterations

### 2. Nested Locks

**Problem:** Deadlocks when locks are acquired in inconsistent order

**Example:**
```rust
// ❌ ANTI-PATTERN
async fn transfer(from: &Component, to: &Component, amount: u32) {
    let mut from_state = from.state.write().await;
    let mut to_state = to.state.write().await; // Deadlock risk!
    from_state.balance -= amount;
    to_state.balance += amount;
}
```

**Solution:** Lock in consistent order (e.g., by component ID)

### 3. Ignoring Lock Contention

**Problem:** Not monitoring or addressing high lock contention

**Solution:** Monitor lock acquisition times and refactor if needed

```rust
// Monitor lock acquisition time
let start = Instant::now();
let state = self.state.write().await;
let elapsed = start.elapsed();

if elapsed > Duration::from_millis(10) {
    log::warn!("High lock contention: {}ms", elapsed.as_millis());
}
```

## When to Use vs When to Avoid Arc<RwLock<T>>

### Use Arc<RwLock<T>> When:

- ✅ State is accessed from multiple threads/actors
- ✅ Read operations are more frequent than writes (read-heavy)
- ✅ Lifecycle hooks need state access
- ✅ State structure is moderately complex (struct with multiple fields)
- ✅ Performance requirements are reasonable (<1µs access time acceptable)

### Avoid Arc<RwLock<T>> When:

- ❌ State is only accessed in message handler (use actor-internal state)
- ❌ State is a single primitive value (use atomic types)
- ❌ Write operations are as frequent as reads (consider message-based)
- ❌ Lock contention is observed (>10ms acquisition time)
- ❌ Deadlocks occur (refactor to eliminate nested locks)

## Conclusion

The `Arc<RwLock<T>>` pattern is the default choice for ComponentActor state management because it balances:

- **Performance:** 37-39ns read/write access (Task 6.2 benchmarks)
- **Safety:** Compile-time guarantees against data races
- **Flexibility:** Works with lifecycle hooks and complex state
- **Simplicity:** Well-understood pattern with good tooling support

Alternative patterns exist for specialized use cases, but for most ComponentActor implementations, `Arc<RwLock<T>>` provides the right balance of performance, safety, and maintainability.

## References

- **Task 6.2 Benchmarks:** `benches/actor_lifecycle_benchmarks.rs` (`bench_state_read_access`, `bench_state_write_access`)
- **Implementation:** `airssys-wasm/src/actor/component/component_actor.rs`
- **Rust Book:** [Shared-State Concurrency](https://doc.rust-lang.org/book/ch16-03-shared-state.html)
- **tokio::sync::RwLock:** [Documentation](https://docs.rs/tokio/latest/tokio/sync/struct.RwLock.html)
