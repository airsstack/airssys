# KNOWLEDGE-WASM-027: Duplicate WASM Runtime - Fatal Architecture Violation

**Document ID:** KNOWLEDGE-WASM-027  
**Created:** 2025-12-21  
**Updated:** 2025-12-21  
**Category:** Architecture / Fatal Errors / Lessons Learned  
**Maturity:** Stable  
**Severity:** 🔴 **CRITICAL / FATAL**

## Overview

This document records a **fatal architectural violation** discovered during WASM-TASK-006 Phase 2 implementation where a duplicate WASM runtime was created in `actor/component/` that uses the **WRONG API** (core WASM `wasmtime::Module`) instead of the **CORRECT API** (Component Model `wasmtime::component::Component`). This violates ADR-WASM-002 and renders the entire WIT interface system non-functional.

## Context

### Problem Statement

During Block 5 (Inter-Component Communication) implementation, we discovered that:

1. **`runtime/engine.rs`** correctly implements `WasmEngine` using **Component Model** API (`wasmtime::component::Component`, `wasmtime::component::Linker`)

2. **`actor/component/child_impl.rs`** created a **DUPLICATE** runtime using **core WASM** API (`wasmtime::Module`, `wasmtime::Linker`)

3. The `actor/` module should be a **thin adapter** that integrates WASM with airssys-rt, NOT contain its own WASM execution logic

4. As a workaround for the wrong API, developers created manual parameter marshalling infrastructure (`WasmBumpAllocator`, `HandleMessageParams`, `HandleMessageResult`) that should NOT exist with Component Model

### Scope

This is a **fundamental architecture violation** that affects:
- All component execution
- All WIT interface bindings (completely unused!)
- All inter-component communication
- All parameter passing between host and components

### What Should Have Been Built

```
┌─────────────────────────────────────────────────────────────┐
│                   CORRECT ARCHITECTURE                       │
├─────────────────────────────────────────────────────────────┤
│                                                              │
│  actor/component/ComponentActor                              │
│       │                                                      │
│       │ uses (dependency injection)                          │
│       ▼                                                      │
│  runtime/WasmEngine                                          │
│       │                                                      │
│       │ uses                                                 │
│       ▼                                                      │
│  wasmtime::component::Component  ← Component Model API       │
│       │                                                      │
│       │ automatic marshalling via                            │
│       ▼                                                      │
│  Canonical ABI (no manual bump allocator needed!)            │
│                                                              │
└─────────────────────────────────────────────────────────────┘
```

### What Was Actually Built (WRONG)

```
┌─────────────────────────────────────────────────────────────┐
│                   ACTUAL ARCHITECTURE (WRONG!)               │
├─────────────────────────────────────────────────────────────┤
│                                                              │
│  actor/component/ComponentActor                              │
│       │                                                      │
│       │ contains (DUPLICATE!)                                │
│       ▼                                                      │
│  WasmRuntime, WasmExports (lines 135-200)                   │
│       │                                                      │
│       │ uses                                                 │
│       ▼                                                      │
│  wasmtime::Module  ← CORE WASM API (WRONG!)                 │
│       │                                                      │
│       │ requires                                             │
│       ▼                                                      │
│  WasmBumpAllocator (536-693)  ← WORKAROUND                  │
│  HandleMessageParams (698-736) ← WORKAROUND                 │
│  HandleMessageResult (739-798) ← WORKAROUND                 │
│                                                              │
│  runtime/WasmEngine ← COMPLETELY IGNORED!                   │
│  generated/airssys_component.rs (154KB) ← UNUSED!           │
│                                                              │
└─────────────────────────────────────────────────────────────┘
```

## Technical Content

### The Fatal Mistake

**File: `actor/component/child_impl.rs` (line 38)**
```rust
// WRONG! This is core WASM API, not Component Model
use wasmtime::{Config, Engine, Linker, Module, Store};
```

**File: `runtime/engine.rs` (line 50)**
```rust
// CORRECT! This is Component Model API
use wasmtime::component::{Component, Linker};
```

### Why This Is Fatal

1. **Component Model is MANDATED by ADR-WASM-002**
   - Decision explicitly chose Wasmtime for Component Model support
   - Core WASM `Module` cannot use WIT interfaces

2. **154KB of Generated Bindings Are UNUSED**
   - `src/generated/airssys_component.rs` contains wit-bindgen generated types
   - These provide automatic marshalling via Canonical ABI
   - With Component Model, calls are typed: `instance.call_handle_message(&mut store, sender, &payload)?;`
   - The bump allocator workaround bypasses ALL of this

