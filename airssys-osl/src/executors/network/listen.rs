//! NetworkListenOperation executor implementation.

use async_trait::async_trait;
use chrono::Utc;

use super::NetworkExecutor;
use crate::core::context::ExecutionContext;
use crate::core::executor::{ExecutionResult, OSExecutor};
use crate::core::operation::OperationType;
use crate::core::result::{OSError, OSResult};
use crate::operations::NetworkListenOperation;

#[async_trait]
impl OSExecutor<NetworkListenOperation> for NetworkExecutor {
    fn name(&self) -> &str {
        &self.name
    }

    fn supported_operation_types(&self) -> Vec<OperationType> {
        vec![OperationType::Network]
    }

    async fn execute(
        &self,
        operation: NetworkListenOperation,
        context: &ExecutionContext,
    ) -> OSResult<ExecutionResult> {
        let started_at = Utc::now();

        // Handle Unix domain socket if socket_path is provided
        #[cfg(unix)]
        if let Some(socket_path) = &operation.socket_path {
            use tokio::net::UnixListener;

            // Remove existing socket file if it exists
            if std::path::Path::new(socket_path).exists() {
                std::fs::remove_file(socket_path).map_err(|e| {
                    OSError::network_error(
                        format!("remove existing socket {socket_path}"),
                        e.to_string(),
                    )
                })?;
            }

            let listener = UnixListener::bind(socket_path).map_err(|e| {
                OSError::network_error(format!("bind to {socket_path}"), e.to_string())
            })?;

            let completed_at = Utc::now();
            let local_addr = listener.local_addr().ok().and_then(|a| {
                a.as_pathname()
                    .and_then(|p| p.to_str())
                    .map(|s| s.to_string())
            });

            let output = format!("Listening on Unix socket: {socket_path}").into_bytes();
            let mut result = ExecutionResult::success_with_timing(output, started_at, completed_at)
                .with_metadata("socket_path".to_string(), socket_path.clone())
                .with_metadata("socket_type".to_string(), "unix".to_string())
                .with_metadata("executor".to_string(), self.name.clone())
                .with_metadata("user".to_string(), context.principal().to_string());

            if let Some(backlog) = operation.backlog {
                result = result.with_metadata("backlog".to_string(), backlog.to_string());
            }
            if let Some(addr) = local_addr {
                result = result.with_metadata("local_address".to_string(), addr);
            }

            return Ok(result);
        }

        // TCP listener
        let listener = tokio::net::TcpListener::bind(&operation.address)
            .await
            .map_err(|e| {
                OSError::network_error(format!("bind to {}", operation.address), e.to_string())
            })?;

        let completed_at = Utc::now();
        let local_addr = listener
            .local_addr()
            .map(|a| a.to_string())
            .unwrap_or_else(|_| operation.address.clone());

        let output = format!("Listening on {local_addr}").into_bytes();
        let mut result = ExecutionResult::success_with_timing(output, started_at, completed_at)
            .with_metadata("address".to_string(), operation.address.clone())
            .with_metadata("local_address".to_string(), local_addr)
            .with_metadata("socket_type".to_string(), "tcp".to_string())
            .with_metadata("executor".to_string(), self.name.clone())
            .with_metadata("user".to_string(), context.principal().to_string());

        if let Some(backlog) = operation.backlog {
            result = result.with_metadata("backlog".to_string(), backlog.to_string());
        }

        Ok(result)
    }

