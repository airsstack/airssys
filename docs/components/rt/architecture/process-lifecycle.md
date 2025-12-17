# Process Lifecycle

The process lifecycle system in `airssys-rt` manages actor state transitions, restart tracking, and supervision integration following Erlang/OTP principles.

> **Note**: All code examples are from actual implementation. See [examples directory](../../examples/) for complete working code.

## Architecture Overview

### Design Principles

The lifecycle system provides:

1. **State Machine** - Well-defined actor states and transitions
2. **Lifecycle Hooks** - Initialization and cleanup callbacks
3. **Restart Tracking** - Failure counting and state change history
4. **Supervision Integration** - Error actions and restart policies

**Performance Characteristics** (from BENCHMARKING.md):
- **Actor spawn**: 624.74 ns (single), 681.40 ns/actor (batch of 10)
- **Lifecycle transition**: Sub-nanosecond (enum assignment)
- **Restart overhead**: 10-50 µs (includes stop → start hooks)

## Actor State Machine

### State Definitions

Actors transition through defined states (from `src/actor/lifecycle.rs`):

```rust
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ActorState {
    Starting,   // Actor is initializing
    Running,    // Actor is active and processing messages
    Stopping,   // Actor is shutting down
    Stopped,    // Actor has stopped successfully
    Failed,     // Actor has failed (requires supervision)
}
```

### State Transitions

Valid transitions in the state machine:

```
           start()
  Starting ────────> Running
     │                 │
     │ error           │ stop()
     ▼                 ▼
  Failed            Stopping
                       │
                       │ cleanup
                       ▼
                    Stopped
                       │
                       │ restart
                       ▼
                    Starting
```

**Key Properties:**

- **Starting** → **Running**: Successful initialization
- **Starting** → **Failed**: Initialization error
- **Running** → **Stopping**: Graceful shutdown request
- **Running** → **Failed**: Runtime error
- **Stopping** → **Stopped**: Successful cleanup
- **Stopped** → **Starting**: Supervisor restart

### State Query Methods

```rust
impl ActorLifecycle {
    /// Check if actor is in terminal state (Stopped or Failed)
    pub fn is_terminal(&self) -> bool {
        matches!(self.state, ActorState::Stopped | ActorState::Failed)
    }

    /// Check if actor is actively running
    pub fn is_running(&self) -> bool {
        self.state == ActorState::Running
    }

    /// Check if actor can accept messages
    pub fn can_process_messages(&self) -> bool {
        self.state == ActorState::Running
    }
}
```

## ActorLifecycle Structure

### Definition

State management structure (from `src/actor/lifecycle.rs`):

```rust
#[derive(Debug, Clone)]
pub struct ActorLifecycle {
    state: ActorState,
    last_state_change: DateTime<Utc>,  // §3.2 chrono DateTime<Utc>
    restart_count: u32,
}
```

**Fields:**

- `state`: Current actor state
- `last_state_change`: When state last changed (UTC timestamp)
- `restart_count`: Number of times actor has been restarted

### Creation and Management

```rust
impl ActorLifecycle {
    /// Create new lifecycle in Starting state
    pub fn new() -> Self {
        Self {
            state: ActorState::Starting,
            last_state_change: Utc::now(),
            restart_count: 0,
        }
    }

    /// Get current state
    pub fn state(&self) -> ActorState {
        self.state
    }

    /// Transition to new state
    pub fn transition_to(&mut self, new_state: ActorState) {
        self.state = new_state;
        self.last_state_change = Utc::now();
    }

    /// Get restart count
    pub fn restart_count(&self) -> u32 {
        self.restart_count
    }

    /// Increment restart counter
    pub fn record_restart(&mut self) {
        self.restart_count += 1;
    }

    /// Get time of last state change
    pub fn last_state_change(&self) -> DateTime<Utc> {
        self.last_state_change
    }
}
```

## Lifecycle Hooks

