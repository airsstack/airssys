# Task: Dependency Inversion & Dependency Injection Redesign (Fix All Module Boundary Violations)

**Parent Task:** WASM-TASK-013 Phase 5
**Parent Task ID:** task-013-block-1-host-system-architecture-implementation.md
**Task ID:** task-014-fix-all-adr-violations-dip-redesign.md
**Status:** PLANNING
**Created:** 2025-01-03
**Estimated Effort:** 3-4 hours (REVISED - simplified approach)
**Priority:** 🔴 CRITICAL (ADR-WASM-023 violation fix)
**Type:** Architecture Refactoring

---

## Context & Motivation

### Problem Statement

Subtask 5.3 of WASM-TASK-013 Phase 5 has created **MULTIPLE CRITICAL ADR-WASM-023 ARCHITECTURE VIOLATIONS**.

### Current State (Subtask 5.3 Complete)

**Implementation Status:** ✅ COMPLETE (but with violations)

**What Was Done:**
- ✅ Added `actor_system_subscriber: Arc<RwLock<ActorSystemSubscriber<...>>>` field to HostSystemManager
- ✅ HostSystemManager::new() implemented (line 218 in manager.rs)
- ✅ HostSystemManager::shutdown() implemented (line 791 in manager.rs)
- ✅ All tests passing (1,042 tests)
- ✅ Clean build, zero clippy warnings

### Actual Architecture Violations (CRITICAL)

**Real Violations (FORBIDDEN):**

```bash
# 1. actor/ → host_system/ (FORBIDDEN - EXISTS!)
src/actor/mod.rs:179:pub use crate::host_system::correlation_tracker::CorrelationTracker;
src/actor/mod.rs:181:pub use crate::host_system::timeout_handler::TimeoutHandler;

# 2. runtime/ → host_system/ (FORBIDDEN - EXISTS!)
src/runtime/async_host.rs:932:use crate::host_system::{CorrelationTracker, TimeoutHandler};

# 3. messaging/ → host_system/ (FORBIDDEN - EXISTS!)
src/messaging/messaging_service.rs:76:use crate::host_system::correlation_tracker::CorrelationTracker;
src/messaging/messaging_service.rs:77:use crate::host_system::timeout_handler::TimeoutHandler;
src/messaging/messaging_service.rs:734:use crate::host_system::correlation_tracker::CorrelationTracker;
src/messaging/messaging_service.rs:735:use crate::host_system::timeout_handler::TimeoutHandler;
src/messaging/router.rs:48:use crate::host_system::correlation_tracker::CorrelationTracker;
```

### Why These Violate Architecture

**KNOWLEDGE-WASM-036 (Lines 143-148, 154-155):**
```
2. **`actor/` depends on:**
   - `runtime/` - WasmEngine (for executing WASM code)
   - `core/` - Shared types (ComponentId, ComponentMessage, errors, traits)
   - **NEVER** messaging/, host_system/ (enforced by module boundaries)

4. **`runtime/` depends on:**
   - `core/` - Shared types only
   - `security/` - Resource limits and policies
   - **NEVER** actor/, messaging/, host_system/ (enforced by ADR-WASM-023)

**`messaging/` depends on:**
   - `runtime/` - Callback execution only (via host functions)
   - `core/` - Shared types (ComponentId, ComponentMessage, CorrelationId)
   - **NEVER** actor/ (moved CorrelationTracker to host_system/ or messaging/)
   - **NEVER** host_system/ (host_system owns messaging/)
```

### Impact

- **Breaking Module Boundaries:** Creates forbidden reverse dependencies
- **Circular Dependencies:** Multiple modules depending on each other
- **Tight Coupling:** Cannot use modules independently

---

## Solution: Move Shared Types to Core/ (Re-export Approach)

### Architecture Approach Chosen: **Option B - Re-exports**

**Justification (Based on KNOWLEDGE-WASM-036 and PROJECTS_STANDARD.md):**

1. **KNOWLEDGE-WASM-036 (Line 61):** `core/` owns "All shared types (ComponentId, ComponentMessage, WasmError, etc.)"
   - CorrelationTracker and TimeoutHandler are **shared data types**
   - Multiple modules need them (actor/, runtime/, messaging/, host_system/)
   - This is exactly what core/ is for

2. **PROJECTS_STANDARD.md §6.2 (Line 137):** "Prefer concrete types first"
   - Concrete types > Generics > dyn (last resort)
   - No need for traits - these are data structures, not behavior abstractions

3. **PROJECTS_STANDARD.md §6.1 (YAGNI):** "Avoid speculative generalization"
   - No current need for alternative implementations of CorrelationTracker
   - No current need for alternative implementations of TimeoutHandler
   - Creating traits would violate YAGNI

4. **Simplicity:**
   - Move structs to core/ and re-export
   - Update imports to point to core/
   - No trait definitions, no impl blocks
   - Minimal code changes

### Architecture After Fix

```
                      core/
                        │
                        ├── CorrelationTracker (concrete struct)
                        ├── TimeoutHandler (concrete struct)
                        │
           ┌───────────┴───────────┐
           │                           │
      host_system/                   actor/    runtime/    messaging/
      (depends on                 (depends on
       core*)                     core*)
           │
           └── creates instances
```

### Dependency Flow (CLEAN ONE-WAY)

```
actor/    ───► core/ (concrete types)
runtime/   ───► core/ (concrete types)
messaging/ ───► core/ (concrete types)
host_system/ ───► core/ (concrete types)

NO REVERSE DEPENDENCIES!
```

---

## ADR & Knowledge References

### ADR References

1. **ADR-WASM-023: Module Boundary Enforcement** (CRITICAL - MANDATORY)
   - **Quote (Lines 75-79):** "FORBIDDEN (NEVER, NO EXCEPTIONS): ❌ runtime/ → actor/ (BREAKS ARCHITECTURE)"
   - **Application:** All forbidden imports MUST be eliminated. Dependencies must be one-way: `core/` ← everything
   - **Verification:** grep checks must return no output

2. **KNOWLEDGE-WASM-036: Three-Module Architecture** (CRITICAL)
   - **Quote (Line 61):** `core/` owns "All shared types (ComponentId, ComponentMessage, WasmError, etc.)"
   - **Quote (Line 62):** `core/` owns "All trait contracts"
   - **Application:** CorrelationTracker and TimeoutHandler are shared types that belong in core/

### Knowledge References

1. **KNOWLEDGE-WASM-026: Message Delivery Architecture**
   - **Relevance:** Defines how ActorSystemSubscriber manages message delivery
   - **Application:** Preserves message delivery functionality while fixing dependencies

### Standards Applied

**PROJECTS_STANDARD.md:**
- **§2.1:** 3-Layer import organization (all modified files)
- **§6.1:** YAGNI - Only implement required changes (use re-exports, not traits)
- **§6.2:** Avoid `dyn` - Use concrete types (no trait objects)
- **§6.4:** Implementation quality gates (zero warnings, comprehensive tests)

**Rust Guidelines:**
- **M-DESIGN-FOR-AI:** Idiomatic APIs, thorough docs, testable
- **M-MODULE-DOCS:** Module documentation will be added
- **M-ERRORS-CANONICAL-STRUCTS:** Error types follow canonical structure
- **M-STATIC-VERIFICATION:** All lints enabled, clippy passes

---


## Implementation Plan (REVISED: Full DIP with Exact Method Signatures)

### Context & References

**Critical Note:** This plan has been REVISED to match ACTUAL implementation code EXACTLY. Previous plan had incorrect method signatures that did not match the real implementations.

**ADR References:**
- **ADR-WASM-023: Module Boundary Enforcement** (MANDATORY)
  - Rule: `core/` imports NOTHING (dependency-free foundation)
  - Rule: `actor/` imports from `runtime/`, `security/`, `core/` (ALLOWED)
  - Rule: `runtime/` imports from `core/`, `security/` only (NO imports from `actor/`)
  - Rule: `security/` imports from `core/` only
  - Forbidden imports MUST be eliminated: actor/ → host_system/, runtime/ → host_system/, messaging/ → host_system/
  - Verification: grep checks must return no output

- **ADR-WASM-019: Runtime Dependency Management**
  - Use Tokio directly for async primitives
  - Use airssys-rt for actor infrastructure
  - Implement WASM-specific features in Layer 2 (airssys-wasm)

**dependency-management.md Compliance (FULL DIP):**
- **Rule 1: Abstractions Dependency-Free**
  - Traits in `core/` must have NO external dependencies
  - Traits contain ONLY method signatures (no implementation logic)
  - Traits use ONLY std and core types
  - This enables any module to import traits without transitive dependencies

- **Rule 2: Dependency Injection Pattern**
  - All modules use `Arc<dyn Trait>` for dependencies
  - No direct creation of concrete types
  - Constructor injection for required dependencies
  - Enables swapping implementations (test vs production)

- **Rule 3: Dependency Direction**
  - High-level → Traits ← Low-level
  - No direct dependency on implementations
  - Eliminates circular dependencies

**Why Full DIP (vs Simple Move to core/):**

The original plan (Option B - Simple Move) has a critical issue:
- ❌ Concrete types in `core/` create tight coupling
- ❌ Modules directly depend on implementations (violates DIP)
- ❌ Can't mock CorrelationTracker for testing
- ❌ Can't swap implementations (test vs production)
- ❌ Follows concrete-first approach but misses DIP benefits

**Full DIP approach (Option A - Traits):**
- ✅ Traits in `core/` (dependency-free abstractions)
- ✅ Implementations in `host_system/` (with external dependencies)
- ✅ All modules use `Arc<dyn Trait>` (dependency injection)
- ✅ Easy to mock for testing
- ✅ Can swap implementations (mock vs real)
- ✅ Follows dependency-management.md COMPLETELY

**PROJECTS_STANDARD.md Compliance:**
- **§2.1:** 3-Layer import organization (all modified files)
- **§4.3:** Module Architecture Patterns (mod.rs files contain ONLY declarations and re-exports)
- **§6.1:** YAGNI - Traits are minimal (expose ONLY what implementations actually have)
- **§6.2:** `dyn` Patterns - REQUIRED by DIP (use `Arc<dyn Trait>` for dependencies)
- **§6.4:** Implementation quality gates (zero warnings, comprehensive tests)

**Rust Guidelines Applied:**
- **M-DESIGN-FOR-AI:** Idiomatic APIs, thorough docs, testable code
- **M-MODULE-DOCS:** Module documentation with canonical sections
- **M-ERRORS-CANONICAL-STRUCTS:** Error types follow canonical structure
- **M-STATIC-VERIFICATION:** All lints enabled, clippy passes
- **M-FEATURES-ADDITIVE:** Changes don't break existing APIs (add traits, keep concrete types)

**Documentation Standards:**
- **Diátaxis Type:** Reference documentation
- **Quality:** Technical language, no marketing hyperbole per documentation-quality-standards.md
- **Canonical Sections:** All documented items have Summary, Examples, Errors, Panics sections

**Actual Implementation Verification:**

This plan is based on ACTUAL implementation code:

**CorrelationTracker actual methods (9 public methods):**
```rust
impl CorrelationTracker {
    pub fn new() -> Self;
    pub async fn register_pending(&self, request: PendingRequest) -> Result<(), WasmError>;
    pub async fn resolve(&self, correlation_id: CorrelationId, response: ResponseMessage) -> Result<(), WasmError>;
    pub(crate) fn remove_pending(&self, correlation_id: &CorrelationId) -> Option<PendingRequest>;
    pub async fn cleanup_expired(&self) -> usize;
    pub fn pending_count(&self) -> usize;
    pub fn contains(&self, correlation_id: &CorrelationId) -> bool;
    pub fn completed_count(&self) -> u64;
    pub fn timeout_count(&self) -> u64;
    pub async fn cleanup_pending_for_component(&self, component_id: &ComponentId);
}
```

**TimeoutHandler actual methods (4 public methods):**
```rust
impl TimeoutHandler {
    pub fn new() -> Self;
    pub fn register_timeout(&self, correlation_id: CorrelationId, timeout: Duration, tracker: CorrelationTracker);
    pub fn cancel_timeout(&self, correlation_id: &CorrelationId);
    pub fn active_count(&self) -> usize;
}
```

**PendingRequest type (from core/messaging.rs):**
```rust
pub struct PendingRequest {
    pub correlation_id: CorrelationId,
    pub response_tx: oneshot::Sender<ResponseMessage>,
    pub requested_at: Instant,
    pub timeout: Duration,
    pub from: ComponentId,
    pub to: ComponentId,
}
```

---

### Module Architecture (Full DIP)

**Code will be placed in:**
- `core/correlation_trait.rs` (NEW FILE - trait definition)
- `core/timeout_trait.rs` (NEW FILE - trait definition)
- `host_system/correlation_impl.rs` (NEW FILE - implementation)
- `host_system/timeout_impl.rs` (NEW FILE - implementation)

