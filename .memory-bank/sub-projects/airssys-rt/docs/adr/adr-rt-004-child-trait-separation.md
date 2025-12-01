# ADR-RT-004: Child Trait Separation from Actor Trait

**ADR ID:** ADR-RT-004  
**Created:** 2025-10-07  
**Updated:** 2025-01-07  
**Status:** Accepted (Revised)  
**Deciders:** Architecture team, RT-TASK-007 implementation team  

## Title
Separate Child Trait for Supervision (Independent from Actor Trait)

## Context

### Problem Statement
The supervisor framework (RT-TASK-007) needs to manage lifecycle operations for supervised entities. We must decide whether the supervision lifecycle interface should be:
1. A separate `Child` trait independent from `Actor`
2. Integrated into the `Actor` trait
3. A blanket implementation pattern only

This decision fundamentally affects the flexibility, composability, and BEAM/OTP alignment of our supervision system.

### Business Context
- **RT-TASK-007**: Supervisor Framework requires entity lifecycle management (start, stop, health check)
- **BEAM Alignment**: Erlang/OTP supervisors manage ANY process, not just specific actor types
- **Future Requirements**: Need to supervise diverse entity types (WASM components, OSL services, background tasks)
- **Integration**: Must work seamlessly with existing Actor trait without breaking changes

### Technical Context
- **Current State**: Actor trait exists with `pre_start()`, `post_stop()`, `on_error()` lifecycle hooks
- **RT-TASK-010**: Monitoring infrastructure complete with `Monitor<SupervisionEvent>` trait
- **Workspace Standards**: §6.2 mandates avoiding `dyn` trait patterns, preferring generic constraints
- **Microsoft Rust Guidelines**: M-DI-HIERARCHY (concrete types > generics > dyn traits)

### Stakeholders
- Actor system developers implementing supervised actors
- System integrators building non-actor supervised entities
- Framework maintainers managing API surface area
- End users requiring fault-tolerant systems

## Decision

### Summary
**ACCEPTED: Implement separate `Child` trait completely independent from `Actor` trait. NO automatic blanket implementation.**

The supervision framework defines a separate `Child` trait for lifecycle management that is intentionally independent from the `Actor` trait. Actors that need to be supervised must **explicitly implement the Child trait**. This maintains true separation of concerns and enables supervision of ANY entity type without coupling Child to Actor-specific concepts like `ActorContext` or `MessageBroker`.

### Rationale

1. **True BEAM/OTP Philosophy** ⭐⭐⭐⭐⭐
   - Erlang supervisors manage ANY process type (gen_server, gen_statem, tasks)
   - Direct conceptual mapping from OTP supervisor behavior
   - Enables supervision of non-message-passing entities

2. **Maximum Flexibility** ⭐⭐⭐⭐⭐
   - Supervise actors, background tasks, I/O handlers, resource pools
   - Future-proof for WASM components, OSL services
   - Not limited to message-passing paradigm

3. **True Independence** ⭐⭐⭐⭐⭐
   - Child trait has NO dependencies on Actor trait
   - No coupling to `ActorContext<M, B>` generic parameters
   - Actor lifecycle methods (`pre_start`, `post_stop`) require context parameter
   - Child lifecycle methods (`start`, `stop`) are context-free
   - Incompatible method signatures prevent automatic blanket impl

4. **Clean Separation of Concerns** ⭐⭐⭐⭐⭐
   - Actor trait: Message handling and actor-specific behavior
   - Child trait: Lifecycle management for supervision
   - Single Responsibility Principle (SRP) compliance
   - Each trait can evolve independently

5. **Composability** ⭐⭐⭐⭐
   - Supervisors can manage heterogeneous children
   - Easy wrapper types for external components
   - Adapter pattern for third-party integrations

### Assumptions
- Actors requiring supervision implement BOTH Actor and Child traits explicitly
- Most supervised entities will be actors (explicit dual implementation common)
- Explicit implementation provides clear lifecycle contract
- Documentation explains Actor/Child independence and implementation patterns

## Considered Options