3. **Manual Marshalling Was Created as Workaround**
   - `WasmBumpAllocator` (158 lines) - manages WASM memory manually
   - `HandleMessageParams` (39 lines) - serializes parameters
   - `HandleMessageResult` (60 lines) - deserializes results
   - **NONE OF THIS IS NEEDED with Component Model!**

4. **Circular Dependency Was Created**
   - `runtime/` imports from `actor/` (ComponentMessage, ComponentRegistry)
   - Violates the correct dependency: `actor/ → runtime/ → core/`

### Evidence of the Violation

**File locations and line counts:**

| File | Lines | Issue |
|------|-------|-------|
| `src/actor/component/component_actor.rs` | 3,564 | Contains duplicate `WasmRuntime`, `WasmExports`, bump allocator |
| `src/actor/component/child_impl.rs` | ~500 | Uses `wasmtime::Module` (WRONG) |
| `src/runtime/engine.rs` | ~500 | Uses `wasmtime::component::Component` (CORRECT, ignored) |
| `src/generated/airssys_component.rs` | 154KB | wit-bindgen types (UNUSED!) |

**Wrong imports in runtime/ (circular dependency):**
| File | Line | Wrong Import |
|------|------|--------------|
| `async_host.rs` | 52 | `use crate::actor::ComponentMessage` |
| `messaging.rs` | 76 | `use crate::actor::ComponentMessage` |
| `messaging_subscription.rs` | 108-109 | `use crate::actor::{ComponentRegistry, ActorSystemSubscriber}` |

### Code That Must Be Deleted

```rust
// DELETE from actor/component/component_actor.rs:

/// WRONG - Should use runtime/WasmEngine instead
struct WasmRuntime { ... }  // lines 135-147

/// WRONG - Should use generated bindings instead
struct WasmExports { ... }  // lines 181-200

/// WORKAROUND - Not needed with Component Model
struct WasmBumpAllocator { ... }  // lines 536-693

/// WORKAROUND - Not needed with Component Model  
struct HandleMessageParams { ... }  // lines 698-736

/// WORKAROUND - Not needed with Component Model
struct HandleMessageResult { ... }  // lines 739-798
```

### Correct Implementation (What Should Exist)

```rust
// actor/component/component_actor.rs

use crate::runtime::WasmEngine;
use crate::core::ComponentHandle;

pub struct ComponentActor<S = ()> {
    component_id: ComponentId,
    engine: Arc<WasmEngine>,           // ← Inject existing runtime
    handle: Option<ComponentHandle>,    // ← From WasmEngine::load_component()
    // ... other fields ...
}

// Child::start() - Use WasmEngine
impl Child for ComponentActor {
    async fn start(&mut self) -> Result<(), Self::Error> {
        let bytes = load_wasm_bytes(&self.component_id).await?;
        let handle = self.engine.load_component(&self.component_id, &bytes).await?;
        self.handle = Some(handle);
        Ok(())
    }
}

// Actor::handle() - Use WasmEngine with Component Model
impl Actor for ComponentActor {
    async fn handle(&mut self, msg: ComponentMessage) -> Result<(), Self::Error> {
        let handle = self.handle.as_ref()?;
        
        // Component Model - automatic marshalling!
        // No bump allocator, no manual serialization
        let output = self.engine.call_handle_message(
            handle,
            &msg.sender,
            &msg.payload
        ).await?;
        
        Ok(())
    }
}
```

## Root Cause Analysis

### How Did This Happen?

1. **Lack of integration awareness**: Developer implementing `actor/` module didn't know `runtime/engine.rs` existed or didn't understand it was the correct component to use

2. **Core WASM familiarity**: Developer may have been more familiar with core WASM API (`Module`) than Component Model API (`Component`)

3. **Working code fallacy**: The core WASM approach "worked" at a basic level, so the violation wasn't immediately obvious

4. **Missing integration tests**: No tests verified that WIT-generated bindings were actually used

5. **Inadequate code review**: Architecture violation wasn't caught during review

### Warning Signs That Were Missed

- Creating 158 lines of `WasmBumpAllocator` when Component Model handles this automatically
- 154KB of generated bindings that were never imported or used
- `runtime/engine.rs` existing but being completely ignored by `actor/`
- `runtime/` importing from `actor/` (reverse dependency)

## Impact Assessment

### Severity: 🔴 CRITICAL

| Aspect | Impact |
|--------|--------|
| **WIT Interfaces** | 100% non-functional - not being used |
| **Type Safety** | Lost - manual byte manipulation instead of typed calls |
| **Security** | Reduced - bypasses Canonical ABI security features |
| **Maintainability** | 250+ lines of workaround code to maintain |
| **Performance** | Unknown - manual marshalling may be slower |
| **Effort to Fix** | 3-5 days of refactoring |

