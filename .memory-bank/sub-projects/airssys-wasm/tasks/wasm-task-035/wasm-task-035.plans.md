# WASM-TASK-035: Implementation Plans

## Plan References
- **ADR-WASM-030:** Runtime Module Design (primary specification)
- **ADR-WASM-026:** Implementation Roadmap (Phase 5)

## Target Structure Reference

```
runtime/
├── mod.rs
├── engine.rs        # (WASM-TASK-031) ✅
├── loader.rs        # (WASM-TASK-032)
├── store.rs         # (WASM-TASK-033)
├── host_fn.rs       # (WASM-TASK-034)
└── limiter.rs       # ← THIS TASK
```

---

## Implementation Actions

### Action 1: Create `runtime/limiter.rs`

**Objective:** Implement resource limiting for WASM execution

**File:** `airssys-wasm/src/runtime/limiter.rs`

**Specification (ADR-WASM-030 lines 380-437):**

```rust
//! Resource limiting for WASM execution.

use wasmtime::{StoreLimits, StoreLimitsBuilder};

use crate::core::runtime::limits::ResourceLimits;

use super::engine::RuntimeError;

/// Resource limiter for WASM execution
pub struct WasmResourceLimiter {
    limits: ResourceLimits,
    store_limits: StoreLimits,
}

impl WasmResourceLimiter {
    /// Create a new resource limiter from configuration
    pub fn new(limits: ResourceLimits) -> Self {
        let store_limits = StoreLimitsBuilder::new()
            .memory_size(limits.max_memory_bytes() as usize)
            .build();

        Self {
            limits,
            store_limits,
        }
    }

    /// Get the store limits for wasmtime
    pub fn store_limits(&self) -> &StoreLimits {
        &self.store_limits
    }

    /// Get fuel limit if configured
    pub fn fuel_limit(&self) -> Option<u64> {
        self.limits.max_fuel()
    }

    /// Get execution timeout in milliseconds
    pub fn timeout_ms(&self) -> u64 {
        self.limits.max_execution_time_ms()
    }

    /// Get the underlying resource limits configuration
    pub fn config(&self) -> &ResourceLimits {
        &self.limits
    }
}

/// Apply resource limits to a store
pub fn apply_limits<T>(
    store: &mut wasmtime::Store<T>,
    limiter: &WasmResourceLimiter,
) -> Result<(), RuntimeError> {
    // Set fuel if configured
    if let Some(fuel) = limiter.fuel_limit() {
        store.set_fuel(fuel).map_err(|e| {
            RuntimeError::ExecutionError(format!("Failed to set fuel: {}", e))
        })?;
    }

    Ok(())
}

/// Create default resource limits for testing
#[cfg(test)]
pub fn default_test_limits() -> ResourceLimits {
    ResourceLimits::new(
        16 * 1024 * 1024,  // 16 MB memory
        Some(1_000_000),    // 1M fuel
        5000,               // 5 second timeout
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_create_limiter() {
        let limits = ResourceLimits::new(
            1024 * 1024,     // 1 MB
            Some(100_000),   // 100K fuel
            1000,            // 1 second
        );
        
        let limiter = WasmResourceLimiter::new(limits);
        
        assert_eq!(limiter.fuel_limit(), Some(100_000));
        assert_eq!(limiter.timeout_ms(), 1000);
    }

    #[test]
    fn test_limiter_no_fuel() {
        let limits = ResourceLimits::new(
            1024 * 1024,
            None,           // No fuel limit
            1000,
        );
        
        let limiter = WasmResourceLimiter::new(limits);
        
        assert_eq!(limiter.fuel_limit(), None);
    }

    #[test]
    fn test_store_limits_accessible() {
        let limits = ResourceLimits::new(1024 * 1024, None, 1000);
        let limiter = WasmResourceLimiter::new(limits);
        
        let _ = limiter.store_limits();
    }

    #[test]
    fn test_config_accessible() {
        let limits = ResourceLimits::new(2 * 1024 * 1024, Some(50_000), 2000);
        let limiter = WasmResourceLimiter::new(limits.clone());
        
        let config = limiter.config();
        assert_eq!(config.max_memory_bytes(), 2 * 1024 * 1024);
        assert_eq!(config.max_fuel(), Some(50_000));
        assert_eq!(config.max_execution_time_ms(), 2000);
    }

    #[test]
    fn test_default_test_limits() {
        let limits = default_test_limits();
        
        assert_eq!(limits.max_memory_bytes(), 16 * 1024 * 1024);
        assert!(limits.max_fuel().is_some());
    }
}
```

### Action 2: Update `runtime/mod.rs`

Add `pub mod limiter;` to module declarations.

---

## Verification Commands

```bash
cargo build -p airssys-wasm
cargo clippy -p airssys-wasm --all-targets -- -D warnings
cargo test -p airssys-wasm --lib runtime::limiter
```

---

## Success Criteria

- [ ] WasmResourceLimiter struct implemented
- [ ] StoreLimits integration
- [ ] apply_limits() function implemented
- [ ] default_test_limits() helper (cfg(test))
- [ ] Build/Clippy pass with zero warnings
- [ ] All unit tests pass (5+ tests)

## Notes

> **ResourceLimits Integration**: This task bridges `core/runtime/limits.rs::ResourceLimits` with wasmtime's native StoreLimits.
> Ensure `core/runtime/limits.rs` has the required accessor methods (max_memory_bytes, max_fuel, max_execution_time_ms).
