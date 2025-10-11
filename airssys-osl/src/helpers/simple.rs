//! Simple helper functions (Level 1 & 2 API).
//!
//! This module contains the basic helper functions that enforce security
//! via middleware integration. Each function has two variants:
//! - Simple: Uses default security middleware (Level 1 API)
//! - With middleware: Accepts custom middleware (Level 2 API)

// Layer 1: Standard library imports
use std::path::Path;

// Layer 2: No third-party imports needed

// Layer 3: Internal module imports
use crate::core::context::{ExecutionContext, SecurityContext};
use crate::core::executor::OSExecutor;
use crate::core::middleware::Middleware;
use crate::core::result::OSResult;
use crate::executors::filesystem::FilesystemExecutor;
use crate::executors::network::NetworkExecutor;
use crate::executors::process::ProcessExecutor;
use crate::middleware::ext::ExecutorExt;
use crate::operations::filesystem::{
    DirectoryCreateOperation, FileDeleteOperation, FileReadOperation, FileWriteOperation,
};
use crate::operations::network::{
    NetworkConnectOperation, NetworkListenOperation, NetworkSocketOperation,
};
use crate::operations::process::{
    ProcessKillOperation, ProcessSignalOperation, ProcessSpawnOperation,
};

// Import factory functions
use super::factories::default_security_middleware;

// ============================================================================
// Filesystem Helpers (4 functions × 2 variants = 8 total)
// ============================================================================

/// Read file contents with default security middleware.
///
/// This function enforces deny-by-default security via [`default_security_middleware()`],
/// which includes ACL policy, RBAC policy, and audit logging.
///
/// # Security
///
/// - **ACL**: Admin user has full access by default
/// - **RBAC**: Role-based permissions enforced
/// - **Audit**: All operations logged to console
/// - **Deny-by-default**: Operations denied unless explicitly allowed
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
///
/// # Custom Security
///
/// For custom middleware, use [`read_file_with_middleware()`].
pub async fn read_file<P: AsRef<Path>>(path: P, user: impl Into<String>) -> OSResult<Vec<u8>> {
    read_file_with_middleware(path, user, default_security_middleware()).await
}

/// Read file contents with custom middleware.
///
/// This variant accepts custom middleware for advanced use cases like
/// custom ACL/RBAC policies, rate limiting, caching, or metrics collection.
///
/// # Example
///
/// ```rust,no_run
/// use airssys_osl::helpers::*;
/// use airssys_osl::middleware::security::*;
///
/// # async fn example() -> airssys_osl::core::result::OSResult<()> {
/// // Custom ACL policy
/// let acl = AccessControlList::new()
///     .add_entry(AclEntry::new(
///         "alice".to_string(),
///         "/data/*".to_string(),
///         vec!["read".to_string()],
///         AclPolicy::Allow
///     ));
///
/// let security = SecurityMiddlewareBuilder::new()
///     .add_policy(Box::new(acl))
///     .build()
///     .expect("Failed to build security middleware");
///
/// let data = read_file_with_middleware("/data/file.txt", "alice", security).await?;
/// # Ok(())
/// # }
/// ```
pub async fn read_file_with_middleware<P, M>(
    path: P,
    user: impl Into<String>,
    middleware: M,
) -> OSResult<Vec<u8>>
where
    P: AsRef<Path>,
    M: Middleware<FileReadOperation>,
{
    let path_str = path.as_ref().display().to_string();
    let operation = FileReadOperation::new(path_str);
    let context = ExecutionContext::new(SecurityContext::new(user.into()));

    let executor = FilesystemExecutor::new().with_middleware(middleware);

    let result = executor.execute(operation, &context).await?;
    Ok(result.output)
}

