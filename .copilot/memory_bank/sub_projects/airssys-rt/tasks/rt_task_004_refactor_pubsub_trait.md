# RT-TASK-004-REFACTOR: MessageBroker Pub-Sub Trait Refactoring

**Task ID:** RT-TASK-004-REFACTOR  
**Parent Task:** RT-TASK-004 (Message Broker Core)  
**Priority:** CRITICAL  
**Status:** Not Started  
**Estimated Time:** 2-3 hours  
**Created:** 2025-10-06  
**Blocking:** RT-TASK-004-PUBSUB, RT-TASK-006 Phase 2  

---

## Objective

Refactor the `MessageBroker<M>` trait from direct routing semantics to true pub-sub pattern by adding `publish()` and `subscribe()` methods. This is Phase 0 of the pub-sub architecture implementation as defined in ADR-006.

## Context

**Architecture Breakthrough**: During RT-TASK-006 Phase 2 implementation, we discovered that MessageBroker must be a true pub-sub message bus, not a direct routing system. This enables:
- ✅ Extensibility hooks (logging, metrics, persistence)
- ✅ Multiple subscribers (routing, monitoring, audit)
- ✅ Distributed broker implementations (Redis, NATS)
- ✅ Dead letter queue support
- ✅ Observability infrastructure

**Related Documentation:**
- **ADR-006**: MessageBroker Pub-Sub Architecture
- **KNOWLEDGE-RT-012**: Pub-Sub MessageBroker Pattern (complete implementation guide)
- **DEBT-RT-005**: Actor System Broker Integration Mismatch

## Scope

### Files to Modify
- `src/broker/traits.rs` - MessageBroker trait definition (~250 lines)
- `src/broker/mod.rs` - Module exports (add MessageStream)
- Update trait tests (~5 tests)

### Files NOT Modified (Out of Scope)
- `src/broker/in_memory.rs` - Will be updated in RT-TASK-004-PUBSUB
- `src/broker/registry.rs` - No changes needed (perfect as-is)
- `src/broker/error.rs` - No new error variants needed yet

---

## Implementation Phases

### Phase 1: Add MessageStream Type (30 minutes)

**File:** `src/broker/traits.rs`

**Add new type before MessageBroker trait:**

```rust
use tokio::sync::mpsc;

/// Message stream from broker subscriptions.
///
/// A stream of messages published to the broker. Multiple subscribers can
/// independently receive all published messages.
///
/// # Example
///
/// ```ignore
/// let mut stream = broker.subscribe().await?;
///
/// while let Some(envelope) = stream.recv().await {
///     // Process message
///     route_to_actor(envelope).await?;
/// }
/// ```
pub struct MessageStream<M: Message> {
    receiver: mpsc::UnboundedReceiver<MessageEnvelope<M>>,
}

impl<M: Message> MessageStream<M> {
    /// Create a new message stream.
    ///
    /// This is typically called internally by broker implementations.
    pub fn new(receiver: mpsc::UnboundedReceiver<MessageEnvelope<M>>) -> Self {
        Self { receiver }
    }

    /// Receive next message from stream.
    ///
    /// Returns `None` when the stream is closed (all publishers dropped).
    pub async fn recv(&mut self) -> Option<MessageEnvelope<M>> {
        self.receiver.recv().await
    }

