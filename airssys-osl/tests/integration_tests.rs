//! Integration tests for airssys-osl core functionality.
//!
//! These tests verify that core traits work together correctly and that
//! the overall system behaves as expected when components are integrated.

#![allow(dead_code)] // Allow helper methods that may not be used in all tests
#![allow(clippy::unwrap_used, clippy::expect_used)] // Allow in test code for clarity

use std::sync::Arc;
use std::time::Duration;

use async_trait::async_trait;
use chrono::{DateTime, Utc};
use tokio::time::sleep;

use airssys_osl::core::{
    context::{ExecutionContext, SecurityContext},
    executor::{ExecutionResult, OSExecutor},
    middleware::{ErrorAction, Middleware, MiddlewareError, MiddlewareResult},
    operation::{Operation, OperationType, Permission},
    result::{OSError, OSResult},
};

// Test operation implementation
#[derive(Debug, Clone)]
struct TestOperation {
    id: String,
    op_type: OperationType,
    permissions: Vec<Permission>,
    created_at: DateTime<Utc>,
    should_fail: bool,
    execution_time: Duration,
}

impl TestOperation {
    fn new(id: &str, op_type: OperationType) -> Self {
        Self {
            id: id.to_string(),
            op_type,
            permissions: vec![Permission::FilesystemRead("/test".to_string())],
            created_at: Utc::now(),
            should_fail: false,
            execution_time: Duration::from_millis(10),
        }
    }

    fn with_failure(mut self) -> Self {
        self.should_fail = true;
        self
    }

    fn with_execution_time(mut self, duration: Duration) -> Self {
        self.execution_time = duration;
        self
    }

    fn with_permissions(mut self, permissions: Vec<Permission>) -> Self {
        self.permissions = permissions;
        self
    }
}

impl Operation for TestOperation {
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

// Test executor implementation
#[derive(Debug)]
struct TestExecutor {
    name: String,
    supported_types: Vec<OperationType>,
    should_timeout: bool,
}

impl TestExecutor {
    fn new(name: &str) -> Self {
        Self {
            name: name.to_string(),
            supported_types: vec![OperationType::Filesystem, OperationType::Process],
            should_timeout: false,
        }
    }

    fn with_timeout(mut self) -> Self {
        self.should_timeout = true;
        self
    }

    fn with_supported_types(mut self, types: Vec<OperationType>) -> Self {
        self.supported_types = types;
        self
    }
}

#[async_trait]
impl OSExecutor<TestOperation> for TestExecutor {
    fn name(&self) -> &str {
        &self.name
    }

    fn supported_operation_types(&self) -> Vec<OperationType> {
        self.supported_types.clone()
    }

    async fn can_execute(
        &self,
        operation: &TestOperation,
        _context: &ExecutionContext,
    ) -> OSResult<bool> {
        Ok(self.supported_types.contains(&operation.operation_type()))
    }

    async fn execute(
        &self,
        operation: TestOperation,
        _context: &ExecutionContext,
    ) -> OSResult<ExecutionResult> {
        if self.should_timeout {
            sleep(Duration::from_secs(10)).await;
        } else {
            sleep(operation.execution_time).await;
        }

        if operation.should_fail {
            Err(OSError::ExecutionFailed {
                reason: "Test operation configured to fail".to_string(),
            })
        } else {
            let output = format!("Executed operation: {}", operation.operation_id());
            Ok(ExecutionResult::success(output.into_bytes()))
        }
    }

    async fn execute_with_timeout(
        &self,
        operation: TestOperation,
        context: &ExecutionContext,
        timeout: Duration,
    ) -> OSResult<ExecutionResult> {
        match tokio::time::timeout(timeout, self.execute(operation, context)).await {
            Ok(result) => result,
            Err(_) => Err(OSError::ExecutionFailed {
                reason: format!("Operation timed out after {timeout:?}"),
            }),
        }
    }

    async fn validate_operation(
        &self,
        operation: &TestOperation,
        _context: &ExecutionContext,
    ) -> OSResult<()> {
        if operation.operation_id().is_empty() {
            return Err(OSError::ExecutionFailed {
                reason: "Operation ID cannot be empty".to_string(),
            });
        }
        Ok(())
    }

