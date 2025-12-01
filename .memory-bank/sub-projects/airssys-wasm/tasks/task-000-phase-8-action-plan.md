# WASM-TASK-000 Phase 8 Action Plan: Messaging & Storage Abstractions

**Task:** WASM-TASK-000 Phase 8 Implementation  
**Created:** 2025-10-22  
**Status:** Ready for Implementation  
**Progress:** 75% → 83% (Phases 1-7 complete, Phase 8 next)  
**Phase:** Domain-Specific Abstractions - Part 3: Messaging & Storage (Days 17-19)

---

## Overview

### Context Summary

**Current Status:**
- ✅ Phases 1-7 Complete (75% of WASM-TASK-000)
- 🎯 Phase 8 Next: Messaging & Storage Abstractions
- 📊 Target: 83% completion (10/12 phases)

**Phase 7 Success Pattern:**
- `actor.rs`: 432 lines, comprehensive rustdoc, unit tests
- `security.rs`: 444 lines, comprehensive rustdoc, unit tests
- Both files follow workspace standards exactly (§2.1, §3.2, §6.1)

**Phase 8 Scope:**
- Task 8.1: Messaging Abstractions (`core/messaging.rs`) - Block 5
- Task 8.2: Storage Abstractions (`core/storage.rs`) - Block 6

---

## Task 8.1: Messaging Abstractions - Detailed Action Plan

### File Specifications

**File:** `airssys-wasm/src/core/messaging.rs`  
**Estimated Lines:** 450-500 lines  
**Pattern Reference:** Phase 7 actor.rs (432 lines)

**Dependencies:**
- Phase 1: `ComponentId` from `core/component.rs`
- Phase 4: `WasmResult`, `WasmError` from `core/error.rs`
- External: `chrono::DateTime<Utc>` (§3.2), `serde`, `async_trait`

**Key Integration Points:**
- MessageBroker from airssys-rt (211ns routing performance)
- Multicodec serialization (ADR-WASM-001)
- Actor-based message passing (push delivery, no polling)
- Three patterns: Fire-and-Forget, Request-Response, Manual Request-Response

**Knowledge Base References:**
- KNOWLEDGE-WASM-005: Messaging Architecture (1985 lines)
- ADR-WASM-009: Component Communication Model
- ADR-WASM-001: Multicodec Compatibility Strategy

---

### Step 1: File Header and Module Documentation

**Action:** Create comprehensive module-level documentation following Phase 7 pattern

**Template Structure:**
```rust
//! Inter-component messaging abstractions for WASM components.
//!
//! These types define the message envelope, routing strategies, delivery guarantees,
//! and message type classifications needed for Block 5 (Inter-Component Communication).
//! They provide the foundation for actor-based message passing via airssys-rt 
//! MessageBroker integration, following YAGNI principles (§6.1).
//!
//! # Design Rationale
//!
//! - **MessageEnvelope**: Uses ComponentId for routing, multicodec for serialization
//!   format (ADR-WASM-001), chrono::DateTime<Utc> per §3.2 for timestamps.
//!   Contains all metadata needed for three messaging patterns.
//! - **MessageType**: Covers fire-and-forget, request-response, and pub-sub patterns.
//! - **RoutingStrategy**: Trait contract for MessageBroker integration (airssys-rt).
//! - **DeliveryGuarantee**: Three semantics levels (at-most-once, at-least-once, 
//!   exactly-once future). ExactlyOnce marked for future implementation.
//!
//! All types are Clone + Serialize/Deserialize for message passing and persistence.
//! No internal dependencies beyond core (zero circular deps).
//!
//! # Architecture
//!
//! Messages flow through airssys-rt MessageBroker with push-based delivery:
//! 1. Sender calls send_message() or send_request() host function
//! 2. Host validates capabilities and creates MessageEnvelope
//! 3. MessageBroker routes message (~211ns routing time)
//! 4. Receiver's handle_message() export is invoked (push delivery)
//!
//! Performance target: <300ns overhead per message (ADR-WASM-009)
//!
//! # References
//!
//! - ADR-WASM-009: Component Communication Model (messaging architecture)
//! - ADR-WASM-001: Multicodec Compatibility Strategy (serialization)
//! - KNOWLEDGE-WASM-005: Messaging Architecture (implementation patterns)
//! - airssys-rt: MessageBroker and routing performance (RT-TASK-008)
```

