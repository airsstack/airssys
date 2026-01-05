# KNOWLEDGE-WASM-030: Module Architecture - Hard Requirements

> ⚠️ **SUPERSEDED FOR REBUILD**
> 
> For the **clean-slate rebuild**, refer to [KNOWLEDGE-WASM-037: Rebuild Architecture - Clean Slate Design](knowledge-wasm-037-rebuild-architecture-clean-slate.md).
> 
> KNOWLEDGE-WASM-037 provides the updated architecture with:
> - Six modules: `core/`, `security/`, `runtime/`, `component/`, `messaging/`, `system/`
> - Layer-organized `core/` with abstractions per module
> - Proper Dependency Inversion patterns
> 
> This document remains as historical reference for the four-module architecture.

**Document ID:** KNOWLEDGE-WASM-030  
**Created:** 2025-12-22  
**Updated:** 2026-01-05  
**Status:** 🔴 **HISTORICAL** (see KNOWLEDGE-WASM-037 for rebuild)  
**Category:** Architecture / Module Design / Enforcement  
**Related ADR:** ADR-WASM-023 (Module Boundary Enforcement)

---

## Purpose

This document defines the **MANDATORY** module architecture for airssys-wasm. These are **HARD REQUIREMENTS** that **MUST BE FOLLOWED** without exception.

This document exists because:
1. Repeated architectural violations have caused significant development delays
2. Planning has proceeded without verifying module boundaries
3. The module separation was designed for specific reasons that were being ignored

---

## The Four Root Modules

### Overview

```
airssys-wasm/src/
├── core/      # Foundation - shared types and abstractions
├── security/  # Security - capabilities, policies, validation
├── runtime/   # Execution - WASM engine and loading
└── actor/     # Integration - actor system and messaging
```

---

## Module 1: `core/`

### Purpose
**Core data types and abstractions shared by ALL other modules.**

### Why It Exists
Core is the foundation layer. It contains types that multiple modules need. By placing shared types here, we prevent circular dependencies.

**Example Problem Without core/:**
- `runtime/` needs `ComponentMessage` to handle messages
- `ComponentMessage` is defined in `actor/`
- But `actor/` needs `runtime/` for WASM execution
- CIRCULAR DEPENDENCY

**Solution:**
- `ComponentMessage` goes in `core/`
- Both `runtime/` and `actor/` import from `core/`
- No cycle

### What Belongs Here

| Category | Examples |
|----------|----------|
| **Identity Types** | `ComponentId`, `CorrelationId` |
| **Message Types** | `ComponentMessage`, `RequestMessage`, `ResponseMessage` |
| **Error Types** | `WasmError`, `RequestError` |
| **Config Types** | `ComponentConfig`, `ResourceLimits` |
| **Trait Abstractions** | `RuntimeEngine`, `SecurityValidator` |
| **Common Structs** | `PendingRequest`, `ComponentHandle` |

### What Does NOT Belong Here

| Category | Why Not | Where It Goes |
|----------|---------|---------------|
| WASM execution logic | Implementation, not abstraction | `runtime/` |
| Actor system logic | Implementation, not abstraction | `actor/` |
| Security validation logic | Implementation, not abstraction | `security/` |
| External crate dependencies | Core should be dependency-minimal | Other modules |

### Dependency Rule

```
core/ imports from: NOTHING (only std)
```

**Verification:**
```bash
# MUST return nothing
grep -r "use crate::" src/core/
```

---

## Module 2: `security/`

### Purpose
**Security-related types and logic: capabilities, policies, validation.**

### Why It Exists
Security must be independent so it can be applied uniformly across all other layers. Security should not depend on how WASM is executed or how actors work.

### What Belongs Here

| Category | Examples |
|----------|----------|
| **Capability Types** | `Capability`, `CapabilitySet`, `CapabilityGrant` |
| **Policy Types** | `SecurityPolicy`, `AccessPolicy` |
| **Validation Logic** | `CapabilityValidator`, `PermissionChecker` |
| **ACL Integration** | airssys-osl ACL bridge |
| **Audit Types** | `SecurityAuditEvent`, `AuditLogger` |

### What Does NOT Belong Here

| Category | Why Not | Where It Goes |
|----------|---------|---------------|
| WASM execution | Not security-specific | `runtime/` |
| Message routing | Not security-specific | `actor/` |
| Actor lifecycle | Not security-specific | `actor/` |

### Dependency Rule

```
security/ imports from: core/ ONLY
```

**Verification:**
```bash
# MUST return nothing
grep -r "use crate::runtime" src/security/
grep -r "use crate::actor" src/security/
```

---

## Module 3: `runtime/`

### Purpose
**WASM execution engine. Implements `core::runtime` abstractions using Wasmtime.**

### Why It Exists
Runtime is responsible for ONE thing: executing WASM code. It provides the engine that loads and runs WebAssembly components. It does NOT orchestrate communication between components.

### What Belongs Here

