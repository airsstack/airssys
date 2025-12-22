# TASK-WASM-RESET-002: Fix Core Architecture Violations

**Task ID:** TASK-WASM-RESET-002  
**Created:** 2025-12-22  
**Status:** 🔴 NOT STARTED  
**Priority:** CRITICAL - BLOCKING  
**Type:** Architecture Remediation  
**Depends On:** TASK-WASM-RESET-001 (Core Module Full Audit) - ✅ COMPLETE

---

## ⚠️ CONTEXT: ARCHITECTURE VIOLATION REMEDIATION

This task was created after TASK-WASM-RESET-001 (Core Module Full Audit) identified **1 CRITICAL** architecture violation that must be fixed before any further development.

**The Violation:**
```
airssys-wasm/src/core/config.rs:82:
use crate::runtime::limits::{CpuConfig, MemoryConfig, ResourceConfig, ResourceLimits};
```

**Rule Violated:** ADR-WASM-011 & ADR-WASM-012 - `core/` MUST have ZERO internal dependencies

---

## PRIMARY REFERENCES (MANDATORY READING)

1. **ADR-WASM-011:** Module Structure Organization
   - `docs/adr/adr-wasm-011-module-structure-organization.md`
   - Key Rule: "core/ has zero internal dependencies (foundation)"
   
2. **ADR-WASM-012:** Comprehensive Core Abstractions Strategy
   - `docs/adr/adr-wasm-012-comprehensive-core-abstractions-strategy.md`
   - Key Rule: "The core/ module MUST have zero internal dependencies within airssys-wasm"
   - Key Rule: "Only external crates allowed (serde, thiserror, chrono, etc.)"

3. **KNOWLEDGE-WASM-033:** AI Fatal Mistakes - Lessons Learned
   - `docs/knowledges/knowledge-wasm-033-ai-fatal-mistakes-lessons-learned.md`
   - Key: Show actual command output as PROOF

---

## VIOLATION ANALYSIS

### What Happened

The file `core/config.rs` imports types from `runtime/limits.rs`:

```rust
// core/config.rs - Line 82 (FORBIDDEN)
use crate::runtime::limits::{CpuConfig, MemoryConfig, ResourceConfig, ResourceLimits};
```

### Why This Is Wrong

Per the module dependency hierarchy:

```
CORRECT HIERARCHY:
  actor/ → runtime/ → security/ → core/
  
WHAT IS HAPPENING:
  core/ → runtime/  ❌ BACKWARDS DEPENDENCY
```

- `core/` is the FOUNDATION layer
- All other modules depend ON `core/`
- `core/` depends on NOTHING (only external crates)

### Types That Need to Move

These types need to move FROM `runtime/limits.rs` TO `core/config.rs`:

| Type | Lines in runtime/limits.rs | Purpose |
|------|---------------------------|---------|
| `MemoryConfig` | 679-682 | Memory configuration struct |
| `CpuConfig` | 715-719 | CPU configuration struct |
| `ResourceConfig` | 830-834 | Combined memory + CPU config |
| `ResourceLimits` | 362-367 | Validated resource limits |
| `ResourceLimitsBuilder` | 482-487 | Builder for ResourceLimits |

### Dependencies to Check

The types in `runtime/limits.rs` have these dependencies:
- `std::sync::atomic::{AtomicU64, Ordering}` - Standard library ✅
- `std::sync::Arc` - Standard library ✅
- `serde::{Deserialize, Serialize}` - External crate ✅
- `wasmtime::ResourceLimiter` - External crate (Wasmtime-specific)
- `crate::core::error::{WasmError, WasmResult}` - Core types ✅

**Issue:** `ResourceLimits` validation uses `WasmError` which is in core. This is fine.
**Issue:** `ComponentResourceLimiter` implements `wasmtime::ResourceLimiter` trait. This is Wasmtime-specific and should STAY in `runtime/`.

---

## REMEDIATION PLAN

### Phase 1: Move Core Types to `core/config.rs`

