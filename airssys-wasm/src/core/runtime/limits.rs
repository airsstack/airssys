//! # Resource Limits
//!
//! Resource limits for WASM component execution.
//!
//! This module defines resource constraints for WASM components to prevent
//! resource exhaustion and ensure fair resource allocation.
//!
//! # Types
//!
//! - [`ResourceLimits`] - Configurable resource constraints
//!
//! # Default Limits
//!
//! - Memory: 64 MB
//! - Execution time: 30 seconds
//! - Fuel: Unlimited

// Layer 1: Standard library imports (per PROJECTS_STANDARD.md §2.1)

// Layer 2: Third-party crate imports (per PROJECTS_STANDARD.md §2.1)
// None needed for this module

// Layer 3: Internal module imports (per PROJECTS_STANDARD.md §2.1)
// None needed for this module (core/ can only import std and own submodules)

/// Resource limits for WASM component execution.
///
/// `ResourceLimits` defines execution constraints that prevent runaway
/// WASM components from consuming excessive resources. These limits
/// are enforced during component execution by the runtime engine.
///
/// # Fields
///
/// - `max_memory_bytes`: Maximum memory allocation in bytes
/// - `max_execution_time_ms`: Maximum execution time in milliseconds
/// - `max_fuel`: Optional fuel limit for instruction counting
///
/// # Architecture Note
///
/// `ResourceLimits` is a pure data structure in `core/runtime/` (Layer 1).
/// It defines WHAT constraints exist, while the runtime/ module (Layer 3B)
/// implements HOW these constraints are enforced during WASM execution.
///
/// # Examples
///
/// ## Using default resource limits
///
/// ```rust
/// use airssys_wasm::core::runtime::limits::ResourceLimits;
///
/// let limits = ResourceLimits::default();
///
/// assert_eq!(limits.max_memory_bytes, 64 * 1024 * 1024); // 64MB
/// assert_eq!(limits.max_execution_time_ms, 30_000); // 30 seconds
/// assert_eq!(limits.max_fuel, None); // No fuel limit
/// ```
///
/// ## Creating custom resource limits
///
/// ```rust
/// use airssys_wasm::core::runtime::limits::ResourceLimits;
///
/// let limits = ResourceLimits {
///     max_memory_bytes: 128 * 1024 * 1024, // 128MB
///     max_execution_time_ms: 60_000, // 60 seconds
///     max_fuel: Some(1_000_000), // 1 million fuel units
/// };
/// ```
///
/// ## Enforcing resource limits
///
/// Resource limits are enforced by the runtime engine during WASM execution.
/// If a component exceeds any limit, execution is halted and an error is returned.
///
/// ```rust
/// use airssys_wasm::core::runtime::limits::ResourceLimits;
///
/// let limits = ResourceLimits {
///     max_memory_bytes: 16 * 1024 * 1024, // 16MB
///     max_execution_time_ms: 10_000, // 10 seconds
///     max_fuel: Some(500_000),
/// };
///
/// // Runtime engine enforces these limits during execution
/// // Engine returns WasmError if limits are exceeded
/// ```
#[derive(Debug, Clone)]
pub struct ResourceLimits {
    /// Maximum memory allocation in bytes.
    ///
    /// Prevents components from allocating excessive memory.
    /// If a component attempts to allocate more than this limit,
    /// the runtime engine halts execution and returns an error.
    pub max_memory_bytes: u64,

    /// Maximum execution time in milliseconds.
    ///
    /// Prevents infinite loops or long-running computations.
    /// If component execution exceeds this duration, the runtime engine
    /// terminates execution and returns a timeout error.
    pub max_execution_time_ms: u64,

    /// Optional fuel limit for instruction counting.
    ///
    /// Fuel limits provide fine-grained control over execution
    /// by counting virtual machine instructions. When fuel reaches zero,
    /// execution halts. If `None`, no fuel limit is enforced.
    pub max_fuel: Option<u64>,
}