| Category | Examples |
|----------|----------|
| **WASM Engine** | `WasmEngine`, Wasmtime integration |
| **Component Loading** | `ComponentLoader`, WASM binary loading |
| **Store Management** | `StoreManager`, `StoreWrapper` |
| **Host Functions** | `SendMessageHostFunction`, `SendRequestHostFunction` (the struct definitions) |
| **Resource Limits** | `ResourceLimiter`, memory/CPU limits |
| **WASM Execution** | `call_handle_message()`, `call_handle_callback()` |

### What Does NOT Belong Here

| Category | Why Not | Where It Goes |
|----------|---------|---------------|
| Message routing | Actor system concern | `actor/message/` |
| Correlation tracking | Actor system concern | `actor/message/` |
| Response routing | Actor system concern | `actor/message/` |
| Timeout handling | Actor system concern | `actor/message/` |
| Mailbox management | Actor system concern | `actor/message/` |
| Subscription management | Actor system concern | `actor/message/` |
| Pending request tracking | Actor system concern | `actor/message/` |
| MessagingService | Orchestration, not execution | `actor/message/` |
| ResponseRouter | Orchestration, not execution | `actor/message/` |

### The Key Distinction

**Runtime EXECUTES.** It does not ORCHESTRATE.

- ✅ `runtime/` executes `handle-message` export on a WASM component
- ❌ `runtime/` does NOT decide when or why to execute it
- ❌ `runtime/` does NOT track who is waiting for responses
- ❌ `runtime/` does NOT route messages between components

**Actor ORCHESTRATES.** It calls runtime to execute.

- ✅ `actor/` receives message from broker
- ✅ `actor/` decides which component should handle it
- ✅ `actor/` calls `runtime/` to execute the WASM
- ✅ `actor/` handles the response routing

### Dependency Rule

```
runtime/ imports from: core/, security/ ONLY
```

**Verification:**
```bash
# MUST return nothing
grep -r "use crate::actor" src/runtime/
```

---

## Module 4: `actor/`

### Purpose
**Actor system integration for WASM components. Handles messaging, lifecycle, supervision.**

### Why It Exists
Actor integrates WASM components with the airssys-rt actor system. It orchestrates communication, manages component lifecycle, and coordinates supervision.

### What Belongs Here

| Category | Examples |
|----------|----------|
| **Component Actor** | `ComponentActor`, `ActorState` |
| **Component Registry** | `ComponentRegistry`, component lookup |
| **Component Spawning** | `ComponentSpawner`, actor creation |
| **Message Routing** | `ActorSystemSubscriber`, `MessageRouter` |
| **Correlation Tracking** | `CorrelationTracker`, pending requests |
| **Response Routing** | `ResponseRouter`, callback invocation |
| **Timeout Handling** | `TimeoutHandler`, request timeouts |
| **Messaging Service** | `MessagingService`, broker integration |
| **Supervision** | `ComponentSupervisor`, restart logic |
| **Health Monitoring** | `HealthMonitor`, health checks |
| **Subscription Management** | `SubscriberManager`, topic subscriptions |

### Dependency Rule

```
actor/ imports from: core/, security/, runtime/
```

Actor is at the TOP of the dependency chain. It can import from all other modules.

---

## The Dependency Diagram

```
                    ┌─────────┐
                    │  actor/ │  ← TOP (orchestrates everything)
                    └────┬────┘
                         │
           ┌─────────────┼─────────────┐
           │             │             │
           ▼             ▼             ▼
      ┌─────────┐   ┌─────────┐   ┌─────────┐
      │runtime/ │   │security/│   │  core/  │
      └────┬────┘   └────┬────┘   └─────────┘
           │             │             ▲
           │             └─────────────┤
           │                           │
           └───────────────────────────┘
           
ARROWS SHOW ALLOWED IMPORT DIRECTION
```

### Allowed Imports

| From | Can Import |
|------|------------|
| `core/` | Nothing (only std) |
| `security/` | `core/` |
| `runtime/` | `core/`, `security/` |
| `actor/` | `core/`, `security/`, `runtime/` |

### Forbidden Imports

| From | CANNOT Import | Why |
|------|---------------|-----|
| `runtime/` | `actor/` | Creates cycle, violates architecture |
| `security/` | `runtime/`, `actor/` | Security must be independent |
| `core/` | Any module | Core is foundation, no dependencies |

---

## Decision Flow: Where Does This Code Belong?

```
START: I need to add new code
         │
         ▼
    ┌────────────────────────────────────┐
    │ Is this a DATA TYPE that multiple  │
    │ modules need to share?             │
    └────────────────┬───────────────────┘
                     │
         ┌───────────┴───────────┐
         │                       │
        YES                      NO
         │                       │
         ▼                       ▼
    ┌─────────┐         ┌────────────────────────────┐
    │  core/  │         │ Is this SECURITY logic?    │
    └─────────┘         │ (capabilities, policies,   │
                        │  validation, ACL)          │
                        └────────────┬───────────────┘
                                     │
                         ┌───────────┴───────────┐
                         │                       │
                        YES                      NO
                         │                       │
                         ▼                       ▼
                    ┌──────────┐     ┌────────────────────────────┐
                    │ security/│     │ Is this WASM EXECUTION?    │
                    └──────────┘     │ (engine, loading, stores,  │
                                     │  host function execution)  │
                                     └────────────┬───────────────┘
                                                  │
                                      ┌───────────┴───────────┐
                                      │                       │
                                     YES                      NO
                                      │                       │
                                      ▼                       ▼
                                 ┌─────────┐             ┌─────────┐
                                 │ runtime/│             │  actor/ │
                                 └─────────┘             └─────────┘
```

