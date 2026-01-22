# WASM-TASK-041: Implement Fire-and-Forget Pattern

**Status:** pending
**Added:** 2026-01-22
**Updated:** 2026-01-22
**Priority:** high
**Estimated Duration:** 2-3 hours
**Phase:** Phase 6 - Component & Messaging Modules (Layer 3)

## Original Request
Implement the fire-and-forget messaging pattern for component communication.

## Thought Process
FireAndForget pattern provides the simplest messaging pattern:
- Send message without waiting for response
- No correlation tracking needed
- Best for notifications, events, one-way commands
- Uses MessageSender trait for abstraction
- Supports asynchronous message delivery

This pattern is foundational for event-driven component communication and is used by higher-level patterns.

## Deliverables
- [ ] `messaging/patterns.rs` with FireAndForget struct
- [ ] FireAndForget::send() async method
- [ ] MessageSender trait definition (send, send_with_correlation)
- [ ] CorrelationManager trait definition (register, complete, is_pending)
- [ ] Unit tests for fire-and-forget send
- [ ] Unit tests for MessageSender trait
- [ ] Update `messaging/mod.rs` with patterns module

## Success Criteria
- [ ] `cargo build -p airssys-wasm` succeeds
- [ ] `cargo clippy -p airssys-wasm --all-targets -- -D warnings` passes
- [ ] FireAndForget::send() correctly uses MessageSender trait
- [ ] MessageSender trait is async and Send + Sync
- [ ] CorrelationManager trait defined (for WASM-TASK-042)
- [ ] Unit tests pass (send operations)

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
- **Upstream:** WASM-TASK-038 (ComponentRegistry needed for routing)
- **Downstream:** WASM-TASK-042 (RequestResponse pattern extends FireAndForget)

## Definition of Done
- [ ] All deliverables complete
- [ ] All success criteria met
- [ ] Build/Clippy pass with zero warnings
- [ ] Unit tests pass
- [ ] No forbidden module imports (verified via architecture checks)
- [ ] Trait-based abstraction pattern followed
