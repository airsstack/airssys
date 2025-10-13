//! Trait-based composition for reusable helper pipelines.
//!
//! This module provides a functional programming approach to building
//! reusable middleware pipelines for OS operations.
//!
//! # The Problem
//!
//! When using Level 1 or Level 2 APIs, middleware must be specified
//! for every operation:
//!
//! ```rust,ignore
//! // Repetitive - middleware configured 3 times
//! let data1 = read_file_with_middleware("/file1", "user", middleware.clone()).await?;
//! let data2 = read_file_with_middleware("/file2", "user", middleware.clone()).await?;
//! let data3 = read_file_with_middleware("/file3", "user", middleware.clone()).await?;
//! ```
//!
//! # The Solution: Trait Composition
//!
//! Build the pipeline **once**, reuse it **many times**:
//!
//! ```rust,no_run
//! use airssys_osl::helpers::composition::*;
//! use airssys_osl::middleware::security::SecurityMiddleware;
//!
//! # async fn example() -> airssys_osl::core::result::OSResult<()> {
//! // Configure once
//! let reader = FileHelper::builder()
//!     .with_security(SecurityMiddleware::default());
//!
//! // Use many times efficiently
//! let data1 = reader.read("/file1", "user").await?;
//! let data2 = reader.read("/file2", "user").await?;
//! let data3 = reader.read("/file3", "user").await?;
//! # Ok(())
//! # }
//! ```
//!
//! # Performance
//!
//! - **Zero-cost abstraction**: No runtime overhead vs manual composition
//! - **Compile-time optimization**: Generic types fully monomorphized
//! - **Efficient reuse**: Pipeline setup amortized across many operations
//! - **No allocations**: All composition happens at compile time
//!
//! # Usage Patterns
//!
//! ## Pattern 1: Specialized Helpers for Different Contexts
//!
//! ```rust,no_run
//! use airssys_osl::helpers::composition::*;
//! use airssys_osl::middleware::security::SecurityMiddleware;
//!
//! # async fn example() -> airssys_osl::core::result::OSResult<()> {
//! // Public-facing reader with default security
//! let public_reader = FileHelper::builder()
//!     .with_security(SecurityMiddleware::default());
//!
//! // Admin writer with strict security
//! let admin_writer = FileHelper::builder()
//!     .with_security(SecurityMiddleware::default());
//! # Ok(())
//! # }
//! ```
//!
//! ## Pattern 2: Long-Running Services
//!
//! ```rust,ignore
//! // Configure at service startup
//! struct MyService {
//!     config_reader: ComposedHelper<...>,
//!     data_writer: ComposedHelper<...>,
//! }
//!
//! impl MyService {
//!     fn new() -> Self {
//!         Self {
//!             config_reader: FileHelper::builder().with_security(...),
//!             data_writer: FileHelper::builder().with_security(...),
//!         }
//!     }
//!
//!     async fn process_request(&self) -> Result<()> {
//!         // Reuse pre-configured pipelines
//!         let config = self.config_reader.read(...).await?;
//!         self.data_writer.write(...).await?;
//!         Ok(())
//!     }
//! }
//! ```
//!
//! # When to Use Level 3
//!
//! **Use Level 3 (Trait Composition) when:**
//! - Running long-lived services or microservices
//! - Processing high-throughput operations (>100 ops/sec)
//! - Building libraries with consistent middleware patterns
//! - Need clear separation of pipeline config from execution
//! - Functional programming style preferred
//!
//! **Use Level 2 (Custom Middleware) when:**
//! - One-off operations with custom middleware
//! - Exploring different middleware combinations
//! - Simple scripts with few operations
//!
//! **Use Level 1 (Simple Helpers) when:**
//! - Quick scripts and utilities
//! - Default security is sufficient
//! - Learning the framework
//!
//! # See Also
//!
//! - [`HelperPipeline`]: Core composition trait
//! - [`ComposedHelper`]: Pipeline wrapper
//! - [`FileHelper`], [`ProcessHelper`], [`NetworkHelper`]: Helper builders
//! - Level 1 API: `airssys_osl::helpers::*` (simple functions)
//! - Level 2 API: `airssys_osl::helpers::*_with_middleware()` variants