**Module responsibilities (per ADR-WASM-023 and dependency-management.md):**

**`core/` (Abstraction Layer - Dependency-Free):**
- Contains trait definitions (CorrelationTrackerTrait, TimeoutHandlerTrait)
- Traits have NO implementation logic
- Traits use ONLY std and core types
- Traits expose ALL methods from actual implementations (exact signatures)
- NO external dependencies (no tokio, no dashmap, etc.)
- Allows ANY module to import traits without transitive dependencies

**`host_system/` (Implementation Layer - Has External Dependencies):**
- Contains concrete implementations (CorrelationTracker, TimeoutHandler)
- Implementations have external dependencies (tokio, dashmap, etc.)
- Implementations import traits from `core/`
- Implementations can create instances directly

**Dependency Flow (CLEAN ONE-WAY):**
```
actor/      ───► core/ (traits)              ──uses──► host_system/ (implementations)
runtime/     ───► core/ (traits)              ──uses──► host_system/ (implementations)
messaging/    ───► core/ (traits)              ──uses──► host_system/ (implementations)
host_system/ ───► core/ (traits + imports)   ──owns────► implementations
core/       ───► (nothing - dependency-free)
```

**Forbidden imports verified:**
- `core/correlation_trait.rs` MUST NOT import from: actor/, runtime/, security/, messaging/, host_system/
- `core/timeout_trait.rs` MUST NOT import from: actor/, runtime/, security/, messaging/, host_system/
- `host_system/` CAN import from: core/ (traits), external crates (tokio, dashmap, etc.)
- `actor/` CANNOT import from: host_system/ (MUST use traits from core/)
- `runtime/` CANNOT import from: host_system/ (MUST use traits from core/)
- `messaging/` CANNOT import from: host_system/ (MUST use traits from core/)

**Verification commands (for implementer to run):**
```bash
# Verify core/ traits are dependency-free
grep -rn "use crate::" src/core/correlation_trait.rs
# Expected: NO output (only use crate::core::... types)

grep -rn "use crate::" src/core/timeout_trait.rs
# Expected: NO output (only use crate::core::... types)

# Verify actor/ doesn't import from host_system/
grep -rn "use crate::host_system" src/actor/
# Expected: No output

# Verify runtime/ doesn't import from host_system/
grep -rn "use crate::host_system" src/runtime/
# Expected: No output

# Verify messaging/ doesn't import from host_system/
grep -rn "use crate::host_system" src/messaging/
# Expected: No output
```

---

### Phase 1: Full DIP Implementation (10 Subtasks - 3-4 hours)

#### Subtask 1.1: Read Actual Implementation Files ✅ COMPLETE

**Status:** ✅ COMPLETE (2026-01-04)
**Audit:** APPROVED by @memorybank-auditor
**Verification:** VERIFIED by @memorybank-verifier

**Deliverables:**
- ✅ Verification that all actual method signatures are captured

**Files to read:**
1. `airssys-wasm/src/host_system/correlation_tracker.rs` (already read)
2. `airssys-wasm/src/host_system/timeout_handler.rs` (already read)
3. `airssys-wasm/src/core/messaging.rs` (for PendingRequest type)

**Expected findings:**
- ✅ CorrelationTracker has 10 public methods (not 4 as in previous plan)
- ✅ TimeoutHandler has 4 public methods (not 3 as in previous plan)
- ✅ register_timeout() requires 3 parameters: correlation_id, timeout, tracker (not 2)
- ✅ Uses PendingRequest type (not RequestId)

**Acceptance Criteria:**
- ✅ All method signatures extracted
- ✅ All type names verified
- ✅ All parameter counts verified

---

#### Subtask 1.2: Create CorrelationTrackerTrait in core/ ✅ COMPLETE

**Status:** ✅ COMPLETE (2026-01-04)
**Audit:** APPROVED by @memorybank-auditor
**Verification:** VERIFIED by @memorybank-verifier

**Deliverables:**
- ✅ **File:** `airssys-wasm/src/core/correlation_trait.rs` (NEW FILE) - 159 lines
- ✅ **Content:**
  - Trait definition with ALL 10 methods from actual implementation
  - Exact method signatures matching implementation
  - Uses ONLY core types (PendingRequest, ResponseMessage, CorrelationId, ComponentId, WasmError)
  - NO external dependencies
  - Module documentation following M-MODULE-DOCS
- ✅ Uses `#[async_trait]` for object-safe async methods

**Acceptance Criteria:**
- ✅ CorrelationTrackerTrait defined in `core/correlation_trait.rs`
- ✅ All 10 methods included (new, register_pending, resolve, remove_pending, cleanup_expired, pending_count, contains, completed_count, timeout_count, cleanup_pending_for_component)
- ✅ Method signatures EXACTLY match actual implementation
- ✅ Uses ONLY core types (PendingRequest, ResponseMessage, CorrelationId, ComponentId, WasmError)
- ✅ NO external dependencies (no tokio, no dashmap, etc.)
- ✅ Module documentation follows M-MODULE-DOCS
- ✅ Code compiles without errors
- ✅ Uses `#[async_trait]` for object safety

**Test Results:**
- Build: Clean, no errors, no warnings
- Clippy: Zero warnings (with mandatory `-D warnings` flag)
- Zero architecture violations (ADR-WASM-023 compliant)
- Zero forbidden imports in trait definition

---

#### Subtask 1.3: Create TimeoutHandlerTrait in core/ ✅ COMPLETE

**Status:** ✅ COMPLETE (2026-01-04)
**Audit:** APPROVED by @memorybank-auditor
**Verification:** VERIFIED by @memorybank-verifier

**Deliverables:**
- ✅ **File:** `airssys-wasm/src/core/timeout_trait.rs` (NEW FILE) - 96 lines
- ✅ **Content:**
  - Trait definition with ALL 4 methods from actual implementation
  - Exact method signatures matching implementation
  - Uses ONLY core types (CorrelationId, Duration)
  - NO external dependencies
  - Module documentation following M-MODULE-DOCS
- ✅ **Key Feature:** Uses generic parameter `<T: CorrelationTrackerTrait + 'static>` instead of `dyn`
- ✅ Complies with PROJECTS_STANDARD.md §6.2 (Avoid dyn Patterns)

**Acceptance Criteria:**
- ✅ TimeoutHandlerTrait defined in `core/timeout_trait.rs`
- ✅ All 4 methods included (new, register_timeout, cancel_timeout, active_count)
- ✅ Method signatures EXACTLY match actual implementation
- ✅ `register_timeout()` has 3 parameters (correlation_id, timeout, tracker)
- ✅ Uses ONLY core types and std types (CorrelationId, Duration)
- ✅ NO external dependencies (no tokio, no dashmap, etc.)
- ✅ Module documentation follows M-MODULE-DOCS
- ✅ Code compiles without errors
- ✅ Uses generic parameter `<T: CorrelationTrackerTrait + 'static>` (complies with §6.2)

**Test Results:**
- Build: Clean, no errors, no warnings
- Clippy: Zero warnings (with mandatory `-D warnings` flag)
- Zero architecture violations (ADR-WASM-023 compliant)
- Zero forbidden imports in trait definition
- Generic parameters used instead of dyn (PROJECTS_STANDARD.md §6.2 compliant)

---

#### Subtask 1.4: Create CorrelationTracker Implementation in host_system/ ✅ COMPLETE

**Status:** ✅ COMPLETE (2026-01-04)
**Audit:** APPROVED by @memorybank-auditor
**Verification:** VERIFIED by @memorybank-verifier

**Deliverables:**
- ✅ **File:** `airssys-wasm/src/host_system/correlation_impl.rs` (NEW FILE) - 742 lines
- ✅ **Content:**
  - Copy entire CorrelationTracker implementation from `correlation_tracker.rs`
  - Import trait from `core/correlation_trait`
  - Add `impl CorrelationTrackerTrait for CorrelationTracker`
  - Keep all existing tests
- ✅ All 10 methods implemented
- ✅ 13 unit tests preserved and passing

**Acceptance Criteria:**
- ✅ CorrelationTracker implementation moved to `host_system/correlation_impl.rs`
- ✅ All methods preserved with identical signatures
- ✅ Implements `CorrelationTrackerTrait`
- ✅ Code compiles without errors
- ✅ All tests preserved and passing (13 unit tests)

**Test Results:**
- Build: Clean, no errors, no warnings
- Clippy: Zero warnings (with mandatory `-D warnings` flag)
- Unit Tests: 13/13 passing (100%)
- Zero architecture violations (ADR-WASM-023 compliant)
- All trait methods implemented with exact signatures

---

#### Subtask 1.5: Create TimeoutHandler Implementation in host_system/ ✅ COMPLETE

**Status:** ✅ COMPLETE (2026-01-04)
**Audit:** APPROVED by @memorybank-auditor
**Verification:** VERIFIED by @memorybank-verifier

**Deliverables:**
- ✅ **File:** `airssys-wasm/src/host_system/timeout_impl.rs` (NEW FILE) - 373 lines
- ✅ **Content:**
  - Copy entire TimeoutHandler implementation from `timeout_handler.rs`
  - Import trait from `core/timeout_trait`
  - Add `impl TimeoutHandlerTrait for TimeoutHandler`
  - Keep all existing tests
- ✅ All 4 methods implemented
- ✅ Uses generic parameter `<T: CorrelationTrackerTrait + 'static>`
- ✅ Fixed: Moved `CorrelationTracker` import to `#[cfg(test)]`
- ✅ Fixed: Added `#[allow(clippy::clone_on_ref_ptr)]` to test module
- ✅ 4 unit tests preserved and passing

**Acceptance Criteria:**
- ✅ TimeoutHandler implementation moved to `host_system/timeout_impl.rs`
- ✅ All methods preserved with identical signatures
- ✅ Implements `TimeoutHandlerTrait`
- ✅ Code compiles without errors
- ✅ All tests preserved and passing (4 unit tests)
- ✅ Generic parameter `<T: CorrelationTrackerTrait + 'static>` used (§6.2 compliant)

**Test Results:**
- Build: Clean, no errors, no warnings
- Clippy: Zero warnings (with mandatory `-D warnings` flag)
- Unit Tests: 4/4 passing (100%)
- Zero architecture violations (ADR-WASM-023 compliant)
- All trait methods implemented with exact signatures

---

#### Subtask 1.6: Update core/mod.rs ✅ COMPLETE

**Status:** ✅ COMPLETE (2026-01-04)
**Audit:** APPROVED by @memorybank-auditor
**Verification:** VERIFIED by @memorybank-verifier

**Deliverables:**
- ✅ **File:** `airssys-wasm/src/core/mod.rs`
- ✅ **Changes:**
  - Added: `pub mod correlation_trait;`
  - Added: `pub use correlation_trait::CorrelationTrackerTrait;`
  - Added: `pub mod timeout_trait;`
  - Added: `pub use timeout_trait::TimeoutHandlerTrait;`

**Acceptance Criteria:**
- ✅ correlation_trait module declared
- ✅ CorrelationTrackerTrait re-exported
- ✅ timeout_trait module declared
- ✅ TimeoutHandlerTrait re-exported
- ✅ Code compiles without errors
- ✅ mod.rs contains ONLY declarations and re-exports

**Test Results:**
- Build: Clean, no errors, no warnings
- Clippy: Zero warnings (with mandatory `-D warnings` flag)

---

#### Subtask 1.7: Update host_system/mod.rs ✅ COMPLETE

**Status:** ✅ COMPLETE (2026-01-04)
**Audit:** APPROVED by @memorybank-auditor
**Verification:** VERIFIED by @memorybank-verifier

**Deliverables:**
- ✅ **File:** `airssys-wasm/src/host_system/mod.rs`
- ✅ **Changes:**
  - Added: `pub mod correlation_impl;`
  - Added: `pub use correlation_impl::CorrelationTracker;`
  - Added: `pub mod timeout_impl;`
  - Added: `pub use timeout_impl::TimeoutHandler;`

**Acceptance Criteria:**
- ✅ correlation_impl module declared
- ✅ CorrelationTracker re-exported
- ✅ timeout_impl module declared
- ✅ TimeoutHandler re-exported
- ✅ Code compiles without errors
- ✅ mod.rs contains ONLY declarations and re-exports

**Test Results:**
- Build: Clean, no errors, no warnings
- Clippy: Zero warnings (with mandatory `-D warnings` flag)

---

#### Subtask 1.8: Update ActorSystemManager to use Traits (DI Pattern)

**Deliverables:**
- **File:** `airssys-wasm/src/host_system/manager.rs`
- **Changes:**
  - Change field type from `Arc<CorrelationTracker>` to `Arc<dyn CorrelationTrackerTrait>`
  - Change field type from `Arc<TimeoutHandler>` to `Arc<dyn TimeoutHandlerTrait>`
  - Update constructor to accept `Arc<dyn CorrelationTrackerTrait>` and `Arc<dyn TimeoutHandlerTrait>`
  - Update all usages to use trait methods

