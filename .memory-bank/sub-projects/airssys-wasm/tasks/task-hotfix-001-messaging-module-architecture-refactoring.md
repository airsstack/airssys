# [WASM-TASK-HOTFIX-001] - Messaging Module Architecture Refactoring

**Task ID:** WASM-TASK-HOTFIX-001  
**Created:** 2025-12-26  
**Updated:** 2025-12-26  
**Priority:** 🔴 CRITICAL / BLOCKING  
**Status:** NOT STARTED  
**Blocks:** All subsequent WASM-TASK-006+ and Block 5+ development  
**Estimated Effort:** 3.5-4.5 weeks  

---

## Executive Summary

### Architectural Problem Discovered

During architectural audit, discovered a **critical module architecture violation** where messaging infrastructure is incorrectly placed in `runtime/` module instead of a dedicated top-level `messaging/` module.

**What's Wrong:**
1. **🔴 Module Boundary Violation**: `src/runtime/messaging.rs` (1,313 lines) contains messaging infrastructure
2. **🔴 Wrong Module Responsibility**: `runtime/` should only handle WASM execution (Block 1), not inter-component communication
3. **🔴 Missing Top-Level Module**: No `messaging/` module exists (should be Block 5)
4. **🔴 Circular Dependency Risk**: `runtime/messaging.rs` imports from `actor/message/`

**What's Correct:**
- `messaging/` should be a **top-level module** (Block 5: Inter-Component Communication)
- `runtime/` should only contain: WasmEngine, ComponentLoader, ResourceLimits, StoreManager
- All messaging infrastructure should be in `src/messaging/`

### Impact

**Violates Multiple Architecture Standards:**
- ❌ ADR-WASM-018: Three-Layer Architecture (one-way dependencies only)
- ❌ KNOWLEDGE-WASM-012: Module Structure Architecture (messaging/ as top-level)
- ❌ ADR-WASM-023: Module Boundary Enforcement

**Blocks Development:**
- ❌ Block 5 (Inter-Component Communication) can't be properly developed
- ❌ Future messaging features have no clear home
- ❌ Creates confusion about where to add messaging code

---

## Problem Statement

### Issue 1: Messaging Infrastructure in Wrong Module

**Current (WRONG):**
```text
src/
  ├── runtime/
  │   ├── engine.rs           ✅ WASM execution
  │   ├── loader.rs           ✅ Component loading
  │   ├── limits.rs           ✅ Resource limits
  │   └── messaging.rs        ❌ WRONG - 1,313 lines of messaging code
  └── actor/
      └── message/
          ├── correlation_tracker.rs
          └── ...
```

**runtime/messaging.rs Contains:**
- MessagingService (manages MessageBroker singleton)
- ResponseRouter (routes request-response messages)
- MessageReceptionMetrics (tracks message delivery)
- MessagingMetrics (publish/subscriber statistics)
- ResponseRouterStats (response routing metrics)
- MessageReceptionMetrics (message delivery tracking)

**Why It's Wrong:**

1. **Module Responsibility Violation**:
   - `runtime/` is Block 1: WASM Runtime Layer
   - Should only handle: Wasmtime engine, component loading, resource limits
   - NOT: Inter-component communication infrastructure

2. **Missing Top-Level Module**:
   - KNOWLEDGE-WASM-012 (lines 506-596) specifies `messaging/` as top-level
   - This module should be Block 5: Inter-Component Communication
   - Currently: Does not exist at top level

3. **Circular Dependency Risk**:
   ```rust
   // src/runtime/messaging.rs:76
   use crate::actor::message::{CorrelationId, CorrelationTracker, RequestError, ResponseMessage};
   ```
   - `runtime/` (lower level) imports from `actor/` (higher level)
   - Violates one-way dependency chain: core → runtime → actor → messaging

**Impact:**
- ❌ Confusing module boundaries
- ❌ Violates ADR-WASM-018 three-layer architecture
- ❌ Makes code harder to navigate
- ❌ Blocks proper Block 5 development
- ❌ Risk of circular dependencies

### Issue 2: Architectural Standards Compliance

**Required Architecture (from KNOWLEDGE-WASM-012):**

