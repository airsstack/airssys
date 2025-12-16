# ADR-WASM-019: Runtime Dependency Management

**Status:** Accepted  
**Date:** 2025-12-16  
**Context:** Phase 5 Task 5.1 (Correlation Tracking) implementation review  
**Related ADRs:** ADR-WASM-018 (Three-Layer Architecture), ADR-WASM-009 (Component Communication)  
**Supersedes:** None  
**Superseded by:** None

---

## Context

During the implementation of correlation tracking (WASM-TASK-004 Phase 5 Task 5.1), questions arose about the proper runtime dependencies for airssys-wasm features:

1. **Should features use Tokio directly or only through airssys-rt?**
2. **What responsibilities belong to each runtime layer?**
3. **How do we maintain clean separation between layers?**
4. **When is direct Tokio usage appropriate vs when should we use airssys-rt?**

The correlation tracking implementation provides a case study in correct runtime dependency architecture, demonstrating:
- Direct Tokio usage for timeout handling (tokio::spawn, tokio::time::sleep)
- Indirect airssys-rt usage for message routing (via Phase 4 components)
- Clean layer boundaries without violations

This ADR formalizes these architectural patterns as the standard for all future airssys-wasm development.

---

## Decision

**We adopt a multi-layer runtime dependency strategy** where different layers of abstraction are used for different concerns:

### Layer 0: Async Runtime (Tokio) - Direct Usage ✅

**Decision:** airssys-wasm features **SHALL** use Tokio directly for async primitives.

**Scope:**
- Async task spawning (tokio::spawn)
- Timer primitives (tokio::time::sleep, tokio::time::Instant, tokio::time::Duration)
- Channel primitives (tokio::sync::oneshot, tokio::sync::mpsc, tokio::sync::broadcast)
- Synchronization primitives (tokio::sync::Mutex, tokio::sync::RwLock, tokio::sync::Semaphore)
- Task cancellation (tokio::task::JoinHandle::abort)

**Rationale:**
- Tokio is the foundational async runtime (Layer 0)
- Direct usage avoids abstraction overhead (zero-cost principle)
- Tokio is stable (1.0+ for years, semantic versioning)
- airssys-rt also uses Tokio directly (consistent pattern)
- No value in wrapping standard primitives

**Examples:**
```rust
// ✅ CORRECT: Direct Tokio usage
use tokio::sync::oneshot;
use tokio::time::{sleep, Duration};

let (tx, rx) = oneshot::channel();
tokio::spawn(async move {
    sleep(Duration::from_secs(5)).await;
});
```

### Layer 3: Actor Runtime (airssys-rt) - Indirect Usage ✅

**Decision:** airssys-wasm features **SHALL** use airssys-rt for actor infrastructure through established integration patterns.

**Scope:**
- Message routing (MessageBroker, ActorRegistry)
- Actor lifecycle (ActorSystem, Actor trait, Child trait)
- Supervision (SupervisorNode, SupervisorStrategy)
- Mailbox management (ActorAddress, Mailbox trait)
- Health monitoring (ChildHealth, HealthMonitor)

**Integration Pattern:**
- airssys-wasm features delegate to Layer 3 via Phase 4 bridge components
- MessageBrokerBridge wraps airssys_rt::broker::MessageBroker
- ComponentRegistry uses airssys_rt::util::ActorAddress
- UnifiedRouter delegates to MessageBroker
- **NO** direct airssys-rt imports in Layer 2 business logic

**Rationale:**
- airssys-rt provides proven infrastructure (~212ns routing, ~625ns spawn)
- Avoids reimplementing complex actor system features
- Maintains airssys-rt as generic (WASM-agnostic)
- Phase 4 components provide proper abstraction layer
- Clean separation between Layer 2 (WASM-specific) and Layer 3 (generic)

**Examples:**
```rust
// ✅ CORRECT: Indirect usage through Phase 4
impl ComponentActor {
    pub async fn send_request(&self, ...) {
        // Uses MessageBroker indirectly
        self.publish_message("requests", &msg).await?;
            ↓
        MessageBrokerBridge.publish()  // Phase 4 bridge
            ↓
        airssys_rt::broker::MessageBroker  // Layer 3
    }
}

// ❌ INCORRECT: Direct airssys-rt import in Layer 2 logic
use airssys_rt::broker::MessageBroker;  // WRONG! Skip bridge layer
```

### Layer 2: WASM Component Lifecycle - Feature Implementation ✅

