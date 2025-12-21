# WASM-TASK-006-HOTFIX: Critical Architecture Remediation

**Task ID:** WASM-TASK-006-HOTFIX  
**Created:** 2025-12-21  
**Priority:** 🔴 **CRITICAL / BLOCKING**  
**Status:** NOT STARTED  
**Blocks:** All WASM-TASK-006 Phase 2+ work  
**Estimated Effort:** 4-6 days (28-42 hours)

---

## Executive Summary

### What's Wrong

Two fatal architectural violations discovered during WASM-TASK-006 Phase 2 implementation:

1. **Duplicate WASM Runtime (WRONG API):** `actor/component/` created its own WASM runtime using core WASM API (`wasmtime::Module`) instead of using the existing `runtime/WasmEngine` which correctly uses Component Model API (`wasmtime::component::Component`). This violates ADR-WASM-002.

2. **Circular Dependency:** `runtime/` imports types from `actor/`, violating the one-way dependency architecture (`actor/ → runtime/ → core/`) mandated by ADR-WASM-018.

### Impact

| Issue | Impact |
|-------|--------|
| **WIT Interfaces** | 100% NON-FUNCTIONAL - not being used |
| **Generated Bindings** | 154KB of code COMPLETELY UNUSED |
| **Type Safety** | BYPASSED - manual byte manipulation instead |
| **Workaround Code** | 250+ lines created to work around wrong API |
| **Layer Architecture** | VIOLATED - circular imports |

### Fix Required Before

- ❌ WASM-TASK-006 Phase 2 Task 2.2 (handle-message)
- ❌ WASM-TASK-006 Phase 2 Task 2.3 (send-message)
- ❌ All subsequent Block 5 tasks
- ❌ All Block 6+ tasks

---

## Phase 1: Fix Circular Dependency (ADR-WASM-022)

**Duration:** 2.5-4.5 hours  
**Must Complete:** Before Phase 2  
**Reference:** ADR-WASM-022, KNOWLEDGE-WASM-028

### Task 1.1: Move ComponentMessage to core/

**Effort:** 1-2 hours  
**Files Changed:** 5-8 files

#### Current State (WRONG)

```
src/actor/component/
    └── ComponentMessage defined here
        ↑
src/runtime/
    └── imports ComponentMessage from actor/ (CIRCULAR!)
```

#### Target State (CORRECT)

```
src/core/
    └── component_message.rs (NEW)
        └── ComponentMessage defined here
            ↑                    ↑
src/runtime/              src/actor/
    └── imports from core/    └── imports from core/
```

#### Implementation Steps

| Step | Action | File | Details |
|------|--------|------|---------|
| 1.1.1 | Create file | `src/core/component_message.rs` | New file |
| 1.1.2 | Move struct | `ComponentMessage` | Copy definition from actor/ |
| 1.1.3 | Update export | `src/core/mod.rs` | Add `pub mod component_message;` and `pub use` |
| 1.1.4 | Update import | `src/runtime/async_host.rs:52` | Change `crate::actor::ComponentMessage` → `crate::core::ComponentMessage` |
| 1.1.5 | Update import | `src/runtime/messaging.rs:76` | Change `crate::actor::ComponentMessage` → `crate::core::ComponentMessage` |
| 1.1.6 | Update import | `src/actor/component/*.rs` | Change to `crate::core::ComponentMessage` |
| 1.1.7 | Delete old | `src/actor/component/message.rs` | Remove if now empty (or just the struct) |
| 1.1.8 | Verify | `cargo build` | Must compile successfully |

#### Code Template

```rust
// src/core/component_message.rs

use crate::core::ComponentId;

/// Message passed between WASM components.
///
/// This is the fundamental unit of inter-component communication,
/// containing sender identity, recipient identity, and multicodec-encoded payload.
#[derive(Debug, Clone)]
pub struct ComponentMessage {
    /// The component sending this message
    pub sender: ComponentId,
    
    /// The target component to receive this message
    pub recipient: ComponentId,
    
    /// Multicodec-encoded payload bytes
    pub payload: Vec<u8>,
    
    /// Optional correlation ID for request-response patterns
    pub correlation_id: Option<String>,
    
    /// Timestamp when message was created
    pub timestamp: chrono::DateTime<chrono::Utc>,
}

impl ComponentMessage {
    /// Create a new component message
    pub fn new(
        sender: ComponentId,
        recipient: ComponentId,
        payload: Vec<u8>,
    ) -> Self {
        Self {
            sender,
            recipient,
            payload,
            correlation_id: None,
            timestamp: chrono::Utc::now(),
        }
    }

    /// Create a new request message with correlation ID
    pub fn new_request(
        sender: ComponentId,
        recipient: ComponentId,
        payload: Vec<u8>,
        correlation_id: String,
    ) -> Self {
        Self {
            sender,
            recipient,
            payload,
            correlation_id: Some(correlation_id),
            timestamp: chrono::Utc::now(),
        }
    }
}
```

