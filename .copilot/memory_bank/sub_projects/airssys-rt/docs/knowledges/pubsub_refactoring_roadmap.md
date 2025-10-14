# Pub-Sub Refactoring Roadmap

**Created:** 2025-10-06  
**Status:** Ready to Start  
**Priority:** CRITICAL  
**Estimated Total Time:** 5-7 hours  

---

## Overview

We're refactoring the MessageBroker from direct routing to pub-sub architecture. This is split into TWO sequential tasks for better incremental progress and easier review.

---

## Task Sequence

### ✅ Step 0: Documentation (COMPLETE)
**Time:** 2 hours  
**Files Created:**
- ADR-006: MessageBroker Pub-Sub Architecture
- KNOWLEDGE-RT-012: Pub-Sub MessageBroker Pattern (600+ lines)
- DEBT-RT-005: Updated with pub-sub analysis
- RT-TASK-004-REFACTOR task file
- RT-TASK-004-PUBSUB task file

---

### 🔲 Step 1: RT-TASK-004-REFACTOR (Trait Definition)
**Status:** Not Started  
**Time:** 2-3 hours  
**Priority:** Do this FIRST

**Scope:**
- Update `MessageBroker<M>` trait with pub-sub methods
- Add `MessageStream<M>` type
- Add `publish()`, `subscribe()`, `publish_request()` methods
- Deprecate old `send()` and `request()` methods
- Update trait-level documentation

**Files:**
- `src/broker/traits.rs` (~280 lines, +40 lines)
- `src/broker/mod.rs` (export MessageStream)

**Phases:**
1. Add MessageStream type (30 min)
2. Add publish() method (45 min)
3. Add subscribe() method (45 min)
4. Add publish_request() (30 min)
5. Update documentation (30 min)
6. Update tests (30 min)

**Outcome:**
- ✅ Trait defines pub-sub API
- ⚠️ InMemoryMessageBroker won't compile (expected)
- ✅ Clear interface for implementation

**Task File:** `tasks/rt_task_004_refactor_pubsub_trait.md`

---

### 🔲 Step 2: RT-TASK-004-PUBSUB (Implementation)
**Status:** Not Started (blocked by Step 1)  
**Time:** 3-4 hours  
**Priority:** Do this SECOND

**Scope:**
- Implement pub-sub in `InMemoryMessageBroker`
- Add subscriber management
- Implement broadcast publishing
- Add extensibility hooks
- Handle subscriber cleanup

**Files:**
- `src/broker/in_memory.rs` (~600 lines, +150 lines)

**Phases:**
1. Add subscriber management (45 min)
2. Implement subscribe() (45 min)
3. Implement publish() with broadcast (1 hour)
4. Implement publish_request() (45 min)
5. Update existing methods (30 min)
6. Update documentation (30 min)
7. Comprehensive testing (45 min)

**Outcome:**
- ✅ Full pub-sub implementation
- ✅ All tests passing (~24 broker tests)
- ✅ Ready for ActorSystem integration

**Task File:** `tasks/rt_task_004_pubsub_implementation.md`

---

### 🔲 Step 3: RT-TASK-006 Phase 2 (Resume)
**Status:** Paused (will resume after Step 2)  
**Time:** 4-5 hours  
**Priority:** Do this THIRD

**Scope:**
- Implement ActorSystem with message router
- Subscribe to broker
- Route messages via ActorRegistry
- Implement dead letter queue

**Files:**
- `src/system/actor_system.rs` (rewrite with pub-sub)
- `src/system/builder.rs` (update types)

**Dependency:** Requires RT-TASK-004-PUBSUB complete

---

## Timeline

### Day 1 (Oct 6, 2025)
- ✅ Documentation complete (2 hours) - DONE
- 🔲 RT-TASK-004-REFACTOR start (2-3 hours)

### Day 2 (Oct 7, 2025)
- 🔲 RT-TASK-004-REFACTOR complete
- 🔲 RT-TASK-004-PUBSUB start (3-4 hours)

### Day 3 (Oct 8, 2025)
- 🔲 RT-TASK-004-PUBSUB complete
- 🔲 RT-TASK-006 Phase 2 resume (4-5 hours)

**Total Estimated:** 11-14 hours over 3 days

---

## Success Criteria

### After RT-TASK-004-REFACTOR
- [x] MessageBroker trait has publish/subscribe methods
- [x] MessageStream<M> type exists
- [x] Trait documentation updated with pub-sub architecture
- [x] Tests validate trait requirements
- [ ] InMemoryMessageBroker does NOT compile (expected - waiting for implementation)

