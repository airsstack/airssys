# ADR-RT-009: OSL Broker Dependency Injection

**Status**: Accepted  
**Date**: 2025-10-14  
**Author**: AI Agent with User Input  
**Related Tasks**: RT-TASK-009 Phase 2, Task 2.1  
**Related ADRs**: ADR-RT-001 (Zero-Cost Abstractions), ADR-RT-002 (Message Passing)

## Context

During implementation of RT-TASK-009 Phase 2 Task 2.1 (OSLSupervisor Module), we identified a critical architectural gap: OSL actors (FileSystemActor, ProcessActor, NetworkActor) were implemented without any message broker integration, making them unable to communicate with application actors or handle message-based operations.

### Current Implementation Problems

1. **No Broker Reference**: OSL actors have no `MessageBroker` instance
2. **Communication Gap**: Actors cannot send/receive messages
3. **Hard Coupling**: If broker was added, it would be hardcoded to `InMemoryMessageBroker`
4. **Untestable**: Cannot inject mock brokers for testing
5. **Violates Microsoft Rust Guidelines**: Concrete type instead of trait bounds (§M-DI-HIERARCHY)

### Code Example - Current Problem

```rust
// Current implementation (BEFORE)
pub struct FileSystemActor {
    // No broker - cannot communicate!
}

pub struct OSLSupervisor {
    // No broker injection - actors are isolated!
    supervisor_fs: Arc<Mutex<SupervisorNode<RestForOne, FileSystemActor, ...>>>,
}
```

## Decision

**We will refactor OSLSupervisor and OSL actors to accept MessageBroker injection via generic constraints.**

This aligns with:
- **Microsoft Rust Guidelines §M-DI-HIERARCHY**: Prefer generic constraints over concrete types
- **Microsoft Rust Guidelines §M-SERVICES-CLONE**: Services implement cheap Clone via Arc pattern
- **YAGNI Principles §6.1**: We DO need broker for message passing (not speculative)
- **Testability Requirements**: Enable mock brokers for testing

## Proposed Architecture

### 1. Generic OSL Actors

```rust
pub struct FileSystemActor<M: Message, B: MessageBroker<M>> {
    broker: B,
    state: HashMap<String, Vec<u8>>,
    _phantom: PhantomData<M>,
}

impl<M, B> FileSystemActor<M, B>
where
    M: Message + serde::Serialize + serde::Deserialize,
    B: MessageBroker<M> + Clone,
{
    pub fn new(broker: B) -> Self {
        Self {
            broker,
            state: HashMap::new(),
            _phantom: PhantomData,
        }
    }
}
```

### 2. Generic OSLSupervisor

```rust
pub struct OSLSupervisor<M, B>
where
    M: Message,
    B: MessageBroker<M>,
{
    broker: B,
    supervisor_fs: Arc<Mutex<SupervisorNode<RestForOne, FileSystemActor<M, B>, ...>>>,
    supervisor_proc: Arc<Mutex<SupervisorNode<RestForOne, ProcessActor<M, B>, ...>>>,
    supervisor_net: Arc<Mutex<SupervisorNode<RestForOne, NetworkActor<M, B>, ...>>>,
    // ...
}

impl<M, B> OSLSupervisor<M, B>
where
    M: Message + serde::Serialize + serde::Deserialize + 'static,
    B: MessageBroker<M> + Clone + Send + Sync + 'static,
{
    pub fn new(broker: B) -> Self {
        // Broker injected and cloned to each actor
    }
}
```

### 3. Factory Function with Broker Closure

```rust
pub async fn start(&self) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    // Capture broker in closure for ChildSpec factory
    let mut sup = self.supervisor_fs.lock().await;
    let broker = self.broker.clone();
    
    let spec = ChildSpec {
        id: "filesystem".to_string(),
        factory: move || FileSystemActor::new(broker.clone()),
        restart_policy: RestartPolicy::Permanent,
        shutdown_policy: ShutdownPolicy::Graceful(Duration::from_secs(5)),
        start_timeout: Duration::from_secs(10),
        shutdown_timeout: Duration::from_secs(10),
    };
    
    sup.start_child(spec).await?;
}
```

### 4. Usage Example

