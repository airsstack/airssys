//! ProcessSignalOperation executor implementation.

use async_trait::async_trait;
use chrono::Utc;

use super::ProcessExecutor;
use crate::core::context::ExecutionContext;
use crate::core::executor::{ExecutionResult, OSExecutor};
use crate::core::operation::OperationType;
use crate::core::result::{OSError, OSResult};
use crate::operations::ProcessSignalOperation;

#[async_trait]
impl OSExecutor<ProcessSignalOperation> for ProcessExecutor {
    fn name(&self) -> &str {
        &self.name
    }

    fn supported_operation_types(&self) -> Vec<OperationType> {
        vec![OperationType::Process]
    }

    async fn execute(
        &self,
        operation: ProcessSignalOperation,
        context: &ExecutionContext,
    ) -> OSResult<ExecutionResult> {
        let started_at = Utc::now();

        // Send signal using platform-specific method
        #[cfg(unix)]
        {
            use nix::sys::signal::Signal;
            use nix::unistd::Pid;

            let pid = Pid::from_raw(operation.pid as i32);
            let signal = Signal::try_from(operation.signal).map_err(|_| {
                OSError::execution_failed(format!("Invalid signal number: {}", operation.signal))
            })?;

            nix::sys::signal::kill(pid, signal).map_err(|e| {
                OSError::process_error(
                    format!("signal {} to {}", operation.signal, operation.pid),
                    e.to_string(),
                )
            })?;
        }

        #[cfg(windows)]
        {
            // Windows signal translation
            match operation.signal {
                2 | 15 => {
                    // SIGTERM/SIGINT -> Normal termination
                    let output = tokio::process::Command::new("taskkill")
                        .args(["/PID", &operation.pid.to_string()])
                        .output()
                        .await
                        .map_err(|e| {
                            OSError::process_error(
                                format!("signal {} to {}", operation.signal, operation.pid),
                                e.to_string(),
                            )
                        })?;

                    if !output.status.success() {
                        let stderr = String::from_utf8_lossy(&output.stderr);
                        return Err(OSError::process_error(
                            format!("signal {} to {}", operation.signal, operation.pid),
                            stderr.to_string(),
                        ));
                    }
                }
                9 => {
                    // SIGKILL -> Force kill
                    let output = tokio::process::Command::new("taskkill")
                        .args(["/F", "/PID", &operation.pid.to_string()])
                        .output()
                        .await
                        .map_err(|e| {
                            OSError::process_error(
                                format!("signal {} to {}", operation.signal, operation.pid),
                                e.to_string(),
                            )
                        })?;

                    if !output.status.success() {
                        let stderr = String::from_utf8_lossy(&output.stderr);
                        return Err(OSError::process_error(
                            format!("signal {} to {}", operation.signal, operation.pid),
                            stderr.to_string(),
                        ));
                    }
                }
                _ => {
                    return Err(OSError::execution_failed(format!(
                        "Signal {} not supported on Windows. Use SIGTERM (15), SIGINT (2), or SIGKILL (9)",
                        operation.signal
                    )));
                }
            }
        }

        let completed_at = Utc::now();
        let signal_name = get_signal_name(operation.signal);

        // Create result
        let result = ExecutionResult::success_with_timing(Vec::new(), started_at, completed_at)
            .with_metadata("pid".to_string(), operation.pid.to_string())
            .with_metadata("signal".to_string(), operation.signal.to_string())
            .with_metadata("signal_name".to_string(), signal_name)
            .with_metadata("executor".to_string(), self.name.clone())
            .with_metadata("user".to_string(), context.principal().to_string());

        Ok(result)
    }

    async fn validate_operation(
        &self,
        operation: &ProcessSignalOperation,
        _context: &ExecutionContext,
    ) -> OSResult<()> {
        // Validate PID is not 0
        if operation.pid == 0 {
            return Err(OSError::execution_failed(
                "Cannot send signal to process with PID 0",
            ));
        }

        // Validate PID is not 1 (init) on Unix
        #[cfg(unix)]
        if operation.pid == 1 {
            return Err(OSError::execution_failed(
                "Cannot send signal to init process (PID 1)",
            ));
        }

        // Validate signal number is reasonable on Unix
        #[cfg(unix)]
        if operation.signal < 1 || operation.signal > 64 {
            return Err(OSError::execution_failed(format!(
                "Signal number must be between 1 and 64, got {}",
                operation.signal
            )));
        }

        // Validate signal is supported on Windows
        #[cfg(windows)]
        if ![2, 9, 15].contains(&operation.signal) {
            return Err(OSError::execution_failed(format!(
                "Signal {} not supported on Windows. Use SIGTERM (15), SIGINT (2), or SIGKILL (9)",
                operation.signal
            )));
        }

        Ok(())
    }
}

/// Get human-readable signal name from signal number.
fn get_signal_name(signal: i32) -> String {
    match signal {
        1 => "SIGHUP".to_string(),
        2 => "SIGINT".to_string(),
        3 => "SIGQUIT".to_string(),
        6 => "SIGABRT".to_string(),
        9 => "SIGKILL".to_string(),
        14 => "SIGALRM".to_string(),
        15 => "SIGTERM".to_string(),
        18 => "SIGCONT".to_string(),
        19 => "SIGSTOP".to_string(),
        20 => "SIGTSTP".to_string(),
        _ => format!("SIGNAL_{signal}"),
    }
}

#[cfg(test)]
#[allow(clippy::unwrap_used)]
mod tests {
    use super::*;
    use crate::core::context::SecurityContext;

