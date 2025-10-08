//! ProcessSpawnOperation executor implementation.

use async_trait::async_trait;
use chrono::Utc;

use super::ProcessExecutor;
use crate::core::context::ExecutionContext;
use crate::core::executor::{ExecutionResult, OSExecutor};
use crate::core::operation::OperationType;
use crate::core::result::{OSError, OSResult};
use crate::operations::ProcessSpawnOperation;

#[async_trait]
impl OSExecutor<ProcessSpawnOperation> for ProcessExecutor {
    fn name(&self) -> &str {
        &self.name
    }

    fn supported_operation_types(&self) -> Vec<OperationType> {
        vec![OperationType::Process]
    }

    async fn execute(
        &self,
        operation: ProcessSpawnOperation,
        context: &ExecutionContext,
    ) -> OSResult<ExecutionResult> {
        let started_at = Utc::now();

        // Build the command
        let mut cmd = tokio::process::Command::new(&operation.command);

        // Add arguments
        if !operation.args.is_empty() {
            cmd.args(&operation.args);
        }

        // Add environment variables
        for (key, value) in &operation.env {
            cmd.env(key, value);
        }

        // Set working directory if provided
        if let Some(working_dir) = &operation.working_dir {
            cmd.current_dir(working_dir);
        }

        // Spawn the process
        let child = cmd.spawn().map_err(|e| {
            OSError::process_error(format!("spawn '{}'", operation.command), e.to_string())
        })?;

        let pid = child
            .id()
            .ok_or_else(|| OSError::process_error("spawn", "Failed to get process ID"))?;

        let completed_at = Utc::now();

        // Create result with PID as output
        let output = format!("{pid}").into_bytes();
        let result = ExecutionResult::success_with_timing(output, started_at, completed_at)
            .with_metadata("command".to_string(), operation.command.clone())
            .with_metadata("pid".to_string(), pid.to_string())
            .with_metadata("args".to_string(), operation.args.join(" "))
            .with_metadata("executor".to_string(), self.name.clone())
            .with_metadata("user".to_string(), context.principal().to_string());

        Ok(result)
    }

    async fn validate_operation(
        &self,
        operation: &ProcessSpawnOperation,
        _context: &ExecutionContext,
    ) -> OSResult<()> {
        // Validate command is not empty
        if operation.command.is_empty() {
            return Err(OSError::execution_failed("Command cannot be empty"));
        }

        // Validate working directory exists if provided
        if let Some(working_dir) = &operation.working_dir {
            let path = std::path::Path::new(working_dir);
            if !path.exists() {
                return Err(OSError::execution_failed(format!(
                    "Working directory does not exist: {working_dir}"
                )));
            }
            if !path.is_dir() {
                return Err(OSError::execution_failed(format!(
                    "Working directory path is not a directory: {working_dir}"
                )));
            }
        }

        Ok(())
    }
}

#[cfg(test)]
#[allow(clippy::unwrap_used)]
mod tests {
    use super::*;
    use crate::core::context::SecurityContext;

    #[allow(clippy::expect_used)]
    #[tokio::test]
    async fn test_spawn_basic_command() {
        let executor = ProcessExecutor::new("test-executor");
        let operation = ProcessSpawnOperation::new("echo").arg("hello");
        let context = ExecutionContext::new(SecurityContext::new("test-user".to_string()));

        let result = executor
            .execute(operation, &context)
            .await
            .expect("Failed to execute spawn operation");

        assert!(result.is_success());
        assert_eq!(result.get_metadata("command").unwrap(), "echo");
        assert!(result.get_metadata("pid").is_some());
        assert_eq!(result.get_metadata("user").unwrap(), "test-user");
    }

    #[allow(clippy::expect_used)]
    #[tokio::test]
    async fn test_spawn_with_args() {
        let executor = ProcessExecutor::new("test-executor");
        let operation = ProcessSpawnOperation::new("echo").arg("hello").arg("world");
        let context = ExecutionContext::new(SecurityContext::new("test-user".to_string()));

        let result = executor
            .execute(operation, &context)
            .await
            .expect("Failed to execute spawn operation");

        assert!(result.is_success());
        assert_eq!(result.get_metadata("args").unwrap(), "hello world");
    }

