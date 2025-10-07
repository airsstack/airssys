# ADR-RT-004: Child Trait Separation from Actor Trait

**ADR ID:** ADR-RT-004  
**Created:** 2025-10-07  
**Updated:** 2025-10-07  
**Status:** Accepted  
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
**ACCEPTED: Implement separate `Child` trait independent from `Actor` trait, with blanket implementation bridge.**

The supervision framework will define a separate `Child` trait for lifecycle management, providing a blanket implementation `impl<A: Actor> Child for A` to automatically make all actors supervisable without code changes.

### Rationale

1. **True BEAM/OTP Philosophy** ⭐⭐⭐⭐⭐
   - Erlang supervisors manage ANY process type (gen_server, gen_statem, tasks)
   - Direct conceptual mapping from OTP supervisor behavior
   - Enables supervision of non-message-passing entities

2. **Maximum Flexibility** ⭐⭐⭐⭐⭐
   - Supervise actors, background tasks, I/O handlers, resource pools
   - Future-proof for WASM components, OSL services
   - Not limited to message-passing paradigm

3. **Zero Breaking Changes** ⭐⭐⭐⭐⭐
   - Blanket impl makes ALL actors automatically supervisable
   - Existing actor implementations work unchanged
   - No modifications to Actor trait required

4. **Clean Separation of Concerns** ⭐⭐⭐⭐
   - Actor trait: Message handling and actor-specific behavior
   - Child trait: Lifecycle management for supervision
   - Single Responsibility Principle (SRP) compliance

5. **Composability** ⭐⭐⭐⭐
   - Supervisors can manage heterogeneous children
   - Easy wrapper types for external components
   - Adapter pattern for third-party integrations

### Assumptions
- Blanket implementation `impl<A: Actor> Child for A` provides seamless bridge
- Most users will only implement Actor (automatic Child via blanket impl)
- Explicit Child implementation only needed for non-actor entities
- Documentation clearly explains relationship between traits

## Considered Options

### Option 1: Separate Child Trait (SELECTED)
**Description:** Define independent `Child` trait with `start()`, `stop()`, `health_check()`, provide blanket impl for Actor

**Pros:**
- ✅ Maximum flexibility - supervise ANY entity type
- ✅ True BEAM/OTP alignment
- ✅ Zero Actor code changes (blanket impl)
- ✅ Clean separation of concerns
- ✅ Future-proof for WASM, OSL integration
- ✅ Composable - mix actors and non-actors

**Cons:**
- ❌ Two traits to understand (moderate learning curve)
- ❌ Generic signatures slightly more complex
- ❌ Small documentation burden explaining relationship
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

3. Implement blanket impl in `src/supervisor/traits.rs`
   ```rust
   #[async_trait]
   impl<A> Child for A
   where
       A: Actor + Send + Sync + 'static,
       A::Error: Error + Send + Sync + 'static,
   {
       type Error = A::Error;
       async fn start(&mut self) -> Result<(), Self::Error> {
           self.pre_start().await
       }
       async fn stop(&mut self, _timeout: Duration) -> Result<(), Self::Error> {
           self.post_stop().await
       }
   }
   ```

**Phase 2: Supervisor Integration**
1. `SupervisorNode<S, C, M>` generic over `C: Child`
2. `ChildSpec<C, F>` generic over factory type (§6.2 compliance)
3. All supervision operations use `Child` trait methods

**Phase 3: Documentation and Examples**
1. Comprehensive rustdoc explaining Child/Actor relationship
2. Example: Supervising actors (automatic via blanket impl)
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
- Blanket impl compiles to direct Actor method calls
- Monomorphization eliminates abstraction cost
- Same performance as calling Actor methods directly

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
- ✅ Documentation clearly explains trait relationships

**Considerations:**
- ⚠️ Need to maintain blanket impl consistency with Actor trait changes
- ⚠️ Documentation must explain Child/Actor relationship clearly

## Mitigation Strategies

### Learning Curve Mitigation
1. **Comprehensive Documentation**: Rustdoc with clear examples
2. **Type Aliases**: Provide convenience aliases like `type ActorChild<A> = ...`
3. **Examples Repository**: Show both actor and non-actor supervision
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

- ✅ All existing Actor implementations work with supervisors unchanged
- ✅ At least one non-actor entity successfully supervised
- ✅ Documentation clearly explains Child/Actor relationship
- ✅ Zero performance regression vs direct Actor method calls
- ✅ Developer feedback shows acceptable learning curve

## References

- **KNOWLEDGE-RT-003**: Supervisor Tree Implementation Strategies
- **KNOWLEDGE-RT-013**: RT-TASK-007 and RT-TASK-010 Action Plans
- **Workspace Standards**: §6.2 (Avoid dyn Patterns), §6.1 (YAGNI Principles)
- **Microsoft Rust Guidelines**: M-DI-HIERARCHY (generic constraints over trait objects)
- **Erlang/OTP Documentation**: Supervisor behavior and gen_server patterns

## Appendix: Code Examples

### Example 1: Actor Supervision (Zero Code Changes)
```rust
// Existing actor - NO CHANGES NEEDED
struct CounterActor { count: u32 }

#[async_trait]
impl Actor for CounterActor {
    type Message = CounterMsg;
    type Error = CounterError;
    
    async fn handle_message(&mut self, msg: Self::Message, ctx: &mut ActorContext<Self::Message>) 
        -> Result<(), Self::Error> 
    {
        self.count += msg.delta;
        Ok(())
    }
}

// ✅ CounterActor is now automatically supervisable via blanket impl!
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
