# KNOWLEDGE-RT-008: Mailbox Metrics Refactoring Plan

**Knowledge Type**: Implementation Plan  
**Category**: Mailbox System  
**Created**: 2025-10-05  
**Related**: RT-TASK-003, KNOWLEDGE-RT-006  
**Status**: Planning - Not Yet Implemented  
**Complexity**: Intermediate  

---

## Overview

This document outlines the complete plan for refactoring the mailbox metrics system from a concrete `MailboxMetrics` struct to a trait-based design with proper encapsulation and dependency injection.

## Motivation

### Current Problems

1. **Poor Encapsulation**: Direct public field access (`metrics.messages_sent.load(Ordering::Relaxed)`)
2. **No Abstraction**: Locked into atomic implementation, can't swap to async/remote exporters
3. **Tight Coupling**: Metrics implementation details exposed to all consumers
4. **Module Organization**: Metrics mixed with core traits in `traits.rs`

### Design Goals

1. ✅ **Encapsulation**: Hide implementation details behind trait methods
2. ✅ **Flexibility**: Support different metrics implementations (atomic, async, no-op)
3. ✅ **YAGNI Compliance**: Keep metrics in `mailbox/` module (tightly coupled)
4. ✅ **Zero-cost abstractions**: Generic constraints, no `dyn Trait` (§6.2)
5. ✅ **Dependency Injection**: Inject metrics at construction time
6. ✅ **Future-proof**: Easy to add new implementations without breaking changes

---

## Architecture Design

### Directory Structure

#### Before
```
src/mailbox/
├── mod.rs
├── traits.rs           # Contains MailboxMetrics struct ❌
├── bounded.rs
├── unbounded.rs
└── backpressure.rs
```

#### After
```
src/mailbox/
├── mod.rs              # Updated: add metrics submodule
├── traits.rs           # Updated: remove MailboxMetrics, keep MailboxReceiver/Sender
├── bounded.rs          # Updated: use R: MetricsRecorder generic
├── unbounded.rs        # Updated: use R: MetricsRecorder generic
├── backpressure.rs     # No changes needed
└── metrics/            # NEW: Metrics subsystem
    ├── mod.rs          # NEW: Module declarations + re-exports
    ├── recorder.rs     # NEW: MetricsRecorder trait
    └── atomic.rs       # NEW: AtomicMetrics implementation
```

### Key Design Decisions

#### 1. Metrics Inside `mailbox/` Module (YAGNI §6.1)

**Decision**: Keep metrics in `mailbox/metrics/` instead of top-level `metrics/`

**Rationale**:
- Current metrics are **mailbox-specific** (`messages_sent`, `messages_received`)
- No other components currently need these metrics
- YAGNI: Don't create abstractions for non-existent use cases
- If actor/broker metrics needed later, create `actor/metrics/`, `broker/metrics/` separately

**Future Evolution**:
```
src/actor/metrics/      # Actor-specific metrics (when needed)
src/broker/metrics/     # Broker-specific metrics (when needed)
src/mailbox/metrics/    # Mailbox-specific metrics (current)
```

#### 2. Trait-Based Design Without Default Type Parameter

**Decision**: Use `R: MetricsRecorder` without default type parameter

**User's Insight**:
> "I think it will be better if we just using `MetricsRecorder` as trait parameter without need to assign it to default implementation which is `AtomicMetric`, but later when an object initiate, we can inject it with `AtomicMetric`"

**Correct Approach**:
```rust
// No default type parameter - cleaner, more explicit
pub struct BoundedMailbox<M: Message, R: MetricsRecorder> {
    receiver: mpsc::Receiver<MessageEnvelope<M>>,
    metrics: Arc<R>,
    capacity: usize,
}

// Default constructor provides AtomicMetrics
impl<M: Message> BoundedMailbox<M, AtomicMetrics> {
    pub fn new(capacity: usize) -> (Self, BoundedMailboxSender<M, AtomicMetrics>) {
        let metrics = Arc::new(AtomicMetrics::default());
        Self::with_metrics(capacity, metrics)
    }
}

// Generic constructor for custom metrics
impl<M: Message, R: MetricsRecorder> BoundedMailbox<M, R> {
    pub fn with_metrics(capacity: usize, metrics: Arc<R>) -> (Self, BoundedMailboxSender<M, R>) {
        // ... implementation
    }
}
```