**Standards Compliance Checklist:**
- ✅ Comprehensive module doc with architecture overview
- ✅ Design rationale for each type (MessageEnvelope, MessageType, RoutingStrategy, DeliveryGuarantee)
- ✅ References to ADRs and knowledge docs
- ✅ Performance targets documented (<300ns per message)
- ✅ YAGNI principles stated (§6.1)
- ✅ Architecture flow diagram (text-based)

---

### Step 2: Import Organization (§2.1 Compliance)

**Action:** Organize imports in mandatory 3-layer structure

**Template:**
```rust
// Layer 1: Standard library
// (None needed for this module)

// Layer 2: External crates
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

// Layer 3: Internal (core only)
use crate::core::component::ComponentId;
use crate::core::error::WasmResult;
```

**Standards Compliance:**
- ✅ §2.1: 3-layer import organization (std → external → internal)
- ✅ §3.2: `chrono::DateTime<Utc>` for timestamps (NEVER `std::time::SystemTime`)
- ✅ Zero internal dependencies beyond core module
- ✅ Blank lines between layers

---

### Step 3: MessageEnvelope Implementation

**Action:** Implement comprehensive message envelope with all 8 required fields

**Type Specification:**
```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MessageEnvelope {
    pub message_type: MessageType,
    pub from: ComponentId,
    pub to: ComponentId,
    pub topic: Option<String>,
    pub payload: Vec<u8>,
    pub codec: u64,
    pub message_id: String,
    pub correlation_id: Option<String>,
    pub timestamp: DateTime<Utc>,
}
```

**Required Methods:**

1. **Constructor: `new()`**
   - All required fields as parameters
   - Sets `timestamp` to `Utc::now()`
   - Sets optional fields to `None`

2. **Builder Methods:**
   - `with_topic(topic: impl Into<String>) -> Self`
   - `with_correlation_id(correlation_id: impl Into<String>) -> Self`

3. **Type Classification Helpers:**
   - `is_fire_and_forget(&self) -> bool`
   - `is_request(&self) -> bool`
   - `is_response(&self) -> bool`
   - `is_publish(&self) -> bool`

4. **Response Helper:**
   - `reply_to(&self, payload: Vec<u8>, codec: u64, message_id: String) -> Self`
   - Swaps `from`/`to`, sets `correlation_id` to original `message_id`

**Documentation Requirements:**
- ✅ Struct-level rustdoc with:
  - Purpose and usage overview
  - Message flow explanation (4 steps)
  - Serialization format (multicodec explanation)
  - Complete example with all fields
  - References to KNOWLEDGE-WASM-005, ADR-WASM-009, ADR-WASM-001
- ✅ Method-level rustdoc for each method with:
  - Purpose description
  - Parameters explanation
  - Returns description
  - Example usage (executable doc test)

**Standards Compliance:**
- ✅ All 8 fields with proper types
- ✅ §3.2: `DateTime<Utc>` for timestamp field
- ✅ Serde support (`Serialize`, `Deserialize` derives)
- ✅ Builder pattern for optional fields
- ✅ Helper methods for ergonomic API
- ✅ 100% rustdoc coverage with examples

---

### Step 4: MessageType Enum Implementation

**Action:** Implement message pattern classification enum

**Type Specification:**
```rust
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum MessageType {
    FireAndForget,
    Request,
    Response,
    Publish,
}
```

**Required Methods:**

1. **Semantic Helpers:**
   - `expects_response(&self) -> bool` - Returns `true` for `Request` only
   - `is_one_way(&self) -> bool` - Returns `true` for `FireAndForget` and `Publish`
   - `description(&self) -> &'static str` - Human-readable description