impl Default for ResourceLimits {
    /// Creates default resource limits.
    ///
    /// Defaults are conservative for typical WASM components:
    /// - Memory: 64MB
    /// - Time: 30 seconds
    /// - Fuel: No limit (None)
    ///
    /// # Examples
    ///
    /// ```rust
    /// use airssys_wasm::core::runtime::limits::ResourceLimits;
    ///
    /// let limits = ResourceLimits::default();
    ///
    /// assert_eq!(limits.max_memory_bytes, 64 * 1024 * 1024);
    /// assert_eq!(limits.max_execution_time_ms, 30_000);
    /// assert_eq!(limits.max_fuel, None);
    /// ```
    fn default() -> Self {
        Self {
            max_memory_bytes: 64 * 1024 * 1024, // 64MB
            max_execution_time_ms: 30_000,      // 30 seconds
            max_fuel: None,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_resource_limits_default_values() {
        let limits = ResourceLimits::default();

        assert_eq!(limits.max_memory_bytes, 64 * 1024 * 1024); // 64MB
        assert_eq!(limits.max_execution_time_ms, 30_000); // 30 seconds
        assert_eq!(limits.max_fuel, None); // No fuel limit
    }

    #[test]
    fn test_resource_limits_custom_values() {
        let limits = ResourceLimits {
            max_memory_bytes: 128 * 1024 * 1024, // 128MB
            max_execution_time_ms: 60_000,       // 60 seconds
            max_fuel: Some(1_000_000),
        };

        assert_eq!(limits.max_memory_bytes, 128 * 1024 * 1024);
        assert_eq!(limits.max_execution_time_ms, 60_000);
        assert_eq!(limits.max_fuel, Some(1_000_000));
    }

    #[test]
    fn test_resource_limits_clone_creates_independent_copy() {
        let limits1 = ResourceLimits {
            max_memory_bytes: 100,
            max_execution_time_ms: 200,
            max_fuel: Some(300),
        };
        let limits2 = limits1.clone();

        assert_eq!(limits1.max_memory_bytes, limits2.max_memory_bytes);
        assert_eq!(limits1.max_execution_time_ms, limits2.max_execution_time_ms);
        assert_eq!(limits1.max_fuel, limits2.max_fuel);

        // Modify original to verify independence
        let mut limits1_mut = limits1;
        limits1_mut.max_memory_bytes = 999;

        assert_eq!(limits2.max_memory_bytes, 100); // Clone unchanged
        assert_eq!(limits1_mut.max_memory_bytes, 999); // Original modified
    }

    #[test]
    fn test_resource_limits_debug_format() {
        let limits = ResourceLimits::default();
        let debug_str = format!("{:?}", limits);

        // Verify Debug trait is implemented and shows structure
        assert!(debug_str.contains("ResourceLimits"));
        assert!(debug_str.contains("max_memory_bytes"));
        assert!(debug_str.contains("max_execution_time_ms"));
        assert!(debug_str.contains("max_fuel"));
    }

    #[test]
    fn test_resource_limits_with_none_fuel() {
        let limits = ResourceLimits {
            max_memory_bytes: 32 * 1024 * 1024,
            max_execution_time_ms: 15_000,
            max_fuel: None,
        };

        assert!(limits.max_fuel.is_none());
    }

    #[test]
    fn test_resource_limits_with_fuel_limit() {
        let limits = ResourceLimits {
            max_memory_bytes: 32 * 1024 * 1024,
            max_execution_time_ms: 15_000,
            max_fuel: Some(250_000),
        };

        assert_eq!(limits.max_fuel, Some(250_000));
    }

    #[test]
    fn test_resource_limits_zero_limits() {
        let limits = ResourceLimits {
            max_memory_bytes: 0,
            max_execution_time_ms: 0,
            max_fuel: Some(0),
        };

        assert_eq!(limits.max_memory_bytes, 0);
        assert_eq!(limits.max_execution_time_ms, 0);
        assert_eq!(limits.max_fuel, Some(0));
    }

    #[test]
    fn test_resource_limits_large_limits() {
        let limits = ResourceLimits {
            max_memory_bytes: u64::MAX,
            max_execution_time_ms: u64::MAX,
            max_fuel: Some(u64::MAX),
        };

        assert_eq!(limits.max_memory_bytes, u64::MAX);
        assert_eq!(limits.max_execution_time_ms, u64::MAX);
        assert_eq!(limits.max_fuel, Some(u64::MAX));
    }
}