### After RT-TASK-004-PUBSUB
- [x] InMemoryMessageBroker compiles successfully
- [x] All broker tests passing (~24 tests)
- [x] Pub-sub broadcast working (multiple subscribers)
- [x] Subscriber cleanup working
- [x] Extensibility hooks in place
- [x] Zero clippy warnings

### After RT-TASK-006 Phase 2
- [x] ActorSystem subscribes to broker
- [x] Message router task routes messages
- [x] Integration with ActorRegistry works
- [x] Dead letter queue handling implemented
- [x] All system tests passing (~50 tests total)

---

## Key Architectural Decisions

### Why Two Tasks?
✅ **Incremental Progress**: Validate API before implementation  
✅ **Easier Review**: Smaller, focused changesets  
✅ **Clear Interface**: Trait as contract, implementation separate  
✅ **Less Risk**: API solidified before implementation complexity  
✅ **Better Testing**: Can test trait independently  

### Why Pub-Sub?
✅ **Extensibility**: Natural hooks for logging, metrics, persistence  
✅ **Multiple Subscribers**: Routing, monitoring, audit independently  
✅ **Distributed Ready**: Redis/NATS without changing actors  
✅ **Observability**: Message flows visible to monitoring  
✅ **Dead Letters**: Undeliverable messages naturally handled  

### Component Responsibilities
| Component | Responsibility | Does NOT Handle |
|-----------|---------------|-----------------|
| **MessageBroker** | Pub-sub transport | Actor routing |
| **ActorRegistry** | Address → Mailbox | Message transport |
| **ActorSystem** | Subscribe & route | Direct delivery |
| **ActorContext** | Publish messages | Address resolution |

---

## Testing Strategy

### RT-TASK-004-REFACTOR Tests
- Trait requirement verification (compile-time)
- MessageStream type tests
- Method signature tests

### RT-TASK-004-PUBSUB Tests
- Single subscriber
- Multiple subscribers (broadcast)
- Subscriber independence
- Disconnected subscriber cleanup
- No subscribers (edge case)
- Late subscriber (doesn't get old messages)
- Request-reply over pub-sub

### Integration Tests (RT-TASK-006)
- ActorSystem router subscribes
- Messages route to correct actors
- Dead letter queue captures undeliverable
- Multiple message types
- Performance benchmarks

---

## Risk Mitigation

### Risk: Breaking Changes
**Mitigation:** Keep old methods deprecated, forward to new methods  
**Impact:** Backward compatible during transition

### Risk: Performance Degradation
**Mitigation:** Benchmark before/after  
**Expected:** Minimal (~1-2μs per message for pub-sub hop)

### Risk: Complex Refactoring
**Mitigation:** Two-task approach, incremental progress  
**Benefit:** Can stop/review between tasks

---

## Documentation Updates Needed

### After RT-TASK-004-REFACTOR
- [ ] Update KNOWLEDGE-RT-009 with pub-sub examples
- [ ] Update KNOWLEDGE-RT-011 with new broker API

### After RT-TASK-004-PUBSUB
- [ ] Update progress.md with completion status
- [ ] Update task status in memory bank
- [ ] Create KNOWLEDGE doc if new patterns emerge

---

## Quick Reference

### Current State
- **MessageBroker trait**: Direct routing (send/request)
- **InMemoryMessageBroker**: Direct delivery via registry
- **Tests passing**: 209 tests (181 foundation + 28 system)

### Target State
- **MessageBroker trait**: Pub-sub (publish/subscribe)
- **InMemoryMessageBroker**: Broadcast to subscribers
- **Tests passing**: ~240 tests (209 + ~15 pub-sub + ~20 system)

### Files Modified
```
src/broker/
├── traits.rs      ← RT-TASK-004-REFACTOR (trait definition)
├── in_memory.rs   ← RT-TASK-004-PUBSUB (implementation)
└── mod.rs         ← RT-TASK-004-REFACTOR (exports)

src/system/
├── actor_system.rs ← RT-TASK-006 Phase 2 (resume later)
└── builder.rs      ← RT-TASK-006 Phase 2 (resume later)
```

---

**Next Action:** Start RT-TASK-004-REFACTOR  
**Command:** Implement Phase 1 (Add MessageStream type)  
**File:** `src/broker/traits.rs`  
**Estimated:** 30 minutes
