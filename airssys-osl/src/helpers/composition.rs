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