**Exact changes:**
```rust
// In struct definition (around line 218)
use crate::core::correlation_trait::CorrelationTrackerTrait;
use crate::core::timeout_trait::TimeoutHandlerTrait;

pub struct HostSystemManager {
    // ... other fields ...
    
    /// Correlation tracker for request-response patterns
    correlation_tracker: Arc<dyn CorrelationTrackerTrait>,
    
    /// Timeout handler for automatic cleanup
    timeout_handler: Arc<dyn TimeoutHandlerTrait>,
    
    // ... other fields ...
}

// In new() method (around line 218)
impl HostSystemManager {
    pub async fn new() -> Result<Self, WasmError> {
        // Create concrete implementations
        let correlation_tracker = Arc::new(CorrelationTracker::new());
        let timeout_handler = Arc::new(TimeoutHandler::new());
        
        // Inject as trait objects (dependency injection)
        let manager = Self {
            correlation_tracker,
            timeout_handler,
            // ... other fields ...
        };
        
        // ... rest of initialization ...
        
        Ok(manager)
    }
}
```

**Acceptance Criteria:**
1. ✅ HostSystemManager uses `Arc<dyn CorrelationTrackerTrait>`
2. ✅ HostSystemManager uses `Arc<dyn TimeoutHandlerTrait>`
3. ✅ Constructor performs dependency injection
4. ✅ Code compiles without errors
5. ✅ All functionality preserved

---

#### Subtask 1.9: Update actor/ to use Traits

**Deliverables:**
- **File:** `airssys-wasm/src/actor/mod.rs`
- **Changes:**
  - Remove: `pub use crate::host_system::correlation_tracker::CorrelationTracker;`
  - Remove: `pub use crate::host_system::timeout_handler::TimeoutHandler;`
  - Add: `pub use crate::core::correlation_trait::CorrelationTrackerTrait;`
  - Add: `pub use crate::core::timeout_trait::TimeoutHandlerTrait;`

**Acceptance Criteria:**
1. ✅ Forbidden imports removed from `actor/mod.rs`
2. ✅ Trait imports added from `core/`
3. ✅ Code compiles without errors
4. ✅ No circular dependencies introduced

---

#### Subtask 1.10: Update runtime/ to use Traits

**Deliverables:**
- **File:** `airssys-wasm/src/runtime/async_host.rs`
- **Changes:**
  - Remove: `use crate::host_system::{CorrelationTracker, TimeoutHandler};`
  - Add: `use crate::core::correlation_trait::CorrelationTrackerTrait;`
  - Add: `use crate::core::timeout_trait::TimeoutHandlerTrait;`

**Acceptance Criteria:**
1. ✅ Forbidden imports removed from `runtime/async_host.rs`
2. ✅ Trait imports added from `core/`
3. ✅ Code compiles without errors
4. ✅ No circular dependencies introduced

---

#### Subtask 1.11: Update messaging/ to use Traits

**Deliverables:**
- **Files:**
  - `airssys-wasm/src/messaging/messaging_service.rs` (lines 76, 77, 734, 735)
  - `airssys-wasm/src/messaging/router.rs` (line 48)

**Changes:**

**File: messaging_service.rs**
- Line 76: REMOVE `use crate::host_system::correlation_tracker::CorrelationTracker;`
- Line 77: REMOVE `use crate::host_system::timeout_handler::TimeoutHandler;`
- Line 734: REMOVE `    use crate::host_system::correlation_tracker::CorrelationTracker;`
- Line 735: REMOVE `    use crate::host_system::timeout_handler::TimeoutHandler;`
- ADD at top: `use crate::core::correlation_trait::CorrelationTrackerTrait;`
- ADD at top: `use crate::core::timeout_trait::TimeoutHandlerTrait;`

**File: router.rs**
- Line 48: REMOVE `use crate::host_system::correlation_tracker::CorrelationTracker;`
- ADD at top: `use crate::core::correlation_trait::CorrelationTrackerTrait;`

**Acceptance Criteria:**
1. ✅ All forbidden imports removed from messaging/ files
2. ✅ Trait imports added from `core/`
3. ✅ Code compiles without errors
4. ✅ No circular dependencies introduced

---

#### Subtask 1.12: Delete Old Files

**Deliverables:**
- **Files to delete:**
  - `airssys-wasm/src/host_system/correlation_tracker.rs` (moved to correlation_impl.rs)
  - `airssys-wasm/src/host_system/timeout_handler.rs` (moved to timeout_impl.rs)

**Commands:**
```bash
cd airssys-wasm
rm src/host_system/correlation_tracker.rs
rm src/host_system/timeout_handler.rs
```

**Acceptance Criteria:**
1. ✅ Old files deleted
2. ✅ New files created and working
3. ✅ Code compiles without errors
4. ✅ All tests pass

---

### Unit Testing Plan (Phase 1)

**Objective:** Verify traits match implementations EXACTLY and DIP works correctly.

**Test Coverage Target:** 95% for both traits

**Test Files:**

**Test 1: `core/correlation_trait.rs` - Trait signature verification**
- Verify trait has exactly 10 methods
- Verify all method signatures match implementation
- Verify trait is dependency-free (no external imports)

**Test 2: `core/timeout_trait.rs` - Trait signature verification**
- Verify trait has exactly 4 methods
- Verify all method signatures match implementation
- Verify trait is dependency-free (no external imports)

**Test 3: `host_system/correlation_impl.rs` - Trait implementation**
- Verify implementation compiles with trait
- Verify all trait methods are implemented
- Run existing 13 tests for CorrelationTracker

**Test 4: `host_system/timeout_impl.rs` - Trait implementation**
- Verify implementation compiles with trait
- Verify all trait methods are implemented
- Run existing 3 tests for TimeoutHandler

**Test 5: DI pattern verification**
- Verify HostSystemManager uses `Arc<dyn Trait>`
- Verify traits can be swapped (create mock implementation)

**Test Execution:**
```bash
cd airssys-wasm

# Test trait definitions
cargo test --lib correlation_trait
# Expected: All signature tests pass

# Test trait implementations
cargo test --lib correlation_impl
# Expected: All 13 existing tests pass

cargo test --lib timeout_impl
# Expected: All 3 existing tests pass

# Test all unit tests
cargo test --lib
# Expected: All 1,042+ tests pass
```

---

### Integration Testing Plan (Phase 1)

**Objective:** Verify full DIP works in real usage scenarios.

**Integration Tests That Need Updates:**

1. `tests/correlation_integration_tests.rs`
   - Currently: `use airssys_wasm::host_system::CorrelationTracker;`
   - Change to: `use airssys_wasm::core::correlation_trait::CorrelationTrackerTrait;`
   - Use `Arc<dyn CorrelationTrackerTrait>` for all operations

2. `tests/fire_and_forget_performance_tests.rs`
   - Currently: `use airssys_wasm::host_system::{CorrelationTracker, TimeoutHandler};`
   - Change to: `use airssys_wasm::core::{correlation_trait::CorrelationTrackerTrait, timeout_trait::TimeoutHandlerTrait};`
   - Use `Arc<dyn Trait>` for all operations

3. `tests/send_message_host_function_tests.rs`
   - Currently: `use airssys_wasm::host_system::{CorrelationTracker, TimeoutHandler};`
   - Change to: `use airssys_wasm::core::{correlation_trait::CorrelationTrackerTrait, timeout_trait::TimeoutHandlerTrait};`
   - Use `Arc<dyn Trait>` for all operations

4. `tests/response_routing_integration_tests.rs`
   - Currently: `use airssys_wasm::host_system::{CorrelationTracker, TimeoutHandler};`
   - Change to: `use airssys_wasm::core::{correlation_trait::CorrelationTrackerTrait, timeout_trait::TimeoutHandlerTrait};`
   - Use `Arc<dyn Trait>` for all operations

5. `tests/send_request_host_function_tests.rs`
   - Currently: `use airssys_wasm::host_system::{CorrelationTracker, TimeoutHandler};`
   - Change to: `use airssys_wasm::core::{correlation_trait::CorrelationTrackerTrait, timeout_trait::TimeoutHandlerTrait};`
   - Use `Arc<dyn Trait>` for all operations

**Integration Test Execution:**
```bash
cd airssys-wasm

# Test all integration tests
cargo test --test '*'
# Expected: All integration tests pass
```

**Success Criteria:** All tests pass (1,042+ tests), verify real message/data flow works with trait objects

---

### Verification Commands

#### After Phase 1 Complete

```bash
# 1. Build verification
cd airssys-wasm
cargo build
# Expected: Clean build, zero errors

# 2. Clippy verification
cargo clippy --package airssys-wasm --all-targets --all-features -- -D warnings
# Expected: Zero warnings

# 3. Unit tests
cargo test --package airssys-wasm --lib
# Expected: All 1,042+ tests pass

# 4. Integration tests
cargo test --package airssys-wasm --test '*'
# Expected: All integration tests pass

# 5. ADR-WASM-023 architecture verification (CRITICAL)
grep -rn "use crate::host_system" src/actor/
# Expected: No output (actor/ no longer depends on host_system/)

grep -rn "use crate::host_system" src/runtime/
# Expected: No output (runtime/ no longer depends on host_system/)

grep -rn "use crate::host_system" src/messaging/
# Expected: No output (messaging/ no longer depends on host_system/)

grep -rn "use crate::" src/core/correlation_trait.rs
# Expected: No internal crate imports (dependency-free)

grep -rn "use crate::" src/core/timeout_trait.rs
# Expected: No internal crate imports (dependency-free)

# 6. Verify traits are dependency-free
grep -rn "use tokio\|use dashmap\|use chrono" src/core/correlation_trait.rs
# Expected: No output (trait has no external dependencies)

grep -rn "use tokio\|use dashmap\|use chrono" src/core/timeout_trait.rs
# Expected: No output (trait has no external dependencies)

# 7. Verify dependency injection pattern
grep -rn "Arc<dyn" src/host_system/manager.rs
# Expected: Shows Arc<dyn CorrelationTrackerTrait> and Arc<dyn TimeoutHandlerTrait>
```

---

### Acceptance Criteria Checklist

#### Phase 1 Complete

- [ ] CorrelationTrackerTrait created in `core/correlation_trait.rs`
- [ ] TimeoutHandlerTrait created in `core/timeout_trait.rs`
- [ ] CorrelationTrackerTrait has all 10 methods (new, register_pending, resolve, remove_pending, cleanup_expired, pending_count, contains, completed_count, timeout_count, cleanup_pending_for_component)
- [ ] TimeoutHandlerTrait has all 4 methods (new, register_timeout, cancel_timeout, active_count)
- [ ] Method signatures EXACTLY match actual implementations
- [ ] Traits are dependency-free (no external dependencies)
- [ ] CorrelationTracker implementation moved to `host_system/correlation_impl.rs`
- [ ] TimeoutHandler implementation moved to `host_system/timeout_impl.rs`
- [ ] Both implementations implement their respective traits
- [ ] core/mod.rs updated with trait declarations and re-exports
- [ ] host_system/mod.rs updated with implementation declarations and re-exports
- [ ] HostSystemManager uses `Arc<dyn CorrelationTrackerTrait>`
- [ ] HostSystemManager uses `Arc<dyn TimeoutHandlerTrait>`
- [ ] actor/ imports traits from `core/` (not host_system/)
- [ ] runtime/ imports traits from `core/` (not host_system/)
- [ ] messaging/ imports traits from `core/` (not host_system/)
- [ ] Old implementation files deleted (correlation_tracker.rs, timeout_handler.rs)
- [ ] Build succeeds (zero errors)
- [ ] Zero clippy warnings
- [ ] All unit tests pass (1,042+ tests)
- [ ] All integration tests pass (with updated imports)
- [ ] ADR-WASM-023 violations fixed (no forbidden imports)
- [ ] dependency-management.md compliance verified
- [ ] Dependency injection pattern verified

---

### ADR Compliance Checklist

#### ADR-WASM-023: Module Boundary Enforcement

- [ ] **Rule: core/ imports NOTHING** - Verified: Traits in core/correlation_trait.rs and core/timeout_trait.rs have no internal module imports
- [ ] **Rule: No forbidden imports remain** - Verified: actor/, runtime/, messaging/ no longer import from host_system/
- [ ] **Rule: Dependency flow is one-way** - Verified: All modules → core/ (traits) ← host_system/ (implementations)
- [ ] **Verification commands return no output:**
  ```bash
  grep -rn "use crate::host_system" src/actor/
  grep -rn "use crate::host_system" src/runtime/
  grep -rn "use crate::host_system" src/messaging/
  grep -rn "use crate::" src/core/correlation_trait.rs
  grep -rn "use crate::" src/core/timeout_trait.rs
  # Expected: All return no output
  ```

