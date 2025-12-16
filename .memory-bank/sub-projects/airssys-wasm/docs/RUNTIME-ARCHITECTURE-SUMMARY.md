# Runtime Architecture Summary

**Created:** 2025-12-16  
**Purpose:** Quick reference for runtime dependency decisions  
**Audience:** Developers working on airssys-wasm

---

## TL;DR

**Three runtime layers, each with specific responsibilities:**

```text
┌─────────────────────────────────────────┐
│ Layer 2: airssys-wasm                  │
│ - WASM-specific features                │
│ - Correlation tracking                  │
│ - Component registry                    │
│ - Permission enforcement                │
└─────────────────────────────────────────┘
        ↓ uses                  ↓ uses
┌──────────────────┐    ┌──────────────────┐
│ Layer 3:         │    │ Layer 0:         │
│ airssys-rt       │    │ Tokio            │
│ - Message routing│    │ - Async tasks    │
│ - Actor lifecycle│    │ - Timers         │
│ - Supervision    │    │ - Channels       │
└──────────────────┘    └──────────────────┘
```

**Golden Rules:**
1. ✅ Use **Tokio directly** for async primitives
2. ✅ Use **airssys-rt indirectly** for message routing
3. ✅ Implement **WASM-specific** features in Layer 2
4. ❌ Don't wrap Tokio (no value added)
5. ❌ Don't pollute airssys-rt with WASM logic

---

## Quick Decision Guide

### "Should I use Tokio or airssys-rt?"

| Your Need | Use This | Example |
|-----------|----------|---------|
| Spawn async task | Tokio | `tokio::spawn(async { ... })` |
| Set timeout | Tokio | `tokio::time::sleep(duration).await` |
| Single response | Tokio | `tokio::sync::oneshot::channel()` |
| Message stream | Tokio | `tokio::sync::mpsc::channel()` |
| Route message | airssys-rt | `self.publish_message(topic, msg)` |
| Spawn actor | airssys-rt | Via ComponentSpawner |
| Supervise actor | airssys-rt | Via SupervisorNode |
| WASM feature | Layer 2 | Implement in airssys-wasm |

### "Where should I implement this feature?"

```
Is it WASM-specific?
    ├─ YES → Implement in airssys-wasm (Layer 2)
    └─ NO → Does airssys-rt already provide it?
           ├─ YES → Use airssys-rt (via Phase 4 bridge)
           └─ NO → Should it be generic?
                  ├─ YES → Consider adding to airssys-rt
                  └─ NO → Implement in airssys-wasm
```

---

## Common Patterns

### ✅ Correct: Timeout Handling

```rust
use tokio::time::{sleep, Duration};

pub async fn with_timeout<T>(
    operation: impl Future<Output = T>,
) -> Result<T, TimeoutError> {
    tokio::time::timeout(Duration::from_secs(5), operation)
        .await
        .map_err(|_| TimeoutError)
}
```

### ✅ Correct: Single Response Channel

```rust
use tokio::sync::oneshot;

pub async fn send_request(&self) -> Result<Response, Error> {
    let (tx, rx) = oneshot::channel();
    
    // Register pending request
    self.tracker.register(tx)?;
    
    // Wait for response
    rx.await.map_err(|_| Error::Cancelled)
}
```

### ✅ Correct: Message Routing (Indirect)

```rust
// Use Phase 4 bridge, not direct airssys-rt import
pub async fn send_to_component(&self, msg: Message) {
    self.publish_message("topic", &msg).await?;
    // Internally delegates to MessageBroker (Layer 3)
}
```

### ❌ Incorrect: Wrapping Tokio

```rust
// DON'T DO THIS - Adds no value
pub struct OurChannel<T> {
    inner: tokio::sync::oneshot::Sender<T>,
}

// Just use Tokio directly!
use tokio::sync::oneshot;
```

### ❌ Incorrect: Direct airssys-rt Import

```rust
// DON'T DO THIS in Layer 2 business logic
use airssys_rt::broker::MessageBroker;

// Use Phase 4 bridge instead:
self.publish_message(topic, msg).await?;
```

---

## Reference Documents

### Primary Documentation

1. **ADR-WASM-019: Runtime Dependency Management**
   - **Location:** `docs/adr/adr-wasm-019-runtime-dependency-management.md`
   - **Purpose:** Official architectural decision
   - **Read:** When you need to justify design decisions

2. **KNOWLEDGE-WASM-019: Runtime Dependency Architecture**
   - **Location:** `docs/knowledges/knowledge-wasm-019-runtime-dependency-architecture.md`
   - **Purpose:** Comprehensive analysis and examples
   - **Read:** When implementing new features

3. **KNOWLEDGE-WASM-016: Actor System Integration Implementation Guide**
   - **Location:** `docs/knowledges/knowledge-wasm-016-actor-system-integration-implementation-guide.md`
   - **Purpose:** Step-by-step implementation guide
   - **Read:** When integrating with actor system

### Related ADRs

- **ADR-WASM-018:** Three-Layer Architecture (layer separation)
- **ADR-WASM-009:** Component Communication Model (messaging patterns)
- **ADR-WASM-001:** Multicodec Compatibility (payload encoding)

---

## Verification Checklist

### Before Submitting Code

**Tokio Usage:**
- [ ] Used Tokio directly for async primitives
- [ ] No unnecessary wrappers
- [ ] Proper error handling (timeout, channel closed)

**airssys-rt Usage:**
- [ ] Message routing via Phase 4 bridge (not direct)
- [ ] Actor lifecycle via ComponentSpawner
- [ ] No direct airssys-rt imports in Layer 2 logic

**Layer Boundaries:**
- [ ] WASM-specific logic in airssys-wasm (Layer 2)
- [ ] Generic logic delegated to airssys-rt (Layer 3)
- [ ] No circular dependencies

**Performance:**
- [ ] No abstraction overhead in hot paths
- [ ] Lock-free data structures (DashMap, not RwLock<HashMap>)
- [ ] Efficient async patterns (no blocking)

---

## FAQ

### Q: Why not wrap Tokio for consistency?

**A:** Tokio IS the consistency. It's the industry-standard async runtime. Wrapping it adds overhead without benefit. Other major Rust projects (actix-web, tower, tonic) also use Tokio directly.

### Q: Should correlation tracking be in airssys-rt?

**A:** No. Correlation tracking is WASM-specific (component request-response). airssys-rt should remain generic and reusable for any actor system.

### Q: Can I import airssys-rt directly in ComponentActor?

**A:** For Phase 4 bridge components (MessageBrokerBridge, UnifiedRouter), yes. For Layer 2 business logic (CorrelationTracker, ComponentActor methods), no - use the bridge.

### Q: What if Tokio changes its API?

**A:** Tokio follows semantic versioning. Breaking changes are rare and announced well in advance. Migration would be mechanical (search/replace imports). Abstraction layers wouldn't help.

### Q: How do I test code that uses Tokio?

**A:** Use tokio::time::pause() for deterministic time tests. Use test doubles for channels. Tokio provides excellent testing support.

---

## Contact

**Questions about runtime architecture?**
- Check: ADR-WASM-019 (official decision)
- Check: KNOWLEDGE-WASM-019 (detailed analysis)
- Ask: Architecture team (via Memory Bank)

**Found a violation?**
- File: Create ADR-WASM-019 violation ticket
- Fix: Refactor to follow layer boundaries
- Update: Add example to this document

---

**Document Version:** 1.0  
**Last Updated:** 2025-12-16  
**Status:** Active (quick reference)
