# Tutorial: Your First ComponentActor

**Category**: Tutorial - Learning-oriented  
**Difficulty**: Beginner  
**Duration**: 30-45 minutes  
**Prerequisites**: Basic Rust knowledge, tokio async/await familiarity

---

## What You'll Build

In this tutorial, you will create a simple ComponentActor that:
- Constructs with basic metadata and capabilities
- Tracks internal state (a message counter)
- Handles incoming messages
- Reports its state

You will learn the fundamental ComponentActor patterns used throughout airssys-wasm.

---

## Expected Output

When you complete this tutorial, running your component will produce:

```text
Component created: hello-component
Component ID: hello-component
State: Creating
WASM loaded: false
---
Component is ready!
Message count: 3
Uptime: 5 seconds
---
Component stopped successfully
```

---

## Step 1: Set Up Your Project

First, create a new binary project:

```bash
cargo new --bin my-first-component
cd my-first-component
```

Add dependencies to `Cargo.toml`:

```toml
[dependencies]
airssys-wasm = { path = "../airssys-wasm" }  # Adjust path as needed
airssys-rt = { path = "../airssys-rt" }
tokio = { version = "1.47", features = ["full"] }
chrono = { version = "0.4", features = ["serde"] }
```

**What we're doing:** Setting up a project with access to ComponentActor and the tokio async runtime.

---

## Step 2: Import Required Types

Open `src/main.rs` and add imports following the 3-layer pattern:

```rust
// Layer 1: Standard library imports
use std::time::Duration;

// Layer 2: Third-party crate imports
use tokio::time::sleep;

// Layer 3: Internal module imports
use airssys_wasm::actor::ComponentActor;
use airssys_wasm::core::{ComponentId, ComponentMetadata, CapabilitySet, ResourceLimits};
```

**What we're doing:** Following the mandatory 3-layer import organization from `PROJECTS_STANDARD.md` §2.1.

---

## Step 3: Create Component Metadata

Define metadata describing your component:

```rust
fn create_metadata() -> ComponentMetadata {
    ComponentMetadata {
        name: "hello-component".to_string(),
        version: "1.0.0".to_string(),
        author: "Tutorial User".to_string(),
        description: Some("My first ComponentActor".to_string()),
        required_capabilities: vec![],
        resource_limits: ResourceLimits {
            max_memory_bytes: 64 * 1024 * 1024,    // 64MB memory limit
            max_fuel: 1_000_000,                    // 1M fuel (CPU limit)
            max_execution_ms: 5000,                 // 5 second timeout
            max_storage_bytes: 10 * 1024 * 1024,   // 10MB storage
        },
    }
}
```

**What we're doing:** Metadata describes the component and sets resource limits. These limits protect the host system from runaway components.

**Notice:**

- `max_memory_bytes`: Wasmtime enforces this memory limit
- `max_fuel`: CPU execution limit (prevents infinite loops)
- `max_execution_ms`: Timeout for function calls

---

## Step 4: Define Custom State

Create a simple state struct to track message counts:

```rust
#[derive(Default, Clone)]
struct HelloState {
    message_count: u64,
}
```

**What we're doing:** Custom state lets components maintain data across their lifetime. The state is protected by `Arc<RwLock<HelloState>>` automatically.

**Notice:**

- `Default` trait: Provides initial state
- `Clone` trait: Enables `get_state()` method
- Fields can be any type implementing `Send + Sync + 'static`

---

## Step 5: Create the ComponentActor

Now construct your first ComponentActor:

```rust
#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Create component ID
    let component_id = ComponentId::new("hello-component");
    
    // Create metadata
    let metadata = create_metadata();
    
    // Create capabilities (empty for now)
    let capabilities = CapabilitySet::new();
    
    // Create initial state
    let initial_state = HelloState::default();
    
    // Construct ComponentActor
    let actor: ComponentActor<HelloState> = ComponentActor::new(
        component_id.clone(),
        metadata,
        capabilities,
        initial_state,
    );
    
    println!("Component created: {}", component_id.as_str());
    
    Ok(())
}
```

**What we're doing:** Creating a ComponentActor with custom state. The actor starts in `ActorState::Creating`.

