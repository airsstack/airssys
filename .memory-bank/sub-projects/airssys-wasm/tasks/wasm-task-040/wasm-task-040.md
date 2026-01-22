# WASM-TASK-040: Implement SupervisorConfig

**Status:** pending
**Added:** 2026-01-22
**Updated:** 2026-01-22
**Priority:** high
**Estimated Duration:** 2-3 hours
**Phase:** Phase 6 - Component & Messaging Modules (Layer 3)

## Original Request
Implement the SupervisorConfig for component actor supervision configuration.

## Thought Process
SupervisorConfig provides fault-tolerance configuration for component actors:
- Defines restart policies (max_restarts, restart_window)
- Configures backoff strategies (Fixed, Exponential)
- Maps to airssys-rt SupervisorNode
- Provides sensible defaults for production use
- Enables customization per component if needed

This is critical for building reliable component systems that can recover from failures automatically.

## Deliverables
- [ ] `component/supervisor.rs` with SupervisorConfig struct
- [ ] BackoffStrategy enum (Fixed, Exponential)
- [ ] Fields: max_restarts, restart_window, backoff_strategy
- [ ] Default trait implementation (3 restarts, 60s window, exponential backoff)
- [ ] new() constructor
- [ ] with_backoff() builder method
- [ ] to_supervisor_node() method for airssys-rt integration
- [ ] Unit tests for config creation
- [ ] Unit tests for builder pattern
- [ ] Unit tests for default values
- [ ] Update `component/mod.rs` with supervisor module

## Success Criteria
- [ ] `cargo build -p airssys-wasm` succeeds
- [ ] `cargo clippy -p airssys-wasm --all-targets -- -D warnings` passes
- [ ] Default configuration is production-ready
- [ ] Builder pattern for customization
- [ ] Maps to airssys-rt SupervisorNode correctly
- [ ] Unit tests pass (creation, builder, defaults)

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
- **Downstream:** WASM-TASK-039 (ComponentSpawner uses SupervisorConfig)

## Definition of Done
- [ ] All deliverables complete
- [ ] All success criteria met
- [ ] Build/Clippy pass with zero warnings
- [ ] Unit tests pass
- [ ] No forbidden module imports (verified via architecture checks)
- [ ] Production-ready default configuration
