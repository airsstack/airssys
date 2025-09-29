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
    /// Creates a new security violation error.
    pub fn security_violation(reason: impl Into<String>) -> Self {
        Self::SecurityViolation {
            reason: reason.into(),
        }
    }

    /// Creates a new middleware failure error.
    pub fn middleware_failed(middleware: impl Into<String>, reason: impl Into<String>) -> Self {
        Self::MiddlewareFailed {
            middleware: middleware.into(),
            reason: reason.into(),
        }
    }

    /// Creates a new execution failure error.
    pub fn execution_failed(reason: impl Into<String>) -> Self {
        Self::ExecutionFailed {
            reason: reason.into(),
        }
    }

    /// Creates a new filesystem error.
    pub fn filesystem_error(
        operation: impl Into<String>,
        path: impl Into<String>,
        reason: impl Into<String>,
    ) -> Self {
        Self::FilesystemError {
            operation: operation.into(),
            path: path.into(),
            reason: reason.into(),
        }
    }

    /// Creates a new process error.
    pub fn process_error(operation: impl Into<String>, reason: impl Into<String>) -> Self {
        Self::ProcessError {
            operation: operation.into(),
            reason: reason.into(),
        }
    }

    /// Creates a new network error.
    pub fn network_error(operation: impl Into<String>, reason: impl Into<String>) -> Self {
        Self::NetworkError {
            operation: operation.into(),
            reason: reason.into(),
        }
    }

    /// Creates a new configuration error.
    pub fn configuration_error(reason: impl Into<String>) -> Self {
        Self::ConfigurationError {
            reason: reason.into(),
        }
    }

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

    /// Returns true if this error should be retried automatically.
    pub fn is_retryable(&self) -> bool {
        matches!(
            self,
            OSError::NetworkError { .. } | OSError::ExecutionFailed { .. }
        )
    }

    /// Returns the error category for logging and metrics.
    pub fn category(&self) -> &'static str {
        match self {
            OSError::SecurityViolation { .. } => "security",
            OSError::MiddlewareFailed { .. } => "middleware",
            OSError::ExecutionFailed { .. } => "execution",
            OSError::FilesystemError { .. } => "filesystem",
            OSError::ProcessError { .. } => "process",
            OSError::NetworkError { .. } => "network",
            OSError::ConfigurationError { .. } => "configuration",
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_error_constructors() {
        let sec_err = OSError::security_violation("test violation");
        assert!(sec_err.is_security_violation());
        assert_eq!(sec_err.category(), "security");

        let fs_err = OSError::filesystem_error("read", "/tmp/test", "file not found");
        assert!(fs_err.is_filesystem_error());
        assert_eq!(fs_err.category(), "filesystem");

        let middleware_err = OSError::middleware_failed("logger", "write error");
        assert!(middleware_err.is_middleware_failure());
        assert_eq!(middleware_err.category(), "middleware");
    }

    #[test]
    fn test_error_categorization() {
        let network_err = OSError::network_error("connect", "connection refused");
        assert!(network_err.is_network_error());
        assert!(network_err.is_retryable());
        assert_eq!(network_err.category(), "network");

        let process_err = OSError::process_error("spawn", "permission denied");
        assert!(process_err.is_process_error());
        assert!(!process_err.is_retryable());
        assert_eq!(process_err.category(), "process");
    }

    #[test]
    fn test_error_display() {
        let sec_err = OSError::security_violation("unauthorized access");
        let error_msg = format!("{sec_err}");
        assert!(error_msg.contains("Security policy violation"));
        assert!(error_msg.contains("unauthorized access"));
    }
}
