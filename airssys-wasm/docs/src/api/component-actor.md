# ComponentActor API Reference

**Category**: Reference - Information-oriented  
**Audience**: Developers implementing ComponentActor-based systems  
**Purpose**: Technical specification of ComponentActor traits and methods

---

## Overview

ComponentActor is the core integration type bridging WebAssembly components with the airssys-rt actor system. It implements a dual-trait pattern:

- **Actor trait**: Message handling via mailbox (inter-component communication)
- **Child trait**: WASM runtime lifecycle management under supervisor control

This architecture enables isolated, sandboxed component execution with actor-based communication patterns.

## Architecture Diagram

```text
┌────────────────────────────────────┐
│      ComponentActor                │
│  ┌──────────────┐  ┌────────────┐ │
│  │ Actor trait  │  │ Child trait│ │
│  │ (messaging)  │  │ (lifecycle)│ │
│  └──────────────┘  └────────────┘ │
│         ↓                ↓         │
│  ┌──────────────────────────────┐ │
│  │   WasmRuntime (Optional)     │ │
│  │   - Wasmtime Engine/Store    │ │
│  │   - Component Exports        │ │
│  └──────────────────────────────┘ │
└────────────────────────────────────┘
```

## Struct: ComponentActor<S>

### Type Parameters

- `S`: Custom state type (default: `()` for no state)
  - Must implement: `Send + Sync + 'static`
  - State is protected by `Arc<RwLock<S>>` for concurrent access

### Fields

| Field | Type | Description |
|-------|------|-------------|
| `component_id` | `ComponentId` | Unique identifier for this component |
| `wasm_runtime` | `Option<WasmRuntime>` | WASM runtime (loaded in `Child::start()`) |
| `capabilities` | `CapabilitySet` | Security capabilities and permissions |
| `state` | `ActorState` | Current lifecycle state |
| `metadata` | `ComponentMetadata` | Component metadata (name, version, limits) |
| `mailbox_rx` | `Option<UnboundedReceiver<ComponentMessage>>` | Mailbox receiver (managed by ActorSystem) |
| `created_at` | `DateTime<Utc>` | Creation timestamp |
| `started_at` | `Option<DateTime<Utc>>` | Start timestamp (set in `Child::start()`) |
| `custom_state` | `Arc<RwLock<S>>` | Custom component state |

### Performance Characteristics

Measured in Task 6.2 (`task-004-phase-6-task-6.2-completion-report.md`):

- **Construction**: 286ns (Checkpoint 1, `component_actor_construction` benchmark)
- **Full lifecycle** (start+stop): 1.49µs (Checkpoint 1, `full_lifecycle_sequence` benchmark)
- **Memory footprint**: < 2MB per instance (target, not yet measured)

---

## Constructor

### `new()`

Create a new ComponentActor instance with custom state.

**Signature:**
```rust
pub fn new(
    component_id: ComponentId,
    metadata: ComponentMetadata,
    capabilities: CapabilitySet,
    initial_state: S,
) -> Self
```

**Parameters:**

- `component_id` - Unique identifier for this component
- `metadata` - Component metadata (name, version, resource limits)
- `capabilities` - Security capabilities and permissions
- `initial_state` - Initial value for custom state (generic type S)

**Returns:** ComponentActor in `ActorState::Creating` state

**Example:**
```rust
use airssys_wasm::actor::ComponentActor;
use airssys_wasm::core::{ComponentId, ComponentMetadata, CapabilitySet, ResourceLimits};

let component_id = ComponentId::new("my-component");
let metadata = ComponentMetadata {
    name: "my-component".to_string(),
    version: "1.0.0".to_string(),
    author: "Example Author".to_string(),
    description: None,
    required_capabilities: vec![],
    resource_limits: ResourceLimits {
        max_memory_bytes: 64 * 1024 * 1024,    // 64MB
        max_fuel: 1_000_000,                    // 1M fuel
        max_execution_ms: 5000,                 // 5s timeout
        max_storage_bytes: 10 * 1024 * 1024,   // 10MB storage
    },
};
let caps = CapabilitySet::new();

// No custom state (default: ())
let actor = ComponentActor::new(component_id, metadata, caps, ());
```

---

## State Management Methods

### `component_id()`

Get the component ID.

**Signature:**
```rust
pub fn component_id(&self) -> &ComponentId
```

**Returns:** Reference to the component ID

**Example:**
```rust
let id = actor.component_id();
println!("Component ID: {}", id.as_str());
```

---

### `state()`

Get the current actor lifecycle state.

**Signature:**
```rust
pub fn state(&self) -> &ActorState
```

**Returns:** Reference to the current `ActorState`

**Example:**
```rust
match *actor.state() {
    ActorState::Creating => println!("Component is being created"),
    ActorState::Ready => println!("Component is ready"),
    _ => println!("Component in other state"),
}
```

