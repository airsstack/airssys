# RT-TASK-007 Phase 1 Design Decisions Summary

**Task:** RT-TASK-007 - Supervisor Framework  
**Phase:** Phase 1 - Supervisor Traits & Core Types  
**Date:** 2025-10-07  
**Status:** Design decisions finalized, ready for implementation  

## Context

Before starting RT-TASK-007 Phase 1 implementation, we conducted a comprehensive design review to finalize key architectural decisions. This document captures all decisions made, their rationale, and implementation guidelines.

## Design Decisions Made

### ✅ Decision 1: Child Trait Design - Separate from Actor Trait

**Question:** Should the Child trait be separate from Actor trait, integrated into Actor, or use blanket impl only?

**DECISION: Option A - Separate Child Trait with Blanket Implementation**

**Rationale:**
- ✅ Maximum flexibility - supervise ANY entity type (actors, tasks, I/O handlers, services)
- ✅ True BEAM/OTP philosophy - supervisors manage processes, not just specific types
- ✅ Zero breaking changes - blanket impl makes all actors automatically supervisable
- ✅ Future-proof for WASM components, OSL services integration
- ✅ Clean separation of concerns (Actor = message handling, Child = lifecycle)

**Implementation Pattern:**
```rust
// Child trait - Universal supervision interface
#[async_trait]
pub trait Child: Send + Sync + 'static {
    type Error: Error + Send + Sync + 'static;
    async fn start(&mut self) -> Result<(), Self::Error>;
    async fn stop(&mut self, timeout: Duration) -> Result<(), Self::Error>;
    async fn health_check(&self) -> ChildHealth { ChildHealth::Healthy }
}

// Blanket implementation - All actors are automatically children
#[async_trait]
impl<A> Child for A
where
    A: Actor + Send + Sync + 'static,
    A::Error: Error + Send + Sync + 'static,
{
    type Error = A::Error;
    async fn start(&mut self) -> Result<(), Self::Error> {
        self.pre_start().await  // Delegates to Actor
    }
    async fn stop(&mut self, _timeout: Duration) -> Result<(), Self::Error> {
        self.post_stop().await  // Delegates to Actor
    }
}
```

**Benefits:**
- Actors work with supervisors unchanged (blanket impl)
- Can supervise non-actor entities (background tasks, I/O handlers)
- Heterogeneous supervision trees (actors + tasks mixed)
- Zero performance overhead (static dispatch via generics)

**Documented In:**
- **ADR-RT-004**: Child Trait Separation from Actor Trait (full decision record)
- **KNOWLEDGE-RT-014**: Child Trait Design Patterns and Integration Strategies (implementation guide)

---

### ✅ Decision 2: Child Factory Pattern - Generic Factory Type

**Question:** Should ChildSpec use `Box<dyn Fn()>`, generic factory type, or require Clone?

**DECISION: Option B - Generic Factory Type Parameter**

**Rationale:**
- ✅ Zero-cost abstraction compliance (§6.2 - avoid dyn)
- ✅ Compile-time type resolution (no runtime overhead)
- ✅ Microsoft Rust Guidelines compliance (M-DI-HIERARCHY)
- ✅ Maximum flexibility without boxing overhead

**Implementation Pattern:**
```rust
pub struct ChildSpec<C, F>
where
    C: Child,
    F: Fn() -> C + Send + Sync + 'static,
{
    pub id: String,
    pub factory: F,  // ← Generic factory, not Box<dyn Fn()>
    pub restart_policy: RestartPolicy,
    pub shutdown_policy: ShutdownPolicy,
    pub start_timeout: Duration,
    pub shutdown_timeout: Duration,
}
```

**Benefits:**
- Zero-cost abstraction (no Box, no dyn)
- Static dispatch via monomorphization
- Compile-time optimization opportunities
- Type-safe factory functions

**Trade-offs:**
- Slightly more complex generic signatures (acceptable)
- Type aliases can simplify common cases
- Follows workspace standards §6.2

---

### ✅ Decision 3: Monitor Integration - Generic Type Parameter

**Question:** Should SupervisorNode use generic `M: Monitor<SupervisionEvent>`, concrete type, or type alias?

**DECISION: Option A - Generic Monitor with Type Alias**

**Rationale:**
- ✅ Maximum flexibility (InMemoryMonitor, NoopMonitor, custom implementations)
- ✅ Dependency injection pattern (M-DI-HIERARCHY)
- ✅ Zero-overhead NoopMonitor when monitoring disabled
- ✅ Consistent with KNOWLEDGE-RT-013 action plans

**Implementation Pattern:**
```rust
pub struct SupervisorNode<S, C, M>
where
    S: SupervisionStrategy,
    C: Child,
    M: Monitor<SupervisionEvent>,  // ← Generic monitor
{
    id: Uuid,
    strategy: S,
    children: HashMap<ChildId, ChildHandle<C>>,
    monitor: M,
}

// Convenience type alias for common cases
pub type ActorSupervisor<A, S> = SupervisorNode<
    S,
    A,
    InMemoryMonitor<SupervisionEvent>
>;
```

**Benefits:**
- Flexible monitoring strategy selection
- Zero-overhead NoopMonitor for tests
- InMemoryMonitor for production observability
- Future custom monitor implementations

**Integration:**
- Uses Monitor<SupervisionEvent> from RT-TASK-010 ✅
- All supervision events recorded via `monitor.record()`
- Enables monitoring-driven supervision decisions

---

### ✅ Decision 4: Testing Approach - Embedded Unit Tests

**Question:** Should tests be embedded in module files, separate test directory, or mixed?

**DECISION: Option A - Embedded Unit Tests (Rust Convention)**

**Rationale:**
- ✅ Follows Rust conventions and best practices
- ✅ Matches existing airssys-rt codebase patterns
- ✅ Tests live close to implementation
- ✅ Easy to maintain test-code coherence

