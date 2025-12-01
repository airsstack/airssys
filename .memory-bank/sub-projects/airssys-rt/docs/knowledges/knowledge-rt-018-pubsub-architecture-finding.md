# Critical Architecture Finding: Pub-Sub MessageBroker Pattern

**Date:** 2025-10-06  
**Status:** Architecture Refinement - Implementation Pending  
**Priority:** CRITICAL  

---

## Quick Summary

During RT-TASK-006 Phase 2 implementation, we discovered that **MessageBroker must be a true pub-sub message bus** instead of a direct routing system.

### The Problem

❌ **Original Design** (Wrong):
```rust
async fn send(&self, recipient: ActorAddress, message: M) -> Result<()>;
```

- Direct routing semantics
- No extensibility hooks
- Cannot support monitoring/observability
- Blocks distributed broker implementations

✅ **Correct Design** (Pub-Sub):
```rust
async fn publish(&self, envelope: MessageEnvelope<M>) -> Result<()>;
async fn subscribe(&self) -> Result<MessageStream<M>>;
```

- True pub-sub transport layer
- Extensibility hooks for logging/metrics/persistence
- Multiple subscribers (routing, monitoring, audit)
- Enables distributed brokers (Redis, NATS)

---

## Impact

### Components Affected
1. **RT-TASK-004**: MessageBroker trait needs pub-sub operations
2. **RT-TASK-006**: ActorSystem must subscribe and route messages
3. **ActorContext**: Publishes messages (actor API)
4. **ActorRegistry**: Resolves addresses (routing table)

### Architecture Flow

```
Actor → ActorContext.send() → Broker.publish() → [Message Bus]
                                                       ↓
                                              Broker.subscribe()
                                                       ↓
                                           ActorSystem (router)
                                                       ↓
                                           Registry.resolve()
                                                       ↓
                                              Actor Mailbox
```

---

## Documentation Created

1. **ADR-006**: MessageBroker Pub-Sub Architecture Decision
   - File: `.memory-bank/sub_projects/airssys-rt/docs/adr/adr_006_messagebroker_pubsub_architecture.md`
   - Complete architectural decision with rationale

2. **DEBT-RT-005**: Updated with pub-sub architecture analysis
   - File: `.memory-bank/sub_projects/airssys-rt/docs/debts/debt_rt_005_actor_system_broker_integration_mismatch.md`
   - Complete system flow diagrams and implementation plan

3. **KNOWLEDGE-RT-012**: Pub-Sub MessageBroker Pattern Implementation Guide
   - File: `.memory-bank/sub_projects/airssys-rt/docs/knowledges/knowledge_rt_012_pubsub_messagebroker_pattern.md`
   - Complete implementation guide with code examples (600+ lines)

4. **Progress Updates**: Updated progress tracking
   - RT-TASK-004: Marked for pub-sub enhancement
   - RT-TASK-006: Marked as blocked pending pub-sub

---

## Implementation Plan

### Phase 0: Update MessageBroker Trait (NEW - MUST DO FIRST)
**File:** `src/broker/traits.rs`

**Changes:**
- Add `MessageStream<M>` type
- Add `publish()` method (rename from `send()`)
- Add `subscribe()` method
- Add `publish_request()` method
- Update documentation

**Estimated:** 2-3 hours  
**Tests:** Update ~30 broker tests

### Phase 1: Update InMemoryMessageBroker
**File:** `src/broker/in_memory.rs`

**Changes:**
- Add `subscribers: RwLock<Vec<UnboundedSender<MessageEnvelope<M>>>>`
- Implement `publish()` with broadcast
- Implement `subscribe()` with registration
- Add extensibility hooks (logging, metrics)
- Handle subscriber cleanup

**Estimated:** 2-3 hours  
**Tests:** ~15 new pub-sub tests

### Phase 2: Update ActorSystem (Resume RT-TASK-006)
**File:** `src/system/actor_system.rs`

**Changes:**
- Subscribe to broker in `new()`
- Spawn message router background task
- Route via ActorRegistry.resolve()
- Implement dead letter queue handling

**Estimated:** 3-4 hours  
**Tests:** ~20-25 tests

### Phase 3: Update ActorContext (Future)
**File:** `src/actor/context.rs`

**Changes:**
- Use `broker.publish()` instead of direct routing
- Keep simple actor-facing API

**Estimated:** 1-2 hours  
**Tests:** Update existing tests

**Total Time:** 8-12 hours

---

## Benefits

✅ **Clean Architecture**: Separation of transport/routing/orchestration  
✅ **Extensibility**: Natural hooks for logging, metrics, persistence  
✅ **Multiple Subscribers**: System, monitor, audit independently  
✅ **Distributed Ready**: Redis/NATS brokers without changing actors  
✅ **Testability**: Stream-based APIs easier to mock  
✅ **Observability**: Message flows visible to monitoring  
✅ **Future-Proof**: Enables event sourcing, CQRS, etc.

---

## Next Steps

1. **Read KNOWLEDGE-RT-012** for complete implementation guide
2. **Implement Phase 0**: Update MessageBroker trait
3. **Implement Phase 1**: Update InMemoryMessageBroker
4. **Resume RT-TASK-006 Phase 2**: ActorSystem with router
5. **Update ActorContext**: Use broker.publish()

---

## Quick Reference

### Key Files to Read
- **ADR-006**: Architecture decision and rationale
- **KNOWLEDGE-RT-012**: Complete implementation guide with examples
- **DEBT-RT-005**: Problem analysis and system flows

### Type Signatures

```rust
// Trait
pub trait MessageBroker<M: Message>: Send + Sync + Clone {
    async fn publish(&self, envelope: MessageEnvelope<M>) -> Result<()>;
    async fn subscribe(&self) -> Result<MessageStream<M>>;
}

// ActorSystem
pub struct ActorSystem<M, S, B>
where
    M: Message,
    S: MailboxSender<M> + Clone,
    B: MessageBroker<M>,  // ← Generic constraint, not dyn
{
    inner: Arc<ActorSystemInner<M, S, B>>,
}

// ActorContext
pub struct ActorContext<M, B>
where
    M: Message,
    B: MessageBroker<M>,
{
    address: ActorAddress,
    broker: B,  // ← For publishing
}
```

---

**Status:** Documentation complete, implementation pending  
**Blocking:** RT-TASK-006 Phase 2  
**Priority:** CRITICAL - Foundation for actor messaging
