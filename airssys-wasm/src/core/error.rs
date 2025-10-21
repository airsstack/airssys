//! Comprehensive error types for airssys-wasm operations.
//!
//! This module defines the error handling strategy for the entire crate,
//! following workspace standards and Microsoft Rust Guidelines (M-ERRORS-CANONICAL-STRUCTS).
//!
//! # Error Philosophy
//!
//! - **Structured Errors**: All errors use structured variants with context
//! - **Actionable Messages**: Error messages guide users toward solutions
//! - **Source Chaining**: Errors carry source errors for debugging
//! - **Type Safety**: Error variants map to specific failure modes
//!
//! # Examples
//!
//! ```
//! use airssys_wasm::core::error::{WasmError, WasmResult};
//! use airssys_wasm::core::capability::{Capability, PathPattern};
//!
//! fn check_permission(cap: &Capability) -> WasmResult<()> {
//!     // Simulate permission denied
//!     Err(WasmError::capability_denied(
//!         cap.clone(),
//!         "Component does not have required capability"
//!     ))
//! }
//!
//! // Using the error
//! let cap = Capability::FileRead(PathPattern::new("/etc/passwd"));
//! match check_permission(&cap) {
//!     Ok(_) => println!("Permission granted"),
//!     Err(e) => eprintln!("Error: {}", e),
//! }
//! ```

// Layer 1: Standard library imports
use std::io;

// Layer 2: Third-party crate imports
use thiserror::Error;

// Layer 3: Internal module imports
use crate::core::capability::Capability;

/// Comprehensive error type for airssys-wasm operations.
///
/// All errors in airssys-wasm are represented by this enum, which provides
/// structured error information with context and source error chaining.
///
/// # Error Categories
///
/// - **Component Errors**: Loading, execution, trapping
/// - **Resource Errors**: Limits exceeded, timeouts
/// - **Security Errors**: Capability denials, permission failures
/// - **Configuration Errors**: Invalid settings, missing required fields
/// - **Integration Errors**: Storage, messaging, actor system failures
/// - **I/O Errors**: Filesystem, network operations
/// - **Internal Errors**: Unexpected failures (bugs)
///
/// # Examples
///
/// ```
/// use airssys_wasm::core::error::WasmError;
///
/// // Using helper constructors
/// let err = WasmError::component_not_found("my-component");
/// assert_eq!(err.to_string(), "Component not found: my-component");
///
/// // Resource limit error
/// let err = WasmError::resource_limit_exceeded("memory", 64 * 1024 * 1024, 128 * 1024 * 1024);
/// assert!(err.to_string().contains("memory"));
/// ```
#[derive(Error, Debug)]
pub enum WasmError {
    /// Component loading failed during instantiation.
    ///
    /// # Examples
    ///
    /// ```
    /// use airssys_wasm::core::error::WasmError;
    ///
    /// let err = WasmError::component_load_failed("image-processor", "Invalid WASM bytecode");
    /// assert!(err.to_string().contains("image-processor"));
    /// ```
    #[error("Failed to load component '{component_id}': {reason}")]
    ComponentLoadFailed {
        /// Component identifier that failed to load
        component_id: String,
        /// Reason for the load failure
        reason: String,
        /// Optional source error for debugging
        #[source]
        source: Option<Box<dyn std::error::Error + Send + Sync>>,
    },

    /// Component execution failed during function call.
    ///
    /// # Examples
    ///
    /// ```
    /// use airssys_wasm::core::error::WasmError;
    ///
    /// let err = WasmError::execution_failed("Function 'process' returned error");
    /// assert!(err.to_string().contains("execution failed"));
    /// ```
    #[error("Component execution failed: {reason}")]
    ExecutionFailed {
        /// Reason for the execution failure
        reason: String,
        /// Optional source error for debugging
        #[source]
        source: Option<Box<dyn std::error::Error + Send + Sync>>,
    },

    /// Component trapped with a WASM trap.
    ///
    /// Traps occur when a component violates WASM semantics (e.g., division by zero,
    /// out-of-bounds memory access, unreachable code).
    ///
    /// # Examples
    ///
    /// ```
    /// use airssys_wasm::core::error::WasmError;
    ///
    /// let err = WasmError::component_trapped("Division by zero", Some(50000));
    /// assert!(err.to_string().contains("trapped"));
    /// ```
    #[error("Component trapped: {reason}")]
    ComponentTrapped {
        /// Reason for the trap
        reason: String,
        /// Fuel consumed before trap (if metering enabled)
        fuel_consumed: Option<u64>,
    },

