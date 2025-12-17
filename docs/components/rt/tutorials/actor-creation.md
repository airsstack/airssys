# Tutorial: Your First Actor

**Learning Objectives:**

- Create a custom actor from scratch
- Understand actor state management
- Implement message handling logic
- Test your actor in isolation

**Prerequisites:**

- Complete [Getting Started](./getting-started.md) tutorial
- Basic Rust knowledge (structs, enums, traits)
- Understanding of async/await

**Estimated time:** 25-30 minutes

---

## What You'll Build

A `GreeterActor` that:
- Maintains a greeting counter
- Personalizes greetings based on history
- Demonstrates state management patterns
- Shows proper error handling

**By the end**, you'll understand how to design and implement production-ready actors.

---

## Step 1: Plan Your Actor's Behavior

Before writing code, define what your actor does:

**State:**

- Count of greetings sent
- Map of person names to greeting count

**Messages it handles:**

- `Greet(name)` - Send a greeting
- `GetStats` - Return greeting statistics
- `Reset` - Clear all state

**Responses:**

- Greeting messages (personalized by count)
- Statistics summary
- Confirmation of reset

---

## Step 2: Define Message Types

Create a new file `greeter.rs` in your `src/` directory:

```rust
use airssys_rt::prelude::*;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

// Message enum with all supported operations
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum GreeterMessage {
    Greet { name: String },
    GetStats,
    Reset,
}

impl Message for GreeterMessage {
    const MESSAGE_TYPE: &'static str = "greeter";
}

// Response types for clarity
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum GreeterResponse {
    Greeting(String),
    Stats {
        total: usize,
        per_person: HashMap<String, usize>,
    },
    ResetConfirmed,
}
```

**Key design decisions:**

- **Enum for messages**: Each variant = one operation
- **Struct fields**: Use named fields for clarity (not tuples)
- **Response types**: Explicit types make API clear
- **Clone**: Messages must be cloneable for routing

---

## Step 3: Design Actor State

Define the internal state your actor maintains:

```rust
pub struct GreeterActor {
    total_greetings: usize,
    greetings_per_person: HashMap<String, usize>,
}

impl GreeterActor {
    // Constructor with sensible defaults
    pub fn new() -> Self {
        Self {
            total_greetings: 0,
            greetings_per_person: HashMap::new(),
        }
    }

    // Helper: Generate personalized greeting
    fn generate_greeting(&self, name: &str, count: usize) -> String {
        match count {
            1 => format!("Hello, {name}! Nice to meet you!"),
            2 => format!("Welcome back, {name}!"),
            3..=5 => format!("Hey {name}! Great to see you again!"),
            _ => format!("Hi {name}! You're a regular now!"),
        }
    }

    // Helper: Calculate statistics
    fn get_statistics(&self) -> GreeterResponse {
        GreeterResponse::Stats {
            total: self.total_greetings,
            per_person: self.greetings_per_person.clone(),
        }
    }

    // Helper: Reset all state
    fn reset_state(&mut self) {
        self.total_greetings = 0;
        self.greetings_per_person.clear();
    }
}
```

**Design principles:**

- **Private state**: Fields are not `pub` (encapsulation)
- **Helper methods**: Keep `handle_message` clean
- **Descriptive names**: Code reads like documentation
- **Immutability where possible**: Clone for reads, mutate only when needed

---

## Step 4: Define Error Type

Actors need clear error handling:

```rust
use std::fmt;

#[derive(Debug)]
pub enum GreeterError {
    InvalidName(String),
    TooManyRequests,
}

impl fmt::Display for GreeterError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::InvalidName(name) => write!(f, "Invalid name: {name}"),
            Self::TooManyRequests => write!(f, "Too many greeting requests"),
        }
    }
}

impl std::error::Error for GreeterError {}
```

**Error design:**

- **Enum for error types**: Different error scenarios
- **Display trait**: Human-readable error messages
- **std::error::Error**: Standard Rust error trait

---

## Step 5: Implement the Actor Trait

Now bring it all together:

```rust
use async_trait::async_trait;

#[async_trait]
impl Actor for GreeterActor {
    type Message = GreeterMessage;
    type Error = GreeterError;

    async fn handle_message<B: MessageBroker<Self::Message>>(
        &mut self,
        message: Self::Message,
        context: &mut ActorContext<Self::Message, B>,
    ) -> Result<(), Self::Error> {
        match message {
            GreeterMessage::Greet { name } => {
                // Validation
                if name.trim().is_empty() {
                    return Err(GreeterError::InvalidName(name));
                }

                // Rate limiting (example business logic)
                if self.total_greetings > 1000 {
                    return Err(GreeterError::TooManyRequests);
                }

                // Update state
                self.total_greetings += 1;
                let count = self.greetings_per_person
                    .entry(name.clone())
                    .and_modify(|c| *c += 1)
                    .or_insert(1);

                // Generate and "send" response
                let greeting = self.generate_greeting(&name, *count);
                println!("{greeting}");

                // Record metrics
                context.record_message();
            }

            GreeterMessage::GetStats => {
                let stats = self.get_statistics();
                println!("Stats: {stats:?}");
                context.record_message();
            }

            GreeterMessage::Reset => {
                self.reset_state();
                println!("State reset successfully");
                context.record_message();
            }
        }

        Ok(())
    }

    // Optional: Custom initialization
    async fn pre_start<B: MessageBroker<Self::Message>>(
        &mut self,
        _context: &mut ActorContext<Self::Message, B>,
    ) -> Result<(), Self::Error> {
        println!("GreeterActor starting up...");
        Ok(())
    }

    // Optional: Custom cleanup
    async fn post_stop<B: MessageBroker<Self::Message>>(
        &mut self,
        _context: &mut ActorContext<Self::Message, B>,
    ) -> Result<(), Self::Error> {
        println!("GreeterActor shutting down. Total greetings: {}", self.total_greetings);
        Ok(())
    }
}
```

**Implementation highlights:**

- **Pattern matching**: Clean separation of message handling
- **Validation first**: Check inputs before processing
- **State updates**: Encapsulated in one place
- **Metrics tracking**: `context.record_message()` after each message
- **Lifecycle hooks**: `pre_start` and `post_stop` for setup/cleanup

---

## Step 6: Test Your Actor

Create a test in your `main.rs` or `tests/`:

```rust
use airssys_rt::prelude::*;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("=== Greeter Actor Test ===\n");

    // Create actor
    let mut actor = GreeterActor::new();

    // Setup context
    let address = ActorAddress::named("greeter");
    let broker = InMemoryMessageBroker::<GreeterMessage>::new();
    let mut context = ActorContext::new(address, broker);

    // Start actor
    actor.pre_start(&mut context).await?;

    // Test case 1: First greeting
    println!("Test 1: First greeting");
    let msg = GreeterMessage::Greet { name: "Alice".to_string() };
    actor.handle_message(msg, &mut context).await?;

    // Test case 2: Repeat greeting
    println!("\nTest 2: Repeat greeting");
    let msg = GreeterMessage::Greet { name: "Alice".to_string() };
    actor.handle_message(msg, &mut context).await?;

    // Test case 3: New person
    println!("\nTest 3: New person");
    let msg = GreeterMessage::Greet { name: "Bob".to_string() };
    actor.handle_message(msg, &mut context).await?;

    // Test case 4: Get statistics
    println!("\nTest 4: Statistics");
    let msg = GreeterMessage::GetStats;
    actor.handle_message(msg, &mut context).await?;

    // Test case 5: Invalid name (error handling)
    println!("\nTest 5: Error handling");
    let msg = GreeterMessage::Greet { name: "".to_string() };
    match actor.handle_message(msg, &mut context).await {
        Ok(()) => println!("✗ Should have failed"),
        Err(e) => println!("✓ Expected error: {e}"),
    }

    // Test case 6: Reset
    println!("\nTest 6: Reset");
    let msg = GreeterMessage::Reset;
    actor.handle_message(msg, &mut context).await?;

    // Test case 7: Verify reset worked
    println!("\nTest 7: Verify reset");
    let msg = GreeterMessage::GetStats;
    actor.handle_message(msg, &mut context).await?;

    // Cleanup
    actor.post_stop(&mut context).await?;

    println!("\n=== Test Complete ===");
    Ok(())
}
```

---

## Step 7: Run and Observe

```bash
cargo run
```

**Expected output:**

```
=== Greeter Actor Test ===

GreeterActor starting up...

Test 1: First greeting
Hello, Alice! Nice to meet you!

Test 2: Repeat greeting
Welcome back, Alice!

Test 3: New person
Hello, Bob! Nice to meet you!

Test 4: Statistics
Stats: Stats { total: 3, per_person: {"Alice": 2, "Bob": 1} }

Test 5: Error handling
✓ Expected error: Invalid name: 

Test 6: Reset
State reset successfully

Test 7: Verify reset
Stats: Stats { total: 0, per_person: {} }

GreeterActor shutting down. Total greetings: 0

=== Test Complete ===
```

