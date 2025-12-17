# WASM Component Framework Architecture

This document describes the architecture of the WASM Component Framework, focusing on the ComponentActor system, its integration with the actor runtime, and key design patterns.

## System Overview

The WASM Component Framework provides a runtime deployment model for WebAssembly components with actor-based concurrency. The system is built on three foundational layers:

```
┌─────────────────────────────────────────────┐
│         ComponentActor Layer                │  ← Lifecycle + Messaging
│  (Dual-trait: Child + Actor)                │
├─────────────────────────────────────────────┤
│         ActorSystem Layer                   │  ← Spawning + Routing
│  (airssys-rt integration)                   │
├─────────────────────────────────────────────┤
│         SupervisorNode Layer                │  ← Fault Tolerance
│  (Automatic restart + recovery)             │
└─────────────────────────────────────────────┘
```

**Key Components:**

- **ComponentActor**: Dual-trait pattern combining WASM lifecycle (Child) with actor messaging (Actor)
- **ActorSystem**: Runtime environment for spawning and managing components
- **ComponentRegistry**: O(1) component lookup and discovery
- **SupervisorNode**: Automatic crash recovery with configurable restart strategies
- **MessageRouter**: Low-latency message routing between components

## ComponentActor Architecture

### Dual-Trait Pattern

ComponentActor uses a dual-trait pattern to separate lifecycle management from message handling:

```rust
// Trait 1: Child (Lifecycle management)
pub trait Child {
    fn pre_start(&mut self, context: &ChildContext) -> Result<(), ChildError>;
    fn post_start(&mut self, context: &ChildContext) -> Result<(), ChildError>;
    fn pre_stop(&mut self, context: &ChildContext) -> Result<(), ChildError>;
    fn post_stop(&mut self, context: &ChildContext) -> Result<(), ChildError>;
}

// Trait 2: Actor (Message handling)
#[async_trait]
pub trait Actor: Send + 'static {
    type Message: Send + 'static;
    type Error: Send + 'static;
    
    async fn handle_message(
        &mut self,
        message: Self::Message,
        context: &ActorContext,
    ) -> Result<(), Self::Error>;
}

// ComponentActor implements both traits
#[derive(Clone)]
pub struct MyComponent {
    state: Arc<RwLock<ComponentState>>,
}

impl Child for MyComponent {
    // Lifecycle hooks
}

#[async_trait]
impl Actor for MyComponent {
    // Message handling
}
```

**Design Rationale:**

1. **Separation of Concerns**: Lifecycle (Child) and messaging (Actor) are independent
2. **Testability**: Can test lifecycle independently from message handling
3. **Reusability**: Child trait usable outside actor context
4. **Clarity**: Clear distinction between initialization and runtime behavior

**Architecture Diagram:**

```
ComponentActor (Dual-Trait)
    │
    ├─── Child Trait (Lifecycle)
    │    ├─ pre_start()   → Initialize component
    │    ├─ post_start()  → Finalize initialization
    │    ├─ pre_stop()    → Prepare for shutdown
    │    └─ post_stop()   → Cleanup resources
    │
    └─── Actor Trait (Messaging)
         └─ handle_message() → Process messages
```

**Performance Characteristics** (Task 6.2 benchmarks):
- Component construction: 286ns
- Full lifecycle (pre_start → post_start → pre_stop → post_stop): 1.49µs
- Message handling: 1.05µs per message

### Lifecycle Execution Order

```
1. Component created (new())
   ↓
2. pre_start() called
   │  - Initialize state
   │  - Load configuration
   │  - Validate capabilities
   ↓
3. Component spawned (ActorSystem::spawn)
   ↓
4. post_start() called
   │  - Finalize initialization
   │  - Register with registry
   │  - Begin processing messages
   ↓
5. handle_message() called (for each message)
   │  - Process message
   │  - Update state
   │  - Send responses
   ↓
6. pre_stop() called (on shutdown)
   │  - Stop accepting new messages
   │  - Persist state
   │  - Notify dependent components
   ↓
7. post_stop() called
   │  - Release resources
   │  - Close file handles
   │  - Unregister from registry
   ↓
8. Component dropped
```

**Timing** (Task 6.2 actor_lifecycle_benchmarks.rs):
- Steps 1-4 (startup): ~750ns
- Steps 6-8 (shutdown): ~740ns
- Total lifecycle: 1.49µs

## Integration with ActorSystem

### Component Spawning

```rust
use airssys_rt::prelude::*;
use airssys_wasm::actor::ComponentActor;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Create ActorSystem
    let actor_system = ActorSystem::new("component-system").await?;
    
    // Create component (implements Child + Actor)
    let component = MyComponent::new();
    
    // Spawn component (calls pre_start, post_start)
    let component_id = actor_system.spawn_component(component).await?;
    
    // Send messages (calls handle_message)
    component_id.send(MyMessage::Process("data".to_string())).await?;
    
    // Stop component (calls pre_stop, post_stop)
    actor_system.stop_component(component_id).await?;
    
    Ok(())
}
```

