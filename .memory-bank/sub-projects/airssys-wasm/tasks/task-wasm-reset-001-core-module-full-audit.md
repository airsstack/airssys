# TASK-WASM-RESET-001: Core Module Full Audit

**Task ID:** TASK-WASM-RESET-001  
**Created:** 2025-12-22  
**Status:** ✅ COMPLETE (Audit finished, see findings below)  
**Priority:** CRITICAL - BLOCKING  
**Type:** Audit & Architecture Reset

---

## ⚠️ CONTEXT: PROJECT RESET TO SQUARE ONE

This task was created because **AI agents made fatal mistakes** that broke the architecture.
We are resetting development to the `core/` module - the foundation of everything.

**Root Cause:** AI agents claimed "verified" without running actual commands, proceeded without reading ADRs, and created architecture violations that went undetected.

---

## AUDIT RESULTS (2025-12-22)

### Summary Statistics

| Metric | Value |
|--------|-------|
| **Total Files Audited** | 23 |
| **Files with Import Violations** | 2 |
| **Files with Classification Issues** | 3 |
| **Total Violations Found** | 5 |
| **Previous Audit Found** | 1 |
| **Missed by Previous Audit** | 4 |

### Verification Status

- **Audited By:** Memory Bank Manager (full re-audit)
- **Verified By:** @memorybank-verifier
- **Verification Result:** ✅ VERIFIED
- **Date:** 2025-12-22

---

## 🚨 VIOLATIONS FOUND (5 Total)

### Violation 1: config.rs:82 - Forbidden Internal Import
```
airssys-wasm/src/core/config.rs:82
use crate::runtime::limits::{CpuConfig, MemoryConfig, ResourceConfig, ResourceLimits};
```
- **Rule Violated**: ADR-WASM-011, ADR-WASM-012 (zero internal dependencies)
- **Severity**: 🔴 CRITICAL
- **Fix**: Move `CpuConfig`, `MemoryConfig`, `ResourceConfig`, `ResourceLimits` from `runtime/limits.rs` to `core/config.rs`

### Violation 2: runtime.rs:71 - Wasmtime Import in Core (NEW)
```
airssys-wasm/src/core/runtime.rs:71
use wasmtime::component::Component;
```
- **Rule Violated**: ADR-WASM-012 (no external crate integrations in core/)
- **Severity**: 🔴 CRITICAL
- **Note**: Per ADR-WASM-012 line 159: "❌ External crate integrations (wasmtime-specific code)"
- **Fix**: Move `ComponentHandle` (which wraps `Arc<Component>`) to `runtime/` module. core/runtime.rs should only define traits.

### Violation 3: rate_limiter.rs - Implementation Logic in Core (NEW)
```
File: airssys-wasm/src/core/rate_limiter.rs (334 lines)
```
- **Problem**: Contains actual sliding window rate limiting implementation, not abstractions
- **Rule Violated**: ADR-WASM-012 line 160: "❌ Business logic (algorithm implementations)"
- **Severity**: 🟡 MEDIUM
- **Fix**: Move to `security/rate_limiter.rs`, keep only trait `RateLimiter` in core/ if needed

### Violation 4: permission_checker.rs - Implementation Logic in Core (NEW)
```
File: airssys-wasm/src/core/permission_checker.rs (840 lines)
Lines 54-55:
use glob::Pattern as GlobPattern;
use lru::LruCache;
```
- **Problem**: Contains 840 lines of implementation logic (pattern matching, LRU caching)
- **Rule Violated**: ADR-WASM-012 line 160: "❌ Business logic (algorithm implementations)"
- **Severity**: 🟡 MEDIUM
- **Note**: The glob/lru crates are general-purpose (like serde), so the violation is the implementation logic, not necessarily the crate usage
- **Fix**: Move to `security/permission_checker.rs`, keep only trait in core/