    /// Execution timeout exceeded maximum allowed time.
    ///
    /// # Examples
    ///
    /// ```
    /// use airssys_wasm::core::error::WasmError;
    ///
    /// let err = WasmError::execution_timeout(5000, Some(1000000));
    /// assert!(err.to_string().contains("5000ms"));
    /// ```
    #[error("Execution timeout exceeded ({max_execution_ms}ms)")]
    ExecutionTimeout {
        /// Maximum execution time allowed (milliseconds)
        max_execution_ms: u64,
        /// Fuel consumed before timeout (if metering enabled)
        fuel_consumed: Option<u64>,
    },

    /// Resource limit exceeded during execution.
    ///
    /// # Examples
    ///
    /// ```
    /// use airssys_wasm::core::error::WasmError;
    ///
    /// let err = WasmError::resource_limit_exceeded(
    ///     "memory",
    ///     64 * 1024 * 1024,      // 64MB limit
    ///     128 * 1024 * 1024      // 128MB attempted
    /// );
    /// assert!(err.to_string().contains("memory"));
    /// ```
    #[error("Resource limit exceeded: {resource} (limit: {limit}, attempted: {attempted})")]
    ResourceLimitExceeded {
        /// Resource type (e.g., "memory", "fuel", "storage")
        resource: String,
        /// Limit value
        limit: u64,
        /// Attempted value that exceeded limit
        attempted: u64,
    },

    /// Capability denied - component lacks required permission.
    ///
    /// # Examples
    ///
    /// ```
    /// use airssys_wasm::core::error::WasmError;
    /// use airssys_wasm::core::capability::{Capability, PathPattern};
    ///
    /// let cap = Capability::FileRead(PathPattern::new("/etc/passwd"));
    /// let err = WasmError::capability_denied(cap, "Component manifest does not declare capability");
    /// assert!(err.to_string().contains("Capability denied"));
    /// ```
    #[error("Capability denied: {capability:?} - {reason}")]
    CapabilityDenied {
        /// Capability that was denied
        capability: Capability,
        /// Reason for denial
        reason: String,
    },

    /// Invalid configuration provided to component or runtime.
    ///
    /// # Examples
    ///
    /// ```
    /// use airssys_wasm::core::error::WasmError;
    ///
    /// let err = WasmError::invalid_configuration("max_memory_bytes must be > 0");
    /// assert!(err.to_string().contains("Invalid configuration"));
    /// ```
    #[error("Invalid configuration: {reason}")]
    InvalidConfiguration {
        /// Reason for invalid configuration
        reason: String,
    },

    /// Component not found in registry or filesystem.
    ///
    /// # Examples
    ///
    /// ```
    /// use airssys_wasm::core::error::WasmError;
    ///
    /// let err = WasmError::component_not_found("non-existent-component");
    /// assert_eq!(err.to_string(), "Component not found: non-existent-component");
    /// ```
    #[error("Component not found: {component_id}")]
    ComponentNotFound {
        /// Component identifier that was not found
        component_id: String,
    },

    /// Storage operation failed.
    ///
    /// # Examples
    ///
    /// ```
    /// use airssys_wasm::core::error::WasmError;
    ///
    /// let err = WasmError::storage_error("Failed to write to cache namespace");
    /// assert!(err.to_string().contains("Storage error"));
    /// ```
    #[error("Storage error: {reason}")]
    StorageError {
        /// Reason for storage failure
        reason: String,
        /// Optional source error for debugging
        #[source]
        source: Option<Box<dyn std::error::Error + Send + Sync>>,
    },

    /// Messaging operation failed.
    ///
    /// # Examples
    ///
    /// ```
    /// use airssys_wasm::core::error::WasmError;
    ///
    /// let err = WasmError::messaging_error("Failed to publish to topic 'events.user'");
    /// assert!(err.to_string().contains("Messaging error"));
    /// ```
    #[error("Messaging error: {reason}")]
    MessagingError {
        /// Reason for messaging failure
        reason: String,
        /// Optional source error for debugging
        #[source]
        source: Option<Box<dyn std::error::Error + Send + Sync>>,
    },

