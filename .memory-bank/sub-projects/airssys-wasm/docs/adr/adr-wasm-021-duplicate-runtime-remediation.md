# ADR-WASM-021: Duplicate WASM Runtime Remediation

**ADR ID:** ADR-WASM-021  
**Created:** 2025-12-21  
**Updated:** 2025-12-21  
**Status:** Accepted  
**Deciders:** Architecture Team  
**Severity:** 🔴 **CRITICAL**

## Title

Remediation of Duplicate WASM Runtime Using Wrong API (Core WASM vs Component Model)

## Context

### Problem Statement

A fatal architectural violation was discovered during WASM-TASK-006 Phase 2 implementation:

1. **`runtime/engine.rs`** correctly implements `WasmEngine` using **Component Model** API
2. **`actor/component/`** created a **duplicate** runtime using **core WASM** API
3. This violates ADR-WASM-002 which MANDATES Component Model usage
4. The `actor/` module contains 250+ lines of workaround code that shouldn't exist

### Technical Evidence

**CORRECT (runtime/engine.rs line 50):**
```rust
use wasmtime::component::{Component, Linker};
```

**WRONG (actor/component/child_impl.rs line 38):**
```rust
use wasmtime::{Config, Engine, Linker, Module, Store};
```

### Business Context

- **Component Model** provides automatic type marshalling via Canonical ABI
- **Core WASM** requires manual memory management and parameter passing
- The current implementation bypasses ALL WIT interfaces, rendering them useless
- 154KB of generated bindings (`src/generated/airssys_component.rs`) are completely unused

### Technical Context

**Current (Wrong) Architecture:**
```
actor/ComponentActor
    └── Contains WasmRuntime (DUPLICATE)
    └── Uses wasmtime::Module (WRONG API)
    └── Has WasmBumpAllocator (WORKAROUND)
    └── Ignores runtime/WasmEngine (CORRECT)
    └── Ignores generated bindings (UNUSED)
```

**Correct Architecture:**
```
actor/ComponentActor
    └── Injects Arc<WasmEngine>
    └── Uses runtime/WasmEngine
    └── Uses wasmtime::component::Component (CORRECT API)
    └── Uses generated bindings (TYPE-SAFE)
    └── No manual marshalling needed
```

### Stakeholders

- All Block 5+ implementers (cannot proceed correctly with current architecture)
- Component developers (their WIT interfaces are not being used)
- Security team (Canonical ABI provides security guarantees being bypassed)

## Decision

### Summary

**DELETE** the duplicate WASM runtime from `actor/component/` and **REFACTOR** `ComponentActor` to use the existing `runtime/WasmEngine` with proper dependency injection.

### Rationale

1. **ADR-WASM-002 Compliance**: Component Model is mandated, not optional
2. **Eliminate Duplication**: One runtime, not two
3. **Correct Layer Separation**: `actor/` adapts, `runtime/` executes
4. **Enable WIT Interfaces**: Generated bindings provide type safety
5. **Remove Workarounds**: No bump allocator needed with Component Model

### Decisions

#### Decision 1: Delete Duplicate Runtime Code

**Action:** Remove the following from `actor/component/component_actor.rs`:
- `WasmRuntime` struct (lines 135-147)
- `WasmExports` struct (lines 181-200)
- `WasmBumpAllocator` struct and impl (lines 536-693)
- `HandleMessageParams` struct and impl (lines 698-736)
- `HandleMessageResult` struct and impl (lines 739-798)

**Rationale:** These are workarounds for using the wrong API. Component Model handles all of this automatically.

#### Decision 2: Inject WasmEngine into ComponentActor

**Action:** Modify `ComponentActor` to receive `Arc<WasmEngine>`:

```rust
pub struct ComponentActor<S = ()> {
    component_id: ComponentId,
    engine: Arc<WasmEngine>,        // ← ADD: Injected runtime
    handle: Option<ComponentHandle>, // ← ADD: From load_component()
    // ... existing fields ...
}

impl ComponentActor {
    pub fn new(
        component_id: ComponentId,
        engine: Arc<WasmEngine>,     // ← ADD: Required parameter
        // ... other params ...
    ) -> Self {
        Self {
            component_id,
            engine,
            handle: None,
            // ...
        }
    }
}
```

**Rationale:** Dependency injection enables:
- Single source of truth for WASM execution
- Proper testability (mock engine in tests)
- Correct layer separation

