# Quick Start: RT-TASK-004-REFACTOR

**Task:** MessageBroker Pub-Sub Trait Refactoring  
**Time:** 2-3 hours  
**Priority:** CRITICAL - Do this FIRST  
**Status:** Ready to start  

---

## Before You Start

### Read These (5-10 minutes)
1. **ADR-006**: `.copilot/memory_bank/sub_projects/airssys-rt/docs/adr/adr_006_messagebroker_pubsub_architecture.md`
   - Understand WHY pub-sub architecture
   - Review architecture diagrams

2. **KNOWLEDGE-RT-012**: `.copilot/memory_bank/sub_projects/airssys-rt/docs/knowledges/knowledge_rt_012_pubsub_messagebroker_pattern.md`
   - Complete implementation guide (600+ lines)
   - Code examples and patterns

3. **Task File**: `.copilot/memory_bank/sub_projects/airssys-rt/tasks/task_004_refactor_pubsub_trait.md`
   - Phase-by-phase implementation plan
   - All code snippets ready to use

---

## What You're Building

### Current Trait (Wrong)
```rust
async fn send(&self, envelope: MessageEnvelope<M>) -> Result<(), Self::Error>;
async fn request<R>(...) -> Result<Option<MessageEnvelope<R>>, Self::Error>;
```

### New Trait (Correct)
```rust
async fn publish(&self, envelope: MessageEnvelope<M>) -> Result<(), Self::Error>;
async fn subscribe(&self) -> Result<MessageStream<M>, Self::Error>;
async fn publish_request<R>(...) -> Result<Option<MessageEnvelope<R>>, Self::Error>;
```

---

## Implementation Phases (2-3 hours)

### Phase 1: Add MessageStream Type (30 min)
**File:** `airssys-rt/src/broker/traits.rs`

1. Add import: `use tokio::sync::mpsc;`
2. Add MessageStream struct before MessageBroker trait
3. Update `src/broker/mod.rs`: export MessageStream
4. Add basic test

**Code:** See task file Phase 1 section

---

### Phase 2: Add `publish()` Method (45 min)
**File:** `airssys-rt/src/broker/traits.rs`

1. Add `publish()` method to MessageBroker trait
2. Copy documentation from task file
3. Add compile-time test

**Code:** See task file Phase 2 section

---

### Phase 3: Add `subscribe()` Method (45 min)
**File:** `airssys-rt/src/broker/traits.rs`

1. Add `subscribe()` method to MessageBroker trait
2. Copy documentation from task file (includes use cases)
3. Add compile-time test

**Code:** See task file Phase 3 section

---

### Phase 4: Add `publish_request()` (30 min)
**File:** `airssys-rt/src/broker/traits.rs`

1. Add `publish_request()` method
2. Keep old `request()` method as deprecated
3. Add compile-time test

**Code:** See task file Phase 4 section

---

### Phase 5: Update Documentation (30 min)
**File:** `airssys-rt/src/broker/traits.rs`

1. Update trait-level doc comment with pub-sub architecture
2. Add architecture diagram from task file
3. Update examples

**Code:** See task file Phase 5 section

---

### Phase 6: Update Tests (30 min)
**File:** `airssys-rt/src/broker/traits.rs`

1. Update existing tests
2. Add new compile-time verification tests
3. Run: `cargo test --package airssys-rt`

**Code:** See task file Phase 6 section

---

## Expected Outcome

### ✅ Success Criteria
- MessageBroker trait compiles successfully
- MessageStream type exists and is exported
- All trait tests pass
- Trait documentation updated with pub-sub architecture
- Zero clippy warnings

### ⚠️ Expected Warnings
```
error[E0046]: not all trait items implemented in type `InMemoryMessageBroker`
  --> src/broker/in_memory.rs:XX:1
   |
   | impl<M, S> MessageBroker<M> for InMemoryMessageBroker<M, S>
   | ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^
   | missing: `publish`, `subscribe`, `publish_request`
```

**This is EXPECTED!** RT-TASK-004-PUBSUB will fix it.

---

## Validation Commands

```bash
# Check trait compiles
cargo check --package airssys-rt

# Run trait tests
cargo test --package airssys-rt broker::traits

# Check for clippy warnings
cargo clippy --package airssys-rt --all-targets --all-features

# Expected: Trait tests pass, InMemoryMessageBroker fails to compile
```

---

## If You Get Stuck

### Problem: Not sure what code to write
**Solution:** Copy code from task file Phase sections - it's all ready to use

### Problem: Tests failing
**Solution:** Check import statements (§2.1 - 3-layer organization)

### Problem: Clippy warnings
**Solution:** Follow workspace standards (§2.1, §6.2, §6.3)

### Problem: Confused about architecture
**Solution:** Re-read ADR-006 architecture section

---

## After Completion

### Update Progress
```bash
# Mark task as complete in memory bank
# Update progress.md with completion status
```

### Next Step
**RT-TASK-004-PUBSUB**: InMemoryMessageBroker Pub-Sub Implementation (3-4 hours)

**Quick Start File:** Same location, different task file

---

## Tips

1. **Use Task File**: All code snippets are ready to copy-paste
2. **Follow Phases**: Do them in order, don't skip
3. **Test Often**: Run tests after each phase
4. **Read Docs**: ADR-006 explains the WHY, task file explains the HOW
5. **Stay Focused**: This is just the trait definition, implementation comes next

---

## Time Budget

- Phase 1 (MessageStream): 30 min
- Phase 2 (publish): 45 min
- Phase 3 (subscribe): 45 min  
- Phase 4 (publish_request): 30 min
- Phase 5 (documentation): 30 min
- Phase 6 (tests): 30 min

**Total: 2.5-3 hours**

---

**Ready to start?**

1. Open task file: `tasks/task_004_refactor_pubsub_trait.md`
2. Open code file: `airssys-rt/src/broker/traits.rs`
3. Start with Phase 1: Add MessageStream type
4. Follow the phases in order
5. Test after each phase

**You got this!** 🚀