**Benefits**:
- ✅ **Explicitness**: Type is clear `BoundedMailbox<M, AtomicMetrics>`
- ✅ **Dependency Injection**: Metrics injected at construction (testable, flexible)
- ✅ **No hidden defaults**: Type signature doesn't hide implementation
- ✅ **Better for evolution**: No cascading default parameters

**Rejected Approach (with default)**:
```rust
// ❌ Rejected: Default type parameter hides implementation
pub struct BoundedMailbox<M: Message, R: MetricsRecorder = AtomicMetrics> {
    // ...
}
```

#### 3. Generic Constraints (No `dyn Trait`)

**Decision**: Use generic constraints `R: MetricsRecorder`, not `Arc<dyn MetricsRecorder>`

**Compliance**: §6.2 Avoid `dyn` Patterns, Microsoft Guidelines M-DI-HIERARCHY

**Benefits**:
- ✅ Static dispatch (zero-cost abstraction)
- ✅ Compile-time monomorphization
- ✅ No vtable overhead
- ✅ Better optimization opportunities

---

## Complete Implementation Plan

### Phase 1: Create Metrics Module (New Files)

#### File 1: `src/mailbox/metrics/mod.rs` (~40 lines)

**Purpose**: Module root with re-exports

```rust
//! Mailbox metrics recording and tracking subsystem.
//!
//! Provides trait-based metrics recording for mailbox operations with
//! pluggable implementations.

mod recorder;
mod atomic;

pub use recorder::MetricsRecorder;
pub use atomic::AtomicMetrics;
```

**Key Points**:
- Re-exports public API
- No implementation code (§4.3 compliance)

---

#### File 2: `src/mailbox/metrics/recorder.rs` (~80 lines)

**Purpose**: Define `MetricsRecorder` trait interface

```rust
//! Metrics recorder trait for mailbox operations.

use chrono::{DateTime, Utc}; // §3.2 MANDATORY

/// Trait for recording mailbox metrics.
///
/// Abstracts metrics recording mechanism, allowing different implementations
/// (atomic counters, async channels, remote exporters) without changing mailbox code.
pub trait MetricsRecorder: Send + Sync {
    /// Record a message send operation
    fn record_sent(&self);

    /// Record a message receive operation
    fn record_received(&self);

    /// Record a dropped message (backpressure or TTL expiration)
    fn record_dropped(&self);

    /// Update the timestamp of the last processed message
    fn update_last_message(&self, timestamp: DateTime<Utc>);

    /// Get total number of messages sent
    fn sent_count(&self) -> u64;

    /// Get total number of messages received
    fn received_count(&self) -> u64;

    /// Get total number of messages dropped
    fn dropped_count(&self) -> u64;

    /// Get timestamp of last processed message
    fn last_message_at(&self) -> Option<DateTime<Utc>>;

    /// Get number of messages currently in-flight (sent but not received)
    fn in_flight(&self) -> u64 {
        self.sent_count().saturating_sub(self.received_count())
    }
}
```

**Design Principles**:
- **Send + Sync**: Thread-safe sharing
- **Method-based**: No public fields exposed
- **Default implementation**: `in_flight()` has default implementation
- **Generic constraints**: Used as `R: MetricsRecorder`, not `dyn`

---

#### File 3: `src/mailbox/metrics/atomic.rs` (~180 lines + 8 tests)

**Purpose**: Default atomic-based metrics implementation

