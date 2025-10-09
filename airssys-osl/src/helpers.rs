//! High-level convenience functions for common OS operations.
//!
//! # Security and Middleware Integration
//!
//! **Current Implementation:** These helpers use direct executor calls for simplicity.
//!
//! **Future Enhancement (OSL-TASK-003, OSL-TASK-004):**
//! - Security policy validation will be integrated
//! - Middleware pipeline support will be added
//! - APIs will remain backward compatible
//!
//! See: OSL-TASK-003 (Security Middleware), OSL-TASK-004 (Middleware Pipeline)

// Layer 1: Standard library imports
use std::path::Path;

// Layer 2: No third-party imports needed

// Layer 3: Internal module imports
use crate::core::context::{ExecutionContext, SecurityContext};
use crate::core::executor::OSExecutor;
use crate::core::result::OSResult;
use crate::executors::filesystem::FilesystemExecutor;
use crate::executors::network::NetworkExecutor;
use crate::executors::process::ProcessExecutor;
use crate::operations::filesystem::{
    DirectoryCreateOperation, FileDeleteOperation, FileReadOperation, FileWriteOperation,
};
use crate::operations::network::{
    NetworkConnectOperation, NetworkListenOperation, NetworkSocketOperation,
};
use crate::operations::process::{
    ProcessKillOperation, ProcessSignalOperation, ProcessSpawnOperation,
};

// ============================================================================
// Filesystem Helpers (4 functions)
// ============================================================================

/// Read file contents with security context.
///
/// # Current Implementation
/// Direct executor call for simplicity.
///
/// # Future Integration (OSL-TASK-003, OSL-TASK-004)
/// - TODO: Add security policy validation
/// - TODO: Wire through middleware pipeline
/// - TODO: Support optional middleware composition
///
/// # Example
///
/// ```rust,no_run
/// use airssys_osl::helpers::*;
///
/// # async fn example() -> airssys_osl::core::result::OSResult<()> {
/// let data = read_file("/etc/hosts", "admin").await?;
/// println!("Read {} bytes", data.len());
/// # Ok(())
/// # }
/// ```
pub async fn read_file<P: AsRef<Path>>(path: P, user: impl Into<String>) -> OSResult<Vec<u8>> {
    // TODO(OSL-TASK-003): Add security validation here
    // TODO(OSL-TASK-004): Wire through middleware pipeline
    let path_str = path.as_ref().display().to_string();
    let operation = FileReadOperation::new(path_str);
    let context = ExecutionContext::new(SecurityContext::new(user.into()));
    let executor = FilesystemExecutor::new();
    let result = executor.execute(operation, &context).await?;
    Ok(result.output)
}

/// Write data to file with security context.
///
/// # Current Implementation
/// Direct executor call for simplicity.
///
/// # Future Integration (OSL-TASK-003, OSL-TASK-004)
/// - TODO: Add security policy validation
/// - TODO: Wire through middleware pipeline
///
/// # Example
///
/// ```rust,no_run
/// use airssys_osl::helpers::*;
///
/// # async fn example() -> airssys_osl::core::result::OSResult<()> {
/// let data = b"Hello, World!".to_vec();
/// write_file("/tmp/test.txt", data, "admin").await?;
/// # Ok(())
/// # }
/// ```
pub async fn write_file<P: AsRef<Path>>(
    path: P,
    data: Vec<u8>,
    user: impl Into<String>,
) -> OSResult<()> {
    // TODO(OSL-TASK-003): Add security validation
    // TODO(OSL-TASK-004): Wire through middleware pipeline
    let path_str = path.as_ref().display().to_string();
    let operation = FileWriteOperation::new(path_str, data);
    let context = ExecutionContext::new(SecurityContext::new(user.into()));
    let executor = FilesystemExecutor::new();
    executor.execute(operation, &context).await?;
    Ok(())
}

