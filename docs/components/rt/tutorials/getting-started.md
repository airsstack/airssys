# Getting Started

Welcome! This guide will get you up and running with AirsSys-RT in under 20 minutes. You'll create your first actor, send messages, and understand the core workflow.

## What You'll Build

A simple counter actor that:
- Receives increment/decrement messages
- Maintains internal state
- Responds to queries
- Handles shutdown gracefully

**Prerequisites:**

- Rust 1.70 or higher
- Basic understanding of async/await in Rust
- Familiarity with Cargo

**Estimated time:** 15-20 minutes

---

## Step 1: Add AirsSys-RT to Your Project

Create a new Rust project and add the dependency:

```bash
cargo new my-actor-app
cd my-actor-app
```

Add to your `Cargo.toml`:

```toml
[dependencies]
airssys-rt = "0.1"
tokio = { version = "1.47", features = ["full"] }
async-trait = "0.1"
serde = { version = "1.0", features = ["derive"] }
```

**Why these dependencies?**
- `airssys-rt` - The actor runtime framework
- `tokio` - Async runtime for concurrent operations
- `async-trait` - Required for async trait methods
- `serde` - Message serialization (optional but recommended)

---

## Step 2: Define Your Messages

Messages are the data your actor receives. Create clear message types using enums:

```rust
use airssys_rt::prelude::*;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
enum CounterMessage {
    Increment,
    Decrement,
    GetValue,
    Shutdown,
}

impl Message for CounterMessage {
    const MESSAGE_TYPE: &'static str = "counter";
}
```

**Key concepts:**

- **Enums for clarity**: Each variant represents a distinct operation
- **Derive Clone**: Messages are cloned when sent
- **Derive Serialize**: Enables message routing and persistence
- **MESSAGE_TYPE constant**: Identifies message type in the system

---

## Step 3: Implement Your Actor

Actors encapsulate state and behavior. Here's a simple counter:

```rust
use async_trait::async_trait;
use std::fmt;

struct CounterActor {
    value: i32,
}

// Define error type
#[derive(Debug)]
struct CounterError(String);

impl fmt::Display for CounterError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Counter error: {}", self.0)
    }
}

impl std::error::Error for CounterError {}

#[async_trait]
impl Actor for CounterActor {
    type Message = CounterMessage;
    type Error = CounterError;

    async fn handle_message<B: MessageBroker<Self::Message>>(
        &mut self,
        message: Self::Message,
        context: &mut ActorContext<Self::Message, B>,
    ) -> Result<(), Self::Error> {
        match message {
            CounterMessage::Increment => {
                self.value += 1;
                println!("Counter incremented to: {}", self.value);
            }
            CounterMessage::Decrement => {
                self.value -= 1;
                println!("Counter decremented to: {}", self.value);
            }
            CounterMessage::GetValue => {
                println!("Current value: {}", self.value);
            }
            CounterMessage::Shutdown => {
                println!("Shutting down counter actor");
                return Err(CounterError("Shutdown requested".to_string()));
            }
        }
        
        // Record that we processed a message
        context.record_message();
        Ok(())
    }
}
```

**Understanding the Actor trait:**

- **Associated types**: Define your message and error types
- **handle_message**: Core message processing logic
- **context**: Provides actor metadata and messaging capabilities
- **Error handling**: Returning `Err` signals the supervisor

---

## Step 4: Create and Run Your Actor

Now bring it all together in your `main.rs`:

```rust
use airssys_rt::prelude::*;
use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use std::fmt;

// ... (include message and actor definitions from above)

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("Starting actor...\n");
    
    // Create actor instance
    let mut actor = CounterActor { value: 0 };

    // Create actor context with address and message broker
    let address = ActorAddress::named("counter");
    let broker = InMemoryMessageBroker::<CounterMessage>::new();
    let mut context = ActorContext::new(address, broker);

    // Create lifecycle tracker
    let mut lifecycle = ActorLifecycle::new();

    // Start the actor
    actor.pre_start(&mut context).await?;
    lifecycle.transition_to(ActorState::Running);

    // Process messages
    let messages = vec![
        CounterMessage::Increment,
        CounterMessage::Increment,
        CounterMessage::GetValue,
        CounterMessage::Decrement,
        CounterMessage::GetValue,
    ];

    for msg in messages {
        match actor.handle_message(msg, &mut context).await {
            Ok(()) => println!("✓ Message processed"),
            Err(e) => {
                println!("✗ Error: {e}");
                let action = actor.on_error(e, &mut context).await;
                if action == ErrorAction::Stop {
                    lifecycle.transition_to(ActorState::Stopping);
                    break;
                }
            }
        }
    }

    // Graceful shutdown
    lifecycle.transition_to(ActorState::Stopping);
    actor.post_stop(&mut context).await?;
    lifecycle.transition_to(ActorState::Stopped);

    println!("\nActor lifecycle complete!");
    Ok(())
}
```

---

## Step 5: Run Your Application

```bash
cargo run --example getting_started
```

**Expected output:**