#### Decision 3: Fix Child::start() to Use WasmEngine

**Action:** Rewrite `child_impl.rs` to use runtime:

```rust
// BEFORE (WRONG):
use wasmtime::{Module, Store, Engine, Linker};
let module = Module::from_binary(&engine, &bytes)?;

// AFTER (CORRECT):
use crate::runtime::WasmEngine;
let handle = self.engine.load_component(&self.component_id, &bytes).await?;
self.handle = Some(handle);
```

**Rationale:** Uses the correct, existing Component Model implementation.

#### Decision 4: Fix Actor::handle() to Use Component Model Calls

**Action:** Rewrite message handling to use typed Component Model calls:

```rust
// BEFORE (WRONG - manual marshalling):
let bump = WasmBumpAllocator::new(&mut store, &exports)?;
let params = HandleMessageParams::write(&mut store, &bump, sender, payload)?;
let result_ptr = exports.handle_message.call(&mut store, params)?;
let result = HandleMessageResult::read(&store, result_ptr)?;

// AFTER (CORRECT - Component Model):
let handle = self.handle.as_ref()?;
let output = self.engine.call_handle_message(handle, sender, &payload).await?;
```

**Rationale:** Component Model's Canonical ABI handles marshalling automatically with type safety.

#### Decision 5: Fix Circular Dependency

**Action:** Move `ComponentMessage` from `actor/` to `core/`:

**BEFORE:**
```
runtime/messaging.rs → imports → actor/ComponentMessage (WRONG!)
```

**AFTER:**
```
core/component.rs → defines → ComponentMessage
runtime/messaging.rs → imports → core/ComponentMessage (CORRECT)
actor/component_actor.rs → imports → core/ComponentMessage (CORRECT)
```

**Rationale:** Data types belong in `core/`, not in layer-specific modules.

### Assumptions

1. `runtime/WasmEngine` API is sufficient for ComponentActor needs
2. If WasmEngine needs extension, it should be extended, not duplicated
3. All tests can be updated to use Component Model

## Considered Options

### Option 1: Delete Duplicate and Use WasmEngine (CHOSEN)

**Description:** Remove all duplicate code, refactor to use existing runtime

**Pros:**
- Eliminates duplication
- Follows ADR-WASM-002
- Enables WIT interfaces
- Removes 250+ lines of workaround code
- Correct architecture

**Cons:**
- Significant refactoring effort (3-5 days)
- All tests need updating
- Breaking changes to ComponentActor API

**Implementation Effort:** High  
**Risk Level:** Medium (well-understood changes)

### Option 2: Keep Duplicate, Document as Technical Debt (REJECTED)

**Description:** Leave current implementation, document as debt to fix later

**Pros:**
- No immediate work required
- Current code "works" at basic level

**Cons:**
- WIT interfaces remain non-functional
- Block 5+ built on wrong foundation
- Technical debt compounds
- Violates ADR-WASM-002

**Implementation Effort:** None  
**Risk Level:** High (compounds over time)

### Option 3: Create Adapter Layer (REJECTED)

**Description:** Create adapter between duplicate runtime and correct runtime

**Pros:**
- Less invasive changes
- Gradual migration possible

**Cons:**
- Adds complexity instead of removing it
- Still maintains wrong code
- Doesn't fix the fundamental issue
- More code to maintain

**Implementation Effort:** Medium  
**Risk Level:** High (adds complexity)

## Implementation

### Implementation Plan

**Phase 1: Fix Circular Dependency (Day 1)**
1. Create `core/component_message.rs`
2. Move `ComponentMessage` type definition
3. Update all imports in `runtime/` and `actor/`
4. Verify build succeeds

**Phase 2: Refactor ComponentActor (Days 2-3)**
1. Add `engine: Arc<WasmEngine>` field to ComponentActor
2. Add `handle: Option<ComponentHandle>` field
3. Update constructor to require WasmEngine
4. Rewrite `Child::start()` in child_impl.rs
5. Rewrite `Actor::handle()` in actor_impl.rs
6. Delete workaround structs (WasmBumpAllocator, etc.)

**Phase 3: Update Tests (Days 4-5)**
1. Update unit tests to provide WasmEngine
2. Update integration tests
3. Add architecture compliance tests
4. Verify all tests pass

### Timeline