#### Verification

```bash
# Must return NO results after fix
grep -r "use crate::actor::ComponentMessage" src/runtime/

# Must compile
cargo build

# Must pass all tests
cargo test
```

---

### Task 1.2: Relocate messaging_subscription.rs

**Effort:** 1-2 hours  
**Files Changed:** 3-4 files

#### Current State (WRONG)

```
src/runtime/messaging_subscription.rs
    └── imports ComponentRegistry, ActorSystemSubscriber from actor/
    └── This is ACTOR-LEVEL integration logic, not RUNTIME logic
```

#### Target State (CORRECT)

```
src/actor/component/messaging_subscription.rs
    └── imports from core/ and runtime/ only
    └── Proper location for actor integration logic
```

#### Implementation Steps

| Step | Action | File | Details |
|------|--------|------|---------|
| 1.2.1 | Move file | `src/runtime/messaging_subscription.rs` | → `src/actor/component/messaging_subscription.rs` |
| 1.2.2 | Update mod | `src/runtime/mod.rs` | Remove `pub mod messaging_subscription;` |
| 1.2.3 | Update mod | `src/actor/component/mod.rs` | Add `pub mod messaging_subscription;` |
| 1.2.4 | Fix imports | New location file | Update internal imports if needed |
| 1.2.5 | Update users | Any files importing from old location | Update import paths |
| 1.2.6 | Verify | `cargo build && cargo test` | Must pass |

#### Verification

```bash
# Must return NO results (runtime/ should not import from actor/)
grep -r "use crate::actor" src/runtime/

# Must compile and pass tests
cargo build && cargo test
```

---

### Task 1.3: Add CI Layer Dependency Enforcement

**Effort:** 30 minutes  
**Files Changed:** 1-2 files

#### Implementation

Create `.github/scripts/check-layer-deps.sh`:

```bash
#!/bin/bash
# Layer Dependency Enforcement Script
# Prevents runtime/ → actor/ and core/ → higher layer imports

set -e

echo "🔍 Checking layer dependencies..."

# Check 1: runtime/ must NOT import from actor/
echo "  Checking runtime/ → actor/ (should be NONE)..."
if grep -rq "use crate::actor" src/runtime/ 2>/dev/null; then
    echo "❌ ERROR: runtime/ imports from actor/"
    echo "   Violations found:"
    grep -rn "use crate::actor" src/runtime/
    exit 1
fi
echo "  ✅ runtime/ clean"

# Check 2: core/ must NOT import from runtime/ or actor/
echo "  Checking core/ → higher layers (should be NONE)..."
if grep -rq "use crate::runtime\|use crate::actor" src/core/ 2>/dev/null; then
    echo "❌ ERROR: core/ imports from higher layers"
    echo "   Violations found:"
    grep -rn "use crate::runtime\|use crate::actor" src/core/
    exit 1
fi
echo "  ✅ core/ clean"

echo ""
echo "✅ All layer dependency checks passed!"
```

Make executable and add to CI workflow.

---

### Phase 1 Completion Criteria

| Criterion | Verification |
|-----------|--------------|
| ✅ Zero `use crate::actor` in `src/runtime/` | `grep -r "use crate::actor" src/runtime/` returns nothing |
| ✅ Zero `use crate::runtime\|actor` in `src/core/` | `grep -r "use crate::runtime\|use crate::actor" src/core/` returns nothing |
| ✅ `ComponentMessage` exported from `core/` | Check `src/core/mod.rs` |
| ✅ `messaging_subscription.rs` in `actor/component/` | File exists at new location |
| ✅ All tests pass | `cargo test` succeeds |
| ✅ CI script passes | `./check-layer-deps.sh` succeeds |

---

## Phase 2: Fix Duplicate Runtime (ADR-WASM-021)

