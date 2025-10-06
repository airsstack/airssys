# DEBT-RT-005: Actor System / Broker Integration Architecture Mismatch

**Status:** critical  
**Type:** Architecture Mismatch  
**Severity:** High - Blocking RT-TASK-006 Phase 2  
**Created:** 2025-10-06  
**Component:** ActorSystem Framework Integration  
**Affected Files:**
- `airssys-rt/src/system/actor_system.rs` (existing, 536 lines, **compilation errors**)
- `airssys-rt/src/system/builder.rs` (newly created, 402 lines, **blocked by actor_system.rs**)
- KNOWLEDGE-RT-011 (implementation guide with **outdated assumptions**)

---

## Problem Statement

The existing `actor_system.rs` file has **fundamental API mismatches** with the current broker and mailbox implementations, causing compilation failures. The knowledge document KNOWLEDGE-RT-011 contains outdated assumptions about generic type parameters that don't match the actual implemented APIs.

### Root Cause

**Knowledge Document Assumption** (KNOWLEDGE-RT-011, line 480):
```rust
// DOCUMENTED (WRONG):
pub struct ActorSystem<B: MessageBroker> { /* ... */ }
```

**Actual Implementation** (src/broker/traits.rs, line 92):
```rust
// ACTUAL (CORRECT):
pub trait MessageBroker<M: Message>: Send + Sync + Clone {
    type Error: Error + Send + Sync + 'static;
    // ...
}
```

**The Mismatch:**
- Knowledge document assumes `MessageBroker` with NO type parameters
- Actual implementation has `MessageBroker<M: Message>` - generic over message type
- This means ActorSystem CANNOT be generic over just `B: MessageBroker`

---

## Detailed Analysis

### Compilation Errors in actor_system.rs

#### Error 1: Missing Generic Parameter
```rust
// LINE 376 - COMPILATION ERROR
impl<B: MessageBroker> Clone for ActorSystem<B> {
    // ❌ ERROR: missing generics for trait `MessageBroker`
    //           expected 1 generic argument
}
```

**Why:** `MessageBroker` is `MessageBroker<M: Message>`, not `MessageBroker`

#### Error 2: BoundedMailbox API Mismatch
```rust
// LINE 272 - COMPILATION ERROR
let (sender, receiver) =
    BoundedMailbox::<A::Message, _>::new(mailbox_capacity, backpressure);
    // ❌ ERROR: this function takes 1 argument but 2 arguments were supplied
```

**Actual API** (src/mailbox/bounded.rs, line 141):
```rust
pub fn new(capacity: usize) -> (Self, BoundedMailboxSender<M, AtomicMetrics>) {
    // Only takes capacity, NOT backpressure
    // Backpressure is in MailboxSender, not constructor
}
```

#### Error 3: Non-existent Broker Methods
```rust
// LINE 280 - COMPILATION ERROR
self.inner.broker
    .register_actor(address.clone(), sender.clone())
    // ❌ ERROR: no method named `register_actor` found for type parameter `B`
```

**Why:** `MessageBroker<M>` trait does NOT have `register_actor` method
- `register_actor` is on `InMemoryMessageBroker<M, S>` concrete type
- It's not part of the trait interface

**Actual Broker API** (src/broker/traits.rs):
```rust
#[async_trait]
pub trait MessageBroker<M: Message>: Send + Sync + Clone {
    async fn send(&self, envelope: MessageEnvelope<M>) -> Result<(), Self::Error>;
    async fn request<R: Message>(...) -> Result<Option<MessageEnvelope<R>>, Self::Error>;
    // NO register_actor method in trait!
}
```

#### Error 4: Wrong Receiver Type
```rust
// LINE 285 - COMPILATION ERROR
let task_handle = self.spawn_actor_task(actor, receiver, context);
// ❌ ERROR: the trait bound `BoundedMailboxSender<..>: MailboxReceiver<..>` is not satisfied
```

**Why:** `BoundedMailbox::new()` returns `(BoundedMailbox, BoundedMailboxSender)`
- The **receiver** is `BoundedMailbox<M, R>` (first in tuple)
- The **sender** is `BoundedMailboxSender<M, R>` (second in tuple)
- Code tried to use sender as receiver

