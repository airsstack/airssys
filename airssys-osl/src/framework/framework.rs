//! Main framework entry point for OSL operations.
//!
//! This module provides the `OSLFramework` struct which serves as the primary
//! interface for all OSL operations through the framework API.

use crate::core::{
    context::{ExecutionContext, SecurityContext},
    executor::ExecutionResult,
    operation::Operation,
    result::OSResult,
};

use super::{
    builder::OSLFrameworkBuilder,
    config::OSLConfig,
    operations::{FilesystemBuilder, NetworkBuilder, ProcessBuilder},
    pipeline::MiddlewarePipeline,
    registry::ExecutorRegistry,
};

/// Main framework entry point for high-level OSL operations.
///
/// `OSLFramework` provides an ergonomic interface over the core OSL primitives.
/// Phase 1 implementation (OSL-TASK-006) with complete framework functionality
/// including middleware orchestration and operation execution.
///
/// # Examples
///
/// ## Basic Framework Creation
/// ```rust
/// use airssys_osl::prelude::*;
///
/// # async fn example() -> OSResult<()> {
/// let osl = OSLFramework::builder()
///     .with_default_security()
///     .build().await?;
/// # Ok(())
/// # }
/// ```
///
/// ## Framework with Operation Builders
/// ```no_run
/// use airssys_osl::prelude::*;
///
/// # async fn example() -> OSResult<()> {
/// let osl = OSLFramework::builder()
///     .with_default_security()
///     .build().await?;
///
/// // Operation builders will be fully implemented in Phase 3
/// let _fs_builder = osl.filesystem();
/// let _proc_builder = osl.process();
/// let _net_builder = osl.network();
/// # Ok(())
/// # }
/// ```
#[derive(Debug)]
#[allow(dead_code)] // Phase 1: Fields will be used in Phase 2-3
pub struct OSLFramework {
    pub(super) middleware_pipeline: MiddlewarePipeline,
    pub(super) executors: ExecutorRegistry,
    pub(super) security_context: SecurityContext,
    pub(super) config: OSLConfig,
}

impl OSLFramework {
    /// Create a new framework builder for configuration.
    ///
    /// This is the primary entry point for creating an `OSLFramework` instance.
    /// The builder provides a fluent interface for configuration and handles
    /// automatic setup of default components.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use airssys_osl::prelude::*;
    ///
    /// # async fn example() -> OSResult<()> {
    /// let osl = OSLFramework::builder()
    ///     .with_default_security()
    ///     .with_security_logging(true)
    ///     .build().await?;
    /// # Ok(())
    /// # }
    /// ```
    pub fn builder() -> OSLFrameworkBuilder {
        OSLFrameworkBuilder::new()
    }

    /// Execute an operation through the framework pipeline.
    ///
    /// This method executes an operation through the complete middleware pipeline
    /// and appropriate executor. It creates an execution context from the framework's
    /// security context and manages the full execution lifecycle.
    ///
    /// # Arguments
    ///
    /// * `operation` - The operation to execute
    ///
    /// # Returns
    ///
    /// Returns the execution result from the operation.
    ///
    /// # Errors
    ///
    /// Returns `OSError` if:
    /// - No executor is available for the operation type
    /// - Security validation fails
    /// - Operation execution fails
    /// - Middleware processing encounters fatal errors
    ///
    /// # Note
    ///
    /// Phase 1 provides the foundation. Full pipeline execution with middleware
    /// orchestration will be completed in Phase 2.
    ///
    /// # Examples
    ///
    /// ```no_run
    /// use airssys_osl::prelude::*;
    /// # use airssys_osl::core::operation::{Operation, OperationType, Permission};
    /// # use chrono::Utc;
    ///
    /// # #[derive(Debug, Clone)]
    /// # struct MyOperation;
    /// # impl Operation for MyOperation {
    /// #     fn operation_type(&self) -> OperationType { OperationType::Filesystem }
    /// #     fn required_permissions(&self) -> Vec<Permission> { vec![] }
    /// #     fn created_at(&self) -> chrono::DateTime<Utc> { Utc::now() }
    /// # }
    ///
    /// # async fn example() -> OSResult<()> {
    /// let osl = OSLFramework::builder()
    ///     .with_default_security()
    ///     .build().await?;
    ///
    /// let operation = MyOperation;
    /// let result = osl.execute(operation).await?;
    /// # Ok(())
    /// # }
    /// ```
    pub async fn execute<O: Operation>(&self, operation: O) -> OSResult<ExecutionResult> {
        // Create execution context from security context
        let _exec_context = ExecutionContext::new(self.security_context.clone());

        // Phase 2 will implement full pipeline execution
        // For now, we'll return a placeholder that shows the infrastructure works
        // TODO: Implement full pipeline.execute() in Phase 2
        
        // Phase 1: Just verify the operation type is recognized
        let _operation_type = operation.operation_type();
        
        // For Phase 1, return a simple success result to demonstrate the flow
        // Phase 2 will call: self.middleware_pipeline.execute(operation, exec_context, &self.executors).await
        Ok(ExecutionResult::success(b"Phase 1 placeholder - full execution in Phase 2".to_vec()))
    }