```rust
// Production: Use InMemoryMessageBroker
let broker = InMemoryMessageBroker::new();
let osl_supervisor = OSLSupervisor::new(broker);
osl_supervisor.start().await?;

// Testing: Use mock broker
let mock_broker = MockBroker::new();
let osl_supervisor = OSLSupervisor::new(mock_broker);
```

## Consequences

### Positive

1. **Loose Coupling**: OSLSupervisor depends on trait, not concrete implementation
2. **Testability**: Can inject mock brokers for comprehensive testing
3. **Flexibility**: Support different broker implementations (in-memory, distributed, etc.)
4. **Standards Compliance**: Follows Microsoft Rust Guidelines M-DI-HIERARCHY
5. **Type Safety**: Compile-time guarantees via generic constraints
6. **Zero-Cost**: Generic monomorphization, no runtime overhead
7. **Clean Architecture**: Dependency inversion principle applied correctly

### Negative

1. **Generic Complexity**: OSLSupervisor becomes generic over `<M, B>`
2. **Type Propagation**: Generics propagate to parent supervisors/systems
3. **Verbosity**: Type signatures become longer
4. **Closure Complexity**: Factory functions need to capture broker in closures
5. **Learning Curve**: More complex for beginners to understand

### Neutral

1. **Breaking Change**: Existing tests need to be updated with broker injection
2. **Documentation**: More examples needed to show broker injection patterns
3. **Migration**: Need to update all OSL actor implementations

## Implementation Plan

### Phase 1: Refactor OSL Actors (Immediate)
1. Make `FileSystemActor<M, B>` generic over broker
2. Make `ProcessActor<M, B>` generic over broker
3. Make `NetworkActor<M, B>` generic over broker
4. Add `broker: B` field to each actor struct
5. Update constructors to accept `broker` parameter

### Phase 2: Refactor OSLSupervisor (Immediate)
1. Make `OSLSupervisor<M, B>` generic
2. Add `broker: B` field
3. Update constructor to accept broker
4. Update `start()` method with broker-capturing factories
5. Update trait bounds for all generic constraints

### Phase 3: Update Tests (Immediate)
1. Update unit tests with `InMemoryMessageBroker::new()`
2. Update doc examples with broker injection
3. Verify all 500 tests still pass

### Phase 4: Update Documentation (Task 2.4)
1. Update module-level documentation
2. Add broker injection examples
3. Document testability benefits
4. Update architecture diagrams

## Alternatives Considered

### Alternative 1: Hardcode InMemoryMessageBroker
**Rejected**: Violates dependency inversion, untestable, tight coupling

### Alternative 2: Use Box<dyn MessageBroker>
**Rejected**: Violates §M-DI-HIERARCHY and §M-AVOID-WRAPPERS, runtime overhead

### Alternative 3: Global Broker Singleton
**Rejected**: Untestable, hidden dependencies, violates clean architecture

### Alternative 4: No Broker (Current State)
**Rejected**: Actors cannot communicate, defeats purpose of actor system

## References

### Microsoft Rust Guidelines
- **M-DI-HIERARCHY**: Concrete types > generics > dyn traits
- **M-AVOID-WRAPPERS**: No smart pointers in public APIs
- **M-SERVICES-CLONE**: Services implement cheap Clone via Arc<Inner>

### Workspace Standards
- **§6.1 YAGNI Principles**: Build only what's needed (broker IS needed)
- **§6.2 Avoid dyn Patterns**: Prefer generic constraints

### Related Files
- `airssys-rt/src/osl/supervisor.rs` (496 lines) - To be refactored
- `airssys-rt/src/osl/actors/filesystem.rs` - To be refactored
- `airssys-rt/src/osl/actors/process.rs` - To be refactored
- `airssys-rt/src/osl/actors/network.rs` - To be refactored
- `airssys-rt/src/broker/traits.rs` - MessageBroker trait definition

## Notes

This ADR was created during RT-TASK-009 Phase 2 implementation after identifying the architectural gap. The user raised this concern before proceeding to Task 2.2 (Example Application), demonstrating excellent architectural awareness.

This decision is critical for the success of Phase 2 deliverables:
- Task 2.2: Example application needs working message passing
- Task 2.3: Integration tests need broker injection for testing
- Task 2.4: Documentation needs to explain broker injection patterns

**Status**: Accepted - Proceeding with refactoring immediately.
