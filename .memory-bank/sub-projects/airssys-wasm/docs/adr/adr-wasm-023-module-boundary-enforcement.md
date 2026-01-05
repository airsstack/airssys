# ADR-WASM-023: Module Boundary Enforcement

**ADR ID:** ADR-WASM-023  
**Created:** 2025-12-22  
**Status:** ACCEPTED  
**Deciders:** Architecture Team  
**Category:** Architecture / Module Design / MANDATORY  
**Severity:** 🔴 **CRITICAL - HARD REQUIREMENT**

---

## Title

Module Boundary Enforcement: Mandatory One-Way Dependencies

---

## Context

### Problem Statement

The airssys-wasm codebase has suffered **REPEATED architectural violations** where modules import from modules they should not depend on. Specifically:

1. **2025-12-21**: Discovered `runtime/` imports `ComponentMessage` from `actor/`
2. **2025-12-22**: Discovered `runtime/messaging.rs` imports `CorrelationTracker`, `PendingRequest`, `ResponseMessage`, `RequestError` from `actor/message/`
3. **2025-12-22**: Discovered `runtime/messaging.rs` itself contains messaging infrastructure that belongs in `actor/`

These violations have caused:
- Development delays (multiple days lost to remediation)
- Broken architecture that required rework
- Plans that would have made the violations WORSE
- Loss of trust in the planning process

### Root Cause

The module structure was designed with clear purposes, but:
1. No enforcement mechanism existed
2. Planning did not verify module boundaries before proposing code locations
3. Implementation proceeded without checking import directions

---

## Decision

### The Four Root Modules and Their Purposes

| Module | Purpose | WHY It Exists |
|--------|---------|---------------|
| `core/` | Core data types and abstractions | Provides shared foundation that ALL modules can depend on. Prevents circular dependencies. |
| `security/` | Security-related types and logic | Isolates security concerns (capabilities, policies, validation). Security must be independent. |
| `runtime/` | WASM execution engine | Implements `core::runtime` abstractions. Focuses ONLY on executing WASM code. |
| `actor/` | Actor system integration | Integrates WASM components with airssys-rt actor system. Handles messaging, lifecycle, supervision. |

### The Dependency Rules (MANDATORY - NO EXCEPTIONS)

```
┌─────────────────────────────────────────────────────────┐
│                                                         │
│   actor/  ──────►  runtime/  ──────►  security/  ──────►  core/
│     │                 │                   │                │
│     │                 │                   │                │
│     └─────────────────┴───────────────────┴────────────────┘
│                           ALL import from core/
│
└─────────────────────────────────────────────────────────┘

ALLOWED:
  ✅ actor/    → runtime/
  ✅ actor/    → security/
  ✅ actor/    → core/
  ✅ runtime/  → security/
  ✅ runtime/  → core/
  ✅ security/ → core/

FORBIDDEN (NEVER, NO EXCEPTIONS):
  ❌ runtime/  → actor/      (BREAKS ARCHITECTURE)
  ❌ security/ → actor/      (BREAKS ARCHITECTURE)
  ❌ security/ → runtime/    (BREAKS ARCHITECTURE)
  ❌ core/     → ANY MODULE  (BREAKS EVERYTHING)
```

### Module Responsibilities (EXCLUSIVE)

#### `core/` - Foundation Layer
**OWNS:**
- All shared data types (ComponentId, ComponentMessage, etc.)
- All trait abstractions (RuntimeEngine, etc.)
- All error types
- All configuration types

**DOES NOT OWN:**
- Any implementation logic
- Any external crate dependencies (except std)
- Any async runtime code

**WHY:** Core must be dependency-free so ALL other modules can import from it without creating cycles.

---

#### `security/` - Security Layer
**OWNS:**
- Capability definitions and validation
- Permission checking
- Security policies
- ACL/RBAC integration (with airssys-osl)
- Security audit types

**DOES NOT OWN:**
- WASM execution
- Actor system logic
- Message routing

**WHY:** Security must be independent so it can be applied uniformly across runtime and actor layers without creating dependencies.

---

#### `runtime/` - WASM Execution Layer
**OWNS:**
- WasmEngine (Wasmtime integration)
- ComponentLoader (WASM loading)
- StoreManager (WASM store management)
- Host function DEFINITIONS (the function signatures)
- Resource limits enforcement
- WASM memory management

**DOES NOT OWN:**
- Message routing logic
- Actor lifecycle management
- Correlation tracking
- Response routing
- Pending request tracking
- Mailbox management
- Subscription management
- ANY inter-component communication orchestration

**WHY:** Runtime is for EXECUTING WASM. It provides primitives that actor/ uses. It should not know about actor system concepts.

---

#### `actor/` - Actor Integration Layer
**OWNS:**
- ComponentActor (wraps WASM in actor)
- ComponentRegistry (tracks components)
- ComponentSpawner (creates component actors)
- ActorSystemSubscriber (subscribes to broker, routes messages)
- CorrelationTracker (tracks pending requests)
- TimeoutHandler (handles request timeouts)
- ResponseRouter (routes responses to callbacks)
- PendingRequest, ResponseMessage, RequestError types
- ALL inter-component messaging orchestration
- Supervision integration
- Health monitoring