// Layer 1: Standard library imports
use std::marker::PhantomData;

// Layer 2: Third-party crate imports
// (none needed)

// Layer 3: Internal module imports
use crate::core::executor::OSExecutor;
use crate::core::middleware::Middleware;
use crate::core::operation::Operation;
use crate::core::result::OSResult;
use crate::middleware::ext::ExecutorExt;

/// Trait for composable helper pipelines.
///
/// This trait provides a fluent API for building operation pipelines
/// with middleware composition.
///
/// # Type Parameters
///
/// - `O`: The operation type this pipeline handles
///
/// # Methods
///
/// - [`with_security()`]: Add SecurityMiddleware to the pipeline
/// - [`with_middleware()`]: Add any custom middleware to the pipeline
/// - [`executor()`]: Access the underlying executor
///
/// # Example
///
/// ```rust,no_run
/// use airssys_osl::helpers::composition::*;
/// use airssys_osl::middleware::security::SecurityMiddleware;
///
/// # async fn example() -> airssys_osl::core::result::OSResult<()> {
/// let helper = FileHelper::builder()
///     .with_security(SecurityMiddleware::default());
///
/// let data = helper.read("/etc/hosts", "admin").await?;
/// # Ok(())
/// # }
/// ```
pub trait HelperPipeline<O: Operation>: Sized {
    /// The underlying executor type.
    type Executor: OSExecutor<O>;

    /// Add security middleware to the pipeline.
    ///
    /// This is a convenience method for adding SecurityMiddleware.
    /// Use [`with_middleware()`] for other middleware types.
    ///
    /// # Example
    ///
    /// ```rust,no_run
    /// # use airssys_osl::helpers::composition::*;
    /// # use airssys_osl::middleware::security::SecurityMiddleware;
    /// # async fn example() -> airssys_osl::core::result::OSResult<()> {
    /// let helper = FileHelper::builder()
    ///     .with_security(SecurityMiddleware::default());
    /// # Ok(())
    /// # }
    /// ```
    fn with_security(
        self,
        middleware: crate::middleware::security::SecurityMiddleware,
    ) -> ComposedHelper<
        O,
        crate::middleware::ext::MiddlewareExecutor<
            Self::Executor,
            crate::middleware::security::SecurityMiddleware,
            O,
        >,
    >;

    /// Add custom middleware to the pipeline.
    ///
    /// This is the generic method that accepts any middleware type.
    /// Use this for custom middleware like rate limiting, caching, metrics, etc.
    ///
    /// # Type Parameters
    ///
    /// - `M`: Middleware type implementing `Middleware<O>`
    ///
    /// # Examples
    ///
    /// ```rust,no_run
    /// # use airssys_osl::helpers::composition::*;
    /// # use airssys_osl::middleware::security::SecurityMiddleware;
    /// // Using custom middleware (pseudocode - requires custom implementation)
    /// // let helper = FileHelper::builder()
    /// //     .with_middleware(RateLimitMiddleware::new(100))
    /// //     .with_middleware(MetricsMiddleware::new());
    /// # async fn example() -> airssys_osl::core::result::OSResult<()> {
    /// # Ok(())
    /// # }
    /// ```
    fn with_middleware<M>(
        self,
        middleware: M,
    ) -> ComposedHelper<O, crate::middleware::ext::MiddlewareExecutor<Self::Executor, M, O>>
    where
        M: Middleware<O> + Send + Sync + std::fmt::Debug + 'static;

    /// Get reference to the underlying executor.
    ///
    /// This is primarily useful for testing or advanced use cases
    /// where direct executor access is needed.
    fn executor(&self) -> &Self::Executor;
}