    /// Actor system operation failed.
    ///
    /// # Examples
    ///
    /// ```
    /// use airssys_wasm::core::error::WasmError;
    ///
    /// let err = WasmError::actor_error("Actor mailbox full");
    /// assert!(err.to_string().contains("Actor system error"));
    /// ```
    #[error("Actor system error: {reason}")]
    ActorError {
        /// Reason for actor system failure
        reason: String,
        /// Optional source error for debugging
        #[source]
        source: Option<Box<dyn std::error::Error + Send + Sync>>,
    },

    /// I/O operation failed (filesystem, network).
    ///
    /// # Examples
    ///
    /// ```
    /// use airssys_wasm::core::error::WasmError;
    /// use std::io;
    ///
    /// let io_err = io::Error::new(io::ErrorKind::NotFound, "file not found");
    /// let err = WasmError::io_error("read /data/config.json", io_err);
    /// assert!(err.to_string().contains("I/O error"));
    /// ```
    #[error("I/O error: {operation}")]
    IoError {
        /// Operation that failed
        operation: String,
        /// Source I/O error
        #[source]
        source: io::Error,
    },

    /// Serialization or deserialization failed.
    ///
    /// # Examples
    ///
    /// ```
    /// use airssys_wasm::core::error::WasmError;
    ///
    /// let err = WasmError::serialization_error("Invalid JSON syntax");
    /// assert!(err.to_string().contains("Serialization error"));
    /// ```
    #[error("Serialization error: {reason}")]
    SerializationError {
        /// Reason for serialization failure
        reason: String,
        /// Optional source error for debugging
        #[source]
        source: Option<Box<dyn std::error::Error + Send + Sync>>,
    },

    /// Internal error - should not happen in normal operation.
    ///
    /// These errors indicate bugs in airssys-wasm itself and should be reported.
    ///
    /// # Examples
    ///
    /// ```
    /// use airssys_wasm::core::error::WasmError;
    ///
    /// let err = WasmError::internal("Unexpected state: component handle was None");
    /// assert!(err.to_string().contains("Internal error"));
    /// ```
    #[error("Internal error: {reason}")]
    Internal {
        /// Reason for internal error
        reason: String,
        /// Optional source error for debugging
        #[source]
        source: Option<Box<dyn std::error::Error + Send + Sync>>,
    },
}

/// Result type alias for airssys-wasm operations.
///
/// This type alias simplifies function signatures throughout the crate.
///
/// # Examples
///
/// ```
/// use airssys_wasm::core::error::WasmResult;
/// use airssys_wasm::core::component::ComponentId;
///
/// fn load_component(id: &ComponentId) -> WasmResult<()> {
///     // Component loading logic
///     Ok(())
/// }
/// ```
pub type WasmResult<T> = Result<T, WasmError>;

impl WasmError {
    /// Create a component load error.
    ///
    /// # Examples
    ///
    /// ```
    /// use airssys_wasm::core::error::WasmError;
    ///
    /// let err = WasmError::component_load_failed("image-processor", "Invalid WASM bytecode");
    /// assert!(err.to_string().contains("image-processor"));
    /// assert!(err.to_string().contains("Invalid WASM bytecode"));
    /// ```
    pub fn component_load_failed(
        component_id: impl Into<String>,
        reason: impl Into<String>,
    ) -> Self {
        Self::ComponentLoadFailed {
            component_id: component_id.into(),
            reason: reason.into(),
            source: None,
        }
    }

    /// Create a component load error with source.
    ///
    /// # Examples
    ///
    /// ```
    /// use airssys_wasm::core::error::WasmError;
    /// use std::io;
    ///
    /// let io_err = io::Error::new(io::ErrorKind::NotFound, "file not found");
    /// let err = WasmError::component_load_failed_with_source(
    ///     "my-component",
    ///     "Failed to read component file",
    ///     io_err
    /// );
    /// ```
    pub fn component_load_failed_with_source(
        component_id: impl Into<String>,
        reason: impl Into<String>,
        source: impl std::error::Error + Send + Sync + 'static,
    ) -> Self {
        Self::ComponentLoadFailed {
            component_id: component_id.into(),
            reason: reason.into(),
            source: Some(Box::new(source)),
        }
    }