/// Delete file with security context.
///
/// # Current Implementation
/// Direct executor call for simplicity.
///
/// # Future Integration (OSL-TASK-003, OSL-TASK-004)
/// - TODO: Add security policy validation
/// - TODO: Wire through middleware pipeline
///
/// # Example
///
/// ```rust,no_run
/// use airssys_osl::helpers::*;
///
/// # async fn example() -> airssys_osl::core::result::OSResult<()> {
/// delete_file("/tmp/test.txt", "admin").await?;
/// # Ok(())
/// # }
/// ```
pub async fn delete_file<P: AsRef<Path>>(path: P, user: impl Into<String>) -> OSResult<()> {
    // TODO(OSL-TASK-003): Add security validation
    // TODO(OSL-TASK-004): Wire through middleware pipeline
    let path_str = path.as_ref().display().to_string();
    let operation = FileDeleteOperation::new(path_str);
    let context = ExecutionContext::new(SecurityContext::new(user.into()));
    let executor = FilesystemExecutor::new();
    executor.execute(operation, &context).await?;
    Ok(())
}

/// Create directory with security context.
///
/// # Current Implementation
/// Direct executor call for simplicity.
///
/// # Future Integration (OSL-TASK-003, OSL-TASK-004)
/// - TODO: Add security policy validation
/// - TODO: Wire through middleware pipeline
///
/// # Example
///
/// ```rust,no_run
/// use airssys_osl::helpers::*;
///
/// # async fn example() -> airssys_osl::core::result::OSResult<()> {
/// create_directory("/tmp/test_dir", "admin").await?;
/// # Ok(())
/// # }
/// ```
pub async fn create_directory<P: AsRef<Path>>(path: P, user: impl Into<String>) -> OSResult<()> {
    // TODO(OSL-TASK-003): Add security validation
    // TODO(OSL-TASK-004): Wire through middleware pipeline
    let path_str = path.as_ref().display().to_string();
    let operation = DirectoryCreateOperation::new(path_str);
    let context = ExecutionContext::new(SecurityContext::new(user.into()));
    let executor = FilesystemExecutor::new();
    executor.execute(operation, &context).await?;
    Ok(())
}

// ============================================================================
// Process Helpers (3 functions)
// ============================================================================

/// Spawn a process with arguments and security context.
///
/// Returns the process ID (PID) as bytes. The process runs in the background.
///
/// # Current Implementation
/// Direct executor call for simplicity. Returns PID immediately without waiting.
///
/// # Future Integration (OSL-TASK-003, OSL-TASK-004)
/// - TODO: Add security policy validation
/// - TODO: Wire through middleware pipeline
///
/// # Example
///
/// ```rust,no_run
/// use airssys_osl::helpers::*;
///
/// # async fn example() -> airssys_osl::core::result::OSResult<()> {
/// let pid_bytes = spawn_process("ls", vec!["-la".to_string()], "admin").await?;
/// let pid = String::from_utf8_lossy(&pid_bytes).parse::<u32>().unwrap();
/// println!("Spawned process with PID: {}", pid);
/// # Ok(())
/// # }
/// ```
pub async fn spawn_process(
    program: impl Into<String>,
    args: Vec<String>,
    user: impl Into<String>,
) -> OSResult<Vec<u8>> {
    // TODO(OSL-TASK-003): Add security validation
    // TODO(OSL-TASK-004): Wire through middleware pipeline
    let operation = ProcessSpawnOperation::new(program).with_args(args);
    let context = ExecutionContext::new(SecurityContext::new(user.into()));
    let executor = ProcessExecutor::new("helper_executor");
    let result = executor.execute(operation, &context).await?;
    Ok(result.output)
}

/// Kill a process by PID with security context.
///
/// # Current Implementation
/// Direct executor call for simplicity.
///
/// # Future Integration (OSL-TASK-003, OSL-TASK-004)
/// - TODO: Add security policy validation
/// - TODO: Wire through middleware pipeline
///
/// # Example
///
/// ```rust,no_run
/// use airssys_osl::helpers::*;
///
/// # async fn example() -> airssys_osl::core::result::OSResult<()> {
/// kill_process(1234, "admin").await?;
/// # Ok(())
/// # }
/// ```
pub async fn kill_process(pid: u32, user: impl Into<String>) -> OSResult<()> {
    // TODO(OSL-TASK-003): Add security validation
    // TODO(OSL-TASK-004): Wire through middleware pipeline
    let operation = ProcessKillOperation::new(pid);
    let context = ExecutionContext::new(SecurityContext::new(user.into()));
    let executor = ProcessExecutor::new("helper_executor");
    executor.execute(operation, &context).await?;
    Ok(())
}

