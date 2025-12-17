# Lifecycle Hooks API Reference

**Category**: Reference - Information-oriented  
**Audience**: Developers implementing ComponentActor lifecycle customization  
**Purpose**: Technical specification of lifecycle hook methods and execution order

---

## Overview

Lifecycle hooks provide extension points for customizing ComponentActor behavior at key lifecycle events. Hooks are invoked automatically by the runtime during state transitions, enabling initialization logic, cleanup, monitoring, and error handling without modifying core ComponentActor code.

## Hook Execution Order

```text
ComponentActor::new()
         ↓
    pre_start() ────────> [hooks called before Child::start()]
         ↓
    Child::start() ─────> [WASM instantiation]
         ↓
    post_start() ───────> [hooks called after successful start]
         ↓
   ActorState::Ready ───> [component processes messages]
         ↓
    pre_stop() ─────────> [hooks called before Child::stop()]
         ↓
    Child::stop() ──────> [WASM cleanup]
         ↓
    post_stop() ────────> [hooks called after successful stop]
         ↓
   ActorState::Terminated
```

### Hook Timing

Measured in Task 6.2 (`task-004-phase-6-task-6.2-completion-report.md`):

- **Lifecycle overhead** (pre_start + post_start + pre_stop + post_stop): 5-8µs
- **Individual hook call**: < 1µs (when using NoOpHooks)
- **Full lifecycle** (construction + start + stop): 1.49µs

---

## Trait: LifecycleHooks

Base trait for implementing custom lifecycle hooks.

**Definition:**
```rust
pub trait LifecycleHooks: Send + Sync {
    fn pre_start(&mut self, ctx: &LifecycleContext) -> HookResult;
    fn post_start(&mut self, ctx: &LifecycleContext) -> HookResult;
    fn pre_stop(&mut self, ctx: &LifecycleContext) -> HookResult;
    fn post_stop(&mut self, ctx: &LifecycleContext) -> HookResult;
    fn on_message_received(&mut self, ctx: &LifecycleContext, msg: &ComponentMessage) -> HookResult;
    fn on_error(&mut self, ctx: &LifecycleContext, error: &WasmError) -> HookResult;
    fn on_restart(&mut self, ctx: &LifecycleContext, reason: &str) -> HookResult;
}
```

---

## Hook Methods

### `pre_start()`

Called before WASM instantiation begins.

**Invocation:** Before `Child::start()` creates the WASM runtime

**Purpose:** 

- Validate component metadata
- Initialize external resources (databases, connections)
- Pre-allocate resources
- Log component startup

**Parameters:**

- `ctx` - Lifecycle context with component ID and metadata

**Returns:** `HookResult` (Ok to continue, Err to abort startup)

**Performance:** < 1µs overhead when using NoOpHooks

**Example:**
```rust
use airssys_wasm::actor::lifecycle::{LifecycleHooks, LifecycleContext, HookResult};

struct MyHooks;

impl LifecycleHooks for MyHooks {
    fn pre_start(&mut self, ctx: &LifecycleContext) -> HookResult {
        println!("Component starting: {}", ctx.component_id.as_str());
        
        // Validate resource limits
        if ctx.metadata.resource_limits.max_memory_bytes < 1_000_000 {
            return HookResult::Err("Insufficient memory allocation".to_string());
        }
        
        HookResult::Ok
    }
    
    // ... other hooks
}
```

---

### `post_start()`

Called after successful WASM instantiation.

**Invocation:** After `Child::start()` successfully creates WASM runtime

**Purpose:**

- Initialize component state
- Subscribe to topics
- Register with external services
- Log successful startup

**Parameters:**

- `ctx` - Lifecycle context with component ID and metadata

**Returns:** `HookResult` (Ok to continue, Err to fail startup)

**Example:**
```rust
fn post_start(&mut self, ctx: &LifecycleContext) -> HookResult {
    println!("Component started successfully: {}", ctx.component_id.as_str());
    
    // Log uptime start
    tracing::info!(
        component_id = %ctx.component_id.as_str(),
        "Component ready for messages"
    );
    
    HookResult::Ok
}
```

---

### `pre_stop()`

Called before component shutdown begins.

**Invocation:** Before `Child::stop()` initiates cleanup

**Purpose:**

- Flush pending operations
- Save state to persistent storage
- Unsubscribe from topics
- Notify dependent components

**Parameters:**

- `ctx` - Lifecycle context with component ID and metadata

**Returns:** `HookResult` (Ok to continue, Err logged but cleanup proceeds)

**Example:**
```rust
fn pre_stop(&mut self, ctx: &LifecycleContext) -> HookResult {
    println!("Component stopping: {}", ctx.component_id.as_str());
    
    // Flush pending operations
    // (Note: Errors in pre_stop don't prevent shutdown)
    
    HookResult::Ok
}
```

---

### `post_stop()`

Called after successful component shutdown.