**Decision:** WASM-specific features **SHALL** be implemented in Layer 2 (airssys-wasm), not in Layer 3 (airssys-rt).

**Scope:**
- Request-response patterns (CorrelationTracker, TimeoutHandler)
- Component registry (WASM-aware lookup)
- Permission enforcement (capability-based security)
- WIT interface management
- Component-specific message routing

**Placement Rule:**
- If feature is **WASM-specific** → Implement in airssys-wasm (Layer 2)
- If feature is **generic actor infrastructure** → Implement in airssys-rt (Layer 3)
- If uncertain, prefer Layer 2 (easier to move down than up)

**Rationale:**
- Keeps airssys-rt generic and reusable
- Allows WASM-specific optimizations
- Maintains clean layer boundaries
- Prevents pollution of generic infrastructure

**Examples:**
```rust
// ✅ CORRECT: WASM-specific in Layer 2
airssys-wasm/src/actor/message/correlation_tracker.rs
airssys-wasm/src/actor/component/component_registry.rs

// ❌ INCORRECT: WASM-specific in Layer 3
airssys-rt/src/correlation/tracker.rs  // WRONG! Pollutes generic layer
```

---

## Layer Boundary Rules

### Rule 1: Dependency Direction ✅

**Allowed Dependencies:**
```text
Layer 2 (airssys-wasm)
    ↓ can import
Layer 3 (airssys-rt)
    ↓ can import
Layer 0 (Tokio)
```

**Forbidden Dependencies:**
```text
Layer 3 (airssys-rt)
    ✗ CANNOT import
Layer 2 (airssys-wasm)  // Circular dependency
```

### Rule 2: Direct vs Indirect Usage ✅

**Direct Usage Allowed:**
- Layer 2 → Layer 0 (Tokio primitives) ✅
- Layer 2 → Layer 1 (Wasmtime) ✅
- Layer 3 → Layer 0 (Tokio primitives) ✅

**Indirect Usage Required:**
- Layer 2 → Layer 3 (via bridge components) ✅

**Why?**
- Maintains clean abstraction layers
- Allows Layer 3 to remain generic
- Provides single point of integration (Phase 4)

### Rule 3: Feature Placement ✅

**Decision Matrix:**

| Feature Characteristic | Layer Placement |
|------------------------|-----------------|
| WASM-specific | Layer 2 (airssys-wasm) |
| Generic actor infrastructure | Layer 3 (airssys-rt) |
| Async primitive | Layer 0 (Tokio) |
| WASM execution | Layer 1 (Wasmtime) |

**Examples:**

| Feature | Placement | Rationale |
|---------|-----------|-----------|
| Correlation tracking | Layer 2 ✅ | Component request-response (WASM-specific) |
| Message routing | Layer 3 ✅ | Pub/sub infrastructure (generic) |
| Timeout handling | Layer 0 ✅ | Timer primitives (foundational) |
| Component loading | Layer 1 ✅ | WASM instantiation (Wasmtime) |
| Health checking | Layer 3 ✅ | Actor health (generic, reused from airssys-rt) |
| Permission enforcement | Layer 2 ✅ | Capability security (WASM-specific) |

---

## Consequences

### Positive

1. **Performance** ✅
   - Direct Tokio usage: Zero abstraction overhead
   - <50ns correlation lookup (DashMap lock-free)
   - <5ms timeout accuracy (Tokio timer wheel)
   - No unnecessary wrapper layers

2. **Maintainability** ✅
   - Clear layer boundaries
   - Easy to reason about dependencies
   - Standard patterns (Tokio is industry-standard)
   - Single integration point (Phase 4)

3. **Reusability** ✅
   - airssys-rt remains generic (WASM-agnostic)
   - Tokio is foundational (used everywhere)
   - Layer 2 features don't pollute Layer 3
   - Clean separation of concerns

4. **Flexibility** ✅
   - Direct Tokio access: Full control over async behavior
   - Can use advanced Tokio features (e.g., tokio::task::LocalSet)
   - No constraints from wrapper abstractions
   - Easy to adopt new Tokio features

5. **ADR Compliance** ✅
   - Perfect adherence to ADR-WASM-018 (Three-Layer Architecture)
   - Maintains architecture consistency
   - Clear precedent for future features
   - Validates existing implementation decisions

### Negative

1. **Tokio Dependency** ⚠️
   - Direct dependency on Tokio (but already required by airssys-rt)
   - Breaking changes in Tokio would require updates (rare, semantic versioning)
   - **Mitigation:** Tokio 1.x stable for years, minimal risk