    async fn cleanup(&self, _context: &ExecutionContext) -> OSResult<()> {
        // Test cleanup - nothing to do
        Ok(())
    }
}

// Test middleware implementation
#[derive(Debug)]
struct TestMiddleware {
    name: String,
    should_reject: bool,
    should_error: bool,
    transform_operation: bool,
}

impl TestMiddleware {
    fn new(name: &str) -> Self {
        Self {
            name: name.to_string(),
            should_reject: false,
            should_error: false,
            transform_operation: false,
        }
    }

    fn with_rejection(mut self) -> Self {
        self.should_reject = true;
        self
    }

    fn with_error(mut self) -> Self {
        self.should_error = true;
        self
    }

    fn with_transformation(mut self) -> Self {
        self.transform_operation = true;
        self
    }
}

#[async_trait]
impl Middleware<TestOperation> for TestMiddleware {
    fn name(&self) -> &str {
        &self.name
    }

    async fn before_execution(
        &self,
        mut operation: TestOperation,
        _context: &ExecutionContext,
    ) -> MiddlewareResult<Option<TestOperation>> {
        if self.should_error {
            return Err(MiddlewareError::Fatal("Test middleware error".to_string()));
        }

        if self.should_reject {
            return Ok(None); // Reject the operation
        }

        if self.transform_operation {
            operation.id = format!("{}-transformed", operation.id);
        }

        Ok(Some(operation))
    }