**Performance:** Construction takes 286ns (measured in Task 6.2).

---

## Step 6: Inspect Component State

Add state inspection after creation:

```rust
    // ... after creating actor
    
    // Inspect initial state
    println!("Component ID: {}", actor.component_id().as_str());
    println!("State: {:?}", actor.state());
    println!("WASM loaded: {}", actor.is_wasm_loaded());
    println!("Uptime: {:?}", actor.uptime());
    println!("---");
```

**What we're doing:** Using ComponentActor getter methods to inspect state.

**Expected output:**
```text
Component ID: hello-component
State: Creating
WASM loaded: false
Uptime: None
---
```

**Notice:**

- `state()` returns `ActorState::Creating` (initial state)
- `is_wasm_loaded()` returns `false` (WASM loads in `Child::start()`)
- `uptime()` returns `None` (not started yet)

---

## Step 7: Simulate Message Processing

Simulate processing messages by updating state:

```rust
    // Simulate processing 3 messages
    for i in 1..=3 {
        actor.with_state_mut(|state| {
            state.message_count += 1;
        }).await;
        
        println!("Processed message {}", i);
        
        // Small delay to simulate work
        sleep(Duration::from_millis(100)).await;
    }
    
    println!("---");
```

**What we're doing:** Using `with_state_mut()` to safely modify state. The RwLock ensures thread-safe access.

**Expected output:**
```text
Processed message 1
Processed message 2
Processed message 3
---
```

---

## Step 8: Read Final State

Read the final message count:

```rust
    // Read final state
    let final_count = actor.with_state(|state| state.message_count).await;
    println!("Message count: {}", final_count);
    
    // Or get a cloned copy
    let state_copy = actor.get_state().await;
    println!("Message count (copy): {}", state_copy.message_count);
```

**What we're doing:** Using `with_state()` for read-only access and `get_state()` to clone the entire state.

**Notice:**

- `with_state()`: Read-only access (multiple readers allowed)
- `with_state_mut()`: Mutable access (exclusive lock)
- `get_state()`: Clones state (requires `Clone` trait)

---

## Step 9: Calculate Simulated Uptime

Since we didn't actually start the WASM runtime (requires WASM bytecode), simulate uptime:

```rust
    // In a real scenario, uptime would be calculated from started_at
    let simulated_uptime = Duration::from_secs(5);
    println!("Uptime: {} seconds", simulated_uptime.as_secs());
    println!("---");
```

---

## Step 10: Cleanup

Add cleanup message:

```rust
    println!("Component stopped successfully");
    
    Ok(())
}
```

---

## Complete Code

Here's the full `src/main.rs`:

```rust
// Layer 1: Standard library imports
use std::time::Duration;

// Layer 2: Third-party crate imports
use tokio::time::sleep;

// Layer 3: Internal module imports
use airssys_wasm::actor::ComponentActor;
use airssys_wasm::core::{ComponentId, ComponentMetadata, CapabilitySet, ResourceLimits};

#[derive(Default, Clone)]
struct HelloState {
    message_count: u64,
}

fn create_metadata() -> ComponentMetadata {
    ComponentMetadata {
        name: "hello-component".to_string(),
        version: "1.0.0".to_string(),
        author: "Tutorial User".to_string(),
        description: Some("My first ComponentActor".to_string()),
        required_capabilities: vec![],
        resource_limits: ResourceLimits {
            max_memory_bytes: 64 * 1024 * 1024,
            max_fuel: 1_000_000,
            max_execution_ms: 5000,
            max_storage_bytes: 10 * 1024 * 1024,
        },
    }
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Create component ID
    let component_id = ComponentId::new("hello-component");
    
    // Create metadata
    let metadata = create_metadata();
    
    // Create capabilities
    let capabilities = CapabilitySet::new();
    
    // Create initial state
    let initial_state = HelloState::default();
    
    // Construct ComponentActor
    let actor: ComponentActor<HelloState> = ComponentActor::new(
        component_id.clone(),
        metadata,
        capabilities,
        initial_state,
    );
    
    println!("Component created: {}", component_id.as_str());
    
    // Inspect initial state
    println!("Component ID: {}", actor.component_id().as_str());
    println!("State: {:?}", actor.state());
    println!("WASM loaded: {}", actor.is_wasm_loaded());
    println!("---");
    
    // Simulate processing 3 messages
    for i in 1..=3 {
        actor.with_state_mut(|state| {
            state.message_count += 1;
        }).await;
        
        println!("Processed message {}", i);
        sleep(Duration::from_millis(100)).await;
    }
    
    println!("---");
    
    // Read final state
    let final_count = actor.with_state(|state| state.message_count).await;
    println!("Message count: {}", final_count);
    
    // Simulated uptime
    let simulated_uptime = Duration::from_secs(5);
    println!("Uptime: {} seconds", simulated_uptime.as_secs());
    println!("---");
    
    println!("Component stopped successfully");
    
    Ok(())
}
```