---

## Historical Context

### What We Implemented (Oct 2-5, 2025)

**RT-TASK-001** through **RT-TASK-005** successfully implemented:
```rust
// ✅ IMPLEMENTED
pub trait Message: Send + Sync + 'static { /* ... */ }
pub struct MessageEnvelope<M: Message> { /* ... */ }
pub trait Actor { type Message: Message; /* ... */ }
pub struct ActorContext<M: Message> { /* ... */ }

// ✅ IMPLEMENTED - Message-generic broker
pub trait MessageBroker<M: Message>: Send + Sync + Clone {
    async fn send(&self, envelope: MessageEnvelope<M>) -> Result<(), Self::Error>;
    async fn request<R: Message>(...) -> Result<...>;
}

// ✅ IMPLEMENTED - Concrete broker with both M and S generics
pub struct InMemoryMessageBroker<M: Message, S: MailboxSender<M>> {
    registry: ActorRegistry<M, S>,
    // ...
}

// ✅ IMPLEMENTED - Mailbox API
pub fn BoundedMailbox::<M, R>::new(capacity: usize) 
    -> (BoundedMailbox<M, R>, BoundedMailboxSender<M, R>)
```

### What the Knowledge Doc Assumed (Oct 6, 2025)

**KNOWLEDGE-RT-011** was written with these assumptions:
```rust
// ❌ WRONG ASSUMPTION
pub struct ActorSystem<B: MessageBroker> { /* ... */ }

// ❌ WRONG ASSUMPTION
impl<B: MessageBroker> ActorSystem<B> {
    pub fn new(config: SystemConfig, broker: B) -> Self { /* ... */ }
}

// ❌ WRONG ASSUMPTION - register_actor doesn't exist on trait
self.broker.register_actor(address, sender).await?;

// ❌ WRONG ASSUMPTION - wrong parameter count
BoundedMailbox::new(capacity, backpressure);
```

---

## Impact Assessment

### Immediate Impact
- ✅ **Phase 1 Complete**: SystemError, SystemConfig (28 tests passing)
- ⚠️ **Phase 2 Blocked**: actor_system.rs has 10+ compilation errors
- ⚠️ **Phase 2 Blocked**: builder.rs depends on working actor_system.rs
- ⚠️ **Tests Failing**: `cargo test` returns exit code 101

### Progress Impact
- **RT-TASK-006**: Stuck at 20% (Phase 1 only)
- **Estimated Fix Time**: 1-2 days for architectural redesign
- **Knowledge Debt**: KNOWLEDGE-RT-011 needs major revision

### Architecture Impact
This reveals a **fundamental design question**:

**The Multi-Message-Type Problem:**
```
ActorSystem needs to manage actors with DIFFERENT message types:
- Actor A handles MessageTypeX
- Actor B handles MessageTypeY  
- Actor C handles MessageTypeZ

But MessageBroker<M> is generic over a SINGLE message type M.

How can ActorSystem<B: MessageBroker<???>> work?
```

---

## Solutions Analysis

### Option 1: Type-Erased Broker (REJECTED - Violates §6.2)
```rust
// ❌ FORBIDDEN - Violates workspace standards §6.2
pub struct ActorSystem {
    broker: Box<dyn Any>, // Type erasure
}
```

**Rejected because:**
- Violates §6.2 "Avoid `dyn` Patterns"
- Violates §6.3 M-DI-HIERARCHY
- Requires runtime type checking
- Performance penalty

### Option 2: Concrete Broker Type (RECOMMENDED)
```rust
// ✅ CORRECT - Use concrete broker implementation
pub struct ActorSystem<M: Message, S: MailboxSender<M>> {
    inner: Arc<ActorSystemInner<M, S>>,
}

struct ActorSystemInner<M: Message, S: MailboxSender<M>> {
    config: SystemConfig,
    broker: InMemoryMessageBroker<M, S>,
    actors: RwLock<HashMap<ActorId, ActorMetadata>>,
    state: RwLock<SystemState>,
}

impl<M: Message, S: MailboxSender<M>> ActorSystem<M, S> {
    pub fn new(
        config: SystemConfig,
        broker: InMemoryMessageBroker<M, S>
    ) -> Self {
        // ...
    }
}
```