**Duration:** 3-5 days (24-36 hours)  
**Must Complete:** Before resuming WASM-TASK-006 Phase 2  
**Reference:** ADR-WASM-021, KNOWLEDGE-WASM-027

### Task 2.1: Delete Workaround Code from component_actor.rs

**Effort:** 2-4 hours  
**Files Changed:** 1 file (major deletions)

#### Code to DELETE

| Item | Lines | Reason |
|------|-------|--------|
| `WasmRuntime` struct | 135-147 | Duplicate of `runtime/WasmEngine` |
| `WasmExports` struct | 181-200 | Not needed with Component Model |
| `WasmBumpAllocator` struct + impl | 536-693 | Workaround for wrong API - Canonical ABI handles this |
| `HandleMessageParams` struct + impl | 698-736 | Workaround for wrong API - Component Model has typed calls |
| `HandleMessageResult` struct + impl | 739-798 | Workaround for wrong API - Component Model has typed returns |

**Total lines to delete:** ~260 lines

#### Implementation Steps

| Step | Action | Details |
|------|--------|---------|
| 2.1.1 | Backup | Make backup of `component_actor.rs` |
| 2.1.2 | Delete `WasmRuntime` | Remove struct definition and all usages |
| 2.1.3 | Delete `WasmExports` | Remove struct definition and all usages |
| 2.1.4 | Delete `WasmBumpAllocator` | Remove struct, impl, and all usages |
| 2.1.5 | Delete `HandleMessageParams` | Remove struct, impl, and all usages |
| 2.1.6 | Delete `HandleMessageResult` | Remove struct, impl, and all usages |
| 2.1.7 | Comment stubs | Add `// TODO: Replace with WasmEngine` where code was removed |
| 2.1.8 | Verify | `cargo build` will likely fail (expected - we deleted used code) |

**Note:** Build will fail after this task. That's expected. Task 2.2 will fix it.

---

### Task 2.2: Add WasmEngine Injection to ComponentActor

**Effort:** 4-6 hours  
**Files Changed:** 2-3 files

#### New Fields to Add

```rust
// src/actor/component/component_actor.rs

use std::sync::Arc;
use crate::runtime::WasmEngine;
use crate::core::ComponentHandle;

pub struct ComponentActor<S = ()> {
    // EXISTING fields...
    component_id: ComponentId,
    state: ActorState,
    config: ComponentConfig,
    // ... other existing fields ...
    
    // NEW fields (ADD THESE):
    /// Shared WASM execution engine (Component Model)
    engine: Arc<WasmEngine>,
    
    /// Handle to loaded component instance
    handle: Option<ComponentHandle>,
    
    // ... rest of existing fields ...
}
```

#### Updated Constructor

```rust
impl<S> ComponentActor<S> {
    /// Create a new ComponentActor with injected WasmEngine
    pub fn new(
        component_id: ComponentId,
        engine: Arc<WasmEngine>,  // ← NEW REQUIRED PARAMETER
        config: ComponentConfig,
        // ... other params ...
    ) -> Self {
        Self {
            component_id,
            engine,              // ← STORE IT
            handle: None,        // ← Initially no handle
            state: ActorState::Created,
            config,
            // ... rest ...
        }
    }
    
    /// Access the WASM engine
    pub fn engine(&self) -> &Arc<WasmEngine> {
        &self.engine
    }
    
    /// Access the component handle (if loaded)
    pub fn handle(&self) -> Option<&ComponentHandle> {
        self.handle.as_ref()
    }
}
```

#### Implementation Steps

| Step | Action | Details |
|------|--------|---------|
| 2.2.1 | Add imports | Add `use crate::runtime::WasmEngine;` and `use crate::core::ComponentHandle;` |
| 2.2.2 | Add fields | Add `engine: Arc<WasmEngine>` and `handle: Option<ComponentHandle>` |
| 2.2.3 | Update `new()` | Add `engine` parameter, store it |
| 2.2.4 | Add accessors | Add `engine()` and `handle()` methods |
| 2.2.5 | Update all callers | Every place that creates `ComponentActor::new()` needs to pass engine |
| 2.2.6 | Verify | `cargo build` - should still fail (handle not populated yet) |

---

### Task 2.3: Rewrite Child::start() to Use WasmEngine

**Effort:** 4-6 hours  
**Files Changed:** 1-2 files

