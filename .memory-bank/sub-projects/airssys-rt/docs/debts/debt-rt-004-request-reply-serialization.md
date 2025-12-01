# DEBT-RT-004: Request-Reply Serialization Performance

**Sub-Project:** airssys-rt  
**Component:** Message Broker (InMemoryMessageBroker)  
**Severity:** high  
**Created:** 2025-10-05  
**Target Resolution:** RT-TASK-010 (Performance Optimization) or Q1 2026  

## Problem Statement

Current request-reply implementation uses JSON serialization for in-process message passing, violating zero-copy architecture principles and creating significant performance overhead.

### Current Implementation

```rust
// airssys-rt/src/broker/in_memory.rs:69
pending_requests: DashMap<uuid::Uuid, oneshot::Sender<Vec<u8>>>,
//                                                      ^^^^^^^^
//                                                      Forces serialization!

// Lines 168-175: Reply routing serializes message
let serialized = serde_json::to_vec(&envelope).map_err(|e| {
    BrokerError::RouteError {
        message_type: M::MESSAGE_TYPE,
        reason: format!("Failed to serialize reply: {e}"),
    }
})?;

// Lines 227-232: Request deserializes response
let response: MessageEnvelope<R> =
    serde_json::from_slice(&serialized).map_err(|e| BrokerError::RouteError {
        message_type: R::MESSAGE_TYPE,
        reason: format!("Failed to deserialize reply: {e}"),
    })?;
```

## Performance Impact

### Measured Costs (Estimated)
- **JSON serialization**: ~1-5μs per message (depending on size)
- **JSON deserialization**: ~1-5μs per message
- **Total overhead**: ~2-10μs per request-reply cycle
- **Memory allocations**: 2+ heap allocations per cycle
- **Throughput reduction**: ~50% compared to zero-copy approach

### Violations
- ❌ Violates **KNOWLEDGE-RT-002**: Message Broker Zero-Copy Patterns
- ❌ Violates **ADR-RT-002**: Message Passing Architecture (lazy serialization)
- ❌ Performance target: <1μs routing overhead (currently 2-10μs)
- ❌ Memory target: <1KB/s allocation rate (currently much higher)

## Root Cause

### Technical Constraint
Using `oneshot::Sender<Vec<u8>>` to handle heterogeneous message types (request type `M` ≠ response type `R`):

```rust
// Problem: Need to send different types through same channel
broker.request::<ResponseType>(envelope, timeout).await?;
//               ^^^^^^^^^^^^^ Type R (response)
//     envelope: MessageEnvelope<M> (request type)

// Solution chosen: Serialize to common type Vec<u8>
// Better solution: Type erasure with TypedBox pattern
```

### Design Decision
Chose simplest working solution (JSON) to complete RT-TASK-004 Phase 4, deferring optimization to avoid premature complexity.

## Proposed Solution

### Phase 1: TypedBox Pattern (In-Process Optimization)

Replace `Vec<u8>` with type-erased wrapper following **KNOWLEDGE-RT-002** patterns:

```rust
/// Type-safe wrapper for heterogeneous message passing
struct TypedMessageBox {
    /// Type-erased message envelope
    message: Box<dyn Any + Send>,
    
    /// Runtime type tag for verification (§M-ESSENTIAL-FN-INHERENT)
    message_type: &'static str,
    
    /// Message metadata for debugging
    correlation_id: uuid::Uuid,
}

impl TypedMessageBox {
    /// Create typed box from message envelope
    pub fn new<M: Message>(envelope: MessageEnvelope<M>) -> Self {
        Self {
            message: Box::new(envelope),
            message_type: M::MESSAGE_TYPE,
            correlation_id: envelope.correlation_id.unwrap_or_else(uuid::Uuid::nil),
        }
    }
    
    /// Safe downcast with type verification
    pub fn downcast<R: Message>(self) -> Result<MessageEnvelope<R>, BrokerError> {
        // Verify type BEFORE downcasting (prevents panic)
        if self.message_type != R::MESSAGE_TYPE {
            return Err(BrokerError::TypeMismatch {
                expected: R::MESSAGE_TYPE,
                got: self.message_type,
                correlation_id: self.correlation_id,
            });
        }
        
        // Safe unwrap: type tag verified
        self.message
            .downcast::<MessageEnvelope<R>>()
            .map(|boxed| *boxed)
            .map_err(|_| BrokerError::InternalError {
                reason: "Type tag verified but downcast failed".to_string(),
            })
    }
}

// Updated broker field:
pending_requests: DashMap<uuid::Uuid, oneshot::Sender<TypedMessageBox>>,
```

**Performance Benefits:**
- ✅ Zero serialization cost
- ✅ Single heap allocation (Box only)
- ✅ Type safety through runtime verification
- ✅ ~100x faster than JSON (nanoseconds vs microseconds)
- ✅ Clear error messages on type mismatch

### Phase 2: Serialization Strategy Selection (Cross-Boundary)

When serialization IS required (WASM, network, persistence), choose high-performance format:

#### Format Comparison