### Files That Need Changes

| File | Action | Effort |
|------|--------|--------|
| `actor/component/component_actor.rs` | Delete 250+ lines, add WasmEngine integration | High |
| `actor/component/child_impl.rs` | Rewrite to use WasmEngine | High |
| `actor/component/actor_impl.rs` | Update to use Component Model calls | Medium |
| `runtime/async_host.rs` | Remove circular imports | Low |
| `runtime/messaging.rs` | Move ComponentMessage to core/ | Low |
| `runtime/messaging_subscription.rs` | Remove circular imports | Low |
| `core/` | Add ComponentMessage type | Low |

## Lessons Learned

### Architectural Enforcement Needed

1. **Mandatory Architecture Review for New Modules**
   - Before creating any new WASM execution code, verify `runtime/` doesn't already provide it
   
2. **Import Direction Linting**
   - `runtime/` should NEVER import from `actor/`
   - Consider adding a linting rule

3. **Generated Code Usage Verification**
   - If wit-bindgen generates 154KB of bindings, tests must verify they're used
   
4. **ADR Compliance Checks**
   - ADR-WASM-002 mandates Component Model
   - New code using core WASM API should be flagged

### Development Guidelines (Going Forward)

✅ **DO:**
- Use `runtime/WasmEngine` for ALL WASM operations
- Use Component Model API (`wasmtime::component::*`)
- Use generated bindings from `src/generated/`
- Follow dependency direction: `actor/ → runtime/ → core/`

❌ **DON'T:**
- Create new WASM execution logic in `actor/`
- Use core WASM API (`wasmtime::Module`)
- Create manual marshalling/bump allocators
- Import from `actor/` in `runtime/`

## Remediation Plan

### Phase 1: Fix Circular Dependency (Prerequisite)
**Effort:** 2-4 hours

1. Move `ComponentMessage` from `actor/` to `core/`
2. Update all imports in `runtime/` to use `core::ComponentMessage`
3. Verify no `runtime/ → actor/` imports remain

### Phase 2: Refactor ComponentActor to Use WasmEngine
**Effort:** 8-12 hours

1. Delete from `component_actor.rs`:
   - `WasmRuntime` struct
   - `WasmExports` struct
   - `WasmBumpAllocator` 
   - `HandleMessageParams`
   - `HandleMessageResult`

2. Add to `ComponentActor`:
   - `engine: Arc<WasmEngine>` field
   - `handle: Option<ComponentHandle>` field

3. Refactor `child_impl.rs`:
   - Remove `use wasmtime::{Module, Store, ...}`
   - Use `self.engine.load_component()` instead

4. Refactor `actor_impl.rs`:
   - Use `self.engine.call_handle_message()` instead of manual calls

### Phase 3: Update Tests
**Effort:** 8-12 hours

1. Update all unit tests to use Component Model
2. Add integration tests verifying generated bindings usage
3. Add architecture compliance tests

**Total Estimated Effort:** 24-38 hours (3-5 days)

## References

### Related ADRs
- **ADR-WASM-002:** WASM Runtime Engine Selection (MANDATES Component Model)
- **ADR-WASM-006:** Component Isolation and Sandboxing (defines ComponentActor)
- **ADR-WASM-018:** Three-Layer Architecture (actor/ → runtime/ → core/)
- **ADR-WASM-021:** Duplicate Runtime Remediation (to be created)

### Related Files
- `src/runtime/engine.rs` - Correct WasmEngine implementation
- `src/actor/component/component_actor.rs` - Contains violations
- `src/actor/component/child_impl.rs` - Contains violations
- `src/generated/airssys_component.rs` - Unused generated bindings

### External References
- [Wasmtime Component Model Documentation](https://docs.wasmtime.dev/api/wasmtime/component/index.html)
- [WebAssembly Component Model Specification](https://github.com/WebAssembly/component-model)
- [Canonical ABI](https://github.com/WebAssembly/component-model/blob/main/design/mvp/CanonicalABI.md)

## History

### Version History
- **2025-12-21:** v1.0 - Initial documentation of fatal architecture violation

### Discovery
- **Discovered:** 2025-12-21 during WASM-TASK-006 Phase 2 Task 2.2 implementation
- **Reporter:** AI Assistant during code analysis
- **Verified By:** Code review of imports and API usage

---
**Severity:** 🔴 CRITICAL  
**Status:** Requires immediate remediation before Block 5 can proceed  
**Template Version:** 1.0
