//! Resource limiter for WASM execution.
//!
//! Bridges `core::runtime::limits::ResourceLimits` with Wasmtime's
//! `StoreLimits` (memory/tables) and Fuel (CPU) mechanisms.
//!
//! # Overview
//!
//! The resource limiter converts high-level resource constraints into
//! Wasmtime's concrete limit mechanisms:
//! - **Memory**: Enforced via `StoreLimits` when WASM calls `memory.grow`
//! - **CPU**: Enforced via Fuel when WASM instructions are executed
//!
//! # Examples
//!
//! ```ignore
//! use airssys_wasm::core::runtime::limits::ResourceLimits;
//! use airssys_wasm::runtime::limiter::WasmResourceLimiter;
//!
//! let limits = ResourceLimits::default();
//! let limiter = WasmResourceLimiter::new(&limits);
//! let store_limits = limiter.into_store_limits();
//! ```

// Layer 1: Standard library imports (per PROJECTS_STANDARD.md §2.1)
// (none needed)

// Layer 2: Third-party crate imports (per PROJECTS_STANDARD.md §2.1)
use wasmtime::{Store, StoreLimits, StoreLimitsBuilder};

// Layer 3: Internal module imports (per PROJECTS_STANDARD.md §2.1)
use crate::core::runtime::errors::WasmError;
use crate::core::runtime::limits::ResourceLimits;

use super::engine::HostState;

/// Resource limiter for WASM execution.
///
/// Wraps Wasmtime's `StoreLimits` for memory/table enforcement
/// and provides fuel configuration.
///
/// # Fields
///
/// * `store_limits` - Wasmtime's StoreLimits for memory and table constraints
/// * `fuel_limit` - Optional fuel limit for instruction counting
pub struct WasmResourceLimiter {
    store_limits: StoreLimits,
    fuel_limit: Option<u64>,
}

impl WasmResourceLimiter {
    /// Create a new resource limiter from core ResourceLimits.
    ///
    /// Converts `core::runtime::limits::ResourceLimits` into Wasmtime's
    /// `StoreLimits` and stores fuel configuration.
    ///
    /// # Arguments
    /// * `limits` - Core resource limits configuration
    ///
    /// # Example
    /// ```ignore
    /// use airssys_wasm::core::runtime::limits::ResourceLimits;
    /// use airssys_wasm::runtime::limiter::WasmResourceLimiter;
    ///
    /// let limits = ResourceLimits::default();
    /// let limiter = WasmResourceLimiter::new(&limits);
    /// ```
    pub fn new(limits: &ResourceLimits) -> Self {
        let store_limits = StoreLimitsBuilder::new()
            .memory_size(limits.max_memory_bytes as usize)
            .table_elements(10_000) // Default table limit
            .build();

        Self {
            store_limits,
            fuel_limit: limits.max_fuel,
        }
    }

    /// Consume the limiter and return the inner StoreLimits.
    ///
    /// Transfers ownership of the StoreLimits to the caller.
    /// This is typically used when configuring a Store.
    pub fn into_store_limits(self) -> StoreLimits {
        self.store_limits
    }

    /// Get fuel limit if configured.
    ///
    /// Returns `Some(fuel)` if a fuel limit is configured,
    /// or `None` if fuel limiting is disabled.
    pub fn fuel_limit(&self) -> Option<u64> {
        self.fuel_limit
    }
}

/// Apply resource limits to a Store.
///
/// This function:
/// 1. Sets the StoreLimits in HostState for memory/table enforcement
/// 2. Configures the store's limiter callback to use HostState.store_limits
/// 3. Sets fuel if configured
///
/// # Arguments
/// * `store` - The Wasmtime store to apply limits to
/// * `limits` - Resource limits to apply
///
/// # Errors
/// Returns `WasmError::RuntimeError` if fuel cannot be set.
///
/// # Example
/// ```ignore
/// use wasmtime::Store;
/// use airssys_wasm::core::runtime::limits::ResourceLimits;
/// use airssys_wasm::runtime::limiter::apply_limits_to_store;
/// use airssys_wasm::runtime::engine::HostState;
///
/// let limits = ResourceLimits::default();
/// let mut store = Store::new(&engine, host_state);
/// apply_limits_to_store(&mut store, &limits)?;
/// ```
pub fn apply_limits_to_store(
    store: &mut Store<HostState>,
    limits: &ResourceLimits,
) -> Result<(), WasmError> {
    // Create the limiter
    let limiter = WasmResourceLimiter::new(limits);

    // 1. Set StoreLimits in HostState
    store.data_mut().store_limits = limiter.into_store_limits();

    // 2. Configure the store's limiter callback
    // This tells Wasmtime to use HostState.store_limits for memory/table checks
    store.limiter(|state| &mut state.store_limits);

    // 3. Set fuel if configured
    if let Some(fuel) = limits.max_fuel {
        store
            .set_fuel(fuel)
            .map_err(|e| WasmError::RuntimeError(format!("Failed to set fuel: {}", e)))?;
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_limiter_creation_default() {
        let limits = ResourceLimits::default();
        let limiter = WasmResourceLimiter::new(&limits);

        assert!(limiter.fuel_limit().is_none());
    }

    #[test]
    fn test_limiter_with_fuel() {
        let limits = ResourceLimits {
            max_memory_bytes: 64 * 1024 * 1024,
            max_execution_time_ms: 30_000,
            max_fuel: Some(1_000_000),
        };
        let limiter = WasmResourceLimiter::new(&limits);

        assert_eq!(limiter.fuel_limit(), Some(1_000_000));
    }

    #[test]
    fn test_limiter_into_store_limits() {
        let limits = ResourceLimits::default();
        let limiter = WasmResourceLimiter::new(&limits);
        let _store_limits = limiter.into_store_limits();
        // StoreLimits is opaque, but we verify it doesn't panic
    }

    #[test]
    fn test_limiter_custom_memory() {
        let limits = ResourceLimits {
            max_memory_bytes: 16 * 1024 * 1024, // 16MB
            max_execution_time_ms: 10_000,
            max_fuel: Some(500_000),
        };
        let limiter = WasmResourceLimiter::new(&limits);

        assert_eq!(limiter.fuel_limit(), Some(500_000));
    }
}
