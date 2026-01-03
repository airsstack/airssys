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

## Implementation Plan

### Phase 1: Move CorrelationTracker to Core/ (45-60 minutes)

#### Subtask 1.1: Create CorrelationTracker in core/

**Deliverables:**
- **File:** `airssys-wasm/src/core/correlation.rs` (new file)
- **Content:**
  - Move entire `CorrelationTracker` struct implementation from `host_system/correlation_tracker.rs`
  - Keep all methods unchanged: `new()`, `register()`, `unregister()`, `lookup()`, `cleanup_expired()`
  - Update module documentation
  - Mark all fields as pub (required for external modules to use)
  - Keep `Send + Sync` bounds

**Acceptance Criteria:**
1. CorrelationTracker struct moved to core/
2. All methods preserved with identical signatures
3. Module documentation follows M-MODULE-DOCS
4. Code compiles without errors
5. Zero clippy warnings

**Specific Changes:**
- Copy content from: `airssys-wasm/src/host_system/correlation_tracker.rs`
- Update imports in new file: remove `use crate::core::*`, keep std imports
- Update documentation: "Correlation tracking for request-response pattern (shared type in core/)"

#### Subtask 1.2: Update core/mod.rs

**Deliverables:**
- **File:** `airssys-wasm/src/core/mod.rs`
- **Changes:**
  - Add: `pub mod correlation;`
  - Add: `pub use correlation::CorrelationTracker;`

**Acceptance Criteria:**
1. CorrelationTracker module declared
2. CorrelationTracker type re-exported
3. Code compiles without errors

---

### Phase 2: Move TimeoutHandler to Core/ (30-45 minutes)

#### Subtask 2.1: Create TimeoutHandler in core/

**Deliverables:**
- **File:** `airssys-wasm/src/core/timeout.rs` (new file)
- **Content:**
  - Move entire `TimeoutHandler` struct implementation from `host_system/timeout_handler.rs`
  - Keep all methods unchanged: `new()`, `register_timeout()`, `cancel_timeout()`, `check_timeouts()`
  - Update module documentation
  - Mark all fields as pub (required for external modules to use)
  - Keep `Send + Sync` bounds

**Acceptance Criteria:**
1. TimeoutHandler struct moved to core/
2. All methods preserved with identical signatures
3. Module documentation follows M-MODULE-DOCS
4. Code compiles without errors
5. Zero clippy warnings

**Specific Changes:**
- Copy content from: `airssys-wasm/src/host_system/timeout_handler.rs`
- Update imports in new file: remove `use crate::core::*`, keep std imports
- Update documentation: "Timeout handling for request-response pattern (shared type in core/)"

#### Subtask 2.2: Update core/mod.rs

**Deliverables:**
- **File:** `airssys-wasm/src/core/mod.rs`
- **Changes:**
  - Add: `pub mod timeout;`
  - Add: `pub use timeout::TimeoutHandler;`

**Acceptance Criteria:**
1. TimeoutHandler module declared
2. TimeoutHandler type re-exported
3. Code compiles without errors

---

### Phase 3: Remove Forbidden Imports (30-45 minutes)

#### Subtask 3.1: Remove actor/ → host_system/ Imports

**Deliverables:**
- **File:** `airssys-wasm/src/actor/mod.rs`
- **Lines to modify:** 179, 181
- **Changes:**
  - REMOVE: `pub use crate::host_system::correlation_tracker::CorrelationTracker;`
  - REMOVE: `pub use crate::host_system::timeout_handler::TimeoutHandler;`
  - ADD: `pub use crate::core::CorrelationTracker;`
  - ADD: `pub use crate::core::TimeoutHandler;`

**Acceptance Criteria:**
1. Forbidden imports removed from `actor/mod.rs`
2. Core type imports added
3. Code compiles without errors
4. No circular dependencies introduced

**ADR Constraints:**
- ADR-WASM-023: `actor/` imports only from `core/`, `runtime/` (ALLOWED)
- No forbidden imports to `host_system/` remain

#### Subtask 3.2: Remove runtime/ → host_system/ Imports

**Deliverables:**
- **File:** `airssys-wasm/src/runtime/async_host.rs`
- **Lines to modify:** 932
- **Changes:**
  - REMOVE: `use crate::host_system::{CorrelationTracker, TimeoutHandler};`
  - ADD: `use crate::core::{CorrelationTracker, TimeoutHandler};`

**Acceptance Criteria:**
1. Forbidden imports removed from `runtime/async_host.rs`
2. Core type imports added
3. Code compiles without errors
4. No circular dependencies introduced

**ADR Constraints:**
- ADR-WASM-023: `runtime/` imports only from `core/`, `security/` (ALLOWED)
- No forbidden imports to `host_system/` remain

#### Subtask 3.3: Remove messaging/ → host_system/ Imports

**Deliverables:**
- **Files:** 
  - `airssys-wasm/src/messaging/messaging_service.rs` (lines 76, 77, 734, 735)
  - `airssys-wasm/src/messaging/router.rs` (line 48)
- **Changes:**
  
**File: messaging_service.rs**
  - Line 76: REMOVE `use crate::host_system::correlation_tracker::CorrelationTracker;`
  - Line 77: REMOVE `use crate::host_system::timeout_handler::TimeoutHandler;`
  - Line 734: REMOVE `    use crate::host_system::correlation_tracker::CorrelationTracker;`
  - Line 735: REMOVE `    use crate::host_system::timeout_handler::TimeoutHandler;`
  - ADD at top: `use crate::core::{CorrelationTracker, TimeoutHandler};`

**File: router.rs**
  - Line 48: REMOVE `use crate::host_system::correlation_tracker::CorrelationTracker;`
  - ADD at top: `use crate::core::CorrelationTracker;`

**Acceptance Criteria:**
1. All forbidden imports removed from messaging/ files
2. Core type imports added
3. Code compiles without errors
4. No circular dependencies introduced

**ADR Constraints:**
- ADR-WASM-023: `messaging/` imports only from `core/`, `runtime/` (ALLOWED)
- No forbidden imports to `host_system/` remain

---

### Phase 4: Update HostSystemManager to Use Core Types (15-30 minutes)

#### Subtask 4.1: Update HostSystemManager Struct

**Deliverables:**
- **File:** `airssys-wasm/src/host_system/manager.rs`
- **Lines to modify:** 218, 228, 229
- **Changes:**
  - Line 218 (in new()): No changes needed - already imports correctly
  - Line 228: Change `Arc<CorrelationTracker>` to import from core
  - Line 229: Change `Arc<TimeoutHandler>` to import from core
  - Update struct field documentation to reference core types

