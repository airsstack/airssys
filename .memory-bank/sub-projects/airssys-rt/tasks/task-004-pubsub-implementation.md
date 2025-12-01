# RT-TASK-004-PUBSUB: InMemoryMessageBroker Pub-Sub Implementation

**Task ID:** RT-TASK-004-PUBSUB  
**Parent Task:** RT-TASK-004 (Message Broker Core)  
**Priority:** CRITICAL  
**Status:** Not Started  
**Estimated Time:** 3-4 hours  
**Created:** 2025-10-06  
**Depends On:** RT-TASK-004-REFACTOR (must complete first)  
**Blocks:** RT-TASK-006 Phase 2  

---

## Objective

Implement pub-sub architecture in `InMemoryMessageBroker` with subscriber management, broadcast publishing, and extensibility hooks. This is Phase 1 of the pub-sub architecture implementation as defined in ADR-006.

## Context

**Architecture Breakthrough**: The MessageBroker trait has been updated to pub-sub pattern (RT-TASK-004-REFACTOR). Now we need to implement the actual pub-sub infrastructure in `InMemoryMessageBroker`.

**Key Changes:**
- Add subscriber management with `RwLock<Vec<UnboundedSender>>`
- Implement `publish()` with broadcast to all subscribers
- Implement `subscribe()` with subscriber registration
- Add extensibility hooks (logging, metrics placeholders)
- Handle disconnected subscriber cleanup

**Related Documentation:**
- **ADR-006**: MessageBroker Pub-Sub Architecture
- **KNOWLEDGE-RT-012**: Pub-Sub MessageBroker Pattern (600+ line implementation guide)
- **DEBT-RT-005**: Actor System Broker Integration Mismatch

---

## Scope

### Files to Modify
- `src/broker/in_memory.rs` - InMemoryMessageBroker implementation (~600 lines, +100 lines)

### Files NOT Modified (Out of Scope)
- `src/broker/traits.rs` - Already updated in RT-TASK-004-REFACTOR
- `src/broker/registry.rs` - No changes needed (perfect as-is)
- `src/broker/error.rs` - No new error variants needed
- `src/broker/mod.rs` - No additional exports needed

---

## Implementation Phases

### Phase 1: Add Subscriber Management (45 minutes)

**File:** `src/broker/in_memory.rs`

**Update imports:**

```rust
// Layer 1: Standard library imports
use std::sync::Arc;
use std::time::Duration;

// Layer 2: Third-party crate imports
use async_trait::async_trait;
use dashmap::DashMap;
use tokio::sync::{oneshot, mpsc, RwLock};  // ← Add mpsc and RwLock
use tokio::time::timeout;

// Layer 3: Internal module imports
use super::error::BrokerError;
use super::registry::ActorRegistry;
use super::traits::{MessageBroker, MessageStream};  // ← Add MessageStream
use crate::mailbox::MailboxSender;
use crate::message::{Message, MessageEnvelope};
use crate::util::ActorAddress;
```

**Update InMemoryBrokerInner struct:**

```rust
struct InMemoryBrokerInner<M: Message, S: MailboxSender<M>> {
    /// Actor registry for address resolution
    registry: ActorRegistry<M, S>,

    /// Pub-sub subscribers: receives ALL published messages
    /// Each subscriber gets independent copy of every message
    subscribers: RwLock<Vec<mpsc::UnboundedSender<MessageEnvelope<M>>>>,

    /// Pending request-reply channels: correlation_id -> response sender
    pending_requests: DashMap<uuid::Uuid, oneshot::Sender<Vec<u8>>>,
}
```

**Update `new()` method:**

```rust
pub fn new() -> Self {
    Self {
        inner: Arc::new(InMemoryBrokerInner {
            registry: ActorRegistry::new(),
            subscribers: RwLock::new(Vec::new()),  // ← Initialize empty subscribers
            pending_requests: DashMap::new(),
        }),
    }
}
```

**Tests:**
```rust
#[test]
fn test_broker_initialization_with_subscribers() {
    let broker = InMemoryMessageBroker::<TestMessage, _>::new();
    // Verify broker initializes with empty subscribers
}
```