**Documentation Requirements:**
- ✅ Enum-level rustdoc with:
  - Pattern descriptions (all 4 variants)
  - Performance characteristics (~280ns fire-and-forget, ~560ns request-response)
  - Use case guidance
  - References to KNOWLEDGE-WASM-005 §4, ADR-WASM-009
- ✅ Variant-level rustdoc for each variant
- ✅ Method-level rustdoc with examples

**Standards Compliance:**
- ✅ 4 variants covering all messaging patterns
- ✅ Comprehensive derives (Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)
- ✅ Helper methods for pattern matching
- ✅ Performance documentation from RT-TASK-008 benchmarks

---

### Step 5: RoutingStrategy Trait Implementation

**Action:** Implement routing contract trait for MessageBroker integration

**Type Specification:**
```rust
pub trait RoutingStrategy: Send + Sync {
    fn route(&self, envelope: &MessageEnvelope) -> WasmResult<()>;
}
```

**Documentation Requirements:**
- ✅ Trait-level rustdoc with:
  - Architecture overview (4 steps: validate, resolve, deliver, track)
  - Implementation list (DirectRoutingStrategy, TopicRoutingStrategy, etc.)
  - Performance requirements (<211ns per message from RT-TASK-008)
  - Integration with airssys-rt MessageBroker
  - Example implementation
  - References to KNOWLEDGE-WASM-005, ADR-WASM-009, RT-TASK-008
- ✅ Method-level rustdoc with:
  - Contract description (what implementations must do)
  - Parameters and returns
  - Performance target
  - Example usage

**Standards Compliance:**
- ✅ Single method trait (clear contract)
- ✅ `Send + Sync` bounds for async/actor usage
- ✅ Integration point documented for airssys-rt MessageBroker
- ✅ Performance target from benchmarks (<211ns routing)

---

### Step 6: DeliveryGuarantee Enum Implementation

**Action:** Implement delivery semantics enum

**Type Specification:**
```rust
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum DeliveryGuarantee {
    AtMostOnce,
    AtLeastOnce,
    #[cfg(feature = "exactly-once-delivery")]
    ExactlyOnce,
}
```

**Required Methods:**

1. **Semantic Helpers:**
   - `may_lose_messages(&self) -> bool`
   - `may_duplicate(&self) -> bool`
   - `is_exactly_once(&self) -> bool`
   - `expected_latency_ns(&self) -> u64`
   - `description(&self) -> &'static str`

**Documentation Requirements:**
- ✅ Enum-level rustdoc with:
  - Semantics explanation (all 3 levels)
  - Implementation status (Phase 1: AtMostOnce, AtLeastOnce; Phase 2+: ExactlyOnce)
  - YAGNI justification for ExactlyOnce deferral
  - References to KNOWLEDGE-WASM-005 §8, ADR-WASM-009
- ✅ Variant-level rustdoc with performance characteristics
- ✅ Method-level rustdoc with examples

**Standards Compliance:**
- ✅ 3 semantics levels documented
- ✅ ExactlyOnce marked as future feature with `cfg` gate
- ✅ YAGNI compliance (§6.1) - ExactlyOnce deferred to Phase 2+
- ✅ Performance data from RT-TASK-008 (280ns, 560ns)

---

### Step 7: Unit Tests

**Action:** Implement comprehensive unit tests targeting >90% coverage

**Test Categories:**

1. **MessageEnvelope Tests:**
   - `test_message_envelope_creation()` - Basic construction
   - `test_message_envelope_with_topic()` - Builder pattern
   - `test_message_envelope_with_correlation_id()` - Builder pattern
   - `test_message_type_classification()` - is_* helpers
   - `test_reply_to()` - Response creation
   - `test_message_envelope_serialization()` - Serde roundtrip

2. **MessageType Tests:**
   - `test_message_type_expects_response()` - Semantic helper
   - `test_message_type_is_one_way()` - Semantic helper

3. **DeliveryGuarantee Tests:**
   - `test_delivery_guarantee_may_lose()` - Semantic helper
   - `test_delivery_guarantee_may_duplicate()` - Semantic helper
   - `test_delivery_guarantee_expected_latency()` - Performance data