    async fn handle_error(&self, error: OSError, _context: &ExecutionContext) -> ErrorAction {
        match error {
            OSError::ExecutionFailed { .. } => ErrorAction::LogAndContinue,
            _ => ErrorAction::Continue,
        }
    }
}

// Integration tests
#[tokio::test]
async fn test_executor_basic_execution() {
    let executor = TestExecutor::new("test-executor");
    let security_context = SecurityContext::new("test-user".to_string());
    let context = ExecutionContext::new(security_context);
    let operation = TestOperation::new("test-op", OperationType::Filesystem);

    // Test basic execution flow
    assert!(executor.can_execute(&operation, &context).await.unwrap());
    assert!(executor
        .validate_operation(&operation, &context)
        .await
        .is_ok());

    let result = executor.execute(operation.clone(), &context).await;
    assert!(result.is_ok());

    let exec_result = result.unwrap();
    assert_eq!(exec_result.exit_code, 0);
    assert!(exec_result.output_as_string().unwrap().contains("test-op"));

    // Test cleanup
    let cleanup_result = executor.cleanup(&context).await;
    assert!(cleanup_result.is_ok());
}

#[tokio::test]
async fn test_executor_timeout_handling() {
    let executor = TestExecutor::new("timeout-executor").with_timeout();
    let security_context = SecurityContext::new("test-user".to_string());
    let context = ExecutionContext::new(security_context);
    let operation = TestOperation::new("timeout-op", OperationType::Filesystem);

    // Test timeout handling
    let result = executor
        .execute_with_timeout(operation, &context, Duration::from_millis(100))
        .await;

    assert!(result.is_err());
    if let Err(OSError::ExecutionFailed { reason }) = result {
        assert!(reason.contains("timed out"));
    } else {
        unreachable!("Expected ExecutionFailed error");
    }
}

#[tokio::test]
async fn test_executor_operation_type_filtering() {
    let fs_executor =
        TestExecutor::new("fs-executor").with_supported_types(vec![OperationType::Filesystem]);
    let security_context = SecurityContext::new("test-user".to_string());
    let context = ExecutionContext::new(security_context);

    let fs_operation = TestOperation::new("fs-op", OperationType::Filesystem);
    let process_operation = TestOperation::new("proc-op", OperationType::Process);

    // Test operation type filtering
    assert!(fs_executor
        .can_execute(&fs_operation, &context)
        .await
        .unwrap());
    assert!(!fs_executor
        .can_execute(&process_operation, &context)
        .await
        .unwrap());
}

#[tokio::test]
async fn test_middleware_operation_transformation() {
    let middleware = TestMiddleware::new("transform-middleware").with_transformation();
    let security_context = SecurityContext::new("test-user".to_string());
    let context = ExecutionContext::new(security_context);
    let operation = TestOperation::new("test-op", OperationType::Filesystem);

    let result = middleware.before_execution(operation, &context).await;
    assert!(result.is_ok());

    let transformed = result.unwrap();
    assert!(transformed.is_some());
    assert_eq!(transformed.unwrap().operation_id(), "test-op-transformed");
}

#[tokio::test]
async fn test_middleware_operation_rejection() {
    let middleware = TestMiddleware::new("reject-middleware").with_rejection();
    let security_context = SecurityContext::new("test-user".to_string());
    let context = ExecutionContext::new(security_context);
    let operation = TestOperation::new("test-op", OperationType::Filesystem);

    let result = middleware.before_execution(operation, &context).await;
    assert!(result.is_ok());
    assert!(result.unwrap().is_none()); // Operation should be rejected
}

#[tokio::test]
async fn test_middleware_error_handling() {
    let middleware = TestMiddleware::new("error-middleware");
    let security_context = SecurityContext::new("test-user".to_string());
    let context = ExecutionContext::new(security_context);

    let error = OSError::ExecutionFailed {
        reason: "Test error".to_string(),
    };

    let action = middleware.handle_error(error, &context).await;
    assert!(matches!(action, ErrorAction::LogAndContinue));
}

#[tokio::test]
async fn test_full_execution_pipeline() {
    // Create components
    let executor = TestExecutor::new("pipeline-executor");
    let middleware = TestMiddleware::new("pipeline-middleware").with_transformation();
    let security_context = SecurityContext::new("test-user".to_string());
    let context = ExecutionContext::new(security_context);
    let operation = TestOperation::new("pipeline-op", OperationType::Filesystem);

    // Test full pipeline: middleware -> executor

    // 1. Middleware processing
    let processed = middleware.before_execution(operation, &context).await;
    assert!(processed.is_ok());
    let transformed_op = processed.unwrap().unwrap();
    assert_eq!(transformed_op.operation_id(), "pipeline-op-transformed");

    // 2. Executor validation and execution
    assert!(executor
        .can_execute(&transformed_op, &context)
        .await
        .unwrap());
    assert!(executor
        .validate_operation(&transformed_op, &context)
        .await
        .is_ok());

    let result = executor.execute(transformed_op.clone(), &context).await;
    assert!(result.is_ok());

    let exec_result = result.unwrap();
    assert_eq!(exec_result.exit_code, 0);
    assert!(exec_result
        .output_as_string()
        .unwrap()
        .contains("pipeline-op-transformed"));

    // 3. Cleanup
    assert!(executor.cleanup(&context).await.is_ok());
    assert!(middleware
        .after_execution(&context, &Ok(exec_result))
        .await
        .is_ok());
}

#[tokio::test]
async fn test_error_propagation_through_pipeline() {
    // Test error handling through the pipeline
    let executor = TestExecutor::new("error-executor");
    let middleware = TestMiddleware::new("error-middleware");
    let security_context = SecurityContext::new("test-user".to_string());
    let context = ExecutionContext::new(security_context);
    let operation = TestOperation::new("error-op", OperationType::Filesystem).with_failure();

    // Execute operation that will fail
    let result = executor.execute(operation.clone(), &context).await;
    assert!(result.is_err());

    // Test middleware error handling
    if let Err(error) = result {
        let action = middleware.handle_error(error, &context).await;
        assert!(matches!(action, ErrorAction::LogAndContinue));
    }
}

#[tokio::test]
async fn test_concurrent_execution() {
    // Test concurrent execution of multiple operations
    let executor = Arc::new(TestExecutor::new("concurrent-executor"));
    let security_context = SecurityContext::new("test-user".to_string());
    let context = ExecutionContext::new(security_context);

    let operations = vec![
        TestOperation::new("op1", OperationType::Filesystem),
        TestOperation::new("op2", OperationType::Filesystem),
        TestOperation::new("op3", OperationType::Filesystem),
    ];

    // Execute operations concurrently
    let mut handles = Vec::new();
    for operation in operations {
        let executor = Arc::clone(&executor);
        let context = context.clone();
        let handle = tokio::spawn(async move { executor.execute(operation, &context).await });
        handles.push(handle);
    }

    // Wait for all executions to complete
    let mut results = Vec::new();
    for handle in handles {
        let result = handle.await.unwrap();
        results.push(result);
    }

    // Verify all executions succeeded
    assert_eq!(results.len(), 3);
    for result in results {
        assert!(result.is_ok());
        assert_eq!(result.unwrap().exit_code, 0);
    }
}
