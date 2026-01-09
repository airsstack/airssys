# WASM-TASK-018: Implementation Plans

## Plan References

### Architecture Documents
- **ADR-WASM-028:** Core Module Structure (updated 2026-01-09 for co-located errors)
- **ADR-WASM-025:** Clean-Slate Rebuild Architecture (dependency inversion principles)
- **KNOWLEDGE-WASM-037:** Rebuild Architecture Clean-Slate (technical reference)
- **KNOWLEDGE-WASM-039:** Runtime Module Responsibility

### Key Architecture Decision
> **Co-located Errors Pattern (2026-01-09)**
> 
> Each module contains its own errors.rs file. WasmError is co-located in 
> `core/runtime/errors.rs`, NOT in a centralized `core/errors/` module.

---

## Target Structure Reference

Per ADR-WASM-028 (updated 2026-01-09):
```
core/runtime/
├── mod.rs           # Module declarations
├── errors.rs        # WasmError (co-located)  ← NEW!
├── traits.rs        # RuntimeEngine, ComponentLoader traits
└── limits.rs        # ResourceLimits
```

---

## Implementation Actions

### Action 1: Create core/runtime/errors.rs (CO-LOCATED)

**Objective:** Create WasmError enum using thiserror (co-located pattern)

**File:** `src/core/runtime/errors.rs`

```rust
//! WASM runtime error types.
//!
//! This module contains error types for WASM runtime operations.
//! Errors are co-located with the runtime module per ADR-WASM-028.

// Layer 1: Standard library imports
// (none)

// Layer 2: Third-party crate imports
use thiserror::Error;

// Layer 3: Internal module imports
// (none - errors have no internal dependencies)

/// WASM runtime errors for component loading and execution.
///
/// This error type is used by the `RuntimeEngine` and `ComponentLoader` traits.
#[derive(Debug, Clone, Error)]
pub enum WasmError {
    /// Component not found.
    #[error("Component not found: {0}")]
    ComponentNotFound(String),

    /// Component instantiation failed.
    #[error("Component instantiation failed: {0}")]
    InstantiationFailed(String),

    /// Export not found.
    #[error("Export not found: {0}")]
    ExportNotFound(String),

    /// Execution timeout.
    #[error("Execution timeout")]
    Timeout,

    /// Resource limit exceeded.
    #[error("Resource limit exceeded: {0}")]
    ResourceLimitExceeded(String),

    /// Invalid component.
    #[error("Invalid component: {0}")]
    InvalidComponent(String),

    /// Runtime error.
    #[error("Runtime error: {0}")]
    RuntimeError(String),
}
```

**Unit Tests:**
```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_component_not_found_display() {
        let err = WasmError::ComponentNotFound("test-comp".to_string());
        assert!(format!("{}", err).contains("Component not found"));
    }

    #[test]
    fn test_timeout_display() {
        let err = WasmError::Timeout;
        assert_eq!(format!("{}", err), "Execution timeout");
    }

    #[test]
    fn test_error_is_clone() {
        let err = WasmError::Timeout;
        let cloned = err.clone();
        assert!(matches!(cloned, WasmError::Timeout));
    }
}
```

---

### Action 2: Update core/runtime/traits.rs

**Objective:** Replace MockWasmError with WasmError from errors.rs

**Changes:**
1. Remove `MockWasmError` enum and Display impl (lines 12-65)
2. Add import: `use super::errors::WasmError;`
3. Replace all `MockWasmError` → `WasmError` in trait definitions
4. Update documentation examples

**Before:**
```rust
pub enum MockWasmError { ... }

pub trait RuntimeEngine: Send + Sync {
    fn load_component(...) -> Result<ComponentHandle, MockWasmError>;
}
```

**After:**
```rust
use super::errors::WasmError;

pub trait RuntimeEngine: Send + Sync {
    fn load_component(...) -> Result<ComponentHandle, WasmError>;
}
```

---

### Action 3: Update core/runtime/mod.rs

**Objective:** Export errors module

**Add:**
```rust
pub mod errors;
```

> **Note:** Per PROJECTS_STANDARD.md §4.3, we do NOT re-export `WasmError` type.
> Callers use: `use crate::core::runtime::errors::WasmError;`

---

### Action 4: Update core/runtime/limits.rs

**No changes needed** - ResourceLimits has no error dependencies.

---

## Verification Commands

```bash
# Build check
cargo build -p airssys-wasm

# Lint check
cargo clippy -p airssys-wasm --all-targets -- -D warnings

# Test check
cargo test -p airssys-wasm core::runtime

# Verify WasmError is co-located (not in core/errors/)
ls src/core/runtime/errors.rs  # Should exist
ls src/core/errors/ 2>/dev/null || echo "GOOD: No core/errors/ directory"
```

---

## Refactoring Summary

| File | Current State | Target State |
|:---|:---|:---|
| `errors.rs` | Does not exist | ✅ Create with `WasmError` |
| `traits.rs` | Has `MockWasmError` inline | ✅ Use `WasmError` from errors.rs |
| `mod.rs` | No errors export | ✅ Add `pub mod errors` |
| `limits.rs` | No changes needed | - |

---

## Success Criteria

- [ ] `core/runtime/errors.rs` exists with `WasmError` enum
- [ ] `WasmError` uses `thiserror` derive macro
- [ ] `MockWasmError` removed from `traits.rs`
- [ ] `traits.rs` imports `WasmError` from `super::errors`
- [ ] `mod.rs` exports errors module
- [ ] All tests pass
- [ ] Zero warnings

---

## Standards Compliance

- **§2.1** 3-Layer Import Organization ✅
- **§4.3** Module Architecture Patterns ✅
- **ADR-WASM-028** Co-located errors ✅
- **KNOWLEDGE-WASM-040** Messaging patterns reference ✅