**Advantages:**
- ✅ Workspace standards compliant (§6.2, §6.3)
- ✅ Compile-time type safety
- ✅ Zero runtime overhead
- ✅ Clear, explicit types

**Trade-offs:**
- Each ActorSystem instance manages actors with ONE message type
- Need multiple ActorSystem instances for different message types
- This matches Erlang/OTP: one supervisor tree per application

### Option 3: Message Envelope Boxing (HYBRID)
```rust
// 🤔 POSSIBLE - Type-erased messages, typed actors
pub struct ActorSystem {
    broker: InMemoryMessageBroker<BoxedMessage>,
    // ...
}

// Actors send/receive typed messages, system uses envelopes
```

**Analysis:**
- Requires boxing all messages (heap allocation overhead)
- Loses compile-time message type safety
- Complex actor spawning with type conversion
- Not recommended unless multi-message-type is critical requirement

---

## Recommended Solution

### Phase 2 Redesign: Concrete Broker Architecture

**Strategy:** Use concrete `InMemoryMessageBroker<M, S>` instead of trait

```rust
// File: src/system/actor_system.rs

use crate::broker::InMemoryMessageBroker;
use crate::mailbox::{BoundedMailboxSender, MailboxSender};
use crate::message::Message;

/// Main actor system managing actor lifecycle.
///
/// Generic over message type M and mailbox sender S.
/// Each ActorSystem manages actors that communicate via message type M.
///
/// For applications requiring multiple message types, create multiple
/// ActorSystem instances (following Erlang/OTP supervision tree pattern).
pub struct ActorSystem<M, S>
where
    M: Message,
    S: MailboxSender<M> + Clone,
{
    inner: Arc<ActorSystemInner<M, S>>,
}

struct ActorSystemInner<M, S>
where
    M: Message,
    S: MailboxSender<M> + Clone,
{
    config: SystemConfig,
    broker: InMemoryMessageBroker<M, S>,
    actors: RwLock<HashMap<ActorId, ActorMetadata>>,
    state: RwLock<SystemState>,
    shutdown_signal: RwLock<Option<oneshot::Sender<()>>>,
}

impl<M, S> ActorSystem<M, S>
where
    M: Message,
    S: MailboxSender<M> + Clone,
{
    /// Create a new actor system with given configuration and broker.
    pub fn new(
        config: SystemConfig,
        broker: InMemoryMessageBroker<M, S>
    ) -> Self {
        Self {
            inner: Arc::new(ActorSystemInner {
                config,
                broker,
                actors: RwLock::new(HashMap::new()),
                state: RwLock::new(SystemState::Running),
                shutdown_signal: RwLock::new(None),
            }),
        }
    }
    
    /// Spawn a new actor (internal method used by builder)
    pub(crate) async fn spawn_actor_internal<A>(
        &self,
        actor: A,
        name: Option<String>,
        mailbox_capacity: usize,
    ) -> Result<ActorId, SystemError>
    where
        A: Actor<Message = M> + Send + 'static,
    {
        // Check if shutting down
        if self.is_shutting_down() {
            return Err(SystemError::ShuttingDown);
        }

        // Create actor ID and address
        let actor_id = ActorId::new();
        let address = if let Some(ref n) = name {
            ActorAddress::named(n)
        } else {
            ActorAddress::anonymous()
        };

        // Create mailbox - CORRECT API
        let (receiver, sender) = BoundedMailbox::<M, _>::new(mailbox_capacity);

        // Create actor context
        let context = ActorContext::<M>::new(address.clone());

        // Register with broker - DIRECT ACCESS to concrete type
        self.inner.broker
            .register_actor(address.clone(), sender)
            .await
            .map_err(|e| SystemError::SpawnFailed(e.to_string()))?;

        // Spawn actor task
        let task_handle = self.spawn_actor_task(actor, receiver, context);

        // Store metadata
        let metadata = ActorMetadata {
            id: actor_id,
            name,
            spawned_at: Utc::now(),
            task_handle,
        };

        self.inner.actors.write().insert(actor_id, metadata);

        Ok(actor_id)
    }
}
```

