# TASK-WASM-RESET-003: Move Implementation Files Out of Core

**Task ID:** TASK-WASM-RESET-003  
**Created:** 2025-12-22  
**Status:** 🔴 NOT STARTED  
**Priority:** HIGH  
**Type:** Architecture Remediation  
**Depends On:** TASK-WASM-RESET-002 (must complete first)

---

## Purpose

Move 3 implementation files out of `core/` module to appropriate modules per ADR-WASM-012.

Per ADR-WASM-012 lines 157-161:
> What DOES NOT go in core/:
> - ❌ Concrete implementations (e.g., WasmEngine, SledBackend)
> - ❌ External crate integrations (e.g., wasmtime-specific code)
> - ❌ Business logic (algorithm implementations)
> - ❌ Helper utilities (those go in util/)

These 3 files contain **implementation logic** and must be moved.

---

## Files to Move

### File 1: rate_limiter.rs (Violation #3 from Audit)

| Field | Value |
|-------|-------|
| **Current Location** | `src/core/rate_limiter.rs` |
| **Lines** | 334 |
| **Target Location** | `src/security/rate_limiter.rs` |
| **Problem** | Contains sliding window algorithm implementation |
| **Rule Violated** | ADR-WASM-012 line 160: "❌ Business logic (algorithm implementations)" |

**Content Summary:**
- `MessageRateLimiter` struct with sliding window implementation
- `RateLimiterConfig` struct
- `check_rate_limit()` method with algorithm logic
- `cleanup_stale_senders()` method

**Action Required:**
1. Move entire file to `src/security/rate_limiter.rs`
2. Update `src/security/mod.rs` to export it
3. Update all imports across codebase
4. Optionally: Keep only `RateLimiter` trait in `core/` if needed

---

### File 2: permission_checker.rs (Violation #4 from Audit)

| Field | Value |
|-------|-------|
| **Current Location** | `src/core/permission_checker.rs` |
| **Lines** | 840 |
| **Target Location** | `src/security/permission_checker.rs` |
| **Problem** | Contains full permission checking implementation with glob/lru |
| **Rule Violated** | ADR-WASM-012 line 160: "❌ Business logic (algorithm implementations)" |

**Content Summary:**
- `PermissionChecker` struct with LRU cache
- Glob pattern matching for filesystem paths
- Network endpoint matching
- `can_read_file()`, `can_write_file()`, `can_connect()` implementations

**External Crates Used:**
```rust
use glob::Pattern as GlobPattern;
use lru::LruCache;
```

**Action Required:**
1. Move entire file to `src/security/permission_checker.rs`
2. Update `src/security/mod.rs` to export it
3. Update all imports across codebase
4. Optionally: Keep only `PermissionChecker` trait in `core/` if needed

---

### File 3: permission_wit.rs (Violation #5 from Audit)

| Field | Value |
|-------|-------|
| **Current Location** | `src/core/permission_wit.rs` |
| **Lines** | 724 |
| **Target Location** | `src/runtime/permission_wit.rs` OR `src/wit/permission.rs` |
| **Problem** | Contains WIT integration implementation (type conversions, wrappers) |
| **Rule Violated** | ADR-WASM-012 lines 159-160: Integration code, not abstraction |

**Content Summary:**
- `WitPermissionManifest` struct (WIT type mirror)
- `WitFilesystemPermissions`, `WitNetworkPermissions`, etc.
- `From` trait implementations for WIT ↔ Rust conversions
- Host function wrapper implementations

**Action Required:**
1. Move entire file to `src/runtime/permission_wit.rs` (or create `src/wit/` module)
2. Update module exports
3. Update all imports across codebase
4. Nothing stays in `core/` - this is pure integration code

---

## Implementation Steps

### Step 1: Preparation
- [ ] Verify TASK-WASM-RESET-002 is complete
- [ ] Run `cargo build -p airssys-wasm` to confirm baseline
- [ ] Document all current importers of these 3 files

### Step 2: Move rate_limiter.rs
- [ ] Create `src/security/rate_limiter.rs` (copy content)
- [ ] Update `src/security/mod.rs` to include and export
- [ ] Update `src/core/mod.rs` to remove export
- [ ] Delete `src/core/rate_limiter.rs`
- [ ] Fix all import errors across codebase
- [ ] Run `cargo build -p airssys-wasm`
- [ ] Run `cargo test -p airssys-wasm`

### Step 3: Move permission_checker.rs
- [ ] Create `src/security/permission_checker.rs` (copy content)
- [ ] Update `src/security/mod.rs` to include and export
- [ ] Update `src/core/mod.rs` to remove export
- [ ] Delete `src/core/permission_checker.rs`
- [ ] Fix all import errors across codebase
- [ ] Run `cargo build -p airssys-wasm`
- [ ] Run `cargo test -p airssys-wasm`

### Step 4: Move permission_wit.rs
- [ ] Decide target: `runtime/` or new `wit/` module
- [ ] Create target file (copy content)
- [ ] Update target module's mod.rs
- [ ] Update `src/core/mod.rs` to remove export
- [ ] Delete `src/core/permission_wit.rs`
- [ ] Fix all import errors across codebase
- [ ] Run `cargo build -p airssys-wasm`
- [ ] Run `cargo test -p airssys-wasm`

### Step 5: Verification
- [ ] Run architecture verification:
  ```bash
  grep -rn "use crate::" airssys-wasm/src/core/ | grep -v "use crate::core"
  ```
- [ ] Run full test suite: `cargo test -p airssys-wasm`
- [ ] Run clippy: `cargo clippy -p airssys-wasm --lib -- -D warnings`
- [ ] Verify no files with implementation logic remain in `core/`

---

## Success Criteria

| Criteria | Required |
|----------|----------|
| `rate_limiter.rs` moved to `security/` | ✅ |
| `permission_checker.rs` moved to `security/` | ✅ |
| `permission_wit.rs` moved to `runtime/` or `wit/` | ✅ |
| All imports updated | ✅ |
| `cargo build` passes | ✅ |
| `cargo test` passes | ✅ |
| `cargo clippy --lib -- -D warnings` passes | ✅ |
| Architecture verification returns NOTHING | ✅ |

---

## Effort Estimate

| File | Estimated Effort | Risk |
|------|------------------|------|
| rate_limiter.rs | 2-3 hours | Low (isolated) |
| permission_checker.rs | 3-4 hours | Medium (widely used) |
| permission_wit.rs | 2-3 hours | Medium (WIT integration) |
| **Total** | **7-10 hours** | Medium |

---

## References

- **TASK-WASM-RESET-001**: Core Module Full Audit (source of violations)
- **ADR-WASM-012**: Comprehensive Core Abstractions Strategy
- **KNOWLEDGE-WASM-033**: AI Fatal Mistakes - Lessons Learned

---

**Created By:** Memory Bank Manager  
**Date:** 2025-12-22  
**Reason:** Address Violations #3, #4, #5 from Core Module Audit