```
=== Getting Started Example ===

1. Starting actor...
   Actor is running

2. Sending messages...
Counter incremented to: 1
   ✓ Message processed (total: 1)
Counter incremented to: 2
   ✓ Message processed (total: 2)
Current value: 2
   ✓ Message processed (total: 3)
Counter decremented to: 1
   ✓ Message processed (total: 4)
Current value: 1
   ✓ Message processed (total: 5)

3. Shutting down...
Shutting down counter actor

4. Final state:
   State: Stopped
   Messages processed: 5
   Restart count: 0

=== Example Complete ===
```

---

## Understanding the Workflow

Let's break down what just happened:

### 1. **Actor Creation**
```rust
let mut actor = CounterActor { value: 0 };
```
- Creates actor instance with initial state
- Actor owns its data (no shared state)
- Mutable reference allows state changes

### 2. **Context Setup**
```rust
let address = ActorAddress::named("counter");
let broker = InMemoryMessageBroker::<CounterMessage>::new();
let mut context = ActorContext::new(address, broker);
```
- **ActorAddress**: Unique identifier for the actor
- **MessageBroker**: Routes messages between actors (dependency injection per ADR-006)
- **ActorContext**: Provides execution environment and metadata

### 3. **Lifecycle Management**
```rust
let mut lifecycle = ActorLifecycle::new();
actor.pre_start(&mut context).await?;
lifecycle.transition_to(ActorState::Running);
```
- **pre_start()**: Initialize resources (open files, connect to services)
- State transition: `Created` → `Starting` → `Running`
- Lifecycle tracking for supervision

### 4. **Message Processing**
```rust
actor.handle_message(msg, &mut context).await?;
```
- Messages processed synchronously (one at a time)
- No race conditions - actor has exclusive access to its state
- **Performance**: ~31.5ns per message processing

### 5. **Error Handling**
```rust
let action = actor.on_error(e, &mut context).await;
```
- Actor decides supervision action
- **Resume**: Continue running
- **Stop**: Shut down gracefully
- **Restart**: Reset state and continue
- **Escalate**: Let supervisor decide

### 6. **Graceful Shutdown**
```rust
lifecycle.transition_to(ActorState::Stopping);
actor.post_stop(&mut context).await?;
lifecycle.transition_to(ActorState::Stopped);
```
- **post_stop()**: Clean up resources (close connections, flush buffers)
- State transition: `Running` → `Stopping` → `Stopped`
- Ensures no resource leaks

---

## Next Steps

Congratulations! You've created your first actor. Here's what to explore next:

### 🎯 **Learn More Patterns**
- [Actor Development Tutorial](../guides/actor-development.md) - Deep dive into actor patterns
- [Message Passing Guide](../guides/message-passing.md) - Advanced messaging patterns

### 🔧 **Add Fault Tolerance**
- [Supervisor Patterns](../guides/supervisor-patterns.md) - Build resilient systems
- [Error Handling](../guides/actor-development.md#error-handling) - Robust error strategies

### 📊 **Monitor Your System**
- [Monitoring Guide](../guides/monitoring.md) - Observability and health checks

### ⚡ **Optimize Performance**
- [Performance Guide](../performance/optimization.md) - Benchmarking and tuning
- [BENCHMARKING.md](../../../BENCHMARKING.md) - Baseline performance metrics

---

## Troubleshooting

### Problem: "trait `Message` is not implemented"

**Solution:** Make sure you've implemented the `Message` trait:
```rust
impl Message for YourMessage {
    const MESSAGE_TYPE: &'static str = "your_type";
}
```

### Problem: "future cannot be sent between threads safely"

**Solution:** Ensure your message types are `Send + Sync`:
```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
struct YourMessage { /* fields */ }
```

### Problem: Messages not processing

**Solution:** Add a small delay after sending to allow processing:
```rust
tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;
```

Or use request/reply pattern for synchronous behavior (see [Message Passing Guide](../guides/message-passing.md)).

### Problem: Actor panics on error

**Solution:** Return `Err(YourError)` instead of panicking. The supervisor will handle it:
```rust
if error_condition {
    return Err(YourError::new("Something went wrong"));
}
```

---

## Quick Reference

### Import Everything You Need
```rust
use airssys_rt::prelude::*;
use async_trait::async_trait;
use serde::{Deserialize, Serialize};
```

### Minimal Actor Template
```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
enum MyMessage { /* variants */ }

impl Message for MyMessage {
    const MESSAGE_TYPE: &'static str = "my_message";
}

struct MyActor { /* fields */ }

#[async_trait]
impl Actor for MyActor {
    type Message = MyMessage;
    type Error = MyError;

    async fn handle_message<B: MessageBroker<Self::Message>>(
        &mut self,
        message: Self::Message,
        context: &mut ActorContext<Self::Message, B>,
    ) -> Result<(), Self::Error> {
        // Handle message
        context.record_message();
        Ok(())
    }
}
```

---

## What You Learned

✅ How to add AirsSys-RT to your project  
✅ Defining message types with the `Message` trait  
✅ Implementing the `Actor` trait  
✅ Creating an actor system  
✅ Spawning actors and sending messages  
✅ Understanding the basic message processing workflow  

**Ready for more?** Check out the [Actor Development Tutorial](../guides/actor-development.md) to learn advanced patterns and best practices!
