# WASM-TASK-044: Implement ResponseRouter

**Status:** pending
**Added:** 2026-01-22
**Updated:** 2026-01-22
**Priority:** high
**Estimated Duration:** 3-4 hours
**Phase:** Phase 6 - Component & Messaging Modules (Layer 3)

## Original Request
Implement the ResponseRouter and ComponentSubscriber for message routing and delivery.

## Thought Process
ResponseRouter + ComponentSubscriber complete the messaging infrastructure:
- ResponseRouter implements MessageRouter trait for inter-component messaging
- ComponentSubscriber manages mailbox senders for each component
- Routes messages to correct component via registry lookup
- Creates ComponentMessage with metadata (correlation, reply_to, timestamp)
- Integrates with airssys-rt for actual message delivery

These components bridge the gap between high-level messaging patterns and low-level actor system delivery.

## Deliverables
- [ ] `messaging/router.rs` with ResponseRouter struct
- [ ] ResponseRouter::new() constructor accepting registry and current_component
- [ ] create_message() helper for ComponentMessage creation
- [ ] MessageRouter trait implementation (send, request, cancel_request)
- [ ] `messaging/subscriber.rs` with ComponentSubscriber struct
- [ ] ComponentSubscriber mailbox sender management
- [ ] register_mailbox() and unregister_mailbox() methods
- [ ] deliver() method for message delivery
- [ ] Unit tests for ResponseRouter
- [ ] Unit tests for ComponentSubscriber
- [ ] Update `messaging/mod.rs` with router and subscriber modules

## Success Criteria
- [ ] `cargo build -p airssys-wasm` succeeds
- [ ] `cargo clippy -p airssys-wasm --all-targets -- -D warnings` passes
- [ ] ResponseRouter implements MessageRouter trait from core/
- [ ] ComponentSubscriber manages mailbox senders correctly
- [ ] Message routing uses registry for target lookup
- [ ] ComponentMessage creation includes all metadata
- [ ] Unit tests pass (routing, subscription, delivery)

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
- **Upstream:** WASM-TASK-043 (CorrelationTracker for correlation support)
- **Downstream:** Phase 7 (System module will wire up routing)

## Definition of Done
- [ ] All deliverables complete
- [ ] All success criteria met
- [ ] Build/Clippy pass with zero warnings
- [ ] Unit tests pass
- [ ] No forbidden module imports (verified via architecture checks)
- [ ] MessageRouter trait implementation complete
- [ ] Mailbox management verified
