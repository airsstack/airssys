//! Integration tests for #[executor] macro configuration features
//!
//! These tests verify that the #[executor] macro correctly handles
//! custom configuration attributes (name, operations).

#![cfg(feature = "macros")]

use airssys_osl::core::executor::OSExecutor;
use airssys_osl::prelude::*;

// =============================================================================
// Test 1: Custom Executor Name
// =============================================================================

#[derive(Debug)]
struct MyCustomExecutor;

#[executor(name = "AwesomeExecutor")]
impl MyCustomExecutor {
    async fn file_read(
        &self,
        operation: FileReadOperation,
        context: &ExecutionContext,
    ) -> OSResult<ExecutionResult> {
        let _ = (operation, context);
        Ok(ExecutionResult::success(b"custom executor".to_vec()))
    }
}

#[tokio::test]
async fn test_custom_executor_name() {
    let executor = MyCustomExecutor;

    // Verify custom name is used instead of type name
    assert_eq!(executor.name(), "AwesomeExecutor");
    assert_ne!(executor.name(), "MyCustomExecutor");
}

// =============================================================================
// Test 2: Custom Operation Types (Single)
// =============================================================================

#[derive(Debug)]
struct SingleOpExecutor;

#[executor(operations = [Filesystem])]
impl SingleOpExecutor {
    async fn file_read(
        &self,
        operation: FileReadOperation,
        context: &ExecutionContext,
    ) -> OSResult<ExecutionResult> {
        let _ = (operation, context);
        Ok(ExecutionResult::success(b"single".to_vec()))
    }
}

#[tokio::test]
async fn test_custom_single_operation_type() {
    let executor = SingleOpExecutor;

    // Verify only Filesystem is in supported operations
    assert_eq!(
        executor.supported_operation_types(),
        vec![OperationType::Filesystem]
    );
}

// =============================================================================
// Test 3: Custom Operation Types (Multiple)
// =============================================================================

#[derive(Debug)]
struct MultiOpExecutor;

#[executor(operations = [Filesystem, Network, Process])]
impl MultiOpExecutor {
    async fn file_read(
        &self,
        operation: FileReadOperation,
        context: &ExecutionContext,
    ) -> OSResult<ExecutionResult> {
        let _ = (operation, context);
        Ok(ExecutionResult::success(b"multi".to_vec()))
    }
}

#[tokio::test]
async fn test_custom_multiple_operation_types() {
    let executor = MultiOpExecutor;

    // Verify all three operation types are included
    let supported = executor.supported_operation_types();
    assert_eq!(supported.len(), 3);
    assert!(supported.contains(&OperationType::Filesystem));
    assert!(supported.contains(&OperationType::Network));
    assert!(supported.contains(&OperationType::Process));
}

// =============================================================================
// Test 4: Both Name and Operations
// =============================================================================

#[derive(Debug)]
struct FullConfigExecutor;

#[executor(name = "SuperExecutor", operations = [Process, Network])]
impl FullConfigExecutor {
    async fn process_spawn(
        &self,
        operation: ProcessSpawnOperation,
        context: &ExecutionContext,
    ) -> OSResult<ExecutionResult> {
        let _ = (operation, context);
        Ok(ExecutionResult::success(b"full config".to_vec()))
    }
}

#[tokio::test]
async fn test_both_name_and_operations() {
    let executor = FullConfigExecutor;

    // Verify custom name
    assert_eq!(executor.name(), "SuperExecutor");

    // Verify custom operation types
    let supported = executor.supported_operation_types();
    assert_eq!(supported.len(), 2);
    assert!(supported.contains(&OperationType::Process));
    assert!(supported.contains(&OperationType::Network));
    assert!(!supported.contains(&OperationType::Filesystem));
}

// =============================================================================
// Test 5: Auto-Detection Still Works (No Config)
// =============================================================================

#[derive(Debug)]
struct AutoDetectExecutor;

#[executor]
impl AutoDetectExecutor {
    async fn file_read(
        &self,
        operation: FileReadOperation,
        context: &ExecutionContext,
    ) -> OSResult<ExecutionResult> {
        let _ = (operation, context);
        Ok(ExecutionResult::success(b"auto".to_vec()))
    }

    async fn process_spawn(
        &self,
        operation: ProcessSpawnOperation,
        context: &ExecutionContext,
    ) -> OSResult<ExecutionResult> {
        let _ = (operation, context);
        Ok(ExecutionResult::success(b"auto".to_vec()))
    }
}

#[tokio::test]
async fn test_auto_detection_without_config() {
    let executor = AutoDetectExecutor;

    // Verify auto-detected name (from type)
    // Use explicit trait syntax since there are multiple implementations
    assert_eq!(
        <AutoDetectExecutor as OSExecutor<FileReadOperation>>::name(&executor),
        "AutoDetectExecutor"
    );

    // Verify auto-detected operation types (from methods)
    let supported =
        <AutoDetectExecutor as OSExecutor<FileReadOperation>>::supported_operation_types(&executor);
    assert_eq!(supported.len(), 2);
    assert!(supported.contains(&OperationType::Filesystem));
    assert!(supported.contains(&OperationType::Process));
}

// =============================================================================
// Test 6: Execute Methods Still Work With Config
// =============================================================================

#[tokio::test]
async fn test_execute_with_custom_config() {
    let executor = FullConfigExecutor;
    let operation = ProcessSpawnOperation::new("echo".to_string());
    let context = ExecutionContext::new(SecurityContext::new("test-user".to_string()));

    // Verify execute method works correctly
    let result = executor.execute(operation, &context).await;
    assert!(result.is_ok(), "Execution should succeed");

    if let Ok(exec_result) = result {
        assert_eq!(exec_result.output, b"full config");
    }
}