    /// Try to receive without blocking.
    ///
    /// Returns:
    /// - `Ok(envelope)` - Message available
    /// - `Err(TryRecvError::Empty)` - No messages available
    /// - `Err(TryRecvError::Disconnected)` - Stream closed
    pub fn try_recv(&mut self) -> Result<MessageEnvelope<M>, mpsc::error::TryRecvError> {
        self.receiver.try_recv()
    }
}
```

**Update module exports in `src/broker/mod.rs`:**

```rust
pub use traits::{MessageBroker, MessageStream};
```

**Tests:**
```rust
#[test]
fn test_message_stream_creation() {
    let (tx, rx) = mpsc::unbounded_channel();
    let stream = MessageStream::<TestMessage>::new(rx);
    // Verify stream is created
}
```

---

### Phase 2: Add `publish()` Method (45 minutes)

**File:** `src/broker/traits.rs`

**Add new method to MessageBroker trait:**

```rust
/// Publish a message to the broker bus.
///
/// Messages are broadcast to all subscribers. This is the fundamental
/// operation for actor-to-actor communication in the pub-sub architecture.
///
/// # Pub-Sub Semantics
///
/// Unlike direct routing, `publish()` does NOT directly deliver to a specific
/// actor. Instead, it broadcasts the message to all subscribers (typically
/// ActorSystem routers), which then route to the appropriate actor mailbox.
///
/// # Extensibility Hooks
///
/// Implementations can add hooks for:
/// - Logging and distributed tracing
/// - Metrics collection (message rates, sizes)
/// - Message persistence for replay
/// - Circuit breakers for resilience
/// - Rate limiting for fairness
///
/// # Arguments
///
/// * `envelope` - The message envelope containing message and metadata
///
/// # Errors
///
/// Returns error if:
/// - Broker is shut down
/// - Persistence layer fails (if enabled)
/// - Circuit breaker is open (if enabled)
///
/// # Example
///
/// ```ignore
/// let envelope = MessageEnvelope::new(message)
///     .with_sender(sender_address)
///     .with_recipient(recipient_address);
///
/// broker.publish(envelope).await?;
/// // Message broadcast to all subscribers
/// ```
async fn publish(&self, envelope: MessageEnvelope<M>) -> Result<(), Self::Error>;
```

**Note**: Keep existing `send()` method for now - we'll deprecate it in Phase 4.

**Tests:**
```rust
#[test]
fn test_publish_method_signature_exists() {
    // Compile-time test that publish() exists with correct signature
    fn assert_publish<B: MessageBroker<TestMessage>>() {}
    assert_publish::<MockBroker>();
}
```

---

### Phase 3: Add `subscribe()` Method (45 minutes)

**File:** `src/broker/traits.rs`

**Add new method to MessageBroker trait:**

```rust
/// Subscribe to message events on the broker.
///
/// Returns a stream of all messages published to the broker. Multiple
/// subscribers can listen to the same message stream independently.
///
/// # Subscriber Lifecycle
///
/// The subscription remains active until the MessageStream is dropped.
/// When the stream is dropped, the subscriber is automatically unregistered.
///
/// # Use Cases
///
/// - **ActorSystem**: Subscribes to route messages to actors via ActorRegistry
/// - **Monitor Service**: Subscribes for observability and metrics collection
/// - **Audit Service**: Subscribes for compliance logging and event sourcing
/// - **Dead Letter Queue**: Subscribes to capture undeliverable messages
///
/// # Multiple Subscribers
///
/// Each subscriber receives ALL published messages independently. The broker
/// maintains separate channels for each subscriber.
///
/// # Errors
///
/// Returns error if:
/// - Broker is shut down
/// - Maximum subscriber limit reached (implementation-specific)
///
/// # Example
///
/// ```ignore
/// // ActorSystem subscribes for routing
/// let mut routing_stream = broker.subscribe().await?;
///
/// tokio::spawn(async move {
///     while let Some(envelope) = routing_stream.recv().await {
///         // Route to actor via registry
///         if let Some(recipient) = envelope.metadata.reply_to {
///             let sender = registry.resolve(&recipient)?;
///             sender.send(envelope).await?;
///         }
///     }
/// });
///
/// // Monitor subscribes for metrics
/// let mut monitor_stream = broker.subscribe().await?;
///
/// tokio::spawn(async move {
///     while let Some(envelope) = monitor_stream.recv().await {
///         metrics.record_message(&envelope);
///     }
/// });
/// ```
async fn subscribe(&self) -> Result<MessageStream<M>, Self::Error>;
```

**Tests:**
```rust
#[test]
fn test_subscribe_method_signature_exists() {
    // Compile-time test that subscribe() exists with correct signature
    fn assert_subscribe<B: MessageBroker<TestMessage>>() {}
    assert_subscribe::<MockBroker>();
}
```

---

### Phase 4: Update `request()` → `publish_request()` (30 minutes)

**File:** `src/broker/traits.rs`

**Rename and update documentation:**

```rust
/// Publish a request and wait for correlated reply.
///
/// This is a convenience method that combines publish with correlation
/// tracking for request-reply patterns. It's built on top of the pub-sub
/// infrastructure.
///
/// # How It Works
///
/// 1. Generate correlation ID for request tracking
/// 2. Register reply channel in pending requests
/// 3. Publish request via `publish()`
/// 4. Wait for reply with timeout
/// 5. Clean up pending request on completion/timeout
///
/// # Type Parameters
///
/// * `R` - The expected response message type (must implement `Message`)
///
/// # Arguments
///
/// * `envelope` - The request message envelope (should include correlation_id)
/// * `timeout` - Maximum duration to wait for a response
///
/// # Returns
///
/// - `Ok(Some(envelope))` - Reply received within timeout
/// - `Ok(None)` - Timeout expired with no reply
/// - `Err(error)` - Broker error occurred
///
/// # Errors
///
/// Returns error if:
/// - Missing correlation_id in envelope
/// - Publish failed
/// - Internal broker error
///
/// # Example
///
/// ```ignore
/// use std::time::Duration;
///
/// let correlation_id = CorrelationId::new();
/// let request = MessageEnvelope::new(AuthRequest { username, password })
///     .with_sender(requester_address)
///     .with_recipient(auth_service_address)
///     .with_correlation_id(correlation_id);
///
/// let reply = broker.publish_request::<AuthResponse>(
///     request,
///     Duration::from_secs(5)
/// ).await?;
///
/// match reply {
///     Some(envelope) => println!("Got reply: {:?}", envelope.message),
///     None => println!("Request timeout"),
/// }
/// ```
///
/// # Performance Considerations
///
/// Request-reply is a blocking operation that holds a task waiting for response.
/// For long-running operations, consider using fire-and-forget with manual
/// correlation IDs instead (see KNOWLEDGE-RT-010 Pattern 3).
async fn publish_request<R: Message + for<'de> serde::Deserialize<'de>>(
    &self,
    envelope: MessageEnvelope<M>,
    timeout: Duration,
) -> Result<Option<MessageEnvelope<R>>, Self::Error>;
```

**Keep old `request()` method for now** - mark as deprecated:

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

---

### Phase 5: Update Documentation (30 minutes)

**Update trait-level documentation in `src/broker/traits.rs`:**

```rust
/// Generic message broker trait for pub-sub message routing.
///
/// The broker implements a **publish-subscribe pattern** where:
/// - Publishers (actors via ActorContext) publish messages to the bus
/// - Subscribers (ActorSystem, monitors, auditors) receive all messages
/// - Routing logic is handled by subscribers, not the broker
///
/// # Architecture: Separation of Concerns
///
/// ```text
/// ┌─────────────────────────────────────────────────────────────┐
/// │                      MESSAGE BUS                             │
/// │                    (MessageBroker)                           │
/// │                                                              │
/// │  Publishers                Topics              Subscribers   │
/// │     │                        │                      │        │
/// │  Actor─────publish──────────>│                      │        │
/// │  Context                     │<─────subscribe───────System   │
/// │                              │                      │        │
/// │                              │──notify────────────> │        │
/// │                              │  (message arrives)   │        │
/// └─────────────────────────────────────────────────────────────┘
///                                                        │
///                                                        ▼
///                                               ActorRegistry.resolve()
///                                                        │
///                                                        ▼
///                                                 Actor Mailbox
/// ```
///
/// # Component Responsibilities
///
/// - **MessageBroker**: Pub-sub transport layer (THIS TRAIT)
///   - Publish messages to all subscribers
///   - Manage subscriber lifecycle
///   - Provide extensibility hooks
///   - Does NOT route to specific actors
///
/// - **ActorRegistry**: Routing table (separate component)
///   - Map ActorAddress → MailboxSender
///   - Register/unregister actors
///   - Resolve addresses to mailboxes
///   - Does NOT transport messages
///
/// - **ActorSystem**: Orchestration (subscriber)
///   - Subscribe to broker's message stream
///   - Route messages using ActorRegistry
///   - Handle dead letters
///   - Spawn/stop actors
///   - Does NOT directly deliver messages
///
/// - **ActorContext**: Publisher API (actor-facing)
///   - Publish messages via broker
///   - Simple send/request API for actors
///   - Does NOT know about routing
///
/// # Why Pub-Sub Architecture?
///
/// ✅ **Extensibility**: Natural hooks for logging, metrics, persistence  
/// ✅ **Multiple Subscribers**: System, monitor, audit independently  
/// ✅ **Distributed Ready**: Redis/NATS brokers without changing actors  
/// ✅ **Testability**: Stream-based APIs easier to mock  
/// ✅ **Observability**: Message flows visible to monitoring  
/// ✅ **Dead Letter Support**: Undeliverable messages naturally handled
///
/// # Example (System-Level Usage)
///
/// ```ignore
/// use airssys_rt::broker::{InMemoryMessageBroker, MessageBroker};
/// use airssys_rt::message::MessageEnvelope;
/// use std::time::Duration;
///
/// // ActorSystem creates broker
/// let broker = InMemoryMessageBroker::<MyMessage>::new();
///
/// // ActorSystem subscribes for routing
/// let mut stream = broker.subscribe().await?;
/// tokio::spawn(async move {
///     while let Some(envelope) = stream.recv().await {
///         // Route via registry to actor mailbox
///         route_to_actor(envelope, &registry).await?;
///     }
/// });
///
/// // Actor publishes message (via ActorContext)
/// let envelope = MessageEnvelope::new(message)
///     .with_sender(sender_address)
///     .with_recipient(recipient_address);
/// broker.publish(envelope).await?;
/// ```
///
/// # Implementation Requirements
///
/// Implementations must:
/// - Be `Send + Sync` for concurrent access across async tasks
/// - Implement `Clone` for cheap broker handle distribution (M-SERVICES-CLONE)
/// - Use generic constraints, not trait objects (§6.2 - Avoid dyn Patterns)
/// - Provide comprehensive error handling via `Error` associated type
/// - Support multiple independent subscribers
/// - Broadcast messages to all subscribers
/// - Handle subscriber disconnection gracefully
///
/// # See Also
///
/// - **ADR-006**: MessageBroker Pub-Sub Architecture Decision
/// - **KNOWLEDGE-RT-012**: Complete pub-sub implementation guide
/// - **KNOWLEDGE-RT-009**: Message broker architecture patterns
```

---

### Phase 6: Update Tests (30 minutes)

**Update existing tests in `src/broker/traits.rs`:**

```rust
#[cfg(test)]
mod tests {
    use super::*;

