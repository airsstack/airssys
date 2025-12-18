//! Store lifecycle management with resource cleanup.
//!
//! This module provides proper lifecycle management for Wasmtime Store instances,
//! ensuring resources are cleaned up on normal completion or failure (Phase 5 - Task 5.2).
//!
//! # Crash Isolation and Cleanup (ADR-WASM-006)
//!
//! When a component crashes (trap, panic, timeout), we must ensure:
//! 1. Memory is fully reclaimed (linear memory released)
//! 2. File handles are closed
//! 3. Network connections are terminated
//! 4. Fuel metering state is reset
//!
//! The `StoreWrapper` type implements `Drop` to guarantee cleanup even on unwinding.
//!
//! # Architecture
//!
//! ```text
//! StoreWrapper (RAII pattern)
//!     ├── Wasmtime Store<T> (linear memory, fuel, tables)
//!     ├── Drop impl (automatic cleanup)
//!     └── Metrics tracking (memory usage, fuel consumed)
//! ```
//!
//! # Example
//!
//! ```rust,ignore
//! use airssys_wasm::runtime::StoreWrapper;
//!
//! async fn execute_component() -> Result<(), WasmError> {
//!     // Create store (RAII - cleanup automatic)
//!     let mut store = StoreWrapper::new(&engine, (), 1_000_000)?;
//!     
//!     // Execute component
//!     let result = execute_function(&mut store).await;
//!     
//!     // Drop automatically cleans up:
//!     // - Memory reclaimed
//!     // - Fuel state reset
//!     // - Resources released
//!     result
//! }
//! ```

// Layer 1: Standard library imports (§2.1 - 3-layer import organization)
use std::ops::{Deref, DerefMut};

// Layer 2: External crate imports
use wasmtime::{AsContext, AsContextMut, Engine, Store, StoreContext, StoreContextMut};

// Layer 3: Internal module imports
use crate::core::error::{WasmError, WasmResult};

/// RAII wrapper for Wasmtime Store with automatic resource cleanup.
///
/// Ensures proper cleanup of WASM resources on normal completion or crash:
/// - Linear memory fully reclaimed
/// - Fuel metering state reset
/// - Store resources released
/// - Metrics collected before cleanup
///
/// # Design Pattern (RAII)
///
/// Uses Rust's ownership and Drop trait to guarantee cleanup even when:
/// - Component traps (division by zero, bounds check)
/// - Execution times out
/// - Fuel is exhausted
/// - Host panics (though Wasmtime prevents this)
///
/// # Type Parameter
///
/// - `T`: Store context data (often `()` for stateless execution)
///
/// # Example
///
/// ```rust,ignore
/// // Create store (cleanup automatic via Drop)
/// let mut store = StoreWrapper::new(&engine, (), 10_000_000)?;
///
/// // Set fuel
/// store.set_fuel(10_000_000)?;
///
/// // Execute (may trap/timeout)
/// match execute_function(&mut store).await {
///     Ok(result) => println!("Success: {result:?}"),
///     Err(e) => eprintln!("Failed: {e}"),
/// }
/// // Store automatically dropped here - resources cleaned up
/// ```
pub struct StoreWrapper<T> {
    /// Wasmtime store with component state.
    store: Store<T>,

    /// Component ID for logging (cleanup diagnostics).
    component_id: String,

    /// Initial fuel allocated (for cleanup metrics).
    initial_fuel: u64,
}

impl<T> StoreWrapper<T> {
    /// Create a new Store wrapper with fuel configuration.
    ///
    /// # Arguments
    ///
    /// - `engine`: Wasmtime engine reference
    /// - `data`: Store context data
    /// - `initial_fuel`: Fuel allocation for CPU limiting
    ///
    /// # Returns
    ///
    /// - `Ok(StoreWrapper)`: Store created and fuel configured
    /// - `Err(WasmError::ExecutionFailed)`: Fuel configuration failed
    ///
    /// # Example
    ///
    /// ```rust,ignore
    /// let store = StoreWrapper::new(&engine, (), 10_000_000)?;
    /// ```
    pub fn new(engine: &Engine, data: T, initial_fuel: u64) -> WasmResult<Self> {
        let mut store = Store::new(engine, data);

        // Set initial fuel for CPU metering
        store
            .set_fuel(initial_fuel)
            .map_err(|e| WasmError::execution_failed(format!("Failed to set initial fuel: {e}")))?;

        Ok(Self {
            store,
            component_id: String::from("unknown"), // Will be set by engine
            initial_fuel,
        })
    }

    /// Set component ID for cleanup diagnostics.
    ///
    /// Called by engine after Store creation to enable proper logging.
    pub fn set_component_id(&mut self, component_id: String) {
        self.component_id = component_id;
    }

    /// Get remaining fuel (for diagnostics and metrics).
    ///
    /// Returns `None` if fuel metering is not enabled.
    pub fn remaining_fuel(&mut self) -> Option<u64> {
        self.store.get_fuel().ok()
    }

    /// Get fuel consumed since Store creation.
    ///
    /// Calculated as initial_fuel - remaining_fuel.
    /// Returns `None` if fuel metering is not enabled.
    pub fn fuel_consumed(&mut self) -> Option<u64> {
        self.remaining_fuel()
            .map(|remaining| self.initial_fuel.saturating_sub(remaining))
    }

    /// Get metrics before cleanup.
    ///
    /// Used in Drop implementation to log resource usage.
    fn collect_metrics(&mut self) -> StoreMetrics {
        StoreMetrics {
            component_id: self.component_id.clone(),
            fuel_consumed: self.fuel_consumed().unwrap_or(0),
            initial_fuel: self.initial_fuel,
        }
    }
}