**Import Changes:**
- At top of file, ensure: `use crate::core::{CorrelationTracker, TimeoutHandler};`

**Acceptance Criteria:**
1. HostSystemManager uses core types
2. Code compiles without errors
3. Zero clippy warnings
4. All existing functionality preserved

#### Subtask 4.2: Minor Fix: Add subscriber.stop() to shutdown()

**Deliverables:**
- **File:** `airssys-wasm/src/host_system/manager.rs`
- **Lines to modify:** 791-812
- **Changes:**
  - Add `subscriber.stop()` call before setting started flag to false
  
**Exact Code Change:**
```rust
// Before line 808 (before "Set started flag to false"):
// Stop subscriber
if let Err(e) = self.actor_system_subscriber.write().await.stop().await {
    eprintln!("Warning: Failed to stop subscriber: {:?}", e);
}

// Then existing code:
self.started.store(false, std::sync::atomic::Ordering::Relaxed);
```

**Acceptance Criteria:**
1. subscriber.stop() called during shutdown
2. Code compiles without errors
3. All tests pass

---

### Phase 5: Remove Old host_system Files (15 minutes)

#### Subtask 5.1: Delete Old Files

**Deliverables:**
- **Files to delete:**
  - `airssys-wasm/src/host_system/correlation_tracker.rs`
  - `airssys-wasm/src/host_system/timeout_handler.rs`

**Acceptance Criteria:**
1. Old files deleted
2. host_system/mod.rs updated to remove module declarations
3. Code compiles without errors
4. All tests pass

---

### Phase 6: Documentation Updates (30 minutes)

#### Subtask 6.1: Update Module Documentation

**Deliverables:**
- **Files:**
  - `airssys-wasm/src/core/correlation.rs`
  - `airssys-wasm/src/core/timeout.rs`
  - `airssys-wasm/src/host_system/manager.rs`

**Changes:**
- Update correlation.rs: "CorrelationTracker - Request-response correlation tracking (shared type in core/)"
- Update timeout.rs: "TimeoutHandler - Request timeout enforcement (shared type in core/)"
- Update manager.rs: Explain that HostSystemManager creates instances of core types
- Add examples showing usage

**Acceptance Criteria:**
1. Documentation follows M-MODULE-DOCS
2. All public APIs documented
3. Examples provided for usage
4. Diátaxis type: Reference documentation

---

### Phase 7: Verification & Testing (30-45 minutes)

#### Subtask 7.1: Verify Architecture Compliance

**Verification Commands:**

```bash
# 1. Build verification
cargo build --package airssys-wasm
# Expected: Clean build, zero errors

# 2. Clippy verification
cargo clippy --package airssys-wasm --all-targets --all-features -- -D warnings
# Expected: Zero warnings

# 3. Unit tests
cargo test --package airssys-wasm --lib
# Expected: All tests pass

# 4. Integration tests
cargo test --package airssys-wasm --test '*'
# Expected: All tests pass

# 5. ADR-WASM-023 architecture verification (CRITICAL)
grep -rn "use crate::host_system" src/actor/
# Expected: No output (actor/ no longer depends on host_system/)

grep -rn "use crate::host_system" src/runtime/
# Expected: No output (runtime/ no longer depends on host_system/)

grep -rn "use crate::host_system" src/messaging/
# Expected: No output (messaging/ no longer depends on host_system/)

grep -rn "use crate::actor" src/runtime/
# Expected: No output (runtime/ no longer depends on actor/)

grep -rn "use crate::" src/core/
# Expected: No output (core/ still dependency-free)
```

**Acceptance Criteria:**
1. Build succeeds with zero errors
2. Zero clippy warnings
3. All unit tests pass
4. All integration tests pass
5. Zero ADR-WASM-023 violations (all forbidden imports removed)

---

## Testing Plan

### Unit Tests (Mandatory - AGENTS.md §8)

**Phase 1-2 (Core Types):**
- No new tests needed - all existing tests for CorrelationTracker and TimeoutHandler will automatically use the new core location

**Phase 3 (Import Removals):**
- Verify compilation succeeds (tests compile correctly)
- No new tests needed - functionality unchanged

**Phase 4 (HostSystemManager):**
- Test 1: `test_host_system_manager_owns_correlation_tracker` - Verify field uses core type
- Test 2: `test_host_system_manager_owns_timeout_handler` - Verify field uses core type
- Test 3: `test_host_system_manager_shutdown_stops_subscriber` - NEW: Verify subscriber.stop() called

**Test Updates Required:**
```rust
// host_system/manager.rs tests
#[tokio::test]
async fn test_host_system_manager_shutdown_stops_subscriber() {
    // Test: Verify shutdown() calls subscriber.stop()
    let mut manager = HostSystemManager::new().await.unwrap();
    
    // Manually call shutdown
    manager.shutdown().await.unwrap();
    
    // Verify subscriber stopped
    let subscriber = manager.actor_system_subscriber.read().await;
    assert!(!subscriber.is_running(), "Subscriber should be stopped after shutdown");
}
```

### Integration Tests (Mandatory - AGENTS.md §8)

**Test: test_correlation_tracking_with_core_types**
- Create HostSystemManager
- Verify CorrelationTracker from core/ works correctly
- Test request-response correlation tracking
- Verify no forbidden imports

**Test: test_timeout_handling_with_core_types**
- Create HostSystemManager
- Verify TimeoutHandler from core/ works correctly
- Test timeout enforcement
- Verify no forbidden imports

**Test: test_host_system_lifecycle_with_core_types**
- Create HostSystemManager
- Verify all infrastructure uses core types
- Test startup and shutdown
- Verify subscriber.stop() called during shutdown
- Verify no circular dependencies

**Success Criteria:** All tests pass (1,042+ tests), verify real message/data flow

---

## Files to Modify (EXACT LIST)

