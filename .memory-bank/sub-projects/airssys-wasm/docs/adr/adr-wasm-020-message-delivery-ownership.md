# ADR-WASM-020: Message Delivery Ownership Architecture

**ADR ID:** ADR-WASM-020  
**Created:** 2025-12-21  
**Status:** Accepted  
**Deciders:** Architecture Review  
**Category:** Communication Architecture / Message Routing

---

## Title

Message Delivery Ownership: ActorSystemSubscriber Owns Delivery, ComponentRegistry Stays Pure

---

## Context

### Problem Statement

WASM-TASK-006 (Block 5 - Inter-Component Communication) Phase 1 Task 1.1 implementation revealed that `ActorSystemSubscriber::route_message_to_subscribers()` was **stubbed** - it extracted the target `ComponentId` from messages but never actually delivered messages to target component mailboxes.

The root cause: `ActorAddress` (stored in `ComponentRegistry`) is an **identifier**, not a **sender**. It provides no mechanism to actually send messages to an actor's mailbox.

### Technical Context

**Current Architecture (Broken):**
```
ComponentRegistry
    └── ComponentId → ActorAddress (identifier only, no send capability)

ActorSystemSubscriber
    ├── Subscribes to MessageBroker
    ├── Receives messages
    ├── Extracts target ComponentId
    ├── Looks up ActorAddress (useless for sending)
    └── STUBBED: Cannot deliver messages
```

**ActorAddress Definition (airssys-rt):**
```rust
pub enum ActorAddress {
    Named { id: ActorId, name: String },
    Anonymous { id: ActorId },
}
// NO send() method - purely an identifier
```

### Business Context

- Block 5 (Inter-Component Communication) is foundational for WASM component interaction
- Without working message delivery, components cannot communicate
- Performance targets: <20μs end-to-end message delivery
- Must maintain clean architectural boundaries established in ADR-WASM-018

### Stakeholders

- Block 5 implementers (immediate)
- All future WASM component developers (long-term)
- Runtime/architecture maintainers

---

## Decision

### Summary

**`ActorSystemSubscriber` owns message delivery. `ComponentRegistry` stays pure (identity lookup only).**

Specifically:
1. `ActorSystemSubscriber` maintains a `mailbox_senders: HashMap<ComponentId, MailboxSender<ComponentMessage>>` map
2. When a `ComponentActor` is spawned, its `MailboxSender` is registered with `ActorSystemSubscriber`
3. `ActorSystemSubscriber::route_message_to_subscribers()` uses this map to deliver messages
4. `ComponentRegistry` remains unchanged - purely for `ComponentId → ActorAddress` lookup

### Rationale

1. **Single Responsibility Principle:**
   - `ComponentRegistry` = "Who exists and what's their address" (identity)
   - `ActorSystemSubscriber` = "How to deliver messages to them" (delivery)
   - Each component has one clear responsibility

2. **ADR-WASM-009 Alignment:**
   - ADR-WASM-009 establishes "ActorSystem as Primary Subscriber" pattern
   - The subscriber receives ALL messages and routes them
   - Delivery is explicitly a subscriber responsibility, not registry responsibility

3. **ADR-WASM-018 Layer Compliance:**
   - `ComponentRegistry` = Layer 2 (Component lifecycle)
   - `ActorSystemSubscriber` = Layer 2/3 boundary (Message routing)
   - Keeping delivery in subscriber maintains clean layer separation

4. **Future Extensibility:**
   - Topic-based routing can be added to subscriber
   - Load balancing can be added to subscriber
   - Registry stays simple and unchanged

### Assumptions

- Each `ComponentActor` has exactly one mailbox
- `MailboxSender` is cloneable and thread-safe
- Component spawn and shutdown are coordinated (no orphan senders)

---

## Considered Options

### Option 1: Extend ComponentRegistry (REJECTED)

**Description:** Store `MailboxSender` alongside `ActorAddress` in `ComponentRegistry`. Add `registry.send_to(component_id, message)` method.