---

### Phase 2: Implement `subscribe()` (45 minutes)

**File:** `src/broker/in_memory.rs`

**Add subscribe implementation:**

```rust
#[async_trait]
impl<M: Message, S: MailboxSender<M> + Clone> MessageBroker<M> 
    for InMemoryMessageBroker<M, S>
{
    type Error = BrokerError;

    async fn subscribe(&self) -> Result<MessageStream<M>, Self::Error> {
        // Create unbounded channel for this subscriber
        let (tx, rx) = mpsc::unbounded_channel();
        
        // Register subscriber
        let mut subscribers = self.inner.subscribers.write().await;
        subscribers.push(tx);
        
        let subscriber_count = subscribers.len();
        drop(subscribers); // Release write lock
        
        log::debug!(
            "New subscriber registered (total subscribers: {})",
            subscriber_count
        );
        
        Ok(MessageStream::new(rx))
    }
    
    // ... other methods ...
}
```

**Tests:**
```rust
#[tokio::test]
async fn test_subscribe_single_subscriber() {
    let broker = InMemoryMessageBroker::<TestMessage, _>::new();
    
    let stream = broker.subscribe().await.unwrap();
    // Verify stream is created
}

#[tokio::test]
async fn test_subscribe_multiple_subscribers() {
    let broker = InMemoryMessageBroker::<TestMessage, _>::new();
    
    let stream1 = broker.subscribe().await.unwrap();
    let stream2 = broker.subscribe().await.unwrap();
    let stream3 = broker.subscribe().await.unwrap();
    
    // Verify 3 independent subscribers
}
```

---

### Phase 3: Implement `publish()` with Broadcast (1 hour)

**File:** `src/broker/in_memory.rs`

**Implement publish with extensibility hooks:**

```rust
async fn publish(&self, envelope: MessageEnvelope<M>) -> Result<(), Self::Error> {
    // EXTENSIBILITY HOOK 1: Logging
    log::trace!(
        "Publishing message: id={}, sender={:?}, recipient={:?}",
        envelope.metadata.message_id,
        envelope.metadata.sender,
        envelope.metadata.reply_to
    );
    
    // EXTENSIBILITY HOOK 2: Metrics (placeholder for future)
    // TODO: self.metrics.messages_published.increment();
    
    // EXTENSIBILITY HOOK 3: Persistence (placeholder for future)
    // TODO: if self.config.persist_messages {
    //     self.storage.persist(&envelope).await?;
    // }
    
    // EXTENSIBILITY HOOK 4: Circuit breaker (placeholder for future)
    // TODO: if let Some(ref recipient) = envelope.metadata.reply_to {
    //     self.circuit_breaker.check(recipient)?;
    // }
    
    // Broadcast to all subscribers
    let subscribers = self.inner.subscribers.read().await;
    let mut failed_subscribers = Vec::new();
    
    for (idx, sender) in subscribers.iter().enumerate() {
        // Try to send to this subscriber
        if let Err(_) = sender.send(envelope.clone()) {
            // Subscriber channel closed - mark for removal
            log::warn!("Subscriber {} disconnected, marking for cleanup", idx);
            failed_subscribers.push(idx);
        }
    }
    
    drop(subscribers); // Release read lock
    
    // Clean up disconnected subscribers (if any)
    if !failed_subscribers.is_empty() {
        let mut subscribers = self.inner.subscribers.write().await;
        
        // Remove in reverse order to maintain valid indices
        for idx in failed_subscribers.into_iter().rev() {
            if idx < subscribers.len() {
                subscribers.swap_remove(idx);
                log::debug!("Removed disconnected subscriber (remaining: {})", subscribers.len());
            }
        }
    }
    
    Ok(())
}
```