4. **RoutingStrategy Tests:**
   - `test_routing_strategy_trait_object()` - Trait implementation

**Test Standards:**
- ✅ Each test has clear name describing what it tests
- ✅ Tests use assert macros with descriptive messages
- ✅ Serde serialization/deserialization tested
- ✅ Helper methods validated
- ✅ Target: >90% code coverage

---

### Task 8.1 Success Criteria

**Pre-Implementation Checklist:**
- [ ] Read KNOWLEDGE-WASM-005 completely (messaging architecture)
- [ ] Review ADR-WASM-009 (component communication model)
- [ ] Understand RT-TASK-008 benchmarks (MessageBroker performance)
- [ ] Review Phase 7 actor.rs implementation pattern

**Implementation Checklist:**
- [ ] File header and module documentation complete
- [ ] Imports organized per §2.1 (3-layer structure)
- [ ] MessageEnvelope implemented with 8 fields
- [ ] MessageType enum with 4 variants
- [ ] RoutingStrategy trait defined
- [ ] DeliveryGuarantee enum with 3 levels
- [ ] All helper methods implemented
- [ ] Unit tests written and passing

**Quality Assurance Checklist:**
- [ ] `cargo check` passes with zero warnings
- [ ] `cargo clippy` passes with zero warnings
- [ ] `cargo test` passes all tests
- [ ] `cargo doc` generates complete documentation
- [ ] Test coverage >90% (verify with coverage tool)
- [ ] All rustdoc examples executable (doc tests pass)
- [ ] Standards compliance verified (§2.1, §3.2, §6.1)
- [ ] Integration validated with Phase 3 Capability types
- [ ] ADR-WASM-009 compliance confirmed

**Integration Validation:**
- [ ] MessageEnvelope uses ComponentId from Phase 1
- [ ] Errors use WasmResult from Phase 4
- [ ] Timestamps use chrono::DateTime<Utc> per §3.2
- [ ] Ready for Phase 9 lifecycle abstractions

---

## Task 8.2: Storage Abstractions - Detailed Action Plan

### File Specifications

**File:** `airssys-wasm/src/core/storage.rs`  
**Estimated Lines:** 450-500 lines  
**Pattern Reference:** Phase 7 security.rs (444 lines)

**Dependencies:**
- Phase 1: `ComponentId` from `core/component.rs` (for namespace isolation)
- Phase 4: `WasmResult`, `WasmError` from `core/error.rs`
- External: `async_trait`, `serde`

**Key Integration Points:**
- NEAR-style KV API (simple, intuitive)
- Sled backend (default, pure Rust)
- RocksDB backend (optional, production)
- Namespace isolation via key prefixing (`component:<id>:key`)

**Knowledge Base References:**
- KNOWLEDGE-WASM-007: Component Storage Architecture (1909 lines)
- KNOWLEDGE-WASM-008: Storage Backend Comparison
- ADR-WASM-007: Storage Backend Selection

---

### Step 1: File Header and Module Documentation

**Action:** Create comprehensive module-level documentation following Phase 7 pattern