| File | Line Numbers | Change Type |
|------|--------------|-------------|
| airssys-wasm/src/core/correlation.rs | NEW FILE | Create (copy from host_system/correlation_tracker.rs) |
| airssys-wasm/src/core/timeout.rs | NEW FILE | Create (copy from host_system/timeout_handler.rs) |
| airssys-wasm/src/core/mod.rs | N/A | Add module declarations and re-exports |
| airssys-wasm/src/actor/mod.rs | 179, 181 | Remove forbidden imports, add core imports |
| airssys-wasm/src/runtime/async_host.rs | 932 | Remove forbidden import, add core import |
| airssys-wasm/src/messaging/messaging_service.rs | 76, 77, 734, 735 | Remove forbidden imports, add core imports |
| airssys-wasm/src/messaging/router.rs | 48 | Remove forbidden import, add core import |
| airssys-wasm/src/host_system/manager.rs | 218, 228, 229, 808 | Update imports, add subscriber.stop() call |
| airssys-wasm/src/host_system/correlation_tracker.rs | ALL | DELETE (moved to core/) |
| airssys-wasm/src/host_system/timeout_handler.rs | ALL | DELETE (moved to core/) |
| airssys-wasm/src/host_system/mod.rs | N/A | Remove module declarations |

---

## Imports to Remove (EXACT LIST)

```
src/actor/mod.rs:179:pub use crate::host_system::correlation_tracker::CorrelationTracker;
src/actor/mod.rs:181:pub use crate::host_system::timeout_handler::TimeoutHandler;
src/runtime/async_host.rs:932:    use crate::host_system::{CorrelationTracker, TimeoutHandler};
src/messaging/messaging_service.rs:76:use crate::host_system::correlation_tracker::CorrelationTracker;
src/messaging/messaging_service.rs:77:use crate::host_system::timeout_handler::TimeoutHandler;
src/messaging/messaging_service.rs:734:    use crate::host_system::correlation_tracker::CorrelationTracker;
src/messaging/messaging_service.rs:735:    use crate::host_system::timeout_handler::TimeoutHandler;
src/messaging/router.rs:48:use crate::host_system::correlation_tracker::CorrelationTracker;
```

---

## Imports to Add (EXACT LIST)

```
src/core/mod.rs:pub mod correlation;
src/core/mod.rs:pub use correlation::CorrelationTracker;
src/core/mod.rs:pub mod timeout;
src/core/mod.rs:pub use timeout::TimeoutHandler;
src/actor/mod.rs:pub use crate::core::CorrelationTracker;
src/actor/mod.rs:pub use crate::core::TimeoutHandler;
src/runtime/async_host.rs:use crate::core::{CorrelationTracker, TimeoutHandler};
src/messaging/messaging_service.rs:use crate::core::{CorrelationTracker, TimeoutHandler};
src/messaging/router.rs:use crate::core::CorrelationTracker;
src/host_system/manager.rs:use crate::core::{CorrelationTracker, TimeoutHandler};
```

---

## Test Updates (SPECIFIC)

### Test Files to Update

**airssys-wasm/src/host_system/manager.rs (tests module):**
- `test_host_system_manager_new_success` - No changes needed (compilation tests)
- `test_host_system_manager_started_flag` - No changes needed
- `test_host_system_manager_shutdown_success` - No changes needed
- **NEW TEST:** `test_host_system_manager_shutdown_stops_subscriber` (see code above)

**All other tests:**
- No changes needed - all tests automatically use the new core location
- Functionality is identical, only imports changed

---

## Verification Commands

```bash
# 1. Build verification
cd /Users/hiraq/Projects/airsstack/airssys/airssys-wasm
cargo build
# Expected: Clean build, zero errors

# 2. Clippy verification
cargo clippy --all-targets --all-features -- -D warnings
# Expected: Zero warnings

# 3. Unit tests
cargo test --lib
# Expected: All tests pass

# 4. Integration tests
cargo test --test '*'
# Expected: All tests pass

# 5. ADR-WASM-023 architecture verification (CRITICAL)
grep -rn "use crate::host_system" src/actor/
# Expected: No output (actor/ no longer depends on host_system/)

grep -rn "use crate::host_system" src/runtime/
# Expected: No output (runtime/ no longer depends on host_system/)

grep -rn "use crate::host_system" src/messaging/
# Expected: No output (messaging/ no longer depends on host_system/)

grep -rn "use crate::actor" src/runtime/
# Expected: No output (runtime/ no longer depends on actor/)

grep -rn "use crate::" src/core/
# Expected: No output (core/ still dependency-free)
```

---

## Quality Standards

**All Phases Must Meet:**

**ADR-WASM-023 Compliance:**
- ✅ No forbidden imports from `actor/`, `runtime/`, `messaging/` to `host_system/`
- ✅ Dependency flow: All modules → `core/` (allowed)
- ✅ No circular dependencies
- ✅ Verification commands pass with no output

**PROJECTS_STANDARD.md Compliance:**
- ✅ §2.1: 3-Layer import organization in all modified files
- ✅ §6.1: YAGNI - Only implement required changes (use re-exports, not traits)
- ✅ §6.2: Use concrete types (CorrelationTracker, TimeoutHandler in core/, no dyn)
- ✅ §6.4: Implementation quality gates (zero warnings, comprehensive tests)

**Rust Guidelines:**
- ✅ M-DESIGN-FOR-AI: Idiomatic APIs, thorough docs, testable
- ✅ M-MODULE-DOCS: Module documentation with examples
- ✅ M-STATIC-VERIFICATION: All lints enabled, clippy passes
- ✅ M-FEATURES-ADDITIVE: Changes don't break existing APIs

**Dependency Management:**
- ✅ Shared types in `core/` (dependency-free layer)
- ✅ Implementations stay in ownership modules (host_system/ creates instances)
- ✅ No circular dependencies
- ✅ Re-export approach (simple, follows YAGNI)

**Documentation Standards:**
- ✅ Diátaxis type: Reference documentation
- ✅ Quality: Technical language, no hyperbole
- ✅ Canonical sections: Summary, Examples, Errors, Panics

**Testing Requirements (AGENTS.md §8):**
- ✅ Unit tests in `#[cfg(test)]` blocks for all modified code
- ✅ Integration tests in `tests/` directory for end-to-end verification
- ✅ All tests are REAL (test actual functionality, not stubs)
- ✅ 100% test pass rate
- ✅ Test coverage maintained or improved

---

## Risk Assessment

### Implementation Risks

**Low Risk:**
- Moving structs to core/ is straightforward
- Import updates are mechanical
- No functionality changes
- Existing tests will automatically validate

**Medium Risk:**
- 9 files need modifications
- Must ensure all import paths updated correctly
- Must update host_system/mod.rs to remove old module declarations

### Mitigation:
- Phased implementation with verification after each phase
- All changes maintain existing functionality
- Comprehensive test coverage maintained
- Clear acceptance criteria for each phase

---

## Success Criteria