/// Write data to file with default security middleware.
///
/// This function enforces deny-by-default security via [`default_security_middleware()`],
/// which includes ACL policy, RBAC policy, and audit logging.
///
/// # Security
///
/// - **ACL**: Admin user has full access by default
/// - **RBAC**: Role-based permissions enforced
/// - **Audit**: All operations logged to console
/// - **Deny-by-default**: Operations denied unless explicitly allowed
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
///
/// # Custom Security
///
/// For custom middleware, use [`write_file_with_middleware()`].
pub async fn write_file<P: AsRef<Path>>(
    path: P,
    data: Vec<u8>,
    user: impl Into<String>,
) -> OSResult<()> {
    write_file_with_middleware(path, data, user, default_security_middleware()).await
}

/// Write data to file with custom middleware.
///
/// This variant accepts custom middleware for advanced use cases like
/// custom ACL/RBAC policies, rate limiting, or metrics collection.
///
/// # Example
///
/// ```rust,no_run
/// use airssys_osl::helpers::*;
/// use airssys_osl::middleware::security::*;
///
/// # async fn example() -> airssys_osl::core::result::OSResult<()> {
/// let security = SecurityMiddlewareBuilder::new()
///     .build()
///     .expect("Failed to build security middleware");
///
/// let data = b"Hello, World!".to_vec();
/// write_file_with_middleware("/tmp/test.txt", data, "admin", security).await?;
/// # Ok(())
/// # }
/// ```
pub async fn write_file_with_middleware<P, M>(
    path: P,
    data: Vec<u8>,
    user: impl Into<String>,
    middleware: M,
) -> OSResult<()>
where
    P: AsRef<Path>,
    M: Middleware<FileWriteOperation>,
{
    let path_str = path.as_ref().display().to_string();
    let operation = FileWriteOperation::new(path_str, data);
    let context = ExecutionContext::new(SecurityContext::new(user.into()));

    let executor = FilesystemExecutor::new().with_middleware(middleware);

    executor.execute(operation, &context).await?;
    Ok(())
}

/// Delete file with default security middleware.
///
/// This function enforces deny-by-default security via [`default_security_middleware()`],
/// which includes ACL policy, RBAC policy, and audit logging.
///
/// # Security
///
/// - **ACL**: Admin user has full access by default
/// - **RBAC**: Role-based permissions enforced
/// - **Audit**: All operations logged to console
/// - **Deny-by-default**: Operations denied unless explicitly allowed
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
///
/// # Custom Security
///
/// For custom middleware, use [`delete_file_with_middleware()`].
pub async fn delete_file<P: AsRef<Path>>(path: P, user: impl Into<String>) -> OSResult<()> {
    delete_file_with_middleware(path, user, default_security_middleware()).await
}

/// Delete file with custom middleware.
///
/// This variant accepts custom middleware for advanced use cases like
/// custom ACL/RBAC policies or audit logging.
///
/// # Example
///
/// ```rust,no_run
/// use airssys_osl::helpers::*;
/// use airssys_osl::middleware::security::*;
///
/// # async fn example() -> airssys_osl::core::result::OSResult<()> {
/// let security = SecurityMiddlewareBuilder::new()
///     .build()
///     .expect("Failed to build security middleware");
///
/// delete_file_with_middleware("/tmp/test.txt", "admin", security).await?;
/// # Ok(())
/// # }
/// ```
pub async fn delete_file_with_middleware<P, M>(
    path: P,
    user: impl Into<String>,
    middleware: M,
) -> OSResult<()>
where
    P: AsRef<Path>,
    M: Middleware<FileDeleteOperation>,
{
    let path_str = path.as_ref().display().to_string();
    let operation = FileDeleteOperation::new(path_str);
    let context = ExecutionContext::new(SecurityContext::new(user.into()));

    let executor = FilesystemExecutor::new().with_middleware(middleware);

    executor.execute(operation, &context).await?;
    Ok(())
}

/// Create directory with default security middleware.
///
/// This function enforces deny-by-default security via [`default_security_middleware()`],
/// which includes ACL policy, RBAC policy, and audit logging.
///
/// # Security
///
/// - **ACL**: Admin user has full access by default
/// - **RBAC**: Role-based permissions enforced
/// - **Audit**: All operations logged to console
/// - **Deny-by-default**: Operations denied unless explicitly allowed
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
///
/// # Custom Security
///
/// For custom middleware, use [`create_directory_with_middleware()`].
pub async fn create_directory<P: AsRef<Path>>(path: P, user: impl Into<String>) -> OSResult<()> {
    create_directory_with_middleware(path, user, default_security_middleware()).await
}