```text
src/
  ├── core/                # Foundation (no internal deps)
  ├── runtime/              # Block 1: WASM Runtime Layer
  │   ├── engine.rs
  │   ├── loader.rs
  │   ├── limits.rs
  │   ├── async_host.rs
  │   └── store_manager.rs
  ├── actor/               # Block 3: Actor System Integration
  └── messaging/            # Block 5: Inter-Component Communication ← MISSING
      ├── messaging_service.rs
      ├── router.rs
      ├── fire_and_forget.rs
      ├── request_response.rs
      ├── codec.rs
      └── topics.rs
```

**One-Way Dependency Chain (ADR-WASM-018):**
```
core/ (foundation)
  ↓
runtime/ (WASM execution)
  ↓
actor/ (Actor system integration)
  ↓
messaging/ (Inter-component communication)
```

**Key Rule**: Dependencies flow ONE WAY (top to bottom). Higher layers CANNOT import from lower layers.

**Current Violation:**
```
runtime/messaging.rs → actor/message/  ← WRONG! Reverse dependency
```

**Impact:**
- ❌ Cannot test runtime/ in isolation
- ❌ Creates circular coupling
- ❌ Makes code harder to understand
- ❌ Violates multiple architectural standards

---

## Context

### Relevant Architecture Documents

**Primary References:**
- **KNOWLEDGE-WASM-012**: Module Structure Architecture (lines 506-596 define messaging/ module)
- **ADR-WASM-018**: Three-Layer Architecture (one-way dependency chain)
- **ADR-WASM-023**: Module Boundary Enforcement (dependency rules)
- **KNOWLEDGE-WASM-034**: Module Architecture Violation - Messaging in Runtime (this document)

**Supporting References:**
- **KNOWLEDGE-WASM-002**: High-Level Overview
- **KNOWLEDGE-WASM-003**: Core Architecture Design
- **KNOWLEDGE-WASM-024**: Component Messaging Clarifications
- **KNOWLEDGE-WASM-029**: Messaging Patterns

**New Documentation Created for This Task:**
- **KNOWLEDGE-WASM-034**: Documents architectural violation
- **ADR-WASM-024**: Decision to refactor messaging to top-level module

### Completed Blocks (Foundation)

- ✅ **WASM-TASK-000**: Core Abstractions (9,283 lines, 363 tests)
- ✅ **WASM-TASK-002**: WASM Runtime Layer (338 lines, 214 tests)
- ✅ **WASM-TASK-003**: WIT Interface System (2,214 lines WIT + 176 lines build)
- ✅ **WASM-TASK-004**: Actor System Integration (15,620+ lines, 589 tests)
- ✅ **WASM-TASK-005**: Security & Isolation Layer (13,500+ lines, 388 tests)

### Foundation Quality Metrics

- **Total Code**: 275K+ lines (9,283 + 338 + 2,390 + 15,620 + 13,500)
- **Total Tests**: 1,654 tests (363 + 214 + 589 + 388)
- **Test Pass Rate**: 100% (all tests passing)
- **Code Quality**: Zero compiler warnings, zero clippy warnings
- **Architecture**: Block 1-4: Complete, Block 5: Blocked by architecture violation

---

## Objectives

### Primary Objective

**Refactor messaging infrastructure from `runtime/` to top-level `messaging/` module to fix architectural violation:**

1. ✅ Create top-level `src/messaging/` module
2. ✅ Move all messaging code from `runtime/messaging.rs` to `messaging/messaging_service.rs`
3. ✅ Update all import statements across codebase
4. ✅ Remove `runtime/messaging.rs`
5. ✅ Enforce one-way dependency chain: core → runtime → actor → messaging
6. ✅ Verify no circular dependencies remain

### Secondary Objectives

- Align with ADR-WASM-018 three-layer architecture
- Follow KNOWLEDGE-WASM-012 module structure specification
- Maintain zero compiler/clippy warnings
- Add comprehensive integration tests
- Improve code navigation and maintainability
- Enable proper Block 5 development

---

## Implementation Plan

### Phase 1: Create Top-Level messaging/ Module (Days 1-2)

#### Task 1.1: Create messaging Module Structure

**Objective:** Create top-level `src/messaging/` module with proper structure.

**Deliverables:**

**Files to Create:**
- `src/messaging/mod.rs` - Module declarations only
- `src/messaging/messaging_service.rs` - Main messaging service (moved from runtime/)
- `src/messaging/router.rs` - MessageBroker routing integration
- `src/messaging/fire_and_forget.rs` - Fire-and-forget pattern
- `src/messaging/request_response.rs` - Request-response pattern
- `src/messaging/codec.rs` - Multicodec message encoding
- `src/messaging/topics.rs` - Topic-based pub-sub (Phase 2+)