### Actor Trait Hooks

Actors can override lifecycle callbacks (from `src/actor/traits.rs`):

```rust
#[async_trait]
pub trait Actor: Send + Sync + 'static {
    type Message: Message;
    type Error: Error + Send + Sync + 'static;

    /// Called before actor starts processing messages
    async fn pre_start<B: MessageBroker<Self::Message>>(
        &mut self,
        context: &mut ActorContext<Self::Message, B>,
    ) -> Result<(), Self::Error> {
        Ok(())  // Default: no-op
    }

    /// Called when actor is stopping
    async fn post_stop<B: MessageBroker<Self::Message>>(
        &mut self,
        context: &mut ActorContext<Self::Message, B>,
    ) -> Result<(), Self::Error> {
        Ok(())  // Default: no-op
    }

    /// Handle errors and return supervision decision
    async fn on_error<B: MessageBroker<Self::Message>>(
        &mut self,
        error: Self::Error,
        context: &mut ActorContext<Self::Message, B>,
    ) -> ErrorAction {
        ErrorAction::Restart  // Default: restart on error
    }

    /// Handle incoming message (required)
    async fn handle_message<B: MessageBroker<Self::Message>>(
        &mut self,
        message: Self::Message,
        context: &mut ActorContext<Self::Message, B>,
    ) -> Result<(), Self::Error>;
}
```

### Hook Usage Patterns

**pre_start** - Initialization:
```rust
async fn pre_start<B: MessageBroker<Self::Message>>(
    &mut self,
    context: &mut ActorContext<Self::Message, B>,
) -> Result<(), Self::Error> {
    println!("Actor {} starting", context.address().name().unwrap_or("anonymous"));
    
    // Initialize resources
    self.database = Some(Database::connect().await?);
    self.cache = Cache::new();
    
    // Subscribe to events
    context.subscribe("system-events").await?;
    
    Ok(())
}
```

**post_stop** - Cleanup:
```rust
async fn post_stop<B: MessageBroker<Self::Message>>(
    &mut self,
    context: &mut ActorContext<Self::Message, B>,
) -> Result<(), Self::Error> {
    println!("Actor {} stopping", context.address().name().unwrap_or("anonymous"));
    
    // Release resources
    if let Some(db) = self.database.take() {
        db.disconnect().await?;
    }
    
    // Unsubscribe from events
    context.unsubscribe("system-events").await?;
    
    Ok(())
}
```

**on_error** - Error handling:
```rust
async fn on_error<B: MessageBroker<Self::Message>>(
    &mut self,
    error: Self::Error,
    context: &mut ActorContext<Self::Message, B>,
) -> ErrorAction {
    eprintln!("Actor {} error: {}", context.id(), error);
    
    match error {
        MyError::Recoverable(_) => ErrorAction::Resume,
        MyError::RestartRequired(_) => ErrorAction::Restart,
        MyError::Fatal(_) => ErrorAction::Stop,
        MyError::SystemIssue(_) => ErrorAction::Escalate,
    }
}
```

See `examples/actor_lifecycle.rs` for complete lifecycle hook examples.

## Lifecycle Execution Flow

### Normal Startup

```
1. SupervisorNode::start_child(spec, child)
   ↓
2. ActorLifecycle::new() → state = Starting
   ↓
3. Actor::pre_start(context) → initialize resources
   ↓
4. lifecycle.transition_to(Running) → state = Running
   ↓
5. Actor starts processing messages
```

**Timing:**

- Steps 1-4: ~624 ns for actor spawn (measured)
- `pre_start()`: Depends on initialization logic (aim for <10ms)

### Normal Shutdown

```
1. SupervisorNode::stop_child(child_id)
   ↓
2. lifecycle.transition_to(Stopping) → state = Stopping
   ↓
3. Actor::post_stop(context) → cleanup resources
   ↓
4. lifecycle.transition_to(Stopped) → state = Stopped
```