**Template Structure:**
```rust
//! Storage backend abstractions for WASM component persistence.
//!
//! These types define the storage backend trait, operation types, transaction
//! abstraction, and namespace isolation needed for Block 6 (Component Storage).
//! They provide a NEAR-style key-value API foundation without backend-specific
//! implementation details, following YAGNI principles (§6.1).
//!
//! # Design Rationale
//!
//! - **StorageBackend**: Async trait for pluggable storage implementations (Sled,
//!   RocksDB, custom). Uses namespace parameter for component isolation via key
//!   prefixing (`component:<id>:key`). Generic over backend for zero-cost abstraction.
//!
//! - **StorageOperation**: Enum representing storage operations for transaction
//!   batching and audit logging. Contains all data needed for execution.
//!
//! - **StorageTransaction**: Async trait for atomic multi-operation transactions.
//!   Uses Box<dyn> pattern (§6.2 exception) for heap-allocated transaction state.
//!
//! All types are async-first (async_trait) for non-blocking I/O and integration
//! with tokio runtime (airssys-rt foundation).
//!
//! # Architecture
//!
//! Storage flow follows NEAR-style KV pattern:
//! 1. Component calls storage host function (get/set/delete/scan_prefix)
//! 2. Host runtime validates capabilities and quotas
//! 3. Storage Manager prefixes key with component ID for isolation
//! 4. Backend implementation handles actual I/O (Sled/RocksDB)
//!
//! Performance targets: <1ms per operation (ADR-WASM-007)
//!
//! # Backend Selection
//!
//! - **Sled (Default)**: Pure Rust, zero C++ dependencies, fast compilation
//! - **RocksDB (Optional)**: Battle-tested, production stability, C++ dependency
//! - **Custom**: Implement StorageBackend trait for specialized needs
//!
//! See KNOWLEDGE-WASM-008 for comprehensive backend comparison.
//!
//! # Namespace Isolation
//!
//! Components are isolated via key prefixing:
//! - Component A stores "config" → "component:a:config"
//! - Component B stores "config" → "component:b:config"
//! - No cross-component access possible
//!
//! # References
//!
//! - ADR-WASM-007: Storage Backend Selection (Sled/RocksDB decision)
//! - KNOWLEDGE-WASM-007: Component Storage Architecture (NEAR-style API)
//! - KNOWLEDGE-WASM-008: Storage Backend Comparison (Sled vs RocksDB)
```

**Standards Compliance Checklist:**
- ✅ Comprehensive module doc with architecture overview
- ✅ Design rationale for each type (StorageBackend, StorageOperation, StorageTransaction)
- ✅ References to ADRs and knowledge docs
- ✅ Backend selection guidance
- ✅ Namespace isolation explanation with examples
- ✅ Performance targets documented (<1ms per operation)
- ✅ YAGNI principles stated (§6.1)

---

### Step 2: Import Organization (§2.1 Compliance)

**Action:** Organize imports in mandatory 3-layer structure

**Template:**
```rust
// Layer 1: Standard library
// (None needed for this module)

// Layer 2: External crates
use async_trait::async_trait;
use serde::{Deserialize, Serialize};

// Layer 3: Internal (core only)
use crate::core::error::WasmResult;
```

**Standards Compliance:**
- ✅ §2.1: 3-layer import organization (std → external → internal)
- ✅ async_trait for async methods in traits
- ✅ Zero internal dependencies beyond core module
- ✅ Blank lines between layers

---

### Step 3: StorageBackend Trait Implementation

**Action:** Implement async storage backend trait with 5 required methods

**Type Specification:**
```rust
#[async_trait]
pub trait StorageBackend: Send + Sync {
    async fn get(&self, namespace: &str, key: &[u8]) -> WasmResult<Option<Vec<u8>>>;
    async fn set(&self, namespace: &str, key: &[u8], value: &[u8]) -> WasmResult<()>;
    async fn delete(&self, namespace: &str, key: &[u8]) -> WasmResult<()>;
    async fn list_keys(&self, namespace: &str, prefix: &[u8]) -> WasmResult<Vec<Vec<u8>>>;
    async fn begin_transaction(&self) -> WasmResult<Box<dyn StorageTransaction>>;
}
```

**Method Specifications:**

1. **`get(namespace, key)`**
   - Returns `Ok(Some(value))` if key exists
   - Returns `Ok(None)` if key not found
   - Returns `Err(WasmError)` on I/O error
   - Performance target: <1ms for values up to 1MB

2. **`set(namespace, key, value)`**
   - Overwrites existing value if key exists
   - Creates key if not found
   - Returns `Ok(())` on success
   - Returns `Err(WasmError)` on I/O error or quota exceeded

3. **`delete(namespace, key)`**
   - Deletes key from namespace
   - No-op if key doesn't exist (not an error)
   - Returns `Ok(())` on success
   - Returns `Err(WasmError)` on I/O error

4. **`list_keys(namespace, prefix)`**
   - Returns all keys starting with prefix
   - Empty prefix returns all keys in namespace
   - Returns `Ok(keys)` (may be empty vector)
   - Performance target: <10ms for up to 1000 keys

