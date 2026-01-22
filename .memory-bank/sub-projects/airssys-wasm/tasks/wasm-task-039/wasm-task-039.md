# WASM-TASK-039: Implement ComponentSpawner

**Status:** pending
**Added:** 2026-01-22
**Updated:** 2026-01-22
**Priority:** high
**Estimated Duration:** 3-4 hours
**Phase:** Phase 6 - Component & Messaging Modules (Layer 3)

## Original Request
Implement the ComponentSpawner for spawning and managing component actors.

## Thought Process
ComponentSpawner orchestrates the entire component lifecycle:
- Loads component bytes via ComponentLoader trait
- Validates component before spawning
- Creates ComponentWrapper with injected RuntimeEngine
- Spawns as supervised actor in the ActorSystem
- Registers the component in ComponentRegistry
- Handles component shutdown

Key Design: Uses trait-based dependency injection for both RuntimeEngine and ComponentLoader, enabling flexible implementations and testing.

## Deliverables
- [ ] `component/spawner.rs` with ComponentSpawner struct
- [ ] Constructor accepting Arc<dyn RuntimeEngine>, Arc<dyn ComponentLoader>, Arc<ComponentRegistry>, SupervisorConfig
- [ ] spawn() method for component actor creation
- [ ] Integration with ActorSystem::spawn_supervised()
- [ ] Component validation before spawning
- [ ] Registry registration after successful spawn
- [ ] stop() method for component shutdown
- [ ] Unit tests for spawn lifecycle
- [ ] Unit tests for error handling
- [ ] Update `component/mod.rs` with spawner module

## Success Criteria
- [ ] `cargo build -p airssys-wasm` succeeds
- [ ] `cargo clippy -p airssys-wasm --all-targets -- -D warnings` passes
- [ ] Uses Arc<dyn RuntimeEngine> and Arc<dyn ComponentLoader> for dependency injection
- [ ] Integrates with ActorSystem and SupervisorConfig
- [ ] Validates components before spawning
- [ ] Registers components in registry
- [ ] Unit tests pass (spawn, stop, error cases)

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
- **Upstream:** WASM-TASK-037 (ComponentWrapper), WASM-TASK-038 (ComponentRegistry)
- **Downstream:** Phase 7 (System module will use ComponentSpawner)

## Definition of Done
- [ ] All deliverables complete
- [ ] All success criteria met
- [ ] Build/Clippy pass with zero warnings
- [ ] Unit tests pass
- [ ] No forbidden module imports (verified via architecture checks)
- [ ] Trait-based dependency injection pattern followed