**Task is Complete When:**
1. ✅ CorrelationTracker moved to `core/`
2. ✅ TimeoutHandler moved to `core/`
3. ✅ All forbidden imports removed (`actor/`, `runtime/`, `messaging/` → `host_system/`)
4. ✅ HostSystemManager updated to use core types
5. ✅ Old host_system files deleted
6. ✅ Documentation updated
7. ✅ All tests updated and passing
8. ✅ **Build succeeds** (zero errors, zero warnings)
9. ✅ **ADR-WASM-023 violations fixed** (no forbidden imports)
10. ✅ **KNOWLEDGE-WASM-036 compliance** (no circular dependencies)
11. ✅ subscriber.stop() called in shutdown()
12. ✅ Zero clippy warnings
13. ✅ All 1,042+ tests passing

---

## Questions Addressed

### 1. Which approach is better?

**Answer: Option B - Re-exports**

**Justification:**
- KNOWLEDGE-WASM-036 says core/ owns "All shared types" (Line 61)
- PROJECTS_STANDARD.md §6.2 says "Prefer concrete types first" (Line 137)
- PROJECTS_STANDARD.md §6.1 (YAGNI): Creating traits would be speculative generalization
- No current need for alternative implementations
- Simpler approach with fewer code changes
- CorrelationTracker and TimeoutHandler are data structures, not behavior abstractions

### 2. What exact files need modification?

**Answer:** 11 files (9 modified, 2 created, 2 deleted)
- See "Files to Modify (EXACT LIST)" section above

### 3. Should we create CoreSubscriber trait?

**Answer: NO**

**Justification:**
- No current need for alternative subscriber implementations
- HostSystemManager already uses concrete ActorSystemSubscriber type
- Creating a trait would violate YAGNI (§6.1)
- No benefit at this time
- Can add abstraction later if/when needed

### 4. What specific test updates are needed?

**Answer:**
- Add 1 new test: `test_host_system_manager_shutdown_stops_subscriber`
- All existing tests automatically work (no functionality changes)
- See "Test Updates (SPECIFIC)" section above

---

## References

**Related Documents:**
- ADR-WASM-023: Module Boundary Enforcement
- KNOWLEDGE-WASM-036: Three-Module Architecture
- KNOWLEDGE-WASM-026: Message Delivery Architecture
- PROJECTS_STANDARD.md: All mandatory patterns
- AGENTS.md §8: Mandatory testing requirements

**Parent Task Status:**
- Subtask 5.1: ✅ COMPLETE
- Subtask 5.2: ✅ COMPLETE
- Subtask 5.3: ✅ COMPLETE (but with ADR violations - FIXED BY THIS TASK)
- This Task (WASM-TASK-014): Fixes all ADR violations

---

## Status

**Created:** 2025-01-03  
**Status:** ✅ PLANNING COMPLETE - Ready for Implementation  
**Estimated Effort:** 3-4 hours (simplified re-export approach)  
**Priority:** 🔴 CRITICAL  
**Next Step:** Run `@memorybank-implementer WASM-TASK-014 DIP Redesign (Fix All Violations)`

---

## Implementation Plan (SOLID COMPLETE VERSION)

### Context & References

**ADR References:**
- **ADR-WASM-023: Module Boundary Enforcement** (MANDATORY)
  - Rule: `core/` imports NOTHING (dependency-free foundation)
  - Rule: All modules can import from `core/` (one-way dependencies only)
  - Forbidden imports MUST be eliminated: actor/ → host_system/, runtime/ → host_system/, messaging/ → host_system/
  - Verification: grep checks must return no output
- **KNOWLEDGE-WASM-036: Three-Module Architecture**
  - `core/` owns "All shared types (ComponentId, ComponentMessage, WasmError, etc.)"
  - `core/` line 61: CorrelationTracker and TimeoutHandler are shared types that belong in core/
  - Dependency flow: All modules → `core/` (allowed, no cycles)
- **KNOWLEDGE-WASM-030: Module Architecture Hard Requirements**
  - `core/` line 61: Shared data types must be in core/ to prevent circular dependencies
  - Module dependency rules MUST be followed without exception

**PROJECTS_STANDARD.md Compliance:**
- **§2.1 3-Layer Import Organization**: All modified files will follow std → external → internal import pattern
- **§4.3 Module Architecture Patterns**: mod.rs files will only contain declarations and re-exports
- **§6.1 YAGNI Principles**: Only implement required changes (re-export approach, no speculative traits)
- **§6.2 Avoid `dyn` Patterns**: Use concrete types (CorrelationTracker, TimeoutHandler in core/, no trait objects)
- **§6.4 Implementation Quality Gates**: Zero warnings, comprehensive tests, all clippy checks pass

**Rust Guidelines Applied:**
- **M-DESIGN-FOR-AI**: Idiomatic APIs with thorough documentation and testable code
- **M-MODULE-DOCS**: Module documentation will follow canonical sections (Summary, Examples, Errors, Panics)
- **M-ERRORS-CANONICAL-STRUCTS**: Error types follow canonical structure
- **M-STATIC-VERIFICATION**: All lints enabled, clippy must pass with zero warnings
- **M-FEATURES-ADDITIVE**: Changes will not break existing APIs

**Documentation Standards:**
- **Diátaxis Type**: Reference documentation for all public APIs
- **Quality**: Technical language, no marketing hyperbole per documentation-quality-standards.md
- **Canonical Sections**: All documented items will have Summary, Examples (where applicable), Errors, Panics sections
- **Evidence**: Code examples showing standards compliance

---

### PROJECTS_STANDARD.md Compliance (EXPLICIT SECTION)

This plan explicitly complies with the following PROJECTS_STANDARD.md sections:

**§2.1 3-Layer Import Organization:**
```rust
// All modified files will follow this pattern:
// Layer 1: Standard library imports
use std::collections::HashMap;
use std::sync::Arc;

// Layer 2: Third-party crate imports
use tokio::sync::RwLock;

// Layer 3: Internal module imports
use crate::core::CorrelationTracker;
```

**§4.3 Module Architecture Patterns:**
- `airssys-wasm/src/core/mod.rs` will contain ONLY module declarations and re-exports
- NO implementation code in mod.rs files

**§6.1 YAGNI Principles:**
- Move structs to core/ (simple, direct solution)
- Re-export from lib.rs (no trait abstractions needed)
- No speculative traits (CorrelationTracker and TimeoutHandler are data structures)

**§6.2 Avoid `dyn` Patterns:**
- Use concrete CorrelationTracker type (no trait objects)
- Use concrete TimeoutHandler type (no trait objects)

