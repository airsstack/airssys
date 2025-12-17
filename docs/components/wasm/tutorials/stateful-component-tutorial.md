# Building a Stateful Component (Tutorial)

**Category:** Tutorial (Learning-Oriented)  
**Purpose:** Step-by-step guide to building a ComponentActor with persistent state.  
**Estimated Time:** 1.5 hours

## Introduction

In this tutorial, we will build a `CounterComponent` that maintains state across its lifecycle. You will learn:

- How to define component state structures
- How to initialize state with Arc<RwLock<T>>
- How to access state in lifecycle hooks
- How to handle messages that mutate state
- How to test concurrent state access

By the end of this tutorial, you'll have a working stateful component that can be deployed in production.

## Prerequisites

- Completed [Your First ComponentActor](./your-first-component-actor.md) tutorial
- Basic understanding of Rust async/await
- Familiarity with Arc and RwLock concepts

## Project Setup

Create a new example in your project:

```bash
touch examples/stateful_counter.rs
```

Add dependencies to `Cargo.toml` (if not already present):

```toml
[dependencies]
airssys-wasm = { path = "airssys-wasm" }
airssys-rt = { path = "airssys-rt" }
tokio = { version = "1.47", features = ["full"] }
chrono = { version = "0.4", features = ["serde"] }
```

## Step 1: Define State Structure

First, define the state your component will maintain.

```rust
// examples/stateful_counter.rs

use std::sync::Arc;

use chrono::{DateTime, Utc};
use tokio::sync::RwLock;

use airssys_wasm::core::ComponentId;

/// State for our counter component.
struct CounterState {
    /// Current counter value
    count: u64,
    /// Total number of operations
    operations: u64,
    /// Last update timestamp
    last_update: DateTime<Utc>,
}

impl CounterState {
    fn new() -> Self {
        Self {
            count: 0,
            operations: 0,
            last_update: Utc::now(),
        }
    }
}
```

**Why separate struct?**
- Clear separation between component and state
- Easy to serialize/deserialize for persistence
- Testable independently

## Step 2: Define Component with Arc<RwLock<State>>

Now create the component struct that wraps the state.

```rust
use airssys_rt::actor::Child;

/// Counter component with persistent state.
#[derive(Clone)]
struct CounterComponent {
    id: ComponentId,
    state: Arc<RwLock<CounterState>>,
}

impl CounterComponent {
    fn new(id: ComponentId) -> Self {
        Self {
            id,
            state: Arc::new(RwLock::new(CounterState::new())),
        }
    }
}
```

**Why Clone derive?**
- ComponentActor must be Clone for lifecycle hooks
- Arc makes cloning cheap (just increments reference count)

## Step 3: Implement Lifecycle Hooks with State Access

Implement the `Child` trait with lifecycle hooks that access state.

```rust
impl Child for CounterComponent {
    fn pre_start(&mut self) {
        println!("[{}] pre_start: Initializing counter", self.id.as_str());
        // State already initialized in new(), but we could load from disk here
    }

    fn post_start(&mut self) {
        println!("[{}] post_start: Counter ready", self.id.as_str());
    }

    fn pre_stop(&mut self) {
        // Access state during shutdown (synchronous access for demo)
        let state = self.state.blocking_read();
        println!(
            "[{}] pre_stop: Final count = {}, operations = {}",
            self.id.as_str(),
            state.count,
            state.operations
        );
    }

    fn post_stop(&mut self) {
        println!("[{}] post_stop: Counter stopped", self.id.as_str());
    }
}
```

**Note:** In `pre_stop`, we use `blocking_read()` because lifecycle hooks are synchronous. For async contexts, use `state.read().await`.

## Step 4: Define Messages

Define messages that will manipulate the state.

```rust
#[derive(Debug, Clone)]
enum CounterMessage {
    Increment,
    Decrement,
    Add(u64),
    GetCount(tokio::sync::oneshot::Sender<u64>),
    GetStats(tokio::sync::oneshot::Sender<CounterStats>),
}

#[derive(Debug, Clone)]
struct CounterStats {
    count: u64,
    operations: u64,
    last_update: DateTime<Utc>,
}
```

**Message Design:**

- `Increment/Decrement`: Simple mutations (no reply needed)
- `Add`: Parameterized mutation
- `GetCount/GetStats`: Queries with reply channel

## Step 5: Implement Message Handler with State Mutations

Implement the `Actor` trait to handle messages.

