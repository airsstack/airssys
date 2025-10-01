//! Tracing ecosystem integration logger implementation.
//!
//! This module provides a logger that integrates with the tracing ecosystem,
//! allowing activity logs to be processed by existing tracing subscribers.

// Layer 1: Standard library imports
// (none for this simple implementation)

// Layer 2: Third-party crate imports
use async_trait::async_trait;
use tracing::{error, info, warn};

// Layer 3: Internal module imports
use crate::middleware::logger::activity::{ActivityLog, ActivityLogger};
use crate::middleware::logger::error::LogError;

/// Tracing ecosystem integration logger.
///
/// Outputs activity logs through the tracing ecosystem, enabling
/// integration with existing tracing infrastructure and subscribers.
/// Provides a minimal bridge between ActivityLog and tracing events.
///
/// # Features
///
/// - **Tracing Integration**: Logs ActivityLog entries as tracing events
/// - **Level Mapping**: Maps log results to appropriate tracing levels
/// - **Structured Fields**: Includes operation metadata as structured fields
/// - **Zero Configuration**: No setup required, uses existing tracing subscribers
///
/// # Examples
///
/// ```rust,ignore
/// use airssys_osl::middleware::logger::loggers::TracingActivityLogger;
///
/// // Create a tracing logger (no configuration needed)
/// let logger = TracingActivityLogger::new();
/// ```
#[derive(Debug, Default, Clone)]
pub struct TracingActivityLogger {
    // No fields needed - uses global tracing infrastructure
}

impl TracingActivityLogger {
    /// Create a new tracing logger.
    ///
    /// Uses the global tracing subscriber configured by the application.
    /// No configuration is needed as it leverages existing tracing setup.
    pub fn new() -> Self {
        Self {}
    }
}

#[async_trait]
impl ActivityLogger for TracingActivityLogger {
    async fn log_activity(&self, log: ActivityLog) -> Result<(), LogError> {
        // Map result to appropriate tracing level
        let is_error = log.result.starts_with("Error");
        let is_warning = log.result.contains("warn") || log.result.contains("timeout");

        // Log with structured fields using the tracing macros
        if is_error {
            error!(
                operation_id = %log.operation_id,
                operation_type = %log.operation_type,
                user_context = ?log.user_context,
                result = %log.result,
                duration_ms = log.duration_ms,
                security_relevant = log.security_relevant,
                metadata = ?log.metadata,
                "Activity completed with error"
            );
        } else if is_warning {
            warn!(
                operation_id = %log.operation_id,
                operation_type = %log.operation_type,
                user_context = ?log.user_context,
                result = %log.result,
                duration_ms = log.duration_ms,
                security_relevant = log.security_relevant,
                metadata = ?log.metadata,
                "Activity completed with warning"
            );
        } else {
            info!(
                operation_id = %log.operation_id,
                operation_type = %log.operation_type,
                user_context = ?log.user_context,
                result = %log.result,
                duration_ms = log.duration_ms,
                security_relevant = log.security_relevant,
                metadata = ?log.metadata,
                "Activity completed successfully"
            );
        }

        Ok(())
    }

    async fn flush(&self) -> Result<(), LogError> {
        // Tracing subscribers handle their own flushing
        // No explicit flush needed for this implementation
        Ok(())
    }
}