**Files to Update:**
- `src/lib.rs` - Add `pub mod messaging;`
- `src/prelude.rs` - Re-export messaging types

**Messaging Module Structure:**
```rust
// src/messaging/mod.rs
//! Inter-component communication infrastructure.
//!
//! This module provides messaging infrastructure for communication
//! between WASM components, including:
//!
//! - MessageBroker integration
//! - Request-response patterns
//! - Fire-and-forget messaging
//! - Topic-based pub/sub (Phase 2)
//! - Multicodec message encoding
//!
//! # Architecture
//!
//! ```text
//! ┌─────────────────────────────────────────┐
//! │              Messaging Module            │
//! │  ┌────────────────────────────────┐     │
//! │  │  • MessageBroker integration  │     │
//! │  │  • Request-response routing │     │
//! │  │  • Fire-and-forget messaging│     │
//! │  │  • Metrics and monitoring     │     │
//! │  └────────────────────────────────┘     │
//! └─────────────────────────────────────────┘
//!                         ↓ uses
//! ┌─────────────────────────────────────────┐
//! │    airssys-rt InMemoryMessageBroker │
//! └─────────────────────────────────────────┘
//! ```

// Module declarations (§4.3 - declaration-only pattern)
pub mod messaging_service;
pub mod router;
pub mod fire_and_forget;
pub mod request_response;
pub mod codec;
pub mod topics;  // Phase 2

// Public re-exports
pub use messaging_service::{MessagingService, MessagingStats, ResponseRouter, ResponseRouterStats};
pub use router::{MessageRouter, RoutingStats};
pub use fire_and_forget::FireAndForget;
pub use request_response::{RequestResponse, RequestError};
pub use codec::MulticodecCodec;
```

**Success Criteria:**
- ✅ `src/messaging/mod.rs` created with module declarations
- ✅ `src/messaging/messaging_service.rs` created
- ✅ `src/lib.rs` updated with `pub mod messaging;`
- ✅ `src/prelude.rs` updated with messaging re-exports
- ✅ `cargo build` succeeds

**Estimated Effort:** 4-6 hours  
**Risk Level:** Low (new module creation)

---

#### Task 1.2: Move Messaging Code from runtime/messaging.rs

**Objective:** Move all messaging infrastructure code to new location.

**Deliverables:**

**File to Create:** `src/messaging/messaging_service.rs`

**Code to Move:** From `src/runtime/messaging.rs` (lines 1-1313)

**Content to Move:**
- MessagingService struct (lines 126-387)
- MessagingMetrics struct (lines 418-431)
- MessagingStats struct (lines 448-467)
- ResponseRouter struct (lines 511-666)
- ResponseRouterMetrics struct (lines 521-531)
- ResponseRouterStats struct (lines 668-679)
- MessageReceptionMetrics struct (lines 736-852)
- MessageReceptionStats struct (lines 868-885)
- All tests (lines 887-1313)

**Import Updates Required:**
```rust
// Update imports in messaging_service.rs:
// FROM: use crate::actor::message::{CorrelationId, CorrelationTracker, RequestError, ResponseMessage};
// TO: use crate::core::messaging::{CorrelationId, CorrelationTracker, RequestError, ResponseMessage};
```

**Success Criteria:**
- ✅ `src/messaging/messaging_service.rs` created with all code moved
- ✅ Imports updated to use `crate::core::messaging::` instead of `crate::actor::message::`
- ✅ `cargo build` succeeds
- ✅ All tests in messaging_service.rs pass
- ✅ No imports from `actor/` (imports from `core/` instead)

**Estimated Effort:** 6-8 hours  
**Risk Level:** Medium (import updates, code verification)

---

#### Task 1.3: Create Remaining Messaging Submodules

**Objective:** Create messaging submodules per KNOWLEDGE-WASM-012 specification.

**Deliverables:**

**Files to Create:**
- `src/messaging/router.rs` - MessageBroker routing
- `src/messaging/fire_and_forget.rs` - Fire-and-forget pattern
- `src/messaging/request_response.rs` - Request-response pattern
- `src/messaging/codec.rs` - Multicodec encoding
- `src/messaging/topics.rs` - Topic-based pub/sub (stub for Phase 2)

**Success Criteria:**
- ✅ All messaging submodules created
- ✅ `src/messaging/mod.rs` declarations match created files
- ✅ `cargo build` succeeds
- ✅ Module structure follows KNOWLEDGE-WASM-012

**Estimated Effort:** 4-6 hours  
**Risk Level:** Low (new modules with clear scope)

---

### Phase 2: Update All Import Statements (Days 2-3)

#### Task 2.1: Update Imports in actor/message/

**Objective:** Change imports to use new messaging module location.

**Deliverables:**

**Files to Update:**
- `src/actor/message/actor_system_subscriber.rs`
- `src/actor/message/correlation_tracker.rs`
- `src/actor/message/message_broker_bridge.rs`
- `src/actor/message/message_publisher.rs`
- `src/actor/message/message_router.rs`
- `src/actor/message/request_response.rs`

**Import Changes:**
```rust
// BEFORE (WRONG):
use crate::runtime::MessagingService;