**Invocation:** After `Child::stop()` completes cleanup

**Purpose:**

- Release external resources
- Log final statistics
- Send termination notifications
- Close connections

**Parameters:**

- `ctx` - Lifecycle context with component ID and metadata

**Returns:** `HookResult` (errors logged, cleanup always completes)

**Example:**
```rust
fn post_stop(&mut self, ctx: &LifecycleContext) -> HookResult {
    println!("Component stopped: {}", ctx.component_id.as_str());
    
    // Log final statistics
    tracing::info!(
        component_id = %ctx.component_id.as_str(),
        "Component terminated successfully"
    );
    
    HookResult::Ok
}
```

---

### `on_message_received()`

Called before routing each incoming message.

**Invocation:** Before `Actor::handle_message()` processes the message

**Purpose:**

- Message-level logging
- Rate limiting
- Message filtering
- Metrics collection

**Parameters:**

- `ctx` - Lifecycle context
- `msg` - Incoming ComponentMessage

**Returns:** `HookResult` (Ok to process, Err to drop message)

**Performance:** < 1µs overhead per message (NoOpHooks)

**Example:**
```rust
fn on_message_received(
    &mut self,
    ctx: &LifecycleContext,
    msg: &ComponentMessage
) -> HookResult {
    // Log message arrival
    tracing::debug!(
        component_id = %ctx.component_id.as_str(),
        message_type = ?msg,
        "Message received"
    );
    
    // Rate limiting example
    if self.message_count > 1000 {
        return HookResult::Err("Rate limit exceeded".to_string());
    }
    
    self.message_count += 1;
    HookResult::Ok
}
```

---

### `on_error()`

Called when an error occurs during component execution.

**Invocation:** After any error in lifecycle or message handling

**Purpose:**

- Error logging
- Error metrics
- Alert notifications
- Recovery attempts

**Parameters:**

- `ctx` - Lifecycle context
- `error` - WasmError that occurred

**Returns:** `HookResult` (return value currently ignored, error propagated)

**Example:**
```rust
fn on_error(
    &mut self,
    ctx: &LifecycleContext,
    error: &WasmError
) -> HookResult {
    tracing::error!(
        component_id = %ctx.component_id.as_str(),
        error = %error,
        "Component error occurred"
    );
    
    // Increment error counter
    self.error_count += 1;
    
    HookResult::Ok
}
```

---

### `on_restart()`

Called when supervisor triggers component restart.

**Invocation:** Before restart attempt after failure

**Purpose:**

- Log restart reason
- Restart metrics
- Reset internal state
- Notify monitoring systems

**Parameters:**

- `ctx` - Lifecycle context
- `reason` - Restart reason description

**Returns:** `HookResult` (Ok to continue, Err logged but restart proceeds)

**Example:**
```rust
fn on_restart(
    &mut self,
    ctx: &LifecycleContext,
    reason: &str
) -> HookResult {
    tracing::warn!(
        component_id = %ctx.component_id.as_str(),
        reason = reason,
        restart_count = self.restart_count,
        "Component restarting"
    );
    
    self.restart_count += 1;
    
    HookResult::Ok
}
```

---

## Struct: LifecycleContext

Context information passed to hook methods.

**Fields:**

- `component_id: ComponentId` - Component identifier
- `metadata: ComponentMetadata` - Component metadata (name, version, limits)

**Example:**
```rust
use airssys_wasm::actor::lifecycle::LifecycleContext;

fn pre_start(&mut self, ctx: &LifecycleContext) -> HookResult {
    println!("Starting: {} v{}", 
        ctx.metadata.name, 
        ctx.metadata.version
    );
    
    println!("Memory limit: {} bytes", 
        ctx.metadata.resource_limits.max_memory_bytes
    );
    
    HookResult::Ok
}
```

---

## Enum: HookResult

Result type returned by hook methods.

**Variants:**

- `Ok` - Hook succeeded, continue execution
- `Err(String)` - Hook failed with error message

**Example:**
```rust
use airssys_wasm::actor::lifecycle::HookResult;

// Success
return HookResult::Ok;

// Error with context
return HookResult::Err(format!(
    "Validation failed: insufficient memory (need {}MB, have {}MB)",
    required_mb, available_mb
));
```

---

## Struct: NoOpHooks

Default zero-overhead hook implementation.

All hook methods return `HookResult::Ok` immediately with no side effects.

**Performance:** < 1µs per hook call

**Example:**
```rust
use airssys_wasm::actor::lifecycle::NoOpHooks;

// Default hooks (no-op)
let hooks = Box::new(NoOpHooks);
actor.set_lifecycle_hooks(hooks);
```

---

## Error Handling in Hooks

### Startup Hooks

- **pre_start() error:** Startup aborted, component transitions to `ActorState::Failed`
- **post_start() error:** Startup fails, cleanup initiated

### Shutdown Hooks