**Pros:**
- Single lookup per message delivery
- Contained change (one module modified)
- Simple registration (one call registers both)

**Cons:**
- ❌ Violates Single Responsibility Principle
- ❌ Mixes concerns (addressing + delivery)
- ❌ Deviates from ADR-WASM-009 design
- ❌ ComponentRegistry becomes a "god object"
- ❌ Future refactoring required

**Implementation Effort:** Low  
**Risk Level:** Medium (architectural debt)

**Verdict:** REJECTED - Clean architecture is more important than implementation simplicity.

### Option 2: Create Separate MailboxRegistry (CONSIDERED)

**Description:** Create a new `MailboxRegistry` module that maps `ComponentId → MailboxSender`. `ActorSystemSubscriber` uses both registries.

**Pros:**
- Pure separation of concerns
- ComponentRegistry unchanged
- MailboxRegistry independently testable

**Cons:**
- Two lookups per message (address + sender)
- Two registries to keep synchronized
- More complex spawn/shutdown coordination
- Additional module to maintain

**Implementation Effort:** Medium  
**Risk Level:** Medium (synchronization complexity)

**Verdict:** CONSIDERED but not chosen - adds unnecessary complexity.

### Option 3: ActorSystemSubscriber Owns MailboxSenders (ACCEPTED)

**Description:** `ActorSystemSubscriber` maintains `mailbox_senders: HashMap<ComponentId, MailboxSender>`. Components register their sender with subscriber on spawn.

**Pros:**
- ✅ Single Responsibility: Registry = identity, Subscriber = delivery
- ✅ ADR-WASM-009 compliant: Subscriber handles routing AND delivery
- ✅ ADR-WASM-018 compliant: Clear layer boundaries
- ✅ Single lookup in subscriber for delivery
- ✅ ComponentRegistry completely unchanged
- ✅ Natural extension of subscriber responsibility

**Cons:**
- Spawn/shutdown must coordinate with subscriber (manageable)
- Subscriber becomes more complex (appropriate complexity)

**Implementation Effort:** Medium  
**Risk Level:** Low (architecturally sound)

**Verdict:** ACCEPTED - Best alignment with existing architecture.

---

## Implementation

### Implementation Plan

1. **Modify ActorSystemSubscriber:**
   - Add `mailbox_senders: Arc<RwLock<HashMap<ComponentId, MailboxSender<ComponentMessage>>>>` field
   - Add `register_mailbox(component_id, sender)` method
   - Add `unregister_mailbox(component_id)` method
   - Fix `route_message_to_subscribers()` to use `mailbox_senders`

2. **Update ComponentSpawner:**
   - After creating mailbox channel, register sender with `ActorSystemSubscriber`
   - On component shutdown, unregister from `ActorSystemSubscriber`

3. **Add Tests:**
   - Unit tests for mailbox registration/unregistration
   - Unit tests for message delivery via subscriber
   - Integration tests for end-to-end message flow

4. **Documentation:**
   - Update KNOWLEDGE-WASM-026 (already done)
   - Update task remediation plan
   - Update progress tracking

### Timeline

- **Phase 1 (Task 1.1 Remediation):** Implement core delivery mechanism
- **Phase 2 (Task 1.3):** Add topic-based routing using `SubscriberManager`
- **Phase 3+:** Performance optimization, metrics integration

### Resources Required

- Modifications to `actor_system_subscriber.rs`
- Modifications to component spawner (wherever that lives)
- New tests in `tests/` directory

### Dependencies

- ADR-WASM-009: Component Communication Model (messaging patterns)
- ADR-WASM-018: Three-Layer Architecture (layer boundaries)
- WASM-TASK-006: Block 5 implementation context

---

## Implications

### System Impact

- **Positive:** Fixes broken message delivery in Block 5
- **Neutral:** No changes to ComponentRegistry API
- **Consideration:** ComponentSpawner must coordinate with ActorSystemSubscriber

### Performance Impact

