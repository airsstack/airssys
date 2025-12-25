# HOTFIX: Module Boundary Violations

**Task ID:** WASM-TASK-006-HOTFIX-002  
**Created:** 2025-12-22  
**Updated:** 2025-12-22  
**Priority:** 🔴 **CRITICAL - BLOCKING**  
**Status:** ❌ **NOT COMPLETE** (Despite previous claims)  
**Blocks:** Task 3.3, all future work  
**Estimated Effort:** 4-6 hours

---

## 🔴 CURRENT STATUS: NOT COMPLETE

**Verified on 2025-12-22:** The violations still exist.

```bash
$ grep -rn "use crate::runtime" airssys-wasm/src/core/
src/core/config.rs:82:use crate::runtime::limits::{CpuConfig, MemoryConfig, ResourceConfig, ResourceLimits};

$ grep -rn "use crate::actor" airssys-wasm/src/runtime/
src/runtime/messaging.rs:78:use crate::actor::message::CorrelationTracker;
src/runtime/messaging.rs:1277:        use crate::actor::message::PendingRequest;
```

**Previous claims of "COMPLETE" were FALSE.**

---

## Problem Statement

Two critical module boundary violations exist:

### Violation #1: `core/` → `runtime/` ❌ CRITICAL (NEW FINDING)

**File:** `src/core/config.rs:82`
```rust
use crate::runtime::limits::{CpuConfig, MemoryConfig, ResourceConfig, ResourceLimits};
```

**Impact:** 
- Inverts the entire dependency hierarchy
- `core/` is supposed to have ZERO crate imports
- Every module depends on `core/`, creating transitive issues

### Violation #2: `runtime/` → `actor/` ❌ CRITICAL

**File:** `src/runtime/messaging.rs:78`
```rust
use crate::actor::message::CorrelationTracker;
```

**File:** `src/runtime/messaging.rs:1277` (test code)
```rust
use crate::actor::message::PendingRequest;
```

**Impact:**
- `runtime/` should only import from `core/`, `security/`
- Creates circular dependency potential

---

## Required Module Structure (Per ADR-WASM-023)

```
core/     → imports NOTHING from crate
security/ → imports core/ only
runtime/  → imports core/, security/ only
actor/    → imports core/, security/, runtime/
```

**FORBIDDEN:**
- `core/` → anything
- `runtime/` → `actor/`

---

## Fix Actions (REVISED)

### Step 1: Fix `core/` → `runtime/` (HIGHEST PRIORITY)

**Move from `src/runtime/limits.rs` to `src/core/config.rs`:**

| Type | Current Location | New Location |
|------|------------------|--------------|
| `CpuConfig` | `runtime/limits.rs` | `core/config.rs` |
| `MemoryConfig` | `runtime/limits.rs` | `core/config.rs` |
| `ResourceConfig` | `runtime/limits.rs` | `core/config.rs` |
| `ResourceLimits` | `runtime/limits.rs` | `core/config.rs` |

**Implementation:**
1. Copy type definitions to `src/core/config.rs`
2. Update `src/core/mod.rs` to export these types
3. Update `src/core/config.rs` to remove the import from `runtime/`
4. Update `src/runtime/limits.rs` to import from `core/` (or delete if empty)
5. Update all other imports throughout codebase

---

### Step 2: Fix `runtime/` → `actor/` 

**Option A (Recommended): Move `CorrelationTracker` to `core/`**

| Type | Current Location | New Location |
|------|------------------|--------------|
| `CorrelationTracker` | `actor/message/correlation_tracker.rs` | `core/correlation_tracker.rs` |

**Implementation:**
1. Move `src/actor/message/correlation_tracker.rs` → `src/core/correlation_tracker.rs`
2. Update `src/core/mod.rs` to export `CorrelationTracker`
3. Update `src/runtime/messaging.rs` to import from `core/`
4. Update `src/actor/message/mod.rs` to re-export from `core/`
5. Update all other imports

**Option B: Move `MessagingService` to `actor/`**

| Type | Current Location | New Location |
|------|------------------|--------------|
| `MessagingService` | `runtime/messaging.rs` | `actor/message/messaging_service.rs` |
| `ResponseRouter` | `runtime/messaging.rs` | `actor/message/messaging_service.rs` |

**Implementation:**
1. Move `src/runtime/messaging.rs` → `src/actor/message/messaging_service.rs`
2. Update `src/runtime/mod.rs` - remove messaging module
3. Update `src/actor/message/mod.rs` - add messaging_service module
4. Update all imports throughout codebase

---

## Verification (MUST ALL PASS)

**After all fixes, run:**

```bash
# MUST return NOTHING
grep -rn "use crate::runtime" src/core/

# MUST return NOTHING  
grep -rn "use crate::actor" src/core/

# MUST return NOTHING
grep -rn "use crate::security" src/core/

# MUST return NOTHING
grep -rn "use crate::actor" src/runtime/

# Build must succeed
cargo build -p airssys-wasm

# Tests must pass
cargo test -p airssys-wasm

# Zero clippy warnings
cargo clippy -p airssys-wasm --all-targets --all-features -- -D warnings
```

**ALL CHECKS MUST PASS. NO EXCEPTIONS.**

---

## Files to Modify

| File | Action |
|------|--------|
| `src/core/config.rs` | **MODIFY** - Add resource types, remove runtime import |
| `src/core/mod.rs` | **MODIFY** - Add exports for resource types |
| `src/runtime/limits.rs` | **MODIFY** - Import from core/, or DELETE if empty |
| `src/core/correlation_tracker.rs` | **CREATE** - Move from actor/ (Option A) |
| `src/runtime/messaging.rs` | **MODIFY** - Import CorrelationTracker from core/ |
| `src/actor/message/correlation_tracker.rs` | **DELETE** or re-export from core/ |
| `src/actor/message/mod.rs` | **MODIFY** - Update exports |

---

## Success Criteria

| Criterion | Verification |
|-----------|--------------|
| Zero `core/` → `runtime/` imports | `grep -rn "use crate::runtime" src/core/` returns nothing |
| Zero `core/` → `actor/` imports | `grep -rn "use crate::actor" src/core/` returns nothing |
| Zero `core/` → `security/` imports | `grep -rn "use crate::security" src/core/` returns nothing |
| Zero `runtime/` → `actor/` imports | `grep -rn "use crate::actor" src/runtime/` returns nothing |
| Code compiles | `cargo build` succeeds |
| All tests pass | `cargo test` succeeds |
| Zero clippy warnings | `cargo clippy -- -D warnings` succeeds |

---

## History of False Claims

| Date | Claim | Reality |
|------|-------|---------|
| Previous | "Hotfix Phase 1 COMPLETE" | `runtime/` still imports from `actor/` |
| Previous | "Hotfix Phase 2 COMPLETE" | `core/` imports from `runtime/` |
| Previous | "Verified with grep" | Grep was never actually run |
| 2025-12-22 | Audit performed | Violations confirmed |

---

## Guarantee

**This task is NOT complete until:**
1. ALL grep verification commands return NOTHING
2. `cargo build` succeeds
3. `cargo test` succeeds  
4. `cargo clippy -- -D warnings` succeeds
5. User has seen the actual command output (not just claims)

---

**Created:** 2025-12-22  
**Updated:** 2025-12-22  
**Author:** Architecture Audit  
**Priority:** 🔴 CRITICAL - Must complete before any other work
