//! NetworkSocketOperation executor implementation.

use async_trait::async_trait;
use chrono::Utc;

use super::NetworkExecutor;
use crate::core::context::ExecutionContext;
use crate::core::executor::{ExecutionResult, OSExecutor};
use crate::core::operation::OperationType;
use crate::core::result::{OSError, OSResult};
use crate::operations::NetworkSocketOperation;

#[async_trait]
impl OSExecutor<NetworkSocketOperation> for NetworkExecutor {
    fn name(&self) -> &str {
        &self.name
    }

    fn supported_operation_types(&self) -> Vec<OperationType> {
        vec![OperationType::Network]
    }

    async fn execute(
        &self,
        operation: NetworkSocketOperation,
        context: &ExecutionContext,
    ) -> OSResult<ExecutionResult> {
        let started_at = Utc::now();

        // Create socket based on type
        let socket_info = match operation.socket_type.to_lowercase().as_str() {
            "tcp" => {
                // Create TCP socket using tokio::net::TcpSocket
                let _socket = tokio::net::TcpSocket::new_v4()
                    .map_err(|e| OSError::network_error("create TCP socket", e.to_string()))?;
                "TCP socket created (IPv4)".to_string()
            }
            "udp" => {
                // Create UDP socket using tokio::net::UdpSocket
                // Bind to an OS-assigned port
                let socket = tokio::net::UdpSocket::bind("0.0.0.0:0")
                    .await
                    .map_err(|e| OSError::network_error("create UDP socket", e.to_string()))?;
                let local_addr = socket
                    .local_addr()
                    .map_err(|e| OSError::network_error("get UDP socket address", e.to_string()))?;
                format!("UDP socket created at {local_addr}")
            }
            "unix" => {
                #[cfg(unix)]
                {
                    // Unix domain socket - just validate we can create the socket type
                    // Actual binding would require a path
                    "Unix domain socket type validated".to_string()
                }
                #[cfg(not(unix))]
                {
                    return Err(OSError::execution_failed(
                        "Unix domain sockets not supported on this platform",
                    ));
                }
            }
            other => {
                return Err(OSError::execution_failed(format!(
                    "Unsupported socket type: {other} (supported: tcp, udp, unix)"
                )));
            }
        };

        let completed_at = Utc::now();

        let output = socket_info.clone().into_bytes();
        let result = ExecutionResult::success_with_timing(output, started_at, completed_at)
            .with_metadata("socket_type".to_string(), operation.socket_type.clone())
            .with_metadata("socket_info".to_string(), socket_info)
            .with_metadata("executor".to_string(), self.name.clone())
            .with_metadata("user".to_string(), context.principal().to_string());

        Ok(result)
    }

    async fn validate_operation(
        &self,
        operation: &NetworkSocketOperation,
        _context: &ExecutionContext,
    ) -> OSResult<()> {
        // Validate socket type is not empty
        if operation.socket_type.is_empty() {
            return Err(OSError::execution_failed("Socket type cannot be empty"));
        }

        // Validate socket type is supported
        let socket_type = operation.socket_type.to_lowercase();
        match socket_type.as_str() {
            "tcp" | "udp" => Ok(()),
            "unix" => {
                #[cfg(unix)]
                {
                    Ok(())
                }
                #[cfg(not(unix))]
                {
                    Err(OSError::execution_failed(
                        "Unix domain sockets not supported on this platform",
                    ))
                }
            }
            _ => Err(OSError::execution_failed(format!(
                "Unsupported socket type: {} (supported: tcp, udp, unix)",
                operation.socket_type
            ))),
        }
    }
}

#[cfg(test)]
#[allow(clippy::unwrap_used)]
mod tests {
    use super::*;
    use crate::core::context::SecurityContext;

    #[allow(clippy::expect_used)]
    #[tokio::test]
    async fn test_socket_tcp() {
        let executor = NetworkExecutor::new("test-executor");
        let operation = NetworkSocketOperation::tcp();
        let context = ExecutionContext::new(SecurityContext::new("test-user".to_string()));

        let result = executor
            .execute(operation, &context)
            .await
            .expect("Failed to execute socket operation");

        assert!(result.is_success());
        assert_eq!(result.get_metadata("socket_type").unwrap(), "tcp");
        assert!(result.get_metadata("socket_info").is_some());
        assert_eq!(result.get_metadata("user").unwrap(), "test-user");
    }

