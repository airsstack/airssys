//! Error types specific to logger middleware.
//!
//! This module defines structured error types for logger operations,
//! following Microsoft Rust Guidelines for canonical error structures.

// Layer 1: Standard library imports
use std::io;

// Layer 2: Third-party crate imports
use thiserror::Error;

// Layer 3: Internal module imports
// (none for this module)

/// Error types for logger middleware operations.
///
/// Provides comprehensive error categorization for all logger-related
/// failures with contextual information for debugging. Follows the
/// Microsoft Rust Guidelines M-ERRORS-CANONICAL-STRUCTS pattern.
///
/// # Design Principles
///
/// - **Contextual information**: Each error includes relevant details
/// - **Error chaining**: Preserves underlying error causes where applicable
/// - **Actionable messages**: Clear descriptions of what went wrong
/// - **Categorization**: Different error types for different failure modes
///
/// # Examples
///
/// ```rust
/// use airssys_osl::middleware::logger::LogError;
/// use std::io;
///
/// let error = LogError::Io {
///     operation: "write_log_file".to_string(),
///     path: "/var/log/app.log".to_string(),
///     source: io::Error::new(io::ErrorKind::PermissionDenied, "Permission denied"),
/// };
/// ```
#[derive(Debug, Error)]
pub enum LogError {
    /// I/O operation failed during logging.
    #[error("I/O error during {operation} on '{path}': {source}")]
    Io {
        /// The operation that failed (e.g., "write", "flush", "create")
        operation: String,
        /// The file path or resource involved
        path: String,
        /// The underlying I/O error
        #[source]
        source: io::Error,
    },

    /// Serialization of log entry failed.
    #[error("Failed to serialize log entry for operation '{operation_id}': {source}")]
    Serialization {
        /// The operation ID that failed to serialize
        operation_id: String,
        /// The underlying serialization error
        #[source]
        source: serde_json::Error,
    },

    /// Log formatting failed.
    #[error("Failed to format log entry for operation '{operation_id}': {message}")]
    Formatting {
        /// The operation ID that failed to format
        operation_id: String,
        /// Description of the formatting failure
        message: String,
    },

    /// Buffer overflow or capacity exceeded.
    #[error("Log buffer capacity exceeded: {current_size}/{max_size} entries")]
    BufferOverflow {
        /// Current number of entries in buffer
        current_size: usize,
        /// Maximum buffer capacity
        max_size: usize,
    },

    /// Logger configuration is invalid.
    #[error("Invalid logger configuration: {field} - {message}")]
    Configuration {
        /// The configuration field that is invalid
        field: String,
        /// Description of why the configuration is invalid
        message: String,
    },

    /// External system or dependency failure.
    #[error("External system failure in {system}: {message}")]
    External {
        /// Name of the external system (e.g., "tracing", "syslog")
        system: String,
        /// Description of the failure
        message: String,
        /// Optional underlying error
        #[source]
        source: Option<Box<dyn std::error::Error + Send + Sync>>,
    },
}

impl LogError {
    /// Create an I/O error with context.
    pub fn io(operation: impl Into<String>, path: impl Into<String>, source: io::Error) -> Self {
        Self::Io {
            operation: operation.into(),
            path: path.into(),
            source,
        }
    }

    /// Create a serialization error with context.
    pub fn serialization(operation_id: impl Into<String>, source: serde_json::Error) -> Self {
        Self::Serialization {
            operation_id: operation_id.into(),
            source,
        }
    }

    /// Create a formatting error with context.
    pub fn formatting(operation_id: impl Into<String>, message: impl Into<String>) -> Self {
        Self::Formatting {
            operation_id: operation_id.into(),
            message: message.into(),
        }
    }

    /// Create a buffer overflow error.
    pub fn buffer_overflow(current_size: usize, max_size: usize) -> Self {
        Self::BufferOverflow {
            current_size,
            max_size,
        }
    }

    /// Create a configuration error.
    pub fn configuration(field: impl Into<String>, message: impl Into<String>) -> Self {
        Self::Configuration {
            field: field.into(),
            message: message.into(),
        }
    }

    /// Create an external system error.
    pub fn external(
        system: impl Into<String>,
        message: impl Into<String>,
        source: Option<Box<dyn std::error::Error + Send + Sync>>,
    ) -> Self {
        Self::External {
            system: system.into(),
            message: message.into(),
            source,
        }
    }
}