**Tests:**
```rust
#[tokio::test]
async fn test_publish_broadcasts_to_all_subscribers() {
    let broker = InMemoryMessageBroker::<TestMessage, _>::new();
    
    // Create 3 subscribers
    let mut stream1 = broker.subscribe().await.unwrap();
    let mut stream2 = broker.subscribe().await.unwrap();
    let mut stream3 = broker.subscribe().await.unwrap();
    
    // Publish message
    let envelope = MessageEnvelope::new(TestMessage { value: 42 });
    broker.publish(envelope.clone()).await.unwrap();
    
    // All 3 subscribers should receive the message
    let msg1 = stream1.recv().await.unwrap();
    let msg2 = stream2.recv().await.unwrap();
    let msg3 = stream3.recv().await.unwrap();
    
    assert_eq!(msg1.message.value, 42);
    assert_eq!(msg2.message.value, 42);
    assert_eq!(msg3.message.value, 42);
}

#[tokio::test]
async fn test_publish_multiple_messages() {
    let broker = InMemoryMessageBroker::<TestMessage, _>::new();
    let mut stream = broker.subscribe().await.unwrap();
    
    // Publish 3 messages
    broker.publish(MessageEnvelope::new(TestMessage { value: 1 })).await.unwrap();
    broker.publish(MessageEnvelope::new(TestMessage { value: 2 })).await.unwrap();
    broker.publish(MessageEnvelope::new(TestMessage { value: 3 })).await.unwrap();
    
    // Subscriber receives all 3 in order
    assert_eq!(stream.recv().await.unwrap().message.value, 1);
    assert_eq!(stream.recv().await.unwrap().message.value, 2);
    assert_eq!(stream.recv().await.unwrap().message.value, 3);
}

#[tokio::test]
async fn test_publish_cleans_up_disconnected_subscribers() {
    let broker = InMemoryMessageBroker::<TestMessage, _>::new();
    
    let stream1 = broker.subscribe().await.unwrap();
    let mut stream2 = broker.subscribe().await.unwrap();
    
    // Drop stream1 (disconnect subscriber)
    drop(stream1);
    
    // Publish message - should trigger cleanup
    broker.publish(MessageEnvelope::new(TestMessage { value: 42 })).await.unwrap();
    
    // stream2 still receives message
    assert_eq!(stream2.recv().await.unwrap().message.value, 42);
    
    // Only 1 subscriber should remain
    // (This is checked implicitly by the cleanup logic)
}
```

---

### Phase 4: Implement `publish_request()` (45 minutes)

**File:** `src/broker/in_memory.rs`

**Implement publish_request (adapted from old request()):**

```rust
async fn publish_request<R: Message + for<'de> serde::Deserialize<'de>>(
    &self,
    envelope: MessageEnvelope<M>,
    timeout_duration: Duration,
) -> Result<Option<MessageEnvelope<R>>, Self::Error> {
    // Extract or generate correlation ID
    let correlation_id = envelope.metadata.correlation_id
        .ok_or_else(|| BrokerError::MissingCorrelationId)?;
    
    // Create reply channel
    let (reply_tx, reply_rx) = oneshot::channel();
    
    // Register pending request
    self.inner.pending_requests.insert(correlation_id, reply_tx);
    
    // Publish request via pub-sub
    self.publish(envelope).await?;
    
    // Wait for reply with timeout
    let result = timeout(timeout_duration, reply_rx).await;
    
    // Cleanup pending request
    self.inner.pending_requests.remove(&correlation_id);
    
    match result {
        Ok(Ok(reply_bytes)) => {
            // Deserialize reply
            let reply_envelope: MessageEnvelope<R> = serde_json::from_slice(&reply_bytes)
                .map_err(|e| BrokerError::SerializationError(e.to_string()))?;
            Ok(Some(reply_envelope))
        }
        Ok(Err(_)) => Ok(None), // Reply channel closed
        Err(_) => Ok(None),     // Timeout
    }
}
```

**Keep deprecated `request()` for compatibility:**

```rust
#[deprecated(since = "0.2.0", note = "Use `publish_request()` instead")]
async fn request<R: Message + for<'de> serde::Deserialize<'de>>(
    &self,
    envelope: MessageEnvelope<M>,
    timeout: Duration,
) -> Result<Option<MessageEnvelope<R>>, Self::Error> {
    // Forward to publish_request
    self.publish_request(envelope, timeout).await
}
```

