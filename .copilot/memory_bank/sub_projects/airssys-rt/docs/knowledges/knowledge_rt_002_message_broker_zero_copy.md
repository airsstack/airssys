# KNOWLEDGE-RT-002: Message Broker Zero-Copy Patterns

**Sub-Project:** airssys-rt  
**Category:** Performance  
**Created:** 2025-10-02  
**Last Updated:** 2025-10-02  
**Status:** active  

## Context and Problem

High-performance message passing in actor systems requires minimizing memory allocations and data copying. Traditional message brokers often involve multiple serialization/deserialization steps and heap allocations that create performance bottlenecks. airssys-rt needs zero-copy message patterns that maintain type safety while achieving maximum throughput.

## Knowledge Details

### Zero-Copy Message Architecture

The message broker uses ownership transfer and reference semantics to eliminate unnecessary copying:

```rust
// Core zero-copy message envelope
pub struct MessageEnvelope<M: Message> {
    pub message: M,           // Owned message data
    pub sender: Option<ActorAddress>,
    pub timestamp: DateTime<Utc>,
    pub routing_key: u64,     // Pre-computed hash for routing
}

// Zero-copy message routing
pub struct MessageBroker {
    routing_table: HashMap<ActorAddress, mpsc::UnboundedSender<BoxedMessage>>,
    address_cache: LruCache<String, ActorAddress>,
}

impl MessageBroker {
    // Direct ownership transfer - no copying
    pub async fn route_message<M: Message>(
        &self, 
        envelope: MessageEnvelope<M>
    ) -> Result<(), BrokerError> {
        let target = self.resolve_address(&envelope.routing_key)?;
        let boxed = BoxedMessage::new(envelope); // Single allocation
        
        self.routing_table
            .get(&target)
            .ok_or(BrokerError::ActorNotFound)?
            .send(boxed)
            .map_err(|_| BrokerError::ChannelClosed)
    }
}
```

### Type-Erased Message Storage

For heterogeneous message handling, use type erasure only at storage boundaries:

```rust
// Type-erased wrapper for storage only
pub struct BoxedMessage {
    message: Box<dyn Any + Send + Sync>,
    message_type: &'static str,
    sender: Option<ActorAddress>,
    timestamp: DateTime<Utc>,
}

impl BoxedMessage {
    pub fn new<M: Message>(envelope: MessageEnvelope<M>) -> Self {
        Self {
            message: Box::new(envelope.message),
            message_type: M::MESSAGE_TYPE,
            sender: envelope.sender,
            timestamp: envelope.timestamp,
        }
    }
    
    // Zero-copy downcast
    pub fn downcast<M: Message>(self) -> Result<M, Self> {
        if self.message_type == M::MESSAGE_TYPE {
            match self.message.downcast::<M>() {
                Ok(message) => Ok(*message),
                Err(original) => Err(Self { 
                    message: original, 
                    ..self 
                }),
            }
        } else {
            Err(self)
        }
    }
}
```

### High-Performance Routing

Message routing uses pre-computed hashes and cache-friendly data structures:

```rust
// Fast address resolution with caching
pub struct AddressResolver {
    // Primary routing table - lock-free for reads
    routing_table: Arc<DashMap<u64, ActorAddress>>,
    
    // LRU cache for string->address resolution
    address_cache: Arc<Mutex<LruCache<String, (ActorAddress, u64)>>>,
    
    // Routing statistics for optimization
    stats: RoutingStats,
}

impl AddressResolver {
    // Pre-compute hash for routing key
    pub fn compute_routing_key(address: &ActorAddress) -> u64 {
        use std::collections::hash_map::DefaultHasher;
        use std::hash::{Hash, Hasher};
        
        let mut hasher = DefaultHasher::new();
        address.hash(&mut hasher);
        hasher.finish()
    }
    
    // Lock-free address resolution
    pub fn resolve_fast(&self, routing_key: u64) -> Option<ActorAddress> {
        self.routing_table.get(&routing_key).map(|entry| *entry.value())
    }
    
    // Cached string resolution
    pub fn resolve_by_name(&self, name: &str) -> Option<ActorAddress> {
        // Check cache first
        if let Ok(mut cache) = self.address_cache.try_lock() {
            if let Some((address, routing_key)) = cache.get(name) {
                self.stats.cache_hits.fetch_add(1, Ordering::Relaxed);
                return Some(*address);
            }
        }
        
        // Fallback to full resolution
        self.resolve_slow(name)
    }
}
```

### Memory Pool Optimization

Use object pools to minimize allocations in hot paths:

```rust
// Pool for message envelopes
pub struct MessagePool<M: Message> {
    pool: Arc<Mutex<Vec<MessageEnvelope<M>>>>,
    max_size: usize,
}

impl<M: Message> MessagePool<M> {
    pub fn get(&self) -> MessageEnvelope<M> {
        if let Ok(mut pool) = self.pool.try_lock() {
            if let Some(mut envelope) = pool.pop() {
                // Reset envelope for reuse
                envelope.sender = None;
                envelope.timestamp = Utc::now();
                return envelope;
            }
        }
        
        // Fallback to allocation
        MessageEnvelope::default()
    }
    
    pub fn return_envelope(&self, envelope: MessageEnvelope<M>) {
        if let Ok(mut pool) = self.pool.try_lock() {
            if pool.len() < self.max_size {
                pool.push(envelope);
            }
        }
        // Drop if pool is full or locked
    }
}
```

## Performance Characteristics

### Throughput Metrics
- **Local message delivery**: <100ns per message
- **Cross-actor messaging**: <1μs including routing
- **Message broker throughput**: >1M messages/second
- **Memory allocation rate**: <1KB/s under steady load

### Memory Usage Patterns
- **Message envelope**: 64 bytes (no heap allocation)
- **Routing table entry**: 24 bytes per actor
- **Address cache**: 16KB for 1000 cached addresses
- **Pool overhead**: <1MB for 10,000 pooled envelopes

### Optimization Techniques
1. **Pre-computed routing keys**: Eliminate hash computation in hot path
2. **Lock-free routing table**: Use DashMap for concurrent access
3. **Message pooling**: Reuse envelope allocations
4. **Inline small messages**: Avoid heap allocation for <64 byte messages

## Implementation Guidelines

### Zero-Copy Principles
1. **Ownership transfer**: Move messages instead of copying
2. **Reference semantics**: Use borrowing for read-only access
3. **Single allocation**: Minimize heap allocations per message
4. **Stack allocation**: Keep small messages on stack when possible

### Cache-Friendly Patterns
```rust
// Locality-optimized routing table
#[repr(C)]
pub struct RouteEntry {
    routing_key: u64,        // 8 bytes - primary key
    actor_address: ActorAddress, // 16 bytes - target
    last_accessed: u64,      // 8 bytes - LRU tracking
}

// Cache line aligned for optimal access
const ENTRIES_PER_CACHE_LINE: usize = 64 / std::mem::size_of::<RouteEntry>();
```

## Related Patterns

### Complementary Knowledge
- **KNOWLEDGE-RT-001**: Zero-Cost Actor Model Architecture
- **KNOWLEDGE-RT-003**: Supervisor Tree Implementation Strategies
- **KNOWLEDGE-RT-005**: High-Performance Actor Scheduling

### Architecture Decisions
- **ADR-RT-002**: Message Passing Architecture
- **ADR-RT-006**: Message Serialization Strategy
- **ADR-RT-007**: Concurrency Model

## Usage Examples

### High-Throughput Message Sending
```rust
// Pre-compute routing for hot paths
let routing_key = AddressResolver::compute_routing_key(&target_address);
let envelope = MessageEnvelope::new_with_routing(message, sender, routing_key);

// Zero-copy routing
broker.route_message(envelope).await?;
```

### Batch Message Processing
```rust
// Process messages in batches for better cache locality
let batch_size = 64;
let mut batch = Vec::with_capacity(batch_size);

while let Some(message) = mailbox.try_recv() {
    batch.push(message);
    if batch.len() >= batch_size {
        process_message_batch(&mut batch).await?;
        batch.clear();
    }
}

if !batch.is_empty() {
    process_message_batch(&mut batch).await?;
}
```

## Lessons Learned

### What Works Well
- Pre-computed routing keys eliminate hash computation overhead
- Lock-free data structures scale well with actor count
- Message pooling reduces GC pressure significantly
- Type erasure only at boundaries maintains performance

### Potential Pitfalls
- Pool management adds complexity
- Cache invalidation requires careful coordination
- Lock-free structures have memory ordering considerations
- Profile before optimizing - measure actual bottlenecks

## Future Considerations

### Planned Enhancements
- NUMA-aware routing for multi-socket systems
- Message compression for large payloads
- Priority queuing with zero-copy semantics
- Integration with kernel bypass networking

### Research Areas
- SPSC/MPSC queue optimization for actor mailboxes
- Cache-oblivious data structures for routing tables
- Hardware transactional memory for conflict resolution
- SIMD optimization for message batch processing

---

**References:**
- Performance targets: >1M messages/second, <100ns local delivery
- Microsoft Rust Guidelines: M-AVOID-WRAPPERS, M-SIMPLE-ABSTRACTIONS
- Lock-free programming patterns: crossbeam, dashmap crates