    #[allow(clippy::expect_used)]
    #[tokio::test]
    async fn test_spawn_with_env() {
        let executor = ProcessExecutor::new("test-executor");
        let operation = ProcessSpawnOperation::new("printenv").env("TEST_VAR", "test_value");
        let context = ExecutionContext::new(SecurityContext::new("test-user".to_string()));

        let result = executor
            .execute(operation, &context)
            .await
            .expect("Failed to execute spawn operation");

        assert!(result.is_success());
        assert_eq!(result.get_metadata("command").unwrap(), "printenv");
    }

    #[allow(clippy::expect_used)]
    #[tokio::test]
    async fn test_spawn_with_working_dir() {
        let executor = ProcessExecutor::new("test-executor");
        let temp_dir = std::env::temp_dir();
        let operation = ProcessSpawnOperation::new("pwd")
            .working_dir(temp_dir.to_str().expect("Failed to get temp dir path"));
        let context = ExecutionContext::new(SecurityContext::new("test-user".to_string()));

        let result = executor
            .execute(operation, &context)
            .await
            .expect("Failed to execute spawn operation");

        assert!(result.is_success());
    }

    #[tokio::test]
    async fn test_validate_empty_command() {
        let executor = ProcessExecutor::new("test-executor");
        let operation = ProcessSpawnOperation::new("");
        let context = ExecutionContext::new(SecurityContext::new("test-user".to_string()));

        let result = executor.validate_operation(&operation, &context).await;
        assert!(result.is_err());
        assert!(result
            .unwrap_err()
            .to_string()
            .contains("Command cannot be empty"));
    }

    #[tokio::test]
    async fn test_validate_nonexistent_working_dir() {
        let executor = ProcessExecutor::new("test-executor");
        let operation =
            ProcessSpawnOperation::new("echo").working_dir("/nonexistent/directory/path");
        let context = ExecutionContext::new(SecurityContext::new("test-user".to_string()));

        let result = executor.validate_operation(&operation, &context).await;
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("does not exist"));
    }

    #[allow(clippy::expect_used)]
    #[tokio::test]
    async fn test_validate_file_as_working_dir() {
        let executor = ProcessExecutor::new("test-executor");

        // Create a temp file
        let temp_file = tempfile::NamedTempFile::new().expect("Failed to create temp file");
        let file_path = temp_file
            .path()
            .to_str()
            .expect("Failed to get temp file path");

        let operation = ProcessSpawnOperation::new("echo").working_dir(file_path);
        let context = ExecutionContext::new(SecurityContext::new("test-user".to_string()));

        let result = executor.validate_operation(&operation, &context).await;
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("not a directory"));
    }

    #[tokio::test]
    async fn test_spawn_invalid_command() {
        let executor = ProcessExecutor::new("test-executor");
        let operation = ProcessSpawnOperation::new("nonexistent_command_12345");
        let context = ExecutionContext::new(SecurityContext::new("test-user".to_string()));

        let result = executor.execute(operation, &context).await;
        assert!(result.is_err());
        // Error message contains either "Failed to spawn" or the process error
        let err_msg = result.unwrap_err().to_string();
        assert!(err_msg.contains("spawn") || err_msg.contains("Process operation failed"));
    }

    #[allow(clippy::expect_used)]
    #[tokio::test]
    async fn test_spawn_metadata_complete() {
        let executor = ProcessExecutor::new("test-executor");
        let operation = ProcessSpawnOperation::new("echo")
            .arg("test")
            .env("VAR", "value");
        let context = ExecutionContext::new(SecurityContext::new("admin".to_string()));

        let result = executor
            .execute(operation, &context)
            .await
            .expect("Failed to execute spawn operation");

        assert!(result.get_metadata("command").is_some());
        assert!(result.get_metadata("pid").is_some());
        assert!(result.get_metadata("args").is_some());
        assert!(result.get_metadata("executor").is_some());
        assert!(result.get_metadata("user").is_some());
        assert_eq!(result.get_metadata("executor").unwrap(), "test-executor");
        assert_eq!(result.get_metadata("user").unwrap(), "admin");
    }
}
