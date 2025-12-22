# airssys-wasm Tasks Index

**Sub-Project:** airssys-wasm  
**Last Updated:** 2025-12-22  
**Status:** 🔴 **ARCHITECTURE RESET IN PROGRESS**

---

## ⚠️ CRITICAL NOTICE: PROJECT RESET TO SQUARE ONE

### Why We Reset

**Date:** 2025-12-22  
**Reason:** AI agents made fatal mistakes that broke the architecture.

The `core/` module - the foundation of everything - has architecture violations.
All previous work was built on a broken foundation.

We are starting over from the `core/` module.

---

## Active Tasks

| Task ID | Name | Status | Priority |
|---------|------|--------|----------|
| **TASK-WASM-RESET-001** | Core Module Full Audit | ✅ **COMPLETE** | CRITICAL |
| **TASK-WASM-RESET-002** | Fix Core Architecture Violations | 🔴 NOT STARTED | CRITICAL |
| **TASK-WASM-RESET-003** | Move Implementation Files | 🔴 NOT STARTED | HIGH |

### TASK-WASM-RESET-001: Core Module Full Audit ✅
- **File:** `task-wasm-reset-001-core-module-full-audit.md`
- **Purpose:** Audit ALL 23 files in `core/` module for architecture violations
- **Scope:** Identify violations, classify files, create remediation plan
- **Status:** ✅ **COMPLETE** - 2025-12-22
- **Verified By:** @memorybank-verifier
- **Result:** **5 violations found** (not 1 as previously claimed)

### TASK-WASM-RESET-002: Fix Core Architecture Violations 🔴
- **File:** `task-wasm-reset-002-fix-core-architecture-violations.md`
- **Purpose:** Fix import violations found in TASK-WASM-RESET-001
- **Scope:** 
  - Move types from `runtime/limits.rs` to `core/config.rs` (Violation #1)
  - Move `ComponentHandle` from `core/runtime.rs` to `runtime/` (Violation #2)
- **Blocking:** ALL other work blocked until this completes
- **Depends On:** TASK-WASM-RESET-001 ✅

### TASK-WASM-RESET-003: Move Implementation Files 🔴 (NEW)
- **File:** `task-wasm-reset-003-move-implementation-files.md`
- **Purpose:** Move implementation files out of core/ to appropriate modules
- **Scope:**
  - Move `rate_limiter.rs` from core/ to security/ (Violation #3)
  - Move `permission_checker.rs` from core/ to security/ (Violation #4)
  - Move `permission_wit.rs` from core/ to runtime/ or wit/ (Violation #5)
- **Depends On:** TASK-WASM-RESET-002

---

## Violation Summary (From Complete Audit)

### Total: 5 Violations Found

| # | File | Line | Issue | Severity |
|---|------|------|-------|----------|
| 1 | `core/config.rs` | 82 | `use crate::runtime::limits::*` | 🔴 CRITICAL |
| 2 | `core/runtime.rs` | 71 | `use wasmtime::component::Component` | 🔴 CRITICAL |
| 3 | `core/rate_limiter.rs` | - | Implementation logic (334 lines) | 🟡 MEDIUM |
| 4 | `core/permission_checker.rs` | - | Implementation logic (840 lines) | 🟡 MEDIUM |
| 5 | `core/permission_wit.rs` | - | WIT integration impl (724 lines) | 🟡 MEDIUM |

### Rules Violated
- **ADR-WASM-011**: Module Structure Organization
- **ADR-WASM-012**: Comprehensive Core Abstractions Strategy
  - Line 157-161: What does NOT go in core/
  - Line 220-223: Allowed dependencies

---

## File Classification Summary

| Category | Count | Files |
|----------|-------|-------|
| ✅ Correctly in core/ | 18 | mod.rs, error.rs, capability.rs, component.rs, messaging.rs, ... |
| ⚠️ Need fixes | 2 | config.rs, runtime.rs |
| ❌ Should be moved | 3 | rate_limiter.rs, permission_checker.rs, permission_wit.rs |

---

## Primary References

### Foundational Knowledges (Must Read)
1. **KNOWLEDGE-WASM-001**: Component Framework Architecture
2. **KNOWLEDGE-WASM-002**: High-Level Overview
3. **KNOWLEDGE-WASM-003**: Core Architecture Design

### Foundational ADRs (Must Read)
1. **ADR-WASM-011**: Module Structure Organization
2. **ADR-WASM-012**: Comprehensive Core Abstractions Strategy

### Lessons Learned (Must Read)
1. **KNOWLEDGE-WASM-033**: AI Fatal Mistakes - Lessons Learned

---

## Pending Tasks

**Waiting for TASK-WASM-RESET-002 to complete.**

---

## Future Tasks (After Remediation)

After TASK-WASM-RESET-002 and TASK-WASM-RESET-003 complete:

1. **TASK-WASM-RESET-004**: Fix Clippy Warnings in Test Files
   - Address 201 clippy errors in test files
   - Ensure `cargo clippy -- -D warnings` passes

2. **Re-evaluate Previous Work**
   - Review archived tasks
   - Determine what can be salvaged
   - Create new tasks as needed

---

## Archived Tasks

All previous tasks archived in `tasks/archived/`:

| Task ID | Name | Status Before Archive |
|---------|------|----------------------|
| TASK-005 | Block 4 - Security and Isolation Layer | Partially Complete |
| TASK-006 | Block 5 - Inter-Component Communication | 🔴 BROKEN |
| TASK-007 | Block 6 - Persistent Storage System | Not Started |
| TASK-008 | Block 7 - Component Lifecycle System | Not Started |
| TASK-009 | Block 8 - airssys-osl Bridge | Not Started |
| TASK-010 | Block 9 - Monitoring & Observability | Not Started |
| TASK-011 | Block 10 - Component Development SDK | Not Started |
| TASK-012 | Block 11 - CLI Tool | Not Started |

---

## Mandatory Verification (Before Any Work)

```bash
# ALL MUST RETURN NOTHING for architecture to be valid
grep -rn "use crate::" airssys-wasm/src/core/ | grep -v "use crate::core"
grep -rn "wasmtime" airssys-wasm/src/core/
```

**Current Status:** ❌ VIOLATIONS EXIST (5 violations found)

---

## Remember

From KNOWLEDGE-WASM-033 (AI Fatal Mistakes):

1. ✅ **RUN** verification commands and show output
2. ✅ **READ** ADRs before any code changes
3. ✅ **ASK** if uncertain about architecture
4. ❌ **NEVER** claim "verified" without evidence
5. ❌ **NEVER** proceed with assumptions

---

**Updated:** 2025-12-22  
**By:** Memory Bank Manager  
**Audit Verified By:** @memorybank-verifier