```rust
//! Atomic-based metrics implementation.

use std::sync::atomic::{AtomicU64, Ordering};
use chrono::{DateTime, Utc};
use parking_lot::RwLock;
use super::MetricsRecorder;

/// Lock-free atomic metrics recorder.
///
/// Default implementation optimized for low overhead (10-30ns per operation).
#[derive(Debug, Default)]
pub struct AtomicMetrics {
    messages_sent: AtomicU64,
    messages_received: AtomicU64,
    messages_dropped: AtomicU64,
    last_message_at: RwLock<Option<DateTime<Utc>>>,
}

impl AtomicMetrics {
    pub fn new() -> Self {
        Self::default()
    }
}

impl MetricsRecorder for AtomicMetrics {
    fn record_sent(&self) {
        self.messages_sent.fetch_add(1, Ordering::Relaxed);
    }

    fn record_received(&self) {
        self.messages_received.fetch_add(1, Ordering::Relaxed);
    }

    fn record_dropped(&self) {
        self.messages_dropped.fetch_add(1, Ordering::Relaxed);
    }

    fn update_last_message(&self, timestamp: DateTime<Utc>) {
        *self.last_message_at.write() = Some(timestamp);
    }

    fn sent_count(&self) -> u64 {
        self.messages_sent.load(Ordering::Relaxed)
    }

    fn received_count(&self) -> u64 {
        self.messages_received.load(Ordering::Relaxed)
    }

    fn dropped_count(&self) -> u64 {
        self.messages_dropped.load(Ordering::Relaxed)
    }

    fn last_message_at(&self) -> Option<DateTime<Utc>> {
        *self.last_message_at.read()
    }
}
```

**Performance Characteristics**:
- Counter operations: ~10-30ns (lock-free atomic)
- Timestamp updates: ~50-100ns (RwLock write)
- No allocations, no blocking (except timestamp write lock)

**Tests** (8 comprehensive tests):
1. `test_atomic_metrics_default` - Zero initialization
2. `test_record_sent` - Send counter increments
3. `test_record_received` - Receive counter increments
4. `test_record_dropped` - Drop counter increments
5. `test_update_last_message` - Timestamp updates
6. `test_in_flight` - Calculated in-flight messages
7. `test_in_flight_saturating` - Edge case handling
8. `test_concurrent_operations` - Thread-safety validation

---

### Phase 2: Update Existing Files

#### File 4: `src/mailbox/mod.rs` (5 lines changed)

**Changes**:
```rust
// ADD: Metrics submodule
pub mod metrics;

// REMOVE from re-exports:
// pub use traits::MailboxMetrics;

// ADD to re-exports:
pub use metrics::{MetricsRecorder, AtomicMetrics};
```

---

#### File 5: `src/mailbox/traits.rs` (~90 lines removed)

**Remove**:
- `MailboxMetrics` struct definition
- Tests: `test_mailbox_metrics_default`, `test_mailbox_metrics_update`, `test_mailbox_metrics_last_message`

**Keep**:
- `MailboxReceiver<M>` trait
- `MailboxSender<M>` trait
- `MailboxCapacity` enum
- `MailboxError` enum
- `TryRecvError` enum
- All existing tests for above types

**No changes to trait signatures** - traits stay generic over `M: Message` only

---

#### File 6: `src/mailbox/bounded.rs` (~50 changes)

**Struct Changes**:
```rust
// BEFORE
pub struct BoundedMailbox<M: Message> {
    receiver: mpsc::Receiver<MessageEnvelope<M>>,
    metrics: Arc<MailboxMetrics>,  // ❌
    capacity: usize,
}

pub struct BoundedMailboxSender<M: Message> {
    sender: mpsc::Sender<MessageEnvelope<M>>,
    metrics: Arc<MailboxMetrics>,  // ❌
    backpressure: BackpressureStrategy,
}

// AFTER
pub struct BoundedMailbox<M: Message, R: MetricsRecorder> {
    receiver: mpsc::Receiver<MessageEnvelope<M>>,
    metrics: Arc<R>,  // ✅
    capacity: usize,
}

pub struct BoundedMailboxSender<M: Message, R: MetricsRecorder> {
    sender: mpsc::Sender<MessageEnvelope<M>>,
    metrics: Arc<R>,  // ✅
    backpressure: BackpressureStrategy,
}
```

