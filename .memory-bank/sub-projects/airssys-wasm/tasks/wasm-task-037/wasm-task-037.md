# WASM-TASK-037: Implement ComponentWrapper

**Status:** pending
**Added:** 2026-01-22
**Updated:** 2026-01-22
**Priority:** high
**Estimated Duration:** 3-4 hours
**Phase:** Phase 6 - Component & Messaging Modules (Layer 3)

## Original Request
Implement the ComponentWrapper that wraps a WASM component as an airssys-rt Actor + Child.

## Thought Process
ComponentWrapper is the core integration point between WASM components and the airssys-rt actor system. It:
- Wraps each WASM component instance as an Actor
- Implements the Child trait for lifecycle management (start/stop)
- Uses trait-based dependency injection (Arc<dyn RuntimeEngine>) instead of concrete types
- Handles component messages through the Actor trait
- Provides isolation and fault tolerance through actor supervision

Key Design Principle: Uses `Arc<dyn RuntimeEngine>` for dependency injection - the concrete engine implementation is provided by the system/ module (Layer 4).

## Deliverables
- [ ] `component/wrapper.rs` with ComponentWrapper struct
- [ ] ComponentActorMessage enum (HandleMessage, HandleCallback, Shutdown)
- [ ] Child trait implementation (start/stop lifecycle)
- [ ] Actor trait implementation (message handling)
- [ ] Unit tests for ComponentWrapper lifecycle
- [ ] Unit tests for message handling
- [ ] Update `component/mod.rs` with wrapper module

## Success Criteria
- [ ] `cargo build -p airssys-wasm` succeeds
- [ ] `cargo clippy -p airssys-wasm --all-targets -- -D warnings` passes
- [ ] Integrates with core/component/handle.rs (ComponentHandle)
- [ ] Uses Arc<dyn RuntimeEngine> for dependency injection
- [ ] Implements airssys-rt Actor and Child traits correctly
- [ ] Unit tests pass (lifecycle, message handling)

## Progress Tracking
**Overall Status:** 0% complete

## Progress Log
### 2026-01-22: Task Created
- Task created based on ADR-WASM-031 specification
- Part of Phase 6 - Component & Messaging Modules rebuild

## Standards Compliance Checklist
- [ ] §2.1 3-Layer Import Organization
- [ ] §2.2 No FQN in Type Annotations
- [ ] §4.3 Module Architecture Patterns
- [ ] ADR-WASM-023 Module Boundary Enforcement
- [ ] ADR-WASM-031 Component & Messaging Module Design

## Dependencies
- **Upstream:** Phase 5 complete (runtime/ module with RuntimeEngine trait)
- **Downstream:** WASM-TASK-038 (ComponentRegistry), WASM-TASK-039 (ComponentSpawner), WASM-TASK-040 (SupervisorConfig)

## Definition of Done
- [ ] All deliverables complete
- [ ] All success criteria met
- [ ] Build/Clippy pass with zero warnings
- [ ] Unit tests pass
- [ ] No forbidden module imports (verified via architecture checks)
- [ ] Trait-based dependency injection pattern followed
