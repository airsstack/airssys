//! OS Executor trait definitions.
//!
//! This module defines the core `OSExecutor` trait that handles the actual
//! execution of operations within the OS Layer Framework.

use std::fmt::Debug;

use async_trait::async_trait;

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
    
    /// Additional metadata from the execution
    pub metadata: std::collections::HashMap<String, String>,
}

impl ExecutionResult {
    /// Creates a new execution result with the given output and exit code.
    pub fn new(output: Vec<u8>, exit_code: i32) -> Self {
        Self {
            output,
            exit_code,
            metadata: std::collections::HashMap::new(),
        }
    }
    
    /// Creates a successful execution result with output.
    pub fn success(output: Vec<u8>) -> Self {
        Self::new(output, 0)
    }
    
    /// Creates a failed execution result with error output.
    pub fn failure(output: Vec<u8>, exit_code: i32) -> Self {
        Self::new(output, exit_code)
    }
    
    /// Adds metadata to this execution result.
    pub fn with_metadata(mut self, key: String, value: String) -> Self {
        self.metadata.insert(key, value);
        self
    }
    
    /// Returns true if the execution was successful (exit code 0).
    pub fn is_success(&self) -> bool {
        self.exit_code == 0
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
}