| Format | Speed | Size | Features | Use Case |
|--------|-------|------|----------|----------|
| **JSON** | ⭐ (slow) | ⭐⭐ | Human-readable, universal | Debugging only |
| **bincode** | ⭐⭐⭐⭐ | ⭐⭐⭐⭐ | Fast, compact, Rust-native | In-process boundaries |
| **MessagePack** | ⭐⭐⭐ | ⭐⭐⭐ | Compact, cross-language | Network protocols |
| **Cap'n Proto** | ⭐⭐⭐⭐⭐ | ⭐⭐⭐⭐⭐ | Zero-copy, schema-based | High-performance IPC |
| **FlatBuffers** | ⭐⭐⭐⭐⭐ | ⭐⭐⭐⭐⭐ | Zero-copy, forward-compatible | WASM boundaries |
| **Multiformat** | ⭐⭐⭐⭐ | ⭐⭐⭐⭐ | Self-describing, upgradable | AirsStack ecosystem |

#### Recommended Strategy

```rust
/// Serialization strategy based on boundary type
pub enum SerializationBoundary {
    /// In-process: Use TypedBox (zero-copy)
    InProcess,
    
    /// Cross-process: Use bincode (fast binary)
    CrossProcess,
    
    /// Network: Use MessagePack (compact, cross-platform)
    Network,
    
    /// WASM: Use FlatBuffers (zero-copy, portable)
    Wasm,
    
    /// Persistence: Use Multiformat (self-describing, upgradable)
    Storage,
}

impl MessageEnvelope<M> {
    /// Serialize with strategy selection
    pub fn serialize(&self, boundary: SerializationBoundary) -> Result<Vec<u8>, Error> {
        match boundary {
            SerializationBoundary::InProcess => {
                panic!("InProcess should use TypedBox, not serialization")
            }
            SerializationBoundary::CrossProcess => {
                bincode::serialize(self)  // ~10x faster than JSON
            }
            SerializationBoundary::Network => {
                rmp_serde::to_vec(self)  // MessagePack
            }
            SerializationBoundary::Wasm => {
                // FlatBuffers (requires schema generation)
                todo!("Implement FlatBuffers serialization")
            }
            SerializationBoundary::Storage => {
                // Multiformat with version tag
                todo!("Implement Multiformat serialization")
            }
        }
    }
}
```

### Phase 3: Multiformat Integration (Future)

Integrate with AirsStack Multiformat system for:
- Self-describing message formats
- Version-aware serialization
- Codec negotiation
- Cross-component compatibility

**Reference:** ADR-RT-006 (Message Serialization Strategy) - TO BE CREATED

## Migration Path

### Step 1: Add TypedMessageBox (Breaking Change)
- Create `TypedMessageBox` struct in `src/broker/typed_box.rs`
- Add `TypeMismatch` variant to `BrokerError`
- Update `InMemoryMessageBroker::pending_requests` type
- Update `send_impl` and `request_impl` implementations
- Update all tests to verify type safety

### Step 2: Add Serialization Strategy (Non-Breaking)
- Add `SerializationBoundary` enum
- Implement `serialize()` method with strategy selection
- Add benchmarks comparing formats
- Document recommendations in ADR-RT-006

### Step 3: Multiformat Integration (Future)
- Coordinate with airssys-wasm for WASM boundaries
- Integrate Multiformat codec system
- Add version negotiation protocol
- Implement backward compatibility

## Acceptance Criteria

### Must Have
- [x] ~~In-process request-reply uses JSON~~ → TypedBox pattern
- [ ] Performance target: <100ns per request-reply overhead
- [ ] Zero serialization for in-process communication
- [ ] Type safety through runtime verification
- [ ] All existing tests pass with new implementation

### Should Have
- [ ] Serialization strategy selection for boundaries
- [ ] Benchmark comparison (JSON vs bincode vs MessagePack)
- [ ] Clear error messages for type mismatches
- [ ] Performance regression tests

### Nice to Have
- [ ] Multiformat integration for cross-component messaging
- [ ] Automatic codec negotiation
- [ ] Compression for large messages
- [ ] Zero-copy deserialization (Cap'n Proto/FlatBuffers)

## Related Work

### Upstream Dependencies
- **RT-TASK-004**: Message Broker Core (current debt source)
- **RT-TASK-006**: Actor System Framework (will use request-reply heavily)
- **RT-TASK-010**: Performance Optimization Sprint (target resolution)

### Downstream Impact
- **RT-TASK-007**: Supervisor Framework (uses request-reply for health checks)
- **RT-TASK-011**: Distributed Actor System (needs network serialization)
- **Integration**: airssys-wasm (needs WASM-compatible serialization)

### Architecture Decisions
- **ADR-RT-002**: Message Passing Architecture (lazy serialization principle)
- **ADR-RT-006**: Message Serialization Strategy (TO BE CREATED)
- **KNOWLEDGE-RT-002**: Message Broker Zero-Copy Patterns (violated by current impl)

## Notes

### Why Not Fixed Immediately?
Following YAGNI principle and iterative development:
1. Current implementation works and is type-safe
2. Optimization requires careful benchmarking
3. Serialization strategy depends on boundary detection
4. Multiformat integration needs coordination with other components

### Performance Priorities
1. **High**: In-process request-reply (TypedBox) - used heavily
2. **Medium**: Cross-process IPC (bincode) - used occasionally  
3. **Low**: Network/WASM (MessagePack/FlatBuffers) - future features

### Risk Assessment
- **Low risk**: TypedBox pattern well-documented in memory bank
- **Medium risk**: Serialization format selection needs benchmarking
- **High risk**: Multiformat integration requires ecosystem coordination

---

**Resolution Target:** Q1 2026 or RT-TASK-010 (Performance Optimization)  
**Estimated Effort:** 2-3 days for Phase 1, 1-2 weeks for full implementation  
**Performance Gain:** ~100x improvement for in-process request-reply  
**Breaking Change:** Yes (API-compatible, implementation change)
