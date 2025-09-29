//! Middleware abstractions for operation processing pipelines.
//!
//! This module defines the middleware system that allows for modular
//! processing of operations before, during, and after execution.

use std::fmt::Debug;

use async_trait::async_trait;

use crate::core::context::ExecutionContext;
use crate::core::operation::Operation;
use crate::core::result::{OSError, OSResult};

/// Result type for middleware operations.
pub type MiddlewareResult<T> = Result<T, MiddlewareError>;

/// Error types specific to middleware processing.
///
/// Provides fine-grained control over middleware error handling and
/// pipeline flow control.
#[derive(Debug, Clone)]
pub enum MiddlewareError {
    /// Fatal error that should stop the pipeline immediately
    Fatal(String),
    
    /// Non-fatal error that should be logged but allow pipeline continuation
    NonFatal(String),
    
    /// Security violation that requires audit logging and pipeline termination
    SecurityViolation(String),
}

impl MiddlewareError {
    /// Returns true if this error should stop pipeline processing.
    pub fn is_fatal(&self) -> bool {
        matches!(self, MiddlewareError::Fatal(_) | MiddlewareError::SecurityViolation(_))
    }
    
    /// Returns true if this error represents a security violation.
    pub fn is_security_violation(&self) -> bool {
        matches!(self, MiddlewareError::SecurityViolation(_))
    }
    
    /// Converts this middleware error to an OS error.
    pub fn to_os_error(self, middleware_name: &str) -> OSError {
        match self {
            MiddlewareError::SecurityViolation(reason) => OSError::SecurityViolation {
                reason: format!("Middleware '{middleware_name}': {reason}"),
            },
            MiddlewareError::Fatal(reason) | MiddlewareError::NonFatal(reason) => {
                OSError::MiddlewareFailed {
                    middleware: middleware_name.to_string(),
                    reason,
                }
            }
        }
    }
}

/// Action to take when middleware encounters an error.
///
/// Provides fine-grained control over error handling behavior in
/// middleware pipelines.
#[derive(Debug, Clone)]
pub enum ErrorAction {
    /// Continue processing with the original error
    Continue,
    
    /// Replace the original error with a different error
    ReplaceError(OSError),
    
    /// Suppress the error and continue (use with extreme caution)
    Suppress,
}

/// Core trait for middleware components in the operation processing pipeline.
///
/// Middleware components can intercept operations before execution, modify
/// execution context, handle errors, and perform post-execution processing.
/// 
/// # Generic Parameters
/// 
/// * `O` - The operation type this middleware can process
/// 
/// # Design Notes
/// 
/// This trait uses generic constraints to maintain type safety and avoid
/// runtime dispatch overhead. Middleware components are composable and
/// can be chained together to form processing pipelines.
#[async_trait]
pub trait Middleware<O>: Debug + Send + Sync + 'static
where
    O: Operation,
{
    /// Returns the name of this middleware for logging and identification.
    fn name(&self) -> &str;
    
    /// Processes an operation before execution.
    ///
    /// This method is called before the operation is passed to the executor.
    /// It can modify the operation, validate security policies, or reject
    /// the operation entirely.
    /// 
    /// # Arguments
    /// 
    /// * `operation` - The operation to process
    /// * `context` - The execution context
    /// 
    /// # Returns
    /// 
    /// Returns `Ok(Some(operation))` to continue with the (possibly modified) operation,
    /// `Ok(None)` to skip execution (middleware handled it), or `Err` to reject.
    async fn before_execution(
        &self,
        operation: O,
        _context: &ExecutionContext,
    ) -> MiddlewareResult<Option<O>> {
        // Default implementation passes through unchanged
        Ok(Some(operation))
    }
    
    /// Handles errors that occur during operation processing.
    ///
    /// This method is called when an error occurs during operation execution
    /// or processing by other middleware components.
    /// 
    /// # Arguments
    /// 
    /// * `error` - The error that occurred
    /// * `context` - The execution context
    /// 
    /// # Returns
    /// 
    /// Returns an `ErrorAction` indicating how to handle the error.
    async fn handle_error(&self, _error: OSError, _context: &ExecutionContext) -> ErrorAction {
        // Default implementation continues with the original error
        ErrorAction::Continue
    }
    
    /// Performs cleanup or post-processing after operation completion.
    ///
    /// This method is called after operation execution completes, regardless
    /// of success or failure. It can be used for cleanup, logging, or
    /// additional processing.
    /// 
    /// # Arguments
    /// 
    /// * `context` - The execution context
    /// * `result` - The result of the operation execution
    async fn after_execution(
        &self,
        _context: &ExecutionContext,
        _result: &OSResult<crate::core::executor::ExecutionResult>,
    ) -> MiddlewareResult<()> {
        // Default implementation does nothing
        Ok(())
    }
}