```rust
use airssys_rt::actor::{Actor, Context, Message};
use airssys_rt::error::ActorError;
use async_trait::async_trait;

impl Message for CounterMessage {
    type Response = ();
}

#[async_trait]
impl Actor for CounterComponent {
    type Message = CounterMessage;

    async fn handle_message(
        &mut self,
        message: Self::Message,
        _context: &Context,
    ) -> Result<(), ActorError> {
        match message {
            CounterMessage::Increment => {
                // Acquire write lock
                let mut state = self.state.write().await;
                state.count += 1;
                state.operations += 1;
                state.last_update = Utc::now();
                println!("[{}] Incremented to {}", self.id.as_str(), state.count);
            }
            CounterMessage::Decrement => {
                let mut state = self.state.write().await;
                state.count = state.count.saturating_sub(1);
                state.operations += 1;
                state.last_update = Utc::now();
                println!("[{}] Decremented to {}", self.id.as_str(), state.count);
            }
            CounterMessage::Add(value) => {
                let mut state = self.state.write().await;
                state.count += value;
                state.operations += 1;
                state.last_update = Utc::now();
                println!(
                    "[{}] Added {}, now {}",
                    self.id.as_str(),
                    value,
                    state.count
                );
            }
            CounterMessage::GetCount(reply) => {
                // Acquire read lock (allows concurrent reads)
                let state = self.state.read().await;
                reply.send(state.count).ok();
            }
            CounterMessage::GetStats(reply) => {
                let state = self.state.read().await;
                let stats = CounterStats {
                    count: state.count,
                    operations: state.operations,
                    last_update: state.last_update,
                };
                reply.send(stats).ok();
            }
        }
        Ok(())
    }
}
```

**Key Points:**

- **Write locks** for mutations (exclusive)
- **Read locks** for queries (concurrent)
- **Drop locks** immediately after use

## Step 6: Create Main Function

Create the main function to spawn and test the component.

```rust
use airssys_rt::system::ActorSystem;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("=== Stateful Counter Component Demo ===\n");

    // Create actor system
    let system = ActorSystem::new("counter-system");

    // Create counter component
    let component_id = ComponentId::new("counter-1");
    let component = CounterComponent::new(component_id.clone());

    // Spawn component
    println!("Spawning counter component...");
    let actor_ref = system
        .spawn_actor("counter-actor", component)
        .await?;

    println!("Counter component spawned\n");

    // Send mutations
    println!("--- Sending Mutations ---");
    actor_ref.send(CounterMessage::Increment).await?;
    tokio::time::sleep(tokio::time::Duration::from_millis(10)).await;

    actor_ref.send(CounterMessage::Increment).await?;
    tokio::time::sleep(tokio::time::Duration::from_millis(10)).await;

    actor_ref.send(CounterMessage::Add(5)).await?;
    tokio::time::sleep(tokio::time::Duration::from_millis(10)).await;

    actor_ref.send(CounterMessage::Decrement).await?;
    tokio::time::sleep(tokio::time::Duration::from_millis(10)).await;

    // Query state
    println!("\n--- Querying State ---");
    let (tx, rx) = tokio::sync::oneshot::channel();
    actor_ref.send(CounterMessage::GetCount(tx)).await?;
    let count = rx.await?;
    println!("Current count: {}", count);

    let (tx, rx) = tokio::sync::oneshot::channel();
    actor_ref.send(CounterMessage::GetStats(tx)).await?;
    let stats = rx.await?;
    println!("Stats: {:?}", stats);

    // Stop component
    println!("\n--- Stopping Component ---");
    system.stop_actor(&actor_ref).await?;

    println!("\n=== Demo Complete ===");
    Ok(())
}
```

## Step 7: Run and Verify

Run the example:

```bash
cargo run --example stateful_counter
```

**Expected Output:**

```
=== Stateful Counter Component Demo ===

Spawning counter component...
[counter-1] pre_start: Initializing counter
[counter-1] post_start: Counter ready
Counter component spawned

--- Sending Mutations ---
[counter-1] Incremented to 1
[counter-1] Incremented to 2
[counter-1] Added 5, now 7
[counter-1] Decremented to 6

--- Querying State ---
Current count: 6
Stats: CounterStats { count: 6, operations: 4, last_update: 2025-12-16T... }

--- Stopping Component ---
[counter-1] pre_stop: Final count = 6, operations = 4
[counter-1] post_stop: Counter stopped

=== Demo Complete ===
```

**Verification:**

- ✅ Lifecycle hooks execute (pre_start → post_start → pre_stop → post_stop)
- ✅ State persists across operations (count = 6)
- ✅ Operations are tracked (operations = 4)
- ✅ Timestamps are updated

## Step 8: Test Concurrent State Access

Add concurrent access test to verify thread safety.