/// Create directory with custom middleware.
///
/// This variant accepts custom middleware for advanced use cases like
/// custom ACL/RBAC policies or audit logging.
///
/// # Example
///
/// ```rust,no_run
/// use airssys_osl::helpers::*;
/// use airssys_osl::middleware::security::*;
///
/// # async fn example() -> airssys_osl::core::result::OSResult<()> {
/// let security = SecurityMiddlewareBuilder::new()
///     .build()
///     .expect("Failed to build security middleware");
///
/// create_directory_with_middleware("/tmp/test_dir", "admin", security).await?;
/// # Ok(())
/// # }
/// ```
pub async fn create_directory_with_middleware<P, M>(
    path: P,
    user: impl Into<String>,
    middleware: M,
) -> OSResult<()>
where
    P: AsRef<Path>,
    M: Middleware<DirectoryCreateOperation>,
{
    let path_str = path.as_ref().display().to_string();
    let operation = DirectoryCreateOperation::new(path_str);
    let context = ExecutionContext::new(SecurityContext::new(user.into()));

    let executor = FilesystemExecutor::new().with_middleware(middleware);

    executor.execute(operation, &context).await?;
    Ok(())
}

// ============================================================================
// Process Helpers (3 functions × 2 variants = 6 total)
// ============================================================================

/// Spawn a process with default security middleware.
///
/// This function enforces deny-by-default security via [`default_security_middleware()`],
/// which includes ACL policy, RBAC policy, and audit logging.
///
/// Returns the process ID (PID) as bytes. The process runs in the background.
///
/// # Security
///
/// - **ACL**: Admin user has full access by default
/// - **RBAC**: Role-based permissions enforced (`process:spawn`)
/// - **Audit**: All operations logged to console
/// - **Deny-by-default**: Operations denied unless explicitly allowed
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
///
/// # Custom Security
///
/// For custom middleware, use [`spawn_process_with_middleware()`].
pub async fn spawn_process(
    program: impl Into<String>,
    args: Vec<String>,
    user: impl Into<String>,
) -> OSResult<Vec<u8>> {
    spawn_process_with_middleware(program, args, user, default_security_middleware()).await
}

/// Spawn a process with custom middleware.
///
/// This variant accepts custom middleware for advanced use cases like
/// custom ACL/RBAC policies, resource limits, or process monitoring.
///
/// # Example
///
/// ```rust,no_run
/// use airssys_osl::helpers::*;
/// use airssys_osl::middleware::security::*;
///
/// # async fn example() -> airssys_osl::core::result::OSResult<()> {
/// let security = SecurityMiddlewareBuilder::new()
///     .build()
///     .expect("Failed to build security middleware");
///
/// let pid_bytes = spawn_process_with_middleware(
///     "ls",
///     vec!["-la".to_string()],
///     "admin",
///     security
/// ).await?;
/// # Ok(())
/// # }
/// ```
pub async fn spawn_process_with_middleware<M>(
    program: impl Into<String>,
    args: Vec<String>,
    user: impl Into<String>,
    middleware: M,
) -> OSResult<Vec<u8>>
where
    M: Middleware<ProcessSpawnOperation>,
{
    let operation = ProcessSpawnOperation::new(program).with_args(args);
    let context = ExecutionContext::new(SecurityContext::new(user.into()));

    let executor = ProcessExecutor::new("helper_executor").with_middleware(middleware);

    let result = executor.execute(operation, &context).await?;
    Ok(result.output)
}