**Implementation Pattern:**
```rust
// At the end of each module file (e.g., src/supervisor/traits.rs)

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_child_id_uniqueness() {
        // Unit test implementation
    }
    
    #[tokio::test]
    async fn test_child_lifecycle() {
        // Async unit test implementation
    }
    
    // ... 20+ tests per module
}
```

**Testing Strategy:**
- Embedded unit tests in each module file
- Integration tests in `tests/supervisor_tests.rs`
- Examples demonstrating usage in `examples/`

**Coverage Targets (Phase 1):**
- `src/supervisor/traits.rs`: 8+ tests
- `src/supervisor/types.rs`: 10+ tests  
- `src/supervisor/error.rs`: 5+ tests
- **Total Phase 1**: 20+ tests

---

## Implementation Guidelines for Phase 1

### Files to Create (RT-TASK-007 Phase 1)

1. **`src/supervisor/mod.rs`** (~100 lines)
   - Module declarations
   - Re-exports
   - Module-level documentation

2. **`src/supervisor/traits.rs`** (~400-500 lines)
   - `Child` trait definition
   - `Supervisor` trait definition
   - `SupervisionStrategy` trait definition
   - Blanket impl `impl<A: Actor> Child for A`
   - Embedded unit tests (8+ tests)

3. **`src/supervisor/types.rs`** (~300-400 lines)
   - `ChildSpec<C, F>` struct
   - `RestartPolicy` enum
   - `ShutdownPolicy` enum
   - `ChildState` enum
   - `ChildHealth` enum
   - `SupervisionDecision` enum
   - `ChildHandle<C>` struct
   - Embedded unit tests (10+ tests)

4. **`src/supervisor/error.rs`** (~200-250 lines)
   - `SupervisorError` enum with thiserror
   - Helper methods (`is_fatal()`, `is_retryable()`)
   - Embedded unit tests (5+ tests)

### Workspace Standards Compliance Checklist

- [ ] **§2.1**: 3-layer import organization (std → third-party → internal)
- [ ] **§3.2**: Use `chrono::DateTime<Utc>` for all timestamps
- [ ] **§4.3**: `mod.rs` contains ONLY declarations and re-exports
- [ ] **§6.1**: YAGNI principles (avoid premature abstraction)
- [ ] **§6.2**: Avoid `dyn` patterns (use generic constraints)
- [ ] **§6.3**: Microsoft Rust Guidelines (M-SERVICES-CLONE, M-DI-HIERARCHY, M-ERRORS-CANONICAL-STRUCTS)

### Phase 1 Acceptance Criteria

- [ ] All 4 files created with proper structure
- [ ] Child trait with comprehensive rustdoc and examples
- [ ] Blanket impl `impl<A: Actor> Child for A` implemented
- [ ] Generic `ChildSpec<C, F>` following §6.2
- [ ] Generic `SupervisorNode<S, C, M>` prepared for Phase 3
- [ ] 20+ unit tests passing (embedded in modules)
- [ ] Zero compiler warnings
- [ ] Zero clippy warnings
- [ ] Full workspace standards compliance

### Knowledge Base References

**MUST READ before implementation:**
1. **ADR-RT-004**: Child Trait Separation from Actor Trait
2. **KNOWLEDGE-RT-014**: Child Trait Design Patterns and Integration Strategies
3. **KNOWLEDGE-RT-013**: RT-TASK-007 and RT-TASK-010 Action Plans (Phase 1 section)
4. **KNOWLEDGE-RT-003**: Supervisor Tree Implementation Strategies

**Related ADRs:**
- **ADR-RT-001**: Zero-Cost Abstractions (generic constraints)
- **ADR-RT-002**: Message Passing Architecture (for supervisor-child communication)

**Related Knowledge:**
- **KNOWLEDGE-RT-001**: Zero-Cost Actor Model Architecture
- **KNOWLEDGE-RT-005**: Actor System Core Implementation Guide

### Integration Points

**Dependencies (Upstream):**
- ✅ **RT-TASK-010**: Monitoring module complete (Monitor<SupervisionEvent> available)
- ✅ **RT-TASK-002**: Actor trait complete (for blanket impl)
- ✅ **Workspace Standards**: §2.1-§6.3 documented

**Downstream Impact:**
- RT-TASK-007 Phase 2: Restart strategies will use these traits
- RT-TASK-007 Phase 3: SupervisorNode will implement Supervisor trait
- RT-TASK-007 Phase 4: Health monitoring will use ChildHealth enum

## Summary

All design decisions have been finalized and documented:

1. ✅ **Child Trait**: Separate from Actor, with blanket impl bridge
2. ✅ **Factory Pattern**: Generic `ChildSpec<C, F>` avoiding dyn
3. ✅ **Monitor Integration**: Generic `M: Monitor<SupervisionEvent>`
4. ✅ **Testing**: Embedded unit tests following Rust conventions

**Documentation Created:**
- **ADR-RT-004**: Architecture decision record
- **KNOWLEDGE-RT-014**: Implementation patterns and examples
- **This Document**: Phase 1 design decisions summary

**Ready for Implementation:** ✅ YES

All knowledge base documentation is complete. Phase 1 implementation can begin with confidence that architectural decisions are sound, documented, and aligned with workspace standards and Microsoft Rust Guidelines.

---

**Next Steps:**
1. Begin RT-TASK-007 Phase 1 implementation
2. Create 4 files: mod.rs, traits.rs, types.rs, error.rs
3. Follow exact patterns from KNOWLEDGE-RT-014
4. Ensure 20+ tests passing
5. Zero warnings compliance

**Estimated Duration:** 12-16 hours (Days 1-2 of RT-TASK-007)
