# Message Routing

**Category:** Reference (Information-Oriented)  
**Purpose:** Technical specification of the message routing system for ComponentActor communication.

## Overview

The message routing system provides efficient, type-safe message delivery between ComponentActor instances. It consists of three core components:

1. **ComponentRegistry:** O(1) component lookup by ComponentId
2. **MessageRouter:** High-level routing API with error handling
3. **MessageBroker:** Low-level message delivery (provided by airssys-rt)

## Architecture

```text
MessageRouter
    ↓
ComponentRegistry.lookup(component_id) → ActorAddress (O(1))
    ↓
MessageBroker.publish(message) → ComponentActor mailbox
    ↓
ComponentActor.handle_message(message)
```

## ComponentRegistry

### Purpose

`ComponentRegistry` provides thread-safe, O(1) lookup of component instances by `ComponentId`. It maps component identifiers to `ActorAddress` references for message delivery.

### Data Structure

```rust
pub struct ComponentRegistry {
    instances: Arc<RwLock<HashMap<ComponentId, ActorAddress>>>,
}
```

**Thread Safety:** Uses `Arc<RwLock<>>` for concurrent access:
- **Multiple readers:** Concurrent lookups allowed
- **Single writer:** Registration/unregistration requires exclusive lock
- **Lock poisoning:** Handled as internal error

### API Methods

#### `new() -> Self`

Creates a new empty registry.

**Complexity:** O(1)  
**Allocations:** Heap allocation for HashMap

**Example:**
```rust
let registry = ComponentRegistry::new();
```

#### `register(component_id: ComponentId, actor_addr: ActorAddress) -> Result<(), WasmError>`

Registers a component instance. Replaces existing registration if component ID already exists.

**Complexity:** O(1)  
**Errors:**
- `WasmError::Internal` if lock is poisoned

**Example:**
```rust
let id = ComponentId::new("component-1");
let addr = ActorAddress::named("actor-1");
registry.register(id, addr)?;
```

#### `lookup(component_id: &ComponentId) -> Result<ActorAddress, WasmError>`

Looks up ActorAddress by ComponentId.

**Complexity:** O(1)  
**Performance:** 36ns average (measured in Task 6.2 `scalability_benchmarks.rs` benchmark `bench_registry_lookup_scale`, constant from 10 to 1,000 components)  
**Errors:**
- `WasmError::ComponentNotFound` if component not registered
- `WasmError::Internal` if lock is poisoned

**Example:**
```rust
let addr = registry.lookup(&component_id)?;
```

#### `unregister(component_id: &ComponentId) -> Result<(), WasmError>`

Removes a component from the registry.

**Complexity:** O(1)  
**Errors:**
- `WasmError::ComponentNotFound` if component not registered
- `WasmError::Internal` if lock is poisoned

**Example:**
```rust
registry.unregister(&component_id)?;
```

#### `count() -> Result<usize, WasmError>`

Returns the number of registered components.

**Complexity:** O(1)  
**Errors:**
- `WasmError::Internal` if lock is poisoned

**Example:**
```rust
let count = registry.count()?;
println!("Registered components: {}", count);
```

#### `list_components() -> Result<Vec<ComponentId>, WasmError>`

Returns a list of all registered component IDs.

**Complexity:** O(n) where n is the number of components  
**Allocations:** New Vec with capacity n  
**Errors:**
- `WasmError::Internal` if lock is poisoned

**Example:**
```rust
let components = registry.list_components()?;
for id in components {
    println!("Component: {}", id.as_str());
}
```

### Performance Characteristics

Based on Task 6.2 benchmarks (`scalability_benchmarks.rs`):

| Operation | Latency | Scalability | Benchmark |
|-----------|---------|-------------|-----------|
| Lookup (10 components) | 37.5ns | O(1) | `bench_registry_lookup_scale` |
| Lookup (100 components) | 35.6ns | O(1) | `bench_registry_lookup_scale` |
| Lookup (1,000 components) | 36.5ns | O(1) | `bench_registry_lookup_scale` |
| Registration (batch 100) | <1ms | O(1) per | `bench_registry_registration_scale` |
| Concurrent lookup (10 threads) | <100µs | Lock-free reads | `bench_registry_concurrent_lookup` |

**Test Conditions:** macOS M1, 100 samples, 95% confidence interval

**Key Insight:** Lookup time remains constant (36ns ±5%) across all scales, confirming true O(1) performance via HashMap.

## MessageRouter

### Purpose

`MessageRouter` provides a high-level API for routing messages to components by ComponentId. It handles registry lookup and error cases gracefully.

### Data Structure

```rust
pub struct MessageRouter<B: MessageBroker<ComponentMessage>> {
    registry: ComponentRegistry,
    broker: Arc<B>,
}
```

**Type Parameter:** `B` is the MessageBroker implementation (typically `InMemoryMessageBroker` from airssys-rt)

### API Methods

#### `new(registry: ComponentRegistry, broker: Arc<B>) -> Self`

Creates a new MessageRouter.

**Complexity:** O(1)

**Example:**
```rust
let router = MessageRouter::new(registry, broker);
```

#### `send_message(target: &ComponentId, message: ComponentMessage) -> Result<(), WasmError>`

Sends a message to a component by ComponentId.

**Complexity:** O(1) lookup + O(1) publish  
**Performance:** ~1.05µs total (measured in Task 6.2 `messaging_benchmarks.rs` benchmark `bench_message_routing_overhead`)  
**Breakdown:**
- Registry lookup: ~36ns
- Message envelope creation: ~20ns
- Broker publish: ~211ns (airssys-rt baseline)
- ActorAddress routing: ~783ns

