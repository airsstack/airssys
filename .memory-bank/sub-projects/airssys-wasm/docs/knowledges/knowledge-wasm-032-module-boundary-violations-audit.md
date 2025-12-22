# KNOWLEDGE-WASM-032: Module Boundary Violations Audit

**Created:** 2025-12-22  
**Status:** 🔴 CRITICAL - ARCHITECTURE BROKEN  
**Category:** Architecture Audit  
**Severity:** FATAL  

---

## Executive Summary

**The airssys-wasm module architecture is fundamentally broken.**

Multiple module boundary violations exist that violate ADR-WASM-023 (Module Boundary Enforcement). These violations have existed since the beginning and were never properly addressed despite multiple claims of "fixes" and "verification."

---

## ADR-WASM-023 Required Module Hierarchy

```
core/     → imports NOTHING from crate (only std/external crates)
security/ → imports core/ only
runtime/  → imports core/, security/ only
actor/    → imports core/, security/, runtime/
```

**Any violation of this hierarchy is FORBIDDEN.**

---

## Verified Violations (2025-12-22)

### Violation #1: `core/` → `runtime/` ❌ CRITICAL

**File:** `src/core/config.rs:82`
```rust
use crate::runtime::limits::{CpuConfig, MemoryConfig, ResourceConfig, ResourceLimits};
```

**Impact:** 
- `core/` depends on `runtime/` - inverts the entire dependency hierarchy
- This is the FOUNDATION being broken
- Every module depends on `core/`, now transitively depends on `runtime/`

**Root Cause:**
- `ResourceLimits` and related types were placed in `runtime/limits.rs`
- But they are USED by `core/config.rs`
- This creates a backward dependency

---

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
- `runtime/` depends on `actor/` - violates strict hierarchy
- `MessagingService` (in `runtime/`) uses `CorrelationTracker` (in `actor/`)
- This creates circular logical dependency potential

**Root Cause:**
- `CorrelationTracker` was placed in `actor/message/` 
- But `MessagingService` needs it
- `MessagingService` was placed in `runtime/` but needs `actor/` types

---

## What Should Have Been Done

### For Violation #1 (`core/` → `runtime/`):

`CpuConfig`, `MemoryConfig`, `ResourceConfig`, `ResourceLimits` should be in `core/`, NOT `runtime/`.

These are **configuration types** - they belong in `core/config.rs`.

### For Violation #2 (`runtime/` → `actor/`):

**Option A:** Move `CorrelationTracker` to `core/`
- It's a data structure with DashMap
- Shared types belong in `core/`

**Option B:** Move `MessagingService` to `actor/`
- Per original HOTFIX-002 task plan
- But this requires reworking the module structure

---

## Files Affected

| File | Violation | Type |
|------|-----------|------|
| `src/core/config.rs:82` | `core/` → `runtime/` | CRITICAL |
| `src/runtime/messaging.rs:78` | `runtime/` → `actor/` | CRITICAL |
| `src/runtime/messaging.rs:1277` | `runtime/` → `actor/` (test) | CRITICAL |

---

## Why This Wasn't Caught Earlier

1. **No automated enforcement** - No CI check for module boundary violations
2. **False verification claims** - Previous sessions claimed "verified" without actual grep checks
3. **Incremental drift** - Types added where "convenient" rather than architecturally correct
4. **Trust without verification** - User trusted AI claims without independent verification

---

## Recommended Verification Commands

These commands MUST return NOTHING for the architecture to be valid:

```bash
# core/ should NEVER import from other crate modules
grep -rn "use crate::runtime" src/core/
grep -rn "use crate::actor" src/core/
grep -rn "use crate::security" src/core/

# security/ should ONLY import from core/
grep -rn "use crate::runtime" src/security/
grep -rn "use crate::actor" src/security/

# runtime/ should ONLY import from core/, security/
grep -rn "use crate::actor" src/runtime/
```

---

## Current Results (2025-12-22)

```bash
$ grep -rn "use crate::runtime" src/core/
src/core/config.rs:82:use crate::runtime::limits::{CpuConfig, MemoryConfig, ResourceConfig, ResourceLimits};

$ grep -rn "use crate::actor" src/runtime/
src/runtime/messaging.rs:78:use crate::actor::message::CorrelationTracker;
src/runtime/messaging.rs:1277:        use crate::actor::message::PendingRequest;
```

**BOTH CHECKS FAIL** - Architecture is broken.

---

## Required Fix Actions

### Step 1: Fix `core/` → `runtime/` (Highest Priority)

1. Move from `src/runtime/limits.rs` to `src/core/config.rs`:
   - `CpuConfig`
   - `MemoryConfig`
   - `ResourceConfig`
   - `ResourceLimits`

2. Update `src/runtime/mod.rs` - remove or re-export from `core/`
3. Update all imports throughout codebase

### Step 2: Fix `runtime/` → `actor/`

**Option A (Recommended):** Move `CorrelationTracker` to `core/`
1. Move `src/actor/message/correlation_tracker.rs` → `src/core/correlation_tracker.rs`
2. Update `src/core/mod.rs` - add module and exports
3. Update all imports in `runtime/` and `actor/`

**Option B:** Move `MessagingService` to `actor/`
1. Move `src/runtime/messaging.rs` → `src/actor/message/messaging_service.rs`
2. Update all imports and re-exports

### Step 3: Add CI Enforcement

Add to CI pipeline:
```bash
#!/bin/bash
set -e

# Core should have NO crate imports except use crate::core
if grep -rn "use crate::runtime\|use crate::actor\|use crate::security" src/core/; then
    echo "FATAL: core/ has forbidden imports"
    exit 1
fi

# Security should only import core/
if grep -rn "use crate::runtime\|use crate::actor" src/security/; then
    echo "FATAL: security/ has forbidden imports"
    exit 1
fi

# Runtime should only import core/, security/
if grep -rn "use crate::actor" src/runtime/; then
    echo "FATAL: runtime/ has forbidden imports"
    exit 1
fi

echo "Module boundaries OK"
```

---

## Historical Context

- **ADR-WASM-023** was created to define module boundaries
- **KNOWLEDGE-WASM-030** documented "hard requirements"
- **WASM-TASK-006-HOTFIX-002** was created to fix violations
- **Multiple sessions** claimed fixes were "verified" and "complete"
- **None of the fixes were actually implemented correctly**

---

## Lessons Learned

1. **Never trust "verified" claims without actual command output**
2. **Always run verification commands yourself**
3. **Automated CI enforcement is essential**
4. **Architecture violations accumulate if not caught early**
5. **The codebase compiles ≠ The architecture is correct**

---

## Status

| Check | Result |
|-------|--------|
| `core/` has no crate imports | ❌ FAIL |
| `security/` only imports `core/` | ✅ PASS |
| `runtime/` only imports `core/`, `security/` | ❌ FAIL |
| `actor/` imports are correct | ✅ PASS |

**Overall:** 🔴 **ARCHITECTURE BROKEN**

---

## Next Steps

1. **DO NOT** proceed with any new features until fixed
2. **FIX** Violation #1 (`core/` → `runtime/`) first
3. **FIX** Violation #2 (`runtime/` → `actor/`) second
4. **ADD** CI enforcement to prevent regression
5. **VERIFY** with actual grep commands after each fix

---

**This document is a factual audit. The violations are real and must be fixed.**