/// A composed helper with middleware pipeline.
///
/// This type wraps an executor with a composable middleware pipeline.
/// It implements [`HelperPipeline`] to enable chaining.
///
/// # Type Parameters
///
/// - `O`: Operation type this helper executes
/// - `E`: Executor type (may be wrapped with middleware)
///
/// # Usage
///
/// `ComposedHelper` is typically created via helper builders like
/// [`FileHelper::builder()`], not constructed directly.
///
/// # Example
///
/// ```rust,no_run
/// use airssys_osl::helpers::composition::*;
/// use airssys_osl::middleware::security::SecurityMiddleware;
///
/// # async fn example() -> airssys_osl::core::result::OSResult<()> {
/// // Created via builder
/// let helper = FileHelper::builder()
///     .with_security(SecurityMiddleware::default());
///
/// // Use the helper
/// let data = helper.read("/file.txt", "user").await?;
/// # Ok(())
/// # }
/// ```
#[derive(Debug)]
pub struct ComposedHelper<O, E>
where
    O: Operation,
    E: OSExecutor<O>,
{
    executor: E,
    _phantom: PhantomData<O>,
}

impl<O, E> ComposedHelper<O, E>
where
    O: Operation,
    E: OSExecutor<O>,
{
    /// Create a new composed helper.
    ///
    /// This is typically called by helper builders, not user code.
    pub fn new(executor: E) -> Self {
        Self {
            executor,
            _phantom: PhantomData,
        }
    }

    /// Get reference to the executor.
    ///
    /// Primarily useful for testing or advanced use cases.
    pub fn executor(&self) -> &E {
        &self.executor
    }
}

// Implement HelperPipeline for ComposedHelper to enable chaining
impl<O, E> HelperPipeline<O> for ComposedHelper<O, E>
where
    O: Operation,
    E: OSExecutor<O> + Send + Sync + std::fmt::Debug,
{
    type Executor = E;

    fn with_security(
        self,
        middleware: crate::middleware::security::SecurityMiddleware,
    ) -> ComposedHelper<
        O,
        crate::middleware::ext::MiddlewareExecutor<
            E,
            crate::middleware::security::SecurityMiddleware,
            O,
        >,
    > {
        ComposedHelper::new(self.executor.with_middleware(middleware))
    }

    fn with_middleware<M>(
        self,
        middleware: M,
    ) -> ComposedHelper<O, crate::middleware::ext::MiddlewareExecutor<E, M, O>>
    where
        M: Middleware<O> + Send + Sync + std::fmt::Debug + 'static,
    {
        ComposedHelper::new(self.executor.with_middleware(middleware))
    }

    fn executor(&self) -> &E {
        &self.executor
    }
}

/// Filesystem operation helper builder.
///
/// Use this to create composable file operation pipelines.
///
/// # Example
///
/// ```rust,no_run
/// use airssys_osl::helpers::composition::*;
/// use airssys_osl::middleware::security::SecurityMiddleware;
///
/// # async fn example() -> airssys_osl::core::result::OSResult<()> {
/// let reader = FileHelper::builder()
///     .with_security(SecurityMiddleware::default());
///
/// let data = reader.read("/etc/hosts", "admin").await?;
/// # Ok(())
/// # }
/// ```
pub struct FileHelper;

impl FileHelper {
    /// Create a new file helper builder.
    ///
    /// Returns a `ComposedHelper` with a fresh `FilesystemExecutor`.
    /// Chain middleware using `.with_security()`, `.with_middleware()`, etc.
    ///
    /// This is a builder method that returns a composable helper pipeline,
    /// not a `FileHelper` instance.
    pub fn builder() -> ComposedHelper<
        crate::operations::filesystem::FileReadOperation,
        crate::executors::filesystem::FilesystemExecutor,
    > {
        ComposedHelper::new(crate::executors::filesystem::FilesystemExecutor::new())
    }
}