#### Current Implementation (WRONG)

```rust
// src/actor/component/child_impl.rs - CURRENT (DELETE THIS)

use wasmtime::{Config, Engine, Linker, Module, Store};

impl<S> Child for ComponentActor<S> {
    async fn start(&mut self) -> Result<(), Self::Error> {
        // WRONG: Creates local engine with core WASM API
        let config = Config::new();
        let engine = Engine::new(&config)?;
        let module = Module::from_binary(&engine, &self.wasm_bytes)?;
        // ... manual setup ...
    }
}
```

#### New Implementation (CORRECT)

```rust
// src/actor/component/child_impl.rs - NEW (REPLACE WITH THIS)

use crate::runtime::WasmEngine;
use crate::core::ComponentHandle;

impl<S> Child for ComponentActor<S>
where
    S: Send + Sync + 'static,
{
    type Error = ComponentError;
    
    async fn start(&mut self) -> Result<(), Self::Error> {
        // Validate state
        if self.state != ActorState::Created {
            return Err(ComponentError::InvalidState {
                expected: ActorState::Created,
                actual: self.state,
            });
        }
        
        self.state = ActorState::Starting;
        
        // Load WASM bytes (from path, registry, etc.)
        let wasm_bytes = self.load_wasm_bytes().await?;
        
        // Use injected WasmEngine (Component Model!)
        let handle = self.engine
            .load_component(&self.component_id, &wasm_bytes)
            .await
            .map_err(|e| ComponentError::LoadFailed {
                component_id: self.component_id.clone(),
                source: e,
            })?;
        
        // Store the handle for later use
        self.handle = Some(handle);
        
        self.state = ActorState::Running;
        
        Ok(())
    }
    
    async fn stop(&mut self) -> Result<(), Self::Error> {
        self.state = ActorState::Stopping;
        
        // Drop the component handle (cleanup)
        if let Some(handle) = self.handle.take() {
            self.engine.unload_component(&handle).await?;
        }
        
        self.state = ActorState::Stopped;
        
        Ok(())
    }
}
```

#### Implementation Steps

| Step | Action | Details |
|------|--------|---------|
| 2.3.1 | Remove wrong imports | Delete `use wasmtime::{Config, Engine, Linker, Module, Store};` |
| 2.3.2 | Add correct imports | Add `use crate::runtime::WasmEngine;` |
| 2.3.3 | Rewrite `start()` | Use `self.engine.load_component()` |
| 2.3.4 | Rewrite `stop()` | Use `self.engine.unload_component()` |
| 2.3.5 | Remove local engine creation | Delete all `Engine::new()`, `Module::from_binary()` etc. |
| 2.3.6 | Verify | `cargo build` - should compile now |

---

### Task 2.4: Rewrite Actor::handle() for Component Model

**Effort:** 2-4 hours  
**Files Changed:** 1 file

#### Current Implementation (WRONG)

```rust
// src/actor/component/actor_impl.rs - CURRENT (DELETE THIS)

impl<S> Actor for ComponentActor<S> {
    async fn handle(&mut self, msg: ComponentMessage) -> Result<(), Self::Error> {
        // WRONG: Manual bump allocator and pointer manipulation
        let bump = WasmBumpAllocator::new(&mut self.store, &self.exports)?;
        let params = HandleMessageParams::write(&mut self.store, &bump, 
            &msg.sender, &msg.payload)?;
        let result_ptr = self.exports.handle_message.call(&mut self.store, params)?;
        let result = HandleMessageResult::read(&self.store, result_ptr)?;
        // ...
    }
}
```

#### New Implementation (CORRECT)

```rust
// src/actor/component/actor_impl.rs - NEW (REPLACE WITH THIS)

use crate::core::ComponentMessage;

impl<S> Actor for ComponentActor<S>
where
    S: Send + Sync + 'static,
{
    type Message = ComponentMessage;
    type Error = ComponentError;
    
    async fn handle(&mut self, msg: Self::Message) -> Result<(), Self::Error> {
        // Ensure we're in a valid state
        if self.state != ActorState::Running {
            return Err(ComponentError::NotRunning {
                component_id: self.component_id.clone(),
                state: self.state,
            });
        }
        
        // Get the component handle
        let handle = self.handle.as_ref()
            .ok_or_else(|| ComponentError::NotLoaded {
                component_id: self.component_id.clone(),
            })?;
        
        // Component Model typed call - automatic marshalling!
        // No bump allocator, no manual serialization
        let result = self.engine
            .call_handle_message(
                handle,
                &msg.sender,
                &msg.payload,
            )
            .await
            .map_err(|e| ComponentError::HandleMessageFailed {
                component_id: self.component_id.clone(),
                source: e,
            })?;
        
        // Process result if needed
        if let Some(response) = result {
            self.process_response(msg.correlation_id, response).await?;
        }
        
        Ok(())
    }
}
```