---

## Run Your Component

Build and run:

```bash
cargo build
cargo run
```

**Expected output:**
```text
Component created: hello-component
Component ID: hello-component
State: Creating
WASM loaded: false
---
Processed message 1
Processed message 2
Processed message 3
---
Message count: 3
Uptime: 5 seconds
---
Component stopped successfully
```

**You have successfully created your first ComponentActor!**

---

## What You Learned

- ✅ Creating ComponentActor with custom state
- ✅ Using ComponentMetadata and ResourceLimits
- ✅ Inspecting component state with getter methods
- ✅ Safely modifying state with `with_state_mut()`
- ✅ Reading state with `with_state()`
- ✅ Following 3-layer import organization

---

## Common Mistakes

### 1. Forgetting `#[tokio::main]`

```rust
// ❌ ERROR: Cannot call async functions without runtime
fn main() {
    let actor = ComponentActor::new(/* ... */);
    actor.with_state(|s| s.count).await;  // Error!
}

// ✅ CORRECT: Use #[tokio::main]
#[tokio::main]
async fn main() {
    // Now .await works
}
```

### 2. Missing Trait Bounds on State

```rust
// ❌ ERROR: State must be Send + Sync
struct BadState {
    value: Rc<u64>,  // Rc is not Send!
}

// ✅ CORRECT: Use Arc for shared ownership
use std::sync::Arc;
struct GoodState {
    value: Arc<u64>,
}
```

### 3. Not Following 3-Layer Imports

```rust
// ❌ BAD: Mixed import order
use airssys_wasm::actor::ComponentActor;
use std::time::Duration;
use tokio::time::sleep;

// ✅ CORRECT: 3-layer organization
// Layer 1: std
// Layer 2: third-party
// Layer 3: crate
```

---

## Next Steps

Now that you've created a basic ComponentActor, try these next tutorials:

1. **[Stateful Component Tutorial](./stateful-component-tutorial.md)** - Build a component with complex state
2. **[Request-Response Pattern](../guides/request-response-pattern.md)** - Implement component communication
3. **[Pub-Sub Broadcasting](../guides/pubsub-broadcasting.md)** - Subscribe to topics and receive messages

---

## Troubleshooting

### "Cannot find ComponentActor in airssys_wasm"

**Solution:** Check that your `Cargo.toml` has the correct path:
```toml
airssys-wasm = { path = "../airssys-wasm" }
```

### "RwLock poison error"

**Solution:** This occurs if a panic happens while holding a lock. Ensure your state operations don't panic:
```rust
// ✅ GOOD: Return Result instead of panicking
actor.with_state_mut(|state| {
    if state.count < u64::MAX {
        state.count += 1;
    }
}).await;
```

### "Send bound not satisfied"

**Solution:** Ensure all types in your state implement `Send + Sync`:
```rust
// ❌ BAD
struct State {
    value: Rc<u64>,  // Not Send!
}

// ✅ GOOD
struct State {
    value: Arc<u64>,  // Send + Sync!
}
```

---

## Related Documentation

- [ComponentActor API Reference](../api/component-actor.md) - Full API documentation
- [Lifecycle Hooks](../api/lifecycle-hooks.md) - Customize lifecycle behavior
- [State Management Patterns](../explanation/state-management-patterns.md) - Best practices for state

## References

- **PROJECTS_STANDARD.md §2.1**: 3-layer import organization
- **Task 6.2 Performance**: 286ns construction time