**Constructor Changes**:
```rust
// Default constructor with AtomicMetrics
impl<M: Message> BoundedMailbox<M, AtomicMetrics> {
    pub fn new(capacity: usize) -> (Self, BoundedMailboxSender<M, AtomicMetrics>) {
        let metrics = Arc::new(AtomicMetrics::default());
        Self::with_metrics(capacity, metrics)
    }

    pub fn with_backpressure(
        capacity: usize,
        strategy: BackpressureStrategy,
    ) -> (Self, BoundedMailboxSender<M, AtomicMetrics>) {
        let metrics = Arc::new(AtomicMetrics::default());
        Self::with_metrics_and_backpressure(capacity, metrics, strategy)
    }
}

// Generic constructor for custom metrics
impl<M: Message, R: MetricsRecorder> BoundedMailbox<M, R> {
    pub fn with_metrics(
        capacity: usize,
        metrics: Arc<R>,
    ) -> (Self, BoundedMailboxSender<M, R>) {
        Self::with_metrics_and_backpressure(capacity, metrics, BackpressureStrategy::default())
    }

    pub fn with_metrics_and_backpressure(
        capacity: usize,
        metrics: Arc<R>,
        strategy: BackpressureStrategy,
    ) -> (Self, BoundedMailboxSender<M, R>) {
        let (sender, receiver) = mpsc::channel(capacity);
        
        let mailbox = Self {
            receiver,
            metrics: Arc::clone(&metrics),
            capacity,
        };
        
        let sender = BoundedMailboxSender {
            sender,
            metrics,
            backpressure: strategy,
        };
        
        (mailbox, sender)
    }
}
```

**Usage Changes** (throughout file):
```rust
// BEFORE: Direct field access
self.metrics.messages_sent.fetch_add(1, Ordering::Relaxed);
self.metrics.messages_received.fetch_add(1, Ordering::Relaxed);
self.metrics.messages_dropped.fetch_add(1, Ordering::Relaxed);
*self.metrics.last_message_at.write() = Some(Utc::now());
let count = self.metrics.messages_sent.load(Ordering::Relaxed);

// AFTER: Method calls
self.metrics.record_sent();
self.metrics.record_received();
self.metrics.record_dropped();
self.metrics.update_last_message(Utc::now());
let count = self.metrics.sent_count();
```

**Trait Implementation Changes**:
```rust
#[async_trait]
impl<M: Message, R: MetricsRecorder> MailboxReceiver<M> for BoundedMailbox<M, R> {
    // Add R: MetricsRecorder to all impls
    // Use metrics.record_*() methods
}

#[async_trait]
impl<M: Message, R: MetricsRecorder> MailboxSender<M> for BoundedMailboxSender<M, R> {
    // Add R: MetricsRecorder to all impls
    // Use metrics.record_*() methods
}

impl<M: Message, R: MetricsRecorder> Clone for BoundedMailboxSender<M, R> {
    // Add R: MetricsRecorder constraint
}
```

**Test Changes** (~30 tests):
- Type inference works: `BoundedMailbox::<TestMessage>::new(100)`
- Or explicit: `BoundedMailbox::<TestMessage, AtomicMetrics>::new(100)`
- Use `metrics.sent_count()` instead of direct access

---

#### File 7: `src/mailbox/unbounded.rs` (~40 changes)

**Same pattern as `bounded.rs`**:

**Struct Changes**:
```rust
pub struct UnboundedMailbox<M: Message, R: MetricsRecorder> {
    receiver: mpsc::UnboundedReceiver<MessageEnvelope<M>>,
    metrics: Arc<R>,
}

pub struct UnboundedMailboxSender<M: Message, R: MetricsRecorder> {
    sender: mpsc::UnboundedSender<MessageEnvelope<M>>,
    metrics: Arc<R>,
}
```

