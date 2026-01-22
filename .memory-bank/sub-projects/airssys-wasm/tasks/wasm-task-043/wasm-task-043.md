# WASM-TASK-043: Implement CorrelationTracker

**Status:** pending
**Added:** 2026-01-22
**Updated:** 2026-01-22
**Priority:** high
**Estimated Duration:** 3-4 hours
**Phase:** Phase 6 - Component & Messaging Modules (Layer 3)

## Original Request
Implement the CorrelationTrackerImpl for request-response correlation tracking.

## Thought Process
CorrelationTrackerImpl manages pending requests and their responses:
- Tracks pending requests with correlation IDs
- Uses oneshot channels for response delivery
- Implements timeout-based expiration
- Provides cleanup for expired correlations
- Thread-safe with RwLock
- Implements the CorrelationTracker trait from core/

This is the concrete implementation that enables request-response patterns to work reliably.

## Deliverables
- [ ] `messaging/correlation.rs` with CorrelationTrackerImpl struct
- [ ] PendingRequest internal struct (sender, deadline)
- [ ] Thread-safe HashMap with RwLock
- [ ] create() method returning oneshot::Receiver
- [ ] cleanup_expired() method for timeout handling
- [ ] CorrelationTracker trait implementation (register, complete, is_pending)
- [ ] Default trait implementation
- [ ] Unit tests for correlation registration
- [ ] Unit tests for correlation completion
- [ ] Unit tests for timeout expiration
- [ ] Unit tests for cleanup
- [ ] Update `messaging/mod.rs` with correlation module

## Success Criteria
- [ ] `cargo build -p airssys-wasm` succeeds
- [ ] `cargo clippy -p airssys-wasm --all-targets -- -D warnings` passes
- [ ] Thread-safe correlation tracking
- [ ] Timeout-based expiration works correctly
- [ ] oneshot channels for response delivery
- [ ] Implements CorrelationTracker trait from core/
- [ ] Unit tests pass (register, complete, timeout, cleanup)

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
- **Upstream:** WASM-TASK-042 (RequestResponse pattern)
- **Downstream:** WASM-TASK-044 (ResponseRouter uses correlation tracking)

## Definition of Done
- [ ] All deliverables complete
- [ ] All success criteria met
- [ ] Build/Clippy pass with zero warnings
- [ ] Unit tests pass
- [ ] No forbidden module imports (verified via architecture checks)
- [ ] Thread-safety verified
- [ ] Timeout handling verified