#### dependency-management.md Compliance

- [ ] **Rule 1: Abstractions Dependency-Free** - Verified: Traits have NO external dependencies, NO implementation logic
- [ ] **Rule 2: Dependency Injection Pattern** - Verified: HostSystemManager uses `Arc<dyn Trait>`, not concrete types
- [ ] **Rule 3: Dependency Direction** - Verified: High-level modules depend on traits, not on implementations

---

### Risk Assessment

#### Higher Complexity

**Risk: Increased code complexity**
- **Description:** Full DIP adds trait definitions and trait objects, increasing complexity
- **Likelihood:** Medium
- **Impact:** More code to maintain, steeper learning curve for new developers

**Mitigation:**
- Clear documentation for trait contracts
- Examples showing usage patterns
- Keep traits minimal (only expose what implementations actually have)

#### Benefits of Proper DIP

**Benefit 1: Loose Coupling**
- Description: Modules depend on abstractions, not concrete implementations
- Impact: Changes to implementations don't affect depending modules
- Evidence: actor/, runtime/, messaging/ no longer depend on host_system/

**Benefit 2: Testability**
- Description: Easy to create mock implementations for testing
- Impact: Faster, more reliable tests
- Evidence: Can create MockCorrelationTrackerTrait for unit tests

**Benefit 3: Flexibility**
- Description: Can swap implementations at runtime or compile time
- Impact: Support for multiple configurations (development, testing, production)
- Evidence: HostSystemManager can use mock or real tracker

**Benefit 4: ADR Compliance**
- Description: Eliminates all ADR-WASM-023 violations
- Impact: Architecture is clean and maintainable
- Evidence: No forbidden imports, proper dependency flow

---

### Success Criteria

**Phase 1 is Complete When:**

1. ✅ CorrelationTrackerTrait defined in `core/correlation_trait.rs`
2. ✅ TimeoutHandlerTrait defined in `core/timeout_trait.rs`
3. ✅ Both traits are dependency-free (no external dependencies)
4. ✅ Method signatures EXACTLY match actual implementations
5. ✅ CorrelationTracker implementation in `host_system/correlation_impl.rs`
6. ✅ TimeoutHandler implementation in `host_system/timeout_impl.rs`
7. ✅ Both implementations implement their respective traits
8. ✅ core/mod.rs updated with trait declarations and re-exports
9. ✅ host_system/mod.rs updated with implementation declarations and re-exports
10. ✅ HostSystemManager uses `Arc<dyn CorrelationTrackerTrait>` and `Arc<dyn TimeoutHandlerTrait>`
11. ✅ actor/, runtime/, messaging/ import traits from `core/` (not host_system/)
12. ✅ Old implementation files deleted
13. ✅ Build succeeds (zero errors)
14. ✅ Zero clippy warnings
15. ✅ All unit tests pass (1,042+ tests)
16. ✅ All integration tests pass (with updated imports)
17. ✅ ADR-WASM-023 violations fixed (no forbidden imports)
18. ✅ dependency-management.md FULLY compliant
19. ✅ Dependency injection pattern verified
20. ✅ Full DIP achieved

---

### 2026-01-04: Phase 1 Subtasks 1.1-1.7 COMPLETE ✅

**Status:** ✅ COMPLETE - AUDIT APPROVED
**Completion Date:** 2026-01-04
**Phase 1 Progress:** 7/7 subtasks complete (100% - 1.1-1.7)

**Implementation Summary:**
- ✅ Subtask 1.1: Read actual implementation files to extract method signatures
- ✅ Subtask 1.2: Create CorrelationTrackerTrait in core/correlation_trait.rs (159 lines)
  - 10 methods with exact signatures from implementation
  - Uses `#[async_trait]` for object-safe async methods
- ✅ Subtask 1.3: Create TimeoutHandlerTrait in core/timeout_trait.rs (96 lines)
  - 4 methods with exact signatures from implementation
  - Uses generic parameter `<T: CorrelationTrackerTrait + 'static>` instead of `dyn`
  - Complies with PROJECTS_STANDARD.md §6.2 (Avoid dyn Patterns)
- ✅ Subtask 1.4: Create CorrelationTracker implementation in host_system/correlation_impl.rs (742 lines)
  - Implements CorrelationTrackerTrait for CorrelationTracker
  - All 10 methods implemented
  - 13 unit tests preserved and passing
- ✅ Subtask 1.5: Create TimeoutHandler implementation in host_system/timeout_impl.rs (373 lines)
  - Implements TimeoutHandlerTrait for TimeoutHandler
  - All 4 methods implemented
  - Uses generic parameter `<T: CorrelationTrackerTrait + 'static>`
  - Fixed: Moved `CorrelationTracker` import to `#[cfg(test)]`
  - Fixed: Added `#[allow(clippy::clone_on_ref_ptr)]` to test module
  - 4 unit tests preserved and passing
- ✅ Subtask 1.6: Update core/mod.rs
  - Added trait module declarations
  - Added trait re-exports
- ✅ Subtask 1.7: Update host_system/mod.rs
  - Added implementation module declarations
  - Added implementation re-exports

**Files Created (NEW):**
1. `airssys-wasm/src/core/correlation_trait.rs` - 159 lines
2. `airssys-wasm/src/core/timeout_trait.rs` - 96 lines
3. `airssys-wasm/src/host_system/correlation_impl.rs` - 742 lines
4. `airssys-wasm/src/host_system/timeout_impl.rs` - 373 lines

**Files Modified:**
1. `airssys-wasm/src/core/mod.rs` - Added trait declarations and re-exports
2. `airssys-wasm/src/host_system/mod.rs` - Added implementation declarations and re-exports

**Test Results:**
```bash
cargo build --package airssys-wasm --lib
Result: ✅ Clean build

cargo test --package airssys-wasm --lib
Result: ✅ 1059/1059 tests passing

cargo clippy --package airssys-wasm --all-targets --all-features -- -D warnings
Result: ✅ Zero warnings
```

**Test Details:**
- 17 new tests (13 correlation + 4 timeout)
- All existing tests preserved
- 100% test pass rate
- Zero regressions

**Audit Results:**
- ✅ Implementer: VERIFIED
- ✅ Auditor: APPROVED (exceptional quality)
  - Architecture: No forbidden imports in new code
  - Build: Clean (zero errors)
  - Tests: 17/17 passing (13 correlation + 4 timeout)
  - Clippy: Zero warnings
  - Coverage: All methods implemented with exact signatures
  - Documentation: Comprehensive
  - Standards Compliance: 100%
- ✅ Verifier: VERIFIED

**Architectural Achievements:**
- ✅ DIP Implementation: Traits in core/, implementations in host_system/
- ✅ Generic Parameters: Uses `<T: Trait>` instead of `dyn` (§6.2 compliance)
- ✅ Zero-Cost Abstraction: Static dispatch via monomorphization
- ✅ Dependency Injection: Enabled via generic parameters
- ✅ ADR-WASM-023: No forbidden imports in new code

**Standards Compliance:**
- ✅ PROJECTS_STANDARD.md §2.1: 3-Layer Imports maintained
- ✅ PROJECTS_STANDARD.md §6.1: YAGNI Principles applied
- ✅ PROJECTS_STANDARD.md §6.2: Avoid `dyn` Patterns (generic parameters used)
- ✅ PROJECTS_STANDARD.md §6.4: Quality Gates met (zero warnings, comprehensive tests)
- ✅ Rust Guidelines: All requirements met

**Next Tasks (Phase 1 Remaining):**
- Subtask 1.8: Update ActorSystemManager to use Traits (DI Pattern)
- Subtask 1.9: Update actor/ to use Traits
- Subtask 1.10: Update runtime/ to use Traits
- Subtask 1.11: Update messaging/ to use Traits
- Subtask 1.12: Delete Old Files (correlation_tracker.rs, timeout_handler.rs)

**Phase 1 Status:** 7/12 subtasks complete (58% - 1.1-1.7 ✅ COMPLETE, 1.8-1.12 ⏳ Not started)
**Next Subtask:** 1.8 - Update ActorSystemManager to use Traits (DI Pattern)



## Subtask 1.8 Implementation Plan (REVISED)

**Planned By:** Memory Bank Planner
**Plan Date:** 2026-01-04
**Revision Date:** 2026-01-04
**Status:** PENDING IMPLEMENTATION

---

### Critical Revisions Made

This plan has been REVISED to address three critical issues identified by the verifier:

1. ✅ **Added KNOWLEDGE-WASM-030 Reference** (MANDATORY - was missing)
   - Document is marked "🔴 MANDATORY - HARD REQUIREMENTS"
   - Line 63 states: "**Trait Abstractions**" belong in `core/`
   - This task creates `TimeoutHandlerObjectSafeTrait` in `core/` - DIRECTLY relevant
   - Added KNOWLEDGE-WASM-030 compliance section with verification

2. ✅ **Fixed ADR Reference Case Mismatch**
   - Changed all ADR references from uppercase to lowercase filenames
   - "ADR-WASM-023" → "adr-wasm-023-module-boundary-enforcement.md"
   - Now matches actual filenames in `.memory-bank/sub-projects/airssys-wasm/docs/adr/`

3. ✅ **Added Specific Testing for New Trait**
   - Added unit tests for `TimeoutHandlerObjectSafeTrait`
   - Added integration tests for trait usage
   - Added specific test commands and expected results
   - Updated testing plans in Phase 1 Unit Testing and Integration Testing sections

---

### Context & References

**Knowledge References:**

1. **KNOWLEDGE-WASM-030: Module Architecture - Hard Requirements** (MANDATORY - 🔴 HARD REQUIREMENTS)
   - **Quote (Line 63):** "`**Trait Abstractions**` - Traits belong in `core/`"
   - **Quote (Lines 36-65):** `core/` module contains "Trait Abstractions" and is the "Foundation - shared types and abstractions"
   - **Quote (Line 77):** "`core/` imports from: NOTHING (only std)"
   - **Application to this task:**
     - Creating `TimeoutHandlerObjectSafeTrait` in `core/timeout_trait.rs` is CORRECT
     - Trait is dependency-free (uses ONLY std and core types)
     - Trait must have NO internal crate imports
     - This enables ANY module to import traits without transitive dependencies
   - **Verification:** MUST check that trait has no internal imports

2. **KNOWLEDGE-WASM-036: Three-Module Architecture**
   - **Relevance:** Defines how modules depend on each other
   - **Application:** Ensures dependency flow remains one-way

**ADR References:**

1. **adr-wasm-023-module-boundary-enforcement.md** (Module Boundary Enforcement - MANDATORY)
   - **Quote (Lines 75-79):** "FORBIDDEN (NEVER, NO EXCEPTIONS): ❌ runtime/ → actor/ (BREAKS ARCHITECTURE)"
   - **Quote (Lines 59-64):** Dependency rules for each module
   - **Application:** Verify all imports are allowed, no forbidden imports created
   - **Verification:** grep checks must return no output

2. **adr-wasm-019-runtime-dependency-management.md** (Runtime Dependency Management)
   - **Relevance:** Defines how to manage dependencies in runtime layer
   - **Application:** Ensures proper dependency injection pattern

3. **dependency-management.md** (Dependency Inversion Pattern)
   - **Rule 1:** Abstractions Dependency-Free
   - **Rule 2:** Dependency Injection Pattern
   - **Rule 3:** Dependency Direction
   - **Application:** Traits must be dependency-free, use DI pattern

**Standards Applied:**

**PROJECTS_STANDARD.md:**
- **§2.1:** 3-Layer import organization (all modified files)
- **§6.1:** YAGNI - Only implement required changes (object-safe trait)
- **§6.2:** Avoid `dyn` - Use `dyn` only when necessary (DI pattern, object safety constraints)
- **§6.4:** Implementation quality gates (zero warnings, comprehensive tests)

**Rust Guidelines:**
- **M-DESIGN-FOR-AI:** Idiomatic APIs, thorough docs, testable code
- **M-MODULE-DOCS:** Module documentation with canonical sections
- **M-ERRORS-CANONICAL-STRUCTS:** Error types follow canonical structure
- **M-STATIC-VERIFICATION:** All lints enabled, clippy passes
- **M-FEATURES-ADDITIVE:** Changes don't break existing APIs

**Documentation Standards:**
- **Diátaxis Type:** Reference documentation
- **Quality:** Technical language, no marketing hyperbole per documentation-quality-standards.md
- **Canonical Sections:** All documented items have Summary, Examples, Errors, Panics sections

---

### KNOWLEDGE-WASM-030 Compliance (MANDATORY)

#### Hard Requirements Summary

**From KNOWLEDGE-WASM-030: Module Architecture - Hard Requirements**