/// Process operation helper builder.
///
/// Use this to create composable process operation pipelines.
///
/// # Example
///
/// ```rust,no_run
/// use airssys_osl::helpers::composition::*;
/// use airssys_osl::middleware::security::SecurityMiddleware;
///
/// # async fn example() -> airssys_osl::core::result::OSResult<()> {
/// let spawner = ProcessHelper::builder()
///     .with_security(SecurityMiddleware::default());
///
/// let pid = spawner.spawn("ls", vec!["-la".to_string()], "admin").await?;
/// # Ok(())
/// # }
/// ```
pub struct ProcessHelper;

impl ProcessHelper {
    /// Create a new process helper builder.
    ///
    /// Returns a `ComposedHelper` with a fresh `ProcessExecutor`.
    ///
    /// This is a builder method that returns a composable helper pipeline,
    /// not a `ProcessHelper` instance.
    pub fn builder() -> ComposedHelper<
        crate::operations::process::ProcessSpawnOperation,
        crate::executors::process::ProcessExecutor,
    > {
        ComposedHelper::new(crate::executors::process::ProcessExecutor::new(
            "composition_helper",
        ))
    }
}

/// Network operation helper builder.
///
/// Use this to create composable network operation pipelines.
///
/// # Example
///
/// ```rust,no_run
/// use airssys_osl::helpers::composition::*;
/// use airssys_osl::middleware::security::SecurityMiddleware;
///
/// # async fn example() -> airssys_osl::core::result::OSResult<()> {
/// let connector = NetworkHelper::builder()
///     .with_security(SecurityMiddleware::default());
///
/// let socket = connector.connect("127.0.0.1:8080", "admin").await?;
/// # Ok(())
/// # }
/// ```
pub struct NetworkHelper;

impl NetworkHelper {
    /// Create a new network helper builder.
    ///
    /// Returns a `ComposedHelper` with a fresh `NetworkExecutor`.
    ///
    /// This is a builder method that returns a composable helper pipeline,
    /// not a `NetworkHelper` instance.
    pub fn builder() -> ComposedHelper<
        crate::operations::network::NetworkConnectOperation,
        crate::executors::network::NetworkExecutor,
    > {
        ComposedHelper::new(crate::executors::network::NetworkExecutor::new(
            "composition_helper",
        ))
    }
}

// ============================================================================
// Execution Methods - Filesystem Operations
// ============================================================================

/// Execution methods for file read operations.
impl<E> ComposedHelper<crate::operations::filesystem::FileReadOperation, E>
where
    E: OSExecutor<crate::operations::filesystem::FileReadOperation> + Send + Sync + std::fmt::Debug,
{
    /// Read file contents.
    ///
    /// # Arguments
    ///
    /// - `path`: File path to read
    /// - `user`: User identity for security context
    ///
    /// # Returns
    ///
    /// File contents as bytes
    ///
    /// # Errors
    ///
    /// - `OSError::SecurityViolation`: Security policy denied access
    /// - `OSError::NotFound`: File does not exist
    /// - `OSError::PermissionDenied`: OS-level permission denied
    ///
    /// # Example
    ///
    /// ```rust,no_run
    /// use airssys_osl::helpers::composition::*;
    /// use airssys_osl::middleware::security::SecurityMiddleware;
    ///
    /// # async fn example() -> airssys_osl::core::result::OSResult<()> {
    /// let reader = FileHelper::builder()
    ///     .with_security(SecurityMiddleware::default());
    ///
    /// let data = reader.read("/etc/hosts", "admin").await?;
    /// println!("Read {} bytes", data.len());
    /// # Ok(())
    /// # }
    /// ```
    pub async fn read<P: AsRef<std::path::Path>>(
        &self,
        path: P,
        user: impl Into<String>,
    ) -> OSResult<Vec<u8>> {
        use crate::core::context::{ExecutionContext, SecurityContext};

        let path_str = path.as_ref().display().to_string();
        let operation = crate::operations::filesystem::FileReadOperation::new(path_str);
        let context = ExecutionContext::new(SecurityContext::new(user.into()));
        let result = self.executor.execute(operation, &context).await?;
        Ok(result.output)
    }
}

