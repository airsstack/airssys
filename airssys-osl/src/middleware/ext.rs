//! Middleware extension trait for executor composition.
//!
//! This module provides the [`ExecutorExt`] trait which enables ergonomic
//! middleware composition using the extension method pattern.
//!
//! The extension trait allows any executor to be wrapped with middleware using
//! the `.with_middleware()` method, creating a composable pipeline of middleware
//! that can be chained together.
//!
//! See [`ExecutorExt`] trait documentation for usage details.

// Layer 1: Standard library imports
use std::sync::Arc;

// Layer 2: Third-party imports
use async_trait::async_trait;

// Layer 3: Internal module imports
use crate::core::context::ExecutionContext;
use crate::core::executor::{ExecutionResult, OSExecutor};
use crate::core::middleware::Middleware;
use crate::core::operation::{Operation, OperationType};
use crate::core::result::OSResult;

/// Middleware-wrapped executor that applies middleware to operation execution.
///
/// This struct wraps an executor and a middleware, creating a new executor
/// that applies the middleware's hooks during operation execution.
///
/// # Type Parameters
///
/// - `E`: The inner executor type
/// - `M`: The middleware type
/// - `O`: The operation type
#[derive(Debug)]
pub struct MiddlewareExecutor<E, M, O>
where
    E: OSExecutor<O>,
    M: Middleware<O>,
    O: Operation,
{
    executor: E,
    middleware: Arc<M>,
    _phantom: std::marker::PhantomData<O>,
}

impl<E, M, O> MiddlewareExecutor<E, M, O>
where
    E: OSExecutor<O>,
    M: Middleware<O>,
    O: Operation,
{
    /// Creates a new middleware-wrapped executor.
    ///
    /// # Arguments
    ///
    /// * `executor` - The inner executor to wrap
    /// * `middleware` - The middleware to apply
    pub fn new(executor: E, middleware: M) -> Self {
        Self {
            executor,
            middleware: Arc::new(middleware),
            _phantom: std::marker::PhantomData,
        }
    }
}

#[async_trait]
impl<E, M, O> OSExecutor<O> for MiddlewareExecutor<E, M, O>
where
    E: OSExecutor<O> + Send + Sync + std::fmt::Debug,
    M: Middleware<O> + Send + Sync + std::fmt::Debug,
    O: Operation + Send + Sync + std::fmt::Debug,
{
    fn name(&self) -> &str {
        self.executor.name()
    }

    fn supported_operation_types(&self) -> Vec<OperationType> {
        self.executor.supported_operation_types()
    }

    async fn execute(&self, operation: O, context: &ExecutionContext) -> OSResult<ExecutionResult> {
        // Check if middleware can process this operation
        if !self.middleware.can_process(&operation, context).await {
            // Middleware doesn't apply, execute directly
            return self.executor.execute(operation, context).await;
        }

        // Apply before_execution hook - it consumes and may transform the operation
        let operation = match self.middleware.before_execution(operation, context).await {
            Ok(Some(op)) => op, // Continue with possibly modified operation
            Ok(None) => {
                // Middleware handled it, return early with empty result
                return Ok(ExecutionResult::success(Vec::new()));
            }
            Err(e) => {
                // Convert MiddlewareError to OSError
                use crate::core::result::OSError;
                return Err(OSError::execution_failed(format!(
                    "Middleware error in before_execution: {e:?}"
                )));
            }
        };

        // Execute the operation
        let result = self.executor.execute(operation, context).await;

        // Apply after_execution hook
        match self.middleware.after_execution(context, &result).await {
            Ok(()) => {} // Continue
            Err(e) => {
                // Log middleware error but don't fail the operation
                eprintln!("Middleware error in after_execution: {e:?}");
            }
        }

        // Handle errors if execution failed
        if let Err(ref error) = result {
            let error_action = self.middleware.handle_error(error.clone(), context).await;
            match error_action {
                crate::core::middleware::ErrorAction::Stop => {
                    // Stop processing, return the error
                    return result;
                }
                crate::core::middleware::ErrorAction::Continue => {
                    // Continue with the error
                    return result;
                }
                _ => {
                    // For other actions, just continue with the error
                    return result;
                }
            }
        }

        result
    }

    async fn validate_operation(&self, operation: &O, context: &ExecutionContext) -> OSResult<()> {
        self.executor.validate_operation(operation, context).await
    }
}

/// Extension trait for adding middleware to executors.
///
/// This trait provides the [`with_middleware`](ExecutorExt::with_middleware) method
/// which enables ergonomic middleware composition on any executor.
///
/// # Automatic Implementation
///
/// This trait is automatically implemented for all sized types, making it available
/// on all executors without requiring explicit implementation.
pub trait ExecutorExt: Sized {
    /// Wraps this executor with the given middleware.
    ///
    /// This method creates a new [`MiddlewareExecutor`] that applies the middleware's
    /// hooks during operation execution.
    ///
    /// # Type Parameters
    ///
    /// - `M`: The middleware type
    /// - `O`: The operation type
    ///
    /// # Arguments
    ///
    /// * `middleware` - The middleware to apply to this executor
    ///
    /// # Returns
    ///
    /// A new executor that wraps this executor with the given middleware
    fn with_middleware<M, O>(self, middleware: M) -> MiddlewareExecutor<Self, M, O>
    where
        Self: OSExecutor<O>,
        M: Middleware<O>,
        O: Operation,
    {
        MiddlewareExecutor::new(self, middleware)
    }
}

// Blanket implementation for all sized types
impl<E> ExecutorExt for E where E: Sized {}

