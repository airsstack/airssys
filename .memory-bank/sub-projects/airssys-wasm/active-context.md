# airssys-wasm Active Context

**Last Updated:** 2025-12-22  
**Current Status:** 🔴 **ARCHITECTURE BROKEN - ALL WORK BLOCKED**  
**Blocking Issue:** Module boundary violations per ADR-WASM-023

---

## 🔴 CRITICAL: ARCHITECTURE IS BROKEN

### The Truth (Verified 2025-12-22)

**The airssys-wasm module architecture violates its own rules (ADR-WASM-023).**

Despite multiple previous claims of "fixes" and "verification," the following violations exist:

### Violation #1: `core/` → `runtime/` ❌

**File:** `src/core/config.rs:82`
```rust
use crate::runtime::limits::{CpuConfig, MemoryConfig, ResourceConfig, ResourceLimits};
```

**Why This Is Fatal:**
- `core/` is the foundation - NOTHING should be imported from other modules
- Every module depends on `core/`, so this creates implicit transitive dependencies
- This inverts the entire dependency hierarchy

### Violation #2: `runtime/` → `actor/` ❌

**File:** `src/runtime/messaging.rs:78`
```rust
use crate::actor::message::CorrelationTracker;
```

**Why This Is Fatal:**
- `runtime/` should only import from `core/` and `security/`
- `actor/` is meant to depend on `runtime/`, not the other way around
- This creates potential circular dependency issues

---

## Required Module Hierarchy (ADR-WASM-023)

```
core/     → imports NOTHING from crate
security/ → imports core/ only  
runtime/  → imports core/, security/ only
actor/    → imports core/, security/, runtime/
```

**Current Reality:**
```
core/     → imports runtime/ ❌ VIOLATION
runtime/  → imports actor/ ❌ VIOLATION
```

---

## Verification Commands (Run These Before ANY Work)

```bash
# ALL must return NOTHING for architecture to be valid

# Check 1: core/ should NEVER import from crate modules
grep -rn "use crate::runtime" airssys-wasm/src/core/
grep -rn "use crate::actor" airssys-wasm/src/core/
grep -rn "use crate::security" airssys-wasm/src/core/

# Check 2: runtime/ should NEVER import from actor/
grep -rn "use crate::actor" airssys-wasm/src/runtime/
```

**Current Results (2025-12-22):**
```
$ grep -rn "use crate::runtime" airssys-wasm/src/core/
src/core/config.rs:82:use crate::runtime::limits::{...}  ← VIOLATION

$ grep -rn "use crate::actor" airssys-wasm/src/runtime/
src/runtime/messaging.rs:78:use crate::actor::message::CorrelationTracker;  ← VIOLATION
```

---

## What Must Be Fixed (BEFORE Any Other Work)

### Fix #1: Move Resource Types to `core/`

**Move from `runtime/limits.rs` to `core/config.rs`:**
- `CpuConfig`
- `MemoryConfig`  
- `ResourceConfig`
- `ResourceLimits`

Then update `runtime/` to import these from `core/`.

### Fix #2: Move `CorrelationTracker` to `core/` OR Move `MessagingService` to `actor/`

**Option A:** Move `CorrelationTracker` to `core/` (simpler)
- It's a data structure, shared types belong in `core/`

**Option B:** Move `MessagingService` to `actor/` (per HOTFIX-002 plan)
- MessagingService is messaging orchestration, belongs with actor system

---

## What's NOT True (Despite Previous Claims)

| Previous Claim | Reality |
|----------------|---------|
| "Hotfix Phase 1 COMPLETE" | ❌ `runtime/` still imports from `actor/` |
| "Hotfix Phase 2 COMPLETE" | ❌ `core/` imports from `runtime/` |
| "Zero circular dependencies" | ❌ Violations create circular potential |
| "Architecture verified" | ❌ Never actually verified with grep |

---

## Block Status (CORRECTED)

| Block | Claimed Status | Actual Status |
|-------|---------------|---------------|
| Block 3 | ✅ COMPLETE | ⚠️ Works but on broken foundation |
| Block 4 | ✅ COMPLETE | ⚠️ Works but on broken foundation |
| Block 5 | 🚀 IN PROGRESS | 🔴 BLOCKED by architecture violations |
| **HOTFIX-002** | "COMPLETE" | ❌ **NOT COMPLETE** |

---

## Required Actions (In Order)

1. **STOP** all feature work
2. **FIX** `core/` → `runtime/` violation (highest priority)
3. **FIX** `runtime/` → `actor/` violation
4. **VERIFY** with grep commands (show actual output)
5. **ADD** CI enforcement to prevent regression
6. **THEN** resume Block 5 Phase 3 Task 3.3

---

## Reference Documents

- **[KNOWLEDGE-WASM-032](docs/knowledges/knowledge-wasm-032-module-boundary-violations-audit.md)** - Full audit
- **[ADR-WASM-023](docs/adr/adr-wasm-023-module-boundary-enforcement.md)** - Module rules
- **[WASM-TASK-006-HOTFIX-002](tasks/task-006-hotfix-module-boundary-violations.md)** - Incomplete hotfix

---

## Lessons Learned

1. **Never trust "verified" without seeing grep output**
2. **Automated CI checks are essential**
3. **"Compiles successfully" ≠ "Architecture is correct"**
4. **Trust but verify - always run the checks yourself**

---

**This document reflects the true state as of 2025-12-22.**