    /// Create an execution failed error.
    ///
    /// # Examples
    ///
    /// ```
    /// use airssys_wasm::core::error::WasmError;
    ///
    /// let err = WasmError::execution_failed("Function returned error code 1");
    /// assert!(err.to_string().contains("execution failed"));
    /// ```
    pub fn execution_failed(reason: impl Into<String>) -> Self {
        Self::ExecutionFailed {
            reason: reason.into(),
            source: None,
        }
    }

    /// Create an execution failed error with source.
    pub fn execution_failed_with_source(
        reason: impl Into<String>,
        source: impl std::error::Error + Send + Sync + 'static,
    ) -> Self {
        Self::ExecutionFailed {
            reason: reason.into(),
            source: Some(Box::new(source)),
        }
    }

    /// Create a component trapped error.
    ///
    /// # Examples
    ///
    /// ```
    /// use airssys_wasm::core::error::WasmError;
    ///
    /// let err = WasmError::component_trapped("Division by zero", Some(50000));
    /// assert!(err.to_string().contains("trapped"));
    /// ```
    pub fn component_trapped(reason: impl Into<String>, fuel_consumed: Option<u64>) -> Self {
        Self::ComponentTrapped {
            reason: reason.into(),
            fuel_consumed,
        }
    }

    /// Create an execution timeout error.
    ///
    /// # Examples
    ///
    /// ```
    /// use airssys_wasm::core::error::WasmError;
    ///
    /// let err = WasmError::execution_timeout(5000, Some(1000000));
    /// assert!(err.to_string().contains("5000ms"));
    /// ```
    pub fn execution_timeout(max_execution_ms: u64, fuel_consumed: Option<u64>) -> Self {
        Self::ExecutionTimeout {
            max_execution_ms,
            fuel_consumed,
        }
    }

    /// Create a resource limit exceeded error.
    ///
    /// # Examples
    ///
    /// ```
    /// use airssys_wasm::core::error::WasmError;
    ///
    /// let err = WasmError::resource_limit_exceeded("memory", 64 * 1024 * 1024, 128 * 1024 * 1024);
    /// assert!(err.to_string().contains("memory"));
    /// assert!(err.to_string().contains("64"));
    /// ```
    pub fn resource_limit_exceeded(
        resource: impl Into<String>,
        limit: u64,
        attempted: u64,
    ) -> Self {
        Self::ResourceLimitExceeded {
            resource: resource.into(),
            limit,
            attempted,
        }
    }

    /// Create a capability denied error.
    ///
    /// # Examples
    ///
    /// ```
    /// use airssys_wasm::core::error::WasmError;
    /// use airssys_wasm::core::capability::{Capability, PathPattern};
    ///
    /// let cap = Capability::FileRead(PathPattern::new("/etc/passwd"));
    /// let err = WasmError::capability_denied(cap, "Not declared in manifest");
    /// assert!(err.to_string().contains("Capability denied"));
    /// ```
    pub fn capability_denied(capability: Capability, reason: impl Into<String>) -> Self {
        Self::CapabilityDenied {
            capability,
            reason: reason.into(),
        }
    }

    /// Create an invalid configuration error.
    ///
    /// # Examples
    ///
    /// ```
    /// use airssys_wasm::core::error::WasmError;
    ///
    /// let err = WasmError::invalid_configuration("max_memory_bytes must be > 0");
    /// assert!(err.to_string().contains("Invalid configuration"));
    /// ```
    pub fn invalid_configuration(reason: impl Into<String>) -> Self {
        Self::InvalidConfiguration {
            reason: reason.into(),
        }
    }

    /// Create a component not found error.
    ///
    /// # Examples
    ///
    /// ```
    /// use airssys_wasm::core::error::WasmError;
    ///
    /// let err = WasmError::component_not_found("my-component");
    /// assert_eq!(err.to_string(), "Component not found: my-component");
    /// ```
    pub fn component_not_found(component_id: impl Into<String>) -> Self {
        Self::ComponentNotFound {
            component_id: component_id.into(),
        }
    }

    /// Create a storage error.
    ///
    /// # Examples
    ///
    /// ```
    /// use airssys_wasm::core::error::WasmError;
    ///
    /// let err = WasmError::storage_error("Failed to write to cache");
    /// assert!(err.to_string().contains("Storage error"));
    /// ```
    pub fn storage_error(reason: impl Into<String>) -> Self {
        Self::StorageError {
            reason: reason.into(),
            source: None,
        }
    }