---

## Action Items

### Immediate (Today - Oct 6, 2025)
1. ✅ **Document the issue** (this file)
2. ⏳ **Update KNOWLEDGE-RT-011**
   - Remove outdated `ActorSystem<B: MessageBroker>` assumption
   - Add correct `ActorSystem<M, S>` architecture
   - Fix all code examples with correct APIs
   - Add Multi-Message-Type Pattern section
3. ⏳ **Fix actor_system.rs**
   - Rewrite with concrete `InMemoryMessageBroker<M, S>`
   - Fix mailbox creation (only capacity parameter)
   - Use direct broker.register_actor() on concrete type
   - Correct receiver/sender usage
4. ⏳ **Update builder.rs**
   - Update type parameters to match new ActorSystem<M, S>
   - Fix spawn() method signature
5. ⏳ **Run tests**
   - Verify all 181 + 28 tests still pass
   - Zero warnings requirement

### Short Term (This Week)
6. ⏳ **Update progress tracking**
   - Document RT-TASK-006 Phase 2 progress
   - Update completion estimates
7. ⏳ **Create ADR if needed**
   - Document multi-message-type design decision
   - Explain concrete broker choice vs trait generic

### Long Term (Post RT-TASK-006)
8. ⏳ **Knowledge validation**
   - Review all knowledge documents for similar assumptions
   - Ensure all examples compile with actual APIs
9. ⏳ **API documentation**
   - Document multi-ActorSystem pattern for different message types
   - Add examples showing multiple systems coordination

---

## Lessons Learned

### Process Improvements Needed
1. **Knowledge doc validation**: All code examples must be tested against actual implementations
2. **API surface review**: Knowledge docs must be updated when implementation APIs change
3. **Type signature alignment**: Generic constraints must match between docs and code
4. **Regular compilation checks**: Knowledge doc examples should be doc-tested

### Documentation Standards
1. **Version alignment**: Knowledge docs must reference actual git commits/versions
2. **API snapshots**: Include actual API signatures from source files
3. **Compilation validation**: All examples must compile without modification
4. **Update triggers**: Code API changes must trigger knowledge doc review

---

## References

### Related Files
- `src/broker/traits.rs` - Actual MessageBroker<M> trait definition
- `src/broker/in_memory.rs` - InMemoryMessageBroker<M, S> implementation
- `src/mailbox/bounded.rs` - BoundedMailbox::new() actual signature
- `src/system/actor_system.rs` - File with compilation errors
- `src/system/builder.rs` - Newly created, blocked by actor_system.rs

### Knowledge Documents
- KNOWLEDGE-RT-011 - Actor System Framework Implementation Guide (NEEDS UPDATE)
- KNOWLEDGE-RT-009 - Message Broker Architecture (reference for correct API)
- KNOWLEDGE-RT-006 - Mailbox System Implementation Guide (reference for mailbox API)

### Workspace Standards
- §6.2 - Avoid `dyn` Patterns (MANDATORY)
- §6.3 M-DI-HIERARCHY - Concrete types > Generics > dyn traits
- §2.1 - 3-Layer Import Organization
- §4.3 - Module Architecture (mod.rs only declarations)

---

## CRITICAL UPDATE (2025-10-06): True Pub-Sub Architecture Discovered

### **Architectural Breakthrough**

Through deep analysis with the user, we discovered the **correct architecture** that was missing from the original design:

**MessageBroker MUST be a true Pub-Sub message bus, NOT a direct routing system!**

### **The Proper Architecture**

#### **1. MessageBroker as Event Bus (Trait Extension Needed)**

```rust
#[async_trait]
pub trait MessageBroker<M: Message>: Send + Sync + Clone {
    type Error: Error + Send + Sync + 'static;
    
    // PUBLISHING (from actors via ActorContext)
    /// Publish a message to the broker bus
    async fn publish(&self, envelope: MessageEnvelope<M>) -> Result<(), Self::Error>;
    
    /// Publish a request and track correlation for reply
    async fn publish_request<R: Message>(
        &self,
        envelope: MessageEnvelope<M>,
        timeout: Duration,
    ) -> Result<Option<MessageEnvelope<R>>, Self::Error>;
    
    // SUBSCRIBING (ActorSystem listens for all messages)
    /// Subscribe to message events on the broker
    /// Returns a stream of messages that need to be routed
    async fn subscribe(&self) -> Result<MessageStream<M>, Self::Error>;
}

/// Stream of messages from the broker
pub struct MessageStream<M: Message> {
    receiver: mpsc::UnboundedReceiver<MessageEnvelope<M>>,
}
```