5. **`begin_transaction()`**
   - Creates new transaction for atomic operations
   - Returns `Ok(Box<dyn StorageTransaction>)`
   - Returns `Err(WasmError)` if transactions not supported

**Documentation Requirements:**
- ✅ Trait-level rustdoc with:
  - NEAR-style API explanation
  - Namespace isolation mechanism
  - Async design rationale
  - Backend implementation list
  - Complete example implementation
  - Performance requirements
  - References to KNOWLEDGE-WASM-007 §5-§6, KNOWLEDGE-WASM-008, ADR-WASM-007
- ✅ Method-level rustdoc for each method with:
  - Purpose and behavior
  - Parameters explanation (namespace, key, value, prefix)
  - Returns description (all variants)
  - Performance targets
  - Example usage

**Standards Compliance:**
- ✅ Async trait with `#[async_trait]` macro
- ✅ `Send + Sync` bounds for async/actor usage
- ✅ 5 methods covering complete KV operations
- ✅ Namespace parameter for component isolation
- ✅ Generic over backend for zero-cost abstraction
- ✅ Performance targets documented (<1ms per op)

---

### Step 4: StorageOperation Enum Implementation

**Action:** Implement storage operation enum for transaction batching

**Type Specification:**
```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum StorageOperation {
    Get { namespace: String, key: Vec<u8> },
    Set { namespace: String, key: Vec<u8>, value: Vec<u8> },
    Delete { namespace: String, key: Vec<u8> },
    List { namespace: String, prefix: Vec<u8> },
}
```

**Required Methods:**

1. **Accessors:**
   - `namespace(&self) -> &str` - Returns namespace for all variants
   - `operation_type(&self) -> &'static str` - Returns "get", "set", "delete", or "list"

**Documentation Requirements:**
- ✅ Enum-level rustdoc with:
  - Purpose (transaction batching, audit logging)
  - Variant descriptions (all 4 operations)
  - Usage patterns
  - Example creation for each variant
  - References to StorageBackend trait
- ✅ Variant-level rustdoc for each variant
- ✅ Method-level rustdoc with examples

**Standards Compliance:**
- ✅ 4 variants covering all operation types
- ✅ Each variant contains all data needed for execution
- ✅ Serialize/Deserialize for persistence and audit
- ✅ Helper methods for operation introspection

---

### Step 5: StorageTransaction Trait Implementation

**Action:** Implement transaction abstraction trait

**Type Specification:**
```rust
#[async_trait]
pub trait StorageTransaction: Send + Sync {
    async fn add_operation(&mut self, op: StorageOperation) -> WasmResult<()>;
    async fn commit(self: Box<Self>) -> WasmResult<()>;
    async fn rollback(self: Box<Self>) -> WasmResult<()>;
}
```

**Method Specifications:**

1. **`add_operation(op)`**
   - Adds operation to transaction batch
   - Does not execute immediately
   - Returns `Ok(())` on success
   - Returns `Err(WasmError)` if operation invalid

2. **`commit(self)`**
   - Executes all operations atomically
   - Takes `Box<Self>` to consume transaction
   - Returns `Ok(())` if all operations succeed
   - Returns `Err(WasmError)` if any operation fails (rolls back all)

3. **`rollback(self)`**
   - Discards all pending operations
   - Takes `Box<Self>` to consume transaction
   - Returns `Ok(())` always
   - Use for explicit abort

**Documentation Requirements:**
- ✅ Trait-level rustdoc with:
  - Transaction semantics (atomicity, isolation)
  - Box<dyn> pattern rationale (§6.2 exception for heap state)
  - Usage pattern (add → commit/rollback)
  - Example transaction workflow
  - References to StorageBackend::begin_transaction()
- ✅ Method-level rustdoc for each method with:
  - Behavior and guarantees
  - Parameters and returns
  - Example usage