/// Send signal to process with security context.
///
/// # Current Implementation
/// Direct executor call for simplicity.
///
/// # Future Integration (OSL-TASK-003, OSL-TASK-004)
/// - TODO: Add security policy validation
/// - TODO: Wire through middleware pipeline
///
/// # Example
///
/// ```rust,no_run
/// use airssys_osl::helpers::*;
///
/// # async fn example() -> airssys_osl::core::result::OSResult<()> {
/// send_signal(1234, 15, "admin").await?;  // SIGTERM
/// # Ok(())
/// # }
/// ```
pub async fn send_signal(pid: u32, signal: i32, user: impl Into<String>) -> OSResult<()> {
    // TODO(OSL-TASK-003): Add security validation
    // TODO(OSL-TASK-004): Wire through middleware pipeline
    let operation = ProcessSignalOperation::new(pid, signal);
    let context = ExecutionContext::new(SecurityContext::new(user.into()));
    let executor = ProcessExecutor::new("helper_executor");
    executor.execute(operation, &context).await?;
    Ok(())
}

// ============================================================================
// Network Helpers (3 functions)
// ============================================================================

/// Connect to network address with security context.
///
/// # Current Implementation
/// Direct executor call for simplicity.
///
/// # Future Integration (OSL-TASK-003, OSL-TASK-004)
/// - TODO: Add security policy validation
/// - TODO: Wire through middleware pipeline
///
/// # Example
///
/// ```rust,no_run
/// use airssys_osl::helpers::*;
///
/// # async fn example() -> airssys_osl::core::result::OSResult<()> {
/// let data = network_connect("127.0.0.1:8080", "admin").await?;
/// # Ok(())
/// # }
/// ```
pub async fn network_connect(
    addr: impl Into<String>,
    user: impl Into<String>,
) -> OSResult<Vec<u8>> {
    // TODO(OSL-TASK-003): Add security validation
    // TODO(OSL-TASK-004): Wire through middleware pipeline
    let operation = NetworkConnectOperation::new(addr.into());
    let context = ExecutionContext::new(SecurityContext::new(user.into()));
    let executor = NetworkExecutor::new("helper_executor");
    let result = executor.execute(operation, &context).await?;
    Ok(result.output)
}

/// Listen on network address with security context.
///
/// # Current Implementation
/// Direct executor call for simplicity.
///
/// # Future Integration (OSL-TASK-003, OSL-TASK-004)
/// - TODO: Add security policy validation
/// - TODO: Wire through middleware pipeline
///
/// # Example
///
/// ```rust,no_run
/// use airssys_osl::helpers::*;
///
/// # async fn example() -> airssys_osl::core::result::OSResult<()> {
/// let data = network_listen("127.0.0.1:8080", "admin").await?;
/// # Ok(())
/// # }
/// ```
pub async fn network_listen(addr: impl Into<String>, user: impl Into<String>) -> OSResult<Vec<u8>> {
    // TODO(OSL-TASK-003): Add security validation
    // TODO(OSL-TASK-004): Wire through middleware pipeline
    let operation = NetworkListenOperation::new(addr.into());
    let context = ExecutionContext::new(SecurityContext::new(user.into()));
    let executor = NetworkExecutor::new("helper_executor");
    let result = executor.execute(operation, &context).await?;
    Ok(result.output)
}

/// Create network socket with security context.
///
/// # Current Implementation
/// Direct executor call for simplicity.
///
/// # Future Integration (OSL-TASK-003, OSL-TASK-004)
/// - TODO: Add security policy validation
/// - TODO: Wire through middleware pipeline
///
/// # Example
///
/// ```rust,no_run
/// use airssys_osl::helpers::*;
///
/// # async fn example() -> airssys_osl::core::result::OSResult<()> {
/// let data = create_socket("tcp", "admin").await?;
/// # Ok(())
/// # }
/// ```
pub async fn create_socket(
    socket_type: impl Into<String>,
    user: impl Into<String>,
) -> OSResult<Vec<u8>> {
    // TODO(OSL-TASK-003): Add security validation
    // TODO(OSL-TASK-004): Wire through middleware pipeline
    let operation = NetworkSocketOperation::new(socket_type.into());
    let context = ExecutionContext::new(SecurityContext::new(user.into()));
    let executor = NetworkExecutor::new("helper_executor");
    let result = executor.execute(operation, &context).await?;
    Ok(result.output)
}