**Integration Flow:**

```
ActorSystem::spawn_component(component)
    ↓
1. Allocate component ID
    ↓
2. Call component.pre_start()
    ↓
3. Spawn actor in tokio runtime
    ↓
4. Call component.post_start()
    ↓
5. Register in ComponentRegistry
    ↓
6. Begin message processing
```

**Performance** (Task 6.2):
- Spawn component: 286ns (measured in actor_lifecycle_benchmarks.rs)
- Spawn rate: 2.65 million components/sec capacity

### Message Routing

```
Sender Component
    │
    │ send_message(target_id, message)
    ↓
MessageRouter
    │
    │ lookup(target_id) → ComponentRegistry (36ns O(1) lookup)
    ↓
Target Component
    │
    │ handle_message(message)
    ↓
Processing + Response
```

**Message Flow:**

1. **Sender** calls `context.send_message(target_id, message)`
2. **MessageRouter** looks up target in ComponentRegistry (36ns)
3. **Target** receives message in `handle_message()`
4. **Target** processes and optionally sends response

**Performance** (Task 6.2 messaging_benchmarks.rs):
- Message routing: 1.05µs per message
- Throughput: 6.12 million messages/sec system-wide
- Registry lookup: 36ns O(1) (constant from 10-1,000 components)

### Supervision Integration

```
SupervisorNode
    │
    │ spawn_child(component)
    ↓
ActorSystem::spawn_component(component)
    ↓
ComponentActor (monitored)
    │
    │ Crash detected
    ↓
SupervisorNode
    │
    │ restart_child(component_id)
    ↓
ActorSystem::spawn_component(component) [restart]
```

**Supervisor Configuration:**

```rust
use airssys_rt::supervisor::{SupervisorNode, SupervisorConfig, RestartStrategy};
use std::time::Duration;

let config = SupervisorConfig {
    max_restarts: 5,
    within_duration: Duration::from_secs(60),
    restart_strategy: RestartStrategy::ExponentialBackoff {
        initial_delay: Duration::from_secs(1),
        max_delay: Duration::from_secs(30),
        multiplier: 2.0,
    },
};

let supervisor = SupervisorNode::new(config);
let supervisor_ref = actor_system.spawn_actor(supervisor).await?;

// Spawn component under supervision
let component = MyComponent::new();
let component_ref = supervisor_ref
    .send(SupervisorMessage::SpawnChild(Box::new(component)))
    .await?;
```

**Failure Recovery:**

1. Component crashes (panic or error)
2. Supervisor detects crash
3. Supervisor waits for restart delay (exponential backoff)
4. Supervisor respawns component (calls pre_start, post_start)
5. Component resumes operation

**Performance Impact:**

- Restart overhead: 1.49µs (full lifecycle)
- Exponential backoff delays: 1s, 2s, 4s, 8s, 16s, 30s (capped)

## Component Registry Architecture

### O(1) Lookup Design

```rust
use dashmap::DashMap;
use std::sync::Arc;

pub struct ComponentRegistry {
    components: Arc<DashMap<ComponentId, ComponentInstance>>,
}

impl ComponentRegistry {
    pub fn register(&self, id: ComponentId, instance: ComponentInstance) {
        self.components.insert(id, instance);
    }
    
    pub fn lookup(&self, id: &ComponentId) -> Option<ComponentInstance> {
        self.components.get(id).map(|entry| entry.value().clone())
    }
    
    pub fn unregister(&self, id: &ComponentId) -> Option<ComponentInstance> {
        self.components.remove(id).map(|(_, instance)| instance)
    }
}
```

**Design Choices:**

1. **DashMap**: Lock-free concurrent HashMap
   - Concurrent reads without blocking (multiple threads can read simultaneously)
   - Concurrent writes with minimal locking (fine-grained locking per shard)
   - O(1) average-case lookup, insert, remove

2. **ComponentId**: Unique identifier (UUID or incremental)
   - Immutable (never changes after creation)
   - Globally unique (no collisions)
   - Efficient hash key (u64 or [u8; 16])

**Performance Validation** (Task 6.2 scalability_benchmarks.rs):
- 10 components: 37.5ns lookup
- 100 components: 35.6ns lookup (5% faster)
- 1,000 components: 36.5ns lookup (3% slower)
- **Conclusion**: Perfect O(1) behavior validated

### Registry Operations

```rust
// Register component (called in post_start)
registry.register(component_id, component_instance);

// Lookup component (called during message routing)
if let Some(instance) = registry.lookup(&component_id) {
    instance.send(message).await?;
}

// List all components (used for shutdown, monitoring)
let all_components: Vec<ComponentId> = registry.list_all().collect();

// Unregister component (called in post_stop)
registry.unregister(&component_id);
```