### Option 1: Separate Child Trait (SELECTED)
**Description:** Define independent `Child` trait with `start()`, `stop()`, `health_check()`. Actors requiring supervision must explicitly implement both Actor and Child traits.

**Pros:**
- ✅ Maximum flexibility - supervise ANY entity type
- ✅ True BEAM/OTP alignment
- ✅ Complete independence - no coupling between traits
- ✅ Clean separation of concerns
- ✅ Future-proof for WASM, OSL integration
- ✅ Composable - mix actors and non-actors
- ✅ Each trait evolves independently
- ✅ No method signature conflicts

**Cons:**
- ❌ Actors must explicitly implement Child (one-time implementation cost)
- ❌ Two traits to understand (moderate learning curve)
- ❌ Generic signatures slightly more complex
- ❌ Documentation burden explaining both traits
- ❌ Slightly larger testing surface area

**Implementation Effort:** Medium  
**Risk Level:** Low  

### Option 2: Integrate into Actor Trait
**Description:** Add `start()`, `stop()`, `health_check()` methods directly to Actor trait

**Pros:**
- ✅ Single trait simplicity
- ✅ Easy to understand for beginners
- ✅ Simpler type signatures

**Cons:**
- ❌ Only actors can be supervised (major limitation)
- ❌ Violates SRP - Actor does too much
- ❌ Breaking changes to Actor trait
- ❌ Cannot supervise background tasks, I/O handlers, etc.
- ❌ Not BEAM-aligned (supervisors manage processes, not just actors)
- ❌ Future integration pain (WASM, OSL)

**Implementation Effort:** Low  
**Risk Level:** Medium (architectural limitation)

### Option 3: Blanket Implementation Only
**Description:** No Child trait, just `impl Supervisor for Actor` blanket impl

**Pros:**
- ✅ Minimal API surface
- ✅ Simple for actor-only use cases
- ✅ No new traits to learn

**Cons:**
- ❌ Only actors supervisable (same as Option 2)
- ❌ No abstraction for non-actor entities
- ❌ Supervisor tightly coupled to Actor
- ❌ Not extensible to new entity types
- ❌ Requires supervisor to understand Actor internals

**Implementation Effort:** Low  
**Risk Level:** High (architectural rigidity)

## Implementation

### Implementation Plan

**Phase 1: Core Trait Definition (RT-TASK-007 Phase 1)**
1. Define `Child` trait in `src/supervisor/traits.rs`
   ```rust
   #[async_trait]
   pub trait Child: Send + Sync + 'static {
       type Error: Error + Send + Sync + 'static;
       async fn start(&mut self) -> Result<(), Self::Error>;
       async fn stop(&mut self, timeout: Duration) -> Result<(), Self::Error>;
       async fn health_check(&self) -> ChildHealth { ChildHealth::Healthy }
   }
   ```

2. Define `ChildHealth` enum in `src/supervisor/types.rs`
   ```rust
   pub enum ChildHealth {
       Healthy,
       Degraded(String),
       Failed(String),
   }
   ```

3. **No blanket implementation** - Child and Actor are independent
   
   **Rationale:** Actor lifecycle methods require `ActorContext<M, B>` parameter:
   ```rust
   // Actor trait (existing)
   async fn pre_start<B>(&mut self, context: &mut ActorContext<M, B>);
   async fn post_stop<B>(&mut self, context: &mut ActorContext<M, B>);
   
   // Child trait (new)
   async fn start(&mut self) -> Result<(), Self::Error>;
   async fn stop(&mut self, timeout: Duration) -> Result<(), Self::Error>;
   ```
   
   **Incompatible signatures** prevent automatic blanket impl. This is intentional - it maintains true independence between traits.

4. Actors implement Child explicitly when supervision needed
   ```rust
   #[async_trait]
   impl Child for MyActor {
       type Error = MyActorError;
       
       async fn start(&mut self) -> Result<(), Self::Error> {
           // Actor-specific initialization for supervision
           Ok(())
       }
       
       async fn stop(&mut self, _timeout: Duration) -> Result<(), Self::Error> {
           // Actor-specific cleanup for supervision
           Ok(())
       }
   }
   ```