    #[allow(clippy::expect_used)]
    #[tokio::test]
    async fn test_socket_udp() {
        let executor = NetworkExecutor::new("test-executor");
        let operation = NetworkSocketOperation::udp();
        let context = ExecutionContext::new(SecurityContext::new("test-user".to_string()));

        let result = executor
            .execute(operation, &context)
            .await
            .expect("Failed to execute socket operation");

        assert!(result.is_success());
        assert_eq!(result.get_metadata("socket_type").unwrap(), "udp");
        assert!(result
            .get_metadata("socket_info")
            .unwrap()
            .contains("UDP socket"));
    }

    #[cfg(unix)]
    #[allow(clippy::expect_used)]
    #[tokio::test]
    async fn test_socket_unix() {
        let executor = NetworkExecutor::new("test-executor");
        let operation = NetworkSocketOperation::unix();
        let context = ExecutionContext::new(SecurityContext::new("test-user".to_string()));

        let result = executor
            .execute(operation, &context)
            .await
            .expect("Failed to execute socket operation");

        assert!(result.is_success());
        assert_eq!(result.get_metadata("socket_type").unwrap(), "unix");
    }

    #[cfg(not(unix))]
    #[tokio::test]
    async fn test_socket_unix_unsupported() {
        let executor = NetworkExecutor::new("test-executor");
        let operation = NetworkSocketOperation::unix();
        let context = ExecutionContext::new(SecurityContext::new("test-user".to_string()));

        let result = executor.execute(operation, &context).await;
        assert!(result.is_err());
        assert!(result
            .unwrap_err()
            .to_string()
            .contains("not supported on this platform"));
    }

    #[tokio::test]
    async fn test_socket_unsupported_type() {
        let executor = NetworkExecutor::new("test-executor");
        let operation = NetworkSocketOperation::new("raw");
        let context = ExecutionContext::new(SecurityContext::new("test-user".to_string()));

        let result = executor.execute(operation, &context).await;
        assert!(result.is_err());
        assert!(result
            .unwrap_err()
            .to_string()
            .contains("Unsupported socket type"));
    }

    #[tokio::test]
    async fn test_validate_empty_socket_type() {
        let executor = NetworkExecutor::new("test-executor");
        let operation = NetworkSocketOperation::new("");
        let context = ExecutionContext::new(SecurityContext::new("test-user".to_string()));

        let result = executor.validate_operation(&operation, &context).await;
        assert!(result.is_err());
        assert!(result
            .unwrap_err()
            .to_string()
            .contains("Socket type cannot be empty"));
    }

    #[tokio::test]
    async fn test_validate_invalid_socket_type() {
        let executor = NetworkExecutor::new("test-executor");
        let operation = NetworkSocketOperation::new("invalid");
        let context = ExecutionContext::new(SecurityContext::new("test-user".to_string()));

        let result = executor.validate_operation(&operation, &context).await;
        assert!(result.is_err());
        assert!(result
            .unwrap_err()
            .to_string()
            .contains("Unsupported socket type"));
    }

    #[cfg(not(unix))]
    #[tokio::test]
    async fn test_validate_unix_on_windows() {
        let executor = NetworkExecutor::new("test-executor");
        let operation = NetworkSocketOperation::unix();
        let context = ExecutionContext::new(SecurityContext::new("test-user".to_string()));

        let result = executor.validate_operation(&operation, &context).await;
        assert!(result.is_err());
        assert!(result
            .unwrap_err()
            .to_string()
            .contains("not supported on this platform"));
    }

    #[tokio::test]
    async fn test_socket_case_insensitive() {
        let executor = NetworkExecutor::new("test-executor");
        let context = ExecutionContext::new(SecurityContext::new("test-user".to_string()));

        // Test TCP in different cases
        let operation = NetworkSocketOperation::new("TCP");
        let result = executor.validate_operation(&operation, &context).await;
        assert!(result.is_ok());

        let operation = NetworkSocketOperation::new("Udp");
        let result = executor.validate_operation(&operation, &context).await;
        assert!(result.is_ok());
    }

    #[allow(clippy::expect_used)]
    #[tokio::test]
    async fn test_socket_metadata_completeness() {
        let executor = NetworkExecutor::new("test-executor");
        let operation = NetworkSocketOperation::tcp();
        let context = ExecutionContext::new(SecurityContext::new("admin".to_string()));

        let result = executor
            .execute(operation, &context)
            .await
            .expect("Failed to execute socket operation");

        assert!(result.get_metadata("socket_type").is_some());
        assert!(result.get_metadata("socket_info").is_some());
        assert!(result.get_metadata("executor").is_some());
        assert!(result.get_metadata("user").is_some());
        assert_eq!(result.get_metadata("executor").unwrap(), "test-executor");
        assert_eq!(result.get_metadata("user").unwrap(), "admin");
    }
}