---

### `is_wasm_loaded()`

Check if WASM runtime is loaded.

**Signature:**
```rust
pub fn is_wasm_loaded(&self) -> bool
```

**Returns:** `true` if `Child::start()` has successfully loaded the WASM runtime

**Example:**
```rust
if actor.is_wasm_loaded() {
    println!("WASM runtime is loaded and ready");
}
```

---

### `uptime()`

Calculate component uptime.

**Signature:**
```rust
pub fn uptime(&self) -> Option<chrono::Duration>
```

**Returns:** Duration since `Child::start()` completed, or `None` if not started

**Example:**
```rust
if let Some(uptime) = actor.uptime() {
    println!("Component uptime: {} seconds", uptime.num_seconds());
}
```

---

## Custom State Methods

### `with_state()`

Execute a closure with read-only access to custom state.

**Signature:**
```rust
pub async fn with_state<F, R>(&self, f: F) -> R
where
    F: FnOnce(&S) -> R,
```

**Parameters:**

- `f` - Closure to execute with state reference

**Returns:** Value returned by the closure

**Example:**
```rust
#[derive(Default)]
struct MyState {
    count: u64,
}

let actor: ComponentActor<MyState> = ComponentActor::new(/* ... */, MyState::default());

// Read state
let count = actor.with_state(|state| state.count).await;
```

---

### `with_state_mut()`

Execute a closure with mutable access to custom state.

**Signature:**
```rust
pub async fn with_state_mut<F, R>(&self, f: F) -> R
where
    F: FnOnce(&mut S) -> R,
```

**Parameters:**

- `f` - Closure to execute with mutable state reference

**Returns:** Value returned by the closure

**Example:**
```rust
// Modify state
actor.with_state_mut(|state| {
    state.count += 1;
}).await;
```

---

### `get_state()`

Get a clone of the custom state (requires `S: Clone`).

**Signature:**
```rust
pub async fn get_state(&self) -> S
where
    S: Clone,
```

**Returns:** Cloned copy of the custom state

**Example:**
```rust
let state_copy = actor.get_state().await;
```

---

### `set_custom_state()`

Replace the custom state with a new value.

**Signature:**
```rust
pub async fn set_custom_state(&self, new_state: S) -> S
```

**Parameters:**

- `new_state` - New state value to set

**Returns:** The previous state value

**Example:**
```rust
let old_state = actor.set_custom_state(MyState { count: 42 }).await;
```

---

### `state_arc()`

Get Arc clone of the custom state RwLock.

**Signature:**
```rust
pub fn state_arc(&self) -> Arc<RwLock<S>>
```

**Returns:** Arc-wrapped RwLock of the custom state

**Example:**
```rust
let state_ref = actor.state_arc();

// Share Arc with other tasks
tokio::spawn(async move {
    let guard = state_ref.read().await;
    // Use state...
});
```

---

## Communication Methods

### `publish_message()`

Publish message to topic via MessageBroker.

**Signature:**
```rust
pub async fn publish_message(
    &self,
    topic: &str,
    message: ComponentMessage,
) -> Result<(), WasmError>
```

**Parameters:**

- `topic` - Topic name (e.g., "events", "notifications.user")
- `message` - ComponentMessage to publish

**Returns:**

- `Ok(())` - Message published successfully
- `Err(WasmError::BrokerNotConfigured)` - Broker not set
- `Err(WasmError::MessageBrokerError)` - Publish failed

**Example:**
```rust
use airssys_wasm::actor::ComponentMessage;

let message = ComponentMessage::InterComponent {
    sender: component_id.clone(),
    payload: vec![1, 2, 3],
};

actor.publish_message("events.user.login", message).await?;
```

---

### `subscribe_topic()`

Subscribe to topic for receiving messages.

**Signature:**
```rust
pub async fn subscribe_topic(
    &mut self,
    topic: &str,
) -> Result<SubscriptionHandle, WasmError>
```

**Parameters:**

- `topic` - Topic name to subscribe to

**Returns:**

- `Ok(SubscriptionHandle)` - Subscription successful
- `Err(WasmError::BrokerNotConfigured)` - Broker not set

**Example:**
```rust
let handle = actor.subscribe_topic("events.user").await?;
// Store handle for later unsubscribe
```

---

### `send_request()`

Send request with correlation tracking and timeout (Phase 5 Task 5.1).

**Signature:**
```rust
pub async fn send_request(
    &self,
    target: &ComponentId,
    payload: Vec<u8>,
    timeout: std::time::Duration,
) -> Result<tokio::sync::oneshot::Receiver<ResponseMessage>, WasmError>
```

**Parameters:**

- `target` - Target component ID
- `payload` - Request payload (multicodec-encoded)
- `timeout` - Timeout duration