**Phase 2: Supervisor Integration**
1. `SupervisorNode<S, C, M>` generic over `C: Child`
2. `ChildSpec<C, F>` generic over factory type (§6.2 compliance)
3. All supervision operations use `Child` trait methods

**Phase 3: Documentation and Examples**
1. Comprehensive rustdoc explaining Child/Actor independence
2. Example: Actor with explicit Child implementation
3. Example: Supervising non-actor background tasks
4. Example: Mixed supervision (actors + tasks)

### Timeline
- **Phase 1**: RT-TASK-007 Phase 1 (Day 1-2)
- **Phase 2**: RT-TASK-007 Phase 3 (Day 5-7)
- **Phase 3**: RT-TASK-007 Phase 5 (Day 10)

### Resources Required
- RT-TASK-007 implementation team
- Documentation review for trait relationship clarity
- Integration testing with existing Actor implementations

### Dependencies
- **Upstream**: RT-TASK-010 (Monitoring) - ✅ COMPLETE
- **Upstream**: Actor trait implementation - ✅ COMPLETE
- **Downstream**: RT-TASK-007 Phase 1-5 implementation
- **Downstream**: RT-TASK-009 (OSL Integration)

## Implications

### System Impact
**Positive:**
- ✅ Supervision system can manage ANY entity type (actors, tasks, services)
- ✅ Clean architectural boundaries (Actor vs Child responsibilities)
- ✅ Enables heterogeneous supervision trees
- ✅ Future integration points (WASM components, OSL services)

**Considerations:**
- ⚠️ Two traits to understand (mitigated by documentation)
- ⚠️ Generic signatures include `C: Child` constraint

### Performance Impact
**Zero Performance Overhead:**
- Static dispatch via generics (no `dyn` trait objects)
- Explicit Child implementations compile to direct method calls
- Monomorphization eliminates abstraction cost
- No runtime overhead from trait separation

### Security Impact
**Neutral/Positive:**
- No security implications from trait separation
- Lifecycle control (`start`, `stop`) clearly defined
- Health checking enables proactive failure detection
- Consistent supervision across all entity types

### Scalability Impact
**Positive:**
- ✅ Enables supervision of diverse entity types at scale
- ✅ Non-actor background tasks can be supervised
- ✅ Resource pool managers, I/O handlers supervisable
- ✅ WASM component supervision (future)

### Maintainability Impact
**Positive:**
- ✅ Clear separation of concerns (SRP)
- ✅ Easier testing (mock Child implementations)
- ✅ Simpler trait implementations (focused responsibilities)
- ✅ Documentation clearly explains trait independence
- ✅ Traits can evolve independently without conflicts

**Considerations:**
- ⚠️ Actors need explicit Child implementation for supervision
- ⚠️ Documentation must explain when/why to implement Child

## Mitigation Strategies

### Implementation Boilerplate Mitigation
1. **Derive Macros** (future): `#[derive(Child)]` for common patterns
2. **Documentation Templates**: Clear copy-paste examples for actor supervision
3. **Examples Repository**: Show various Child implementation patterns
4. **Migration Guide**: Clear guide for existing Actor implementations (no changes needed)

### Complexity Mitigation
1. **Generic Type Aliases**: Simplify complex signatures
   ```rust
   type ActorSupervisor<A> = SupervisorNode<OneForOne, A, InMemoryMonitor<SupervisionEvent>>;
   ```
2. **Builder Patterns**: Hide generic complexity in builder APIs
3. **Prelude Module**: Export commonly used types and traits

### Testing Mitigation
1. **MockChild Utilities**: Provide test utilities in `src/supervisor/testing.rs`
2. **Property-Based Tests**: Verify Child trait contract compliance
3. **Integration Examples**: Show proper testing patterns

## Alternatives Revisited

If this decision proves problematic in practice, we can:

1. **Add Convenience Layer**: Keep Child trait, add higher-level API for simple cases
2. **Type Aliases**: Provide aliases for common patterns to reduce complexity
3. **Builder Simplification**: Enhanced builders to hide generic complexity

