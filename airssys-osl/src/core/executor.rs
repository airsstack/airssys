//! OS Executor trait definitions.
//!
//! This module defines the core `OSExecutor` trait that handles the actual
//! execution of operations within the OS Layer Framework.

use std::collections::HashMap;
use std::fmt::Debug;
use std::time::Duration;

use async_trait::async_trait;
use chrono::{DateTime, Utc};

use crate::core::context::ExecutionContext;
use crate::core::operation::Operation;
use crate::core::result::OSResult;

/// Result of executing an operation.
///
/// Contains the output and metadata from a successful operation execution.
#[derive(Debug, Clone)]
pub struct ExecutionResult {
    /// The output data from the operation execution
    pub output: Vec<u8>,

    /// Exit code or status code from the operation
    pub exit_code: i32,

    /// When the execution started
    pub started_at: DateTime<Utc>,

    /// When the execution completed
    pub completed_at: DateTime<Utc>,

    /// Duration of the execution
    pub duration: Duration,

    /// Additional metadata from the execution
    pub metadata: HashMap<String, String>,
}

impl ExecutionResult {
    /// Creates a new execution result with the given output and exit code.
    pub fn new(output: Vec<u8>, exit_code: i32) -> Self {
        let now = Utc::now();
        Self {
            output,
            exit_code,
            started_at: now,
            completed_at: now,
            duration: Duration::from_millis(0),
            metadata: HashMap::new(),
        }
    }

    /// Creates a new execution result with timing information.
    pub fn with_timing(
        output: Vec<u8>,
        exit_code: i32,
        started_at: DateTime<Utc>,
        completed_at: DateTime<Utc>,
    ) -> Self {
        let duration = completed_at
            .signed_duration_since(started_at)
            .to_std()
            .unwrap_or_else(|_| Duration::from_millis(0));

        Self {
            output,
            exit_code,
            started_at,
            completed_at,
            duration,
            metadata: HashMap::new(),
        }
    }

    /// Creates a successful execution result with output.
    pub fn success(output: Vec<u8>) -> Self {
        Self::new(output, 0)
    }

    /// Creates a successful execution result with timing.
    pub fn success_with_timing(
        output: Vec<u8>,
        started_at: DateTime<Utc>,
        completed_at: DateTime<Utc>,
    ) -> Self {
        Self::with_timing(output, 0, started_at, completed_at)
    }

    /// Creates a failed execution result with error output.
    pub fn failure(output: Vec<u8>, exit_code: i32) -> Self {
        Self::new(output, exit_code)
    }

    /// Creates a failed execution result with timing.
    pub fn failure_with_timing(
        output: Vec<u8>,
        exit_code: i32,
        started_at: DateTime<Utc>,
        completed_at: DateTime<Utc>,
    ) -> Self {
        Self::with_timing(output, exit_code, started_at, completed_at)
    }

    /// Adds metadata to this execution result.
    pub fn with_metadata(mut self, key: String, value: String) -> Self {
        self.metadata.insert(key, value);
        self
    }

    /// Adds multiple metadata entries to this execution result.
    pub fn with_metadata_map(mut self, metadata: HashMap<String, String>) -> Self {
        self.metadata.extend(metadata);
        self
    }

    /// Gets metadata value by key.
    pub fn get_metadata(&self, key: &str) -> Option<&str> {
        self.metadata.get(key).map(|s| s.as_str())
    }

    /// Returns true if the execution was successful (exit code 0).
    pub fn is_success(&self) -> bool {
        self.exit_code == 0
    }

    /// Returns true if the execution failed.
    pub fn is_failure(&self) -> bool {
        !self.is_success()
    }

    /// Returns the output as a UTF-8 string if possible.
    pub fn output_as_string(&self) -> Result<String, std::string::FromUtf8Error> {
        String::from_utf8(self.output.clone())
    }

    /// Returns true if the execution took longer than the specified duration.
    pub fn exceeded_duration(&self, max_duration: Duration) -> bool {
        self.duration > max_duration
    }

    /// Returns a summary string for logging.
    pub fn summary(&self) -> String {
        format!(
            "Exit: {}, Duration: {:?}, Output: {} bytes",
            self.exit_code,
            self.duration,
            self.output.len()
        )
    }
}