/// Kill a process by PID with default security middleware.
///
/// This function enforces deny-by-default security via [`default_security_middleware()`],
/// which includes ACL policy, RBAC policy, and audit logging.
///
/// # Security
///
/// - **ACL**: Admin user has full access by default
/// - **RBAC**: Role-based permissions enforced (`process:kill`)
/// - **Audit**: All operations logged to console
/// - **Deny-by-default**: Operations denied unless explicitly allowed
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
///
/// # Custom Security
///
/// For custom middleware, use [`kill_process_with_middleware()`].
pub async fn kill_process(pid: u32, user: impl Into<String>) -> OSResult<()> {
    kill_process_with_middleware(pid, user, default_security_middleware()).await
}

/// Kill a process with custom middleware.
///
/// This variant accepts custom middleware for advanced use cases like
/// custom ACL/RBAC policies or audit logging.
///
/// # Example
///
/// ```rust,no_run
/// use airssys_osl::helpers::*;
/// use airssys_osl::middleware::security::*;
///
/// # async fn example() -> airssys_osl::core::result::OSResult<()> {
/// let security = SecurityMiddlewareBuilder::new()
///     .build()
///     .expect("Failed to build security middleware");
///
/// kill_process_with_middleware(1234, "admin", security).await?;
/// # Ok(())
/// # }
/// ```
pub async fn kill_process_with_middleware<M>(
    pid: u32,
    user: impl Into<String>,
    middleware: M,
) -> OSResult<()>
where
    M: Middleware<ProcessKillOperation>,
{
    let operation = ProcessKillOperation::new(pid);
    let context = ExecutionContext::new(SecurityContext::new(user.into()));

    let executor = ProcessExecutor::new("helper_executor").with_middleware(middleware);

    executor.execute(operation, &context).await?;
    Ok(())
}

/// Send signal to process with default security middleware.
///
/// This function enforces deny-by-default security via [`default_security_middleware()`],
/// which includes ACL policy, RBAC policy, and audit logging.
///
/// # Security
///
/// - **ACL**: Admin user has full access by default
/// - **RBAC**: Role-based permissions enforced (`process:signal`)
/// - **Audit**: All operations logged to console
/// - **Deny-by-default**: Operations denied unless explicitly allowed
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
///
/// # Custom Security
///
/// For custom middleware, use [`send_signal_with_middleware()`].
pub async fn send_signal(pid: u32, signal: i32, user: impl Into<String>) -> OSResult<()> {
    send_signal_with_middleware(pid, signal, user, default_security_middleware()).await
}

/// Send signal to process with custom middleware.
///
/// This variant accepts custom middleware for advanced use cases like
/// custom ACL/RBAC policies or signal validation.
///
/// # Example
///
/// ```rust,no_run
/// use airssys_osl::helpers::*;
/// use airssys_osl::middleware::security::*;
///
/// # async fn example() -> airssys_osl::core::result::OSResult<()> {
/// let security = SecurityMiddlewareBuilder::new()
///     .build()
///     .expect("Failed to build security middleware");
///
/// send_signal_with_middleware(1234, 15, "admin", security).await?;
/// # Ok(())
/// # }
/// ```
pub async fn send_signal_with_middleware<M>(
    pid: u32,
    signal: i32,
    user: impl Into<String>,
    middleware: M,
) -> OSResult<()>
where
    M: Middleware<ProcessSignalOperation>,
{
    let operation = ProcessSignalOperation::new(pid, signal);
    let context = ExecutionContext::new(SecurityContext::new(user.into()));

    let executor = ProcessExecutor::new("helper_executor").with_middleware(middleware);

    executor.execute(operation, &context).await?;
    Ok(())
}

// ============================================================================
// Network Helpers (3 functions × 2 variants = 6 total)
// ============================================================================

/// Connect to network address with default security middleware.
///
/// This function enforces deny-by-default security via [`default_security_middleware()`],
/// which includes ACL policy, RBAC policy, and audit logging.
///
/// # Security
///
/// - **ACL**: Admin user has full access by default
/// - **RBAC**: Role-based permissions enforced (`network:connect`)
/// - **Audit**: All operations logged to console
/// - **Deny-by-default**: Operations denied unless explicitly allowed
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
///
/// # Custom Security
///
/// For custom middleware, use [`network_connect_with_middleware()`].
pub async fn network_connect(
    addr: impl Into<String>,
    user: impl Into<String>,
) -> OSResult<Vec<u8>> {
    network_connect_with_middleware(addr, user, default_security_middleware()).await
}