**Types to Move:**
1. `MemoryConfig` struct
2. `MemoryConfig::validate()` method
3. `CpuConfig` struct  
4. `CpuConfig::validate()` method
5. `ResourceConfig` struct
6. `ResourceLimits` struct
7. `ResourceLimits` constants (MIN_MEMORY_BYTES, MAX_MEMORY_BYTES, etc.)
8. `ResourceLimits` methods (builder, getters)
9. `ResourceLimitsBuilder` struct and methods
10. `TryFrom<ResourceConfig> for ResourceLimits` impl

**Types to KEEP in runtime/limits.rs:**
1. `ComponentResourceLimiter` - Wasmtime-specific implementation
2. `MemoryMetrics` - Runtime metrics tracking
3. All `impl ResourceLimiter for ComponentResourceLimiter` - Wasmtime trait impl

### Phase 2: Update Imports

**In `runtime/limits.rs`:**
```rust
// REMOVE:
// (no internal imports for types being moved)

// ADD:
use crate::core::config::{CpuConfig, MemoryConfig, ResourceConfig, ResourceLimits};
```

**In `core/config.rs`:**
```rust
// REMOVE:
use crate::runtime::limits::{CpuConfig, MemoryConfig, ResourceConfig, ResourceLimits};

// ADD:
// (the types are now defined here, no import needed)
```

### Phase 3: Update `core/mod.rs`

Add re-exports for the moved types:
```rust
pub use config::{CpuConfig, MemoryConfig, ResourceConfig, ResourceLimits, ResourceLimitsBuilder};
```

### Phase 4: Update All Dependent Files

Files that import from `runtime/limits`:
```bash
grep -rn "use crate::runtime::limits" airssys-wasm/src/
```

Update each file to import from `core::config` instead (for the moved types).

### Phase 5: Verification

**MANDATORY: Run ALL verification commands and show output**

```bash
# Check 1: Verify core/ has NO forbidden imports
grep -rn "use crate::" airssys-wasm/src/core/ | grep -v "use crate::core"
# MUST return NOTHING

# Check 2: Verify build passes
cargo build -p airssys-wasm

# Check 3: Verify tests pass
cargo test -p airssys-wasm --lib

# Check 4: Verify clippy passes
cargo clippy -p airssys-wasm --all-targets --all-features -- -D warnings
```

---

## DETAILED IMPLEMENTATION STEPS

### Step 1: Prepare `core/config.rs`

1. Open `core/config.rs`
2. Remove line 82: `use crate::runtime::limits::{...}`
3. Add necessary imports from std (if not already present):
   ```rust
   use std::sync::Arc;
   use std::sync::atomic::{AtomicU64, Ordering};
   ```
4. Add serde import (if not already present):
   ```rust
   use serde::{Deserialize, Serialize};
   ```

### Step 2: Move Type Definitions

Copy the following from `runtime/limits.rs` to `core/config.rs`:

1. **ResourceLimits** (lines 362-458)
   - Struct definition
   - Associated constants (MIN_MEMORY_BYTES, etc.)
   - All methods (builder, getters)

2. **ResourceLimitsBuilder** (lines 482-641)
   - Struct definition
   - All methods (new, setters, build)

3. **MemoryConfig** (lines 679-762)
   - Struct definition with Serialize, Deserialize derives
   - validate() method

4. **CpuConfig** (lines 715-823)
   - Struct definition with Serialize, Deserialize derives
   - validate() method

5. **ResourceConfig** (lines 830-851)
   - Struct definition
   - TryFrom implementation

### Step 3: Update `runtime/limits.rs`

1. Remove moved type definitions
2. Add import from core:
   ```rust
   use crate::core::config::{CpuConfig, MemoryConfig, ResourceConfig, ResourceLimits};
   ```
3. Keep `ComponentResourceLimiter`, `MemoryMetrics`, and Wasmtime integration code

### Step 4: Update Unit Tests

Move relevant unit tests for the moved types:
- Tests in `runtime/limits.rs` for ResourceLimits, MemoryConfig, CpuConfig, ResourceConfig
- Should move to `core/config.rs` (or stay as integration tests in runtime/)

### Step 5: Find and Update All Consumers

```bash
# Find all files importing from runtime/limits
grep -rln "use crate::runtime::limits" airssys-wasm/src/
```