/// Execution methods for file write operations.
impl<E> ComposedHelper<crate::operations::filesystem::FileWriteOperation, E>
where
    E: OSExecutor<crate::operations::filesystem::FileWriteOperation>
        + Send
        + Sync
        + std::fmt::Debug,
{
    /// Write data to file.
    ///
    /// # Arguments
    ///
    /// - `path`: File path to write
    /// - `data`: Data to write (bytes)
    /// - `user`: User identity for security context
    ///
    /// # Returns
    ///
    /// Number of bytes written (as bytes)
    ///
    /// # Errors
    ///
    /// - `OSError::SecurityViolation`: Security policy denied access
    /// - `OSError::PermissionDenied`: OS-level permission denied
    /// - `OSError::IoError`: I/O error during write
    ///
    /// # Example
    ///
    /// ```rust,no_run
    /// use airssys_osl::helpers::composition::*;
    /// use airssys_osl::middleware::security::SecurityMiddleware;
    ///
    /// # async fn example() -> airssys_osl::core::result::OSResult<()> {
    /// let writer = FileHelper::builder()
    ///     .with_security(SecurityMiddleware::default());
    ///
    /// let result = writer.write("/tmp/test.txt", b"data".to_vec(), "admin").await?;
    /// # Ok(())
    /// # }
    /// ```
    pub async fn write<P: AsRef<std::path::Path>>(
        &self,
        path: P,
        data: Vec<u8>,
        user: impl Into<String>,
    ) -> OSResult<Vec<u8>> {
        use crate::core::context::{ExecutionContext, SecurityContext};

        let path_str = path.as_ref().display().to_string();
        let operation = crate::operations::filesystem::FileWriteOperation::new(path_str, data);
        let context = ExecutionContext::new(SecurityContext::new(user.into()));
        let result = self.executor.execute(operation, &context).await?;
        Ok(result.output)
    }
}

/// Execution methods for directory creation operations.
impl<E> ComposedHelper<crate::operations::filesystem::DirectoryCreateOperation, E>
where
    E: OSExecutor<crate::operations::filesystem::DirectoryCreateOperation>
        + Send
        + Sync
        + std::fmt::Debug,
{
    /// Create a new directory.
    ///
    /// # Arguments
    ///
    /// - `path`: Directory path to create
    /// - `user`: User identity for security context
    ///
    /// # Returns
    ///
    /// Unit on successful creation
    ///
    /// # Errors
    ///
    /// - `OSError::SecurityViolation`: Security policy denied access
    /// - `OSError::AlreadyExists`: Directory already exists
    /// - `OSError::PermissionDenied`: OS-level permission denied
    ///
    /// # Example
    ///
    /// ```rust,no_run
    /// use airssys_osl::helpers::composition::*;
    /// use airssys_osl::middleware::security::SecurityMiddleware;
    ///
    /// # async fn example() -> airssys_osl::core::result::OSResult<()> {
    /// let creator = FileHelper::builder()
    ///     .with_security(SecurityMiddleware::default());
    ///
    /// creator.create("/tmp/newdir", "admin").await?;
    /// # Ok(())
    /// # }
    /// ```
    pub async fn create<P: AsRef<std::path::Path>>(
        &self,
        path: P,
        user: impl Into<String>,
    ) -> OSResult<()> {
        use crate::core::context::{ExecutionContext, SecurityContext};

        let path_str = path.as_ref().display().to_string();
        let operation = crate::operations::filesystem::DirectoryCreateOperation::new(path_str);
        let context = ExecutionContext::new(SecurityContext::new(user.into()));
        self.executor.execute(operation, &context).await?;
        Ok(())
    }
}