// AFTER (CORRECT):
use crate::messaging::MessagingService;
```

**Success Criteria:**
- ✅ All actor/message/ files updated
- ✅ No imports of `runtime::MessagingService` remain
- ✅ All imports use `messaging::` instead
- ✅ `cargo build` succeeds
- ✅ `cargo test` passes

**Estimated Effort:** 2-3 hours  
**Risk Level:** Low (straightforward search-and-replace)

---

#### Task 2.2: Update Imports in runtime/ Modules

**Objective:** Remove messaging imports from runtime/ (should not exist after refactoring).

**Deliverables:**

**Files to Update:**
- `src/runtime/async_host.rs`
- `src/runtime/engine.rs`
- `src/runtime/mod.rs`

**Import Changes:**
```rust
// Remove any imports like:
use crate::actor::message::{CorrelationId, CorrelationTracker, RequestError, ResponseMessage};

// If messaging types needed, import from messaging/ instead:
use crate::messaging::{CorrelationId, CorrelationTracker, RequestError, ResponseMessage};
```

**Success Criteria:**
- ✅ `src/runtime/` no longer imports from `actor/message/`
- ✅ `grep -rn "use crate::actor::message" src/runtime/` returns nothing
- ✅ If messaging types needed, imported from `messaging/` instead
- ✅ `cargo build` succeeds
- ✅ `cargo test` passes

**Estimated Effort:** 2-3 hours  
**Risk Level:** Low (should be minimal after move)

---

#### Task 2.3: Update Imports in Integration Tests

**Objective:** Update all test files to use new import paths.

**Deliverables:**

**Files to Update:**
- All integration test files in `tests/` that reference messaging types
- Test files with `use airssys_wasm::runtime::MessagingService`

**Import Changes:**
```rust
// BEFORE (WRONG):
use airssys_wasm::runtime::{MessagingService, MessagingStats};

// AFTER (CORRECT):
use airssys_wasm::messaging::{MessagingService, MessagingStats};
```

**Success Criteria:**
- ✅ All integration tests updated
- ✅ `grep -rn "use airssys_wasm::runtime::MessagingService" tests/` returns nothing
- ✅ `cargo test --test` passes all tests
- ✅ All tests use correct import paths

**Estimated Effort:** 3-4 hours  
**Risk Level:** Low (test file updates)

---

#### Task 2.4: Update Imports in Examples

**Objective:** Update all example files to use new import paths.

**Deliverables:**

**Files to Update:**
- All example files in `examples/` that reference messaging types
- Example files with `use airssys_wasm::runtime::MessagingService`

**Import Changes:**
```rust
// BEFORE (WRONG):
use airssys_wasm::runtime::MessagingService;

// AFTER (CORRECT):
use airssys_wasm::messaging::MessagingService;
```

**Success Criteria:**
- ✅ All examples updated
- ✅ `grep -rn "use airssys_wasm::runtime::MessagingService" examples/` returns nothing
- ✅ All examples compile (`cargo check --examples`)
- ✅ Examples run correctly

**Estimated Effort:** 2-3 hours  
**Risk Level:** Low (example file updates)

---

### Phase 3: Remove runtime/messaging.rs (Days 3-4)

#### Task 3.1: Verify All Imports Updated

**Objective:** Ensure no code still imports from runtime/messaging.rs before deletion.

**Deliverables:**

**Verification Commands:**
```bash
# Check 1: No direct imports of runtime/messaging
grep -rn "use airssys_wasm::runtime::messaging" src/ tests/ examples/