/// Connect to network address with custom middleware.
///
/// This variant accepts custom middleware for advanced use cases like
/// custom ACL/RBAC policies, connection pooling, or rate limiting.
///
/// # Example
///
/// ```rust,no_run
/// use airssys_osl::helpers::*;
/// use airssys_osl::middleware::security::*;
///
/// # async fn example() -> airssys_osl::core::result::OSResult<()> {
/// let security = SecurityMiddlewareBuilder::new()
///     .build()
///     .expect("Failed to build security middleware");
///
/// let data = network_connect_with_middleware("127.0.0.1:8080", "admin", security).await?;
/// # Ok(())
/// # }
/// ```
pub async fn network_connect_with_middleware<M>(
    addr: impl Into<String>,
    user: impl Into<String>,
    middleware: M,
) -> OSResult<Vec<u8>>
where
    M: Middleware<NetworkConnectOperation>,
{
    let operation = NetworkConnectOperation::new(addr.into());
    let context = ExecutionContext::new(SecurityContext::new(user.into()));

    let executor = NetworkExecutor::new("helper_executor").with_middleware(middleware);

    let result = executor.execute(operation, &context).await?;
    Ok(result.output)
}

/// Listen on network address with default security middleware.
///
/// This function enforces deny-by-default security via [`default_security_middleware()`],
/// which includes ACL policy, RBAC policy, and audit logging.
///
/// # Security
///
/// - **ACL**: Admin user has full access by default
/// - **RBAC**: Role-based permissions enforced (`network:listen`)
/// - **Audit**: All operations logged to console
/// - **Deny-by-default**: Operations denied unless explicitly allowed
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
///
/// # Custom Security
///
/// For custom middleware, use [`network_listen_with_middleware()`].
pub async fn network_listen(addr: impl Into<String>, user: impl Into<String>) -> OSResult<Vec<u8>> {
    network_listen_with_middleware(addr, user, default_security_middleware()).await
}

/// Listen on network address with custom middleware.
///
/// This variant accepts custom middleware for advanced use cases like
/// custom ACL/RBAC policies, connection limits, or metrics collection.
///
/// # Example
///
/// ```rust,no_run
/// use airssys_osl::helpers::*;
/// use airssys_osl::middleware::security::*;
///
/// # async fn example() -> airssys_osl::core::result::OSResult<()> {
/// let security = SecurityMiddlewareBuilder::new()
///     .build()
///     .expect("Failed to build security middleware");
///
/// let data = network_listen_with_middleware("127.0.0.1:8080", "admin", security).await?;
/// # Ok(())
/// # }
/// ```
pub async fn network_listen_with_middleware<M>(
    addr: impl Into<String>,
    user: impl Into<String>,
    middleware: M,
) -> OSResult<Vec<u8>>
where
    M: Middleware<NetworkListenOperation>,
{
    let operation = NetworkListenOperation::new(addr.into());
    let context = ExecutionContext::new(SecurityContext::new(user.into()));

    let executor = NetworkExecutor::new("helper_executor").with_middleware(middleware);

    let result = executor.execute(operation, &context).await?;
    Ok(result.output)
}

/// Create network socket with default security middleware.
///
/// This function enforces deny-by-default security via [`default_security_middleware()`],
/// which includes ACL policy, RBAC policy, and audit logging.
///
/// # Security
///
/// - **ACL**: Admin user has full access by default
/// - **RBAC**: Role-based permissions enforced (`network:socket`)
/// - **Audit**: All operations logged to console
/// - **Deny-by-default**: Operations denied unless explicitly allowed
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
///
/// # Custom Security
///
/// For custom middleware, use [`create_socket_with_middleware()`].
pub async fn create_socket(
    socket_type: impl Into<String>,
    user: impl Into<String>,
) -> OSResult<Vec<u8>> {
    create_socket_with_middleware(socket_type, user, default_security_middleware()).await
}