### Violation 5: permission_wit.rs - Implementation Logic in Core (NEW)
```
File: airssys-wasm/src/core/permission_wit.rs (724 lines)
```
- **Problem**: Contains WIT integration implementation code (type conversions, host function wrappers)
- **Rule Violated**: ADR-WASM-012 line 159-160: Integration code, not abstraction
- **Severity**: 🟡 MEDIUM
- **Fix**: Move to `runtime/permission_wit.rs` or dedicated `wit/` module

---

## 📊 FILE CLASSIFICATION (All 23 Files)

### Files That Correctly Belong in core/ (18 files)

| File | Lines | Status | Notes |
|------|-------|--------|-------|
| mod.rs | 237 | ✅ PASS | Module declarations |
| error.rs | 1480 | ✅ PASS | Error types (core abstractions) |
| capability.rs | 1045 | ✅ PASS | Pure enums/structs |
| component.rs | 818 | ✅ PASS | Core types |
| messaging.rs | 815 | ✅ PASS | Message abstractions |
| observability.rs | 752 | ✅ PASS | Traits/types only |
| management.rs | 617 | ✅ PASS | Trait definitions |
| manifest.rs | 599 | ✅ PASS | Manifest types |
| permission.rs | 581 | ✅ PASS | Permission types |
| lifecycle.rs | 569 | ✅ PASS | Lifecycle types |
| storage.rs | 557 | ✅ PASS | Storage traits |
| multicodec_prefix.rs | 547 | ✅ PASS | Validation constants |
| interface.rs | 529 | ✅ PASS | WIT interface types |
| bridge.rs | 504 | ✅ PASS | Host function traits |
| multicodec.rs | 491 | ✅ PASS | Codec types |
| security.rs | 455 | ✅ PASS | Security policy traits |
| actor.rs | 438 | ✅ PASS | Actor message types |
| component_message.rs | 356 | ✅ PASS | Message types |

### Files That Need Fixes (2 files)

| File | Lines | Issue | Fix Required |
|------|-------|-------|--------------|
| config.rs | 1372 | ⚠️ VIOLATION #1 | Remove import from runtime/ |
| runtime.rs | 562 | ⚠️ VIOLATION #2 | Remove wasmtime import |

### Files That Should Be Moved (3 files)

| File | Lines | Should Move To | Reason |
|------|-------|----------------|--------|
| rate_limiter.rs | 334 | ❌ → security/ | Implementation logic |
| permission_checker.rs | 840 | ❌ → security/ | Implementation logic |
| permission_wit.rs | 724 | ❌ → runtime/ or wit/ | WIT integration impl |

---

## 🔧 REMEDIATION PLAN

### Priority 1: Fix Import Violations (BLOCKING)

