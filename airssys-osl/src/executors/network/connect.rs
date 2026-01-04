//! NetworkConnectOperation executor implementation.

use async_trait::async_trait;
use chrono::Utc;

use super::NetworkExecutor;
use crate::core::context::ExecutionContext;
use crate::core::executor::{ExecutionResult, OSExecutor};
use crate::core::operation::OperationType;
use crate::core::result::{OSError, OSResult};
use crate::operations::NetworkConnectOperation;

#[async_trait]
impl OSExecutor<NetworkConnectOperation> for NetworkExecutor {
    fn name(&self) -> &str {
        &self.name
    }

    fn supported_operation_types(&self) -> Vec<OperationType> {
        vec![OperationType::Network]
    }

    async fn execute(
        &self,
        operation: NetworkConnectOperation,
        context: &ExecutionContext,
    ) -> OSResult<ExecutionResult> {
        let started_at = Utc::now();

        // Connect to the address with timeout if specified
        let stream = if let Some(timeout) = operation.timeout {
            tokio::time::timeout(timeout, tokio::net::TcpStream::connect(&operation.address))
                .await
                .map_err(|_| {
                    OSError::network_error(
                        format!("connect to {}", operation.address),
                        "Connection timeout",
                    )
                })?
                .map_err(|e| {
                    OSError::network_error(
                        format!("connect to {}", operation.address),
                        e.to_string(),
                    )
                })?
        } else {
            tokio::net::TcpStream::connect(&operation.address)
                .await
                .map_err(|e| {
                    OSError::network_error(
                        format!("connect to {}", operation.address),
                        e.to_string(),
                    )
                })?
        };

        let completed_at = Utc::now();

        // Get local and peer addresses
        let local_addr = stream.local_addr().map(|a| a.to_string()).ok();
        let peer_addr = stream.peer_addr().map(|a| a.to_string()).ok();

        // Create result with connection info
        let output = if let Some(peer) = &peer_addr {
            format!("Connected to {peer}")
        } else {
            format!("Connected to {}", operation.address)
        }
        .into_bytes();

        let mut result = ExecutionResult::success_with_timing(output, started_at, completed_at)
            .with_metadata("address".to_string(), operation.address.clone())
            .with_metadata("executor".to_string(), self.name.clone())
            .with_metadata("user".to_string(), context.principal().to_string());

        if let Some(local) = local_addr {
            result = result.with_metadata("local_address".to_string(), local);
        }
        if let Some(peer) = peer_addr {
            result = result.with_metadata("peer_address".to_string(), peer);
        }
        if let Some(timeout) = operation.timeout {
            result = result.with_metadata("timeout".to_string(), format!("{timeout:?}"));
        }

        Ok(result)
    }

    async fn validate_operation(
        &self,
        operation: &NetworkConnectOperation,
        _context: &ExecutionContext,
    ) -> OSResult<()> {
        // Validate address is not empty
        if operation.address.is_empty() {
            return Err(OSError::execution_failed("Address cannot be empty"));
        }

        // Validate address format (basic check for colon separator)
        if !operation.address.contains(':') {
            return Err(OSError::execution_failed(format!(
                "Invalid address format: {} (expected host:port)",
                operation.address
            )));
        }

        Ok(())
    }
}

#[cfg(test)]
#[allow(clippy::unwrap_used)]
mod tests {
    use super::*;
    use crate::core::context::SecurityContext;
    use std::time::Duration;

    #[allow(clippy::expect_used)]
    #[tokio::test]
    async fn test_connect_basic() {
        // Start a simple TCP listener
        let listener = tokio::net::TcpListener::bind("127.0.0.1:0")
            .await
            .expect("Failed to bind listener");
        let addr = listener.local_addr().expect("Failed to get address");

        let executor = NetworkExecutor::new("test-executor");
        let operation = NetworkConnectOperation::new(addr.to_string());
        let context = ExecutionContext::new(SecurityContext::new("test-user".to_string()));

        let result = executor
            .execute(operation, &context)
            .await
            .expect("Failed to execute connect operation");

        assert!(result.is_success());
        assert_eq!(result.get_metadata("address").unwrap(), &addr.to_string());
        assert_eq!(result.get_metadata("user").unwrap(), "test-user");
        assert!(result.get_metadata("local_address").is_some());
        assert!(result.get_metadata("peer_address").is_some());
    }

    #[allow(clippy::expect_used)]
    #[tokio::test]
    async fn test_connect_with_timeout() {
        let listener = tokio::net::TcpListener::bind("127.0.0.1:0")
            .await
            .expect("Failed to bind listener");
        let addr = listener.local_addr().expect("Failed to get address");

        let executor = NetworkExecutor::new("test-executor");
        let operation =
            NetworkConnectOperation::new(addr.to_string()).with_timeout(Duration::from_secs(5));
        let context = ExecutionContext::new(SecurityContext::new("test-user".to_string()));

        let result = executor
            .execute(operation, &context)
            .await
            .expect("Failed to execute connect operation");

        assert!(result.is_success());
        assert!(result.get_metadata("timeout").is_some());
    }

    #[tokio::test]
    async fn test_connect_timeout_expires() {
        let executor = NetworkExecutor::new("test-executor");
        // Use a non-routable IP to trigger timeout
        let operation = NetworkConnectOperation::new("192.0.2.1:9999".to_string())
            .with_timeout(Duration::from_millis(100));
        let context = ExecutionContext::new(SecurityContext::new("test-user".to_string()));

        let result = executor.execute(operation, &context).await;
        assert!(result.is_err());
        assert!(result
            .unwrap_err()
            .to_string()
            .contains("Connection timeout"));
    }

    #[tokio::test]
    async fn test_connect_invalid_address() {
        let executor = NetworkExecutor::new("test-executor");
        let operation = NetworkConnectOperation::new("invalid-address:99999".to_string());
        let context = ExecutionContext::new(SecurityContext::new("test-user".to_string()));

        let result = executor.execute(operation, &context).await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_validate_empty_address() {
        let executor = NetworkExecutor::new("test-executor");
        let operation = NetworkConnectOperation::new("");
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
        let operation = NetworkConnectOperation::new("invalid-no-port");
        let context = ExecutionContext::new(SecurityContext::new("test-user".to_string()));

        let result = executor.validate_operation(&operation, &context).await;
        assert!(result.is_err());
        assert!(result
            .unwrap_err()
            .to_string()
            .contains("Invalid address format"));
    }

    #[allow(clippy::expect_used)]
    #[tokio::test]
    async fn test_connect_metadata_completeness() {
        let listener = tokio::net::TcpListener::bind("127.0.0.1:0")
            .await
            .expect("Failed to bind listener");
        let addr = listener.local_addr().expect("Failed to get address");

        let executor = NetworkExecutor::new("test-executor");
        let operation =
            NetworkConnectOperation::new(addr.to_string()).with_timeout(Duration::from_secs(10));
        let context = ExecutionContext::new(SecurityContext::new("admin".to_string()));

        let result = executor
            .execute(operation, &context)
            .await
            .expect("Failed to execute connect operation");

        assert!(result.get_metadata("address").is_some());
        assert!(result.get_metadata("local_address").is_some());
        assert!(result.get_metadata("peer_address").is_some());
        assert!(result.get_metadata("timeout").is_some());
        assert!(result.get_metadata("executor").is_some());
        assert!(result.get_metadata("user").is_some());
        assert_eq!(result.get_metadata("executor").unwrap(), "test-executor");
        assert_eq!(result.get_metadata("user").unwrap(), "admin");
    }
}
