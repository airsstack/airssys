# KNOWLEDGE-RT-001: Zero-Cost Actor Model Architecture

**Sub-Project:** airssys-rt  
**Category:** Actor Model  
**Created:** 2025-10-02  
**Last Updated:** 2025-10-02  
**Status:** active  

## Context and Problem

Traditional actor model implementations in Rust often rely on `Box<dyn Actor>` trait objects and runtime dispatch, which introduces performance overhead and limits compile-time optimizations. For airssys-rt, we need a zero-cost abstraction approach that maintains BEAM-inspired patterns while leveraging Rust's type system for maximum performance.

## Knowledge Details

### Core Architecture Pattern

The airssys-rt actor model uses compile-time generic constraints instead of runtime polymorphism:

```rust
// ✅ Zero-cost approach with generics
pub trait Actor<M: Message> {
    type Error: Error + Send + Sync + 'static;
    
    async fn handle_message(&mut self, message: M) -> Result<(), Self::Error>;
    async fn handle_stop(&mut self) -> Result<(), Self::Error>;
}

// ✅ Generic actor context
pub struct ActorContext<A, M> 
where 
    A: Actor<M>,
    M: Message,
{
    actor: A,
    mailbox: Mailbox<M>,
    address: ActorAddress,
}

// ❌ Avoid trait objects (runtime overhead)
pub struct LegacyActorContext {
    actor: Box<dyn Actor>, // Runtime dispatch overhead
    mailbox: Box<dyn Mailbox>, // Additional allocation
}
```

### Message System Architecture

Messages use compile-time type identification with const generics:

```rust
pub trait Message: Send + Sync + 'static {
    const MESSAGE_TYPE: &'static str;
}

pub struct MessageEnvelope<M: Message> {
    pub message: M,
    pub sender: Option<ActorAddress>,
    pub timestamp: DateTime<Utc>,
}

// Type-safe message handling at compile time
impl<M: Message> MessageEnvelope<M> {
    pub fn new(message: M, sender: Option<ActorAddress>) -> Self {
        Self {
            message,
            sender,
            timestamp: Utc::now(),
        }
    }
}
```

### Supervisor Tree Pattern

Supervisor trees maintain type safety while supporting heterogeneous actors:

```rust
pub enum SupervisionStrategy {
    OneForOne,
    OneForAll,
    RestForOne,
}

pub struct SupervisorSpec<S: SupervisionStrategy> {
    strategy: S,
    max_restarts: u32,
    restart_period: Duration,
    children: Vec<ChildSpec>,
}

// Child specifications maintain type information
pub struct ChildSpec {
    id: String,
    restart_policy: RestartPolicy,
    shutdown_timeout: Duration,
    child_type: ChildType,
}
```

## Performance Characteristics

### Compile-time Benefits
- **Zero runtime dispatch**: All actor calls resolved at compile time
- **Monomorphization**: Specialized code generated for each actor type
- **Inline optimizations**: Method calls can be inlined by compiler
- **Memory layout optimization**: No vtable indirection overhead

### Runtime Performance
- **Message passing**: <100ns for local message delivery
- **Actor creation**: <1μs for lightweight actor spawning
- **Memory overhead**: ~64 bytes per actor context
- **Supervisor operations**: <10μs for restart decisions

## Implementation Guidelines

### Type Safety Principles
1. **Generic constraints over trait objects**: Always prefer `T: Actor<M>` over `Box<dyn Actor>`
2. **Compile-time message validation**: Use const MESSAGE_TYPE for type checking
3. **Zero-allocation patterns**: Minimize heap allocations in hot paths
4. **Borrowing over ownership**: Use references where lifetime permits

### Error Handling Patterns
```rust
#[derive(Debug, thiserror::Error)]
pub enum ActorError {
    #[error("Actor {id} failed to start: {source}")]
    StartupFailed { id: String, source: Box<dyn Error + Send + Sync> },
    
    #[error("Message delivery failed to {target}: {reason}")]
    MessageDeliveryFailed { target: ActorAddress, reason: String },
    
    #[error("Supervisor restart limit exceeded: {max_restarts} in {period:?}")]
    RestartLimitExceeded { max_restarts: u32, period: Duration },
}
```

## Related Patterns

### Complementary Knowledge
- **KNOWLEDGE-RT-002**: Message Broker Zero-Copy Patterns
- **KNOWLEDGE-RT-003**: Supervisor Tree Implementation Strategies
- **KNOWLEDGE-RT-004**: Actor Lifecycle Management Patterns

### Architecture Decisions
- **ADR-RT-001**: Actor Model Implementation Strategy
- **ADR-RT-002**: Message Passing Architecture
- **ADR-RT-005**: Async Runtime Selection

## Usage Examples

### Basic Actor Implementation
```rust
#[derive(Debug)]
pub struct CounterMessage(pub i32);

impl Message for CounterMessage {
    const MESSAGE_TYPE: &'static str = "counter_message";
}

pub struct CounterActor {
    count: i32,
}

impl Actor<CounterMessage> for CounterActor {
    type Error = ActorError;
    
    async fn handle_message(&mut self, message: CounterMessage) -> Result<(), Self::Error> {
        self.count += message.0;
        Ok(())
    }
    
    async fn handle_stop(&mut self) -> Result<(), Self::Error> {
        println!("Counter stopped at: {}", self.count);
        Ok(())
    }
}
```

### Actor System Integration
```rust
let actor_system = ActorSystem::new("counter_system").await?;
let counter = CounterActor { count: 0 };
let actor_ref = actor_system.spawn(counter).await?;

// Type-safe message sending
actor_ref.send(CounterMessage(42)).await?;
```

## Lessons Learned

### What Works Well
- Generic constraints provide excellent performance
- Compile-time type checking prevents runtime errors
- Monomorphization enables aggressive optimizations
- Zero-cost abstractions maintain ergonomics

### Potential Pitfalls
- Generic bounds can make APIs complex
- Compilation time increases with many instantiations
- Error messages can be verbose with complex generics
- Debugging can be challenging with monomorphized code

## Future Considerations

### Planned Enhancements
- Message routing optimization with type-specific paths
- Actor pool implementations with generic constraints
- Hot-path specialization for common message types
- Integration with OSL security contexts

### Research Areas
- Const generics for message type IDs
- Generic associated types for advanced patterns
- Compile-time actor dependency validation
- Zero-copy message serialization patterns

---

**References:**
- Microsoft Rust Guidelines: M-DI-HIERARCHY, M-AVOID-WRAPPERS
- Workspace Standards: §6.2 Avoid `dyn` Patterns
- Performance targets: <1ms message delivery, 10,000+ concurrent actors