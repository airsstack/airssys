# KNOWLEDGE-WASM-037: Rebuild Architecture - Clean Slate Design

**Document ID:** KNOWLEDGE-WASM-037  
**Created:** 2026-01-05  
**Status:** Active Reference  
**Category:** Architecture / Module Design / Rebuild  
**Maturity:** Stable  
**Severity:** 🔴 **CRITICAL - FOUNDATION DOCUMENT**

## Overview

This document defines the **clean-slate rebuild architecture** for airssys-wasm. It supersedes previous module designs that suffered from circular dependencies, DI/DIP violations, and unclear module responsibilities. This architecture is designed from first principles to prevent architectural violations.

**Key Innovation:** Layer-organized `core/` module where each outer module has a corresponding `core/<module>/` containing its abstractions. Modules depend on abstractions, not concrete implementations.

## Context

### Problem Statement (Previous Architecture)

The previous airssys-wasm implementation suffered from:

1. **Circular Dependencies:** `runtime/` → `actor/` → `runtime/` (documented in KNOWLEDGE-WASM-028)
2. **DI/DIP Violations:** Modules importing concrete implementations instead of abstractions
3. **Module Boundary Confusion:** Code placed in wrong modules (KNOWLEDGE-WASM-032)
4. **Fake Tests:** Tests that didn't validate actual functionality (KNOWLEDGE-WASM-033)

### Scope

This architecture applies to the **complete rebuild** of airssys-wasm from scratch, including:
- Module structure and dependencies
- Integration with airssys-rt and airssys-osl
- WIT interface organization
- Testing strategy

### Prerequisites

- **ADR-WASM-023:** Module Boundary Enforcement
- **KNOWLEDGE-WASM-030:** Module Architecture Hard Requirements
- **KNOWLEDGE-WASM-031:** Foundational Architecture
- **KNOWLEDGE-WASM-033:** AI Fatal Mistakes - Lessons Learned

## Technical Content

### Layer Architecture

```
┌─────────────────────────────────────────────┐
│           LAYER 4: system/                  │
│  The COORDINATOR - wires everything         │
│  Imports: ALL internal + airssys-rt/osl     │
│  Injects concrete types into lower layers   │
└─────────────────────┬───────────────────────┘
                      │ (injects dependencies)
      ┌───────────────┼───────────────┐
      ▼               ▼               │
┌───────────┐   ┌───────────┐        │
│component/ │   │messaging/ │  LAYER 3
│ Uses ONLY │   │ Uses ONLY │        │
│ core/     │   │ core/     │        │
│ traits    │   │ traits    │        │
│ +airssys- │   │ +airssys- │        │
│ rt        │   │ rt        │        │
└───────────┘   └───────────┘        │
      │               │               │
      └───────┬───────┘               │
              ▼                       │
      ┌───────────────┐               │
      │   runtime/    │  LAYER 2B    │
      │ impl core/    │               │
      │ traits        │               │
      └───────┬───────┘               │
              │                       │
      ┌───────┴───────┐               │
      ▼               │               │
┌───────────┐         │               │
│ security/ │   LAYER 2A              │
│ impl core │                         │
│ traits    │                         │
└─────┬─────┘                         │
      │                               │
      ▼                               │
┌─────────────────────────────────────┤
│         LAYER 1: core/              │
│    std ONLY - Traits + Types        │
└─────────────────────────────────────┘
```

### Module Structure

```
airssys-wasm/
├── src/
│   ├── lib.rs
│   ├── prelude.rs
│   │
│   ├── core/               # LAYER 1: Foundation (std only)
│   │   ├── mod.rs
│   │   ├── component/      # Abstractions for component/ module
│   │   │   ├── id.rs       # ComponentId
│   │   │   ├── handle.rs   # ComponentHandle
│   │   │   ├── message.rs  # ComponentMessage
│   │   │   └── traits.rs   # Component-related traits
│   │   ├── messaging/      # Abstractions for messaging/ module
│   │   │   ├── correlation.rs
│   │   │   ├── payload.rs
│   │   │   └── traits.rs   # MessageRouter trait, etc.
│   │   ├── runtime/        # Abstractions for runtime/ module
│   │   │   ├── traits.rs   # RuntimeEngine, ComponentLoader
│   │   │   └── limits.rs   # ResourceLimits
│   │   ├── security/       # Abstractions for security/ module
│   │   │   ├── capability.rs
│   │   │   └── traits.rs   # SecurityValidator
│   │   ├── storage/        # Abstractions for storage (future)
│   │   │   └── traits.rs   # ComponentStorage trait
│   │   ├── config/
│   │   │   └── component.rs
│   │   └── errors/
│   │       ├── wasm.rs
│   │       ├── security.rs
│   │       └── messaging.rs
│   │
│   ├── security/           # LAYER 2A: Security
│   │   ├── mod.rs
│   │   ├── capability/     # Dedicated submodule
│   │   │   ├── types.rs
│   │   │   ├── set.rs
│   │   │   ├── validator.rs
│   │   │   └── grant.rs
│   │   ├── policy/
│   │   └── audit.rs        # Bridges to airssys-osl
│   │
│   ├── runtime/            # LAYER 2B: WASM Execution Only
│   │   ├── mod.rs
│   │   ├── engine.rs       # WasmtimeEngine
│   │   ├── loader.rs       # ComponentLoader
│   │   ├── store.rs        # StoreManager
│   │   ├── host_fn.rs      # Host functions
│   │   └── limiter.rs      # ResourceLimiter
│   │
│   ├── component/          # LAYER 3A: airssys-rt Integration
│   │   ├── mod.rs
│   │   ├── wrapper.rs      # ComponentWrapper (Actor + Child)
│   │   ├── registry.rs     # ComponentRegistry
│   │   ├── spawner.rs      # ComponentSpawner
│   │   └── supervisor.rs   # SupervisorConfig
│   │
│   ├── messaging/          # LAYER 3B: Messaging Patterns
│   │   ├── mod.rs
│   │   ├── patterns.rs     # FireAndForget, RequestResponse
│   │   ├── correlation.rs  # CorrelationTracker
│   │   ├── router.rs       # ResponseRouter
│   │   └── subscriber.rs   # ComponentSubscriber
│   │
│   └── system/             # LAYER 4: Coordination
│       ├── mod.rs
│       ├── manager.rs      # RuntimeManager
│       ├── lifecycle.rs    # System init/shutdown
│       └── builder.rs      # RuntimeBuilder
│
└── wit/                    # WIT Interface Definitions
    ├── core/
    │   ├── types.wit
    │   └── errors.wit
    ├── component/
    │   ├── lifecycle.wit
    │   └── messaging.wit
    └── host/
        └── messaging.wit
```