**Standards Compliance:**
- ✅ Async trait with `#[async_trait]` macro
- ✅ `Send + Sync` bounds for async usage
- ✅ Box<Self> consumption for commit/rollback (prevent reuse)
- ✅ §6.2 exception justified (heap-allocated transaction state)
- ✅ 3 methods for complete transaction lifecycle

---

### Step 6: Unit Tests

**Action:** Implement comprehensive unit tests targeting >90% coverage

**Test Categories:**

1. **StorageOperation Tests:**
   - `test_storage_operation_get_creation()` - Get variant construction
   - `test_storage_operation_set_creation()` - Set variant construction
   - `test_storage_operation_delete_creation()` - Delete variant construction
   - `test_storage_operation_list_creation()` - List variant construction
   - `test_storage_operation_namespace()` - Accessor method
   - `test_storage_operation_type()` - Operation type identification
   - `test_storage_operation_serialization()` - Serde roundtrip

2. **StorageBackend Tests:**
   - `test_storage_backend_trait_object()` - Trait implementation
   - `test_storage_backend_async_methods()` - Async method signatures

3. **StorageTransaction Tests:**
   - `test_storage_transaction_trait_object()` - Trait implementation
   - `test_storage_transaction_lifecycle()` - Add → commit/rollback pattern

**Mock Implementation for Testing:**
```rust
struct MockBackend;

#[async_trait]
impl StorageBackend for MockBackend {
    async fn get(&self, _ns: &str, _key: &[u8]) -> WasmResult<Option<Vec<u8>>> {
        Ok(None)
    }
    // ... other methods
}
```

**Test Standards:**
- ✅ Each test has clear name describing what it tests
- ✅ Tests use assert macros with descriptive messages
- ✅ Async tests use `#[tokio::test]` or equivalent
- ✅ Serde serialization/deserialization tested
- ✅ Helper methods validated
- ✅ Target: >90% code coverage

---

### Task 8.2 Success Criteria

**Pre-Implementation Checklist:**
- [ ] Read KNOWLEDGE-WASM-007 completely (storage architecture)
- [ ] Review KNOWLEDGE-WASM-008 (backend comparison)
- [ ] Review ADR-WASM-007 (storage backend selection)
- [ ] Review Phase 7 security.rs implementation pattern

**Implementation Checklist:**
- [ ] File header and module documentation complete
- [ ] Imports organized per §2.1 (3-layer structure)
- [ ] StorageBackend trait with 5 async methods
- [ ] StorageOperation enum with 4 variants
- [ ] StorageTransaction trait with 3 methods
- [ ] All helper methods implemented
- [ ] Unit tests written and passing

**Quality Assurance Checklist:**
- [ ] `cargo check` passes with zero warnings
- [ ] `cargo clippy` passes with zero warnings
- [ ] `cargo test` passes all tests
- [ ] `cargo doc` generates complete documentation
- [ ] Test coverage >90% (verify with coverage tool)
- [ ] All rustdoc examples executable (doc tests pass)
- [ ] Standards compliance verified (§2.1, §6.1, §6.2)
- [ ] Integration validated with Phase 4 error types
- [ ] ADR-WASM-007 compliance confirmed

**Integration Validation:**
- [ ] Errors use WasmResult from Phase 4
- [ ] Async trait properly configured
- [ ] Namespace isolation pattern documented
- [ ] Ready for Phase 9 lifecycle abstractions

---

## Phase 8 Final Integration

### Module Updates Required

**File:** `airssys-wasm/src/core/mod.rs`

**Action:** Add module declarations for Phase 8

**Changes:**
```rust
// Domain-Specific Abstractions (Phase 6+)
pub mod interface;
pub mod runtime;
pub mod actor;
pub mod security;
pub mod messaging;  // Phase 8.1: NEW
pub mod storage;    // Phase 8.2: NEW

// Re-exports for public API
pub use messaging::{MessageEnvelope, MessageType, RoutingStrategy, DeliveryGuarantee};
pub use storage::{StorageBackend, StorageOperation, StorageTransaction};
```

---

### Memory Bank Updates Required

**Files to Update:**

1. **`task_000_core_abstractions_design.md`**
   - Mark Phase 8 tasks 8.1 and 8.2 as complete
   - Update completion percentage to 83%