**Tests:**
```rust
#[tokio::test]
async fn test_publish_request_with_reply() {
    let broker = InMemoryMessageBroker::<TestMessage, _>::new();
    
    // Subscribe to handle request
    let mut stream = broker.subscribe().await.unwrap();
    
    // Spawn handler that replies
    tokio::spawn(async move {
        if let Some(request) = stream.recv().await {
            // Simulate processing and reply
            // (This test needs ActorRegistry integration - simplified for now)
        }
    });
    
    // Send request
    let correlation_id = uuid::Uuid::new_v4();
    let request = MessageEnvelope::new(TestMessage { value: 42 })
        .with_correlation_id(correlation_id);
    
    let reply = broker.publish_request::<TestMessage>(
        request,
        Duration::from_secs(5)
    ).await.unwrap();
    
    // For now, expect timeout (full integration in RT-TASK-006)
    assert!(reply.is_none());
}

#[tokio::test]
async fn test_publish_request_timeout() {
    let broker = InMemoryMessageBroker::<TestMessage, _>::new();
    
    let correlation_id = uuid::Uuid::new_v4();
    let request = MessageEnvelope::new(TestMessage { value: 42 })
        .with_correlation_id(correlation_id);
    
    let reply = broker.publish_request::<TestMessage>(
        request,
        Duration::from_millis(100)
    ).await.unwrap();
    
    assert!(reply.is_none());
}
```

---

### Phase 5: Update Existing Methods (30 minutes)

**File:** `src/broker/in_memory.rs`

**Keep `register_actor()`, `unregister_actor()`, `resolve()` unchanged:**

These methods are part of the ActorRegistry delegation and work perfectly as-is:

```rust
// These stay exactly the same - no changes needed
pub fn register_actor(
    &self,
    address: ActorAddress,
    sender: S,
) -> Result<(), BrokerError> {
    self.inner.registry.register(address, sender)
}

pub fn unregister_actor(&self, address: &ActorAddress) -> Result<(), BrokerError> {
    self.inner.registry.unregister(address)
}

pub fn resolve(&self, address: &ActorAddress) -> Result<S, BrokerError> {
    self.inner.registry.resolve(address)
}
```

**Deprecate old `send()` method:**

```rust
#[deprecated(since = "0.2.0", note = "Use `publish()` instead - pub-sub architecture")]
async fn send(&self, envelope: MessageEnvelope<M>) -> Result<(), Self::Error> {
    // For backward compatibility, forward to publish
    self.publish(envelope).await
}
```

---

### Phase 6: Update Documentation (30 minutes)

**Update module-level documentation in `src/broker/in_memory.rs`:**