    #[allow(clippy::expect_used)]
    #[tokio::test]
    async fn test_signal_term() {
        let executor = ProcessExecutor::new("test-executor");
        let context = ExecutionContext::new(SecurityContext::new("test-user".to_string()));

        // Spawn a long-running process
        let mut child = tokio::process::Command::new("sleep")
            .arg("10")
            .spawn()
            .expect("Failed to spawn sleep process");

        let pid = child.id().expect("Failed to get PID");
        let operation = ProcessSignalOperation::terminate(pid);

        let result = executor
            .execute(operation, &context)
            .await
            .expect("Failed to execute signal operation");

        assert!(result.is_success());
        assert_eq!(result.get_metadata("pid").unwrap(), &pid.to_string());
        assert_eq!(result.get_metadata("signal").unwrap(), "15");
        assert_eq!(result.get_metadata("signal_name").unwrap(), "SIGTERM");

        // Verify process is terminated
        let wait_result =
            tokio::time::timeout(std::time::Duration::from_secs(1), child.wait()).await;
        assert!(wait_result.is_ok(), "Process should have been terminated");
    }

    #[allow(clippy::expect_used)]
    #[tokio::test]
    async fn test_signal_kill() {
        let executor = ProcessExecutor::new("test-executor");
        let context = ExecutionContext::new(SecurityContext::new("test-user".to_string()));

        // Spawn a long-running process
        let mut child = tokio::process::Command::new("sleep")
            .arg("10")
            .spawn()
            .expect("Failed to spawn sleep process");

        let pid = child.id().expect("Failed to get PID");
        let operation = ProcessSignalOperation::kill(pid);

        let result = executor
            .execute(operation, &context)
            .await
            .expect("Failed to execute signal operation");

        assert!(result.is_success());
        assert_eq!(result.get_metadata("signal").unwrap(), "9");
        assert_eq!(result.get_metadata("signal_name").unwrap(), "SIGKILL");

        // Clean up
        let _ = child.wait().await;
    }

    #[tokio::test]
    async fn test_validate_pid_zero() {
        let executor = ProcessExecutor::new("test-executor");
        let operation = ProcessSignalOperation::new(0, 15);
        let context = ExecutionContext::new(SecurityContext::new("test-user".to_string()));

        let result = executor.validate_operation(&operation, &context).await;
        assert!(result.is_err());
        assert!(result
            .unwrap_err()
            .to_string()
            .contains("Cannot send signal to process with PID 0"));
    }

    #[cfg(unix)]
    #[tokio::test]
    async fn test_validate_pid_one() {
        let executor = ProcessExecutor::new("test-executor");
        let operation = ProcessSignalOperation::new(1, 15);
        let context = ExecutionContext::new(SecurityContext::new("test-user".to_string()));

        let result = executor.validate_operation(&operation, &context).await;
        assert!(result.is_err());
        assert!(result
            .unwrap_err()
            .to_string()
            .contains("Cannot send signal to init process"));
    }

    #[cfg(unix)]
    #[tokio::test]
    async fn test_validate_invalid_signal_number() {
        let executor = ProcessExecutor::new("test-executor");
        let context = ExecutionContext::new(SecurityContext::new("test-user".to_string()));

        // Test signal number too low
        let operation = ProcessSignalOperation::new(1000, 0);
        let result = executor.validate_operation(&operation, &context).await;
        assert!(result.is_err());

        // Test signal number too high
        let operation = ProcessSignalOperation::new(1000, 100);
        let result = executor.validate_operation(&operation, &context).await;
        assert!(result.is_err());
    }

    #[cfg(windows)]
    #[tokio::test]
    async fn test_validate_unsupported_signal_windows() {
        let executor = ProcessExecutor::new("test-executor");
        let operation = ProcessSignalOperation::new(1000, 19); // SIGSTOP not supported on Windows
        let context = ExecutionContext::new(SecurityContext::new("test-user".to_string()));

        let result = executor.validate_operation(&operation, &context).await;
        assert!(result.is_err());
        assert!(result
            .unwrap_err()
            .to_string()
            .contains("not supported on Windows"));
    }

    #[tokio::test]
    async fn test_signal_nonexistent_process() {
        let executor = ProcessExecutor::new("test-executor");
        let operation = ProcessSignalOperation::new(999999, 15);
        let context = ExecutionContext::new(SecurityContext::new("test-user".to_string()));

        let result = executor.execute(operation, &context).await;
        assert!(result.is_err());
    }

    #[test]
    fn test_signal_name_mapping() {
        assert_eq!(get_signal_name(1), "SIGHUP");
        assert_eq!(get_signal_name(2), "SIGINT");
        assert_eq!(get_signal_name(9), "SIGKILL");
        assert_eq!(get_signal_name(15), "SIGTERM");
        assert_eq!(get_signal_name(99), "SIGNAL_99");
    }

    #[allow(clippy::expect_used)]
    #[tokio::test]
    async fn test_signal_metadata() {
        let executor = ProcessExecutor::new("test-executor");
        let context = ExecutionContext::new(SecurityContext::new("admin".to_string()));

        // Spawn a process to signal
        let mut child = tokio::process::Command::new("sleep")
            .arg("10")
            .spawn()
            .expect("Failed to spawn sleep process");

        let pid = child.id().expect("Failed to get PID");
        let operation = ProcessSignalOperation::terminate(pid);

        let result = executor
            .execute(operation, &context)
            .await
            .expect("Failed to execute signal operation");

        assert!(result.get_metadata("pid").is_some());
        assert!(result.get_metadata("signal").is_some());
        assert!(result.get_metadata("signal_name").is_some());
        assert!(result.get_metadata("executor").is_some());
        assert!(result.get_metadata("user").is_some());
        assert_eq!(result.get_metadata("executor").unwrap(), "test-executor");
        assert_eq!(result.get_metadata("user").unwrap(), "admin");

        // Clean up
        let _ = child.wait().await;
    }
}