**Constructor Changes**:
```rust
impl<M: Message> UnboundedMailbox<M, AtomicMetrics> {
    pub fn new() -> (Self, UnboundedMailboxSender<M, AtomicMetrics>) {
        let metrics = Arc::new(AtomicMetrics::default());
        Self::with_metrics(metrics)
    }
}

impl<M: Message, R: MetricsRecorder> UnboundedMailbox<M, R> {
    pub fn with_metrics(metrics: Arc<R>) -> (Self, UnboundedMailboxSender<M, R>) {
        let (sender, receiver) = mpsc::unbounded_channel();
        
        let mailbox = Self {
            receiver,
            metrics: Arc::clone(&metrics),
        };
        
        let sender = UnboundedMailboxSender {
            sender,
            metrics,
        };
        
        (mailbox, sender)
    }
}
```

**Same usage and test changes as `bounded.rs`**

---

## Migration Guide

### User Code Impact

#### Before (Current)
```rust
use airssys_rt::mailbox::{BoundedMailbox, MailboxMetrics};
use std::sync::atomic::Ordering;

let (mailbox, sender) = BoundedMailbox::<MyMessage>::new(100);

// Direct field access (will break)
let count = mailbox.metrics().messages_sent.load(Ordering::Relaxed);
```

#### After (Refactored)
```rust
use airssys_rt::mailbox::{BoundedMailbox, MetricsRecorder};

// Constructor API unchanged!
let (mailbox, sender) = BoundedMailbox::<MyMessage>::new(100);

// Use methods instead
let count = mailbox.metrics().sent_count();
```

### Breaking Changes

**API Breaks**:
- ❌ Direct field access: `metrics.messages_sent.load(Ordering::Relaxed)`
- ❌ Import path: `use airssys_rt::mailbox::MailboxMetrics;`

**Non-Breaking**:
- ✅ Constructor API: `BoundedMailbox::new(100)` stays same
- ✅ Type inference: No need to specify `AtomicMetrics`
- ✅ Basic usage patterns unchanged

### Custom Metrics Example

```rust
use airssys_rt::mailbox::{BoundedMailbox, MetricsRecorder};
use std::sync::Arc;

// Define custom metrics
struct MyCustomMetrics {
    // Custom implementation (e.g., Prometheus exporter)
}

impl MetricsRecorder for MyCustomMetrics {
    fn record_sent(&self) {
        // Custom logic (e.g., increment Prometheus counter)
    }
    // ... implement all required methods
}

// Use custom metrics
let metrics = Arc::new(MyCustomMetrics::new());
let (mailbox, sender) = BoundedMailbox::with_metrics(100, metrics);
```

---

## Future Implementations

### Possible Future Metrics Implementations

#### 1. No-Op Metrics (Production Performance)
```rust
pub struct NoOpMetrics;

impl MetricsRecorder for NoOpMetrics {
    fn record_sent(&self) {}  // No-op
    fn record_received(&self) {}  // No-op
    fn sent_count(&self) -> u64 { 0 }
    // ... all no-op
}
```

#### 2. Async Metrics (Fire-and-Forget)
```rust
pub struct AsyncMetrics {
    tx: mpsc::UnboundedSender<MetricEvent>,
}

impl MetricsRecorder for AsyncMetrics {
    fn record_sent(&self) {
        let _ = self.tx.send(MetricEvent::Sent);  // Fire-and-forget
    }
    // Background task aggregates events
}
```

#### 3. Prometheus Metrics
```rust
pub struct PrometheusMetrics {
    counter_sent: prometheus::Counter,
    counter_received: prometheus::Counter,
    gauge_in_flight: prometheus::Gauge,
}

impl MetricsRecorder for PrometheusMetrics {
    fn record_sent(&self) {
        self.counter_sent.inc();
        self.gauge_in_flight.inc();
    }
    // Export to Prometheus registry
}
```

---

## Implementation Checklist