/// Core trait for executing operations within the OS Layer Framework.
///
/// Implementors of this trait handle the actual execution of operations
/// while maintaining security boundaries and proper error handling.
///
/// # Generic Parameters
///
/// * `O` - The operation type this executor can handle
///
/// # Design Notes
///
/// This trait uses generic constraints instead of `dyn` patterns to maintain
/// compile-time type safety and avoid runtime dispatch overhead.
#[async_trait]
pub trait OSExecutor<O>: Debug + Send + Sync + 'static
where
    O: Operation,
{
    /// Returns the name of this executor for logging and identification.
    fn name(&self) -> &str;

    /// Returns the operation types that this executor can handle.
    fn supported_operation_types(&self) -> Vec<crate::core::operation::OperationType>;

    /// Validates whether this executor can handle the given operation.
    ///
    /// This method performs pre-execution validation without actually
    /// executing the operation. It checks operation compatibility,
    /// required permissions, and basic security constraints.
    async fn can_execute(&self, operation: &O, _context: &ExecutionContext) -> OSResult<bool> {
        // Default implementation checks operation type support
        let op_type = operation.operation_type();
        let supported = self.supported_operation_types();

        if supported.is_empty() || supported.contains(&op_type) {
            Ok(true)
        } else {
            Ok(false)
        }
    }

    /// Executes the given operation within the provided context.
    ///
    /// # Arguments
    ///
    /// * `operation` - The operation to execute
    /// * `context` - The execution context containing security and metadata
    ///
    /// # Returns
    ///
    /// Returns `Ok(ExecutionResult)` on successful execution, or `Err(OSError)`
    /// if the operation fails or is rejected by security policies.
    ///
    /// # Security
    ///
    /// Implementors must ensure that all security policies are enforced
    /// and that the operation is properly authorized before execution.
    async fn execute(&self, operation: O, context: &ExecutionContext) -> OSResult<ExecutionResult>;

    /// Executes the operation with a timeout.
    ///
    /// This method provides a default timeout mechanism for operation execution.
    /// Implementors can override this to provide custom timeout handling.
    async fn execute_with_timeout(
        &self,
        operation: O,
        context: &ExecutionContext,
        timeout: Duration,
    ) -> OSResult<ExecutionResult> {
        // Default implementation uses tokio::time::timeout
        match tokio::time::timeout(timeout, self.execute(operation, context)).await {
            Ok(result) => result,
            Err(_) => Err(crate::core::result::OSError::execution_failed(format!(
                "Operation timed out after {timeout:?}"
            ))),
        }
    }

    /// Validates the operation before execution.
    ///
    /// This method performs comprehensive validation including permission
    /// checks, resource availability, and security policy compliance.
    async fn validate_operation(&self, operation: &O, context: &ExecutionContext) -> OSResult<()> {
        // Check if executor can handle this operation
        if !self.can_execute(operation, context).await? {
            return Err(crate::core::result::OSError::execution_failed(format!(
                "Executor '{}' cannot handle operation type '{}'",
                self.name(),
                operation.operation_type().as_str()
            )));
        }

        // Default implementation performs basic validation
        Ok(())
    }

    /// Performs cleanup after operation execution.
    ///
    /// This method is called after operation execution completes,
    /// regardless of success or failure. It can be used for resource
    /// cleanup, logging, or state management.
    async fn cleanup(&self, _context: &ExecutionContext) -> OSResult<()> {
        // Default implementation does nothing
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashMap;

    #[test]
    fn test_execution_result_creation() {
        let result = ExecutionResult::new(b"test output".to_vec(), 0);
        assert!(result.is_success());
        assert!(!result.is_failure());
        assert_eq!(result.output, b"test output");
        assert_eq!(result.exit_code, 0);
        assert!(result.duration.as_millis() == 0);
    }

    #[test]
    fn test_execution_result_success_failure() {
        let success = ExecutionResult::success(b"success".to_vec());
        assert!(success.is_success());
        assert!(!success.is_failure());

        let failure = ExecutionResult::failure(b"error".to_vec(), 1);
        assert!(!failure.is_success());
        assert!(failure.is_failure());
        assert_eq!(failure.exit_code, 1);
    }

    #[test]
    fn test_execution_result_with_timing() {
        let start = Utc::now();
        let end = start + chrono::Duration::milliseconds(100);

        let result = ExecutionResult::with_timing(b"timed output".to_vec(), 0, start, end);

        assert!(result.duration.as_millis() >= 100);
        assert_eq!(result.started_at, start);
        assert_eq!(result.completed_at, end);
    }

    #[test]
    fn test_execution_result_metadata() {
        let result = ExecutionResult::success(b"test".to_vec())
            .with_metadata("key1".to_string(), "value1".to_string())
            .with_metadata("key2".to_string(), "value2".to_string());

        assert_eq!(result.get_metadata("key1"), Some("value1"));
        assert_eq!(result.get_metadata("key2"), Some("value2"));
        assert_eq!(result.get_metadata("nonexistent"), None);
    }

    #[test]
    fn test_execution_result_metadata_map() {
        let mut metadata = HashMap::new();
        metadata.insert("batch_key1".to_string(), "batch_value1".to_string());
        metadata.insert("batch_key2".to_string(), "batch_value2".to_string());

        let result = ExecutionResult::success(b"test".to_vec()).with_metadata_map(metadata);

        assert_eq!(result.get_metadata("batch_key1"), Some("batch_value1"));
        assert_eq!(result.get_metadata("batch_key2"), Some("batch_value2"));
    }

    #[test]
    fn test_execution_result_output_as_string() {
        let result = ExecutionResult::success("hello world".as_bytes().to_vec());
        assert_eq!(
            result.output_as_string().expect("Valid UTF-8"),
            "hello world"
        );

        // Test with invalid UTF-8
        let invalid_result = ExecutionResult::success(vec![0xff, 0xfe]);
        assert!(invalid_result.output_as_string().is_err());
    }

    #[test]
    fn test_execution_result_duration_checks() {
        let start = Utc::now();
        let end = start + chrono::Duration::milliseconds(200);

        let result = ExecutionResult::with_timing(b"test".to_vec(), 0, start, end);

        assert!(result.exceeded_duration(Duration::from_millis(100)));
        assert!(!result.exceeded_duration(Duration::from_millis(300)));
    }

    #[test]
    fn test_execution_result_summary() {
        let result = ExecutionResult::failure(b"error output".to_vec(), 1);
        let summary = result.summary();

        assert!(summary.contains("Exit: 1"));
        assert!(summary.contains("Output: 12 bytes"));
        assert!(summary.contains("Duration:"));
    }
}