- **pre_stop() error:** Error logged, shutdown continues
- **post_stop() error:** Error logged, cleanup completes

### Message Hooks

- **on_message_received() error:** Message dropped, error logged

### Other Hooks

- **on_error()**: Return value ignored, error propagated
- **on_restart()**: Error logged, restart proceeds

---

## Best Practices

### Keep Hooks Fast

Hooks are called in the critical path. Minimize execution time:

```rust
// ✅ GOOD - Fast validation
fn pre_start(&mut self, ctx: &LifecycleContext) -> HookResult {
    if ctx.metadata.name.is_empty() {
        return HookResult::Err("Component name required".to_string());
    }
    HookResult::Ok
}

// ❌ AVOID - Slow I/O in critical path
fn pre_start(&mut self, ctx: &LifecycleContext) -> HookResult {
    // Blocking database query in startup path!
    let result = database.query("SELECT * FROM components").unwrap();
    HookResult::Ok
}
```

### Idempotent Hooks

Design hooks to be safely called multiple times:

```rust
fn post_start(&mut self, ctx: &LifecycleContext) -> HookResult {
    // ✅ GOOD - Idempotent subscription
    if !self.subscribed {
        broker.subscribe("events").await?;
        self.subscribed = true;
    }
    HookResult::Ok
}
```

### Error Recovery

Hooks should not panic. Return `HookResult::Err` for errors:

```rust
fn on_error(&mut self, ctx: &LifecycleContext, error: &WasmError) -> HookResult {
    // ✅ GOOD - Graceful error handling
    if let Err(e) = self.send_alert(error) {
        tracing::warn!("Failed to send alert: {}", e);
        // Continue anyway, don't propagate alert failure
    }
    HookResult::Ok
}
```

---

## Complete Example

```rust
use airssys_wasm::actor::lifecycle::{LifecycleHooks, LifecycleContext, HookResult};
use airssys_wasm::actor::ComponentMessage;
use airssys_wasm::core::WasmError;
use std::sync::atomic::{AtomicU64, Ordering};

struct MetricsHooks {
    message_count: AtomicU64,
    error_count: AtomicU64,
    restart_count: AtomicU64,
}

impl MetricsHooks {
    fn new() -> Self {
        Self {
            message_count: AtomicU64::new(0),
            error_count: AtomicU64::new(0),
            restart_count: AtomicU64::new(0),
        }
    }
}

impl LifecycleHooks for MetricsHooks {
    fn pre_start(&mut self, ctx: &LifecycleContext) -> HookResult {
        tracing::info!("Component starting: {}", ctx.component_id.as_str());
        HookResult::Ok
    }

    fn post_start(&mut self, ctx: &LifecycleContext) -> HookResult {
        tracing::info!("Component ready: {}", ctx.component_id.as_str());
        HookResult::Ok
    }

    fn pre_stop(&mut self, ctx: &LifecycleContext) -> HookResult {
        tracing::info!(
            "Component stopping: {} (processed {} messages)",
            ctx.component_id.as_str(),
            self.message_count.load(Ordering::Relaxed)
        );
        HookResult::Ok
    }

    fn post_stop(&mut self, ctx: &LifecycleContext) -> HookResult {
        tracing::info!("Component stopped: {}", ctx.component_id.as_str());
        HookResult::Ok
    }

    fn on_message_received(
        &mut self,
        _ctx: &LifecycleContext,
        _msg: &ComponentMessage
    ) -> HookResult {
        self.message_count.fetch_add(1, Ordering::Relaxed);
        HookResult::Ok
    }

    fn on_error(
        &mut self,
        ctx: &LifecycleContext,
        error: &WasmError
    ) -> HookResult {
        self.error_count.fetch_add(1, Ordering::Relaxed);
        tracing::error!(
            component_id = %ctx.component_id.as_str(),
            error = %error,
            "Component error"
        );
        HookResult::Ok
    }

    fn on_restart(
        &mut self,
        ctx: &LifecycleContext,
        reason: &str
    ) -> HookResult {
        self.restart_count.fetch_add(1, Ordering::Relaxed);
        tracing::warn!(
            component_id = %ctx.component_id.as_str(),
            reason = reason,
            "Component restarting"
        );
        HookResult::Ok
    }
}

// Usage
let mut actor = ComponentActor::new(/* ... */);
actor.set_lifecycle_hooks(Box::new(MetricsHooks::new()));
```

---

## Related Documentation

- [ComponentActor API](./component-actor.md) - Core API reference
- [Tutorial: Stateful Component](../tutorials/stateful-component-tutorial.md) - Using hooks with state
- [State Management Patterns](../explanation/state-management-patterns.md) - State design rationale

## References

- **ADR-WASM-006**: Component Isolation and Sandboxing
- **Phase 5 Task 5.2**: Lifecycle Hooks Implementation
- **Task 6.2 Performance Report**: Hook overhead measurements