    async fn validate_operation(
        &self,
        operation: &NetworkListenOperation,
        _context: &ExecutionContext,
    ) -> OSResult<()> {
        // For Unix sockets, socket_path is required
        #[cfg(unix)]
        if let Some(socket_path) = &operation.socket_path {
            if socket_path.is_empty() {
                return Err(OSError::execution_failed("Socket path cannot be empty"));
            }
            // Validate parent directory exists
            if let Some(parent) = std::path::Path::new(socket_path).parent() {
                if !parent.exists() {
                    return Err(OSError::execution_failed(format!(
                        "Socket parent directory does not exist: {}",
                        parent.display()
                    )));
                }
            }
            return Ok(());
        }

        // For TCP, validate address is not empty
        if operation.address.is_empty() {
            return Err(OSError::execution_failed("Address cannot be empty"));
        }

        // Validate address format for TCP
        if !operation.address.contains(':') {
            return Err(OSError::execution_failed(format!(
                "Invalid address format: {} (expected host:port)",
                operation.address
            )));
        }

        // Validate backlog if provided
        if let Some(backlog) = operation.backlog {
            if backlog < 1 {
                return Err(OSError::execution_failed(format!(
                    "Invalid backlog: {backlog} (must be >= 1)"
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
    async fn test_listen_tcp_basic() {
        let executor = NetworkExecutor::new("test-executor");
        let operation = NetworkListenOperation::new("127.0.0.1:0");
        let context = ExecutionContext::new(SecurityContext::new("test-user".to_string()));

        let result = executor
            .execute(operation, &context)
            .await
            .expect("Failed to execute listen operation");

        assert!(result.is_success());
        assert!(result.get_metadata("address").is_some());
        assert!(result.get_metadata("local_address").is_some());
        assert_eq!(result.get_metadata("socket_type").unwrap(), "tcp");
        assert_eq!(result.get_metadata("user").unwrap(), "test-user");
    }

    #[allow(clippy::expect_used)]
    #[tokio::test]
    async fn test_listen_with_backlog() {
        let executor = NetworkExecutor::new("test-executor");
        let operation = NetworkListenOperation::new("127.0.0.1:0").with_backlog(128);
        let context = ExecutionContext::new(SecurityContext::new("test-user".to_string()));

        let result = executor
            .execute(operation, &context)
            .await
            .expect("Failed to execute listen operation");

        assert!(result.is_success());
        assert_eq!(result.get_metadata("backlog").unwrap(), "128");
    }

    #[cfg(unix)]
    #[allow(clippy::expect_used)]
    #[tokio::test]
    async fn test_listen_unix_socket() {
        let executor = NetworkExecutor::new("test-executor");
        let temp_dir = std::env::temp_dir();
        let socket_path = temp_dir
            .join("test-socket.sock")
            .to_str()
            .expect("Failed to get socket path")
            .to_string();

        // Clean up socket if it exists
        let _ = std::fs::remove_file(&socket_path);

        let operation = NetworkListenOperation::new("unix-listener")
            .with_socket_path(&socket_path)
            .with_backlog(64);
        let context = ExecutionContext::new(SecurityContext::new("test-user".to_string()));

        let result = executor
            .execute(operation, &context)
            .await
            .expect("Failed to execute listen operation");

        assert!(result.is_success());
        assert_eq!(result.get_metadata("socket_path").unwrap(), &socket_path);
        assert_eq!(result.get_metadata("socket_type").unwrap(), "unix");
        assert_eq!(result.get_metadata("backlog").unwrap(), "64");

        // Verify socket file was created
        assert!(std::path::Path::new(&socket_path).exists());

        // Clean up
        let _ = std::fs::remove_file(&socket_path);
    }

    #[cfg(unix)]
    #[allow(clippy::expect_used)]
    #[tokio::test]
    async fn test_listen_unix_socket_replaces_existing() {
        let executor = NetworkExecutor::new("test-executor");
        let temp_dir = std::env::temp_dir();
        let socket_path = temp_dir
            .join("test-replace-socket.sock")
            .to_str()
            .expect("Failed to get socket path")
            .to_string();

        // Create existing socket
        let _ = std::fs::remove_file(&socket_path);
        let _first =
            tokio::net::UnixListener::bind(&socket_path).expect("Failed to create first socket");
        drop(_first);

        let operation = NetworkListenOperation::new("unix-listener").with_socket_path(&socket_path);
        let context = ExecutionContext::new(SecurityContext::new("test-user".to_string()));

        let result = executor
            .execute(operation, &context)
            .await
            .expect("Failed to execute listen operation");

        assert!(result.is_success());

        // Clean up
        let _ = std::fs::remove_file(&socket_path);
    }

    #[tokio::test]
    async fn test_listen_invalid_address() {
        let executor = NetworkExecutor::new("test-executor");
        // Port out of range
        let operation = NetworkListenOperation::new("127.0.0.1:99999");
        let context = ExecutionContext::new(SecurityContext::new("test-user".to_string()));

        let result = executor.execute(operation, &context).await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_validate_empty_address() {
        let executor = NetworkExecutor::new("test-executor");
        let operation = NetworkListenOperation::new("");
        let context = ExecutionContext::new(SecurityContext::new("test-user".to_string()));

        let result = executor.validate_operation(&operation, &context).await;
        assert!(result.is_err());
        assert!(result
            .unwrap_err()
            .to_string()
            .contains("Address cannot be empty"));
    }

    #[tokio::test]
    async fn test_validate_invalid_format() {
        let executor = NetworkExecutor::new("test-executor");
        let operation = NetworkListenOperation::new("invalid-no-port");
        let context = ExecutionContext::new(SecurityContext::new("test-user".to_string()));

        let result = executor.validate_operation(&operation, &context).await;
        assert!(result.is_err());
        assert!(result
            .unwrap_err()
            .to_string()
            .contains("Invalid address format"));
    }

    #[tokio::test]
    async fn test_validate_invalid_backlog() {
        let executor = NetworkExecutor::new("test-executor");
        let operation = NetworkListenOperation::new("127.0.0.1:8080").with_backlog(0);
        let context = ExecutionContext::new(SecurityContext::new("test-user".to_string()));

        let result = executor.validate_operation(&operation, &context).await;
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("Invalid backlog"));
    }

    #[cfg(unix)]
    #[tokio::test]
    async fn test_validate_unix_socket_empty_path() {
        let executor = NetworkExecutor::new("test-executor");
        let operation = NetworkListenOperation::new("unix-listener").with_socket_path("");
        let context = ExecutionContext::new(SecurityContext::new("test-user".to_string()));

        let result = executor.validate_operation(&operation, &context).await;
        assert!(result.is_err());
        assert!(result
            .unwrap_err()
            .to_string()
            .contains("Socket path cannot be empty"));
    }

    #[cfg(unix)]
    #[tokio::test]
    async fn test_validate_unix_socket_invalid_parent() {
        let executor = NetworkExecutor::new("test-executor");
        let operation = NetworkListenOperation::new("unix-listener")
            .with_socket_path("/nonexistent/directory/socket.sock");
        let context = ExecutionContext::new(SecurityContext::new("test-user".to_string()));

        let result = executor.validate_operation(&operation, &context).await;
        assert!(result.is_err());
        assert!(result
            .unwrap_err()
            .to_string()
            .contains("parent directory does not exist"));
    }

    #[allow(clippy::expect_used)]
    #[tokio::test]
    async fn test_listen_metadata_completeness() {
        let executor = NetworkExecutor::new("test-executor");
        let operation = NetworkListenOperation::new("127.0.0.1:0").with_backlog(256);
        let context = ExecutionContext::new(SecurityContext::new("admin".to_string()));

        let result = executor
            .execute(operation, &context)
            .await
            .expect("Failed to execute listen operation");

        assert!(result.get_metadata("address").is_some());
        assert!(result.get_metadata("local_address").is_some());
        assert!(result.get_metadata("socket_type").is_some());
        assert!(result.get_metadata("backlog").is_some());
        assert!(result.get_metadata("executor").is_some());
        assert!(result.get_metadata("user").is_some());
        assert_eq!(result.get_metadata("executor").unwrap(), "test-executor");
        assert_eq!(result.get_metadata("user").unwrap(), "admin");
    }
}