# Check 2: No MessagingService from runtime/
grep -rn "use airssys_wasm::runtime::MessagingService" src/ tests/ examples/

# Check 3: Build succeeds
cargo build

# Check 4: All tests pass
cargo test --lib
```

**Success Criteria:**
- ✅ No imports of `runtime::messaging` found
- ✅ No imports of `runtime::MessagingService` found
- ✅ `cargo build` succeeds
- ✅ `cargo test --lib` passes

**Estimated Effort:** 1-2 hours  
**Risk Level:** Low (verification only)

---

#### Task 3.2: Delete runtime/messaging.rs

**Objective:** Remove old messaging file after all imports updated.

**Deliverables:**

**File to Delete:** `src/runtime/messaging.rs`

**Files to Update:**
- `src/runtime/mod.rs` - Remove messaging from re-exports

**Updates to runtime/mod.rs:**
```rust
// BEFORE (WRONG):
pub use messaging::{MessageReceptionMetrics, MessageReceptionStats, MessagingService, MessagingStats, ResponseRouter, ResponseRouterStats};

// AFTER (CORRECT):
// Remove these re-exports entirely
```

**Success Criteria:**
- ✅ `src/runtime/messaging.rs` deleted
- ✅ `src/runtime/mod.rs` no longer exports messaging types
- ✅ `cargo build` succeeds
- ✅ `cargo test` passes
- ✅ No references to deleted file exist

**Estimated Effort:** 1 hour  
**Risk Level:** Medium (deletion requires verification)

---

### Phase 4: Add Architecture Compliance Tests (Days 4-5)

#### Task 4.1: Create Architecture Compliance Tests

**Objective:** Create tests that verify architectural rules are followed.

**Deliverables:**

**File to Create:** `tests/architecture_compliance_tests.rs`

**Test Cases:**
```rust
#[test]
fn test_runtime_never_imports_from_actor() {
    let runtime_code = include_str!("../src/runtime/mod.rs");
    assert!(!runtime_code.contains("use crate::actor"), 
        "runtime/ should not import from actor/");
}

#[test]
fn test_runtime_never_imports_messaging_types() {
    let runtime_code = include_str!("../src/runtime/async_host.rs");
    assert!(!runtime_code.contains("use crate::actor::message::"), 
        "runtime/ should not import from actor/message/");
}

#[test]
fn test_core_never_imports_from_higher() {
    let core_code = include_str!("../src/core/mod.rs");
    assert!(!core_code.contains("use crate::runtime"), 
        "core should not import from runtime/");
    assert!(!core_code.contains("use crate::actor"), 
        "core should not import from actor/");
    assert!(!core_code.contains("use crate::messaging"), 
        "core should not import from messaging/");
}

#[test]
fn test_messaging_module_exists() {
    let lib_code = include_str!("../src/lib.rs");
    assert!(lib_code.contains("pub mod messaging;"), 
        "top-level messaging module should exist");
}

#[test]
fn test_messaging_module_independent() {
    let messaging_code = include_str!("../src/messaging/mod.rs");
    assert!(!messaging_code.contains("use crate::runtime"), 
        "messaging/ should not import from runtime/");
}

#[test]
fn test_no_imports_from_deleted_file() {
    // Verify no imports from deleted runtime/messaging.rs
    let lib_code = include_str!("../src/lib.rs");
    let runtime_mod = include_str!("../src/runtime/mod.rs");
    assert!(!lib_code.contains("runtime::messaging"), 
        "Should not import from deleted runtime/messaging.rs");
    assert!(!runtime_mod.contains("pub mod messaging;"), 
        "runtime/ should not export messaging module");
}
```

**Success Criteria:**
- ✅ Architecture compliance tests created
- ✅ All compliance tests pass
- ✅ Tests verify no circular dependencies
- ✅ Tests verify correct module structure
- ✅ `cargo test` passes

**Estimated Effort:** 3-4 hours  
**Risk Level:** Low (new tests only)

---

### Phase 5: Verification & Testing (Days 5-7)

#### Task 5.1: Run All Tests

**Objective:** Verify all tests pass after refactoring.

**Deliverables:**

**Commands to Run:**
```bash
# Run all tests
cargo test --all

# Run with verbose output
cargo test --all -- --nocapture