For each file, update imports to use `core::config` for the moved types.

---

## SUCCESS CRITERIA

This task is **COMPLETE** when:

1. ✅ `grep -rn "use crate::" airssys-wasm/src/core/ | grep -v "use crate::core"` returns **NOTHING**
2. ✅ `cargo build -p airssys-wasm` passes with no errors
3. ✅ `cargo test -p airssys-wasm --lib` - all tests pass
4. ✅ `cargo clippy -p airssys-wasm --all-targets --all-features -- -D warnings` - no warnings
5. ✅ All verification command outputs shown as evidence

---

## EFFORT ESTIMATE

| Phase | Estimated Time |
|-------|----------------|
| Phase 1: Move types | 2-3 hours |
| Phase 2: Update imports | 30 min |
| Phase 3: Update mod.rs | 15 min |
| Phase 4: Update consumers | 1-2 hours |
| Phase 5: Verification | 30 min |
| **Total** | **4-6 hours** |

---

## RISK ASSESSMENT

| Risk | Level | Mitigation |
|------|-------|------------|
| Breaking dependent code | Medium | Run full test suite after each phase |
| Missing import updates | Low | Use grep to find all consumers |
| Test failures | Medium | Move tests along with types |
| Circular dependency introduced | Low | Verify with grep after changes |

---

## ROLLBACK PLAN

If issues arise:
1. `git checkout -- airssys-wasm/src/` to restore original files
2. Investigate the issue
3. Try again with fixes

---

## CHECKLIST

### Pre-Implementation
- [ ] Read ADR-WASM-011 completely
- [ ] Read ADR-WASM-012 completely
- [ ] Read KNOWLEDGE-WASM-033 completely
- [ ] Understand which types to move and which to keep

### Implementation
- [ ] Remove forbidden import from `core/config.rs`
- [ ] Add necessary std imports to `core/config.rs`
- [ ] Move `ResourceLimits` struct and methods
- [ ] Move `ResourceLimitsBuilder` struct and methods
- [ ] Move `MemoryConfig` struct and methods
- [ ] Move `CpuConfig` struct and methods
- [ ] Move `ResourceConfig` struct and impl
- [ ] Update `runtime/limits.rs` to import from core
- [ ] Update `core/mod.rs` re-exports
- [ ] Find and update all consumer files
- [ ] Move relevant unit tests

### Verification
- [ ] Run architecture check: `grep -rn "use crate::" airssys-wasm/src/core/ | grep -v "use crate::core"`
- [ ] Run build: `cargo build -p airssys-wasm`
- [ ] Run tests: `cargo test -p airssys-wasm --lib`
- [ ] Run clippy: `cargo clippy -p airssys-wasm --all-targets --all-features -- -D warnings`
- [ ] Show all command outputs as evidence

### Post-Implementation
- [ ] Update TASK-WASM-RESET-001 status to COMPLETE
- [ ] Update this task status to COMPLETE
- [ ] Update `_index.md` with completion status

---

## NOTES

### What This Task Fixes
- The architecture violation at `core/config.rs:82`
- Restores `core/` module to have ZERO internal dependencies

### What This Task Does NOT Include
- Fixing clippy warnings in test files (separate task)
- Reviewing other files for ADR-WASM-012 compliance (separate task)
- Any new feature development

### Remember the Lessons (KNOWLEDGE-WASM-033)
1. ✅ Run verification commands and SHOW OUTPUT
2. ✅ Never claim "verified" without proof
3. ✅ Read ADRs before making changes
4. ❌ Do NOT claim complete without running verifications

---

## REFERENCES

- **TASK-WASM-RESET-001**: Core Module Full Audit (dependency)
- **ADR-WASM-011**: Module Structure Organization
- **ADR-WASM-012**: Comprehensive Core Abstractions Strategy
- **KNOWLEDGE-WASM-033**: AI Fatal Mistakes - Lessons Learned

---

**Created By:** Memory Bank Manager  
**Date:** 2025-12-22  
**Reason:** Remediate architecture violation found in TASK-WASM-RESET-001 audit