#### **2. Complete System Flow**

```
┌─────────────────────────────────────────────────────────────┐
│                      MESSAGE BUS                             │
│                    (MessageBroker)                           │
│                                                              │
│  Publishers                Topics              Subscribers   │
│     │                        │                      │        │
│     ├──────publish──────────>│                      │        │
│     │                        │<─────subscribe───────┤        │
│     │                        │                      │        │
│     │                        │──notify────────────> │        │
│     │                        │  (message arrives)   │        │
└─────────────────────────────────────────────────────────────┘
        ▲                                              ▼
   ActorContext                              ActorSystem Listener
   (publishers)                              (subscriber/router)
                                                      │
                                                      ▼
                                              ActorRegistry.resolve()
                                                      │
                                                      ▲
                                               Deliver to Actor
```

**Key Principles**:
1. **MessageBroker** = Pure pub-sub transport layer (extensibility hooks)
2. **ActorRegistry** = Actor address → mailbox mapping (routing table)
3. **ActorSystem** = Subscriber & router (orchestration)
4. **ActorContext** = Publisher interface (actor API)

#### **3. ActorSystem Integration**

```rust
pub struct ActorSystem<M, S, B>
where
    M: Message,
    S: MailboxSender<M> + Clone,
    B: MessageBroker<M>,
{
    inner: Arc<ActorSystemInner<M, S, B>>,
}

struct ActorSystemInner<M, S, B> {
    config: SystemConfig,
    broker: B,                      // ← Injected (DI pattern)
    registry: ActorRegistry<M, S>,  // ← Created internally
    actors: RwLock<HashMap<ActorId, ActorMetadata>>,
    state: RwLock<SystemState>,
}

impl<M, S, B> ActorSystem<M, S, B> {
    /// Dependency Injection - receives broker
    pub async fn new(config: SystemConfig, broker: B) -> Result<Self, SystemError> {
        // Subscribe to broker's message stream
        let message_stream = broker.subscribe().await?;
        
        let system = Self {
            inner: Arc::new(ActorSystemInner {
                config,
                broker,
                registry: ActorRegistry::new(),
                actors: RwLock::new(HashMap::new()),
                state: RwLock::new(SystemState::Running),
            }),
        };
        
        // Spawn background task to route messages
        system.spawn_message_router(message_stream);
        
        Ok(system)
    }
    
    /// Background task that routes messages from broker to actors
    fn spawn_message_router(&self, mut stream: MessageStream<M>) {
        let inner = Arc::clone(&self.inner);
        
        tokio::spawn(async move {
            while let Some(envelope) = stream.next().await {
                let recipient = envelope.reply_to.clone()
                    .unwrap_or_else(ActorAddress::anonymous);
                
                // Resolve actor via registry
                match inner.registry.resolve(&recipient) {
                    Ok(sender) => {
                        // Forward to actor's mailbox
                        if let Err(e) = sender.send(envelope).await {
                            log::error!("Failed to deliver: {:?}", e);
                            // TODO: Dead letter queue
                        }
                    }
                    Err(e) => {
                        log::warn!("Actor not found: {:?}", e);
                        // TODO: Dead letter queue
                    }
                }
            }
        });
    }
    
    async fn spawn_actor_internal<A>(...) -> Result<ActorId, SystemError> {
        // Register with registry (NOT broker)
        self.inner.registry.register(address.clone(), sender)?;
        
        // Create context with broker reference (for publishing)
        let context = ActorContext::new(address.clone(), self.inner.broker.clone());
        
        // ...
    }
}
```

#### **4. ActorContext as Publisher**