**DOES NOT OWN:**
- WASM execution engine (uses runtime/)
- WASM loading (uses runtime/)
- Host function execution (uses runtime/)

**WHY:** Actor layer integrates WASM with the actor system. It USES runtime/ for execution but OWNS all actor-related concerns.

---

## The Critical Insight

### Why This Matters

The module separation is NOT arbitrary. It exists because:

1. **`core/` at the bottom prevents cycles**: If ComponentMessage is in `actor/`, and `runtime/` needs it, `runtime/` must import from `actor/`. But `actor/` needs `runtime/` for execution. CYCLE.

2. **`runtime/` is execution only**: If `runtime/` contains messaging logic, it must know about actors. But actors need runtime. CYCLE.

3. **`actor/` orchestrates**: The actor layer sits on TOP. It uses runtime for execution and orchestrates everything else.

### The Flow

```
Component A wants to send request to Component B:

1. Component A's WASM calls send-request host function
   └── runtime/async_host.rs handles host call
       └── Creates correlation ID
       └── Returns to caller (actor/)

2. actor/ registers the pending request
   └── actor/message/correlation_tracker.rs
       └── Stores (correlation_id, response_tx)
       └── Starts timeout

3. actor/ publishes message to broker
   └── actor/message/message_publisher.rs
       └── Uses airssys-rt MessageBroker

4. actor/ receives message for Component B
   └── actor/message/actor_system_subscriber.rs
       └── Routes to Component B's mailbox

5. actor/ invokes handle-message on Component B
   └── actor/component/component_actor.rs
       └── Calls runtime/engine.rs to execute WASM

6. Component B's handle-message returns response
   └── Return value captured by actor/

7. actor/ routes response to callback
   └── actor/message/response_router.rs (SHOULD BE HERE)
       └── Resolves correlation
       └── Calls runtime/ to invoke handle-callback on Component A
```

**Notice:** `runtime/` is called INTO by `actor/`. `runtime/` never calls `actor/`.

---

## Implementation Requirements

### Immediate Actions Required

1. **Move `runtime/messaging.rs` to `actor/`**
   - `MessagingService` → `actor/message/messaging_service.rs`
   - `ResponseRouter` → `actor/message/response_router.rs`
   - These are messaging orchestration, NOT WASM execution

2. **Move messaging types to `core/`**
   - `CorrelationId` → `core/messaging.rs`
   - `PendingRequest` → `core/messaging.rs`
   - `ResponseMessage` → `core/messaging.rs`
   - `RequestError` → `core/messaging.rs`
   - These are DATA TYPES that both layers need

3. **Keep in `runtime/`**
   - `WasmEngine`
   - `ComponentLoader`
   - `StoreManager`
   - Host function structs (SendMessageHostFunction, SendRequestHostFunction)
   - But host functions call INTO actor/ services, NOT import actor/ types

4. **Verification**
   ```bash
   # MUST return NOTHING
   grep -r "use crate::actor" src/runtime/
   grep -r "use crate::runtime" src/security/
   grep -r "use crate::actor" src/security/
   grep -r "use crate::" src/core/
   ```

---

## Enforcement

### Pre-Implementation Checklist (MANDATORY)

Before writing ANY code, verify:

- [ ] What module does this code belong in?
- [ ] What modules will this code import from?
- [ ] Does this violate the dependency rules?
- [ ] If adding types that multiple modules need → put in `core/`
- [ ] If adding WASM execution logic → put in `runtime/`
- [ ] If adding actor/messaging logic → put in `actor/`

### Pre-Merge Verification (MANDATORY)

Before ANY code is considered complete:

```bash
# These MUST return empty results
grep -r "use crate::actor" src/runtime/
grep -r "use crate::runtime" src/security/
grep -r "use crate::actor" src/security/
grep -r "use crate::" src/core/
```

If ANY of these return results → CODE IS REJECTED.

---

## Consequences

### Positive
- Clear architecture that prevents cycles
- Each module has single responsibility
- Easy to understand where code belongs
- Prevents repeated architectural violations

### Constraints
- **Planning must verify module boundaries BEFORE proposing code locations**
- **Implementation must check imports BEFORE writing code**
- **No shortcuts, no exceptions, no "we'll fix it later"**

---

## Related Documents

- **KNOWLEDGE-WASM-037**: Rebuild Architecture - Clean Slate Design (**CURRENT FOR REBUILD**)
- **KNOWLEDGE-WASM-030**: Module Architecture - Hard Requirements (historical reference)
- **ADR-WASM-018**: Three-Layer Architecture
- **ADR-WASM-022**: Circular Dependency Remediation
- **KNOWLEDGE-WASM-028**: Circular Dependency Documentation

---

## History

| Date | Status | Reason |
|------|--------|--------|
| 2025-12-22 | ACCEPTED | Created after repeated architectural violations |

---

**This ADR is a HARD REQUIREMENT. Violations will result in immediate code rejection and rework.**

