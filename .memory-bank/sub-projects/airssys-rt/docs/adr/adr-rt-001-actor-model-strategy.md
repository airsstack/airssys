# ADR-RT-001: Actor Model Implementation Strategy

**Status:** Accepted  
**Date:** 2025-10-02  
**Deciders:** Architecture Team  
**Context:** airssys-rt foundation design

## Context

airssys-rt requires an actor model implementation that balances performance, type safety, and BEAM-inspired patterns. The choice between traditional trait objects (`Box<dyn Actor>`) and generic constraints significantly impacts runtime performance, memory usage, and API ergonomics.

## Decision

We will implement the actor model using **zero-cost abstractions with generic constraints** instead of trait objects, prioritizing compile-time type safety and runtime performance.

### Core Architecture

```rust
// ✅ CHOSEN: Generic constraint approach
pub trait Actor<M: Message> {
    type Error: Error + Send + Sync + 'static;
    async fn handle_message(&mut self, message: M) -> Result<(), Self::Error>;
}

pub struct ActorContext<A, M> 
where 
    A: Actor<M>,
    M: Message,
{
    actor: A,
    mailbox: Mailbox<M>,
    address: ActorAddress,
}

// ❌ REJECTED: Trait object approach
pub trait ActorTrait {
    async fn handle_message(&mut self, message: Box<dyn Any>) -> Result<(), ActorError>;
}
pub struct ActorContext {
    actor: Box<dyn ActorTrait>,  // Runtime dispatch overhead
    mailbox: Box<dyn MailboxTrait>, // Additional heap allocation
}
```

## Rationale

### Performance Benefits
1. **Zero runtime dispatch**: All actor method calls resolved at compile time
2. **Memory efficiency**: No vtable overhead, optimal memory layout
3. **Optimization opportunities**: Compiler can inline and optimize aggressively
4. **Cache locality**: Better data layout for performance-critical paths

### Type Safety Advantages
1. **Compile-time validation**: Message type mismatches caught at compile time
2. **Monomorphization**: Specialized code for each actor/message combination
3. **Error propagation**: Type-safe error handling with structured errors
4. **API clarity**: Clear type relationships in public APIs

### BEAM Compatibility
1. **"Let it crash" philosophy**: Structured error handling with supervisor trees
2. **Message passing**: Type-safe message envelopes with routing
3. **Lightweight processes**: Minimal per-actor overhead (~64 bytes)
4. **Supervisor hierarchies**: Type-safe supervision with generic constraints

## Consequences

### Positive Consequences
- **Performance**: Target <100ns message delivery achieved through zero-cost abstractions
- **Type safety**: Compile-time guarantees prevent runtime message type errors
- **Maintainability**: Clear type relationships make code easier to understand
- **Optimization**: Rust compiler can optimize across actor boundaries

### Negative Consequences
- **Compilation time**: Generic instantiation increases compile time
- **Code complexity**: Generic bounds can make APIs more complex
- **Binary size**: Monomorphization may increase binary size
- **Learning curve**: Developers need to understand generic constraints

### Mitigation Strategies
1. **Compilation time**: Use dynamic dispatch sparingly for non-critical paths
2. **API complexity**: Provide high-level convenience APIs over generic core
3. **Binary size**: Profile and optimize hot paths, accept size increase for performance
4. **Learning curve**: Comprehensive documentation and examples

## Implementation Details

### Message System
```rust
pub trait Message: Send + Sync + 'static {
    const MESSAGE_TYPE: &'static str;
}

pub struct MessageEnvelope<M: Message> {
    pub message: M,
    pub sender: Option<ActorAddress>,
    pub timestamp: DateTime<Utc>,
}
```

### Actor System Integration
```rust
pub struct ActorSystem {
    message_broker: MessageBroker,
    supervisor_tree: SupervisorTree,
    address_registry: AddressRegistry,
}

impl ActorSystem {
    pub async fn spawn<A, M>(&self, actor: A) -> Result<ActorRef<M>, ActorError>
    where 
        A: Actor<M> + Send + 'static,
        M: Message,
    {
        // Type-safe actor spawning with generic constraints
    }
}
```

### Supervisor Integration
```rust
pub trait Supervisor<C: Child> {
    async fn start_child(&mut self, spec: ChildSpec<C>) -> Result<ChildId, Self::Error>;
    async fn handle_child_error(&mut self, id: &ChildId, error: C::Error) -> SupervisionDecision;
}
```

## Alternative Considered

### Option 1: Trait Objects with Type Erasure
```rust
pub trait Actor {
    async fn handle_message(&mut self, message: Box<dyn Any>) -> Result<(), ActorError>;
}
```

**Rejected because:**
- Runtime dispatch overhead
- Type erasure loses compile-time safety
- Additional heap allocations
- Incompatible with zero-cost abstraction goals

### Option 2: Enum-Based Message Dispatch
```rust
pub enum Message {
    Counter(CounterMessage),
    Timer(TimerMessage),
    // ... all possible messages
}
```

**Rejected because:**
- Requires central message enum
- Poor extensibility
- Large memory footprint
- Violates open/closed principle

### Option 3: Hybrid Approach
```rust
// Generic for performance-critical actors
pub trait FastActor<M: Message> { ... }

// Trait objects for flexibility
pub trait SlowActor { ... }
```

**Rejected because:**
- API inconsistency
- Cognitive overhead
- Performance unpredictability
- Maintenance complexity

## Monitoring and Validation

### Performance Metrics
- Message delivery latency: Target <100ns for local delivery
- Actor creation time: Target <1μs for lightweight actors  
- Memory overhead: Target ~64 bytes per actor context
- Throughput: Target >1M messages/second

### Success Criteria
1. All performance targets met
2. Zero-cost abstractions verified through benchmarks
3. Type safety validated through comprehensive testing
4. API ergonomics confirmed through example implementations

### Review Schedule
- Initial implementation review: 2 weeks after RT-TASK-001 completion
- Performance validation: After RT-TASK-008 completion
- Final architecture review: Before RT-TASK-011 completion

## References

- **Microsoft Rust Guidelines**: M-DI-HIERARCHY (prefer concrete types over trait objects)
- **Workspace Standards**: §6.2 Avoid `dyn` Patterns
- **BEAM OTP**: Actor model patterns and supervision trees
- **Zero-cost abstractions**: Rust language design principles

---

**Related Decisions:**
- ADR-RT-002: Message Passing Architecture
- ADR-RT-005: Async Runtime Selection
- ADR-RT-004: Supervisor Tree Design