| Operation | Latency | Impact |
|-----------|---------|--------|
| Register mailbox | <1μs | One-time on spawn |
| Unregister mailbox | <1μs | One-time on shutdown |
| Message lookup | <100ns | HashMap read per message |
| Message send | ~100ns | Tokio channel send |
| **Total overhead** | **<200ns** | Within targets |

### Security Impact

- No security impact - delivery mechanism is internal to runtime
- Capability checks happen before message reaches subscriber

### Scalability Impact

- HashMap scales O(1) for lookup
- RwLock allows concurrent reads (most operations are reads)
- Write contention only during spawn/shutdown (rare)

### Maintainability Impact

- **Positive:** Clean separation of concerns
- **Positive:** Each component has single responsibility
- **Positive:** Easier to test in isolation

---

## Compliance

### Workspace Standards

- **§4.3 Module Architecture:** Subscriber maintains clear responsibility
- **§2.1 Import Organization:** Standard import patterns used
- **§5.1 Testing:** Unit and integration tests required

### ADR Alignment

- **ADR-WASM-009:** Subscriber pattern for message routing ✅
- **ADR-WASM-018:** Layer 2/3 boundary for routing ✅
- **ADR-WASM-006:** Actor-based component isolation ✅

### Technical Debt

- **Debt Created:** None - architecturally sound design
- **Debt Resolved:** Fixes stubbed message delivery

---

## Monitoring and Validation

### Success Criteria

1. Messages sent from Component A arrive at Component B's `handle_message` export
2. End-to-end latency < 20μs (Block 5 target)
3. All existing tests pass
4. New tests prove actual delivery (not just stub)

### Key Metrics

- Message delivery success rate (target: 100% for reachable components)
- Message delivery latency (target: <20μs)
- Mailbox registration count (monitoring)

### Review Schedule

- Post-implementation review after Task 1.1 remediation
- Performance review after Block 5 Phase 1 completion

---

## Risks and Mitigations

### Identified Risks

| Risk | Probability | Impact | Mitigation |
|------|-------------|--------|------------|
| Orphan mailbox senders | Low | Medium | Ensure unregister on all shutdown paths |
| RwLock contention | Low | Low | Reads far outnumber writes |
| Sender/receiver mismatch | Low | High | Registration happens atomically with spawn |

### Contingency Plans

- If performance issues arise: Consider sharded HashMap or DashMap
- If synchronization issues: Add health checks for sender validity
- If complexity grows: Can extract to dedicated DeliveryService later

---

## References

### Related ADRs

- **ADR-WASM-009:** Component Communication Model (messaging patterns)
- **ADR-WASM-018:** Three-Layer Architecture (layer boundaries)
- **ADR-WASM-006:** Component Isolation and Sandboxing (actor pattern)
- **ADR-WASM-019:** Runtime Dependency Management (Tokio usage)

### Knowledge Documents

- **KNOWLEDGE-WASM-026:** Message Delivery Architecture - Final Decision (detailed implementation)
- **KNOWLEDGE-WASM-025:** (SUPERSEDED by KNOWLEDGE-WASM-026)
- **KNOWLEDGE-WASM-024:** Component Messaging Clarifications
- **KNOWLEDGE-WASM-018:** Component Definitions and Architecture Layers
- **KNOWLEDGE-WASM-005:** Inter-Component Messaging Architecture

### Task Documentation

- **WASM-TASK-006:** Block 5 - Inter-Component Communication
- **Task 1.1 Remediation Plan:** Implementation steps

### Source Files

- `airssys-wasm/src/actor/message/actor_system_subscriber.rs` - Main implementation
- `airssys-wasm/src/actor/component/component_registry.rs` - Unchanged (pure identity)
- `airssys-wasm/src/actor/component/component_actor.rs` - Mailbox receiver

---

## History

### Status Changes

| Date | Status | Reason |
|------|--------|--------|
| 2025-12-21 | Accepted | Architectural review approved decision |

### Updates

| Date | Description |
|------|-------------|
| 2025-12-21 | Initial ADR created |

---

**End of ADR-WASM-020**