**Timing:**

- Steps 2-4: ~10-50 µs typical (depends on cleanup logic)

### Failure and Restart

```
1. Actor::handle_message() → returns Err(error)
   ↓
2. lifecycle.transition_to(Failed) → state = Failed
   ↓
3. Supervisor detects failure
   ↓
4. Actor::on_error(error, context) → returns ErrorAction
   ↓
5. Supervisor applies restart strategy
   ↓
   a. OneForOne: restart only this actor
   b. OneForAll: restart all children
   c. RestForOne: restart this + subsequent children
   ↓
6. lifecycle.record_restart() → restart_count++
   ↓
7. Actor::post_stop() → cleanup failed state
   ↓
8. lifecycle.transition_to(Starting) → state = Starting
   ↓
9. Actor::pre_start() → reinitialize
   ↓
10. lifecycle.transition_to(Running) → state = Running
```

**Timing:**

- Complete restart cycle: 10-50 µs (OneForOne)
- Complete restart cycle: 30-150 µs (OneForAll, 3 children)

## Error Actions

### ErrorAction Enum

Actors return `ErrorAction` from `on_error` to control supervision (from `src/actor/traits.rs`):

```rust
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ErrorAction {
    Resume,    // Continue processing (ignore error)
    Restart,   // Restart the actor
    Stop,      // Stop the actor permanently
    Escalate,  // Pass error to supervisor
}
```

### Action Semantics

**Resume** - Transient error, continue:
```rust
async fn on_error<B: MessageBroker<Self::Message>>(
    &mut self,
    error: Self::Error,
    _context: &mut ActorContext<Self::Message, B>,
) -> ErrorAction {
    match error {
        MyError::Timeout => {
            eprintln!("Timeout, will retry next message");
            ErrorAction::Resume  // Continue processing
        }
        _ => ErrorAction::Restart,
    }
}
```

**Restart** - Recoverable error, reset state:
```rust
ErrorAction::Restart  // Supervisor will stop → start actor
```

**Stop** - Unrecoverable error, terminate:
```rust
MyError::ConfigurationInvalid(_) => {
    eprintln!("Invalid configuration, cannot continue");
    ErrorAction::Stop  // Supervisor won't restart
}
```

**Escalate** - Supervisor decision needed:
```rust
MyError::SystemFailure(_) => {
    eprintln!("System-level failure, escalating to supervisor");
    ErrorAction::Escalate  // Let supervisor handle
}
```

## Restart Tracking

### Restart Counter

Track restart frequency for monitoring:

```rust
let lifecycle = actor_context.lifecycle();
println!("Actor has been restarted {} times", lifecycle.restart_count());

if lifecycle.restart_count() > 10 {
    eprintln!("Warning: Actor restarting frequently, check for issues");
}
```

### Restart Rate Limiting

Prevent restart storms with rate limiting (not yet implemented, planned):

```rust
// Future enhancement
pub struct RestartRateLimiter {
    max_restarts: u32,
    time_window: Duration,
    restart_history: VecDeque<DateTime<Utc>>,
}

impl RestartRateLimiter {
    pub fn should_allow_restart(&mut self) -> bool {
        let now = Utc::now();
        let cutoff = now - self.time_window;
        
        // Remove old restarts
        self.restart_history.retain(|&time| time > cutoff);
        
        // Check limit
        if self.restart_history.len() >= self.max_restarts as usize {
            false  // Too many restarts, give up
        } else {
            self.restart_history.push_back(now);
            true
        }
    }
}
```

## Integration with Supervision

### Child Trait Implementation

Actors implement `Child` trait for supervision (from `src/supervisor/child.rs`):