### Dependency Inversion Principle (Critical)

**Rule:** If Module A needs Module B, it depends on B's **trait in `core/`**, not B's concrete implementation.

```rust
// core/runtime/traits.rs - ABSTRACTION
pub trait RuntimeEngine: Send + Sync {
    fn call_handle_message(&self, handle: &ComponentHandle, msg: &ComponentMessage) 
        -> Result<Option<Vec<u8>>, WasmError>;
}

// component/wrapper.rs - CONSUMER (depends on abstraction)
use crate::core::runtime::traits::RuntimeEngine;

pub struct ComponentWrapper {
    engine: Arc<dyn RuntimeEngine>,  // Injected by system/
}

// runtime/engine.rs - IMPLEMENTATION
use crate::core::runtime::traits::RuntimeEngine;

pub struct WasmtimeEngine { /* ... */ }

impl RuntimeEngine for WasmtimeEngine {
    fn call_handle_message(&self, handle: &ComponentHandle, msg: &ComponentMessage) 
        -> Result<Option<Vec<u8>>, WasmError> {
        // Real implementation
    }
}

// system/manager.rs - COORDINATOR (injects concrete)
use crate::runtime::WasmtimeEngine;
use crate::component::ComponentWrapper;

let engine = Arc::new(WasmtimeEngine::new());
let wrapper = ComponentWrapper::new(engine);  // Inject
```

### Integration Points

| Module | Integrates With | Key Types Used |
|--------|-----------------|----------------|
| **security/** | `airssys-osl` | SecurityContext, ExecutionContext |
| **component/** | `airssys-rt` | Actor, Child, SupervisorNode, ActorSystem |
| **messaging/** | `airssys-rt` | MessageBroker, Mailbox |
| **system/** | Both | Coordinates all integrations |

### runtime/ Scope (WASM Only)

**Purpose:** WASM binary management ONLY. NO actor/component logic.

- ✅ Load WASM binary
- ✅ Validate WASM binary
- ✅ Execute WASM exports (handle-message, handle-callback)
- ✅ Manage WASM stores and memory
- ✅ Enforce resource limits
- ❌ NO actor lifecycle management
- ❌ NO message routing between components
- ❌ NO correlation tracking
- ❌ NO component registry

## Confirmed Architectural Features

### 1. WASI Preview 2 Integration (Default)

Integrated by default. Components use WASI Preview 2 for standardized host capabilities.

### 2. Component-Isolated Storage (Solana-Inspired)

Each component has its own isolated storage namespace:
```
[storage-root]/
├── [component-id-a]/
└── [component-id-b]/
```

### 3. Hot Reload (Blockchain-Inspired)

Update individual components without restarting the host system.

## Usage Patterns

### Verification Commands (CI Enforcement)

```bash
# Module boundary checks - ALL must return empty
grep -rn "use crate::component" src/runtime/
grep -rn "use crate::messaging" src/runtime/
grep -rn "use crate::system" src/runtime/
grep -rn "use crate::runtime" src/security/
grep -rn "use crate::" src/core/  # Should be empty
```

### WIT Build Strategy

Use `wit-bindgen::generate!` macro directly (no build.rs):

```rust
wit_bindgen::generate!({
    world: "component",
    path: "wit",
});
```

## Testing Strategy

| Level | Scope | Mocking Strategy |
|-------|-------|------------------|
| **Unit** | Single function/struct | Mock traits defined in `core/` |
| **Module Integration** | Cross-module | Real internal, mock external |
| **WASM Integration** | End-to-end | Real WASM fixtures, real wasmtime |

### Required Test Fixtures

| Fixture | Purpose |
|---------|---------|
| `echo.wasm` | Message echo |
| `counter.wasm` | State management |
| `timeout.wasm` | Timeout testing |
| `error.wasm` | Error handling |
| `callback.wasm` | Request-response |

## References

### Related Documentation
- **ADR-WASM-025:** Clean-Slate Rebuild Architecture (decision record for this knowledge)
- **ADR-WASM-023:** Module Boundary Enforcement
- **KNOWLEDGE-WASM-030:** Module Architecture Hard Requirements (historical)
- **KNOWLEDGE-WASM-031:** Foundational Architecture
- **KNOWLEDGE-WASM-033:** AI Fatal Mistakes - Lessons Learned
- **KNOWLEDGE-WASM-036:** Previous Three-Module Architecture (superseded by this)

### Supersedes
- KNOWLEDGE-WASM-035: Contained incorrect circular dependency
- KNOWLEDGE-WASM-036: Previous architecture without proper DIP

## History

- **2026-01-05:** 1.0 - Initial creation for clean-slate rebuild

---
**Template Version:** 1.0  
**Last Updated:** 2026-01-05
