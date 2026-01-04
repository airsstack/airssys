//! Activity logging types and core trait definition.
//!
//! This module defines the core types for structured activity logging,
//! including the ActivityLog structure and ActivityLogger trait.

// Layer 1: Standard library imports
use std::collections::HashMap;

// Layer 2: Third-party crate imports
use async_trait::async_trait;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use serde_json;

// Layer 3: Internal module imports
use super::error::LogError;

/// Structured log entry representing a single OS operation activity.
///
/// This structure contains comprehensive metadata about an operation
/// execution, suitable for audit trails and debugging. All timestamps
/// use UTC for consistency across different time zones.
///
/// # Fields
///
/// - **timestamp**: When the activity occurred (UTC)
/// - **operation_id**: Unique identifier for the operation instance
/// - **operation_type**: String representation of the operation type
/// - **user_context**: Optional user/principal information
/// - **result**: Success or failure status with details
/// - **duration_ms**: How long the operation took to execute
/// - **metadata**: Additional structured data about the operation
/// - **security_relevant**: Flag indicating if this requires security audit
///
/// # Examples
///
/// ```rust
/// use airssys_osl::middleware::logger::ActivityLog;
/// use chrono::Utc;
/// use std::collections::HashMap;
///
/// let log = ActivityLog {
///     timestamp: Utc::now(),
///     operation_id: "op_123".to_string(),
///     operation_type: "file_read".to_string(),
///     user_context: Some("user123".to_string()),
///     result: "Success".to_string(),
///     duration_ms: 150,
///     metadata: HashMap::new(),
///     security_relevant: true,
/// };
/// ```
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ActivityLog {
    /// UTC timestamp when the activity occurred (§3.2 compliance)
    pub timestamp: DateTime<Utc>,

    /// Unique identifier for this specific operation instance
    pub operation_id: String,

    /// String representation of the operation type (e.g., "file_read", "process_spawn")
    pub operation_type: String,

    /// Optional user/principal context information
    pub user_context: Option<String>,

    /// Result status and details (success/failure with context)
    pub result: String,

    /// Duration of operation execution in milliseconds
    pub duration_ms: u64,

    /// Additional structured metadata about the operation
    pub metadata: HashMap<String, serde_json::Value>,

    /// Flag indicating if this operation requires security audit trail
    pub security_relevant: bool,
}

impl ActivityLog {
    /// Create a new ActivityLog with the current timestamp.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use airssys_osl::middleware::logger::ActivityLog;
    ///
    /// let log = ActivityLog::new(
    ///     "op_123".to_string(),
    ///     "file_read".to_string(),
    ///     Some("user123".to_string()),
    ///     "Success".to_string(),
    ///     150,
    /// );
    /// ```
    pub fn new(
        operation_id: String,
        operation_type: String,
        user_context: Option<String>,
        result: String,
        duration_ms: u64,
    ) -> Self {
        Self {
            timestamp: Utc::now(),
            operation_id,
            operation_type,
            user_context,
            result,
            duration_ms,
            metadata: HashMap::new(),
            security_relevant: false,
        }
    }

    /// Mark this activity as security-relevant for audit trails.
    pub fn mark_security_relevant(mut self) -> Self {
        self.security_relevant = true;
        self
    }

    /// Add metadata key-value pair to the activity log.
    pub fn with_metadata<V>(mut self, key: String, value: V) -> Self
    where
        V: Into<serde_json::Value>,
    {
        self.metadata.insert(key, value.into());
        self
    }
}

/// Core trait for pluggable activity logging destinations.
///
/// Implementations can target different output destinations (console, file,
/// tracing, external systems) while maintaining a consistent interface.
/// All methods are async to support non-blocking I/O operations.
///
/// # Design Principles
///
/// - **Async-first**: All operations are non-blocking
/// - **Error handling**: Comprehensive error reporting with LogError
/// - **Flushing support**: Explicit flushing for buffered implementations
/// - **Thread safety**: Must be Send + Sync for concurrent usage
///
/// # Examples
///
/// ```rust,ignore
/// use airssys_osl::middleware::logger::{ActivityLog, ActivityLogger};
///
/// struct MyLogger;
///
/// #[async_trait::async_trait]
/// impl ActivityLogger for MyLogger {
///     async fn log_activity(&self, log: ActivityLog) -> Result<(), LogError> {
///         println!("Logging: {:?}", log);
///         Ok(())
///     }
///     
///     async fn flush(&self) -> Result<(), LogError> {
///         // Flush any buffered logs
///         Ok(())
///     }
/// }
/// ```
#[async_trait]
pub trait ActivityLogger: std::fmt::Debug + Send + Sync + 'static {
    /// Log a single activity entry.
    ///
    /// This method should handle the activity log entry according to the
    /// specific implementation (write to file, send to console, etc.).
    ///
    /// # Errors
    ///
    /// Returns `LogError` if the logging operation fails. Implementations
    /// should provide detailed error context for debugging.
    async fn log_activity(&self, log: ActivityLog) -> Result<(), LogError>;

    /// Flush any buffered log entries.
    ///
    /// This method ensures that all pending log entries are written to their
    /// destination. It should be called before shutdown or when immediate
    /// consistency is required.
    ///
    /// # Errors
    ///
    /// Returns `LogError` if the flush operation fails.
    async fn flush(&self) -> Result<(), LogError>;
}