# Run integration tests only
cargo test --test
```

**Success Criteria:**
- ✅ All unit tests pass (`cargo test --lib`)
- ✅ All integration tests pass (`cargo test --test`)
- ✅ Zero test failures
- ✅ Zero compiler warnings
- ✅ Zero clippy warnings

**Estimated Effort:** 1-2 hours  
**Risk Level:** Low (verification only)

---

#### Task 5.2: Verify No Circular Dependencies

**Objective:** Confirm one-way dependency chain is enforced.

**Deliverables:**

**Verification Commands:**
```bash
# Check 1: runtime/ doesn't import from actor/
echo "Checking runtime/ → actor/..."
if grep -rn "use crate::actor" src/runtime/; then
    echo "❌ FAILED: runtime/ imports from actor/"
    exit 1
fi
echo "✅ PASSED: runtime/ clean"

# Check 2: core/ doesn't import from runtime/ or actor/
echo "Checking core/ → runtime/ actor/..."
if grep -rn "use crate::runtime\|use crate::actor" src/core/; then
    echo "❌ FAILED: core/ imports from higher layers"
    exit 1
fi
echo "✅ PASSED: core/ clean"

# Check 3: messaging/ doesn't import from runtime/
echo "Checking messaging/ → runtime/..."
if grep -rn "use crate::runtime" src/messaging/; then
    echo "❌ FAILED: messaging/ imports from runtime/"
    exit 1
fi
echo "✅ PASSED: messaging/ clean"

# Check 4: No imports from deleted runtime/messaging.rs
echo "Checking no imports from deleted runtime/messaging.rs..."
if grep -rn "runtime::messaging" src/ tests/ examples/; then
    echo "❌ FAILED: Found imports from deleted runtime/messaging.rs"
    exit 1
fi
echo "✅ PASSED: No imports from deleted file"
```

**Success Criteria:**
- ✅ All grep checks return nothing
- ✅ runtime/ doesn't import from actor/
- ✅ core/ doesn't import from runtime/ or actor/
- ✅ messaging/ doesn't import from runtime/
- ✅ No imports from deleted runtime/messaging.rs

**Estimated Effort:** 1 hour  
**Risk Level:** Low (verification only)

---

#### Task 5.3: Run Benchmarks

**Objective:** Verify performance hasn't regressed.

**Deliverables:**

**Commands to Run:**
```bash
# Run all benchmarks
cargo bench

# Run specific messaging benchmarks
cargo bench --bench messaging

# Run actor benchmarks
cargo bench --bench actor
```

**Success Criteria:**
- ✅ All benchmarks compile
- ✅ All benchmarks run successfully
- ✅ No performance regressions
- ✅ Results documented

**Estimated Effort:** 2-3 hours  
**Risk Level:** Low (performance validation only)

---

### Phase 6: Documentation Updates (Days 7-8)

#### Task 6.1: Update Documentation

**Objective:** Update all documentation to reflect new module structure.

**Deliverables:**

**Files to Update:**
- README.md
- All knowledge documents referencing runtime/messaging
- All ADR documents referencing runtime/messaging
- Code examples in documentation

**Changes:**
- Replace `use airssys_wasm::runtime::MessagingService` with `use airssys_wasm::messaging::MessagingService`
- Update module structure diagrams
- Update architecture references
- Add migration notes for breaking changes

**Success Criteria:**
- ✅ All documentation updated
- ✅ No references to old import paths remain
- ✅ Module structure diagrams correct
- ✅ Migration notes clear

**Estimated Effort:** 3-4 hours  
**Risk Level:** Low (documentation updates only)

---

#### Task 6.2: Document Breaking Changes

**Objective:** Create clear documentation for the breaking import path changes.

**Deliverables:**

**Content to Add to Documentation:**

**Migration Notes to README.md:**
```markdown
## Breaking Changes

### Version 0.2.0

#### Messaging Module Moved to Top-Level

**Change:** Messaging infrastructure has been moved from `runtime::` to top-level `messaging::` module.

**Impact:** Import paths for messaging types have changed.

**Migration:**

**Old Import:**
```rust
use airssys_wasm::runtime::MessagingService;
use airssys_wasm::runtime::{MessagingStats, ResponseRouter};
```

**New Import:**
```rust
use airssys_wasm::messaging::MessagingService;
use airssys_wasm::messaging::{MessagingStats, ResponseRouter};
```

