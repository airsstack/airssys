# ADR-WASM-014: RoutingStrategy Trait Removal

**ADR ID:** ADR-WASM-014  
**Created:** 2025-10-22  
**Updated:** 2025-10-22  
**Status:** Accepted  
**Deciders:** AirsSys Architecture Team  

## Title
Remove RoutingStrategy Trait - Routing Handled by airssys-rt MessageBroker Architecture

## Context

### Problem Statement

During Phase 8 implementation (Messaging Abstractions), a `RoutingStrategy` trait was implemented in `airssys-wasm/src/core/messaging.rs` (lines 484-588). However, **ADR-WASM-009 Component Communication Model specifies that routing is handled exclusively by airssys-rt MessageBroker**, not by pluggable strategy abstractions.

This created critical issues:

1. **Architecture Conflict**: ADR-WASM-009 §3 specifies MessageBroker (211ns routing, 4.7M msg/sec) as the routing mechanism, not pluggable strategies
2. **Fictional Documentation**: Trait docs list DirectRoutingStrategy, TopicRoutingStrategy, BroadcastRoutingStrategy, CustomRoutingStrategy - **NONE exist**
3. **Zero Usage**: Only 1 test implementation (`TestRouter` in tests), no production implementations
4. **YAGNI Violation**: ~104 lines of speculative abstraction with no actual requirement

### Business Context

**Component Communication Requirements:**
- Direct messaging between components (fire-and-forget, request-response)
- Pub-sub topic-based communication
- Host-mediated security enforcement
- Performance targets: >3M msg/sec throughput, <260ns overhead

**ADR-WASM-009 Architecture Provides:**
- airssys-rt `InMemoryMessageBroker` handles all routing (211ns per message)
- ActorSystem performs topic subscription management and direct lookups
- Host functions enforce security at message entry point
- Message flow: `Component → Host Function → MessageBroker.publish() → ActorSystem subscriber → ComponentActor mailbox`

### Technical Context

**Current Implementation:**
- `RoutingStrategy` trait with single `route(&MessageEnvelope) -> WasmResult<()>` method (lines 484-588)
- Public API export in `core/mod.rs` line 70
- Extensive rustdoc listing fictional implementations (DirectRoutingStrategy, TopicRoutingStrategy, etc.)
- Only `TestRouter` in tests exists (line 927-940, 14 lines)

**ADR-WASM-009 Architecture Decision:**
- **Decision 1**: Use airssys-rt `InMemoryMessageBroker` for routing (211ns routing, 4.7M msg/sec)
- **Decision 2**: Pure pub-sub architecture (no custom routing strategies)
- **Decision 3**: ActorSystem as primary subscriber (does component resolution)
- **Decision 4**: Host-mediated security model (components cannot bypass validation)

**MessageBroker Internal Routing:**
```rust
// From RT-TASK-008 benchmarks
MessageBroker::publish(topic, message) {
    // 1. Lookup subscribers for topic (~100ns HashMap lookup)
    // 2. Send to each subscriber mailbox (~111ns per message)
    // 3. Return immediately (fire-and-forget)
}
// Total: ~211ns per message, no pluggable strategy needed
```

**ActorSystem Resolution Pattern:**
```rust
// ActorSystem handles routing internally
ActorSystem::route_to_component(component_id, message) {
    // 1. Lookup ComponentActor by ID (~50ns direct lookup)
    // 2. Send to actor mailbox (111ns)
    // Total: ~161ns, fixed routing strategy
}
```

**Workspace Standards Alignment:**
- **§6.1 YAGNI Principles**: "Build only what's needed", "Avoid speculative generalization"
- **§6.2 Avoid dyn Patterns**: Trait uses `Box<dyn>` anti-pattern (would need dynamic dispatch)
- **Microsoft Rust Guidelines M-SIMPLE-ABSTRACTIONS**: Avoid unnecessary abstraction layers

### Stakeholders

- **Component Developers**: Use host functions for messaging, unaffected
- **Runtime Engineers**: Implement MessageBroker and ActorSystem routing
- **Framework Maintainers**: Maintain messaging abstractions
- **Security Team**: Enforce host-mediated security model