    /// Create a storage error with source.
    pub fn storage_error_with_source(
        reason: impl Into<String>,
        source: impl std::error::Error + Send + Sync + 'static,
    ) -> Self {
        Self::StorageError {
            reason: reason.into(),
            source: Some(Box::new(source)),
        }
    }

    /// Create a messaging error.
    ///
    /// # Examples
    ///
    /// ```
    /// use airssys_wasm::core::error::WasmError;
    ///
    /// let err = WasmError::messaging_error("Failed to publish message");
    /// assert!(err.to_string().contains("Messaging error"));
    /// ```
    pub fn messaging_error(reason: impl Into<String>) -> Self {
        Self::MessagingError {
            reason: reason.into(),
            source: None,
        }
    }

    /// Create a messaging error with source.
    pub fn messaging_error_with_source(
        reason: impl Into<String>,
        source: impl std::error::Error + Send + Sync + 'static,
    ) -> Self {
        Self::MessagingError {
            reason: reason.into(),
            source: Some(Box::new(source)),
        }
    }

    /// Create an actor system error.
    ///
    /// # Examples
    ///
    /// ```
    /// use airssys_wasm::core::error::WasmError;
    ///
    /// let err = WasmError::actor_error("Mailbox full");
    /// assert!(err.to_string().contains("Actor system error"));
    /// ```
    pub fn actor_error(reason: impl Into<String>) -> Self {
        Self::ActorError {
            reason: reason.into(),
            source: None,
        }
    }

    /// Create an actor system error with source.
    pub fn actor_error_with_source(
        reason: impl Into<String>,
        source: impl std::error::Error + Send + Sync + 'static,
    ) -> Self {
        Self::ActorError {
            reason: reason.into(),
            source: Some(Box::new(source)),
        }
    }

    /// Create an I/O error.
    ///
    /// # Examples
    ///
    /// ```
    /// use airssys_wasm::core::error::WasmError;
    /// use std::io;
    ///
    /// let io_err = io::Error::new(io::ErrorKind::NotFound, "file not found");
    /// let err = WasmError::io_error("read /data/config.json", io_err);
    /// assert!(err.to_string().contains("I/O error"));
    /// ```
    pub fn io_error(operation: impl Into<String>, source: io::Error) -> Self {
        Self::IoError {
            operation: operation.into(),
            source,
        }
    }

    /// Create a serialization error.
    ///
    /// # Examples
    ///
    /// ```
    /// use airssys_wasm::core::error::WasmError;
    ///
    /// let err = WasmError::serialization_error("Invalid JSON syntax at line 5");
    /// assert!(err.to_string().contains("Serialization error"));
    /// ```
    pub fn serialization_error(reason: impl Into<String>) -> Self {
        Self::SerializationError {
            reason: reason.into(),
            source: None,
        }
    }

    /// Create a serialization error with source.
    pub fn serialization_error_with_source(
        reason: impl Into<String>,
        source: impl std::error::Error + Send + Sync + 'static,
    ) -> Self {
        Self::SerializationError {
            reason: reason.into(),
            source: Some(Box::new(source)),
        }
    }

    /// Create an internal error.
    ///
    /// # Examples
    ///
    /// ```
    /// use airssys_wasm::core::error::WasmError;
    ///
    /// let err = WasmError::internal("Unexpected None value in component handle");
    /// assert!(err.to_string().contains("Internal error"));
    /// ```
    pub fn internal(reason: impl Into<String>) -> Self {
        Self::Internal {
            reason: reason.into(),
            source: None,
        }
    }