```rust
#[async_trait]
impl<A: Actor> Child for A {
    async fn start(&mut self) -> Result<(), Box<dyn Error + Send + Sync>> {
        // Calls Actor::pre_start internally
        self.pre_start(&mut context).await
            .map_err(|e| Box::new(e) as Box<dyn Error + Send + Sync>)
    }

    async fn stop(&mut self) -> Result<(), Box<dyn Error + Send + Sync>> {
        // Calls Actor::post_stop internally
        self.post_stop(&mut context).await
            .map_err(|e| Box::new(e) as Box<dyn Error + Send + Sync>)
    }

    async fn health_check(&self) -> ChildHealth {
        // Default implementation, can be overridden
        if self.lifecycle.is_running() {
            ChildHealth::Healthy
        } else {
            ChildHealth::Unhealthy(format!("State: {:?}", self.lifecycle.state()))
        }
    }
}
```

### Restart Policy Integration

Restart policies control when `Child::start()` is called:

```rust
// In Supervisor
async fn handle_child_failure(&mut self, child_id: &ChildId) -> Result<(), SupervisorError> {
    let (spec, child) = self.find_child_mut(child_id)?;
    
    match spec.restart_policy {
        RestartPolicy::Permanent => {
            // Always restart
            child.stop().await?;
            child.lifecycle.record_restart();
            child.lifecycle.transition_to(ActorState::Starting);
            child.start().await?;
            child.lifecycle.transition_to(ActorState::Running);
        }
        RestartPolicy::Transient => {
            // Restart only if abnormal termination
            if child.lifecycle.state() == ActorState::Failed {
                child.stop().await?;
                child.lifecycle.record_restart();
                child.lifecycle.transition_to(ActorState::Starting);
                child.start().await?;
                child.lifecycle.transition_to(ActorState::Running);
            }
        }
        RestartPolicy::Temporary => {
            // Never restart, just cleanup
            child.stop().await?;
            child.lifecycle.transition_to(ActorState::Stopped);
        }
    }
    
    Ok(())
}
```

## Best Practices

### Lifecycle Hook Design

**DO:**

- ✅ Keep `pre_start()` fast (<10ms ideal) for quick spawning
- ✅ Make `post_stop()` idempotent (safe to call multiple times)
- ✅ Handle all resource cleanup in `post_stop()`
- ✅ Use `on_error()` for fine-grained error handling
- ✅ Log state transitions for debugging

**DON'T:**

- ❌ Block indefinitely in lifecycle hooks
- ❌ Ignore errors in `pre_start()` (return proper errors)
- ❌ Leave resources open if `post_stop()` fails
- ❌ Use `panic!()` in lifecycle hooks (return errors)
- ❌ Assume `pre_start()` always completes (may fail)

### Error Action Selection

**Resume:**

- Use for: Transient network errors, timeouts, retryable operations
- Example: HTTP request timeout, temporary database unavailable
- Effect: Actor continues with current state

**Restart:**

- Use for: Corrupted state, logical errors, recoverable failures
- Example: Invalid state detected, cache corruption, connection lost
- Effect: Actor stops, cleans up, and restarts fresh

**Stop:**

- Use for: Configuration errors, unrecoverable failures, fatal issues
- Example: Invalid config file, missing required resource, critical bug
- Effect: Actor stops permanently, supervisor won't restart (unless Permanent policy)

**Escalate:**

- Use for: System-level issues, supervisor decision needed
- Example: Disk full, out of memory, authentication service down
- Effect: Supervisor handles based on its strategy

### State Validation

```rust
impl MyActor {
    /// Validate actor is in correct state for operation
    fn ensure_running(&self) -> Result<(), MyError> {
        if !self.lifecycle.is_running() {
            return Err(MyError::NotRunning {
                current_state: self.lifecycle.state(),
            });
        }
        Ok(())
    }

    /// Check if actor is healthy
    fn is_healthy(&self) -> bool {
        self.lifecycle.is_running() && 
        self.lifecycle.restart_count() < 10 &&
        self.connection.is_some()
    }
}
```

