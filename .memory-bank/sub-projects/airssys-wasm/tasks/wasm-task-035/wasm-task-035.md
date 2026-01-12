# WASM-TASK-035: Implement ResourceLimiter

**Status:** pending
**Added:** 2026-01-12
**Updated:** 2026-01-12
**Priority:** high
**Estimated Duration:** 2-3 hours
**Phase:** Phase 5 - Runtime Module (Layer 2B)

## Original Request
Implement the ResourceLimiter for managing WASM execution resource limits (fuel, memory, timeout).

## Thought Process
ResourceLimiter bridges the `core/runtime/limits.rs::ResourceLimits` configuration with wasmtime's StoreLimits. It provides:
- Memory size limits
- Fuel consumption limits
- Execution timeout configuration

These are applied to WASM stores before execution.

## Deliverables
- [ ] `runtime/limiter.rs` with WasmResourceLimiter
- [ ] StoreLimits integration
- [ ] apply_limits helper function
- [ ] Unit tests for resource limiting
- [ ] Update `runtime/mod.rs` with limiter module

## Success Criteria
- [ ] `cargo build -p airssys-wasm` succeeds
- [ ] `cargo clippy -p airssys-wasm --all-targets -- -D warnings` passes
- [ ] Integrates with core/runtime/limits.rs
- [ ] StoreLimitsBuilder used correctly
- [ ] Unit tests pass

## Progress Tracking
**Overall Status:** 0% complete

## Progress Log
### 2026-01-12: Task Created
- Task created based on ADR-WASM-030 specification

## Standards Compliance Checklist
- [ ] §2.1 3-Layer Import Organization
- [ ] §4.3 Module Architecture Patterns
- [ ] ADR-WASM-030 Runtime Module Design

## Dependencies
- **Upstream:** WASM-TASK-031 (WasmtimeEngine)
- **Downstream:** Phase 6 (Component & Messaging)

## Definition of Done
- [ ] All deliverables complete
- [ ] All success criteria met
- [ ] Build/Clippy pass with zero warnings
- [ ] Unit tests pass
