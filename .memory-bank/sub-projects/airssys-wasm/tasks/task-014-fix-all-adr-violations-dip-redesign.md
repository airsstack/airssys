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