## Decision

### Summary

**Remove `RoutingStrategy` trait and all related code** from airssys-wasm messaging abstractions. Routing is handled exclusively by airssys-rt MessageBroker and ActorSystem per ADR-WASM-009 architecture.

**Rationale:**

1. **ADR-WASM-009 Specifies Fixed Architecture**: MessageBroker handles routing, no pluggable strategies in design
2. **Zero Usage**: Only test implementation exists, no production implementations
3. **Fictional Documentation**: All listed implementations (DirectRoutingStrategy, TopicRoutingStrategy, etc.) are fictional
4. **Security Model Conflict**: Host-mediated security prevents custom routing (components cannot implement custom strategies that bypass security)
5. **YAGNI Violation**: Speculative abstraction with no identified use case
6. **Similar to StorageTransaction**: Same anti-pattern as ADR-WASM-013 removal

### Assumptions

1. airssys-rt MessageBroker provides sufficient routing capabilities (pub-sub, direct messaging)
2. ActorSystem handles component resolution and topic subscription management
3. Host functions enforce all security policies at message entry point
4. No custom routing strategies needed beyond MessageBroker's built-in pub-sub
5. Component-to-component routing is mediated by host runtime (no direct routing)

## Considered Options

### Option 1: Remove RoutingStrategy Trait (SELECTED)

**Description**: Completely remove `RoutingStrategy` trait, related documentation, and test implementation. Rely on MessageBroker and ActorSystem for routing.