**Thread Safety:**

- All operations are thread-safe (concurrent access safe)
- No global locks (DashMap uses sharded locking)
- Wait-free reads (readers don't block each other)

## Layer Boundaries (ADR-WASM-018)

### Layered Architecture

```
Layer 4: Application Code
   ↓ (uses ComponentActor API)
Layer 3: ComponentActor
   ↓ (uses ActorSystem API)
Layer 2: ActorSystem
   ↓ (uses Tokio API)
Layer 1: Tokio Runtime
```

**Boundary Rules:**

1. **Layer N can only depend on Layer N-1**
   - ComponentActor depends on ActorSystem ✅
   - ComponentActor depends directly on Tokio ❌

2. **Abstraction Principle**
   - Each layer exposes clean API (hides implementation)
   - Application code sees ComponentActor API, not ActorSystem internals

3. **Testability Principle**
   - Each layer testable independently
   - Mock lower layers for unit testing

**ADR-WASM-018 Compliance:**

| Layer | Responsibility | Dependencies | API Surface |
|-------|----------------|--------------|-------------|
| ComponentActor | Lifecycle + Messaging | ActorSystem | Child + Actor traits |
| ActorSystem | Actor spawning + routing | Tokio | spawn(), send(), stop() |
| Tokio | Task execution | OS threads | spawn(), sleep(), select!() |

**Benefits:**

- Clear responsibilities (no overlap)
- Easy to test (mock one layer at a time)
- Easy to swap implementations (e.g., replace ActorSystem)
- Prevents circular dependencies (enforced by compiler)

## Performance Characteristics

### Lifecycle Performance (Task 6.2 actor_lifecycle_benchmarks.rs)

| Operation | Performance | Throughput |
|-----------|-------------|------------|
| Component construction | 286ns | 2.65M/sec |
| Full lifecycle (start+stop) | 1.49µs | 671k/sec |
| pre_start hook | ~188ns | - |
| post_start hook | ~187ns | - |
| pre_stop hook | ~185ns | - |
| post_stop hook | ~180ns | - |

**Test Conditions:** macOS M1, 100 samples, 95% confidence interval, Criterion framework

### Messaging Performance (Task 6.2 messaging_benchmarks.rs)

| Operation | Performance | Throughput |
|-----------|-------------|------------|
| Message routing | 1.05µs | 952k msg/sec (per component) |
| Request-response cycle | 3.18µs | 314k req/sec |
| Message throughput (system) | - | 6.12M msg/sec |
| Pub-sub fanout (100 subscribers) | 85.2µs | 11,737 fanouts/sec |

**Test Conditions:** macOS M1, 100 samples, 95% confidence interval

### Registry Performance (Task 6.2 scalability_benchmarks.rs)

| Operation | Performance | Scaling |
|-----------|-------------|---------|
| Registry lookup | 36ns | O(1) constant |
| Component registration | ~1.03µs | Linear |
| Component spawn rate | - | 2.65M/sec |
| Concurrent ops (100) | 120µs | 833k ops/sec |

**O(1) Validation:**

- 10 components: 37.5ns
- 100 components: 35.6ns (5% faster)
- 1,000 components: 36.5ns (3% slower)
- **Conclusion**: Perfect O(1) behavior

## State Management Architecture

### Component State Pattern

```rust
use std::sync::Arc;
use tokio::sync::RwLock;

#[derive(Clone)]
pub struct MyComponent {
    // State wrapped in Arc<RwLock<T>>
    state: Arc<RwLock<ComponentState>>,
}

#[derive(Debug)]
struct ComponentState {
    counter: u64,
    data: String,
}

impl MyComponent {
    pub fn new() -> Self {
        Self {
            state: Arc::new(RwLock::new(ComponentState {
                counter: 0,
                data: String::new(),
            })),
        }
    }
}

#[async_trait]
impl Actor for MyComponent {
    type Message = MyMessage;
    type Error = ComponentError;
    
    async fn handle_message(
        &mut self,
        message: Self::Message,
        _context: &ActorContext,
    ) -> Result<(), Self::Error> {
        match message {
            MyMessage::Increment => {
                let mut state = self.state.write().await;  // Acquire write lock
                state.counter += 1;
                // Lock released when `state` goes out of scope
            }
            MyMessage::Get => {
                let state = self.state.read().await;  // Acquire read lock
                println!("Counter: {}", state.counter);
                // Lock released
            }
        }
        Ok(())
    }
}
```

**Design Rationale:**

1. **Arc<T>**: Shared ownership (component cloned for each Actor instance)
2. **RwLock<T>**: Reader-writer lock (multiple readers, single writer)
3. **Async-aware**: tokio::sync::RwLock (doesn't block executor)

**Performance** (Task 6.2 actor_lifecycle_benchmarks.rs):
- State read access: 37ns
- State write access: 39ns
- Low contention: <5% variance across 5 runs

**Best Practices:**

- Minimize lock duration (hold lock briefly, release quickly)
- Avoid holding locks across await points (causes contention)
- Clone data out of lock before expensive operations

### Stateless vs Stateful Components

**Stateless Component (Preferred):**

```rust
#[derive(Clone)]
pub struct StatelessParser {
    // No internal state
}

impl StatelessParser {
    pub fn parse(&self, input: &str) -> Result<ParsedData, ParseError> {
        // Pure function - no state mutation
    }
}
```

**Benefits:**

- No lock contention (no shared state)
- Trivially scalable (spawn multiple instances)
- Easy to test (no setup/teardown)
- Fast (no lock overhead)

**Stateful Component (When Necessary):**

```rust
#[derive(Clone)]
pub struct StatefulAccumulator {
    state: Arc<RwLock<AccumulatorState>>,  // Shared state
}
```

**Use When:**

- Caching required (avoid recomputing)
- Aggregation needed (sum, count, average)
- Session management (user sessions, connections)

## Error Handling Architecture

### Error Propagation Pattern

```rust
use thiserror::Error;

#[derive(Debug, Error)]
pub enum ComponentError {
    #[error("Lifecycle error: {0}")]
    Lifecycle(String),
    
    #[error("Message handling error: {0}")]
    MessageHandling(String),
    
    #[error("State access error: {0}")]
    StateAccess(String),
}

#[async_trait]
impl Actor for MyComponent {
    type Error = ComponentError;
    
    async fn handle_message(
        &mut self,
        message: Self::Message,
        context: &ActorContext,
    ) -> Result<(), Self::Error> {
        // Propagate errors with ?
        let data = self.fetch_data().await
            .map_err(|e| ComponentError::MessageHandling(e.to_string()))?;
        
        // Update state or propagate error
        self.update_state(data).await
            .map_err(|e| ComponentError::StateAccess(e.to_string()))?;
        
        Ok(())
    }
}
```

**Error Handling Strategy:**

1. **Propagate with ?**: Use Result<T, E> and ? operator
2. **Log at boundaries**: Log errors when crossing layer boundaries
3. **Structured errors**: Use thiserror for type-safe error variants
4. **Context**: Include contextual information (component ID, operation)

### Supervision and Error Recovery

```
Component error propagates
    ↓
Supervisor detects failure
    ↓
Supervisor decides action:
    - Restart component (crash recovery)
    - Escalate to parent supervisor
    - Ignore (log and continue)
```

**Restart Triggers:**

- Component panic (unhandled panic in handle_message)
- Repeated errors (error rate > threshold)
- Health check failure (component not responding)

## Summary

ComponentActor architecture provides:

1. **Dual-Trait Pattern**: Separation of lifecycle (Child) and messaging (Actor)
2. **ActorSystem Integration**: Spawning, routing, and supervision via airssys-rt
3. **O(1) Registry**: Constant-time component lookup (36ns)
4. **High Performance**: 286ns spawn, 6.12M msg/sec throughput
5. **Fault Tolerance**: Automatic crash recovery via SupervisorNode
6. **Layer Boundaries**: Clean separation per ADR-WASM-018

**Architecture Diagram:**

```
┌────────────────────────────────────────────────┐
│  ComponentActor (Dual-Trait)                   │
│    ├─ Child (Lifecycle: pre/post start/stop)   │
│    └─ Actor (Messaging: handle_message)        │
├────────────────────────────────────────────────┤
│  ActorSystem Integration                       │
│    ├─ spawn_component() → 286ns               │
│    ├─ ComponentRegistry → 36ns O(1) lookup    │
│    └─ MessageRouter → 1.05µs routing          │
├────────────────────────────────────────────────┤
│  SupervisorNode (Fault Tolerance)              │
│    ├─ Restart strategies (immediate/delayed)   │
│    ├─ Exponential backoff (1s → 30s)          │
│    └─ Health monitoring (future)               │
└────────────────────────────────────────────────┘
```

**Performance Baseline** (Task 6.2):
- Component spawn: 286ns (2.65M/sec capacity)
- Message throughput: 6.12M msg/sec system-wide
- Registry lookup: 36ns O(1) (constant 10-1,000 components)
- Full lifecycle: 1.49µs (start+stop)

## Next Steps

- [ComponentActor API Reference](api/component-actor.md) - Complete API documentation
- [Lifecycle Hooks Reference](api/lifecycle-hooks.md) - Hook execution order and usage
- [Dual-Trait Design Explanation](explanation/dual-trait-design.md) - Design rationale
- [Performance Characteristics](reference/performance-characteristics.md) - Complete benchmark results