2. **Knowledge Requirement** ⚠️
   - Developers need to understand layer boundaries
   - Must know when to use Tokio vs airssys-rt
   - **Mitigation:** This ADR provides clear guidelines

3. **No Central Abstraction** ⚠️
   - Cannot globally switch async runtime (but no need to)
   - Cannot intercept all async operations (but no use case)
   - **Mitigation:** Tokio is industry standard, no alternative needed

### Neutral

1. **Consistency** ✓
   - Same pattern as airssys-rt (also uses Tokio directly)
   - Idiomatic Rust async (standard practice)
   - **Observation:** This is the norm in Rust async ecosystem

2. **Testing** ✓
   - Can use tokio::time::pause() for deterministic tests
   - Can mock channels with test doubles
   - **Observation:** Tokio provides excellent testing support

---

## Implementation Guidelines

### For Async Primitives (Layer 0)

**When to use Tokio directly:**
- ✅ Need channels (oneshot, mpsc, broadcast)
- ✅ Need timers (sleep, interval, timeout)
- ✅ Need task spawning (tokio::spawn)
- ✅ Need synchronization (Mutex, RwLock, Semaphore)
- ✅ airssys-rt doesn't provide relevant abstraction

**Pattern:**
```rust
use tokio::sync::oneshot;
use tokio::time::{sleep, Duration};

pub async fn with_timeout<T>(
    operation: impl Future<Output = T>,
    timeout: Duration,
) -> Result<T, TimeoutError> {
    match tokio::time::timeout(timeout, operation).await {
        Ok(result) => Ok(result),
        Err(_) => Err(TimeoutError),
    }
}
```

### For Actor Infrastructure (Layer 3)

**When to use airssys-rt (indirectly):**
- ✅ Need message routing (MessageBroker)
- ✅ Need actor lifecycle (ActorSystem)
- ✅ Need supervision (SupervisorNode)
- ✅ Need mailbox management (ActorAddress)
- ✅ Existing airssys-rt abstraction fits

**Pattern:**
```rust
// Don't import airssys-rt directly in business logic
// Use Phase 4 bridge components instead

impl ComponentActor {
    pub async fn send_to_component(&self, msg: Message) {
        // ✅ CORRECT: Use bridge
        self.publish_message("topic", &msg).await?;
        
        // ❌ INCORRECT: Direct airssys-rt usage
        // self.broker.publish(msg).await?;
    }
}
```

### For WASM-Specific Features (Layer 2)

**When to implement in airssys-wasm:**
- ✅ Feature is component-specific (not generic)
- ✅ Feature requires WASM knowledge
- ✅ Feature is communication pattern (request-response)
- ✅ Feature is security enforcement (capabilities)
- ✅ airssys-rt shouldn't know about WASM

**Pattern:**
```rust
// ✅ CORRECT: WASM-specific in Layer 2
// airssys-wasm/src/actor/message/correlation_tracker.rs

pub struct CorrelationTracker {
    // WASM-specific: Request-response for components
    pending: Arc<DashMap<CorrelationId, PendingRequest>>,
    timeout_handler: Arc<TimeoutHandler>,
}
```

---

## Verification Checklist

### For Code Reviews

When reviewing new features, verify:

**Tokio Usage (Layer 0):**
- [ ] Tokio used directly for async primitives
- [ ] No unnecessary wrappers around Tokio types
- [ ] Standard patterns (oneshot for single response, mpsc for streams)
- [ ] Proper error handling (timeout errors, channel closed errors)

**airssys-rt Usage (Layer 3):**
- [ ] Message routing delegates to MessageBroker (via bridge)
- [ ] Actor lifecycle delegates to ActorSystem (via ComponentSpawner)
- [ ] No direct airssys-rt imports in Layer 2 business logic
- [ ] Phase 4 bridge components used correctly

**Layer Boundaries:**
- [ ] WASM-specific logic in airssys-wasm (Layer 2)
- [ ] Generic actor logic in airssys-rt (Layer 3) or delegated
- [ ] No circular dependencies (Layer 3 → Layer 2)
- [ ] Clear separation of concerns

**Performance:**
- [ ] No abstraction overhead in hot paths
- [ ] Lock-free data structures where appropriate (DashMap)
- [ ] Efficient async patterns (no blocking in async code)
- [ ] Minimal allocations (Arc for shared state, not clone)