/// Execution methods for file deletion operations.
impl<E> ComposedHelper<crate::operations::filesystem::FileDeleteOperation, E>
where
    E: OSExecutor<crate::operations::filesystem::FileDeleteOperation>
        + Send
        + Sync
        + std::fmt::Debug,
{
    /// Delete a file.
    ///
    /// # Arguments
    ///
    /// - `path`: File path to delete
    /// - `user`: User identity for security context
    ///
    /// # Returns
    ///
    /// Unit on successful deletion
    ///
    /// # Errors
    ///
    /// - `OSError::SecurityViolation`: Security policy denied access
    /// - `OSError::NotFound`: File does not exist
    /// - `OSError::PermissionDenied`: OS-level permission denied
    ///
    /// # Example
    ///
    /// ```rust,no_run
    /// use airssys_osl::helpers::composition::*;
    /// use airssys_osl::middleware::security::SecurityMiddleware;
    ///
    /// # async fn example() -> airssys_osl::core::result::OSResult<()> {
    /// let deleter = FileHelper::builder()
    ///     .with_security(SecurityMiddleware::default());
    ///
    /// deleter.delete("/tmp/oldfile.txt", "admin").await?;
    /// # Ok(())
    /// # }
    /// ```
    pub async fn delete<P: AsRef<std::path::Path>>(
        &self,
        path: P,
        user: impl Into<String>,
    ) -> OSResult<()> {
        use crate::core::context::{ExecutionContext, SecurityContext};

        let path_str = path.as_ref().display().to_string();
        let operation = crate::operations::filesystem::FileDeleteOperation::new(path_str);
        let context = ExecutionContext::new(SecurityContext::new(user.into()));
        self.executor.execute(operation, &context).await?;
        Ok(())
    }
}

// ============================================================================
// Execution Methods - Process Operations
// ============================================================================

/// Execution methods for process spawn operations.
impl<E> ComposedHelper<crate::operations::process::ProcessSpawnOperation, E>
where
    E: OSExecutor<crate::operations::process::ProcessSpawnOperation>
        + Send
        + Sync
        + std::fmt::Debug,
{
    /// Spawn a new process.
    ///
    /// # Arguments
    ///
    /// - `command`: Command to execute
    /// - `args`: Command arguments
    /// - `user`: User identity for security context
    ///
    /// # Returns
    ///
    /// Process ID as bytes (use `String::from_utf8_lossy()` to convert)
    ///
    /// # Errors
    ///
    /// - `OSError::SecurityViolation`: Security policy denied access
    /// - `OSError::NotFound`: Command not found
    /// - `OSError::PermissionDenied`: OS-level permission denied
    ///
    /// # Example
    ///
    /// ```rust,no_run
    /// use airssys_osl::helpers::composition::*;
    /// use airssys_osl::middleware::security::SecurityMiddleware;
    ///
    /// # async fn example() -> airssys_osl::core::result::OSResult<()> {
    /// let spawner = ProcessHelper::builder()
    ///     .with_security(SecurityMiddleware::default());
    ///
    /// let pid = spawner.spawn("ls", vec!["-la".to_string()], "admin").await?;
    /// println!("Spawned process PID: {}", String::from_utf8_lossy(&pid));
    /// # Ok(())
    /// # }
    /// ```
    pub async fn spawn(
        &self,
        command: impl Into<String>,
        args: Vec<String>,
        user: impl Into<String>,
    ) -> OSResult<Vec<u8>> {
        use crate::core::context::{ExecutionContext, SecurityContext};

        let operation =
            crate::operations::process::ProcessSpawnOperation::new(command.into()).with_args(args);
        let context = ExecutionContext::new(SecurityContext::new(user.into()));
        let result = self.executor.execute(operation, &context).await?;
        Ok(result.output)
    }
}