    // Test that MessageBroker trait requirements are properly defined
    #[test]
    fn test_trait_requirements() {
        fn assert_send_sync<T: Send + Sync>() {}
        fn assert_clone<T: Clone>() {}
        
        // MessageBroker must be Send + Sync + Clone
        fn check<M: Message, B: MessageBroker<M>>() {
            assert_send_sync::<B>();
            assert_clone::<B>();
        }
    }

    #[test]
    fn test_message_stream_type_exists() {
        // Compile-time verification that MessageStream exists
        fn assert_stream<M: Message>() {
            let (_tx, rx) = tokio::sync::mpsc::unbounded_channel();
            let _stream = MessageStream::<M>::new(rx);
        }
    }

    #[test]
    fn test_publish_method_in_trait() {
        // Verify publish() is part of trait (compile-time check)
        fn has_publish<M: Message, B: MessageBroker<M>>() {}
    }

    #[test]
    fn test_subscribe_method_in_trait() {
        // Verify subscribe() is part of trait (compile-time check)
        fn has_subscribe<M: Message, B: MessageBroker<M>>() {}
    }

    #[test]
    fn test_publish_request_method_in_trait() {
        // Verify publish_request() is part of trait (compile-time check)
        fn has_publish_request<M: Message, B: MessageBroker<M>>() {}
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

### Documentation
- [ ] Trait-level documentation updated with pub-sub architecture
- [ ] All new methods have comprehensive doc comments
- [ ] Examples in documentation are clear and correct
- [ ] Cross-references to ADR-006 and KNOWLEDGE-RT-012

### Testing
- [ ] Trait requirement tests updated
- [ ] MessageStream type tests added
- [ ] Method signature tests added (compile-time verification)
- [ ] All existing tests still pass

### Standards Compliance
- [ ] §2.1: 3-layer import organization ✅
- [ ] §3.2: chrono DateTime<Utc> (not applicable)
- [ ] §4.3: Module architecture - trait definition only
- [ ] §6.1: YAGNI - pub-sub needed for monitoring/observability
- [ ] §6.2: Avoid dyn - using generic constraints `B: MessageBroker<M>`
- [ ] §6.3: M-DI-HIERARCHY - generic constraints over trait objects ✅

---

## Dependencies

### Depends On
- None (foundation refactoring)

### Blocks
- **RT-TASK-004-PUBSUB**: Cannot implement pub-sub until trait is updated
- **RT-TASK-006 Phase 2**: Cannot implement ActorSystem until broker has subscribe()

---

## Expected Outcomes

### Files Modified
- `src/broker/traits.rs` - Trait with pub-sub methods (~280 lines, +40 lines)
- `src/broker/mod.rs` - Export MessageStream (~60 lines, +1 line)

### Tests
- Existing tests: 3 tests (updated)
- New tests: 5 tests
- Total: 8 tests passing

### Documentation
- Trait-level pub-sub architecture documented
- All methods with comprehensive examples
- Cross-references to ADR-006 and KNOWLEDGE-RT-012

### Breaking Changes
- None initially (keeping `send()` and `request()` deprecated)
- InMemoryMessageBroker will need updates in RT-TASK-004-PUBSUB

---

## Notes

### Why Separate from Implementation?
1. **Incremental Progress**: Can validate API design before implementation
2. **Clear Interface**: Trait changes are contract changes
3. **Better Review**: Focused on API design, not implementation details
4. **Compilation Check**: InMemoryMessageBroker will fail to compile, showing what needs updating

### Temporary State
After this task, `InMemoryMessageBroker` will NOT compile because it doesn't implement new methods. This is intentional - RT-TASK-004-PUBSUB will fix it.

### Migration Path
1. Complete this task (trait definition)
2. InMemoryMessageBroker compilation fails (expected)
3. Complete RT-TASK-004-PUBSUB (implement new methods)
4. All tests pass
5. Resume RT-TASK-006 Phase 2 with correct architecture

---

**Status**: Ready to implement  
**Next Task**: RT-TASK-004-PUBSUB (depends on this)  
**Estimated Total Time**: 2-3 hours  
**Priority**: CRITICAL - Blocks RT-TASK-006 Phase 2
