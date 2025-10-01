//! Generic logger middleware implementation.
//!
//! This module contains the core LoggerMiddleware implementation that
//! integrates with the middleware pipeline to provide activity logging.

// Layer 1: Standard library imports
use std::sync::Arc;

// Layer 2: Third-party crate imports
use async_trait::async_trait;

// Layer 3: Internal module imports
use super::activity::{ActivityLog, ActivityLogger};
use super::config::LoggerConfig;
use crate::core::context::ExecutionContext;
use crate::core::executor::ExecutionResult;
use crate::core::middleware::{ErrorAction, Middleware, MiddlewareError, MiddlewareResult};
use crate::core::operation::Operation;
use crate::core::result::OSResult;

/// Generic logger middleware for activity logging and audit trails.
///
/// This middleware logs operation execution before and after processing,
/// providing comprehensive audit trails for security and debugging.
/// Uses generic constraints instead of dynamic dispatch for zero-cost abstractions.
///
/// # Type Parameters
///
/// - `L`: The concrete logger implementation that handles log output
///
/// # Examples
///
/// ```rust,ignore
/// use airssys_osl::middleware::logger::{LoggerMiddleware, ConsoleActivityLogger};
///
/// let logger = ConsoleActivityLogger::new();
/// let middleware = LoggerMiddleware::new(logger, LoggerConfig::default());
/// ```
///
/// Implementation will be completed in Phase 3.
#[derive(Debug)]
pub struct LoggerMiddleware<L: ActivityLogger> {
    logger: Arc<L>,
    config: LoggerConfig,
}

impl<L: ActivityLogger> LoggerMiddleware<L> {
    /// Create a new logger middleware with the specified logger and configuration.
    ///
    /// # Examples
    ///
    /// ```rust,ignore
    /// use airssys_osl::middleware::logger::{LoggerMiddleware, ConsoleActivityLogger, LoggerConfig};
    ///
    /// let logger = ConsoleActivityLogger::new();
    /// let config = LoggerConfig::development();
    /// let middleware = LoggerMiddleware::new(logger, config);
    /// ```
    pub fn new(logger: L, config: LoggerConfig) -> Self {
        Self {
            logger: Arc::new(logger),
            config,
        }
    }

    /// Create a new logger middleware with default configuration.
    ///
    /// # Examples
    ///
    /// ```rust,ignore
    /// use airssys_osl::middleware::logger::{LoggerMiddleware, ConsoleActivityLogger};
    ///
    /// let logger = ConsoleActivityLogger::new();
    /// let middleware = LoggerMiddleware::with_default_config(logger);
    /// ```
    pub fn with_default_config(logger: L) -> Self {
        Self::new(logger, LoggerConfig::default())
    }

    /// Get a reference to the logger configuration.
    pub fn config(&self) -> &LoggerConfig {
        &self.config
    }

    /// Get a reference to the underlying logger.
    pub fn logger(&self) -> &Arc<L> {
        &self.logger
    }
}

// Implementation of Middleware<O> trait for comprehensive activity logging
#[async_trait]
impl<O, L> Middleware<O> for LoggerMiddleware<L>
where
    O: Operation,
    L: ActivityLogger,
{
    fn name(&self) -> &str {
        "logger"
    }

    fn priority(&self) -> u32 {
        200 // Run after security middleware (priority 100)
    }

    async fn initialize(&mut self) -> MiddlewareResult<()> {
        // Flush any existing logs on initialization
        self.logger.flush().await.map_err(|e| {
            MiddlewareError::Dependency(format!("Logger initialization failed: {e}"))
        })?;
        Ok(())
    }

    async fn before_execution(
        &self,
        operation: O,
        context: &ExecutionContext,
    ) -> MiddlewareResult<Option<O>> {
        // Log operation start
        let log = ActivityLog::new(
            operation.operation_id(),
            format!("{:?}", operation.operation_type()),
            Some(context.principal().to_string()),
            "Started".to_string(),
            0, // Duration will be updated in after_execution
        );

        // Log asynchronously - don't block operation execution on logging errors
        if let Err(e) = self.logger.log_activity(log).await {
            // Convert LogError to MiddlewareError for proper error handling
            return Err(MiddlewareError::NonFatal(format!(
                "Failed to log operation start: {e}"
            )));
        }

        // Always pass through the operation unchanged
        Ok(Some(operation))
    }

    async fn after_execution(
        &self,
        context: &ExecutionContext,
        result: &OSResult<ExecutionResult>,
    ) -> MiddlewareResult<()> {
        // Create activity log based on execution result
        let (result_status, duration_ms) = match result {
            Ok(exec_result) => (
                format!("Success (exit: {})", exec_result.exit_code),
                exec_result.duration.as_millis() as u64,
            ),
            Err(error) => (
                format!("Error: {error}"),
                context.age().num_milliseconds() as u64,
            ),
        };

        let mut log = ActivityLog::new(
            context.execution_id.to_string(),
            "operation".to_string(), // We don't have operation type in context
            Some(context.principal().to_string()),
            result_status,
            duration_ms,
        );

        // Mark security-relevant operations (all operations for now)
        log = log.mark_security_relevant();

        // Add operation metadata if available
        if let Ok(exec_result) = result {
            if !exec_result.output.is_empty() {
                log = log.with_metadata("output_size".to_string(), exec_result.output.len());
            }

            if !exec_result.metadata.is_empty() {
                for (key, value) in &exec_result.metadata {
                    log = log.with_metadata(format!("exec_{key}"), value.clone());
                }
            }
        }

        // Add execution context metadata
        for (key, value) in &context.metadata {
            log = log.with_metadata(format!("ctx_{key}"), value.clone());
        }

        // Log the completed operation
        if let Err(e) = self.logger.log_activity(log).await {
            // Return non-fatal error - don't fail the operation due to logging issues
            return Err(MiddlewareError::NonFatal(format!(
                "Failed to log operation completion: {e}"
            )));
        }

        Ok(())
    }

    async fn handle_error(
        &self,
        error: crate::core::result::OSError,
        context: &ExecutionContext,
    ) -> ErrorAction {
        // Create error activity log
        let log = ActivityLog::new(
            context.execution_id.to_string(),
            "operation".to_string(),
            Some(context.principal().to_string()),
            format!("Error: {error}"),
            context.age().num_milliseconds() as u64,
        )
        .mark_security_relevant(); // All errors are security-relevant for audit

        // Log the error - ignore logging failures in error handler
        let _ = self.logger.log_activity(log).await;

        // Always continue with the original error
        ErrorAction::Continue
    }

    async fn shutdown(&mut self) -> MiddlewareResult<()> {
        // Flush all pending logs on shutdown
        self.logger
            .flush()
            .await
            .map_err(|e| MiddlewareError::Dependency(format!("Logger shutdown failed: {e}")))?;
        Ok(())
    }
}