**If using prelude:** No changes required if using `use airssys_wasm::prelude::*;`

**Rationale:** This change aligns with ADR-WASM-018 three-layer architecture and KNOWLEDGE-WASM-012 module structure specification.
```

**Success Criteria:**
- ✅ Breaking changes documented in README.md
- ✅ Migration path provided for all affected import paths
- ✅ Examples showing before/after imports
- ✅ Links to ADR-WASM-024 for rationale

**Estimated Effort:** 2-3 hours  
**Risk Level:** Low (documentation only)

---

## Success Criteria

This task is complete when:

### Phase 1: Top-Level messaging/ Module Created ✅
- [ ] `src/messaging/mod.rs` created with module declarations
- [ ] `src/messaging/messaging_service.rs` created with all code moved
- [ ] `src/messaging/router.rs` created
- [ ] `src/messaging/fire_and_forget.rs` created
- [ ] `src/messaging/request_response.rs` created
- [ ] `src/messaging/codec.rs` created
- [ ] `src/messaging/topics.rs` created (stub)
- [ ] `src/lib.rs` updated with `pub mod messaging;`
- [ ] `src/prelude.rs` updated with messaging re-exports
- [ ] Module structure follows KNOWLEDGE-WASM-012 specification

### Phase 2: All Imports Updated ✅
- [ ] All `actor/message/` files updated to use `messaging::` imports
- [ ] All `runtime/` files no longer import from `actor/message/`
- [ ] All integration tests updated
- [ ] All examples updated
- [ ] `grep -rn "use airssys_wasm::runtime::MessagingService" src/` returns nothing
- [ ] `grep -rn "use crate::runtime::messaging" src/` returns nothing
- [ ] All imports use `messaging::` instead of `runtime::messaging`
- [ ] `cargo build` succeeds

### Phase 3: runtime/messaging.rs Removed ✅
- [ ] All imports verified updated
- [ ] `src/runtime/messaging.rs` deleted
- [ ] `src/runtime/mod.rs` no longer exports messaging types
- [ ] No references to deleted file exist
- [ ] `cargo build` succeeds
- [ ] `cargo test` passes

### Phase 4: Architecture Compliance Added ✅
- [ ] Architecture compliance tests created
- [ ] All compliance tests pass
- [ ] Tests verify no circular dependencies
- [ ] Tests verify correct module structure

### Phase 5: Verification Complete ✅
- [ ] All tests pass (`cargo test --all`)
- [ ] Zero test failures
- [ ] Zero compiler warnings
- [ ] Zero clippy warnings
- [ ] `grep -rn "use crate::actor" src/runtime/` returns nothing
- [ ] `grep -rn "use crate::runtime\|use crate::actor" src/core/` returns nothing
- [ ] `grep -rn "use crate::runtime" src/messaging/` returns nothing
- [ ] No imports from deleted runtime/messaging.rs found
- [ ] All benchmarks run successfully
- [ ] No performance regressions

### Phase 6: Documentation Complete ✅
- [ ] README.md updated with breaking changes
- [ ] Migration notes added for import path changes
- [ ] Before/after import examples provided
- [ ] Links to ADR-WASM-024 added

### Overall Quality Gates ✅
- [ ] Zero compiler warnings (`cargo build`)
- [ ] Zero clippy warnings
- [ ] All tests passing
- [ ] No circular dependencies
- [ ] Module architecture compliant with ADR-WASM-018
- [ ] Module structure follows KNOWLEDGE-WASM-012
- [ ] End-to-end messaging functional
- [ ] Inter-component communication working

---

## Timeline Summary

| Phase | Tasks | Duration | Dependencies |
|-------|-------|----------|--------------|
| **Phase 1** | 1.1-1.3 | 1-2 days | None |
| **Phase 2** | 2.1-2.4 | 2-3 days | Phase 1 complete |
| **Phase 3** | 3.1-3.2 | 1 day | Phase 2 complete |
| **Phase 4** | 4.1 | 1-2 days | Phase 3 complete |
| **Phase 5** | 5.1-5.3 | 2-3 days | Phase 4 complete |
| **Phase 6** | 6.1-6.2 | 1-2 days | Phase 5 complete |
| **TOTAL** | **3.5-4.5 weeks** | All phases sequential |

---

## Risk Assessment

### Identified Risks

| Risk | Likelihood | Impact | Mitigation |
|-------|-------------|---------|-------------|
| **Breaking import paths** | High | Medium | Comprehensive search for all imports; update README with migration notes; provide before/after examples |
| **Missed imports during update** | Medium | Medium | Use grep to find all references; verify with build/tests |
| **External code references** | Low | Medium | Document breaking changes in README; provide migration path |
| **Test failures after move** | Medium | Medium | Comprehensive test coverage; incremental verification |
| **Circular dependency reintroduced** | Low | High | Architecture compliance tests; verification checks |
| **Performance regressions** | Low | Medium | Run all benchmarks before/after; document results |

### Contingency Plans

**Plan A (Primary):** Execute refactoring as planned in 3.5-4.5 weeks

**Plan B (Fallback):** If blocking issues arise, defer messaging/ submodules to future phase, focus only on moving core MessagingService

**Plan C (Rollback):** Keep re-exports in runtime/ as deprecated aliases for migration period, document clearly in README with deprecation warnings

---

## References

### New Documentation Created

**Knowledge:**
- **KNOWLEDGE-WASM-034**: Module Architecture Violation - Messaging in Runtime

**ADR:**
- **ADR-WASM-024**: Refactor Messaging from Runtime to Top-Level Module

### Architecture Documents

**ADRs:**
- **ADR-WASM-018**: Three-Layer Architecture (actor → runtime → core)
- **ADR-WASM-023**: Module Boundary Enforcement
- **ADR-WASM-011**: Module Structure Organization

**Knowledge:**
- **KNOWLEDGE-WASM-002**: High-Level Overview
- **KNOWLEDGE-WASM-003**: Core Architecture Design
- **KNOWLEDGE-WASM-012**: Module Structure Architecture (PRIMARY REFERENCE - lines 506-596)
- **KNOWLEDGE-WASM-024**: Component Messaging Clarifications
- **KNOWLEDGE-WASM-029**: Messaging Patterns
- **KNOWLEDGE-WASM-034**: Module Architecture Violation

### Archived Tasks

- WASM-TASK-HOTFIX-001 (Old): Architecture Hotfix & Integration Glue (absorbed into this task)

### Related Technical Debt

- **DEBT-WASM-004**: Message Delivery Runtime Glue Missing
- **DEBT-WASM-027**: Duplicate WASM Runtime Fatal Architecture Violation
- **DEBT-WASM-028**: Circular Dependency Actor Runtime

---

## Notes

### Why This Refactoring is Critical

**Current State (BROKEN):**
- Messaging infrastructure scattered: `runtime/messaging.rs` + `actor/message/`
- No clear home for messaging code
- Module boundaries violated
- Circular dependency risk present
- Blocks proper Block 5 development

**Target State (CORRECT):**
- All messaging infrastructure in top-level `messaging/` module
- Clear module boundaries
- One-way dependency chain enforced
- Easy navigation and understanding
- Enables Block 5 development

**Key Benefits:**
1. **Fixes Architectural Violation**: Aligns with ADR-WASM-018 and KNOWLEDGE-WASM-012
2. **Eliminates Circular Dependency Risk**: Enforces one-way dependency chain
3. **Improves Navigation**: All messaging code in one place
4. **Enables Future Development**: Clear foundation for Block 5 features
5. **Prevents Future Violations**: Architecture compliance tests prevent mistakes

### Key Principles Applied

1. **Architectural Compliance**: Follow ADR-WASM-018 and KNOWLEDGE-WASM-012 exactly
2. **Incremental Delivery**: Each phase has clear milestones and success criteria
3. **Verification Gates**: Don't mark complete until verified
4. **Real Testing**: Add comprehensive integration tests, not just unit tests
5. **Documentation First**: Explain what and why before coding
6. **Clear Breaking Change Documentation**: Migration path clearly documented in README

---

## History

| Date | Version | Changes |
|-------|---------|---------|
| 2025-12-26 | 1.0 | Initial creation - messaging module refactoring task |
| 2025-12-26 | 1.1 | Revised Phase 4 (removed CI checks) and Phase 6 (clarified migration guide) |

---

**Task ID:** WASM-TASK-HOTFIX-001  
**Status:** NOT STARTED  
**Priority:** 🔴 CRITICAL / BLOCKING  
**Blocker For:** All WASM-TASK-006+ work and Block 5+ work  
---