#### Implementation Steps

| Step | Action | Details |
|------|--------|---------|
| 2.4.1 | Remove bump allocator usage | Delete all `WasmBumpAllocator` references |
| 2.4.2 | Remove manual params | Delete all `HandleMessageParams` references |
| 2.4.3 | Remove manual result | Delete all `HandleMessageResult` references |
| 2.4.4 | Use Component Model | Use `self.engine.call_handle_message()` |
| 2.4.5 | Verify | `cargo build` - should compile |
| 2.4.6 | Test | `cargo test` - update tests as needed |

---

### Task 2.5: Extend WasmEngine if Needed

**Effort:** 2-4 hours (contingency)  
**Files Changed:** 1-2 files

If `WasmEngine` doesn't have all required methods, add them:

```rust
// src/runtime/engine.rs - ADD if missing

impl WasmEngine {
    /// Call the handle-message export on a component
    pub async fn call_handle_message(
        &self,
        handle: &ComponentHandle,
        sender: &ComponentId,
        payload: &[u8],
    ) -> Result<Option<Vec<u8>>, WasmError> {
        // Use Component Model bindings
        let mut store = handle.store.lock().await;
        
        // This uses the generated bindings from src/generated/airssys_component.rs
        let result = handle.instance
            .call_handle_message(&mut *store, sender.as_str(), payload)
            .map_err(|e| WasmError::InvocationFailed(e.to_string()))?;
        
        Ok(result)
    }
}
```

---

### Task 2.6: Update All Tests

**Effort:** 8-12 hours  
**Files Changed:** 10-20 test files

#### Test Updates Required

| Test Category | Action |
|---------------|--------|
| ComponentActor unit tests | Update to provide `Arc<WasmEngine>` |
| Child trait tests | Update to use Component Model |
| Actor trait tests | Remove bump allocator test code |
| Integration tests | Use real Component Model invocations |
| Add new tests | Verify generated bindings are used |

#### New Test: Verify Component Model Usage

```rust
// tests/component_model_usage_tests.rs

#[test]
fn test_component_model_api_used() {
    // Verify we're using Component Model, not core WASM
    let engine = WasmEngine::new(WasmConfig::default()).unwrap();
    
    // Load a test component
    let handle = engine.load_component(
        &ComponentId::new("test"),
        include_bytes!("fixtures/test_component.wasm"),
    ).await.unwrap();
    
    // This should use Component Model typed call
    let result = engine.call_handle_message(
        &handle,
        &ComponentId::new("sender"),
        b"test payload",
    ).await.unwrap();
    
    // Verify it worked
    assert!(result.is_some());
}

#[test]
fn test_no_core_wasm_in_actor() {
    // Meta-test: Verify no wasmtime::Module usage in actor/
    let actor_code = include_str!("../src/actor/component/child_impl.rs");
    assert!(!actor_code.contains("wasmtime::Module"), 
        "actor/ should not use wasmtime::Module (core WASM)");
    assert!(!actor_code.contains("WasmBumpAllocator"),
        "actor/ should not have bump allocator (workaround)");
}
```

---

### Phase 2 Completion Criteria

| Criterion | Verification |
|-----------|--------------|
| ✅ Zero `wasmtime::Module` in `src/actor/` | `grep -r "wasmtime::Module" src/actor/` returns nothing |
| ✅ Zero `WasmBumpAllocator` anywhere | `grep -r "WasmBumpAllocator" src/` returns nothing |
| ✅ Zero `HandleMessageParams` anywhere | `grep -r "HandleMessageParams" src/` returns nothing |
| ✅ Zero `HandleMessageResult` anywhere | `grep -r "HandleMessageResult" src/` returns nothing |
| ✅ ComponentActor has `engine` field | Check struct definition |
| ✅ ComponentActor has `handle` field | Check struct definition |
| ✅ `Child::start()` uses `WasmEngine` | Code review |
| ✅ `Actor::handle()` uses Component Model | Code review |
| ✅ All tests pass | `cargo test` succeeds |
| ✅ Zero clippy warnings | `cargo clippy --all-targets --all-features -- -D warnings` succeeds |
| ✅ Generated bindings used | Tests verify `src/generated/` is imported |