// ============================================================================
// Tests
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    // Filesystem helper tests

    #[allow(clippy::expect_used)]
    #[tokio::test]
    async fn test_read_file_helper() {
        let temp_dir = TempDir::new().expect("Failed to create temp dir");
        let file_path = temp_dir.path().join("test.txt");

        // Write test data first
        std::fs::write(&file_path, b"test data").expect("Failed to write test file");

        // Test helper function
        let result = read_file(&file_path, "test_user").await;
        assert!(result.is_ok());
        assert_eq!(result.expect("Should have result"), b"test data");
    }

    #[allow(clippy::expect_used)]
    #[tokio::test]
    async fn test_write_file_helper() {
        let temp_dir = TempDir::new().expect("Failed to create temp dir");
        let file_path = temp_dir.path().join("write_test.txt");

        // Test helper function
        let result = write_file(&file_path, b"hello world".to_vec(), "test_user").await;
        assert!(result.is_ok());

        // Verify file was written
        let content = std::fs::read(&file_path).expect("Failed to read file");
        assert_eq!(content, b"hello world");
    }

    #[allow(clippy::expect_used)]
    #[tokio::test]
    async fn test_delete_file_helper() {
        let temp_dir = TempDir::new().expect("Failed to create temp dir");
        let file_path = temp_dir.path().join("delete_test.txt");

        // Create file first
        std::fs::write(&file_path, b"delete me").expect("Failed to write test file");
        assert!(file_path.exists());

        // Test helper function
        let result = delete_file(&file_path, "test_user").await;
        assert!(result.is_ok());

        // Verify file was deleted
        assert!(!file_path.exists());
    }

    #[allow(clippy::expect_used)]
    #[tokio::test]
    async fn test_create_directory_helper() {
        let temp_dir = TempDir::new().expect("Failed to create temp dir");
        let dir_path = temp_dir.path().join("new_dir");

        // Test helper function
        let result = create_directory(&dir_path, "test_user").await;
        assert!(result.is_ok());

        // Verify directory was created
        assert!(dir_path.exists());
        assert!(dir_path.is_dir());
    }

    // Process helper tests

    #[allow(clippy::expect_used)]
    #[tokio::test]
    async fn test_spawn_process_helper() {
        // Test spawning a simple command
        #[cfg(unix)]
        let result = spawn_process("sleep", vec!["0".to_string()], "test_user").await;

        #[cfg(windows)]
        let result = spawn_process(
            "cmd",
            vec!["/C".to_string(), "timeout".to_string(), "0".to_string()],
            "test_user",
        )
        .await;

        // Should succeed and return PID as output
        assert!(result.is_ok());
        let pid_bytes = result.expect("Should have PID");
        let pid_str = String::from_utf8_lossy(&pid_bytes);
        // PID should be a parseable number
        assert!(pid_str.parse::<u32>().is_ok());
    }

    #[tokio::test]
    async fn test_kill_process_helper() {
        // Test with non-existent PID (should fail gracefully)
        let result = kill_process(99999, "test_user").await;
        // We expect this to fail since PID doesn't exist
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_send_signal_helper() {
        // Test with non-existent PID (should fail gracefully)
        let result = send_signal(99999, 15, "test_user").await;
        // We expect this to fail since PID doesn't exist
        assert!(result.is_err());
    }

    // Network helper tests

    #[tokio::test]
    async fn test_network_connect_helper() {
        // Test with invalid address format (missing port - should fail during connect)
        let result = network_connect("invalid", "test_user").await;
        // We expect this to fail (invalid address format or connection error)
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_network_listen_helper() {
        // Test with invalid address format (should fail)
        let result = network_listen("invalid-address", "test_user").await;
        // Should fail (invalid address format or bind error)
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_create_socket_helper() {
        // Test with unsupported socket type (should fail validation)
        let result = create_socket("invalid_type", "test_user").await;
        // Should fail (unsupported socket type)
        assert!(result.is_err());
    }
}
