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
#[derive(Debug, Clone, PartialEq, Error)]
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

    /// Store not initialized.
    #[error("Store not initialized - call initialize() before using")]
    StoreNotInitialized,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_component_not_found_display() {
        let err = WasmError::ComponentNotFound("test-comp".to_string());
        assert!(format!("{}", err).contains("Component not found"));
        assert!(format!("{}", err).contains("test-comp"));
    }

    #[test]
    fn test_timeout_display() {
        let err = WasmError::Timeout;
        assert_eq!(format!("{}", err), "Execution timeout");
    }

    #[test]
    fn test_instantiation_failed_display() {
        let err = WasmError::InstantiationFailed("memory allocation failed".to_string());
        let display_str = format!("{}", err);
        assert!(display_str.contains("Component instantiation failed"));
        assert!(display_str.contains("memory allocation failed"));
    }

    #[test]
    fn test_export_not_found_display() {
        let err = WasmError::ExportNotFound("handle-message".to_string());
        let display_str = format!("{}", err);
        assert!(display_str.contains("Export not found"));
        assert!(display_str.contains("handle-message"));
    }

    #[test]
    fn test_resource_limit_exceeded_display() {
        let err = WasmError::ResourceLimitExceeded("memory limit of 64MB exceeded".to_string());
        let display_str = format!("{}", err);
        assert!(display_str.contains("Resource limit exceeded"));
        assert!(display_str.contains("memory limit of 64MB exceeded"));
    }

    #[test]
    fn test_invalid_component_display() {
        let err = WasmError::InvalidComponent("invalid WASM magic number".to_string());
        let display_str = format!("{}", err);
        assert!(display_str.contains("Invalid component"));
        assert!(display_str.contains("invalid WASM magic number"));
    }

    #[test]
    fn test_runtime_error_display() {
        let err = WasmError::RuntimeError("stack overflow".to_string());
        let display_str = format!("{}", err);
        assert!(display_str.contains("Runtime error"));
        assert!(display_str.contains("stack overflow"));
    }

    #[test]
    fn test_error_is_clone() {
        let err = WasmError::Timeout;
        let cloned = err.clone();
        assert!(matches!(cloned, WasmError::Timeout));
    }

    #[test]
    fn test_error_is_debug() {
        let err = WasmError::RuntimeError("test error".to_string());
        let debug_str = format!("{:?}", err);
        assert!(debug_str.contains("RuntimeError"));
    }

    #[test]
    fn test_error_with_empty_string() {
        let err = WasmError::ComponentNotFound(String::new());
        assert!(format!("{}", err).contains("Component not found"));
    }

    #[test]
    fn test_error_is_send_sync() {
        // Verify WasmError can be used in concurrent contexts
        fn requires_send<T: Send>(_val: T) {}
        fn requires_sync<T: Sync>(_val: T) {}

        let err = WasmError::Timeout;
        requires_send(err.clone());
        requires_sync(err);
    }

    #[test]
    fn test_store_not_initialized_display() {
        let err = WasmError::StoreNotInitialized;
        assert_eq!(
            format!("{}", err),
            "Store not initialized - call initialize() before using"
        );
    }
}