2. **`progress.md`**
   - Update phase completion status
   - Add Phase 8 completion entry
   - Update next steps to Phase 9

3. **`current_context.md`**
   - Update active context if Phase 8 completes WASM-TASK-000
   - Update completion percentage

---

## Standards Compliance Summary

### Workspace Standards (Mandatory)

**§2.1: 3-Layer Import Organization**
- ✅ Layer 1: Standard library imports
- ✅ Layer 2: Third-party crate imports
- ✅ Layer 3: Internal module imports
- ✅ Blank lines between layers

**§3.2: chrono DateTime<Utc> Standard**
- ✅ MessageEnvelope.timestamp uses `chrono::DateTime<Utc>`
- ❌ NEVER use `std::time::SystemTime` for business logic

**§4.3: Module Architecture**
- ✅ mod.rs files: ONLY module declarations and re-exports
- ✅ No implementation code in mod.rs
- ✅ Clear module boundaries

**§6.1: YAGNI Principles**
- ✅ Build only what's needed for Phase 8
- ✅ DeliveryGuarantee::ExactlyOnce deferred to Phase 2+ with feature gate
- ✅ No speculative features

**§6.2: Avoid `dyn` Patterns**
- ✅ Prefer trait bounds over trait objects
- ✅ Exception: StorageTransaction uses `Box<dyn>` for heap-allocated state (justified)

### Microsoft Rust Guidelines Integration

**M-DI-HIERARCHY: Type Hierarchy**
- ✅ Concrete types preferred
- ✅ Generics for StorageBackend (zero-cost abstraction)
- ✅ `dyn` only where necessary (StorageTransaction)

**M-ESSENTIAL-FN-INHERENT: Core Functionality**
- ✅ Helper methods as inherent methods (is_*, with_*, reply_to)
- ✅ Trait methods for backend contracts only

**M-MOCKABLE-SYSCALLS: Testability**
- ✅ Traits enable mock implementations for testing
- ✅ All I/O behind trait abstractions

---

## Performance Targets Summary

### Messaging Performance (from RT-TASK-008)
- Fire-and-Forget: ~280ns end-to-end
- Request-Response: ~560ns round-trip
- MessageBroker routing: ~211ns
- Target overhead: <300ns per message

### Storage Performance (from ADR-WASM-007)
- get/set/delete: <1ms for values up to 1MB
- list_keys: <10ms for up to 1000 keys
- Sled/RocksDB both meet targets

---

## References

### Primary Knowledge Documents
- KNOWLEDGE-WASM-005: Messaging Architecture (1985 lines)
- KNOWLEDGE-WASM-007: Component Storage Architecture (1909 lines)
- KNOWLEDGE-WASM-008: Storage Backend Comparison

### Architecture Decision Records
- ADR-WASM-001: Multicodec Compatibility Strategy
- ADR-WASM-007: Storage Backend Selection
- ADR-WASM-009: Component Communication Model

### Related Tasks
- RT-TASK-008: MessageBroker Performance Benchmarks
- WASM-TASK-000: Core Abstractions Design (parent task)

### Workspace Standards
- `.memory-bank/workspace/shared_patterns.md` (§2.1-§6.1)
- `.memory-bank/workspace/microsoft_rust_guidelines.md`
- `.memory-bank/workspace/documentation_terminology_standards.md`

---

## Implementation Timeline

**Estimated Time:** 1-2 days (Days 17-19 of WASM-TASK-000)

**Day 17:**
- Morning: Task 8.1 implementation (messaging.rs)
- Afternoon: Task 8.1 tests and documentation review

**Day 18:**
- Morning: Task 8.2 implementation (storage.rs)
- Afternoon: Task 8.2 tests and documentation review

**Day 19:**
- Morning: Module integration (mod.rs updates)
- Afternoon: Quality assurance, memory bank updates, validation

**Milestone:** Phase 8 complete → 83% progress (10/12 phases)

---

**Next Phase:** Phase 9 - Lifecycle & Management Abstractions (Days 20-22)
