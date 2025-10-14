# Core Types

This section documents the core types used throughout `airssys-rt`.

## ActorAddress

Unique identifier and optional name for actors.

```rust
pub struct ActorAddress {
    id: ActorId,
    name: Option<String>,
}
```

### Methods

```rust
impl ActorAddress {
    /// Create an anonymous address (UUID-based)
    pub fn anonymous() -> Self;
    
    /// Create a named address
    pub fn named(name: impl Into<String>) -> Self;
    
    /// Get the actor's ID
    pub fn id(&self) -> &ActorId;
    
    /// Get the actor's name (if any)
    pub fn name(&self) -> Option<&str>;
}
```

### Example

```rust
use airssys_rt::util::ActorAddress;

// Anonymous actor
let addr1 = ActorAddress::anonymous();

// Named actor
let addr2 = ActorAddress::named("worker-1");
assert_eq!(addr2.name(), Some("worker-1"));
```

## ActorId

Unique identifier for actors using UUIDs.

```rust
pub struct ActorId(uuid::Uuid);
```

### Methods

```rust
impl ActorId {
    /// Generate a new unique actor ID
    pub fn new() -> Self;
}
```

### Traits

Implements: `Clone`, `Copy`, `Debug`, `PartialEq`, `Eq`, `Hash`

## ActorState

Lifecycle states for actors.

```rust
pub enum ActorState {
    Starting,   // Actor is initializing
    Running,    // Actor is active and processing messages
    Stopping,   // Actor is shutting down
    Stopped,    // Actor has stopped successfully
    Failed,     // Actor has failed (requires supervision)
}
```

### Example

```rust
use airssys_rt::ActorState;

let state = ActorState::Running;
assert_eq!(state, ActorState::Running);
```

## ActorLifecycle

Tracks actor lifecycle state and transitions.

```rust
pub struct ActorLifecycle {
    state: ActorState,
    last_state_change: DateTime<Utc>,
    restart_count: u32,
}
```

### Methods

```rust
impl ActorLifecycle {
    /// Create new lifecycle tracker
    pub fn new() -> Self;
    
    /// Get current state
    pub fn state(&self) -> ActorState;
    
    /// Transition to new state
    pub fn transition_to(&mut self, new_state: ActorState);
    
    /// Get restart count
    pub fn restart_count(&self) -> u32;
    
    /// Get last state change timestamp
    pub fn last_state_change(&self) -> DateTime<Utc>;
    
    /// Check if in terminal state (Stopped or Failed)
    pub fn is_terminal(&self) -> bool;
    
    /// Check if currently running
    pub fn is_running(&self) -> bool;
}
```

### Example

```rust
use airssys_rt::{ActorLifecycle, ActorState};

let mut lifecycle = ActorLifecycle::new();
assert_eq!(lifecycle.state(), ActorState::Starting);

lifecycle.transition_to(ActorState::Running);
assert!(lifecycle.is_running());
```

## ErrorAction

Control supervision behavior when actor errors occur.

```rust
pub enum ErrorAction {
    Resume,     // Continue processing (ignore error)
    Restart,    // Restart the actor
    Stop,       // Stop the actor permanently
    Escalate,   // Pass error to supervisor
}
```

### Example

```rust
use airssys_rt::ErrorAction;

async fn on_error<B: MessageBroker<Self::Message>>(
    &mut self,
    error: Self::Error,
    context: &mut ActorContext<Self::Message, B>,
) -> ErrorAction {
    match error {
        MyError::Temporary => ErrorAction::Resume,
        MyError::Recoverable => ErrorAction::Restart,
        MyError::Fatal => ErrorAction::Stop,
        _ => ErrorAction::Escalate,
    }
}
```

## MessageId

Unique identifier for messages using UUIDs.

```rust
pub struct MessageId(uuid::Uuid);
```

### Methods

```rust
impl MessageId {
    /// Generate a new unique message ID
    pub fn new() -> Self;
}
```

## ChildId

Unique identifier for supervised children.

```rust
pub struct ChildId(uuid::Uuid);
```

### Methods

```rust
impl ChildId {
    /// Generate a new unique child ID
    pub fn new() -> Self;
}
```

## SupervisorId

Unique identifier for supervisors.

```rust
pub struct SupervisorId(uuid::Uuid);
```

### Methods

```rust
impl SupervisorId {
    /// Generate a new unique supervisor ID
    pub fn new() -> Self;
}
```

All core types are defined in the respective modules and exported through the prelude for convenient access.