/// Create network socket with custom middleware.
///
/// This variant accepts custom middleware for advanced use cases like
/// custom ACL/RBAC policies, socket validation, or resource tracking.
///
/// # Example
///
/// ```rust,no_run
/// use airssys_osl::helpers::*;
/// use airssys_osl::middleware::security::*;
///
/// # async fn example() -> airssys_osl::core::result::OSResult<()> {
/// let security = SecurityMiddlewareBuilder::new()
///     .build()
///     .expect("Failed to build security middleware");
///
/// let data = create_socket_with_middleware("tcp", "admin", security).await?;
/// # Ok(())
/// # }
/// ```
pub async fn create_socket_with_middleware<M>(
    socket_type: impl Into<String>,
    user: impl Into<String>,
    middleware: M,
) -> OSResult<Vec<u8>>
where
    M: Middleware<NetworkSocketOperation>,
{
    let operation = NetworkSocketOperation::new(socket_type.into());
    let context = ExecutionContext::new(SecurityContext::new(user.into()));

    let executor = NetworkExecutor::new("helper_executor").with_middleware(middleware);

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

        // Test helper function (using "admin" user allowed by default ACL)
        let result = read_file(&file_path, "admin").await;
        assert!(result.is_ok());
        assert_eq!(result.expect("Should have result"), b"test data");
    }

    #[allow(clippy::expect_used)]
    #[tokio::test]
    async fn test_write_file_helper() {
        let temp_dir = TempDir::new().expect("Failed to create temp dir");
        let file_path = temp_dir.path().join("write_test.txt");

        // Test helper function (using "admin" user allowed by default ACL)
        let result = write_file(&file_path, b"hello world".to_vec(), "admin").await;
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

        // Test helper function (using "admin" user allowed by default ACL)
        let result = delete_file(&file_path, "admin").await;
        assert!(result.is_ok());

        // Verify file was deleted
        assert!(!file_path.exists());
    }

    #[allow(clippy::expect_used)]
    #[tokio::test]
    async fn test_create_directory_helper() {
        let temp_dir = TempDir::new().expect("Failed to create temp dir");
        let dir_path = temp_dir.path().join("new_dir");

        // Test helper function (using "admin" user allowed by default ACL)
        let result = create_directory(&dir_path, "admin").await;
        assert!(result.is_ok());

        // Verify directory was created
        assert!(dir_path.exists());
        assert!(dir_path.is_dir());
    }

    // Process helper tests

    #[allow(clippy::expect_used)]
    #[tokio::test]
    async fn test_spawn_process_helper() {
        // Test spawning a simple command (using "admin" user allowed by default ACL+RBAC)
        #[cfg(unix)]
        let result = spawn_process("sleep", vec!["0".to_string()], "admin").await;

        #[cfg(windows)]
        let result = spawn_process(
            "cmd",
            vec!["/C".to_string(), "timeout".to_string(), "0".to_string()],
            "admin",
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
        // Test with non-existent PID (should fail gracefully even for admin)
        let result = kill_process(99999, "admin").await;
        // We expect this to fail since PID doesn't exist
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_send_signal_helper() {
        // Test with non-existent PID (should fail gracefully even for admin)
        let result = send_signal(99999, 15, "admin").await;
        // We expect this to fail since PID doesn't exist
        assert!(result.is_err());
    }

    // Network helper tests

    #[tokio::test]
    async fn test_network_connect_helper() {
        // Test with invalid address format (missing port - should fail during connect)
        let result = network_connect("invalid", "admin").await;
        // We expect this to fail (invalid address format or connection error)
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_network_listen_helper() {
        // Test with invalid address format (should fail)
        let result = network_listen("invalid-address", "admin").await;
        // Should fail (invalid address format or bind error)
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_create_socket_helper() {
        // Test with unsupported socket type (should fail validation)
        let result = create_socket("invalid_type", "admin").await;
        // Should fail (unsupported socket type)
        assert!(result.is_err());
    }
}