```rust
async fn test_concurrent_access(
    actor_ref: &airssys_rt::util::ActorAddress,
) -> Result<(), Box<dyn std::error::Error>> {
    println!("\n--- Testing Concurrent Access ---");

    // Spawn 10 concurrent increment tasks
    let mut handles = Vec::new();
    for i in 0..10 {
        let actor_ref_clone = actor_ref.clone();
        let handle = tokio::spawn(async move {
            for _ in 0..10 {
                actor_ref_clone.send(CounterMessage::Increment).await.ok();
            }
            println!("Task {} completed 10 increments", i);
        });
        handles.push(handle);
    }

    // Wait for all tasks
    for handle in handles {
        handle.await?;
    }

    // Verify final count
    let (tx, rx) = tokio::sync::oneshot::channel();
    actor_ref.send(CounterMessage::GetCount(tx)).await?;
    let final_count = rx.await?;

    println!("Final count after concurrent access: {}", final_count);
    assert_eq!(final_count, 100, "Expected 100 increments");

    Ok(())
}

// Call from main():
// test_concurrent_access(&actor_ref).await?;
```

**Run again:**

```bash
cargo run --example stateful_counter
```

**Expected Concurrent Output:**

```
--- Testing Concurrent Access ---
Task 0 completed 10 increments
Task 3 completed 10 increments
...
Task 9 completed 10 increments
Final count after concurrent access: 100
```

**What This Proves:**

- ✅ RwLock prevents race conditions
- ✅ All increments are applied correctly
- ✅ No data loss under concurrency

## Common Mistakes and Solutions

### Mistake 1: Holding Lock Across Await

**Wrong:**
```rust
let mut state = self.state.write().await;
tokio::time::sleep(Duration::from_secs(1)).await; // Lock held!
state.count += 1;
```

**Correct:**
```rust
tokio::time::sleep(Duration::from_secs(1)).await; // Do work first
let mut state = self.state.write().await;
state.count += 1; // Lock held briefly
```

### Mistake 2: Using Blocking in Async Context

**Wrong:**
```rust
async fn handle_message(&mut self, msg: Message) {
    let state = self.state.blocking_read(); // Blocks async runtime!
}
```

**Correct:**
```rust
async fn handle_message(&mut self, msg: Message) {
    let state = self.state.read().await; // Async read
}
```

### Mistake 3: Not Dropping Locks

**Wrong:**
```rust
let state = self.state.read().await;
// ... long computation ...
// state still borrowed
self.other_operation().await; // May deadlock if it needs lock
```

**Correct:**
```rust
let value = {
    let state = self.state.read().await;
    state.count // Extract value
}; // Lock dropped here

self.other_operation().await; // Safe
```

## Extension: Integration with Request-Response

Extend the counter to support request-response patterns.

```rust
use airssys_wasm::actor::message::{RequestMessage, ResponseMessage};

// In handle_message:
CounterMessage::Request(request) => {
    // Parse request
    let value = u64::from_le_bytes(request.payload[..8].try_into()?);
    
    // Apply operation
    let mut state = self.state.write().await;
    state.count += value;
    let new_count = state.count;
    drop(state); // Release lock
    
    // Send response
    let response = ResponseMessage::success(
        request.correlation_id,
        self.id.clone(),
        request.from.clone(),
        new_count.to_le_bytes().to_vec(),
    );
    
    // Route response back (via MessageRouter)
    router.send_message(&request.from, ComponentMessage::Response(response)).await?;
}
```

See [Request-Response Pattern Guide](../guides/request-response-pattern.md) for complete implementation.

## Summary

You've successfully built a stateful ComponentActor! You learned:

- ✅ How to define state structures
- ✅ How to wrap state in Arc<RwLock<T>>
- ✅ How to access state in lifecycle hooks
- ✅ How to mutate state in message handlers
- ✅ How to test concurrent state access

## Next Steps

- **Production Deployment:** See [Production Deployment Guide](../guides/production-deployment.md)
- **Advanced Patterns:** Explore [State Management Patterns](../explanation/state-management-patterns.md)
- **Request-Response:** Integrate with [Request-Response Pattern](../guides/request-response-pattern.md)
- **Pub-Sub:** Learn [Pub-Sub Broadcasting](../guides/pubsub-broadcasting.md)

## References

- **API Reference:** [ComponentActor API](../api/component-actor.md)
- **Explanation:** [State Management Patterns](../explanation/state-management-patterns.md)
- **Example Code:** `examples/stateful_component.rs` (Checkpoint 1 example)
- **Performance:** Task 6.2 benchmarks show 37-39ns read/write access
