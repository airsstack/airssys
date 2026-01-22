# WASM-TASK-038: Implement ComponentRegistry

**Status:** pending
**Added:** 2026-01-22
**Updated:** 2026-01-22
**Priority:** high
**Estimated Duration:** 2-3 hours
**Phase:** Phase 6 - Component & Messaging Modules (Layer 3)

## Original Request
Implement the ComponentRegistry for tracking loaded components and their actor addresses.

## Thought Process
ComponentRegistry maintains the mapping between ComponentId and ActorAddress. It:
- Provides thread-safe registration/lookup of components
- Maps ComponentId → ActorAddress for message routing
- Supports component lifecycle management (register/unregister)
- Enables component discovery (list, count, contains)
- Uses RwLock for concurrent read access with minimal write contention

This is a critical piece of infrastructure that enables the messaging module to route messages to the correct component actors.

## Deliverables
- [ ] `component/registry.rs` with ComponentRegistry struct
- [ ] Thread-safe HashMap<ComponentId, ActorAddress> with RwLock
- [ ] register() method for adding components
- [ ] unregister() method for removing components
- [ ] get() method for address lookup
- [ ] contains() method for existence check
- [ ] list() method for listing all component IDs
- [ ] count() method for registry size
- [ ] Default trait implementation
- [ ] Unit tests for all registry operations
- [ ] Update `component/mod.rs` with registry module

## Success Criteria
- [ ] `cargo build -p airssys-wasm` succeeds
- [ ] `cargo clippy -p airssys-wasm --all-targets -- -D warnings` passes
- [ ] Thread-safe operations (RwLock usage)
- [ ] All CRUD operations implemented
- [ ] Unit tests pass (register, unregister, get, list, count, contains)

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
- **Upstream:** WASM-TASK-037 (ComponentWrapper)
- **Downstream:** WASM-TASK-039 (ComponentSpawner), WASM-TASK-041 (FireAndForget pattern)

## Definition of Done
- [ ] All deliverables complete
- [ ] All success criteria met
- [ ] Build/Clippy pass with zero warnings
- [ ] Unit tests pass
- [ ] No forbidden module imports (verified via architecture checks)
- [ ] Thread-safety verified