**§6.4 Implementation Quality Gates:**
```bash
cargo build --package airssys-wasm                    # Clean build
cargo clippy --package airssys-wasm --all-targets --all-features -- -D warnings  # Zero warnings
cargo test --package airssys-wasm --lib             # All unit tests pass
cargo test --package airssys-wasm --test '*'        # All integration tests pass
```

---

### Rust Guidelines Applied (EXPLICIT SECTION)

**M-DESIGN-FOR-AI:**
- Idiomatic APIs: CorrelationTracker and TimeoutHandler follow Rust API Guidelines
- Thorough docs: All public items have Summary, Examples, Errors, Panics sections
- Testable code: All functionality is covered by unit and integration tests

**M-MODULE-DOCS:**
```rust
//! Correlation tracking for request-response pattern (shared type in core/)
//!
//! Tracks pending request-response pairs in a request-response messaging pattern.
//!
//! # Examples
//!
//! ```rust
//! use airssys_wasm::core::CorrelationTracker;
//! ```
//!
//! # Errors
//!
//! - Returns `WasmError::DuplicateCorrelationId` if attempting to register
//!   a duplicate correlation ID
```

**M-ERRORS-CANONICAL-STRUCTS:**
- All error types use thiserror derive macro
- Errors include contextual information

**M-STATIC-VERIFICATION:**
- All lints enabled in clippy.toml
- Zero clippy warnings

**M-FEATURES-ADDITIVE:**
- Changes are additive (no breaking changes to existing APIs)
- Re-export from lib.rs maintains backward compatibility

---

### Documentation Standards (EXPLICIT SECTION)

**Diátaxis Framework Compliance:**
- **Type**: Reference documentation
- **Purpose**: Provide technical descriptions of APIs
- **Characteristics**: Information-oriented, austere, authoritative

**Quality Standards:**
- No hyperbolic terms (excellent, amazing, incredible, etc.)
- Technical precision: All claims are measurable
- Concrete examples: All APIs have working code examples

**Canonical Sections:**
All documented items MUST have:
1. **Summary sentence** (< 15 words)
2. **Extended documentation** (free form)
3. **# Examples** (working code examples)
4. **# Errors** (list known error conditions)
5. **# Panics** (list when panic may happen)

**Standards Compliance Checklist:**
```markdown
## Standards Compliance Checklist

**PROJECTS_STANDARD.md Applied:**
- [ ] **§2.1 3-Layer Import Organization**
- [ ] **§4.3 Module Architecture Patterns**
- [ ] **§6.1 YAGNI**
- [ ] **§6.2 Avoid `dyn` Patterns**
- [ ] **§6.4 Implementation Quality Gates**

**Rust Guidelines Applied:**
- [ ] **M-DESIGN-FOR-AI**
- [ ] **M-MODULE-DOCS**
- [ ] **M-ERRORS-CANONICAL-STRUCTS**
- [ ] **M-STATIC-VERIFICATION**