**Note:** We CANNOT easily move from integrated (Option 2) to separated (Option 1) without breaking changes. Moving from separated to integrated is also difficult. This decision is largely one-way, making flexibility the safer choice.

## Success Metrics

- ✅ Actors can be supervised by explicitly implementing Child trait
- ✅ At least one non-actor entity successfully supervised
- ✅ Documentation clearly explains Child/Actor independence
- ✅ Zero performance regression vs direct method calls
- ✅ Developer feedback shows acceptable implementation patterns
- ✅ Test suite validates both actor and non-actor supervision

## References

- **KNOWLEDGE-RT-003**: Supervisor Tree Implementation Strategies
- **KNOWLEDGE-RT-013**: RT-TASK-007 and RT-TASK-010 Action Plans
- **KNOWLEDGE-RT-014**: Child Trait Design Patterns and Integration Guide
- **Workspace Standards**: §6.2 (Avoid dyn Patterns), §6.1 (YAGNI Principles)
- **Microsoft Rust Guidelines**: M-DI-HIERARCHY (generic constraints over trait objects)
- **Erlang/OTP Documentation**: Supervisor behavior and gen_server patterns

## Appendix: Code Examples

### Example 1: Actor Supervision with Explicit Child Implementation

```rust
// Define actor
struct CounterActor { count: u32 }

#[async_trait]
impl Actor for CounterActor {
    type Message = CounterMsg;
    type Error = CounterError;
    
    async fn handle_message<B: MessageBroker<Self::Message>>(
        &mut self, 
        msg: Self::Message, 
        ctx: &mut ActorContext<Self::Message, B>
    ) -> Result<(), Self::Error> 
    {
        self.count += msg.delta;
        Ok(())
    }
}

// Explicitly implement Child for supervision
#[async_trait]
impl Child for CounterActor {
    type Error = CounterError;
    
    async fn start(&mut self) -> Result<(), Self::Error> {
        // Actor-specific initialization
        println!("CounterActor starting");
        Ok(())
    }
    
    async fn stop(&mut self, _timeout: Duration) -> Result<(), Self::Error> {
        // Actor-specific cleanup
        println!("CounterActor stopping");
        Ok(())
    }
}

// ✅ CounterActor can now be supervised!
let supervisor = SupervisorNode::<OneForOne, _, _>::new(OneForOne, monitor);
supervisor.add_child(ChildSpec {
    id: "counter".into(),
    factory: || CounterActor { count: 0 },
    // ...
}).await?;
```

### Example 2: Non-Actor Background Task Supervision
```rust
// Custom background task (NOT an actor)
struct FileWatcher {
    path: PathBuf,
    handle: Option<JoinHandle<()>>,
}

#[async_trait]
impl Child for FileWatcher {
    type Error = FileWatcherError;
    
    async fn start(&mut self) -> Result<(), Self::Error> {
        let path = self.path.clone();
        self.handle = Some(tokio::spawn(async move {
            // Watch filesystem
        }));
        Ok(())
    }
    
    async fn stop(&mut self, timeout: Duration) -> Result<(), Self::Error> {
        if let Some(h) = self.handle.take() {
            tokio::select! {
                _ = h => Ok(()),
                _ = tokio::time::sleep(timeout) => Err(FileWatcherError::Timeout)
            }
        } else {
            Ok(())
        }
    }
}

// ✅ FileWatcher is supervisable even though it's NOT an actor!
```

### Example 3: Mixed Supervision Tree
```rust
let mut supervisor = SupervisorNode::<OneForAll, _, _>::new(OneForAll, monitor);

// Supervise an actor
supervisor.add_child(ChildSpec {
    id: "counter_actor".into(),
    factory: || CounterActor { count: 0 },
    // ...
}).await?;

// Supervise a non-actor background task
supervisor.add_child(ChildSpec {
    id: "file_watcher".into(),
    factory: || FileWatcher { path: PathBuf::from("/tmp"), handle: None },
    // ...
}).await?;

// ✅ Supervisor manages BOTH actors and non-actors seamlessly!
```

---

**Decision Status**: ✅ **ACCEPTED** - Implementation to begin in RT-TASK-007 Phase 1