    /// Create an internal error with source.
    pub fn internal_with_source(
        reason: impl Into<String>,
        source: impl std::error::Error + Send + Sync + 'static,
    ) -> Self {
        Self::Internal {
            reason: reason.into(),
            source: Some(Box::new(source)),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::capability::PathPattern;
    use std::error::Error;

    #[test]
    fn test_component_load_failed() {
        let err = WasmError::component_load_failed("test-component", "Invalid bytecode");
        assert!(err.to_string().contains("test-component"));
        assert!(err.to_string().contains("Invalid bytecode"));
    }

    #[test]
    fn test_execution_failed() {
        let err = WasmError::execution_failed("Function returned error");
        assert!(err.to_string().contains("execution failed"));
    }

    #[test]
    fn test_component_trapped() {
        let err = WasmError::component_trapped("Division by zero", Some(50000));
        assert!(err.to_string().contains("trapped"));
        assert!(err.to_string().contains("Division by zero"));
    }

    #[test]
    fn test_execution_timeout() {
        let err = WasmError::execution_timeout(5000, Some(1000000));
        assert!(err.to_string().contains("5000ms"));
    }

    #[test]
    fn test_resource_limit_exceeded() {
        let err = WasmError::resource_limit_exceeded("memory", 64 * 1024 * 1024, 128 * 1024 * 1024);
        let msg = err.to_string();
        assert!(msg.contains("memory"));
        assert!(msg.contains("67108864")); // 64MB
        assert!(msg.contains("134217728")); // 128MB
    }

    #[test]
    fn test_capability_denied() {
        let cap = Capability::FileRead(PathPattern::new("/etc/passwd"));
        let err = WasmError::capability_denied(cap, "Not in manifest");
        assert!(err.to_string().contains("Capability denied"));
        assert!(err.to_string().contains("Not in manifest"));
    }

    #[test]
    fn test_invalid_configuration() {
        let err = WasmError::invalid_configuration("max_memory must be > 0");
        assert!(err.to_string().contains("Invalid configuration"));
        assert!(err.to_string().contains("max_memory must be > 0"));
    }

    #[test]
    fn test_component_not_found() {
        let err = WasmError::component_not_found("missing-component");
        assert_eq!(err.to_string(), "Component not found: missing-component");
    }

    #[test]
    fn test_storage_error() {
        let err = WasmError::storage_error("Write failed");
        assert!(err.to_string().contains("Storage error"));
        assert!(err.to_string().contains("Write failed"));
    }

    #[test]
    fn test_messaging_error() {
        let err = WasmError::messaging_error("Publish failed");
        assert!(err.to_string().contains("Messaging error"));
    }

    #[test]
    fn test_actor_error() {
        let err = WasmError::actor_error("Mailbox full");
        assert!(err.to_string().contains("Actor system error"));
    }

    #[test]
    fn test_io_error() {
        let io_err = io::Error::new(io::ErrorKind::NotFound, "file not found");
        let err = WasmError::io_error("read /data/config.json", io_err);
        assert!(err.to_string().contains("I/O error"));
        assert!(err.to_string().contains("read /data/config.json"));
    }

    #[test]
    fn test_serialization_error() {
        let err = WasmError::serialization_error("Invalid JSON");
        assert!(err.to_string().contains("Serialization error"));
    }

    #[test]
    fn test_internal_error() {
        let err = WasmError::internal("Unexpected None");
        assert!(err.to_string().contains("Internal error"));
    }

    #[test]
    fn test_error_with_source() {
        let io_err = io::Error::new(io::ErrorKind::PermissionDenied, "access denied");
        let err = WasmError::component_load_failed_with_source(
            "my-component",
            "Failed to read file",
            io_err,
        );

        assert!(err.to_string().contains("my-component"));
        assert!(err.to_string().contains("Failed to read file"));

        // Check source error is present
        assert!(err.source().is_some());
    }

    #[test]
    fn test_debug_format() -> Result<(), Box<dyn std::error::Error>> {
        let err = WasmError::component_not_found("test");
        let debug_str = format!("{err:?}");
        assert!(debug_str.contains("ComponentNotFound"));
        assert!(debug_str.contains("test"));
        Ok(())
    }

    #[test]
    fn test_wasm_result_type() {
        fn example_function() -> WasmResult<u32> {
            Ok(42)
        }

        match example_function() {
            Ok(val) => assert_eq!(val, 42),
            Err(_) => unreachable!("Should not error"),
        }
    }

    #[test]
    fn test_error_source_chaining() -> Result<(), Box<dyn std::error::Error>> {
        let io_err = io::Error::new(io::ErrorKind::NotFound, "file not found");
        let err = WasmError::storage_error_with_source("Failed to read cache", io_err);

        // Verify source is accessible
        let source = err.source();
        assert!(source.is_some());

        if let Some(s) = source {
            let source_str = s.to_string();
            assert!(source_str.contains("file not found"));
        }
        Ok(())
    }
}