| Phase | Duration | Dependencies |
|-------|----------|--------------|
| Phase 1 | 0.5 days | None |
| Phase 2 | 2 days | Phase 1 complete |
| Phase 3 | 2 days | Phase 2 complete |
| **Total** | **4.5 days** | |

### Resources Required

- Developer familiar with both wasmtime Component Model and core WASM APIs
- Access to run full test suite
- No external dependencies

### Dependencies

- Must be completed before WASM-TASK-006 Phase 2 can resume
- Blocks all Block 5+ tasks

## Implications

### System Impact

| Aspect | Impact |
|--------|--------|
| **WIT Interfaces** | Enabled (currently non-functional) |
| **Generated Bindings** | Now used (154KB currently unused) |
| **ComponentActor API** | Breaking change (requires WasmEngine injection) |
| **Test Suite** | All component tests need updates |

### Performance Impact

**Expected Improvement:**
- Component Model's Canonical ABI is optimized
- Eliminates manual memory management overhead
- Type-safe calls may enable further optimization

### Security Impact

**Positive:**
- Canonical ABI provides security guarantees
- Type safety prevents memory corruption
- Proper sandboxing enforcement

### Maintainability Impact

**Significant Improvement:**
- 250+ lines of workaround code deleted
- Single WASM runtime to maintain
- Clear layer separation
- Type-safe API

## Compliance

### Workspace Standards

- **§4.3 Module Architecture:** Restored correct dependency direction
- **ADR-WASM-002:** Now compliant (Component Model usage)
- **ADR-WASM-018:** Now compliant (layer separation)

### Technical Debt

- **Debt Resolved:** Eliminates DEBT-WASM-XXX (duplicate runtime)
- **Debt Created:** None

## Monitoring and Validation

### Success Criteria

1. ✅ Zero `wasmtime::Module` usage in `actor/` module
2. ✅ All component tests use `wasmtime::component::Component`
3. ✅ `WasmBumpAllocator` deleted
4. ✅ No `runtime/` → `actor/` imports
5. ✅ Generated bindings actively used in tests

### Key Metrics

- Lines of code removed: 250+
- Unused code activated: 154KB of generated bindings
- Circular dependencies eliminated: 3 files

### Review Schedule

- **Post-implementation:** Architecture review before merging
- **1 week after:** Verify no regressions
- **Monthly:** Ensure no new violations

## Risks and Mitigations

### Identified Risks

| Risk | Likelihood | Impact | Mitigation |
|------|------------|--------|------------|
| Tests fail after refactor | Medium | Medium | Thorough test updates, run full suite |
| WasmEngine API insufficient | Low | Medium | Extend WasmEngine if needed |
| Performance regression | Low | Low | Benchmark before/after |
| Breaking external code | Low | Low | No external consumers yet |

### Contingency Plans

- If WasmEngine needs features: Add them to WasmEngine, don't create duplicate
- If timeline slips: Prioritize core functionality over edge cases

## References

### Related Documents

- **KNOWLEDGE-WASM-027:** Duplicate WASM Runtime - Fatal Architecture Violation
- **ADR-WASM-002:** WASM Runtime Engine Selection (mandates Component Model)
- **ADR-WASM-006:** Component Isolation and Sandboxing
- **ADR-WASM-018:** Three-Layer Architecture

### Key Files

| File | Role |
|------|------|
| `src/runtime/engine.rs` | Correct WasmEngine (to be used) |
| `src/actor/component/component_actor.rs` | Contains violations (to be fixed) |
| `src/actor/component/child_impl.rs` | Contains violations (to be fixed) |
| `src/generated/airssys_component.rs` | Generated bindings (to be used) |

### External References

- [Wasmtime Component Model](https://docs.wasmtime.dev/api/wasmtime/component/index.html)
- [Canonical ABI Specification](https://github.com/WebAssembly/component-model/blob/main/design/mvp/CanonicalABI.md)

## History

### Status Changes

- **2025-12-21:** Status set to Accepted - Critical remediation required

### Discovery Timeline

- **2025-12-21:** Violation discovered during WASM-TASK-006 Phase 2 Task 2.2
- **2025-12-21:** Root cause analysis completed
- **2025-12-21:** ADR created with remediation plan

---
**ADR Status:** Accepted  
**Priority:** 🔴 CRITICAL - Must be resolved before Block 5 can proceed  
**Template Version:** 1.0