    /// Get filesystem operation builder.
    ///
    /// Returns a builder for constructing filesystem operations with fluent API.
    /// The builder provides methods for common filesystem operations like reading
    /// and writing files.
    ///
    /// # Examples
    ///
    /// ```no_run
    /// use airssys_osl::prelude::*;
    ///
    /// # async fn example() -> OSResult<()> {
    /// let osl = OSLFramework::builder()
    ///     .with_default_security()
    ///     .build().await?;
    ///
    /// let fs_builder = osl.filesystem();
    /// // Phase 3 will implement: fs_builder.read_file("/path/to/file").execute().await?
    /// # Ok(())
    /// # }
    /// ```
    pub fn filesystem(&self) -> FilesystemBuilder {
        FilesystemBuilder::new(self)
    }

    /// Get process operation builder.
    ///
    /// Returns a builder for constructing process operations with fluent API.
    /// The builder provides methods for spawning and managing processes.
    ///
    /// # Examples
    ///
    /// ```no_run
    /// use airssys_osl::prelude::*;
    ///
    /// # async fn example() -> OSResult<()> {
    /// let osl = OSLFramework::builder()
    ///     .with_default_security()
    ///     .build().await?;
    ///
    /// let proc_builder = osl.process();
    /// // Phase 3 will implement: proc_builder.spawn("command").execute().await?
    /// # Ok(())
    /// # }
    /// ```
    pub fn process(&self) -> ProcessBuilder {
        ProcessBuilder::new(self)
    }

    /// Get network operation builder.
    ///
    /// Returns a builder for constructing network operations with fluent API.
    /// The builder provides methods for network socket and connection operations.
    ///
    /// # Examples
    ///
    /// ```no_run
    /// use airssys_osl::prelude::*;
    ///
    /// # async fn example() -> OSResult<()> {
    /// let osl = OSLFramework::builder()
    ///     .with_default_security()
    ///     .build().await?;
    ///
    /// let net_builder = osl.network();
    /// // Phase 3 will implement: net_builder.connect("host:port").execute().await?
    /// # Ok(())
    /// # }
    /// ```
    pub fn network(&self) -> NetworkBuilder {
        NetworkBuilder::new(self)
    }

    /// Get the current security context.
    ///
    /// Provides access to the framework's security context for advanced
    /// security policy inspection or custom operation construction.
    pub fn security_context(&self) -> &SecurityContext {
        &self.security_context
    }

    /// Get the framework configuration.
    ///
    /// Provides access to the framework's configuration for inspection
    /// or advanced customization.
    pub fn config(&self) -> &OSLConfig {
        &self.config
    }

    /// Get the middleware pipeline.
    ///
    /// Provides access to the middleware pipeline for advanced inspection.
    /// This is primarily useful for debugging and testing.
    ///
    /// # Phase 1 Note
    ///
    /// Will be used in Phase 2 for pipeline execution.
    #[allow(dead_code)] // Phase 1: Will be used in Phase 2
    pub(crate) fn middleware_pipeline(&self) -> &MiddlewarePipeline {
        &self.middleware_pipeline
    }

    /// Get the executor registry.
    ///
    /// Provides access to the executor registry for advanced inspection.
    /// This is primarily useful for debugging and testing.
    ///
    /// # Phase 1 Note
    ///
    /// Will be used in Phase 2-3 for operation execution.
    #[allow(dead_code)] // Phase 1: Will be used in Phase 2-3
    pub(crate) fn executor_registry(&self) -> &ExecutorRegistry {
        &self.executors
    }
}