## Monitoring and Observability

### Lifecycle Metrics

Track lifecycle events for monitoring:

```rust
use airssys_rt::monitoring::ActorMetrics;

impl MyActor {
    async fn pre_start<B: MessageBroker<Self::Message>>(
        &mut self,
        context: &mut ActorContext<Self::Message, B>,
    ) -> Result<(), Self::Error> {
        // Record startup
        metrics::increment_counter!("actor_starts_total", 
            "actor" => context.address().name().unwrap_or("unknown"));
        
        let start_time = Instant::now();
        
        // Initialize
        self.initialize().await?;
        
        // Record startup duration
        metrics::histogram!("actor_start_duration_seconds", 
            start_time.elapsed().as_secs_f64(),
            "actor" => context.address().name().unwrap_or("unknown"));
        
        Ok(())
    }
}
```

### Restart Alerts

Monitor restart frequency:

```rust
async fn on_error<B: MessageBroker<Self::Message>>(
    &mut self,
    error: Self::Error,
    context: &mut ActorContext<Self::Message, B>,
) -> ErrorAction {
    let restart_count = context.lifecycle().restart_count();
    
    if restart_count > 5 {
        eprintln!("WARNING: Actor {} has restarted {} times", 
            context.id(), restart_count);
        
        // Alert monitoring system
        metrics::increment_counter!("actor_restart_threshold_exceeded",
            "actor" => context.address().name().unwrap_or("unknown"),
            "restart_count" => restart_count.to_string());
    }
    
    ErrorAction::Restart
}
```

## Testing Patterns

### Lifecycle Testing

```rust
#[tokio::test]
async fn test_actor_lifecycle() {
    let mut actor = MyActor::new();
    let broker = InMemoryMessageBroker::new();
    let mut context = ActorContext::new(
        ActorAddress::named("test"),
        broker,
    );
    
    // Test initialization
    actor.pre_start(&mut context).await.expect("pre_start failed");
    assert_eq!(actor.lifecycle.state(), ActorState::Running);
    
    // Test shutdown
    actor.post_stop(&mut context).await.expect("post_stop failed");
    assert_eq!(actor.lifecycle.state(), ActorState::Stopped);
}

#[tokio::test]
async fn test_error_handling() {
    let mut actor = MyActor::new();
    let broker = InMemoryMessageBroker::new();
    let mut context = ActorContext::new(
        ActorAddress::named("test"),
        broker,
    );
    
    // Trigger error
    let error = MyError::Temporary("test error".to_string());
    let action = actor.on_error(error, &mut context).await;
    
    // Verify error action
    assert_eq!(action, ErrorAction::Restart);
}
```

### State Transition Testing

```rust
#[test]
fn test_state_transitions() {
    let mut lifecycle = ActorLifecycle::new();
    assert_eq!(lifecycle.state(), ActorState::Starting);
    
    lifecycle.transition_to(ActorState::Running);
    assert_eq!(lifecycle.state(), ActorState::Running);
    assert!(lifecycle.is_running());
    
    lifecycle.transition_to(ActorState::Failed);
    assert_eq!(lifecycle.state(), ActorState::Failed);
    assert!(lifecycle.is_terminal());
    
    lifecycle.record_restart();
    assert_eq!(lifecycle.restart_count(), 1);
}
```

## Working Examples

Explore process lifecycle in these examples:

| Example | Demonstrates | Command |
|---------|--------------|---------|
| `actor_lifecycle.rs` | Complete lifecycle hooks | `cargo run --example actor_lifecycle` |
| `actor_basic.rs` | Basic lifecycle flow | `cargo run --example actor_basic` |
| `supervisor_basic.rs` | Restart lifecycle | `cargo run --example supervisor_basic` |
| `supervisor_strategies.rs` | Error actions and restart | `cargo run --example supervisor_strategies` |

All examples are in the `examples/` directory with complete, runnable implementations.
