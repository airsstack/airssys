//! Middleware abstractions for operation processing pipelines.
//!
//! This module defines the middleware system that allows for modular
//! processing of operations before, during, and after execution.

use std::fmt::Debug;
use std::time::Duration;

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

    /// Timeout error for middleware operations
    Timeout(Duration),

    /// Configuration error in middleware setup
    Configuration(String),

    /// Dependency error when middleware dependencies are not met
    Dependency(String),
}

impl MiddlewareError {
    /// Returns true if this error should stop pipeline processing.
    pub fn is_fatal(&self) -> bool {
        matches!(
            self,
            MiddlewareError::Fatal(_)
                | MiddlewareError::SecurityViolation(_)
                | MiddlewareError::Timeout(_)
                | MiddlewareError::Configuration(_)
                | MiddlewareError::Dependency(_)
        )
    }

    /// Returns true if this error represents a security violation.
    pub fn is_security_violation(&self) -> bool {
        matches!(self, MiddlewareError::SecurityViolation(_))
    }

    /// Returns true if this error is retryable.
    pub fn is_retryable(&self) -> bool {
        matches!(
            self,
            MiddlewareError::NonFatal(_) | MiddlewareError::Timeout(_)
        )
    }

    /// Returns the error category for logging and metrics.
    pub fn category(&self) -> &'static str {
        match self {
            MiddlewareError::Fatal(_) => "fatal",
            MiddlewareError::NonFatal(_) => "non_fatal",
            MiddlewareError::SecurityViolation(_) => "security",
            MiddlewareError::Timeout(_) => "timeout",
            MiddlewareError::Configuration(_) => "configuration",
            MiddlewareError::Dependency(_) => "dependency",
        }
    }

    /// Converts this middleware error to an OS error.
    pub fn to_os_error(self, middleware_name: &str) -> OSError {
        match self {
            MiddlewareError::SecurityViolation(reason) => OSError::SecurityViolation {
                reason: format!("Middleware '{middleware_name}': {reason}"),
            },
            MiddlewareError::Fatal(reason)
            | MiddlewareError::NonFatal(reason)
            | MiddlewareError::Configuration(reason)
            | MiddlewareError::Dependency(reason) => OSError::MiddlewareFailed {
                middleware: middleware_name.to_string(),
                reason,
            },
            MiddlewareError::Timeout(duration) => OSError::MiddlewareFailed {
                middleware: middleware_name.to_string(),
                reason: format!("Timeout after {duration:?}"),
            },
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

    /// Retry the operation with optional delay
    Retry {
        /// Maximum number of retry attempts
        max_attempts: u32,
        /// Delay between retry attempts
        delay: Duration,
    },

    /// Stop the entire pipeline immediately
    Stop,

    /// Log the error and continue with a warning
    LogAndContinue,
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

    /// Returns the priority of this middleware (lower numbers = higher priority).
    fn priority(&self) -> u32 {
        100 // Default priority
    }

    /// Returns true if this middleware is enabled.
    fn is_enabled(&self) -> bool {
        true // Default enabled
    }

    /// Returns the operation types this middleware can handle.
    fn supported_operation_types(&self) -> Vec<crate::core::operation::OperationType> {
        // Default: handle all operation types
        vec![]
    }

    /// Initializes the middleware.
    ///
    /// This method is called once when the middleware is added to a pipeline.
    /// It can be used for setup, resource allocation, or configuration validation.
    async fn initialize(&mut self) -> MiddlewareResult<()> {
        // Default implementation does nothing
        Ok(())
    }

    /// Validates whether this middleware can process the given operation.
    async fn can_process(&self, operation: &O, _context: &ExecutionContext) -> bool {
        if !self.is_enabled() {
            return false;
        }

        let supported = self.supported_operation_types();
        if supported.is_empty() {
            return true; // Support all if none specified
        }

        supported.contains(&operation.operation_type())
    }

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

    /// Processes an operation during execution (optional hook).
    ///
    /// This method can be called during operation execution to provide
    /// monitoring, progress tracking, or intervention capabilities.
    async fn during_execution(&self, _context: &ExecutionContext) -> MiddlewareResult<()> {
        // Default implementation does nothing
        Ok(())
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

    /// Shuts down the middleware.
    ///
    /// This method is called when the middleware is being removed from a pipeline
    /// or when the pipeline is shutting down. It can be used for cleanup,
    /// resource deallocation, or final logging.
    async fn shutdown(&mut self) -> MiddlewareResult<()> {
        // Default implementation does nothing
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::operation::{Operation, OperationType, Permission};
    use chrono::{DateTime, Utc};

    // Mock operation for testing
    #[derive(Debug, Clone)]
    struct MockOperation {
        id: String,
        op_type: OperationType,
        permissions: Vec<Permission>,
        created_at: DateTime<Utc>,
    }

    impl MockOperation {
        fn new(id: &str, op_type: OperationType, permissions: Vec<Permission>) -> Self {
            Self {
                id: id.to_string(),
                op_type,
                permissions,
                created_at: Utc::now(),
            }
        }
    }

    impl Operation for MockOperation {
        fn operation_id(&self) -> String {
            self.id.clone()
        }

        fn operation_type(&self) -> OperationType {
            self.op_type
        }

        fn required_permissions(&self) -> Vec<Permission> {
            self.permissions.clone()
        }

        fn created_at(&self) -> DateTime<Utc> {
            self.created_at
        }
    }

    // Mock middleware for testing trait functionality
    #[derive(Debug)]
    struct MockMiddleware {
        name: String,
        enabled: bool,
        priority: u32,
        supported_ops: Vec<OperationType>,
    }

    impl MockMiddleware {
        fn new(name: &str) -> Self {
            Self {
                name: name.to_string(),
                enabled: true,
                priority: 100,
                supported_ops: vec![],
            }
        }

        fn with_priority(mut self, priority: u32) -> Self {
            self.priority = priority;
            self
        }

        fn with_enabled(mut self, enabled: bool) -> Self {
            self.enabled = enabled;
            self
        }

        fn with_supported_operations(mut self, ops: Vec<OperationType>) -> Self {
            self.supported_ops = ops;
            self
        }
    }

    #[async_trait]
    impl Middleware<MockOperation> for MockMiddleware {
        fn name(&self) -> &str {
            &self.name
        }

        fn priority(&self) -> u32 {
            self.priority
        }

        fn is_enabled(&self) -> bool {
            self.enabled
        }

        fn supported_operation_types(&self) -> Vec<OperationType> {
            self.supported_ops.clone()
        }
    }

    #[tokio::test]
    async fn test_middleware_error_categorization() {
        use std::time::Duration;

        let timeout_error = MiddlewareError::Timeout(Duration::from_secs(30));
        assert!(timeout_error.is_retryable());
        assert_eq!(timeout_error.category(), "timeout");

        let fatal_error = MiddlewareError::Fatal("test".to_string());
        assert!(!fatal_error.is_retryable());
        assert_eq!(fatal_error.category(), "fatal");

        let non_fatal_error = MiddlewareError::NonFatal("test".to_string());
        assert!(non_fatal_error.is_retryable());
        assert_eq!(non_fatal_error.category(), "non_fatal");

        let security_error = MiddlewareError::SecurityViolation("test".to_string());
        assert!(!security_error.is_retryable());
        assert_eq!(security_error.category(), "security");

        let config_error = MiddlewareError::Configuration("test".to_string());
        assert!(!config_error.is_retryable());
        assert_eq!(config_error.category(), "configuration");

        let dependency_error = MiddlewareError::Dependency("test".to_string());
        assert!(!dependency_error.is_retryable());
        assert_eq!(dependency_error.category(), "dependency");
    }

    #[tokio::test]
    async fn test_error_action_variants() {
        use std::time::Duration;

        // Test Continue variant
        let continue_action = ErrorAction::Continue;
        assert!(matches!(continue_action, ErrorAction::Continue));

        // Test Stop variant
        let stop_action = ErrorAction::Stop;
        assert!(matches!(stop_action, ErrorAction::Stop));

        // Test LogAndContinue variant
        let log_continue = ErrorAction::LogAndContinue;
        assert!(matches!(log_continue, ErrorAction::LogAndContinue));

        // Test Retry variant
        let retry_action = ErrorAction::Retry {
            max_attempts: 3,
            delay: Duration::from_millis(100),
        };
        assert!(matches!(retry_action, ErrorAction::Retry { .. }));

        // Test Suppress variant
        let suppress_action = ErrorAction::Suppress;
        assert!(matches!(suppress_action, ErrorAction::Suppress));

        // Test ReplaceError variant
        let replace_action =
            ErrorAction::ReplaceError(crate::core::result::OSError::ExecutionFailed {
                reason: "test".to_string(),
            });
        assert!(matches!(replace_action, ErrorAction::ReplaceError(_)));
    }

    #[tokio::test]
    async fn test_middleware_trait_defaults() {
        use crate::core::context::SecurityContext;

        let mut middleware = MockMiddleware::new("test-middleware");
        let security_context = SecurityContext::new("test-user".to_string());
        let context = ExecutionContext::new(security_context);

        let operation = MockOperation::new(
            "test-op",
            OperationType::Filesystem,
            vec![crate::core::operation::Permission::FilesystemRead(
                "/test".to_string(),
            )],
        );

        // Test default implementations
        assert_eq!(middleware.name(), "test-middleware");
        assert_eq!(middleware.priority(), 100);
        assert!(middleware.is_enabled());
        assert!(middleware.supported_operation_types().is_empty());

        // Test initialization
        assert!(middleware.initialize().await.is_ok());

        // Test can_process with empty supported types (should accept all)
        assert!(middleware.can_process(&operation, &context).await);

        // Test before_execution default
        let result = middleware
            .before_execution(operation.clone(), &context)
            .await;
        assert!(result.is_ok());
        assert!(result.expect("Valid execution").is_some());

        // Test during_execution default
        assert!(middleware.during_execution(&context).await.is_ok());

        // Test handle_error default
        let error = crate::core::result::OSError::ExecutionFailed {
            reason: "test".to_string(),
        };
        let action = middleware.handle_error(error, &context).await;
        assert!(matches!(action, ErrorAction::Continue));

        // Test after_execution default
        let exec_result = Ok(crate::core::executor::ExecutionResult::success(
            b"test".to_vec(),
        ));
        assert!(middleware
            .after_execution(&context, &exec_result)
            .await
            .is_ok());

        // Test shutdown
        assert!(middleware.shutdown().await.is_ok());
    }

    #[tokio::test]
    async fn test_middleware_can_process_filtering() {
        use crate::core::context::SecurityContext;

        let security_context = SecurityContext::new("test-user".to_string());
        let context = ExecutionContext::new(security_context);

        let fs_operation = MockOperation::new(
            "fs-op",
            OperationType::Filesystem,
            vec![crate::core::operation::Permission::FilesystemRead(
                "/test".to_string(),
            )],
        );
        let process_operation = MockOperation::new(
            "proc-op",
            OperationType::Process,
            vec![crate::core::operation::Permission::ProcessSpawn],
        );

        // Test disabled middleware
        let disabled = MockMiddleware::new("disabled").with_enabled(false);
        assert!(!disabled.can_process(&fs_operation, &context).await);

        // Test specific operation type support
        let fs_only = MockMiddleware::new("fs-only")
            .with_supported_operations(vec![OperationType::Filesystem]);
        assert!(fs_only.can_process(&fs_operation, &context).await);
        assert!(!fs_only.can_process(&process_operation, &context).await);

        // Test multiple operation type support
        let multi_support = MockMiddleware::new("multi")
            .with_supported_operations(vec![OperationType::Filesystem, OperationType::Process]);
        assert!(multi_support.can_process(&fs_operation, &context).await);
        assert!(
            multi_support
                .can_process(&process_operation, &context)
                .await
        );
    }

    #[tokio::test]
    async fn test_middleware_priority_system() {
        let high_priority = MockMiddleware::new("high").with_priority(10);
        let low_priority = MockMiddleware::new("low").with_priority(200);
        let default_priority = MockMiddleware::new("default");

        assert_eq!(high_priority.priority(), 10);
        assert_eq!(low_priority.priority(), 200);
        assert_eq!(default_priority.priority(), 100);

        // Verify ordering (lower number = higher priority)
        assert!(high_priority.priority() < default_priority.priority());
        assert!(default_priority.priority() < low_priority.priority());
    }
}
