//! Component error types.
//!
//! This module contains error types for component lifecycle operations.
//! Errors are co-located with the component module per ADR-WASM-028.

// Layer 1: Standard library imports
// (none)

// Layer 2: Third-party crate imports
use thiserror::Error;

// Layer 3: Internal module imports
// (none - errors have no internal dependencies)

/// Component-related errors for lifecycle and operations.
///
/// This error type is used by the `ComponentLifecycle` trait for
/// initialization, shutdown, and other component operations.
///
/// # Architecture Note
///
/// ComponentError is co-located with the component module (in `core/component/errors.rs`)
/// rather than in a centralized `core/errors/` module. This ensures module isolation
/// and prevents cross-dependencies within `core/`.
#[derive(Debug, Clone, Error)]
pub enum ComponentError {
    /// Component initialization failed.
    #[error("Component initialization failed: {0}")]
    InitializationFailed(String),

    /// Component shutdown failed.
    #[error("Component shutdown failed: {0}")]
    ShutdownFailed(String),

    /// Component not found.
    #[error("Component not found: {0}")]
    NotFound(String),

    /// Component already exists.
    #[error("Component already exists: {0}")]
    AlreadyExists(String),

    /// Invalid component state.
    #[error("Invalid component state: {0}")]
    InvalidState(String),
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_initialization_failed_display() {
        let err = ComponentError::InitializationFailed("WASM module load error".to_string());
        assert_eq!(
            format!("{}", err),
            "Component initialization failed: WASM module load error"
        );
    }

    #[test]
    fn test_shutdown_failed_display() {
        let err = ComponentError::ShutdownFailed("cleanup error".to_string());
        assert_eq!(
            format!("{}", err),
            "Component shutdown failed: cleanup error"
        );
    }

    #[test]
    fn test_not_found_display() {
        let err = ComponentError::NotFound("app/service/001".to_string());
        assert_eq!(format!("{}", err), "Component not found: app/service/001");
    }

    #[test]
    fn test_already_exists_display() {
        let err = ComponentError::AlreadyExists("app/service/001".to_string());
        assert_eq!(
            format!("{}", err),
            "Component already exists: app/service/001"
        );
    }

    #[test]
    fn test_invalid_state_display() {
        let err = ComponentError::InvalidState("not initialized".to_string());
        assert_eq!(
            format!("{}", err),
            "Invalid component state: not initialized"
        );
    }

    #[test]
    fn test_error_is_clone() {
        let err = ComponentError::NotFound("test".to_string());
        let cloned = err.clone();
        assert!(matches!(cloned, ComponentError::NotFound(_)));
    }

    #[test]
    fn test_error_is_debug() {
        let err = ComponentError::InitializationFailed("test".to_string());
        let debug_str = format!("{:?}", err);
        assert!(debug_str.contains("InitializationFailed"));
    }

    // Gap analysis tests

    #[test]
    fn test_component_error_implements_std_error() {
        let err: Box<dyn std::error::Error> =
            Box::new(ComponentError::NotFound("test".to_string()));
        assert!(err.to_string().contains("Component not found"));
    }

    #[test]
    fn test_component_error_is_send_sync() {
        fn requires_send<T: Send>(_val: T) {}
        fn requires_sync<T: Sync>(_val: T) {}

        let err = ComponentError::NotFound("test".to_string());
        requires_send(err.clone());
        requires_sync(err);
    }
}