#[cfg(test)]
#[allow(clippy::unwrap_used)] // Test code - unwrap is acceptable for test setup
mod tests {
    use super::*;
    use crate::core::context::SecurityContext;
    use crate::core::middleware::{ErrorAction, MiddlewareResult};
    use crate::executors::filesystem::FilesystemExecutor;
    use crate::operations::filesystem::FileReadOperation;

    // Mock middleware for testing
    #[derive(Debug)]
    struct MockMiddleware {
        can_process_called: std::sync::Arc<std::sync::atomic::AtomicBool>,
        before_called: std::sync::Arc<std::sync::atomic::AtomicBool>,
        after_called: std::sync::Arc<std::sync::atomic::AtomicBool>,
    }

    impl MockMiddleware {
        fn new() -> Self {
            Self {
                can_process_called: std::sync::Arc::new(std::sync::atomic::AtomicBool::new(false)),
                before_called: std::sync::Arc::new(std::sync::atomic::AtomicBool::new(false)),
                after_called: std::sync::Arc::new(std::sync::atomic::AtomicBool::new(false)),
            }
        }

        #[allow(dead_code)]
        fn was_can_process_called(&self) -> bool {
            self.can_process_called
                .load(std::sync::atomic::Ordering::SeqCst)
        }

        #[allow(dead_code)]
        fn was_before_called(&self) -> bool {
            self.before_called.load(std::sync::atomic::Ordering::SeqCst)
        }

        #[allow(dead_code)]
        fn was_after_called(&self) -> bool {
            self.after_called.load(std::sync::atomic::Ordering::SeqCst)
        }
    }

    #[async_trait]
    impl Middleware<FileReadOperation> for MockMiddleware {
        fn name(&self) -> &str {
            "mock_middleware"
        }

        fn priority(&self) -> u32 {
            100
        }

        async fn can_process(
            &self,
            _operation: &FileReadOperation,
            _context: &ExecutionContext,
        ) -> bool {
            self.can_process_called
                .store(true, std::sync::atomic::Ordering::SeqCst);
            true
        }

        async fn before_execution(
            &self,
            operation: FileReadOperation,
            _context: &ExecutionContext,
        ) -> MiddlewareResult<Option<FileReadOperation>> {
            self.before_called
                .store(true, std::sync::atomic::Ordering::SeqCst);
            Ok(Some(operation))
        }

        async fn after_execution(
            &self,
            _context: &ExecutionContext,
            _result: &OSResult<ExecutionResult>,
        ) -> MiddlewareResult<()> {
            self.after_called
                .store(true, std::sync::atomic::Ordering::SeqCst);
            Ok(())
        }

        async fn handle_error(
            &self,
            _error: crate::core::result::OSError,
            _context: &ExecutionContext,
        ) -> ErrorAction {
            ErrorAction::Continue
        }
    }

    #[test]
    fn test_executor_ext_trait_available() {
        // Test that the trait is available on executors
        let executor = FilesystemExecutor::new();
        let middleware = MockMiddleware::new();

        // This should compile, proving the extension trait works
        let _wrapped = executor.with_middleware(middleware);
    }

    #[test]
    fn test_middleware_executor_creation() {
        // Test creating a middleware executor
        let executor = FilesystemExecutor::new();
        let middleware = MockMiddleware::new();

        let wrapped = MiddlewareExecutor::new(executor, middleware);

        // Verify name is preserved
        assert_eq!(wrapped.name(), "filesystem-executor");
    }

    #[test]
    fn test_middleware_executor_preserves_operation_types() {
        // Test that wrapped executor preserves operation types
        let executor = FilesystemExecutor::new();
        let middleware = MockMiddleware::new();

        let wrapped = executor.with_middleware(middleware);

        let types = wrapped.supported_operation_types();
        assert_eq!(types.len(), 1);
        assert_eq!(types[0], OperationType::Filesystem);
    }

    #[tokio::test]
    async fn test_middleware_executor_calls_hooks() {
        // Test that middleware hooks are called during execution
        use std::io::Write;
        use tempfile::NamedTempFile;

        // Create a temporary file
        let mut temp_file = NamedTempFile::new().unwrap();
        write!(temp_file, "test data").unwrap();
        let file_path = temp_file.path().display().to_string();

        let executor = FilesystemExecutor::new();
        let middleware = MockMiddleware::new();

        // Capture references before moving into wrapped executor
        let can_process_ref = Arc::clone(&middleware.can_process_called);
        let before_ref = Arc::clone(&middleware.before_called);
        let after_ref = Arc::clone(&middleware.after_called);

        let wrapped = executor.with_middleware(middleware);

        // Execute an operation
        let operation = FileReadOperation::new(file_path);
        let context = ExecutionContext::new(SecurityContext::new("test_user".to_string()));

        let result = wrapped.execute(operation, &context).await;
        assert!(result.is_ok());

        // Verify all hooks were called
        assert!(can_process_ref.load(std::sync::atomic::Ordering::SeqCst));
        assert!(before_ref.load(std::sync::atomic::Ordering::SeqCst));
        assert!(after_ref.load(std::sync::atomic::Ordering::SeqCst));
    }

    #[test]
    fn test_middleware_chaining() {
        // Test that multiple middleware can be chained
        let executor = FilesystemExecutor::new();
        let middleware1 = MockMiddleware::new();
        let middleware2 = MockMiddleware::new();

        // Chain two middleware
        let wrapped = executor
            .with_middleware(middleware1)
            .with_middleware(middleware2);

        // Verify name is still preserved through the chain
        assert_eq!(wrapped.name(), "filesystem-executor");
    }
}