---

## Phase 3: Verification & Documentation

**Duration:** 2-4 hours  
**Must Complete:** Before resuming WASM-TASK-006

### Task 3.1: Full Verification

```bash
# Run all verification commands
cargo build
cargo test
cargo clippy --all-targets --all-features -- -D warnings

# Layer dependency check
grep -r "use crate::actor" src/runtime/
grep -r "use crate::runtime\|use crate::actor" src/core/

# Wrong API check
grep -r "wasmtime::Module" src/actor/
grep -r "WasmBumpAllocator\|HandleMessageParams\|HandleMessageResult" src/
```

### Task 3.2: Update Memory Bank

- Update `active-context.md` with remediation completion
- Update `progress.md` with fix details
- Update `_index.md` to reflect Task 1.2 status

### Task 3.3: Create Context Snapshot

Create snapshot: `2025-12-XX-architecture-remediation-complete.md`

---

## Timeline Summary

| Phase | Tasks | Duration | Dependencies |
|-------|-------|----------|--------------|
| **Phase 1** | 1.1-1.3 (Circular Dependency) | 2.5-4.5 hours | None |
| **Phase 2** | 2.1-2.6 (Duplicate Runtime) | 24-36 hours | Phase 1 |
| **Phase 3** | 3.1-3.3 (Verification) | 2-4 hours | Phase 2 |
| **TOTAL** | 12 tasks | **28-44 hours (4-6 days)** | |

---

## Risk Assessment

| Risk | Likelihood | Impact | Mitigation |
|------|------------|--------|------------|
| WasmEngine API insufficient | Low | Medium | Extend WasmEngine (Task 2.5) |
| Test failures | Medium | Low | Fix tests as we go |
| Missed import | Low | Low | Compiler catches it |
| Performance regression | Low | Medium | Benchmark before/after |

---

## Success Criteria

### After Remediation Complete ✅

- [ ] Zero circular dependencies (`runtime/` → `actor/`)
- [ ] Zero core WASM API in `actor/` module
- [ ] WasmEngine is single source of truth for WASM execution
- [ ] ComponentActor receives `Arc<WasmEngine>` via injection
- [ ] Generated bindings (`src/generated/`) are actively used
- [ ] All 250+ lines of workaround code deleted
- [ ] All tests passing
- [ ] Zero warnings (compiler + clippy)
- [ ] CI layer check in place

### Then Resume

- ✅ WASM-TASK-006 Phase 2 Task 2.2 (handle-message becomes trivial)
- ✅ WASM-TASK-006 Phase 2 Task 2.3 (send-message)
- ✅ All Block 5+ tasks

---

## References

### ADRs
- **ADR-WASM-002:** WASM Runtime Engine Selection (MANDATES Component Model)
- **ADR-WASM-018:** Three-Layer Architecture
- **ADR-WASM-021:** Duplicate Runtime Remediation (this task)
- **ADR-WASM-022:** Circular Dependency Remediation (this task)

### Knowledge Documents
- **KNOWLEDGE-WASM-027:** Duplicate WASM Runtime - Fatal Architecture Violation
- **KNOWLEDGE-WASM-028:** Circular Dependency Between actor/ and runtime/

### Key Files
| File | Current State | Target State |
|------|--------------|--------------|
| `src/runtime/engine.rs` | ✅ Correct (Component Model) | Use this |
| `src/actor/component/component_actor.rs` | ❌ Contains duplicate runtime | Delete workarounds, inject WasmEngine |
| `src/actor/component/child_impl.rs` | ❌ Uses core WASM | Rewrite to use WasmEngine |
| `src/actor/component/actor_impl.rs` | ❌ Uses bump allocator | Rewrite for Component Model |
| `src/generated/airssys_component.rs` | ❌ UNUSED (154KB) | Start using via WasmEngine |

---

**Created:** 2025-12-21  
**Author:** Architecture Team  
**Priority:** 🔴 CRITICAL - BLOCKING  
**Status:** NOT STARTED - Ready to begin Phase 1
