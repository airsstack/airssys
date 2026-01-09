# WASM-TASK-019: Create core/messaging/ Submodule

**Status:** pending  
**Added:** 2026-01-08  
**Updated:** 2026-01-09  
**Priority:** high  
**Estimated Duration:** 1-2 hours  
**Phase:** Phase 3 - Core Module (Layer 1)

## Original Request
Create the `core/messaging/` submodule containing messaging routing traits and correlation types per ADR-WASM-028.

## Thought Process
This task creates the messaging-related core abstractions for routing and correlation patterns. Key types include:
- `MessageRouter` trait - Message routing abstraction
- `CorrelationTracker` trait - Request-response correlation
- `CorrelationId` - Correlation identifier types

**Note:** `MessagePayload` is now defined in `core/component/message.rs` per ADR-WASM-028 v1.1.

## Deliverables
- [ ] `core/messaging/mod.rs` created with module declarations
- [ ] `core/messaging/errors.rs` with `MessagingError` enum (co-located)
- [ ] `core/messaging/correlation.rs` with correlation ID types
- [ ] `core/messaging/traits.rs` with `MessageRouter` and `CorrelationTracker` traits
- [ ] `core/mod.rs` updated to export messaging submodule

## Success Criteria
- [ ] `cargo build -p airssys-wasm` succeeds
- [ ] `cargo clippy -p airssys-wasm --all-targets -- -D warnings` passes
- [ ] Traits reference `MessagePayload` from `core/component/`
- [ ] `MessagingError` co-located in `core/messaging/errors.rs`
- [ ] All types properly documented with rustdoc
- [ ] Types align with ADR-WASM-028 specifications

## Progress Tracking
**Overall Status:** 0% complete

## Progress Log
*(No progress yet)*

## Standards Compliance Checklist
- [ ] **§2.1 3-Layer Import Organization** - Only std and core/ imports
- [ ] **§4.3 Module Architecture Patterns** - mod.rs only declarations
- [ ] **ADR-WASM-028 v1.1** - Core module structure compliance
- [ ] **ADR-WASM-025** - Clean-slate rebuild architecture
- [ ] **KNOWLEDGE-WASM-037** - Technical reference alignment

## Dependencies
- **Upstream:** 
  - WASM-TASK-017 (core/component/) - for ComponentId, MessagePayload
  - ~~WASM-TASK-022 (core/errors/)~~ - **ABANDONED**: MessagingError now co-located
- **Downstream:** WASM-TASK-024 (Core unit tests), Phase 6 messaging implementation

## Definition of Done
- [ ] All deliverables complete
- [ ] All success criteria met
- [ ] Build passes with zero warnings
- [ ] Messaging abstractions ready for implementation