**Pros:**
- Aligns implementation with ADR-WASM-009 architectural decision
- YAGNI compliance (removes unused speculative code)
- Simplifies messaging API surface (~115 lines removed)
- Eliminates fictional documentation (DirectRoutingStrategy, TopicRoutingStrategy don't exist)
- Removes dyn pattern anti-pattern
- MessageBroker provides all required routing capabilities
- Security model enforced at host function layer (no custom routing bypass)

**Cons:**
- Cannot implement custom routing strategies outside MessageBroker
- Re-adding later requires trait reintroduction (non-trivial)
- Component developers cannot customize routing behavior

**Implementation Effort:** Low (code removal, documentation updates)  
**Risk Level:** Low (zero current usage, MessageBroker architecture proven)

### Option 2: Keep RoutingStrategy Trait

**Description**: Keep trait for future extensibility, require actual implementations for documented strategies.

**Pros:**
- Enables custom routing strategies if future use cases emerge
- Allows backend-specific routing optimizations
- Familiar strategy pattern for developers

**Cons:**
- Violates ADR-WASM-009 fixed MessageBroker architecture
- YAGNI violation (no current need)
- Requires implementing all documented strategies (DirectRoutingStrategy, TopicRoutingStrategy, etc.)
- Contradicts host-mediated security model (components cannot implement strategies)
- Maintains fictional documentation
- MessageBroker already provides routing (211ns, 4.7M msg/sec)

**Implementation Effort:** High (implement all documented strategies)  
**Risk Level:** Medium (architectural divergence, security model conflict)

### Option 3: Make RoutingStrategy Internal Implementation Detail

**Description**: Keep trait but make it internal (not public API), use within MessageBroker implementation.

**Pros:**
- Provides internal abstraction for MessageBroker variants
- Not exposed to component developers (no security risk)
- Maintains flexibility for runtime implementation changes

**Cons:**
- MessageBroker already has internal routing logic (no trait needed)
- YAGNI violation (MessageBroker doesn't use strategy pattern)
- Maintains unused code path
- Current MessageBroker implementation is HashMap-based pub-sub (no strategy needed)

**Implementation Effort:** Low (change to `pub(crate)` visibility)  
**Risk Level:** Low (isolated to runtime internals)

## Implementation

### Implementation Plan

**Phase 1: Code Removal (Immediate)**
1. Remove `RoutingStrategy` trait (messaging.rs:484-588, ~104 lines)
2. Remove `RoutingStrategy` from `core/mod.rs` public exports (line 70)
3. Remove `test_routing_strategy_trait_object` test (messaging.rs:927-940, ~14 lines)
4. Update messaging.rs module documentation (line 14, remove RoutingStrategy reference)

**Phase 2: Documentation Updates (Immediate)**
1. Update KNOWLEDGE-WASM-005: Confirm routing handled by MessageBroker, no custom strategies
2. Update ADR-WASM-009: Add confirmation note that implementation followed MessageBroker architecture
3. Create ADR-WASM-014: Document removal decision and reasoning (this document)

**Phase 3: Validation (Immediate)**
1. Run `cargo test -p airssys-wasm` - expect 254 tests passing (1 test removed)
2. Run `cargo clippy -p airssys-wasm --all-targets --all-features` - expect zero warnings
3. Verify no external references to `RoutingStrategy`
4. Update memory bank progress tracking

### Timeline

- **Phase 1-3**: 1-2 hours (same session)
- **Total**: Immediate completion (2025-10-22)

### Resources Required

- AI agent development time: 1-2 hours
- No additional tools or infrastructure needed

### Dependencies

- **ADR-WASM-009**: Component Communication Model provides architectural foundation
- **Phase 8 Completion**: Messaging abstractions already implemented
- **airssys-rt Integration**: MessageBroker provides routing infrastructure

## Implications

### System Impact

**Positive:**
- Simplified messaging abstraction API
- Clearer routing model (MessageBroker-only, no custom strategies)
- Aligned implementation with architectural decisions
- Eliminated fictional documentation

**Neutral:**
- No impact on component developers (never used RoutingStrategy)
- Routing capabilities unchanged (MessageBroker provides)

**Negative:**
- None identified (MessageBroker provides all required routing)

### Performance Impact

**Positive:**
- No abstraction overhead (direct MessageBroker calls)
- No dynamic dispatch penalty (no dyn RoutingStrategy)
- MessageBroker routing remains at 211ns per message

**Neutral:**
- No performance change for components (never used trait)

### Security Impact

**Positive:**
- Reinforces host-mediated security model (no custom routing bypass)
- Components cannot implement routing strategies that skip security checks
- All messages routed through host function security layer

**Neutral:**
- Security enforcement unchanged (always at host function layer)

### Scalability Impact

**Positive:**
- MessageBroker scales to >3M msg/sec throughput
- Fixed routing architecture optimizes for pub-sub pattern
- No strategy selection overhead

### Maintainability Impact

**Positive:**
- Less code to maintain (~115 lines removed)
- Clearer mental model (MessageBroker-only routing)
- Eliminated fictional documentation maintenance
- No strategy pattern complexity

## Compliance

### Workspace Standards

**§6.1 YAGNI Principles:**
- ✅ "Build only what's needed" - Custom routing not needed (MessageBroker suffices)
- ✅ "Avoid speculative generalization" - Trait was speculative future-proofing
- ✅ "Remove unused complexity" - Only test implementation exists

**§6.2 Avoid dyn Patterns:**
- ✅ Would have required `Box<dyn RoutingStrategy>` pattern
- ✅ Removes trait object pattern from messaging API

**§4.3 Module Architecture:**
- ✅ Cleaner trait boundaries in messaging.rs
- ✅ Reduced API surface in core module

**Microsoft Rust Guidelines:**
- ✅ **M-SIMPLE-ABSTRACTIONS**: Removes unnecessary abstraction layer
- ✅ **M-YAGNI**: Eliminates speculative capability
- ✅ **M-DI-HIERARCHY**: Prefer concrete types (MessageBroker) over traits

### Technical Debt

**Debt Resolved:**
- **DEBT-001**: Documentation lists fictional implementations (DirectRoutingStrategy, TopicRoutingStrategy, etc.)
- **DEBT-002**: Implementation divergence from ADR-WASM-009 MessageBroker architecture
- **DEBT-003**: YAGNI violation with unused trait implementation

**Debt Created:**
- None (removal aligns with architecture, no compromises made)

## Monitoring and Validation

### Success Criteria

1. ✅ All code removed successfully (zero compilation errors)
2. ✅ All tests pass (cargo test -p airssys-wasm, 254 tests expected)
3. ✅ Zero warnings (cargo clippy -p airssys-wasm)
4. ✅ Documentation synchronized (ADR-WASM-009 ↔ KNOWLEDGE-WASM-005)
5. ✅ No external references to `RoutingStrategy` remain

### Key Metrics

- **Code Reduction**: ~115 lines removed (104 trait + 11 test)
- **API Surface**: 1 fewer public trait in core/messaging
- **Test Count**: 254 tests (1 test removed)
- **Build Time**: Marginal improvement (less code to compile)

### Review Schedule

- **Immediate**: Validate removal during implementation
- **Phase 9 Start**: Confirm messaging API ergonomics for lifecycle management
- **6 months**: Review if any use cases emerged requiring custom routing

## Risks and Mitigations

### Identified Risks

**Risk 1: Custom Routing Requirements Emerge**
- **Likelihood**: Very Low
- **Impact**: Medium (trait re-introduction needed)
- **Mitigation**: MessageBroker pub-sub pattern handles all identified use cases (fire-and-forget, request-response, topic-based). ActorSystem provides component resolution. No use cases identified requiring custom routing.
- **Evidence**: ADR-WASM-009 analysis validated pub-sub sufficiency

**Risk 2: Backend-Specific Routing Optimizations Lost**
- **Likelihood**: Very Low
- **Impact**: Low (marginal performance)
- **Mitigation**: MessageBroker already optimized (211ns routing, 4.7M msg/sec). Backend-specific optimizations can be implemented within MessageBroker internals without exposing strategy trait.

**Risk 3: Component Developers Want Custom Routing**
- **Likelihood**: Very Low
- **Impact**: Low (design constraint)
- **Mitigation**: Host-mediated security model prevents custom routing (security requirement). Components must use host functions for messaging. Custom logic can be implemented at message handler level, not routing level.

### Contingency Plans

**If Custom Routing Support Needed Later:**
1. Assess specific use case and requirements
2. Validate MessageBroker pub-sub pattern cannot provide equivalent functionality
3. Design routing extension based on actual requirements (not speculation)
4. Ensure compatibility with host-mediated security model
5. Update ADR-WASM-014 status to "Superseded"
6. Create new ADR documenting routing extension rationale

**If MessageBroker Architecture Changes:**
1. Review ADR-WASM-009 for architectural impact
2. Consider internal routing abstractions within airssys-rt (not public API)
3. Keep component-facing API unchanged (host functions remain interface)

## References

### Related Documents

**ADRs:**
- **ADR-WASM-009**: Component Communication Model (MessageBroker architecture)
- **ADR-WASM-006**: Component Isolation and Sandboxing (actor-based routing)
- **ADR-WASM-005**: Capability-Based Security Model (host-mediated security)
- **ADR-WASM-013**: StorageTransaction Removal (similar YAGNI pattern)

**Knowledge Docs:**
- **KNOWLEDGE-WASM-005**: Inter-Component Messaging Architecture (§5 Routing Architecture)
- **RT-TASK-008**: MessageBroker Performance Benchmarks (211ns routing)

**Workspace Standards:**
- **§6.1**: YAGNI Principles (build only what's needed)
- **§6.2**: Avoid dyn Patterns (removes trait object pattern)
- **§4.3**: Module Architecture (cleaner trait boundaries)

**External References:**
- airssys-rt MessageBroker pub-sub architecture
- Actor model message routing patterns

### Code References

**Files Modified:**
- `airssys-wasm/src/core/messaging.rs` (trait removal, test removal, docs update)
- `airssys-wasm/src/core/mod.rs` (export removal)
- `.memory-bank/sub_projects/airssys-wasm/docs/knowledges/knowledge_wasm_005_inter_component_messaging_architecture.md` (documentation sync)

**Routing Implementation Location:**
- `airssys-rt/src/broker/in_memory.rs`: InMemoryMessageBroker implementation
- `airssys-rt/src/system/actor_system.rs`: ActorSystem component resolution

## History

### Status Changes
- **2025-10-22**: Status set to Accepted - Decision made to remove trait

### Updates
- **2025-10-22**: Initial ADR creation documenting removal decision

### Reviews
- **2025-10-22**: Architectural review confirmed alignment with ADR-WASM-009

---

**Template Version:** 1.0  
**Last Updated:** 2025-10-22