---

## Examples

### Example 1: Timeout Handling (Correct) ✅

```rust
// CorrelationTracker uses Tokio directly for timeouts
// Location: airssys-wasm/src/actor/message/timeout_handler.rs

use tokio::task::JoinHandle;
use tokio::time::{sleep, Duration};

pub struct TimeoutHandler {
    active_timeouts: Arc<DashMap<CorrelationId, JoinHandle<()>>>,
}

impl TimeoutHandler {
    pub fn register_timeout(
        &self,
        correlation_id: CorrelationId,
        timeout: Duration,
        tracker: CorrelationTracker,
    ) -> JoinHandle<()> {
        let corr_id = correlation_id;
        
        // ✅ CORRECT: Direct Tokio usage
        let handle = tokio::spawn(async move {
            sleep(timeout).await;  // Tokio timer primitive
            
            if let Some(pending) = tracker.remove_pending(&corr_id) {
                let _ = pending.response_tx.send(ResponseMessage {
                    correlation_id: corr_id,
                    result: Err(RequestError::Timeout),
                    // ...
                });
            }
        });
        
        self.active_timeouts.insert(correlation_id, handle);
        handle
    }
}
```

**Why Correct:**
- Uses tokio::spawn directly (no wrapper needed)
- Uses tokio::time::sleep directly (timer primitive)
- Efficient: No abstraction overhead
- Standard: Idiomatic Rust async

### Example 2: Message Routing (Correct) ✅

```rust
// ComponentActor uses airssys-rt indirectly for message routing
// Location: airssys-wasm/src/actor/component/component_actor.rs

impl ComponentActor {
    pub async fn send_request<T: Serialize>(
        &self,
        target: &ComponentId,
        request: T,
        timeout: Duration,
    ) -> Result<oneshot::Receiver<ResponseMessage>, WasmError> {
        // Register with CorrelationTracker
        let (response_tx, response_rx) = oneshot::channel();
        let correlation_id = Uuid::new_v4();
        
        self.correlation_tracker
            .register_pending(PendingRequest { /* ... */ })
            .await?;
        
        // ✅ CORRECT: Indirect airssys-rt usage through Phase 4
        self.publish_message("requests", &request_msg).await?;
        //     ↓
        // MessageBrokerBridge.publish() (Phase 4)
        //     ↓
        // airssys_rt::broker::MessageBroker (Layer 3)
        
        Ok(response_rx)
    }
}
```

**Why Correct:**
- Uses MessageBroker indirectly (via publish_message)
- Phase 4 bridge handles Layer 3 interaction
- Clean layer boundaries
- No direct airssys-rt imports

### Example 3: Feature Placement (Correct) ✅

```rust
// Correlation tracking is WASM-specific, belongs in Layer 2
// Location: airssys-wasm/src/actor/message/correlation_tracker.rs

use tokio::sync::oneshot;  // ✅ Layer 0 import
use crate::core::ComponentId;  // ✅ Layer 2 import

pub struct CorrelationTracker {
    // WASM-specific: Component request-response
    pending: Arc<DashMap<CorrelationId, PendingRequest>>,
}

// ✅ CORRECT: No airssys-rt imports (uses indirectly via Phase 4)
// ❌ INCORRECT: use airssys_rt::broker::MessageBroker;
```

**Why Correct:**
- WASM-specific feature in Layer 2
- Uses Tokio directly (Layer 0)
- Delegates to airssys-rt via Phase 4
- Keeps airssys-rt generic

---

## Migration Path

### If This ADR Conflicts with Existing Code

**Assessment Phase:**
1. Identify violations (direct airssys-rt imports in Layer 2 business logic)
2. Categorize severity (critical path vs edge cases)
3. Estimate refactoring effort

**Refactoring Strategy:**
1. **Phase 1:** New code follows ADR (immediate)
2. **Phase 2:** Refactor critical paths (high-traffic code)
3. **Phase 3:** Refactor remaining violations (gradual)

**Transition Rules:**
- ✅ New features MUST follow this ADR
- ✅ Existing code SHOULD be refactored opportunistically
- ✅ Breaking changes MAY be batched in major versions

### If Tokio 2.0 Breaks Compatibility

**Scenario:** Tokio 2.0 changes API

**Migration Strategy:**
1. Update Cargo.toml: `tokio = "2.0"`
2. Search/replace: Update import paths if changed
3. Compiler errors guide remaining changes
4. Run test suite to verify