**Returns:** Oneshot receiver for response

**Performance:** 3.18µs roundtrip (Task 6.2 Checkpoint 2, `request_response_roundtrip` benchmark)

**Example:**
```rust
use std::time::Duration;

let target = ComponentId::new("user-service");
let request = vec![/* multicodec-encoded data */];

let response_rx = actor.send_request(&target, request, Duration::from_secs(5)).await?;

match response_rx.await {
    Ok(response) => {
        match response.result {
            Ok(payload) => println!("Response received"),
            Err(e) => eprintln!("Request failed: {}", e),
        }
    }
    Err(_) => eprintln!("Response channel closed"),
}
```

---

### `send_response()`

Send response to correlated request.

**Signature:**
```rust
pub async fn send_response(
    &self,
    correlation_id: uuid::Uuid,
    result: Result<Vec<u8>, RequestError>,
) -> Result<(), WasmError>
```

**Parameters:**

- `correlation_id` - Correlation ID from incoming request
- `result` - Response payload or error

**Returns:**

- `Ok(())` - Response sent successfully
- `Err(WasmError)` - Correlation ID not found or tracker not configured

**Example:**
```rust
// Handle incoming request with correlation_id
let response_payload = vec![/* response data */];
actor.send_response(correlation_id, Ok(response_payload)).await?;
```

---

## Enum: ActorState

Actor lifecycle state machine.

**Variants:**

- `Creating` - Initial state after construction
- `Starting` - `Child::start()` in progress (loading WASM)
- `Ready` - WASM loaded, actor processing messages
- `Stopping` - `Child::stop()` in progress (cleanup)
- `Terminated` - Actor stopped, resources released
- `Failed(String)` - Unrecoverable error occurred

**State Transitions:**

```text
Creating --[Child::start()]--> Starting --[success]--> Ready
                                   ↓
                             Failed(reason)

Ready --[Child::stop()]--> Stopping --[success]--> Terminated
         ↓                       ↓
    Failed(reason)          Failed(reason)
```

**Example:**
```rust
use airssys_wasm::actor::ActorState;

let state = ActorState::Creating;
assert_eq!(state, ActorState::Creating);

let failed = ActorState::Failed("WASM load error".to_string());
match failed {
    ActorState::Failed(reason) => println!("Failed: {}", reason),
    _ => {}
}
```

---

## Enum: ComponentMessage

Message types for actor communication.

**Variants:**

- `Invoke { function: String, args: Vec<u8> }` - Call WASM function
- `InvokeResult { result: Vec<u8>, error: Option<String> }` - Function result
- `InterComponent { sender: ComponentId, payload: Vec<u8> }` - Inter-component message
- `InterComponentWithCorrelation { sender: ComponentId, payload: Vec<u8>, correlation_id: uuid::Uuid }` - Request-response message
- `Shutdown` - Signal to stop the actor
- `HealthCheck` - Request health status
- `HealthStatus(HealthStatus)` - Health status response

**Example:**
```rust
use airssys_wasm::actor::ComponentMessage;
use airssys_wasm::core::ComponentId;

// Invoke WASM function
let msg = ComponentMessage::Invoke {
    function: "process_data".to_string(),
    args: vec![1, 2, 3, 4], // Multicodec-encoded
};

// Inter-component message
let sender = ComponentId::new("sender-component");
let msg = ComponentMessage::InterComponent {
    sender,
    payload: vec![0x70, 0x01, 0x00, 0x01], // Multicodec Borsh
};

// Health check
let msg = ComponentMessage::HealthCheck;
```

---

## Enum: HealthStatus

Component health status for monitoring.

**Variants:**

- `Healthy` - Component operating normally
- `Degraded { reason: String }` - Operational but experiencing issues
- `Unhealthy { reason: String }` - Failed or non-functional

**Serialization:** Supports Borsh, CBOR, and JSON

**Example:**
```rust
use airssys_wasm::actor::HealthStatus;
use serde_json;

let status = HealthStatus::Degraded {
    reason: "High memory usage".to_string(),
};

// JSON serialization
let json = serde_json::to_string(&status).unwrap();
assert!(json.contains("degraded"));
```

---

## Related Documentation

- [Lifecycle Hooks API](./lifecycle-hooks.md) - Lifecycle hook methods and execution order
- [Tutorial: Your First ComponentActor](../tutorials/your-first-component-actor.md) - Step-by-step tutorial
- [Dual-Trait Design Explanation](../explanation/dual-trait-design.md) - Architecture rationale

## References

- **ADR-WASM-006**: Component Isolation and Sandboxing (Actor-based approach)
- **ADR-RT-004**: Actor and Child Trait Separation
- **Task 6.2 Performance Report**: `task-004-phase-6-task-6.2-completion-report.md`