/// Execution methods for process kill operations.
impl<E> ComposedHelper<crate::operations::process::ProcessKillOperation, E>
where
    E: OSExecutor<crate::operations::process::ProcessKillOperation> + Send + Sync + std::fmt::Debug,
{
    /// Kill a process by PID.
    ///
    /// # Arguments
    ///
    /// - `pid`: Process ID to kill
    /// - `user`: User identity for security context
    ///
    /// # Returns
    ///
    /// Unit on successful termination
    ///
    /// # Errors
    ///
    /// - `OSError::SecurityViolation`: Security policy denied access
    /// - `OSError::NotFound`: Process not found
    /// - `OSError::PermissionDenied`: OS-level permission denied
    ///
    /// # Example
    ///
    /// ```rust,no_run
    /// use airssys_osl::helpers::composition::*;
    /// use airssys_osl::middleware::security::SecurityMiddleware;
    ///
    /// # async fn example() -> airssys_osl::core::result::OSResult<()> {
    /// let killer = ProcessHelper::builder()
    ///     .with_security(SecurityMiddleware::default());
    ///
    /// killer.kill(1234, "admin").await?;
    /// # Ok(())
    /// # }
    /// ```
    pub async fn kill(&self, pid: u32, user: impl Into<String>) -> OSResult<()> {
        use crate::core::context::{ExecutionContext, SecurityContext};

        let operation = crate::operations::process::ProcessKillOperation::new(pid);
        let context = ExecutionContext::new(SecurityContext::new(user.into()));
        self.executor.execute(operation, &context).await?;
        Ok(())
    }
}

/// Execution methods for process signal operations.
impl<E> ComposedHelper<crate::operations::process::ProcessSignalOperation, E>
where
    E: OSExecutor<crate::operations::process::ProcessSignalOperation>
        + Send
        + Sync
        + std::fmt::Debug,
{
    /// Send a signal to a process.
    ///
    /// # Arguments
    ///
    /// - `pid`: Process ID to signal
    /// - `signal`: Signal number to send
    /// - `user`: User identity for security context
    ///
    /// # Returns
    ///
    /// Unit on successful signal delivery
    ///
    /// # Errors
    ///
    /// - `OSError::SecurityViolation`: Security policy denied access
    /// - `OSError::NotFound`: Process not found
    /// - `OSError::PermissionDenied`: OS-level permission denied
    ///
    /// # Example
    ///
    /// ```rust,no_run
    /// use airssys_osl::helpers::composition::*;
    /// use airssys_osl::middleware::security::SecurityMiddleware;
    ///
    /// # async fn example() -> airssys_osl::core::result::OSResult<()> {
    /// let signaler = ProcessHelper::builder()
    ///     .with_security(SecurityMiddleware::default());
    ///
    /// signaler.send_signal(1234, 15, "admin").await?; // SIGTERM
    /// # Ok(())
    /// # }
    /// ```
    pub async fn send_signal(
        &self,
        pid: u32,
        signal: i32,
        user: impl Into<String>,
    ) -> OSResult<()> {
        use crate::core::context::{ExecutionContext, SecurityContext};

        let operation = crate::operations::process::ProcessSignalOperation::new(pid, signal);
        let context = ExecutionContext::new(SecurityContext::new(user.into()));
        self.executor.execute(operation, &context).await?;
        Ok(())
    }
}

// ============================================================================
// Execution Methods - Network Operations
// ============================================================================

/// Execution methods for network connect operations.
impl<E> ComposedHelper<crate::operations::network::NetworkConnectOperation, E>
where
    E: OSExecutor<crate::operations::network::NetworkConnectOperation>
        + Send
        + Sync
        + std::fmt::Debug,
{
    /// Connect to a remote network endpoint.
    ///
    /// # Arguments
    ///
    /// - `address`: Remote address to connect to (e.g., "127.0.0.1:8080")
    /// - `user`: User identity for security context
    ///
    /// # Returns
    ///
    /// Socket file descriptor as bytes
    ///
    /// # Errors
    ///
    /// - `OSError::SecurityViolation`: Security policy denied access
    /// - `OSError::ConnectionRefused`: Connection refused by remote
    /// - `OSError::NetworkError`: Network error during connection
    ///
    /// # Example
    ///
    /// ```rust,no_run
    /// use airssys_osl::helpers::composition::*;
    /// use airssys_osl::middleware::security::SecurityMiddleware;
    ///
    /// # async fn example() -> airssys_osl::core::result::OSResult<()> {
    /// let connector = NetworkHelper::builder()
    ///     .with_security(SecurityMiddleware::default());
    ///
    /// let socket = connector.connect("127.0.0.1:8080", "admin").await?;
    /// # Ok(())
    /// # }
    /// ```
    pub async fn connect(
        &self,
        address: impl Into<String>,
        user: impl Into<String>,
    ) -> OSResult<Vec<u8>> {
        use crate::core::context::{ExecutionContext, SecurityContext};

        let operation = crate::operations::network::NetworkConnectOperation::new(address.into());
        let context = ExecutionContext::new(SecurityContext::new(user.into()));
        let result = self.executor.execute(operation, &context).await?;
        Ok(result.output)
    }
}