**Why This Works:**
- Tokio follows semantic versioning
- Breaking changes announced well in advance
- Migration is mechanical (not architectural)
- No need for abstraction layer (wouldn't help)

---

## Decision Rationale

### Why Not Wrap Tokio?

**Considered Alternative:** Create airssys-rt abstractions for all async primitives

**Rejected Because:**
1. **No value added:** Tokio is already the abstraction
2. **Performance cost:** Adds indirection overhead
3. **Maintenance burden:** Must update wrapper for every Tokio change
4. **Flexibility lost:** Constrains usage to wrapper API
5. **Standard practice:** Other Rust projects use Tokio directly

**Examples of projects using Tokio directly:**
- actix-web (actor framework)
- tower (service framework)
- tonic (gRPC framework)
- hyper (HTTP library)

### Why Keep airssys-rt Generic?

**Considered Alternative:** Add WASM-specific features to airssys-rt

**Rejected Because:**
1. **Reusability:** airssys-rt useful for non-WASM actors
2. **Separation:** WASM concerns belong in airssys-wasm
3. **Maintenance:** Single responsibility principle
4. **Testing:** Generic code easier to test in isolation

**Benefit:**
- airssys-rt can be used independently
- Other projects can use airssys-rt without WASM
- Clear separation of concerns

### Why Indirect airssys-rt Usage?

**Considered Alternative:** Direct airssys-rt imports everywhere

**Rejected Because:**
1. **Layer violations:** Breaks clean architecture
2. **Coupling:** Tight coupling to Layer 3 internals
3. **Flexibility:** Hard to change integration patterns
4. **Testing:** Difficult to mock Layer 3 dependencies

**Benefit:**
- Phase 4 components provide single integration point
- Easy to change Layer 3 implementation
- Clean layer boundaries

---

## References

### Related ADRs
- **ADR-WASM-018:** Three-Layer Architecture (layer separation)
- **ADR-WASM-009:** Component Communication Model (request-response patterns)
- **ADR-WASM-001:** Multicodec Compatibility Strategy (payload encoding)

### Knowledge Documents
- **KNOWLEDGE-WASM-019:** Runtime Dependency Architecture (detailed analysis)
- **KNOWLEDGE-WASM-016:** Actor System Integration Implementation Guide
- **KNOWLEDGE-WASM-018:** Component Definitions and Architecture Layers

### Implementation References
- **WASM-TASK-004 Phase 5 Task 5.1:** Correlation Tracking (reference implementation)
- **Phase 4:** MessageBroker Integration (bridge pattern)
- **Phase 2:** ActorSystem Integration (ComponentSpawner pattern)

### External References
- Tokio Documentation: https://docs.rs/tokio
- Async Book: https://rust-lang.github.io/async-book/
- Zero-Cost Abstractions: https://blog.rust-lang.org/2015/05/11/traits.html

---

## Appendix: Decision Matrix

### Quick Reference Table

| Concern | Use Tokio Directly | Use airssys-rt | Implement in Layer 2 |
|---------|-------------------|----------------|----------------------|
| **Async task spawning** | ✅ Yes | ❌ No | N/A |
| **Timer/timeout** | ✅ Yes | ❌ No | N/A |
| **Channels** | ✅ Yes | ❌ No | N/A |
| **Message routing** | ❌ No | ✅ Yes (indirect) | N/A |
| **Actor lifecycle** | ❌ No | ✅ Yes (indirect) | N/A |
| **Supervision** | ❌ No | ✅ Yes (indirect) | N/A |
| **Request-response** | ❌ No | ❌ No | ✅ Yes |
| **Correlation tracking** | ❌ No | ❌ No | ✅ Yes |
| **Permission enforcement** | ❌ No | ❌ No | ✅ Yes |
| **Component registry** | ❌ No | Partial | ✅ Yes (WASM-aware) |

---

## Status

**Accepted:** 2025-12-16  
**Implemented:** WASM-TASK-004 Phase 5 Task 5.1 (correlation tracking)  
**Validated:** Code review + audit passed (9.5/10 quality)  
**Reference Implementation:** `src/actor/message/correlation_tracker.rs`

**Next Review:** 2026-Q2 (after Layer 2 completion)

---

**ADR Version:** 1.0  
**Last Updated:** 2025-12-16  
**Authors:** Architecture team (memorybank-manager)  
**Reviewers:** rust-reviewer, memorybank-auditor  
**Status:** Active (architectural standard)