---

## What You Built

Let's review the key components:

### 1. **Message Design**
```rust
enum GreeterMessage {
    Greet { name: String },  // Named fields (not tuples)
    GetStats,                 // No data needed
    Reset,                    // Simple command
}
```
- ✅ Clear, self-documenting variants
- ✅ Named fields for complex data
- ✅ Simple variants for commands

### 2. **State Management**
```rust
struct GreeterActor {
    total_greetings: usize,
    greetings_per_person: HashMap<String, usize>,
}
```
- ✅ Private fields (encapsulation)
- ✅ Owned data (no shared state)
- ✅ Standard Rust collections

### 3. **Error Handling**
```rust
enum GreeterError {
    InvalidName(String),
    TooManyRequests,
}
```
- ✅ Specific error types
- ✅ Contextual error information
- ✅ Implements `std::error::Error`

### 4. **Actor Implementation**
```rust
impl Actor for GreeterActor {
    async fn handle_message(...) -> Result<(), Self::Error> {
        match message {
            // Handle each message type
        }
    }
}
```
- ✅ Pattern matching for message routing
- ✅ Validation before processing
- ✅ Metrics tracking with `context.record_message()`

---

## Best Practices You Applied

✅ **Separation of Concerns**: Helper methods keep `handle_message` clean  
✅ **Validation**: Check inputs before processing  
✅ **Error Handling**: Return `Err` instead of panicking  
✅ **Encapsulation**: Private state, public interface  
✅ **Lifecycle Management**: `pre_start` and `post_stop` hooks  
✅ **Metrics**: Track message processing  

---

## Common Mistakes to Avoid

### ❌ Don't: Use `panic!` in message handlers
```rust
if name.is_empty() {
    panic!("Invalid name!");  // ❌ Kills the actor
}
```

### ✅ Do: Return errors
```rust
if name.is_empty() {
    return Err(GreeterError::InvalidName(name));  // ✅ Supervisor handles it
}
```

### ❌ Don't: Forget to record metrics
```rust
async fn handle_message(...) {
    // Process message
    Ok(())  // ❌ No metrics tracking
}
```

### ✅ Do: Always record message processing
```rust
async fn handle_message(...) {
    // Process message
    context.record_message();  // ✅ Metrics tracked
    Ok(())
}
```

### ❌ Don't: Share mutable state
```rust
struct BadActor {
    shared: Arc<Mutex<HashMap<String, usize>>>,  // ❌ Defeats actor model
}
```

### ✅ Do: Own your state
```rust
struct GoodActor {
    state: HashMap<String, usize>,  // ✅ Owned, no locks needed
}
```

---

## Next Steps

Congratulations! You've built a complete actor with:
- ✅ State management
- ✅ Message handling
- ✅ Error handling
- ✅ Lifecycle management

### Continue Learning:
- **[Message Handling Tutorial](./message-handling.md)** - Advanced messaging patterns
- **[Supervision Setup Tutorial](./supervision-setup.md)** - Add fault tolerance
- **[Actor Development Guide](../guides/actor-development.md)** - Production patterns

### Explore Examples:
- `examples/actor_basic.rs` - Simple actor patterns
- `examples/actor_lifecycle.rs` - Lifecycle management
- [API Reference: Actors](../reference/api/actors.md) - Complete API docs

---

## Quick Reference

### Actor Implementation Checklist

```rust
// 1. Define messages
#[derive(Debug, Clone, Serialize, Deserialize)]
enum MyMessage { /* variants */ }
impl Message for MyMessage { /* ... */ }

// 2. Define errors
#[derive(Debug)]
enum MyError { /* variants */ }
impl Display for MyError { /* ... */ }
impl std::error::Error for MyError {}

// 3. Define actor state
struct MyActor { /* fields */ }

// 4. Implement Actor trait
#[async_trait]
impl Actor for MyActor {
    type Message = MyMessage;
    type Error = MyError;
    
    async fn handle_message(...) -> Result<(), Self::Error> {
        match message {
            // Handle messages
        }
        context.record_message();
        Ok(())
    }
}
```

**Ready for message handling patterns?** Continue to [Message Handling Tutorial](./message-handling.md)!