/// Execution methods for network listen operations.
impl<E> ComposedHelper<crate::operations::network::NetworkListenOperation, E>
where
    E: OSExecutor<crate::operations::network::NetworkListenOperation>
        + Send
        + Sync
        + std::fmt::Debug,
{
    /// Listen on a local network endpoint.
    ///
    /// # Arguments
    ///
    /// - `address`: Local address to bind to (e.g., "0.0.0.0:8080")
    /// - `user`: User identity for security context
    ///
    /// # Returns
    ///
    /// Listening socket file descriptor as bytes
    ///
    /// # Errors
    ///
    /// - `OSError::SecurityViolation`: Security policy denied access
    /// - `OSError::AddressInUse`: Address already in use
    /// - `OSError::NetworkError`: Network error during bind
    ///
    /// # Example
    ///
    /// ```rust,no_run
    /// use airssys_osl::helpers::composition::*;
    /// use airssys_osl::middleware::security::SecurityMiddleware;
    ///
    /// # async fn example() -> airssys_osl::core::result::OSResult<()> {
    /// let listener = NetworkHelper::builder()
    ///     .with_security(SecurityMiddleware::default());
    ///
    /// let socket = listener.listen("0.0.0.0:8080", "admin").await?;
    /// # Ok(())
    /// # }
    /// ```
    pub async fn listen(
        &self,
        address: impl Into<String>,
        user: impl Into<String>,
    ) -> OSResult<Vec<u8>> {
        use crate::core::context::{ExecutionContext, SecurityContext};

        let operation = crate::operations::network::NetworkListenOperation::new(address.into());
        let context = ExecutionContext::new(SecurityContext::new(user.into()));
        let result = self.executor.execute(operation, &context).await?;
        Ok(result.output)
    }
}

/// Execution methods for network socket creation operations.
impl<E> ComposedHelper<crate::operations::network::NetworkSocketOperation, E>
where
    E: OSExecutor<crate::operations::network::NetworkSocketOperation>
        + Send
        + Sync
        + std::fmt::Debug,
{
    /// Create a network socket.
    ///
    /// # Arguments
    ///
    /// - `socket_type`: Socket type ("tcp", "udp", etc.)
    /// - `user`: User identity for security context
    ///
    /// # Returns
    ///
    /// Created socket file descriptor as bytes
    ///
    /// # Errors
    ///
    /// - `OSError::SecurityViolation`: Security policy denied access
    /// - `OSError::InvalidInput`: Invalid socket type
    /// - `OSError::NetworkError`: Error creating socket
    ///
    /// # Example
    ///
    /// ```rust,no_run
    /// use airssys_osl::helpers::composition::*;
    /// use airssys_osl::middleware::security::SecurityMiddleware;
    ///
    /// # async fn example() -> airssys_osl::core::result::OSResult<()> {
    /// let socket_creator = NetworkHelper::builder()
    ///     .with_security(SecurityMiddleware::default());
    ///
    /// let socket = socket_creator.create_socket("udp", "admin").await?;
    /// # Ok(())
    /// # }
    /// ```
    pub async fn create_socket(
        &self,
        socket_type: impl Into<String>,
        user: impl Into<String>,
    ) -> OSResult<Vec<u8>> {
        use crate::core::context::{ExecutionContext, SecurityContext};

        let operation = crate::operations::network::NetworkSocketOperation::new(socket_type.into());
        let context = ExecutionContext::new(SecurityContext::new(user.into()));
        let result = self.executor.execute(operation, &context).await?;
        Ok(result.output)
    }
}