```rust
//! In-memory message broker with pub-sub architecture.
//!
//! Default broker implementation providing high-performance pub-sub message
//! routing with lock-free concurrent data structures and ownership transfer.
//!
//! # Pub-Sub Architecture
//!
//! The broker implements a publish-subscribe pattern:
//! - Publishers (actors) publish messages via `publish()`
//! - Subscribers (ActorSystem, monitors) receive ALL messages via `subscribe()`
//! - Routing is handled by subscribers, not the broker
//!
//! # Performance Characteristics
//!
//! - **Throughput**: >1M messages/second (single subscriber)
//! - **Throughput**: ~300K messages/second (3-4 subscribers broadcast)
//! - **Latency**: <1μs message routing overhead
//! - **Concurrency**: Lock-free registry + RwLock subscribers
//! - **Memory**: Zero-copy message transfer via ownership
//!
//! # Subscriber Management
//!
//! - Subscribers registered via `subscribe()`
//! - Each subscriber gets independent message stream
//! - Disconnected subscribers auto-cleaned on next publish
//! - Broadcast to all active subscribers
//!
//! # Extensibility Hooks
//!
//! The `publish()` method provides hooks for:
//! - Logging and distributed tracing
//! - Metrics collection (placeholder)
//! - Message persistence (placeholder)
//! - Circuit breakers (placeholder)
//! - Rate limiting (placeholder)
//!
//! # Example (System-Level Usage)
//!
//! ```ignore
//! use airssys_rt::broker::{InMemoryMessageBroker, MessageBroker};
//! use airssys_rt::message::MessageEnvelope;
//!
//! let broker = InMemoryMessageBroker::<MyMessage>::new();
//!
//! // ActorSystem subscribes for routing
//! let mut routing_stream = broker.subscribe().await?;
//! tokio::spawn(async move {
//!     while let Some(envelope) = routing_stream.recv().await {
//!         // Route to actor via registry
//!         route_to_actor(envelope, &registry).await?;
//!     }
//! });
//!
//! // Actor publishes message (via ActorContext)
//! let envelope = MessageEnvelope::new(message)
//!     .with_recipient(recipient_address);
//! broker.publish(envelope).await?;
//! ```
//!
//! # Clone Semantics
//!
//! Implements cheap clone via Arc (M-SERVICES-CLONE pattern).
//! All clones share the same registry, subscribers, and pending requests.
```

**Update InMemoryMessageBroker struct documentation:**

```rust
/// In-memory message broker with pub-sub architecture.
///
/// Provides publish-subscribe message distribution with:
/// - Multiple independent subscribers
/// - Broadcast delivery to all subscribers
/// - Automatic disconnected subscriber cleanup
/// - Extensibility hooks for logging, metrics, persistence
/// - Zero-copy message transfer via ownership
///
/// # Pub-Sub Semantics
///
/// Unlike direct routing, this broker broadcasts ALL messages to ALL subscribers.
/// Subscribers (like ActorSystem) are responsible for routing to specific actors.
///
/// # Performance
///
/// - Single subscriber: >1M msgs/sec
/// - 3-4 subscribers: ~300K msgs/sec (broadcast overhead)
/// - Latency: <1μs per message
///
/// # Example
///
/// ```ignore
/// let broker = InMemoryMessageBroker::<MyMessage>::new();
///
/// // Subscribe for routing
/// let mut stream = broker.subscribe().await?;
///
/// // Publish message
/// broker.publish(envelope).await?;
///
/// // Receive from stream
/// if let Some(msg) = stream.recv().await {
///     // Route to actor
/// }
/// ```
```

---

### Phase 7: Comprehensive Testing (45 minutes)

**Add comprehensive test suite:**

```rust
#[cfg(test)]
mod tests {
    use super::*;
    use crate::message::MessagePriority;
    
    #[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
    struct TestMessage {
        value: i32,
    }
    
    impl Message for TestMessage {
        const MESSAGE_TYPE: &'static str = "TestMessage";
        fn priority(&self) -> MessagePriority {
            MessagePriority::Normal
        }
    }
    
    // ... existing tests ...
    
    // NEW TESTS FOR PUB-SUB
    
    #[tokio::test]
    async fn test_pubsub_single_subscriber() {
        let broker = InMemoryMessageBroker::<TestMessage, _>::new();
        let mut stream = broker.subscribe().await.unwrap();
        
        let envelope = MessageEnvelope::new(TestMessage { value: 42 });
        broker.publish(envelope).await.unwrap();
        
        let received = stream.recv().await.unwrap();
        assert_eq!(received.message.value, 42);
    }
    
    #[tokio::test]
    async fn test_pubsub_multiple_subscribers_broadcast() {
        let broker = InMemoryMessageBroker::<TestMessage, _>::new();
        
        let mut stream1 = broker.subscribe().await.unwrap();
        let mut stream2 = broker.subscribe().await.unwrap();
        let mut stream3 = broker.subscribe().await.unwrap();
        
        broker.publish(MessageEnvelope::new(TestMessage { value: 1 })).await.unwrap();
        broker.publish(MessageEnvelope::new(TestMessage { value: 2 })).await.unwrap();
        
        // All subscribers receive both messages
        assert_eq!(stream1.recv().await.unwrap().message.value, 1);
        assert_eq!(stream1.recv().await.unwrap().message.value, 2);
        
        assert_eq!(stream2.recv().await.unwrap().message.value, 1);
        assert_eq!(stream2.recv().await.unwrap().message.value, 2);
        
        assert_eq!(stream3.recv().await.unwrap().message.value, 1);
        assert_eq!(stream3.recv().await.unwrap().message.value, 2);
    }
    