```rust
pub struct ActorContext<M, B>
where
    M: Message,
    B: MessageBroker<M>,
{
    address: ActorAddress,
    broker: B,  // ← For publishing messages
    // metadata...
    _marker: PhantomData<M>,
}

impl<M, B> ActorContext<M, B>
where
    M: Message,
    B: MessageBroker<M>,
{
    pub async fn send(&self, recipient: ActorAddress, message: M) -> Result<(), B::Error> {
        let envelope = MessageEnvelope::new(message)
            .with_sender(self.address.clone())
            .with_recipient(recipient);
        
        // Publish to broker (pub-sub pattern)
        self.broker.publish(envelope).await
    }
}
```

### **Benefits of Pub-Sub Architecture**

✅ **Extensibility in MessageBroker**
```rust
async fn publish(&self, envelope: MessageEnvelope<M>) -> Result<(), Self::Error> {
    // Hook 1: Logging
    self.logger.log_message(&envelope);
    
    // Hook 2: Metrics  
    self.metrics.record_published();
    
    // Hook 3: Persistence
    self.storage.persist(&envelope).await?;
    
    // Hook 4: Circuit breakers
    self.circuit_breaker.check(&envelope.reply_to)?;
    
    // Then broadcast to all subscribers
    self.broadcast_to_subscribers(envelope).await
}
```

✅ **Multiple Subscribers**
```rust
// ActorSystem subscribes for routing
let routing_stream = broker.subscribe().await?;

// Monitoring service subscribes for observability
let monitor_stream = broker.subscribe().await?;

// Audit logger subscribes for compliance
let audit_stream = broker.subscribe().await?;
```

✅ **Dead Letter Queue Support**
```rust
match registry.resolve(&recipient) {
    Ok(sender) => { /* deliver */ }
    Err(_) => {
        dead_letter_queue.push(envelope).await;
    }
}
```

✅ **Future: Distributed Brokers**
```rust
// Redis pub-sub
impl<M: Message> MessageBroker<M> for RedisMessageBroker<M> {
    async fn publish(&self, envelope: MessageEnvelope<M>) -> Result<()> {
        let serialized = serde_json::to_vec(&envelope)?;
        self.redis.publish("actor-messages", serialized).await
    }
    
    async fn subscribe(&self) -> Result<MessageStream<M>> {
        let stream = self.redis.subscribe("actor-messages").await?;
        Ok(MessageStream::from_redis(stream))
    }
}
```

### **Required Changes**

#### **Phase 0: Update MessageBroker Trait** (NEW - CRITICAL)
- Add `publish()` method (rename from `send()`)
- Add `subscribe()` method (returns MessageStream)
- Add `publish_request()` method (rename from `request()`)
- Update InMemoryMessageBroker to implement new trait
- Update all existing tests

#### **Phase 1: Update InMemoryMessageBroker**
- Implement pub-sub channels (mpsc for subscribers)
- Implement subscriber registration
- Implement broadcast to all subscribers
- Add extensibility hooks in publish()

#### **Phase 2: Update ActorSystem**
- Add message router background task
- Subscribe to broker on initialization
- Route messages via ActorRegistry
- Implement dead letter queue handling

#### **Phase 3: Update ActorContext**
- Change to use `broker.publish()` instead of direct routing
- Keep simple actor-facing API

---

## Resolution Strategy

### **Step 0: Trait Extension (MUST DO FIRST)**
1. Extend MessageBroker trait with publish/subscribe
2. Update InMemoryMessageBroker implementation
3. Add MessageStream type
4. Update all broker tests
5. Estimated: 2-3 hours

### **Step 1-8: Original plan still valid**
- But with corrected architecture using pub-sub pattern
- Estimated: Additional 1-2 hours for pub-sub integration

### **Total Additional Time: 3-5 hours**

---

**Status**: Architecture breakthrough - True pub-sub pattern identified  
**Priority**: CRITICAL - Must implement before Phase 2  
**Impact**: Significant - Changes broker trait and all integration points  
**Benefit**: Massive - Proper extensibility, monitoring, and future distributed support

**Next Action:** 
1. Create ADR documenting pub-sub architecture decision
2. Update MessageBroker trait (RT-TASK-004 modification)
3. Then proceed with RT-TASK-006 Phase 2 using correct architecture