---

## Common Mistakes and Corrections

### Mistake 1: Putting Messaging Infrastructure in runtime/

**WRONG:**
```
runtime/messaging.rs
  └── MessagingService
  └── ResponseRouter
  └── imports CorrelationTracker from actor/
```

**CORRECT:**
```
actor/message/messaging_service.rs
  └── MessagingService
  
actor/message/response_router.rs
  └── ResponseRouter
  
core/messaging.rs
  └── CorrelationId, ResponseMessage, RequestError (data types)
```

**Why:** Messaging orchestration is an actor concern, not a WASM execution concern.

---

### Mistake 2: Host Functions Importing Actor Types

**WRONG:**
```rust
// runtime/async_host.rs
use crate::actor::message::{CorrelationTracker, PendingRequest};
```

**CORRECT:**
```rust
// runtime/async_host.rs
use crate::core::{CorrelationId, PendingRequest};  // Types from core

// Host function returns correlation ID, actor/ handles the tracking
```

**Why:** Host functions execute WASM. They should not know about actor system internals.

---

### Mistake 3: Response Loop in runtime/

**WRONG:**
```rust
// runtime/async_host.rs
tokio::spawn(async move {
    // Wait for response and invoke callback
    if let Ok(response) = response_rx.await {
        engine.call_handle_callback(...).await;
    }
});
```

**CORRECT:**
```rust
// actor/message/response_handler.rs
tokio::spawn(async move {
    // Wait for response and invoke callback
    if let Ok(response) = response_rx.await {
        engine.call_handle_callback(...).await;  // actor/ calls INTO runtime/
    }
});
```

**Why:** Response handling is orchestration. Actor orchestrates, runtime executes.

---

## Verification Commands

**Run these BEFORE any code is committed:**

```bash
# Check 1: runtime/ must NOT import from actor/
echo "Checking runtime/ -> actor/ violations..."
VIOLATIONS=$(grep -r "use crate::actor" src/runtime/ 2>/dev/null)
if [ -n "$VIOLATIONS" ]; then
    echo "❌ VIOLATION: runtime/ imports from actor/"
    echo "$VIOLATIONS"
    exit 1
fi
echo "✅ runtime/ is clean"

# Check 2: security/ must NOT import from runtime/ or actor/
echo "Checking security/ -> runtime/actor/ violations..."
VIOLATIONS=$(grep -r "use crate::runtime\|use crate::actor" src/security/ 2>/dev/null)
if [ -n "$VIOLATIONS" ]; then
    echo "❌ VIOLATION: security/ imports from runtime/ or actor/"
    echo "$VIOLATIONS"
    exit 1
fi
echo "✅ security/ is clean"

# Check 3: core/ must NOT import from any internal module
echo "Checking core/ -> internal module violations..."
VIOLATIONS=$(grep -r "use crate::" src/core/ 2>/dev/null)
if [ -n "$VIOLATIONS" ]; then
    echo "❌ VIOLATION: core/ imports from internal modules"
    echo "$VIOLATIONS"
    exit 1
fi
echo "✅ core/ is clean"

echo ""
echo "✅ All module boundary checks passed!"
```

---

## Summary Table

| Module | Purpose | Imports From | Never Imports From |
|--------|---------|--------------|-------------------|
| `core/` | Shared types & abstractions | Nothing | Everything |
| `security/` | Security logic | `core/` | `runtime/`, `actor/` |
| `runtime/` | WASM execution | `core/`, `security/` | `actor/` |
| `actor/` | Actor integration | `core/`, `security/`, `runtime/` | N/A (top layer) |

---

## Enforcement

### This Document Is A Hard Requirement

- **Planners MUST verify module boundaries before proposing code locations**
- **Implementers MUST run verification commands before committing**
- **Reviewers MUST reject code that violates these rules**
- **No exceptions. No shortcuts. No "we'll fix it later."**

### Consequences of Violation

1. Code will be rejected
2. Rework will be required
3. Development time will be lost
4. This has already happened twice - DO NOT let it happen again

---

## References

- **ADR-WASM-023**: Module Boundary Enforcement (the decision)
- **ADR-WASM-018**: Three-Layer Architecture
- **ADR-WASM-022**: Circular Dependency Remediation
- **KNOWLEDGE-WASM-028**: Circular Dependency Documentation

---

**Created:** 2025-12-22  
**Author:** Architecture Team  
**Status:** 🔴 MANDATORY - HARD REQUIREMENTS