### Phase 1: Create Metrics Module
- [ ] Create directory: `src/mailbox/metrics/`
- [ ] Create file: `src/mailbox/metrics/recorder.rs` (trait definition)
- [ ] Create file: `src/mailbox/metrics/atomic.rs` (implementation + 8 tests)
- [ ] Create file: `src/mailbox/metrics/mod.rs` (module root)

### Phase 2: Update Module Structure
- [ ] Modify: `src/mailbox/mod.rs` (add metrics module, update exports)
- [ ] Modify: `src/mailbox/traits.rs` (remove MailboxMetrics, remove 3 tests)

### Phase 3: Update Mailbox Implementations
- [ ] Modify: `src/mailbox/bounded.rs`
  - [ ] Add generic parameter `R: MetricsRecorder` to structs
  - [ ] Update constructor methods (default + generic versions)
  - [ ] Update trait implementations with generics
  - [ ] Replace direct field access with method calls
  - [ ] Update all ~13 tests
- [ ] Modify: `src/mailbox/unbounded.rs`
  - [ ] Add generic parameter `R: MetricsRecorder` to structs
  - [ ] Update constructor methods (default + generic versions)
  - [ ] Update trait implementations with generics
  - [ ] Replace direct field access with method calls
  - [ ] Update all ~13 tests

### Phase 4: Validation
- [ ] Run tests: `cargo test --package airssys-rt`
- [ ] Run clippy: `cargo clippy --package airssys-rt --all-targets --all-features`
- [ ] Verify test count: 105 → 110 (+5 new tests in atomic.rs)
- [ ] Verify zero warnings

### Phase 5: Documentation
- [ ] Update KNOWLEDGE-RT-006 with metrics refactoring notes
- [ ] Create ADR documenting trait-based metrics decision
- [ ] Update task tracking with refactoring completion

---

## Estimated Impact

### Code Changes
- **Lines Added**: ~300 (new metrics module)
- **Lines Modified**: ~150 (bounded + unbounded updates)
- **Lines Removed**: ~90 (MailboxMetrics from traits)
- **Net Change**: +160 lines

### Test Changes
- **New tests**: 8 in `atomic.rs`
- **Updated tests**: ~55 in `bounded.rs` + `unbounded.rs`
- **Removed tests**: 3 from `traits.rs`
- **Net change**: +5 tests (105 → 110 total)

### Performance Impact
- **Runtime**: Zero impact (same atomic operations)
- **Compilation**: Minimal (generic monomorphization)
- **Binary size**: Negligible increase per metrics implementation used

---

## Compliance Verification

### Workspace Standards
- ✅ **§2.1** 3-layer imports in all new files
- ✅ **§3.2** chrono DateTime<Utc> in recorder trait and atomic implementation
- ✅ **§4.3** Module architecture (metrics as submodule with mod.rs)
- ✅ **§6.1** YAGNI (metrics in mailbox/, not premature top-level abstraction)
- ✅ **§6.2** No dyn traits (generic constraints: `R: MetricsRecorder`)

### Microsoft Rust Guidelines
- ✅ **M-DI-HIERARCHY**: Dependency injection of metrics at construction
- ✅ **M-ESSENTIAL-FN-INHERENT**: Core functionality in trait methods
- ✅ **M-SERVICES-CLONE**: Cheap clone via `Arc<R>`
- ✅ **M-DESIGN-FOR-AI**: Clear trait interface with comprehensive docs

---

## References

- **RT-TASK-003**: Mailbox System Implementation (completed)
- **KNOWLEDGE-RT-006**: Mailbox System Implementation Guide
- **KNOWLEDGE-RT-007**: Backpressure Strategy Behavior and Selection Guide
- **§2.1-§6.3**: Workspace Shared Patterns (`.copilot/memory_bank/workspace/shared_patterns.md`)
- **Microsoft Rust Guidelines**: Complete guidelines in workspace memory bank

---

## Version History

- **v1.0** (2025-10-05): Initial refactoring plan
  - Trait-based metrics design
  - AtomicMetrics as default implementation
  - Dependency injection pattern
  - Complete file-by-file implementation plan
