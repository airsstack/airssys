# ADR-RT-002: Message Passing Architecture

**Status:** Accepted  
**Date:** 2025-10-02  
**Deciders:** Architecture Team  
**Context:** airssys-rt message system design

## Context

The message passing system is the core communication mechanism for airssys-rt. Design choices around message routing, serialization, delivery guarantees, and performance characteristics will fundamentally impact the entire runtime's capabilities and performance profile.

## Decision

We will implement a **hybrid message passing architecture** that combines zero-copy local delivery with type-safe routing and optional persistence for critical messages.

### Core Architecture

```rust
// Message trait with compile-time type identification
pub trait Message: Send + Sync + 'static {
    const MESSAGE_TYPE: &'static str;
    
    // Optional: Define delivery semantics
    fn delivery_guarantee() -> DeliveryGuarantee {
        DeliveryGuarantee::AtMostOnce  // Default
    }
    
    fn priority() -> MessagePriority {
        MessagePriority::Normal  // Default
    }
}

// Type-safe message envelope
pub struct MessageEnvelope<M: Message> {
    pub message: M,
    pub sender: Option<ActorAddress>,
    pub timestamp: DateTime<Utc>,
    pub routing_key: u64,  // Pre-computed for performance
    pub correlation_id: Option<Uuid>,
}

// Message broker with multiple delivery modes
pub struct MessageBroker {
    local_router: LocalRouter,           // Zero-copy local delivery
    persistent_router: PersistentRouter, // Durable message delivery
    remote_router: RemoteRouter,         // Cross-node messaging (future)
}
```

## Rationale

### Performance Requirements
1. **Local delivery**: <100ns for same-process actor communication
2. **Throughput**: >1M messages/second sustained throughput
3. **Memory efficiency**: Minimal allocations in hot paths
4. **Scalability**: Support 10,000+ concurrent actors

### Reliability Requirements
1. **At-most-once delivery**: Default semantics for performance
2. **At-least-once delivery**: Optional for critical messages
3. **Exactly-once delivery**: Available for transactional scenarios
4. **Message ordering**: FIFO within actor mailboxes

### Type Safety Requirements
1. **Compile-time validation**: Message type mismatches caught early
2. **Structured routing**: Type-safe address resolution
3. **Error propagation**: Comprehensive error handling
4. **API clarity**: Clear message flow semantics

## Implementation Details

### Local Message Routing

Fast path for same-process communication:

```rust
pub struct LocalRouter {
    routing_table: Arc<DashMap<ActorAddress, mpsc::UnboundedSender<BoxedMessage>>>,
    address_resolver: AddressResolver,
    delivery_stats: DeliveryStats,
}

impl LocalRouter {
    // Zero-copy message delivery
    pub async fn deliver_local<M: Message>(
        &self,
        envelope: MessageEnvelope<M>
    ) -> Result<(), DeliveryError> {
        let target = self.address_resolver.resolve_fast(envelope.routing_key)?;
        let boxed = BoxedMessage::new(envelope);  // Single allocation
        
        self.routing_table
            .get(&target)
            .ok_or(DeliveryError::ActorNotFound)?
            .send(boxed)
            .map_err(|_| DeliveryError::MailboxFull)
    }
}
```

### Message Serialization Strategy

Lazy serialization for cross-boundary communication:

```rust
// Trait for message serialization
pub trait SerializableMessage: Message + Serialize + for<'de> Deserialize<'de> {
    fn serialize_binary(&self) -> Result<Vec<u8>, SerializationError>;
    fn deserialize_binary(data: &[u8]) -> Result<Self, SerializationError>;
}

// Optional serialization wrapper
pub struct SerializedMessage {
    message_type: &'static str,
    data: Vec<u8>,
    metadata: MessageMetadata,
}

impl SerializedMessage {
    pub fn from_message<M: SerializableMessage>(message: M) -> Result<Self, SerializationError> {
        Ok(Self {
            message_type: M::MESSAGE_TYPE,
            data: message.serialize_binary()?,
            metadata: MessageMetadata::default(),
        })
    }
}
```

### Delivery Guarantees

Configurable delivery semantics per message type:

```rust
#[derive(Debug, Clone, Copy)]
pub enum DeliveryGuarantee {
    AtMostOnce,   // Fire and forget - fastest
    AtLeastOnce,  // Retry until acknowledged
    ExactlyOnce,  // Transactional delivery
}

#[derive(Debug, Clone, Copy)]
pub enum MessagePriority {
    Critical,  // Highest priority queue
    High,      // High priority queue  
    Normal,    // Default priority
    Low,       // Background processing
}

// Message with delivery requirements
impl Message for CriticalSystemMessage {
    const MESSAGE_TYPE: &'static str = "critical_system";
    
    fn delivery_guarantee() -> DeliveryGuarantee {
        DeliveryGuarantee::AtLeastOnce
    }
    
    fn priority() -> MessagePriority {
        MessagePriority::Critical
    }
}
```