**Errors:**
- `WasmError::ComponentNotFound` if target not registered
- `WasmError::MessagingError` if broker publish fails

**Example:**
```rust
let message = ComponentMessage::HealthCheck;
router.send_message(&target_id, message).await?;
```

### Routing Decision Logic

```text
1. Lookup ActorAddress in ComponentRegistry
   ├─ Component found → Continue to step 2
   └─ Component not found → Return WasmError::ComponentNotFound

2. Create MessageEnvelope with reply_to address
   └─ Envelope wraps ComponentMessage

3. Publish via MessageBroker
   ├─ Broker enqueues message to ActorAddress mailbox
   ├─ Publish success → Return Ok(())
   └─ Publish fails → Return WasmError::MessagingError
```

## MessageBroker Integration

The MessageRouter delegates low-level message delivery to the `MessageBroker` trait from airssys-rt. This separation provides:

- **Flexibility:** Swap broker implementations (in-memory, distributed, persistent)
- **Performance:** Broker is optimized for high-throughput delivery
- **Testability:** Mock brokers for unit testing

### MessageBroker Trait

```rust
pub trait MessageBroker<M: Message> {
    async fn publish(&self, envelope: MessageEnvelope<M>) -> Result<(), BrokerError>;
    async fn subscribe(&self, topic: &str, address: ActorAddress) -> Result<(), BrokerError>;
    async fn unsubscribe(&self, topic: &str, address: &ActorAddress) -> Result<(), BrokerError>;
}
```

**Implementation:** `InMemoryMessageBroker` is provided by airssys-rt for single-node deployments.

## Error Handling

### Component Not Found

Occurs when target ComponentId is not registered.

**Error Type:** `WasmError::ComponentNotFound(ComponentId)`

**Common Causes:**
- Component never registered
- Component already unregistered
- Typo in ComponentId

**Handling:**
```rust
match router.send_message(&target, message).await {
    Ok(_) => {}
    Err(WasmError::ComponentNotFound(id)) => {
        log::warn!("Component {} not found", id.as_str());
        // Retry, queue, or drop message
    }
    Err(e) => {
        log::error!("Routing failed: {}", e);
    }
}
```

### Component Stopped

If a component is stopped but still registered, messages may fail delivery.

**Error Type:** `WasmError::MessagingError(String)`

**Handling:**
```rust
// Unregister component during shutdown
impl Drop for ComponentActor {
    fn drop(&mut self) {
        registry.unregister(&self.id).ok();
    }
}
```

### Lock Poisoning

If a thread panics while holding the registry lock, the lock becomes poisoned.

**Error Type:** `WasmError::Internal(String)`

**Mitigation:**
- Use panic-free code in registry operations
- Catch panics at thread boundaries
- Restart affected components

## Performance Tuning

### Registry Optimization

**Pre-allocation:** If component count is known, pre-allocate HashMap capacity:

```rust
// Custom registry with capacity hint
let mut map = HashMap::with_capacity(1000);
// Then wrap in Arc<RwLock<>>
```

**Read-heavy workloads:** RwLock favors readers, so registry performs well with many concurrent lookups.

### Router Optimization

**Clone Router:** Router is Clone and cheap (Arc clone). Share one router across components:

```rust
// Create once
let router = MessageRouter::new(registry, broker);

// Share across components
let router_clone = router.clone();
```

**Batch Messages:** If sending multiple messages, batch them to amortize overhead:

```rust
// Good: batch send
for target in targets {
    router.send_message(target, message.clone()).await?;
}

// Avoid: awaiting each send sequentially if order doesn't matter
```

## Scalability Limits

Based on Task 6.2 benchmarks:

| Limit | Value | Notes |
|-------|-------|-------|
| Max components per registry | 1,000+ | O(1) lookup validated up to 1,000 |
| Concurrent lookups | Unlimited | RwLock allows parallel reads |
| Message throughput | 6.12 million msg/sec | Measured in `bench_sustained_message_throughput` |
| Registry memory overhead | ~24 bytes per component | HashMap entry size (ComponentId + ActorAddress) |

**Production Recommendation:** Registries with < 10,000 components perform excellently on single node.

## Implementation References

### Source Files

- **ComponentRegistry:** `airssys-wasm/src/actor/component/component_registry.rs`
- **MessageRouter:** `airssys-wasm/src/actor/message/message_router.rs`
- **MessageBroker:** `airssys-rt/src/broker/in_memory.rs`

### Benchmarks

- **Registry scalability:** `airssys-wasm/benches/scalability_benchmarks.rs`
- **Message routing:** `airssys-wasm/benches/messaging_benchmarks.rs`

### ADRs

- **ADR-WASM-006:** Component Isolation and Sandboxing (actor-based isolation)
- **ADR-WASM-009:** Inter-Component Communication (message routing < 500ns target)
- **ADR-WASM-018:** Layer Separation (ComponentActor boundary)

## Related Documentation

- **How-To Guide:** [Request-Response Pattern](../guides/request-response-pattern.md)
- **How-To Guide:** [Pub-Sub Broadcasting](../guides/pubsub-broadcasting.md)
- **API Reference:** [ComponentActor API](../api/component-actor.md)
- **Explanation:** [State Management Patterns](../explanation/state-management-patterns.md)
