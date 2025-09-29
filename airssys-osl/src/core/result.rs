//! Error types and result types for the OS Layer Framework.
//!
//! This module provides structured error handling following Microsoft Rust
//! Guidelines M-ERRORS-CANONICAL-STRUCTS pattern.

use thiserror::Error;

/// Result type alias for OS Layer Framework operations.
pub type OSResult<T> = Result<T, OSError>;

/// Comprehensive error types for OS Layer Framework operations.
///
/// Provides structured error information with contextual helper methods
/// for categorization and error handling patterns.
#[derive(Error, Debug, Clone)]
pub enum OSError {
    /// Security policy violation error
    #[error("Security policy violation: {reason}")]
    SecurityViolation { reason: String },
    
    /// Middleware processing error
    #[error("Middleware '{middleware}' failed: {reason}")]
    MiddlewareFailed { middleware: String, reason: String },
    
    /// Operation execution error
    #[error("Operation execution failed: {reason}")]
    ExecutionFailed { reason: String },
    
    /// Filesystem operation error
    #[error("Filesystem operation failed: {operation} on '{path}': {reason}")]
    FilesystemError {
        operation: String,
        path: String,
        reason: String,
    },
    
    /// Process management error
    #[error("Process operation failed: {operation}: {reason}")]
    ProcessError { operation: String, reason: String },
    
    /// Network operation error
    #[error("Network operation failed: {operation}: {reason}")]
    NetworkError { operation: String, reason: String },
    
    /// Configuration error
    #[error("Configuration error: {reason}")]
    ConfigurationError { reason: String },
}

impl OSError {
    /// Returns true if this error represents a security policy violation.
    pub fn is_security_violation(&self) -> bool {
        matches!(self, OSError::SecurityViolation { .. })
    }
    
    /// Returns true if this error represents a middleware failure.
    pub fn is_middleware_failure(&self) -> bool {
        matches!(self, OSError::MiddlewareFailed { .. })
    }
    
    /// Returns true if this error represents a filesystem operation failure.
    pub fn is_filesystem_error(&self) -> bool {
        matches!(self, OSError::FilesystemError { .. })
    }
    
    /// Returns true if this error represents a process operation failure.
    pub fn is_process_error(&self) -> bool {
        matches!(self, OSError::ProcessError { .. })
    }
    
    /// Returns true if this error represents a network operation failure.
    pub fn is_network_error(&self) -> bool {
        matches!(self, OSError::NetworkError { .. })
    }
    
    /// Returns true if this error represents a configuration error.
    pub fn is_configuration_error(&self) -> bool {
        matches!(self, OSError::ConfigurationError { .. })
    }
}