**Documentation Quality:**
- [ ] **No hyperbolic terms**
- [ ] **Technical precision**
- [ ] **Diátaxis compliance**
```

---

### Phase Summary

**Phase 1-2:** Move CorrelationTracker and TimeoutHandler to core/
**Phase 3:** Update lib.rs re-exports (CRITICAL - fixes integration tests)
**Phase 4-6:** Remove forbidden imports from actor/, runtime/, messaging/
**Phase 7:** Update integration test imports (CRITICAL - fixes broken tests)
**Phase 8:** Update HostSystemManager
**Phase 9:** Delete old host_system files
**Phase 10:** Verification and testing

### Files to Modify (EXACT LIST)

| File | Line Numbers | Change Type |
|------|--------------|-------------|
| airssys-wasm/src/core/correlation.rs | ALL | CREATE (copy from host_system/correlation_tracker.rs) |
| airssys-wasm/src/core/timeout.rs | ALL | CREATE (copy from host_system/timeout_handler.rs) |
| airssys-wasm/src/core/mod.rs | N/A | Add module declarations and re-exports |
| airssys-wasm/src/lib.rs | After re-exports | Add core type re-exports (backward compatibility) |
| airssys-wasm/src/actor/mod.rs | 179, 181 | Remove forbidden imports, add core imports |
| airssys-wasm/src/runtime/async_host.rs | 932 | Remove forbidden import, add core import |
| airssys-wasm/src/messaging/messaging_service.rs | 76, 77, 734, 735 | Remove forbidden imports, add core imports |
| airssys-wasm/src/messaging/router.rs | 48 | Remove forbidden import, add core import |
| airssys-wasm/src/host_system/manager.rs | Import section, 791-808 | Update imports, add subscriber.stop() |
| airssys-wasm/src/host_system/correlation_tracker.rs | ALL | DELETE |
| airssys-wasm/src/host_system/timeout_handler.rs | ALL | DELETE |
| airssys-wasm/src/host_system/mod.rs | N/A | Remove module declarations |
| tests/correlation_integration_tests.rs | 33 | Update import from host_system to crate root |
| tests/fire_and_forget_performance_tests.rs | 43 | Update import from host_system to crate root |
| tests/send_message_host_function_tests.rs | 34 | Update import from host_system to crate root |
| tests/response_routing_integration_tests.rs | 25 | Update import from host_system to crate root |
| tests/send_request_host_function_tests.rs | 34 | Update import from host_system to crate root |

### Imports to Remove (EXACT LIST)

```
src/actor/mod.rs:179:pub use crate::host_system::correlation_tracker::CorrelationTracker;
src/actor/mod.rs:181:pub use crate::host_system::timeout_handler::TimeoutHandler;
src/runtime/async_host.rs:932:use crate::host_system::{CorrelationTracker, TimeoutHandler};
src/messaging/messaging_service.rs:76:use crate::host_system::correlation_tracker::CorrelationTracker;
src/messaging/messaging_service.rs:77:use crate::host_system::timeout_handler::TimeoutHandler;
src/messaging/messaging_service.rs:734:use crate::host_system::correlation_tracker::CorrelationTracker;
src/messaging/messaging_service.rs:735:use crate::host_system::timeout_handler::TimeoutHandler;
src/messaging/router.rs:48:use crate::host_system::correlation_tracker::CorrelationTracker;
tests/correlation_integration_tests.rs:33:use airssys_wasm::host_system::CorrelationTracker;
tests/fire_and_forget_performance_tests.rs:43:use airssys_wasm::host_system::{CorrelationTracker, TimeoutHandler};
tests/send_message_host_function_tests.rs:34:use airssys_wasm::host_system::{CorrelationTracker, TimeoutHandler};
tests/response_routing_integration_tests.rs:25:use airssys_wasm::host_system::{CorrelationTracker, TimeoutHandler};
tests/send_request_host_function_tests.rs:34:use airssys_wasm::host_system::{CorrelationTracker, TimeoutHandler};
```

### Imports to Add (EXACT LIST)

```
src/core/mod.rs:pub mod correlation;
src/core/mod.rs:pub use correlation::CorrelationTracker;
src/core/mod.rs:pub mod timeout;
src/core/mod.rs:pub use timeout::TimeoutHandler;
src/lib.rs:pub use core::{CorrelationTracker, TimeoutHandler};
src/actor/mod.rs:pub use crate::core::CorrelationTracker;
src/actor/mod.rs:pub use crate::core::TimeoutHandler;
src/runtime/async_host.rs:use crate::core::{CorrelationTracker, TimeoutHandler};
src/messaging/messaging_service.rs:use crate::core::{CorrelationTracker, TimeoutHandler};
src/messaging/router.rs:use crate::core::CorrelationTracker;
tests/correlation_integration_tests.rs:use airssys_wasm::CorrelationTracker;
tests/fire_and_forget_performance_tests.rs:use airssys_wasm::{CorrelationTracker, TimeoutHandler};
tests/send_message_host_function_tests.rs:use airssys_wasm::{CorrelationTracker, TimeoutHandler};
tests/response_routing_integration_tests.rs:use airssys_wasm::{CorrelationTracker, TimeoutHandler};
tests/send_request_host_function_tests.rs:use airssys_wasm::{CorrelationTracker, TimeoutHandler};
```

### Unit Testing Plan (COMPLETE)

**Objective:** Verify CorrelationTracker and TimeoutHandler work correctly from core/ location.

**Test 1: test_correlation_tracker_new_success** (core/correlation.rs)
- Verify CorrelationTracker creates correctly from core/ location

**Test 2: test_correlation_tracker_register_success** (core/correlation.rs)
- Verify registration of pending requests works from core/

**Test 3: test_correlation_tracker_lookup_success** (core/correlation.rs)
- Verify lookup of pending requests works from core/

**Test 4: test_timeout_handler_new_success** (core/timeout.rs)
- Verify TimeoutHandler creates correctly from core/ location

**Test 5: test_timeout_handler_register_and_fire** (core/timeout.rs)
- Verify timeout registration and firing works from core/

**Coverage Targets:**
- CorrelationTracker: 95% coverage for all public methods
- TimeoutHandler: 95% coverage for all public methods

### Integration Testing Plan (COMPLETE)

**Test 1: test_correlation_tracking_with_core_types** (tests/correlation_integration_tests.rs)
- Update import from `airssys_wasm::host_system::CorrelationTracker`
- To: `use airssys_wasm::CorrelationTracker;`

**Test 2: test_timeout_handling_with_core_types** (tests/fire_and_forget_performance_tests.rs)
- Update import from `airssys_wasm::host_system::{CorrelationTracker, TimeoutHandler}`
- To: `use airssys_wasm::{CorrelationTracker, TimeoutHandler};`

**Test 3: test_host_system_lifecycle_with_core_types** (tests/host_system-integration-tests.rs)
- No changes required (HostSystemManager internally uses core/ types)

**Test 4: test_host_system_manager_shutdown_stops_subscriber** (src/host_system/manager.rs)
- Verify shutdown() calls subscriber.stop()

**Test 5: End-to-End Message Flow** (tests/send_request_host_function_tests.rs)
- Update import from `airssys_wasm::host_system::{CorrelationTracker, TimeoutHandler}`
- To: `use airssys_wasm::{CorrelationTracker, TimeoutHandler};`

**Verification Commands:**
```bash
cargo test --package airssys-wasm --lib          # All unit tests
cargo test --package airssys-wasm --test '*'       # All integration tests
grep -rn "use crate::host_system" src/actor/   # Expected: No output
grep -rn "use crate::host_system" src/runtime/  # Expected: No output
grep -rn "use crate::host_system" src/messaging/ # Expected: No output
grep -rn "use crate::" src/core/               # Expected: No output
```

### Quality Standards

**ADR-WASM-023 Compliance:**
- ✅ No forbidden imports from actor/, runtime/, messaging/ to host_system/
- ✅ Dependency flow: All modules → core/ (allowed)
- ✅ No circular dependencies

**PROJECTS_STANDARD.md Compliance:**
- ✅ §2.1: 3-Layer import organization
- ✅ §4.3: Module architecture patterns
- ✅ §6.1: YAGNI principles
- ✅ §6.2: Avoid `dyn` patterns
- ✅ §6.4: Implementation quality gates

**Rust Guidelines:**
- ✅ M-DESIGN-FOR-AI
- ✅ M-MODULE-DOCS
- ✅ M-ERRORS-CANONICAL-STRUCTS
- ✅ M-STATIC-VERIFICATION
- ✅ M-FEATURES-ADDITIVE

### Success Criteria

1. ✅ CorrelationTracker moved to core/
2. ✅ TimeoutHandler moved to core/
3. ✅ lib.rs re-exports core types (backward compatibility)
4. ✅ All forbidden imports removed
5. ✅ All integration test imports updated
6. ✅ HostSystemManager updated to use core types
7. ✅ Old host_system files deleted
8. ✅ Documentation updated with canonical sections
9. ✅ Unit tests added and passing (5 new tests)
10. ✅ Integration tests updated and passing (5 test files)
11. ✅ Build succeeds (zero errors, zero warnings)
12. ✅ ADR-WASM-023 violations fixed (no forbidden imports)
13. ✅ Zero clippy warnings
14. ✅ All 1,042+ tests passing


## Implementation Plan (SOLID COMPLETE VERSION)

### Context & References

**ADR References:**
- **ADR-WASM-023: Module Boundary Enforcement** (MANDATORY)
- **KNOWLEDGE-WASM-036: Three-Module Architecture**
- **KNOWLEDGE-WASM-030: Module Architecture Hard Requirements**

**PROJECTS_STANDARD.md Compliance:**
- **§2.1 3-Layer Import Organization**: All modified files will follow std → external → internal import pattern
- **§4.3 Module Architecture Patterns**: mod.rs files will only contain declarations and re-exports
- **§6.1 YAGNI Principles**: Only implement required changes (re-export approach, no speculative traits)
- **§6.2 Avoid `dyn` Patterns**: Use concrete types (CorrelationTracker, TimeoutHandler in core/, no trait objects)
- **§6.4 Implementation Quality Gates**: Zero warnings, comprehensive tests, all clippy checks pass

**Rust Guidelines Applied:**
- **M-DESIGN-FOR-AI**: Idiomatic APIs with thorough documentation and testable code
- **M-MODULE-DOCS**: Module documentation will follow canonical sections (Summary, Examples, Errors, Panics)
- **M-ERRORS-CANONICAL-STRUCTS**: Error types follow canonical structure
- **M-STATIC-VERIFICATION**: All lints enabled, clippy must pass with zero warnings
- **M-FEATURES-ADDITIVE**: Changes will not break existing APIs

**Documentation Standards:**
- **Diátaxis Type**: Reference documentation for all public APIs
- **Quality**: Technical language, no marketing hyperbole
- **Canonical Sections**: All documented items will have Summary, Examples (where applicable), Errors, Panics sections

---

### PROJECTS_STANDARD.md Compliance (EXPLICIT SECTION)

**§2.1 3-Layer Import Organization:**
- All modified files will follow: std → external → internal import pattern
- Verification: Check import grouping in all modified files

**§4.3 Module Architecture Patterns:**
- mod.rs files will contain ONLY declarations and re-exports
- NO implementation code in mod.rs files

**§6.1 YAGNI Principles:**
- Move structs to core/ (simple, direct solution)
- Re-export from lib.rs (no trait abstractions needed)
- No speculative traits

**§6.2 Avoid `dyn` Patterns:**
- Use concrete CorrelationTracker and TimeoutHandler types
- No trait objects used

**§6.4 Implementation Quality Gates:**
```bash
cargo build --package airssys-wasm                    # Clean build
cargo clippy --package airssys-wasm --all-targets --all-features -- -D warnings  # Zero warnings
cargo test --package airssys-wasm --lib             # All unit tests pass
cargo test --package airssys-wasm --test '*'        # All integration tests pass
```

---

### Rust Guidelines Applied (EXPLICIT SECTION)

**M-DESIGN-FOR-AI:**
- Idiomatic APIs: CorrelationTracker and TimeoutHandler follow Rust API Guidelines
- Thorough docs: All public items have Summary, Examples, Errors, Panics sections
- Testable code: All functionality is covered by unit and integration tests

**M-MODULE-DOCS:**
- Module documentation will include Summary, Examples, Errors, Panics sections

**M-ERRORS-CANONICAL-STRUCTS:**
- All error types use thiserror derive macro
- Errors include contextual information

**M-STATIC-VERIFICATION:**
- All lints enabled in clippy.toml
- Zero clippy warnings required

**M-FEATURES-ADDITIVE:**
- Changes are additive (no breaking changes to existing APIs)
- Re-export from lib.rs maintains backward compatibility

---

### Documentation Standards (EXPLICIT SECTION)

**Diátaxis Framework Compliance:**
- **Type**: Reference documentation
- **Purpose**: Provide technical descriptions of APIs
- **Characteristics**: Information-oriented, austere, authoritative

**Quality Standards:**
- No hyperbolic terms
- Technical precision: All claims are measurable
- Concrete examples: All APIs have working code examples

**Canonical Sections:**
All documented items MUST have:
1. **Summary sentence** (< 15 words)
2. **Extended documentation** (free form)
3. **# Examples** (working code examples)
4. **# Errors** (list known error conditions)
5. **# Panics** (list when panic may happen)

---

### Module Architecture

**Code will be placed in:** `core/`

**Module responsibilities (per ADR-WASM-023):**
- `core/` owns shared data types that multiple modules need
- CorrelationTracker and TimeoutHandler are shared types (used by actor/, runtime/, messaging/, host_system/)

**Forbidden imports verified:**
- **core/ MUST NOT import from**: actor/, runtime/, security/, messaging/, host_system/

---

### Files to Modify (EXACT LIST)

| File | Line Numbers | Change Type |
|------|--------------|-------------|
| src/core/correlation.rs | ALL | CREATE (copy from host_system/correlation_tracker.rs) |
| src/core/timeout.rs | ALL | CREATE (copy from host_system/timeout_handler.rs) |
| src/core/mod.rs | N/A | Add module declarations and re-exports |
| src/lib.rs | After re-exports | Add core type re-exports (backward compatibility) |
| src/actor/mod.rs | 179, 181 | Remove forbidden imports, add core imports |
| src/runtime/async_host.rs | 932 | Remove forbidden import, add core import |
| src/messaging/messaging_service.rs | 76, 77, 734, 735 | Remove forbidden imports, add core imports |
| src/messaging/router.rs | 48 | Remove forbidden import, add core import |
| src/host_system/manager.rs | Import section, 791-808 | Update imports, add subscriber.stop() call |
| src/host_system/correlation_tracker.rs | ALL | DELETE |
| src/host_system/timeout_handler.rs | ALL | DELETE |
| src/host_system/mod.rs | N/A | Remove module declarations |
| tests/correlation_integration_tests.rs | 33 | Update import from host_system to crate root |
| tests/fire_and_forget_performance_tests.rs | 43 | Update import from host_system to crate root |
| tests/send_message_host_function_tests.rs | 34 | Update import from host_system to crate root |
| tests/response_routing_integration_tests.rs | 25 | Update import from host_system to crate root |
| tests/send_request_host_function_tests.rs | 34 | Update import from host_system to crate root |

---

### Imports to Remove (EXACT LIST)

```
src/actor/mod.rs:179:pub use crate::host_system::correlation_tracker::CorrelationTracker;
src/actor/mod.rs:181:pub use crate::host_system::timeout_handler::TimeoutHandler;
src/runtime/async_host.rs:932:use crate::host_system::{CorrelationTracker, TimeoutHandler};
src/messaging/messaging_service.rs:76:use crate::host_system::correlation_tracker::CorrelationTracker;
src/messaging/messaging_service.rs:77:use crate::host_system::timeout_handler::TimeoutHandler;
src/messaging/messaging_service.rs:734:use crate::host_system::correlation_tracker::CorrelationTracker;
src/messaging/messaging_service.rs:735:use crate::host_system::timeout_handler::TimeoutHandler;
src/messaging/router.rs:48:use crate::host_system::correlation_tracker::CorrelationTracker;
tests/correlation_integration_tests.rs:33:use airssys_wasm::host_system::CorrelationTracker;
tests/fire_and_forget_performance_tests.rs:43:use airssys_wasm::host_system::{CorrelationTracker, TimeoutHandler};
tests/send_message_host_function_tests.rs:34:use airssys_wasm::host_system::{CorrelationTracker, TimeoutHandler};
tests/response_routing_integration_tests.rs:25:use airssys_wasm::host_system::{CorrelationTracker, TimeoutHandler};
tests/send_request_host_function_tests.rs:34:use airssys_wasm::host_system::{CorrelationTracker, TimeoutHandler};
```

---

### Imports to Add (EXACT LIST)

```
src/core/mod.rs:pub mod correlation;
src/core/mod.rs:pub use correlation::CorrelationTracker;
src/core/mod.rs:pub mod timeout;
src/core/mod.rs:pub use timeout::TimeoutHandler;
src/lib.rs:pub use core::{CorrelationTracker, TimeoutHandler};
src/actor/mod.rs:pub use crate::core::CorrelationTracker;
src/actor/mod.rs:pub use crate::core::TimeoutHandler;
src/runtime/async_host.rs:use crate::core::{CorrelationTracker, TimeoutHandler};
src/messaging/messaging_service.rs:use crate::core::{CorrelationTracker, TimeoutHandler};
src/messaging/router.rs:use crate::core::CorrelationTracker;
tests/correlation_integration_tests.rs:use airssys_wasm::CorrelationTracker;
tests/fire_and_forget_performance_tests.rs:use airssys_wasm::{CorrelationTracker, TimeoutHandler};
tests/send_message_host_function_tests.rs:use airssys_wasm::{CorrelationTracker, TimeoutHandler};
tests/response_routing_integration_tests.rs:use airssys_wasm::{CorrelationTracker, TimeoutHandler};
tests/send_request_host_function_tests.rs:use airssys_wasm::{CorrelationTracker, TimeoutHandler};
```

---

### Unit Testing Plan (COMPLETE)

**Objective:** Verify CorrelationTracker and TimeoutHandler work correctly from core/ location.

**Test 1: test_correlation_tracker_new_success** (core/correlation.rs)
- Verify CorrelationTracker creates correctly from core/ location

**Test 2: test_correlation_tracker_register_success** (core/correlation.rs)
- Verify registration of pending requests works from core/

**Test 3: test_correlation_tracker_lookup_success** (core/correlation.rs)
- Verify lookup of pending requests works from core/

**Test 4: test_timeout_handler_new_success** (core/timeout.rs)
- Verify TimeoutHandler creates correctly from core/ location

**Test 5: test_timeout_handler_register_and_fire** (core/timeout.rs)
- Verify timeout registration and firing works from core/

**Coverage Targets:**
- CorrelationTracker: 95% coverage for all public methods
- TimeoutHandler: 95% coverage for all public methods

---

### Integration Testing Plan (COMPLETE)

**Test 1: test_correlation_tracking_with_core_types** (tests/correlation_integration_tests.rs)
- Update import: `use airssys_wasm::host_system::CorrelationTracker;` → `use airssys_wasm::CorrelationTracker;`
- Verify CorrelationTracker from core/ works in real usage

**Test 2: test_timeout_handling_with_core_types** (tests/fire_and_forget_performance_tests.rs)
- Update import: `use airssys_wasm::host_system::{CorrelationTracker, TimeoutHandler};` → `use airssys_wasm::{CorrelationTracker, TimeoutHandler};`
- Verify TimeoutHandler from core/ works in real usage

**Test 3: test_host_system_lifecycle_with_core_types** (tests/host_system-integration-tests.rs)
- No changes required (HostSystemManager internally uses core/ types)
- Verify complete lifecycle works correctly

**Test 4: test_host_system_manager_shutdown_stops_subscriber** (src/host_system/manager.rs)
- Verify shutdown() calls subscriber.stop()

**Test 5: End-to-End Message Flow** (tests/send_request_host_function_tests.rs)
- Update import: `use airssys_wasm::host_system::{CorrelationTracker, TimeoutHandler};` → `use airssys_wasm::{CorrelationTracker, TimeoutHandler};`
- Verify complete message flow works correctly

---

### Verification Commands

```bash
# 1. Build verification
cargo build --package airssys-wasm
# Expected: Clean build, zero errors