**Fix 1.1: Move ResourceLimits to core/ (Violation #1)**
```
Action: Move CpuConfig, MemoryConfig, ResourceConfig, ResourceLimits 
        from runtime/limits.rs to core/config.rs (or new core/limits.rs)
Update: config.rs line 82 → use super::limits::{...}
Effort: 2-3 hours
Risk: Low - type move, no logic change
Task: TASK-WASM-RESET-002
```

**Fix 1.2: Remove wasmtime from core/runtime.rs (Violation #2)**
```
Action: Move ComponentHandle (Arc<wasmtime::Component>) to runtime/engine.rs
        Keep only RuntimeEngine trait and ExecutionContext in core/
Update: core/runtime.rs line 71 → remove wasmtime import
Effort: 1-2 hours
Risk: Medium - affects API surface
Task: TASK-WASM-RESET-002 (extended scope)
```

### Priority 2: Move Implementation Files (After Priority 1)

**Fix 2.1: Move rate_limiter.rs (Violation #3)**
```
From: src/core/rate_limiter.rs
To:   src/security/rate_limiter.rs
Keep in core/: Only RateLimiter trait (if needed)
Effort: 2-3 hours
Risk: Low - isolated module
Task: TASK-WASM-RESET-003
```

**Fix 2.2: Move permission_checker.rs (Violation #4)**
```
From: src/core/permission_checker.rs
To:   src/security/permission_checker.rs
Keep in core/: Only PermissionChecker trait (if needed)
Effort: 3-4 hours
Risk: Medium - widely used
Task: TASK-WASM-RESET-003
```

**Fix 2.3: Move permission_wit.rs (Violation #5)**
```
From: src/core/permission_wit.rs
To:   src/runtime/permission_wit.rs (or src/wit/)
Keep in core/: Nothing - pure WIT integration
Effort: 2-3 hours
Risk: Medium - WIT integration changes
Task: TASK-WASM-RESET-003
```

---

## ✅ VERIFICATION EVIDENCE

### Command 1: Forbidden Internal Imports
```bash
$ grep -rn "use crate::" airssys-wasm/src/core/ | grep -v "use crate::core"
airssys-wasm/src/core//config.rs:82:use crate::runtime::limits::{CpuConfig, MemoryConfig, ResourceConfig, ResourceLimits};
```
**Result:** 1 internal import violation found

### Command 2: Wasmtime in Core
```bash
$ grep -rn "wasmtime" airssys-wasm/src/core/
airssys-wasm/src/core//runtime.rs:71:use wasmtime::component::Component;
```
**Result:** 1 wasmtime import found (forbidden per ADR-WASM-012)

### Command 3: Implementation Crates in Core
```bash
$ grep -n "use glob::\|use lru::" airssys-wasm/src/core/*.rs
airssys-wasm/src/core/permission_checker.rs:54:use glob::Pattern as GlobPattern;
airssys-wasm/src/core/permission_checker.rs:55:use lru::LruCache;
```
**Result:** Found implementation crates (indicator of implementation logic)

### Command 4: Line Counts
```bash
$ wc -l airssys-wasm/src/core/*.rs | sort -n | tail -10
     752 airssys-wasm/src/core/observability.rs
     815 airssys-wasm/src/core/messaging.rs
     818 airssys-wasm/src/core/component.rs
     840 airssys-wasm/src/core/permission_checker.rs
    1045 airssys-wasm/src/core/capability.rs
    1372 airssys-wasm/src/core/config.rs
    1480 airssys-wasm/src/core/error.rs
   15222 total
```
**Result:** 15,222 total lines, 23 files

### Command 5: Build Check
```bash
$ cargo build -p airssys-wasm
Finished `dev` profile [unoptimized + debuginfo] target(s) in 0.81s
```
**Result:** ✅ PASSES

### Command 6: Clippy Check
```bash
$ cargo clippy -p airssys-wasm --lib -- -D warnings
Finished `dev` profile [unoptimized + debuginfo] target(s) in 0.27s
```
**Result:** ✅ PASSES (no warnings in lib)

---

## SUCCESS CRITERIA

| Criteria | Status |
|----------|--------|
| ALL 23 files in core/ audited | ✅ COMPLETE |
| ALL violations documented with file:line precision | ✅ COMPLETE (5 violations) |
| ALL files classified (belongs in core or not) | ✅ COMPLETE (18/2/3) |
| Remediation plan created for each violation | ✅ COMPLETE |
| Verification commands run and output captured | ✅ COMPLETE |
| Report reviewed and verified | ✅ VERIFIED by @memorybank-verifier |

---

## NEXT STEPS

1. **TASK-WASM-RESET-002**: Fix Priority 1 violations (config.rs, runtime.rs)
2. **TASK-WASM-RESET-003**: Move implementation files (rate_limiter.rs, permission_checker.rs, permission_wit.rs)

---

## PRIMARY REFERENCES

### Foundational ADRs
1. **ADR-WASM-011**: Module Structure Organization
2. **ADR-WASM-012**: Comprehensive Core Abstractions Strategy (key rules at lines 157-161, 220-223)

### Lessons Learned
1. **KNOWLEDGE-WASM-033**: AI Fatal Mistakes - Lessons Learned

---

**Created By:** Memory Bank Manager  
**Date:** 2025-12-22  
**Audit Completed:** 2025-12-22  
**Verified By:** @memorybank-verifier  
**Reason:** Architecture reset due to AI fatal mistakes