### Actor Mailbox Integration

Type-safe mailbox system with backpressure:

```rust
pub struct Mailbox<M: Message> {
    receiver: mpsc::UnboundedReceiver<MessageEnvelope<M>>,
    sender: mpsc::UnboundedSender<MessageEnvelope<M>>,
    backpressure_config: BackpressureConfig,
    metrics: MailboxMetrics,
}

impl<M: Message> Mailbox<M> {
    pub async fn recv(&mut self) -> Option<MessageEnvelope<M>> {
        self.receiver.recv().await
    }
    
    pub fn try_recv(&mut self) -> Result<MessageEnvelope<M>, TryRecvError> {
        self.receiver.try_recv()
    }
    
    pub async fn send(&self, envelope: MessageEnvelope<M>) -> Result<(), SendError> {
        // Apply backpressure if configured
        if self.should_apply_backpressure() {
            return Err(SendError::Backpressure);
        }
        
        self.sender.send(envelope).map_err(|_| SendError::Disconnected)
    }
}
```

## Consequences

### Positive Consequences
- **Performance**: Zero-copy local delivery achieves target <100ns latency
- **Flexibility**: Multiple delivery modes support different use cases
- **Type safety**: Compile-time message validation prevents runtime errors
- **Scalability**: Design supports target of 10,000+ concurrent actors

### Negative Consequences
- **Complexity**: Multiple routing modes increase implementation complexity
- **Memory usage**: Message envelopes add overhead (~64 bytes per message)
- **API surface**: More configuration options increase cognitive load
- **Testing burden**: Multiple paths require comprehensive test coverage

### Mitigation Strategies
1. **Complexity**: Provide simple default configurations for common cases
2. **Memory usage**: Implement message pooling for high-frequency scenarios
3. **API surface**: Create convenience APIs that hide complexity
4. **Testing**: Automated testing for all delivery modes and error scenarios

## Alternative Approaches

### Option 1: Channel-Only Architecture
```rust
// Simple channel-based messaging
type ActorMailbox = mpsc::UnboundedReceiver<Box<dyn Any>>;
```

**Rejected because:**
- Type erasure loses compile-time safety
- No routing optimization opportunities
- Limited delivery guarantee options
- Poor observability and debugging

### Option 2: Message Queue Integration
```rust
// External message queue (Redis, RabbitMQ)
pub struct ExternalMessageBroker {
    redis_client: RedisClient,
    // ...
}
```

**Rejected because:**
- External dependency increases complexity
- Network latency impacts performance
- Additional operational overhead
- Not suitable for local actor communication

### Option 3: Shared Memory Architecture
```rust
// Lock-free shared memory queues
pub struct SharedMemoryMailbox {
    ring_buffer: AtomicRingBuffer<Message>,
    // ...
}
```

**Rejected because:**
- Complex lock-free programming
- Platform-specific optimizations required
- Limited to single-node deployment
- Memory safety concerns

## Performance Validation

### Benchmarking Strategy
1. **Latency testing**: Message delivery time measurement
2. **Throughput testing**: Sustained message rate testing
3. **Memory profiling**: Allocation and GC pressure analysis
4. **Concurrency testing**: Multi-actor communication patterns

### Performance Targets
- **Local delivery latency**: <100ns (P95)
- **Message throughput**: >1M messages/second
- **Memory overhead**: <64 bytes per message envelope
- **Actor scalability**: 10,000+ concurrent actors

### Monitoring Metrics
```rust
pub struct DeliveryStats {
    pub messages_sent: AtomicU64,
    pub messages_delivered: AtomicU64,
    pub delivery_errors: AtomicU64,
    pub average_latency: AtomicU64,  // Nanoseconds
    pub throughput: AtomicU64,       // Messages per second
}
```

## Future Considerations

### Planned Enhancements
1. **Remote messaging**: Cross-node actor communication
2. **Message persistence**: Durable message storage for critical workflows
3. **Message compression**: Bandwidth optimization for large messages
4. **Priority scheduling**: Message priority-based delivery

### Integration Points
- **OSL integration**: Security context propagation in messages
- **WASM integration**: Message passing to WASM components
- **Monitoring**: Metrics and tracing integration
- **Testing**: Chaos engineering and fault injection

## References

- **BEAM OTP**: Message passing semantics and mailbox behavior
- **Akka**: Actor model message passing patterns
- **Microsoft Rust Guidelines**: M-AVOID-WRAPPERS, M-SIMPLE-ABSTRACTIONS
- **Performance targets**: <100ns delivery, >1M messages/second

---

**Related Decisions:**
- ADR-RT-001: Actor Model Implementation Strategy
- ADR-RT-006: Message Serialization Strategy
- ADR-RT-007: Concurrency Model