# 2. Clippy verification
cargo clippy --package airssys-wasm --all-targets --all-features -- -D warnings
# Expected: Zero warnings

# 3. Unit tests
cargo test --package airssys-wasm --lib
# Expected: All tests pass

# 4. Integration tests
cargo test --package airssys-wasm --test '*'
# Expected: All tests pass

# 5. ADR-WASM-023 architecture verification (CRITICAL)
grep -rn "use crate::host_system" src/actor/
grep -rn "use crate::host_system" src/runtime/
grep -rn "use crate::host_system" src/messaging/
grep -rn "use crate::actor" src/runtime/
grep -rn "use crate::" src/core/
# Expected: No output (all forbidden imports removed)

# 6. Verify integration tests use re-exports
grep -rn "use airssys_wasm::host_system::" tests/
# Expected: No output (all tests use crate-level re-exports)
```

---

### Quality Standards

**ADR-WASM-023 Compliance:**
- ✅ No forbidden imports from actor/, runtime/, messaging/ to host_system/
- ✅ Dependency flow: All modules → core/ (allowed)
- ✅ No circular dependencies

**PROJECTS_STANDARD.md Compliance:**
- ✅ §2.1: 3-Layer import organization
- ✅ §4.3: Module architecture patterns
- ✅ §6.1: YAGNI principles
- ✅ §6.2: Avoid `dyn` patterns
- ✅ §6.4: Implementation quality gates

**Rust Guidelines:**
- ✅ M-DESIGN-FOR-AI
- ✅ M-MODULE-DOCS
- ✅ M-ERRORS-CANONICAL-STRUCTS
- ✅ M-STATIC-VERIFICATION
- ✅ M-FEATURES-ADDITIVE

---

### Success Criteria

1. ✅ CorrelationTracker moved to core/
2. ✅ TimeoutHandler moved to core/
3. ✅ lib.rs re-exports core types (backward compatibility)
4. ✅ All forbidden imports removed
5. ✅ All integration test imports updated (5 test files)
6. ✅ HostSystemManager updated to use core types
7. ✅ Old host_system files deleted
8. ✅ Documentation updated with canonical sections
9. ✅ Unit tests added and passing (5 new tests)
10. ✅ Integration tests updated and passing (5 test files)
11. ✅ Build succeeds (zero errors, zero warnings)
12. ✅ ADR-WASM-023 violations fixed (no forbidden imports)
13. ✅ Zero clippy warnings
14. ✅ All 1,042+ tests passing