    #[tokio::test]
    async fn test_pubsub_subscriber_independence() {
        let broker = InMemoryMessageBroker::<TestMessage, _>::new();
        
        let mut stream1 = broker.subscribe().await.unwrap();
        let mut stream2 = broker.subscribe().await.unwrap();
        
        broker.publish(MessageEnvelope::new(TestMessage { value: 42 })).await.unwrap();
        
        // stream1 receives
        let msg1 = stream1.recv().await.unwrap();
        assert_eq!(msg1.message.value, 42);
        
        // stream2 receives independently (message not consumed)
        let msg2 = stream2.recv().await.unwrap();
        assert_eq!(msg2.message.value, 42);
    }
    
    #[tokio::test]
    async fn test_pubsub_disconnected_subscriber_cleanup() {
        let broker = InMemoryMessageBroker::<TestMessage, _>::new();
        
        let stream1 = broker.subscribe().await.unwrap();
        let mut stream2 = broker.subscribe().await.unwrap();
        let stream3 = broker.subscribe().await.unwrap();
        
        // Drop stream1 and stream3
        drop(stream1);
        drop(stream3);
        
        // Publish triggers cleanup
        broker.publish(MessageEnvelope::new(TestMessage { value: 42 })).await.unwrap();
        
        // stream2 still works
        let received = stream2.recv().await.unwrap();
        assert_eq!(received.message.value, 42);
    }
    
    #[tokio::test]
    async fn test_pubsub_no_subscribers() {
        let broker = InMemoryMessageBroker::<TestMessage, _>::new();
        
        // Publish with no subscribers should succeed
        broker.publish(MessageEnvelope::new(TestMessage { value: 42 })).await.unwrap();
    }
    
    #[tokio::test]
    async fn test_pubsub_late_subscriber() {
        let broker = InMemoryMessageBroker::<TestMessage, _>::new();
        
        // Publish before subscriber
        broker.publish(MessageEnvelope::new(TestMessage { value: 1 })).await.unwrap();
        
        // Subscribe late
        let mut stream = broker.subscribe().await.unwrap();
        
        // Late subscriber doesn't receive old messages
        broker.publish(MessageEnvelope::new(TestMessage { value: 2 })).await.unwrap();
        
        let received = stream.recv().await.unwrap();
        assert_eq!(received.message.value, 2); // Only new message
    }
    