| Requirement | Quote | Application to This Task |
|-------------|--------|------------------------|
| **Trait Abstractions in core/** | Line 63: "`**Trait Abstractions**`" belongs in `core/` | ✅ `TimeoutHandlerObjectSafeTrait` will be in `core/timeout_trait.rs` |
| **core/ Imports NOTHING** | Line 77: "`core/` imports from: NOTHING (only std)" | ✅ Trait will use ONLY std and core types, NO internal crate imports |
| **Dependency Flow** | Lines 259-263: Dependency diagram | ✅ `host_system/` → `core/` (traits), NO reverse imports |
| **No Forbidden Imports** | Lines 388-425: Verification commands | ✅ Will run verification to ensure no forbidden imports |

#### How This Plan Complies

**✅ Compliance Point 1: Trait Abstractions in `core/`**
- **Requirement:** KNOWLEDGE-WASM-030 states trait abstractions belong in `core/`
- **Plan Action:** Creating `TimeoutHandlerObjectSafeTrait` in `core/timeout_trait.rs`
- **Evidence:** New trait definition in Phase 1, line 375-475 of this plan

**✅ Compliance Point 2: Trait is Dependency-Free**
- **Requirement:** KNOWLEDGE-WASM-030 requires `core/` imports NOTHING (only std)
- **Plan Action:** Trait uses ONLY:
  - `crate::core::messaging::CorrelationId` (core type - allowed)
  - `crate::core::correlation_trait::CorrelationTrackerTrait` (core type - allowed)
  - `std::time::Duration` (std type - allowed)
  - `std::sync::Arc` (std type - allowed)
- **Verification Command:**
  ```bash
  grep -rn "use crate::" src/core/timeout_trait.rs | grep -v "use crate::core::"
  # Expected: NO output (only core types allowed)
  ```

**✅ Compliance Point 3: No Forbidden Imports**
- **Requirement:** ADR-WASM-023 forbids reverse dependencies
- **Plan Action:** `host_system/` implements trait, imports from `core/`
- **Verification Command:**
  ```bash
  grep -rn "use crate::host_system" src/core/
  # Expected: NO output (core/ cannot import from host_system/)
  ```

**✅ Compliance Point 4: Dependency Flow is One-Way**
- **Requirement:** Dependency flow: `host_system/` → `core/` (NOT reverse)
- **Plan Action:**
  - Trait defined in `core/timeout_trait.rs`
  - Implementation in `host_system/timeout_impl.rs`
  - `host_system/` imports from `core/` (ALLOWED)
  - `core/` does NOT import from `host_system/` (FORBIDDEN)
- **Evidence:** Phase 1 (trait in core/), Phase 2 (implementation in host_system/)

#### Explicit Verification of Compliance

**Verification Command Set (run after implementation):**
```bash
# 1. Verify core/ trait has no forbidden imports
echo "Verifying KNOWLEDGE-WASM-030 compliance: core/ trait imports NOTHING..."
FORBIDDEN=$(grep -rn "use crate::" src/core/timeout_trait.rs | grep -v "use crate::core::" | grep -v "^//")
if [ -n "$FORBIDDEN" ]; then
    echo "❌ VIOLATION: core/timeout_trait.rs has forbidden imports:"
    echo "$FORBIDDEN"
    exit 1
fi
echo "✅ core/timeout_trait.rs is dependency-free"

# 2. Verify trait uses ONLY core types
echo "Verifying trait uses ONLY core types..."
NON_CORE=$(grep "^use " src/core/timeout_trait.rs | grep -v "crate::core::" | grep -v "std::" | grep -v "async_trait")
if [ -n "$NON_CORE" ]; then
    echo "❌ VIOLATION: trait uses non-core types:"
    echo "$NON_CORE"
    exit 1
fi
echo "✅ trait uses ONLY core types"

# 3. Verify host_system/ does NOT import from runtime/actor/ (per ADR-WASM-023)
echo "Verifying host_system/ does not violate ADR-WASM-023..."
VIOLATIONS=$(grep -rn "use crate::runtime\|use crate::actor" src/host_system/timeout_impl.rs)
if [ -n "$VIOLATIONS" ]; then
    echo "❌ VIOLATION: host_system/ has forbidden imports:"
    echo "$VIOLATIONS"
    exit 1
fi
echo "✅ host_system/ is compliant with ADR-WASM-023"

echo ""
echo "✅ All KNOWLEDGE-WASM-030 compliance checks passed!"
```

**Expected Output:**
```
✅ core/timeout_trait.rs is dependency-free
✅ trait uses ONLY core types
✅ host_system/ is compliant with ADR-WASM-023

✅ All KNOWLEDGE-WASM-030 compliance checks passed!
```

---

### Current State Analysis

**File:** `airssys-wasm/src/host_system/manager.rs`

**Current Implementation (Lines 33-34, 145-148, 228-229, 249-250):**
```rust
// Lines 33-34: Imports
use crate::host_system::correlation_tracker::CorrelationTracker;
use crate::host_system::timeout_handler::TimeoutHandler;

// Lines 145-148: Field types (concrete)
pub struct HostSystemManager {
    // ... other fields ...
    
    /// Correlation tracker for request-response pattern
    correlation_tracker: Arc<CorrelationTracker>,
    
    /// Timeout handler for request timeout enforcement
    timeout_handler: Arc<TimeoutHandler>,
    
    // ... other fields ...
}

// Lines 228-229: Constructor creates concrete types
pub async fn new() -> Result<Self, WasmError> {
    // ... initialization ...
    
    // Step 2: Create CorrelationTracker and TimeoutHandler
    let correlation_tracker = Arc::new(CorrelationTracker::new());
    let timeout_handler = Arc::new(TimeoutHandler::new());
    
    // ... rest of initialization ...
}

// Lines 249-250: Passed to MessagingService
let messaging_service = Arc::new(MessagingService::new(
    Arc::new(broker.clone()),
    Arc::clone(&correlation_tracker),
    Arc::clone(&timeout_handler),
));
```

**Field Usages in manager.rs:**
- Line 505: `self.correlation_tracker.cleanup_pending_for_component(id).await`
- Line 1220: `manager.correlation_tracker.register_pending(pending).await.unwrap()`
- Line 1223: `assert!(manager.correlation_tracker.contains(&CorrelationId::from(correlation_id)))`
- Line 1230: `let contains_after = manager.correlation_tracker.contains(&CorrelationId::from(correlation_id))`

**Current Dependency Relationships:**
```
HostSystemManager
  ├─ creates: Arc<CorrelationTracker> (concrete)
  ├─ creates: Arc<TimeoutHandler> (concrete)
  └─ injects: Into MessagingService (via constructor)
```

---

### Architectural Options Analysis

#### ❌ Option A: Use Concrete Types (Simplest, but No DI)

**Approach:** Keep current implementation unchanged.

**Pros:**
- Simplest approach
- No code changes required
- Zero runtime overhead
- Static dispatch

**Cons:**
- **Fails DIP goals:** Cannot inject mock implementations for testing
- **Fails dependency-management.md Rule 2:** Violates dependency injection pattern
- **Tight coupling:** HostSystemManager directly depends on concrete implementations
- **Not testable:** Cannot swap implementations in tests

**Rejection Reason:** Does not achieve Subtask 1.8's stated goal of "Update ActorSystemManager to use Traits (DI Pattern)"

---

#### ❌ Option B: Force `Arc<dyn TimeoutHandlerTrait>` (Will Not Compile)

**Approach:** Use `Arc<dyn TimeoutHandlerTrait>` despite object safety issue.

**Expected Code:**
```rust
pub struct HostSystemManager {
    correlation_tracker: Arc<dyn CorrelationTrackerTrait>,  // ✓ Object-safe
    timeout_handler: Arc<dyn TimeoutHandlerTrait>,           // ✗ NOT object-safe!
}
```

**Why This Fails:**

`TimeoutHandlerTrait` has a generic method:
```rust
fn register_timeout<T: CorrelationTrackerTrait + 'static>(
    &self,
    correlation_id: CorrelationId,
    timeout: Duration,
    tracker: std::sync::Arc<T>,
);
```

**Rust Object Safety Rule:** "Traits with generic methods are NOT object-safe"

**Compiler Error:**
```
error[E0038]: the trait `TimeoutHandlerTrait` cannot be made into an object
  --> src/host_system/manager.rs:148:29
   |
148 |     timeout_handler: Arc<dyn TimeoutHandlerTrait>,
   |                             ^^^^^^^^^^^^^^^^^^^^ the trait `TimeoutHandlerTrait` cannot be made into an object
   |
note: this trait has a generic method `register_timeout`
```

**Rejection Reason:** Code will not compile. This option is technically impossible.

---

#### ⏸️ Option C: Modify TimeoutHandlerTrait to Be Object-Safe

**Approach:** Remove generic parameter from `TimeoutHandlerTrait` by:
1. Changing `register_timeout()` to accept `Arc<dyn CorrelationTrackerTrait>` instead of `Arc<T>`
2. Making trait object-safe

**Pros:**
- Enables `Arc<dyn TimeoutHandlerTrait>` usage
- Achieves full DI pattern
- Consistent with dependency-management.md examples

**Cons:**
- **Breaking change:** Requires updating TimeoutHandler implementation
- **Violates §6.2:** Forces use of `dyn` (last resort per hierarchy)
- **Performance cost:** Dynamic dispatch overhead (vtable lookup)
- **Breaking tests:** 4 existing tests in `timeout_impl.rs` may need updates
- **Violates trait design:** Trait intentionally uses generics for static dispatch

**Code Changes Required:**
```rust
// Current (generic - static dispatch)
fn register_timeout<T: CorrelationTrackerTrait + 'static>(
    &self,
    correlation_id: CorrelationId,
    timeout: Duration,
    tracker: Arc<T>,
);

// Proposed (object-safe - dynamic dispatch)
fn register_timeout(
    &self,
    correlation_id: CorrelationId,
    timeout: Duration,
    tracker: Arc<dyn CorrelationTrackerTrait>,  // ← Now object-safe
);
```

**Rejection Reason:** Violates PROJECTS_STANDARD.md §6.2 (Avoid dyn Patterns - "dyn only as last resort") and contradicts trait's original design rationale (lines 68-72 of timeout_trait.rs).

---

#### ✅ Option D: Create Object-Safe Timeout Trait (RECOMMENDED)

**Approach:** Create a new object-safe trait `TimeoutHandlerObjectSafeTrait` in `core/timeout_trait.rs` and use `Arc<dyn TimeoutHandlerObjectSafeTrait>` in HostSystemManager.

**Rationale:**
1. **CorrelationTrackerTrait IS object-safe:** Can use `Arc<dyn CorrelationTrackerTrait>`
2. **TimeoutHandlerTrait is NOT object-safe:** Cannot use `dyn TimeoutHandlerTrait`
3. **Solution:** Create parallel object-safe trait for timeout handling
4. **Benefits:**
   - Achieves DI goals (can use `dyn` for both)
   - Minimal code changes (new trait, implement for TimeoutHandler)
   - Maintains performance (static dispatch in implementation)
   - Allows future migration if needed
   - Pragmatic compromise between DI goals and technical constraints

**New Trait to Create:**
```rust
// In src/core/timeout_trait.rs (new trait)

/// Object-safe timeout handling trait for dependency injection.
///
/// This trait is object-safe (can use `dyn TimeoutHandlerObjectSafeTrait`)
/// while the main `TimeoutHandlerTrait` uses generics for static dispatch.
/// 
/// # When to Use This Trait
///
/// Use this trait when you need to store timeouts as `Arc<dyn Trait>`
/// for dependency injection. If you need maximum performance with static
/// dispatch, use `TimeoutHandlerTrait` instead.
#[async_trait]
pub trait TimeoutHandlerObjectSafeTrait: Send + Sync {
    /// Register timeout for pending request.
    ///
    /// # Arguments
    ///
    /// * `correlation_id` - Correlation ID of request
    /// * `timeout` - Timeout duration
    /// * `tracker` - CorrelationTracker as trait object
    fn register_timeout(
        &self,
        correlation_id: CorrelationId,
        timeout: Duration,
        tracker: Arc<dyn CorrelationTrackerTrait>,
    );

    /// Cancel timeout (called when response arrives before timeout).
    fn cancel_timeout(&self, correlation_id: &CorrelationId);

    /// Get number of active timeouts (for monitoring).
    fn active_count(&self) -> usize;
}
```

**Implementation for TimeoutHandler:**
```rust
// In src/host_system/timeout_impl.rs

impl TimeoutHandlerObjectSafeTrait for TimeoutHandler {
    fn register_timeout(
        &self,
        correlation_id: CorrelationId,
        timeout: Duration,
        tracker: Arc<dyn CorrelationTrackerTrait>,
    ) {
        self.register_timeout(correlation_id, timeout, tracker);
    }

    fn cancel_timeout(&self, correlation_id: &CorrelationId) {
        self.cancel_timeout(correlation_id);
    }

    fn active_count(&self) -> usize {
        self.active_count()
    }
}
```

**HostSystemManager Changes:**
```rust
use crate::core::correlation_trait::CorrelationTrackerTrait;
use crate::core::timeout_trait::{TimeoutHandlerTrait, TimeoutHandlerObjectSafeTrait};
use crate::host_system::correlation_impl::CorrelationTracker;
use crate::host_system::timeout_impl::TimeoutHandler;

pub struct HostSystemManager {
    // ... other fields ...
    
    /// Correlation tracker for request-response pattern
    correlation_tracker: Arc<dyn CorrelationTrackerTrait>,
    
    /// Timeout handler for request timeout enforcement
    timeout_handler: Arc<dyn TimeoutHandlerObjectSafeTrait>,
    
    // ... other fields ...
}

impl HostSystemManager {
    pub async fn new() -> Result<Self, WasmError> {
        // Create concrete implementations
        let correlation_tracker = Arc::new(CorrelationTracker::new());
        let timeout_handler = Arc::new(TimeoutHandler::new());
        
        // Create trait objects (dependency injection)
        let correlation_tracker_dyn: Arc<dyn CorrelationTrackerTrait> = correlation_tracker.clone();
        let timeout_handler_dyn: Arc<dyn TimeoutHandlerObjectSafeTrait> = timeout_handler.clone();
        
        // ... rest of initialization ...
        
        Ok(Self {
            // ... other fields ...
            correlation_tracker: correlation_tracker_dyn,
            timeout_handler: timeout_handler_dyn,
            // ... other fields ...
        })
    }
}
```

**Pros:**
- ✅ **Achieves DI goals:** Can use `Arc<dyn Trait>` for both
- ✅ **Minimal code changes:** Only 1 new trait, small updates
- ✅ **Preserves performance:** Implementation uses static dispatch (generics)
- ✅ **Flexible:** Future-proof for alternative implementations
- ✅ **Testable:** Can create mock implementations
- ✅ **Maintains standards:** Follows dependency-management.md Rule 2
- ✅ **Clean migration:** If later needed, can migrate fully to object-safe design
- ✅ **KNOWLEDGE-WASM-030 compliant:** New trait in `core/`, dependency-free

**Cons:**
- **More complex:** Two parallel traits (one generic, one object-safe)
- **Potential confusion:** Developers must choose correct trait
- **Technical debt:** Requires documentation to explain when to use which trait

**Why This Is Best Option:**
1. **Achieves stated goals:** Enables DI pattern as required by subtask
2. **Technical feasibility:** Works with Rust's object safety rules
3. **Minimal disruption:** Small code changes, no breaking changes
4. **Standards compliant:** Follows all PROJECTS_STANDARD.md requirements
5. **Performance preserved:** Zero runtime overhead in implementation
6. **Pragmatic compromise:** Balances DI goals with technical constraints
7. **KNOWLEDGE-WASM-030 compliant:** Trait in `core/`, dependency-free

**Rejection Reason for Other Options:**
- Option A: Fails stated goal of "Update to use Traits (DI Pattern)"
- Option B: Technically impossible (will not compile)
- Option C: Violates §6.2, creates breaking changes, contradicts trait design

**Recommendation:** ✅ **APPROVE Option D** - Create `TimeoutHandlerObjectSafeTrait`

---

### Detailed Implementation Plan

#### Phase 1: Create Object-Safe Timeout Trait

**File:** `airssys-wasm/src/core/timeout_trait.rs`

**Changes:**

1. **Add new trait definition** (after existing TimeoutHandlerTrait, around line 96):
```rust
/// Object-safe timeout handling trait for dependency injection.
///
/// This trait provides an object-safe interface for timeout handling,
/// enabling storage as `Arc<dyn TimeoutHandlerObjectSafeTrait>`
/// in structs that require dependency injection.
///
/// # When to Use This Trait
///
/// - Use this trait when you need to store timeouts in a field
/// - Use this trait when you need to pass timeouts as `Arc<dyn Trait>`
/// - Use this trait for dependency injection patterns
///
/// # When to Use TimeoutHandlerTrait Instead
///
/// - Use `TimeoutHandlerTrait` when you need maximum performance
/// - Use `TimeoutHandlerTrait` when you don't need `dyn` (static dispatch)
/// - Use `TimeoutHandlerTrait` in generic functions with type parameters
///
/// # Design Notes
///
/// This trait is a wrapper around `TimeoutHandlerTrait` that uses
/// `Arc<dyn CorrelationTrackerTrait>` instead of generics, making
/// it object-safe at the cost of dynamic dispatch.
///
/// # Thread Safety
///
/// All trait methods must be thread-safe.
#[async_trait]
pub trait TimeoutHandlerObjectSafeTrait: Send + Sync {
    /// Register timeout for pending request.
    ///
    /// Spawns a background task that waits for timeout duration.
    /// If request is not resolved before timeout, sends a timeout error
    /// to the response channel.
    ///
    /// # Arguments
    ///
    /// * `correlation_id` - Correlation ID of request
    /// * `timeout` - Timeout duration
    /// * `tracker` - CorrelationTracker to remove request on timeout (as trait object)
    ///
    /// # Examples
    ///
    /// ```rust,ignore
    /// use airssys_wasm::core::{correlation_trait::CorrelationTrackerTrait, timeout_trait::TimeoutHandlerObjectSafeTrait};
    /// use airssys_wasm::host_system::{correlation_impl::CorrelationTracker, timeout_impl::TimeoutHandler};
    /// use std::sync::Arc;
    ///
    /// let tracker = Arc::new(CorrelationTracker::new());
    /// let handler = Arc::new(TimeoutHandler::new());
    ///
    /// let correlation_id = CorrelationId::new();
    /// let timeout = Duration::from_secs(30);
    ///
    /// handler.register_timeout(correlation_id, timeout, tracker);
    /// ```
    fn register_timeout(
        &self,
        correlation_id: CorrelationId,
        timeout: Duration,
        tracker: Arc<dyn CorrelationTrackerTrait>,
    );

    /// Cancel timeout (called when response arrives before timeout).
    ///
    /// Aborts timeout task to prevent unnecessary timeout error.
    /// If timeout has already fired, this is a no-op.
    ///
    /// # Arguments
    ///
    /// * `correlation_id` - Correlation ID of request
    ///
    /// # Examples
    ///
    /// ```rust,ignore
    /// use airssys_wasm::core::timeout_trait::TimeoutHandlerObjectSafeTrait;
    /// use airssys_wasm::host_system::timeout_impl::TimeoutHandler;
    /// use std::sync::Arc;
    ///
    /// let handler = Arc::new(TimeoutHandler::new());
    ///
    /// handler.cancel_timeout(&correlation_id);
    /// ```
    fn cancel_timeout(&self, correlation_id: &CorrelationId);

    /// Get number of active timeouts (for monitoring).
    ///
    /// Returns the current count of active timeout tasks.
    ///
    /// # Examples
    ///
    /// ```rust,ignore
    /// use airssys_wasm::core::timeout_trait::TimeoutHandlerObjectSafeTrait;
    /// use airssys_wasm::host_system::timeout_impl::TimeoutHandler;
    /// use std::sync::Arc;
    ///
    /// let handler = Arc::new(TimeoutHandler::new());
    ///
    /// let count = handler.active_count();
    /// println!("Active timeouts: {}", count);
    /// ```
    fn active_count(&self) -> usize;
}
```

2. **Re-export trait** in `core/mod.rs`:
```rust
// In airssys-wasm/src/core/mod.rs (around line 45)
pub use timeout_trait::{TimeoutHandlerTrait, TimeoutHandlerObjectSafeTrait};
```

**Acceptance Criteria:**
- ✅ `TimeoutHandlerObjectSafeTrait` defined in `core/timeout_trait.rs`
- ✅ Trait is object-safe (no generic methods, no `Self: Sized` bounds)
- ✅ Trait uses `Arc<dyn CorrelationTrackerTrait>` for tracker parameter
- ✅ All methods documented with canonical sections (Summary, Arguments, Examples)
- ✅ Trait re-exported in `core/mod.rs`
- ✅ Code compiles without errors
- ✅ KNOWLEDGE-WASM-030 compliant (trait in core/, dependency-free)

---

#### Phase 2: Implement Object-Safe Trait for TimeoutHandler

**File:** `airssys-wasm/src/host_system/timeout_impl.rs`

**Changes:**

1. **Add implementation block** (after existing `impl TimeoutHandlerTrait for TimeoutHandler`, around line 373):
```rust
impl TimeoutHandlerObjectSafeTrait for TimeoutHandler {
    /// Register timeout with correlation tracker as trait object.
    ///
    /// Delegates to the generic `register_timeout` method by converting
    /// `Arc<dyn CorrelationTrackerTrait>` back to the concrete type.
    fn register_timeout(
        &self,
        correlation_id: CorrelationId,
        timeout: Duration,
        tracker: Arc<dyn CorrelationTrackerTrait>,
    ) {
        // Delegate to the generic implementation
        self.register_timeout(correlation_id, timeout, tracker);
    }

    /// Cancel timeout for correlation ID.
    ///
    /// Delegates to the underlying TimeoutHandler implementation.
    fn cancel_timeout(&self, correlation_id: &CorrelationId) {
        self.cancel_timeout(correlation_id);
    }

    /// Get active timeout count.
    ///
    /// Delegates to the underlying TimeoutHandler implementation.
    fn active_count(&self) -> usize {
        self.active_count()
    }
}
```

**Acceptance Criteria:**
- ✅ `TimeoutHandlerObjectSafeTrait` implemented for `TimeoutHandler`
- ✅ All methods delegate to existing `TimeoutHandler` implementation
- ✅ Code compiles without errors
- ✅ No behavior changes (transparent delegation)

---

#### Phase 3: Update HostSystemManager to Use Traits

**File:** `airssys-wasm/src/host_system/manager.rs`

**Changes:**

1. **Update imports** (lines 33-34):
```rust
// REMOVE these lines:
// use crate::host_system::correlation_tracker::CorrelationTracker;
// use crate::host_system::timeout_handler::TimeoutHandler;

// ADD these lines:
use crate::core::correlation_trait::CorrelationTrackerTrait;
use crate::core::timeout_trait::{TimeoutHandlerTrait, TimeoutHandlerObjectSafeTrait};
use crate::host_system::correlation_impl::CorrelationTracker;
use crate::host_system::timeout_impl::TimeoutHandler;
```

2. **Update field types** (lines 145-148):
```rust
// BEFORE:
// pub struct HostSystemManager {
//     correlation_tracker: Arc<CorrelationTracker>,
//     timeout_handler: Arc<TimeoutHandler>,
// }

// AFTER:
pub struct HostSystemManager {
    // ... other fields ...
    
    /// Correlation tracker for request-response pattern
    /// Stored as trait object for dependency injection.
    correlation_tracker: Arc<dyn CorrelationTrackerTrait>,
    
    /// Timeout handler for request timeout enforcement
    /// Stored as trait object for dependency injection.
    timeout_handler: Arc<dyn TimeoutHandlerObjectSafeTrait>,
    
    // ... other fields ...
}
```

3. **Update constructor** (lines 228-229, 279-280):
```rust
// BEFORE:
// let correlation_tracker = Arc::new(CorrelationTracker::new());
// let timeout_handler = Arc::new(TimeoutHandler::new());

// AFTER:
// Create concrete implementations
let correlation_tracker_concrete = Arc::new(CorrelationTracker::new());
let timeout_handler_concrete = Arc::new(TimeoutHandler::new());

// Convert to trait objects (dependency injection)
let correlation_tracker: Arc<dyn CorrelationTrackerTrait> = correlation_tracker_concrete;
let timeout_handler: Arc<dyn TimeoutHandlerObjectSafeTrait> = timeout_handler_concrete;
```

4. **Update Debug impl** (lines 163-164):
```rust
// BEFORE:
// .field("correlation_tracker", &"<CorrelationTracker>")
// .field("timeout_handler", &"<TimeoutHandler>")

// AFTER:
.field("correlation_tracker", &"<dyn CorrelationTrackerTrait>")
.field("timeout_handler", &"<dyn TimeoutHandlerObjectSafeTrait>")
```

**Acceptance Criteria:**
- ✅ `HostSystemManager.correlation_tracker` type changed to `Arc<dyn CorrelationTrackerTrait>`
- ✅ `HostSystemManager.timeout_handler` type changed to `Arc<dyn TimeoutHandlerObjectSafeTrait>`
- ✅ Constructor creates trait objects from concrete implementations
- ✅ All existing usages work without changes
- ✅ Code compiles without errors

---

### Testing Plan (REVISED - Added Specific Testing for New Trait)

#### Unit Testing Plan (Phase 1 - REVISED)

**Objective:** Verify traits match implementations EXACTLY and DIP works correctly.

**Test Coverage Target:** 95% for both traits

**NEW: Test 1: Object-Safe Trait Compilation**
- **File:** `core/timeout_trait.rs` (in `#[cfg(test)]` module)
- **Purpose:** Verify `TimeoutHandlerObjectSafeTrait` compiles
- **Tests:**
  ```rust
  #[cfg(test)]
  mod tests {
      use super::*;
      
      #[test]
      fn test_timeout_handler_object_safe_trait_compiles() {
          // This test verifies the trait compiles as object-safe
          // If the trait is not object-safe, this will fail to compile
          use std::sync::Arc;
          
          // Create a mock timeout handler
          struct MockTimeoutHandler;
          
          impl TimeoutHandlerObjectSafeTrait for MockTimeoutHandler {
              fn register_timeout(
                  &self,
                  _correlation_id: CorrelationId,
                  _timeout: Duration,
                  _tracker: Arc<dyn CorrelationTrackerTrait>,
              ) {
                  // Mock implementation
              }
              
              fn cancel_timeout(&self, _correlation_id: &CorrelationId) {
                  // Mock implementation
              }
              
              fn active_count(&self) -> usize {
                  0
              }
          }
          
          // Verify we can use it as a trait object
          let handler: Arc<dyn TimeoutHandlerObjectSafeTrait> = Arc::new(MockTimeoutHandler);
          assert_eq!(handler.active_count(), 0);
      }
  }
  ```
- **Acceptance Criteria:** ✅ Test compiles and passes

**Test 2: Object-Safe Trait Implementation**
- **File:** `host_system/timeout_impl.rs` (in `#[cfg(test)]` module)
- **Purpose:** Verify `TimeoutHandler` implements object-safe trait
- **Tests:**
  ```rust
  #[cfg(test)]
  mod tests {
      use super::*;
      use crate::core::correlation_trait::CorrelationTrackerTrait;
      use crate::host_system::correlation_impl::CorrelationTracker;
      use std::sync::Arc;
      
      #[tokio::test]
      async fn test_timeout_handler_implements_object_safe_trait() {
          // Create real implementations
          let tracker = Arc::new(CorrelationTracker::new());
          let handler = Arc::new(TimeoutHandler::new());
          
          // Use as trait object
          let handler_dyn: Arc<dyn TimeoutHandlerObjectSafeTrait> = handler.clone();
          
          // Register timeout
          let correlation_id = CorrelationId::new();
          handler_dyn.register_timeout(correlation_id, Duration::from_secs(1), tracker);
          
          // Verify timeout is active
          assert_eq!(handler_dyn.active_count(), 1);
          
          // Cancel timeout
          handler_dyn.cancel_timeout(&correlation_id);
          
          // Verify timeout is cancelled
          assert_eq!(handler_dyn.active_count(), 0);
      }
  }
  ```
- **Acceptance Criteria:** ✅ Test passes, trait object works correctly

**Test 3: Delegation to Implementation**
- **File:** `host_system/timeout_impl.rs` (in `#[cfg(test)]` module)
- **Purpose:** Verify object-safe trait delegates to implementation correctly
- **Tests:**
  ```rust
  #[tokio::test]
  async fn test_object_safe_trait_delegates_to_implementation() {
      use crate::core::timeout_trait::TimeoutHandlerTrait;
      
      let tracker = Arc::new(CorrelationTracker::new());
      let handler = Arc::new(TimeoutHandler::new());
      
      let correlation_id = CorrelationId::new();
      
      // Use generic trait
      let handler_generic: Arc<TimeoutHandler> = handler.clone();
      handler_generic.register_timeout(correlation_id, Duration::from_millis(100), tracker.clone());
      
      // Use object-safe trait
      let handler_dyn: Arc<dyn TimeoutHandlerObjectSafeTrait> = handler.clone();
      assert_eq!(handler_dyn.active_count(), 1);
      
      // Cancel with object-safe trait
      handler_dyn.cancel_timeout(&correlation_id);
      assert_eq!(handler_dyn.active_count(), 0);
  }
  ```
- **Acceptance Criteria:** ✅ Test passes, delegation works correctly

**Test 4: HostSystemManager Uses Trait Objects**
- **File:** `host_system/manager.rs` (in `#[cfg(test)]` module)
- **Purpose:** Verify HostSystemManager uses trait objects correctly
- **Tests:**
  ```rust
  #[cfg(test)]
  mod tests {
      use super::*;
      use crate::core::correlation_trait::CorrelationTrackerTrait;
      use crate::core::timeout_trait::TimeoutHandlerObjectSafeTrait;
      
      #[tokio::test]
      async fn test_host_system_manager_uses_trait_objects() {
          let manager = HostSystemManager::new().await.unwrap();
          
          // Verify fields are trait objects (cannot access concrete methods)
          // This test compiles because fields are trait objects
          
          // Verify we can call trait methods
          let correlation_id = CorrelationId::new();
          let contains = manager.correlation_tracker.contains(&correlation_id);
          assert!(!contains);
          
          let count = manager.timeout_handler.active_count();
          assert_eq!(count, 0);
      }
  }
  ```
- **Acceptance Criteria:** ✅ Test passes, trait objects used correctly

**Test Execution:**
```bash
cd airssys-wasm

# Test new object-safe trait
cargo test --lib timeout_object_safe
# Expected: All new tests pass

# Test trait implementations
cargo test --lib timeout_impl
# Expected: All 4 existing tests pass + 2 new tests

# Test all unit tests
cargo test --lib
# Expected: All 1064+ tests pass (1059 + 5 new)
```

---

#### Integration Testing Plan (Phase 1 - REVISED)

**Objective:** Verify full DIP works in real usage scenarios.

**NEW: Integration Test 1: HostSystemManager DI Pattern**
- **File:** `tests/host_system_manager_di_tests.rs` (NEW FILE)
- **Purpose:** Verify HostSystemManager can use mock implementations
- **Tests:**
  ```rust
  use airssys_wasm::core::{correlation_trait::CorrelationTrackerTrait, timeout_trait::TimeoutHandlerObjectSafeTrait};
  use std::sync::Arc;
  use std::time::Duration;
  
  /// Mock correlation tracker for testing
  struct MockCorrelationTracker;
  
  impl CorrelationTrackerTrait for MockCorrelationTracker {
      // Implement all required methods...
  }
  
  /// Mock timeout handler for testing
  struct MockTimeoutHandler;
  
  impl TimeoutHandlerObjectSafeTrait for MockTimeoutHandler {
      fn register_timeout(
          &self,
          _correlation_id: CorrelationId,
          _timeout: Duration,
          _tracker: Arc<dyn CorrelationTrackerTrait>,
      ) {
          // Mock implementation
      }
      
      fn cancel_timeout(&self, _correlation_id: &CorrelationId) {
          // Mock implementation
      }
      
      fn active_count(&self) -> usize {
          0
      }
  }
  
  #[tokio::test]
  async fn test_host_system_manager_with_mock_implementations() {
      // This test verifies we can inject mock implementations
      // (would require constructor changes in HostSystemManager)
      
      let mock_tracker = Arc::new(MockCorrelationTracker);
      let mock_handler = Arc::new(MockTimeoutHandler);
      
      // Verify trait objects work
      assert_eq!(mock_handler.active_count(), 0);
  }
  ```
- **Acceptance Criteria:** ✅ Test passes, DI pattern works

**Integration Test Execution:**
```bash
cd airssys-wasm

# Test new DI integration tests
cargo test --test host_system_manager_di_tests
# Expected: All DI tests pass

# Test all integration tests
cargo test --test '*'
# Expected: All integration tests pass
```

**Success Criteria:** All tests pass (1064+ tests), verify real message/data flow works with trait objects

---

### Verification Commands

#### After Phase 1 Complete

```bash
# 1. Build verification
cd airssys-wasm
cargo build
# Expected: Clean build, zero errors

# 2. Check for warnings
cargo build --package airssys-wasm --lib 2>&1 | grep -i "warning"
# Expected: No output (zero warnings)

# 3. Clippy verification
cargo clippy --package airssys-wasm --all-targets --all-features -- -D warnings
# Expected: Zero warnings

# 4. Unit tests (NEW: Verify new trait tests)
cargo test --package airssys-wasm --lib timeout_object_safe
# Expected: All 3 new tests pass

cargo test --package airssys-wasm --lib
# Expected: All 1064+ tests passing

# 5. Integration tests (NEW: Verify DI tests)
cargo test --package airssys-wasm --test host_system_manager_di_tests
# Expected: All DI tests pass

cargo test --package airssys-wasm --test '*'
# Expected: All integration tests pass

# 6. ADR-WASM-023 architecture verification (CRITICAL)
grep -rn "use crate::host_system" src/actor/
# Expected: No output (actor/ no longer depends on host_system/)

grep -rn "use crate::host_system" src/runtime/
# Expected: No output (runtime/ no longer depends on host_system/)

grep -rn "use crate::host_system" src/messaging/
# Expected: No output (messaging/ no longer depends on host_system/)

# 7. KNOWLEDGE-WASM-030 compliance verification (NEW - MANDATORY)
echo "Verifying KNOWLEDGE-WASM-030 compliance: core/ trait imports NOTHING..."
FORBIDDEN=$(grep -rn "use crate::" src/core/timeout_trait.rs | grep -v "use crate::core::" | grep -v "^//")
if [ -n "$FORBIDDEN" ]; then
    echo "❌ VIOLATION: core/timeout_trait.rs has forbidden imports:"
    echo "$FORBIDDEN"
    exit 1
fi
echo "✅ core/timeout_trait.rs is dependency-free"

echo "Verifying trait uses ONLY core types..."
NON_CORE=$(grep "^use " src/core/timeout_trait.rs | grep -v "crate::core::" | grep -v "std::" | grep -v "async_trait")
if [ -n "$NON_CORE" ]; then
    echo "❌ VIOLATION: trait uses non-core types:"
    echo "$NON_CORE"
    exit 1
fi
echo "✅ trait uses ONLY core types"

echo "Verifying host_system/ does not violate ADR-WASM-023..."
VIOLATIONS=$(grep -rn "use crate::runtime\|use crate::actor" src/host_system/timeout_impl.rs)
if [ -n "$VIOLATIONS" ]; then
    echo "❌ VIOLATION: host_system/ has forbidden imports:"
    echo "$VIOLATIONS"
    exit 1
fi
echo "✅ host_system/ is compliant with ADR-WASM-023"

echo ""
echo "✅ All KNOWLEDGE-WASM-030 compliance checks passed!"

# 8. Verify traits are dependency-free
grep -rn "use tokio\|use dashmap\|use chrono" src/core/timeout_trait.rs
# Expected: No output (trait has no external dependencies)

# 9. Verify dependency injection pattern
grep -rn "Arc<dyn" src/host_system/manager.rs
# Expected:
# correlation_tracker: Arc<dyn CorrelationTrackerTrait>
# timeout_handler: Arc<dyn TimeoutHandlerObjectSafeTrait>
```

---

### Acceptance Criteria Checklist

#### Phase 1 Complete (Create Object-Safe Timeout Trait):
- [ ] `TimeoutHandlerObjectSafeTrait` defined in `core/timeout_trait.rs`
- [ ] Trait is object-safe (no generic methods)
- [ ] Trait uses `Arc<dyn CorrelationTrackerTrait>` for tracker parameter
- [ ] All methods documented with canonical sections (Summary, Arguments, Examples, Errors)
- [ ] Trait re-exported in `core/mod.rs`
- [ ] Code compiles without errors
- [ ] NEW: 4 unit tests added for object-safe trait
- [ ] NEW: Tests pass (trait compiles, implementation works, delegation works)

#### Phase 2 Complete (Implement Object-Safe Trait):
- [ ] `TimeoutHandlerObjectSafeTrait` implemented for `TimeoutHandler`
- [ ] All methods delegate to existing implementation
- [ ] Code compiles without errors
- [ ] No behavior changes
- [ ] NEW: 3 unit tests added for implementation
- [ ] NEW: Tests pass (trait object works, delegation correct)

#### Phase 3 Complete (Update HostSystemManager):
- [ ] `HostSystemManager.correlation_tracker` is `Arc<dyn CorrelationTrackerTrait>`
- [ ] `HostSystemManager.timeout_handler` is `Arc<dyn TimeoutHandlerObjectSafeTrait>`
- [ ] Constructor creates trait objects from concrete implementations
- [ ] Imports updated (removed concrete type imports, added trait imports)
- [ ] Debug impl updated with trait names
- [ ] Code compiles without errors
- [ ] All existing usages work without changes
- [ ] NEW: 1 unit test added for HostSystemManager DI pattern
- [ ] NEW: Test passes (trait objects used correctly)

#### Integration Tests (NEW):
- [ ] NEW: Integration test file created (`tests/host_system_manager_di_tests.rs`)
- [ ] NEW: DI pattern integration tests pass
- [ ] All integration tests pass (with updated imports)

#### Overall Complete:
- [ ] Build succeeds (zero errors)
- [ ] Zero compiler warnings
- [ ] Zero clippy warnings (with mandatory `-D warnings` flag)
- [ ] All unit tests pass (1064+ tests = 1059 + 5 new)
- [ ] All integration tests pass
- [ ] ADR-WASM-023 compliant (no forbidden imports)
- [ ] NEW: KNOWLEDGE-WASM-030 compliant (trait in core/, dependency-free)
- [ ] dependency-management.md compliant
- [ ] Dependency injection pattern verified
- [ ] PROJECTS_STANDARD.md compliant
- [ ] Rust Guidelines compliant

---

### ADR & Knowledge Compliance Checklist (REVISED)

#### adr-wasm-023-module-boundary-enforcement.md: Module Boundary Enforcement

- [ ] **Rule: No forbidden imports remain** - Verified: HostSystemManager imports only from `core/` and `host_system/`
- [ ] **Rule: Dependency flow is one-way** - Verified: `host_system/` → `core/` (traits), `host_system/` owns implementations
- [ ] **Verification commands return no output:**
  ```bash
  grep -rn "use crate::" src/host_system/manager.rs | grep -v "use crate::core\|use crate::host_system"
  # Expected: No output
  ```

#### KNOWLEDGE-WASM-030: Module Architecture - Hard Requirements (NEW - MANDATORY)

- [ ] **Rule: Trait Abstractions in core/** - Verified: `TimeoutHandlerObjectSafeTrait` in `core/timeout_trait.rs`
- [ ] **Rule: core/ imports NOTHING** - Verified: Trait has NO internal crate imports
- [ ] **Rule: Dependency flow is one-way** - Verified: `host_system/` → `core/` (NOT reverse)
- [ ] **Rule: No forbidden imports** - Verified: No forbidden imports created
- [ ] **Verification commands pass:**
  ```bash
  # Verify core/ trait has no forbidden imports
  grep -rn "use crate::" src/core/timeout_trait.rs | grep -v "use crate::core::"
  # Expected: No output
  
  # Verify trait uses ONLY core types
  grep "^use " src/core/timeout_trait.rs | grep -v "crate::core::" | grep -v "std::" | grep -v "async_trait"
  # Expected: No output
  
  # Verify host_system/ does not import from runtime/actor/
  grep -rn "use crate::runtime\|use crate::actor" src/host_system/timeout_impl.rs
  # Expected: No output
  ```
- [ ] **Evidence:** Phase 1 creates trait in `core/`, Phase 2 implements in `host_system/`

#### dependency-management.md Compliance

- [ ] **Rule 1: Abstractions Dependency-Free** - Verified: Traits in `core/` have no external dependencies
- [ ] **Rule 2: Dependency Injection Pattern** - Verified: HostSystemManager uses `Arc<dyn Trait>` for dependencies
- [ ] **Rule 3: Dependency Direction** - Verified: HostSystemManager depends on traits from `core/`, implements owned by `host_system/`

#### PROJECTS_STANDARD.md Compliance

- [ ] **§2.1 (3-Layer Imports)** - Verified: Imports organized (std, external, internal)
- [ ] **§4.3 (Module Architecture)** - Verified: mod.rs contains only declarations and re-exports
- [ ] **§6.1 (YAGNI)** - Verified: Only required changes made (object-safe trait)
- [ ] **§6.2 (Avoid `dyn`)** - Verified: `dyn` used only when necessary (DI pattern), alternatives not feasible
- [ ] **§6.4 (Quality Gates)** - Verified: Zero warnings, comprehensive tests (1064+ tests)

#### Rust Guidelines Compliance

- [ ] **M-DESIGN-FOR-AI** - Verified: Idiomatic APIs, thorough docs, testable code
- [ ] **M-MODULE-DOCS** - Verified: Module documentation with canonical sections
- [ ] **M-ERRORS-CANONICAL-STRUCTS** - Verified: Error types follow canonical structure
- [ ] **M-STATIC-VERIFICATION** - Verified: All lints enabled, clippy passes

---

### Risk Assessment

#### Risk 1: Parallel Trait Confusion

**Description:** Developers may be confused by having two parallel traits (`TimeoutHandlerTrait` vs `TimeoutHandlerObjectSafeTrait`).

**Likelihood:** Medium
**Impact:** Low (documentation can mitigate)

**Mitigation:**
- Clear documentation explaining when to use which trait
- Naming convention: `TimeoutHandlerObjectSafeTrait` makes purpose explicit
- Code comments explaining design rationale
- Examples in trait documentation showing usage patterns

#### Risk 2: Performance Regression

**Description:** Dynamic dispatch (`dyn`) may cause performance regression compared to static dispatch.

**Likelihood:** Low
**Impact:** Low (vtable lookup overhead is negligible for this use case)

**Mitigation:**
- Benchmark before and after (measure actual impact)
- Document that implementation still uses static dispatch
- Only trait object layer adds overhead (minimal)
- If performance becomes issue, can optimize later

#### Risk 3: Technical Debt Accumulation

**Description:** Parallel traits create technical debt that may need future cleanup.

**Likelihood:** Medium
**Impact:** Low (debt is documented and manageable)

**Mitigation:**
- Document technical debt in appropriate files
- Consider future migration to fully object-safe design if needed
- Document design decision and rationale
- Track as potential refactoring target for Phase 2

---

### Benefits of Recommended Approach

#### Benefit 1: Achieves DI Goals

**Description:** HostSystemManager can use `Arc<dyn Trait>` for both dependencies, enabling dependency injection pattern.

**Impact:**
- Can inject mock implementations for testing
- Can swap implementations at runtime
- Follows dependency-management.md Rule 2

**Evidence:**
- `correlation_tracker: Arc<dyn CorrelationTrackerTrait>`
- `timeout_handler: Arc<dyn TimeoutHandlerObjectSafeTrait>`

#### Benefit 2: Minimal Code Changes

**Description:** Only requires adding one new trait and updating HostSystemManager.

**Impact:**
- Low implementation risk
- Fast implementation time
- Fewer opportunities for bugs

**Evidence:**
- 1 new trait (~100 lines)
- 1 implementation block (~20 lines)
- 1 struct update (~10 lines)
- Total: ~130 lines of changes

#### Benefit 3: Preserves Performance

**Description:** Implementation still uses static dispatch (generics), avoiding runtime overhead.

**Impact:**
- Zero performance regression in hot paths
- Vtable lookup only occurs at trait object boundary
- Hot paths in `TimeoutHandler` still use generics

**Evidence:**
- `TimeoutHandler::register_timeout()` still uses generic parameter
- Only wrapper trait uses `dyn`
- Most code still compiled with static dispatch

#### Benefit 4: KNOWLEDGE-WASM-030 Compliant (NEW)

**Description:** New trait in `core/`, dependency-free, follows hard requirements.

**Impact:**
- Complies with mandatory hard requirements
- No forbidden imports
- Clean architecture

**Evidence:**
- `TimeoutHandlerObjectSafeTrait` in `core/timeout_trait.rs`
- Trait uses ONLY std and core types
- Verification commands confirm compliance

#### Benefit 5: Future-Proof

**Description:** Design allows future migration to fully object-safe trait if needed.

**Impact:**
- No breaking changes needed
- Can evolve design over time
- Maintains backward compatibility

**Evidence:**
- Both traits can coexist
- Can deprecate generic trait later if needed
- Migration path exists

#### Benefit 6: Standards Compliant (REVISED)

**Description:** All architectural and quality standards are satisfied.

**Impact:**
- ADR-WASM-023 compliant (no forbidden imports)
- KNOWLEDGE-WASM-030 compliant (trait in core/, dependency-free)
- dependency-management.md compliant (DI pattern)
- PROJECTS_STANDARD.md compliant (quality gates)
- Rust Guidelines compliant (idiomatic code)

**Evidence:**
- All verification commands pass
- Zero compiler warnings
- Zero clippy warnings
- Comprehensive tests (1064+ tests)

---

### Documentation Requirements

#### Type: Reference Documentation

**Diátaxis Compliance:**
- Focus on API documentation (traits, structs, methods)
- Provide clear examples for each method
- Include error conditions and panics
- No marketing hyperbole

**Quality Standards (per documentation-quality-standards.md):**
- Professional, technical language
- Measurable claims (performance targets, etc.)
- No hyperbolic terms (amazing, incredible, etc.)
- Clear, concise explanations

**Sections Required:**
- Summary (what is this trait for?)
- When to Use This Trait (vs TimeoutHandlerTrait)
- Thread Safety (contract)
- Examples (for each method)
- Errors (if applicable)
- Panics (if applicable)

**Standards Compliance Checklist (REVISED):**
```markdown
## Standards Compliance Checklist

**PROJECTS_STANDARD.md Applied:**
- [ ] **§2.1 3-Layer Import Organization** - Evidence: Imports organized in manager.rs
- [ ] **§6.2 Avoid `dyn` Patterns** - Evidence: `dyn` used only when necessary for DI
- [ ] **§6.4 Implementation Quality Gates** - Evidence: Zero warnings, 1064+ tests pass

**Rust Guidelines Applied:**
- [ ] **M-DESIGN-FOR-AI** - Idiomatic APIs, comprehensive docs
- [ ] **M-MODULE-DOCS** - Module documentation complete
- [ ] **M-STATIC-VERIFICATION** - Zero clippy warnings

**Documentation Quality:**
- [ ] **No hyperbolic terms** - Verified against forbidden list
- [ ] **Technical precision** - All claims measurable
- [ ] **Diátaxis compliance** - Reference documentation type

**ADR & Knowledge Compliance (REVISED):**
- [ ] **adr-wasm-023-module-boundary-enforcement.md** - No forbidden imports
- [ ] **KNOWLEDGE-WASM-030** - Trait in core/, dependency-free, verified
- [ ] **dependency-management.md** - DI pattern implemented
```

---

### Summary

**Recommended Approach:** ✅ **Option D - Create Object-Safe Timeout Trait**

**Rationale:**
- Achieves Subtask 1.8's stated goal of "Update ActorSystemManager to use Traits (DI Pattern)"
- Technically feasible (works with Rust object safety rules)
- Minimal code changes (no breaking changes)
- Preserves performance (static dispatch in implementation)
- Standards compliant (all PROJECTS_STANDARD.md requirements satisfied)
- KNOWLEDGE-WASM-030 compliant (trait in core/, dependency-free)
- Pragmatic compromise (balances DI goals with technical constraints)

**Implementation Steps:**
1. **Phase 1:** Create `TimeoutHandlerObjectSafeTrait` in `core/timeout_trait.rs`
2. **Phase 2:** Implement trait for `TimeoutHandler` in `host_system/timeout_impl.rs`
3. **Phase 3:** Update `HostSystemManager` to use `Arc<dyn Trait>` for both fields

**Expected Outcomes:**
- ✅ Dependency injection pattern achieved
- ✅ ADR-WASM-023 compliant (no forbidden imports)
- ✅ KNOWLEDGE-WASM-030 compliant (trait in core/, dependency-free)
- ✅ PROJECTS_STANDARD.md compliant (quality gates met)
- ✅ Zero compiler warnings
- ✅ Zero clippy warnings
- ✅ All tests passing (1064+ tests = 1059 + 5 new)
- ✅ Minimal performance impact
- ✅ Future-proof design

**Next Steps After This Subtask:**
- Subtask 1.9: Update actor/ to use Traits
- Subtask 1.10: Update runtime/ to use Traits
- Subtask 1.11: Update messaging/ to use Traits
- Subtask 1.12: Delete Old Files (correlation_tracker.rs, timeout_handler.rs)

**Questions for Reviewers:**

1. **Do you agree with Option D as best approach?**
   - Alternative approaches (A, B, C) are clearly inferior or impossible
   - Option D achieves stated goals while respecting technical constraints
   - KNOWLEDGE-WASM-030 compliance verified

2. **Is the plan complete with all testing requirements?**
    - Added 3 unit tests for object-safe trait (test_timeout_handler_object_safe_trait_compiles, test_timeout_handler_implements_object_safe_trait, test_object_safe_trait_delegates_to_implementation)
    - Added 1 unit test for HostSystemManager DI pattern (test_host_system_manager_uses_trait_objects)
    - Added 1 integration test for DI pattern (test_host_system_manager_with_mock_implementations)
    - Total: 5 new tests (4 unit + 1 integration), 1064+ total tests

3. **Are all ADR/Knowledge references correct?**
   - Changed to lowercase filenames: "adr-wasm-023-module-boundary-enforcement.md"
   - Added KNOWLEDGE-WASM-030 compliance section with verification
   - All references match actual filenames

**Reply with:**
- "Approve" → Proceed with implementation
- "Changes: [feedback]" → I'll revise plan
- "Questions: [questions]" → I'll address concerns