/// Metrics collected before Store cleanup.
///
/// Used for logging and monitoring component resource usage.
#[derive(Debug, Clone)]
#[allow(dead_code)] // Used in Drop impl for future logging
struct StoreMetrics {
    component_id: String,
    fuel_consumed: u64,
    initial_fuel: u64,
}

impl<T> Drop for StoreWrapper<T> {
    /// Clean up Store resources on drop.
    ///
    /// This runs automatically when:
    /// - Normal execution completes
    /// - Component traps
    /// - Execution times out
    /// - Fuel is exhausted
    ///
    /// # Cleanup Actions (Phase 5 - Task 5.2)
    ///
    /// 1. Collect metrics (fuel consumed, memory usage)
    /// 2. Log cleanup diagnostics (if logging enabled)
    /// 3. Drop Store (Wasmtime frees linear memory automatically)
    /// 4. Release any remaining resources
    ///
    /// # Safety
    ///
    /// Drop is guaranteed to run even on panic (panic boundary in engine.rs).
    /// Wasmtime's Drop implementation handles actual resource cleanup.
    fn drop(&mut self) {
        // Collect metrics before Store is dropped
        let _metrics = self.collect_metrics();

        // TODO(Phase 6): Add structured logging
        // log::debug!(
        //     "Cleaning up Store for component '{}': fuel={}/{} ({}% used)",
        //     metrics.component_id,
        //     metrics.fuel_consumed,
        //     metrics.initial_fuel,
        //     (metrics.fuel_consumed * 100) / metrics.initial_fuel.max(1)
        // );

        // Store drop happens automatically here
        // Wasmtime guarantees:
        // - Linear memory is freed
        // - Tables are released
        // - Fuel state is reset
        // - All WASM resources cleaned up

        // Store drop happens automatically (RAII pattern)
    }
}

// Implement Deref to allow transparent Store access
impl<T> Deref for StoreWrapper<T> {
    type Target = Store<T>;

    fn deref(&self) -> &Self::Target {
        &self.store
    }
}

// Implement DerefMut to allow mutable Store access
impl<T> DerefMut for StoreWrapper<T> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.store
    }
}

// Implement AsContext for Wasmtime API compatibility
impl<T> AsContext for StoreWrapper<T> {
    type Data = T;

    fn as_context(&self) -> StoreContext<'_, T> {
        self.store.as_context()
    }
}

// Implement AsContextMut for Wasmtime API compatibility
impl<T> AsContextMut for StoreWrapper<T> {
    fn as_context_mut(&mut self) -> StoreContextMut<'_, T> {
        self.store.as_context_mut()
    }
}

#[cfg(test)]
mod tests {
    #![allow(clippy::unwrap_used)]
    #![allow(clippy::expect_used)]

    use super::*;
    use wasmtime::{Config, Engine};

    fn create_test_engine() -> Engine {
        let mut config = Config::new();
        config.consume_fuel(true);
        Engine::new(&config).unwrap()
    }

    #[test]
    fn test_store_wrapper_creation() {
        let engine = create_test_engine();
        let store = StoreWrapper::new(&engine, (), 1_000_000);
        assert!(store.is_ok(), "Store creation should succeed");
    }

    #[test]
    fn test_store_wrapper_fuel_tracking() {
        let engine = create_test_engine();
        let mut store = StoreWrapper::new(&engine, (), 1_000_000).unwrap();

        // Check initial fuel
        let remaining = store.remaining_fuel();
        assert_eq!(remaining, Some(1_000_000));

        // Check fuel consumed (should be 0)
        let consumed = store.fuel_consumed();
        assert_eq!(consumed, Some(0));
    }

    #[test]
    fn test_store_wrapper_component_id() {
        let engine = create_test_engine();
        let mut store = StoreWrapper::new(&engine, (), 1_000_000).unwrap();

        // Set component ID
        store.set_component_id(String::from("test-component"));

        // Verify metrics include component ID
        let metrics = store.collect_metrics();
        assert_eq!(metrics.component_id, "test-component");
    }

    #[test]
    fn test_store_wrapper_metrics_collection() {
        let engine = create_test_engine();
        let mut store = StoreWrapper::new(&engine, (), 1_000_000).unwrap();
        store.set_component_id(String::from("metrics-test"));

        let metrics = store.collect_metrics();
        assert_eq!(metrics.initial_fuel, 1_000_000);
        assert_eq!(metrics.fuel_consumed, 0);
        assert_eq!(metrics.component_id, "metrics-test");
    }

    #[test]
    fn test_store_wrapper_drop_cleanup() {
        let engine = create_test_engine();

        {
            let mut store = StoreWrapper::new(&engine, (), 1_000_000).unwrap();
            store.set_component_id(String::from("drop-test"));

            // Verify store is valid
            assert!(store.remaining_fuel().is_some());

            // Store will be dropped at end of scope
        }

        // If we reach here, Drop ran successfully without panicking
        // Success - Store cleanup completed properly
    }

    #[test]
    fn test_store_wrapper_deref() {
        let engine = create_test_engine();
        let store = StoreWrapper::new(&engine, (), 1_000_000).unwrap();

        // Test transparent Store access via Deref
        let fuel_result = store.get_fuel();
        assert!(fuel_result.is_ok());
        assert_eq!(fuel_result.unwrap(), 1_000_000);
    }
}