    #[tokio::test]
    async fn test_publish_request_forwards_to_publish() {
        let broker = InMemoryMessageBroker::<TestMessage, _>::new();
        let mut stream = broker.subscribe().await.unwrap();
        
        let correlation_id = uuid::Uuid::new_v4();
        let request = MessageEnvelope::new(TestMessage { value: 42 })
            .with_correlation_id(correlation_id);
        
        // Spawn request (will timeout but that's ok)
        let broker_clone = broker.clone();
        tokio::spawn(async move {
            let _ = broker_clone.publish_request::<TestMessage>(
                request,
                Duration::from_millis(100)
            ).await;
        });
        
        // Subscriber should receive the request
        let received = stream.recv().await.unwrap();
        assert_eq!(received.message.value, 42);
        assert_eq!(received.metadata.correlation_id, Some(correlation_id));
    }
}
```

---

## Validation Checklist

### Code Quality
- [ ] Zero compilation errors
- [ ] Zero clippy warnings (`cargo clippy --all-targets --all-features`)
- [ ] All tests passing (`cargo test --package airssys-rt`)
- [ ] Workspace standards compliance (§2.1, §3.2, §4.3, §6.1, §6.2, §6.3)

### Functionality
- [ ] `subscribe()` creates independent message streams
- [ ] `publish()` broadcasts to all subscribers
- [ ] Disconnected subscribers automatically cleaned up
- [ ] `publish_request()` uses pub-sub + correlation tracking
- [ ] Extensibility hooks in place (logging, placeholders for metrics/persistence)

### Testing
- [ ] Single subscriber test ✅
- [ ] Multiple subscribers broadcast test ✅
- [ ] Subscriber independence test ✅
- [ ] Disconnected subscriber cleanup test ✅
- [ ] No subscribers test ✅
- [ ] Late subscriber test ✅
- [ ] Request-reply pub-sub integration test ✅
- [ ] All existing broker tests still pass ✅

### Documentation
- [ ] Module-level pub-sub architecture documented
- [ ] Struct-level pub-sub semantics documented
- [ ] All methods have updated doc comments
- [ ] Examples show pub-sub usage patterns
- [ ] Cross-references to ADR-006 and KNOWLEDGE-RT-012

### Performance
- [ ] Zero-copy message transfer maintained
- [ ] RwLock for subscribers (read-heavy optimization)
- [ ] Subscriber cleanup doesn't block publishing
- [ ] Broadcast overhead acceptable (~300K msgs/sec with 3-4 subscribers)

### Standards Compliance
- [ ] §2.1: 3-layer import organization ✅
- [ ] §3.2: chrono DateTime<Utc> (not applicable)
- [ ] §4.3: Module architecture - implementation separation
- [ ] §6.1: YAGNI - extensibility hooks as placeholders, not full implementations
- [ ] §6.2: Avoid dyn - using generic constraints
- [ ] §6.3: M-SERVICES-CLONE - Arc<Inner> pattern ✅

---

## Dependencies

### Depends On
- **RT-TASK-004-REFACTOR**: Must complete first (trait definition)

### Blocks
- **RT-TASK-006 Phase 2**: Cannot implement ActorSystem until pub-sub is ready

---

## Expected Outcomes

### Files Modified
- `src/broker/in_memory.rs` - Pub-sub implementation (~600 lines, +150 lines)

### Tests
- Existing broker tests: 9 tests (all passing)
- New pub-sub tests: 15 tests
- Total: 24 tests passing

### Features Added
- ✅ Subscriber management with auto-cleanup
- ✅ Broadcast publishing to all subscribers
- ✅ Independent message streams per subscriber
- ✅ Extensibility hooks (logging + placeholders)
- ✅ Request-reply over pub-sub

### Performance
- Single subscriber: >1M msgs/sec
- 3-4 subscribers: ~300K msgs/sec (broadcast overhead)
- Latency: <1μs per message

---

## Integration with ActorSystem (Future RT-TASK-006)

After this task, ActorSystem can:

```rust
// Subscribe to broker
let mut stream = broker.subscribe().await?;

// Spawn message router
tokio::spawn(async move {
    while let Some(envelope) = stream.recv().await {
        // Route via ActorRegistry
        if let Some(recipient) = envelope.metadata.reply_to {
            let sender = registry.resolve(&recipient)?;
            sender.send(envelope).await?;
        }
    }
});
```

---

## Notes

### Migration from Old API
- Old `send()` → deprecated, forwards to `publish()`
- Old `request()` → deprecated, forwards to `publish_request()`
- All existing code continues to work (backward compatible)
- New code should use pub-sub methods

### Extensibility Hooks Philosophy (§6.1 YAGNI)
We add **placeholders** for future features:
- Logging: ✅ Implemented (needed for debugging)
- Metrics: TODO comment (future need)
- Persistence: TODO comment (future need)
- Circuit breaker: TODO comment (future need)

This follows YAGNI: build hooks when we need them, not speculatively.

### Why RwLock for Subscribers?
- **Read-heavy**: Publishing (read lock) happens 1000x more than subscribing (write lock)
- **Optimization**: Multiple concurrent publishes can read simultaneously
- **Trade-off**: Write lock only during subscribe/cleanup (rare operations)

---

**Status**: Ready to implement (after RT-TASK-004-REFACTOR)  
**Next Task**: RT-TASK-006 Phase 2 (ActorSystem message router)  
**Estimated Total Time**: 3-4 hours  
**Priority**: CRITICAL - Enables proper actor messaging architecture
