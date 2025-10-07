# airssys-rt Technical Debt Index

**Sub-Project:** airssys-rt  
**Last Updated:** 2025-10-07  
**Total Debt Items:** 3  
**Active Debt Items:** 3  

## Debt Statistics

- **Total Debts**: 3
- **Critical Priority**: 1 (DEBT-RT-006) ⚠️ **BLOCKING RT-TASK-007 Phase 3**
- **High Priority**: 2 (DEBT-RT-004, DEBT-RT-005)
- **Medium Priority**: 0
- **Low Priority**: 0
- **Resolved**: 0

## Pending Debts

### Critical Priority (BLOCKING)

- **DEBT-RT-006**: Phase 3 node.rs Implementation Alignment Fixes ⚠️ **NEW - URGENT**
  - **Component**: Supervisor Framework (RT-TASK-007 Phase 3)
  - **Issue**: node.rs has 39 compilation errors due to API misalignment with Phase 1 and RT-TASK-010
  - **Root Cause**: Implementation based on knowledge doc examples without verifying actual Phase 1 APIs
  - **Impact**: RT-TASK-007 Phase 3 BLOCKED - cannot proceed to tree.rs
  - **Key Mismatches**:
    - Supervisor trait takes no generic parameter (uses associated type)
    - SupervisionEvent is struct, not enum (has event_kind field)
    - RestartBackoff should be per-child HashMap, not global
    - SupervisorError uses String IDs, not ChildId
    - Missing fields: id (Uuid), state (SupervisorState)
  - **Solution**: Systematic API alignment following actual Phase 1 implementations
  - **Target**: TODAY (Oct 7, 2025) - 3-4 hours to fix
  - **File**: `debt_rt_006_phase3_node_implementation_fixes.md`

### High Priority

- **DEBT-RT-005**: Actor System / Broker Integration Architecture Mismatch ⚠️ **URGENT**
  - **Component**: ActorSystem Framework (RT-TASK-006 Phase 2)
  - **Issue**: Knowledge doc assumes `ActorSystem<B: MessageBroker>` but actual `MessageBroker<M>` is message-generic
  - **Root Cause**: KNOWLEDGE-RT-011 written with outdated assumptions about generic parameters
  - **Impact**: RT-TASK-006 Phase 2 BLOCKED - 10+ compilation errors in actor_system.rs
  - **Compilation Errors**:
    - Missing generic parameter for MessageBroker<M>
    - BoundedMailbox::new() API mismatch (takes 1 arg, not 2)
    - register_actor() not on MessageBroker trait (only concrete type)
    - Wrong receiver/sender types from mailbox creation
  - **Solution**: Redesign to use concrete `InMemoryMessageBroker<M, S>` instead of trait
  - **Target**: TODAY (Oct 6, 2025) - must fix before continuing RT-TASK-006
  - **Action Items**:
    1. Update KNOWLEDGE-RT-011 with correct architecture
    2. Rewrite actor_system.rs with concrete broker type
    3. Update builder.rs to match new type parameters
    4. Verify all tests pass (181 + 28 expected)
  - **File**: `debt_rt_005_actor_system_broker_integration_mismatch.md`

### High Priority

- **DEBT-RT-004**: Request-Reply Serialization Performance
  - **Component**: Message Broker (InMemoryMessageBroker)
  - **Issue**: Uses JSON serialization for in-process request-reply (2-10μs overhead)
  - **Solution**: TypedBox pattern for zero-copy type erasure (~100ns overhead)
  - **Impact**: ~100x performance improvement for request-reply pattern
  - **Target**: RT-TASK-010 (Performance Optimization) or Q1 2026
  - **File**: `debt_rt_004_request_reply_serialization.md`

### Medium Priority

- Message serialization strategy selection for cross-boundary communication
- Memory pooling implementations postponed
- Actor scheduling optimizations delayed

### Expected DEBT-QUALITY
- Error handling simplifications in actor boundaries
- Testing coverage gaps during rapid development
- Documentation gaps for complex actor patterns

### Debt Prevention Strategy
- Follow workspace standards (§2.1, §3.2, §4.3, §5.1) from start
- Implement comprehensive testing early
- Regular performance benchmarking
- Continuous integration with debt detection

---
**Note:** Debt tracking will begin when development starts.