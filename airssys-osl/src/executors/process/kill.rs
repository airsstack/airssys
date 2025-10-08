//! ProcessKillOperation executor implementation.

use async_trait::async_trait;
use chrono::Utc;

use super::ProcessExecutor;
use crate::core::context::ExecutionContext;
use crate::core::executor::{ExecutionResult, OSExecutor};
use crate::core::operation::OperationType;
use crate::core::result::{OSError, OSResult};
use crate::operations::ProcessKillOperation;

#[async_trait]
impl OSExecutor<ProcessKillOperation> for ProcessExecutor {
    fn name(&self) -> &str {
        &self.name
    }

    fn supported_operation_types(&self) -> Vec<OperationType> {
        vec![OperationType::Process]
    }

    async fn execute(
        &self,
        operation: ProcessKillOperation,
        context: &ExecutionContext,
    ) -> OSResult<ExecutionResult> {
        let started_at = Utc::now();

        // Kill the process using platform-specific method
        #[cfg(unix)]
        {
            use nix::sys::signal::{kill, Signal};
            use nix::unistd::Pid;

            let pid = Pid::from_raw(operation.pid as i32);
            kill(pid, Signal::SIGKILL).map_err(|e| {
                OSError::process_error(format!("kill {}", operation.pid), e.to_string())
            })?;
        }

        #[cfg(windows)]
        {
            // Windows implementation using taskkill
            let output = tokio::process::Command::new("taskkill")
                .args(["/F", "/PID", &operation.pid.to_string()])
                .output()
                .await
                .map_err(|e| {
                    OSError::process_error(format!("kill {}", operation.pid), e.to_string())
                })?;

            if !output.status.success() {
                let stderr = String::from_utf8_lossy(&output.stderr);
                return Err(OSError::process_error(
                    format!("kill {}", operation.pid),
                    stderr.to_string(),
                ));
            }
        }

        let completed_at = Utc::now();

        // Create result
        let result = ExecutionResult::success_with_timing(Vec::new(), started_at, completed_at)
            .with_metadata("pid".to_string(), operation.pid.to_string())
            .with_metadata("signal".to_string(), "SIGKILL".to_string())
            .with_metadata("executor".to_string(), self.name.clone())
            .with_metadata("user".to_string(), context.principal().to_string());

        Ok(result)
    }

    async fn validate_operation(
        &self,
        operation: &ProcessKillOperation,
        _context: &ExecutionContext,
    ) -> OSResult<()> {
        // Validate PID is not 0
        if operation.pid == 0 {
            return Err(OSError::execution_failed("Cannot kill process with PID 0"));
        }

        // Validate PID is not 1 (init) on Unix
        #[cfg(unix)]
        if operation.pid == 1 {
            return Err(OSError::execution_failed("Cannot kill init process (PID 1)"));
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
    async fn test_kill_spawned_process() {
        let executor = ProcessExecutor::new("test-executor");
        let context = ExecutionContext::new(SecurityContext::new("test-user".to_string()));

        // Spawn a long-running process
        let mut child = tokio::process::Command::new("sleep")
            .arg("10")
            .spawn()
            .expect("Failed to spawn sleep process");

        let pid = child.id().expect("Failed to get PID");
        let operation = ProcessKillOperation::new(pid);

        let result = executor
            .execute(operation, &context)
            .await
            .expect("Failed to execute kill operation");

        assert!(result.is_success());
        assert_eq!(result.get_metadata("pid").unwrap(), &pid.to_string());
        assert_eq!(result.get_metadata("signal").unwrap(), "SIGKILL");
        assert_eq!(result.get_metadata("user").unwrap(), "test-user");

        // Verify process is killed
        let wait_result = tokio::time::timeout(
            std::time::Duration::from_secs(1),
            child.wait()
        ).await;
        assert!(wait_result.is_ok(), "Process should have been killed");
    }

    #[tokio::test]
    async fn test_validate_pid_zero() {
        let executor = ProcessExecutor::new("test-executor");
        let operation = ProcessKillOperation::new(0);
        let context = ExecutionContext::new(SecurityContext::new("test-user".to_string()));

        let result = executor.validate_operation(&operation, &context).await;
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("Cannot kill process with PID 0"));
    }

    #[cfg(unix)]
    #[tokio::test]
    async fn test_validate_pid_one() {
        let executor = ProcessExecutor::new("test-executor");
        let operation = ProcessKillOperation::new(1);
        let context = ExecutionContext::new(SecurityContext::new("test-user".to_string()));

        let result = executor.validate_operation(&operation, &context).await;
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("Cannot kill init process"));
    }

    #[tokio::test]
    async fn test_kill_nonexistent_process() {
        let executor = ProcessExecutor::new("test-executor");
        let operation = ProcessKillOperation::new(999999);
        let context = ExecutionContext::new(SecurityContext::new("test-user".to_string()));

        let result = executor.execute(operation, &context).await;
        assert!(result.is_err());
    }

    #[allow(clippy::expect_used)]
    #[tokio::test]
    async fn test_kill_metadata() {
        let executor = ProcessExecutor::new("test-executor");
        let context = ExecutionContext::new(SecurityContext::new("admin".to_string()));

        // Spawn a process to kill
        let mut child = tokio::process::Command::new("sleep")
            .arg("10")
            .spawn()
            .expect("Failed to spawn sleep process");

        let pid = child.id().expect("Failed to get PID");
        let operation = ProcessKillOperation::new(pid);

        let result = executor
            .execute(operation, &context)
            .await
            .expect("Failed to execute kill operation");

        assert!(result.get_metadata("pid").is_some());
        assert!(result.get_metadata("signal").is_some());
        assert!(result.get_metadata("executor").is_some());
        assert!(result.get_metadata("user").is_some());
        assert_eq!(result.get_metadata("executor").unwrap(), "test-executor");
        assert_eq!(result.get_metadata("user").unwrap(), "admin");

        // Clean up
        let _ = child.wait().await;
    }
}
