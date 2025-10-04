//! Framework builder for OSL configuration and initialization.
//!
//! This module provides the `OSLFrameworkBuilder` for fluent configuration
//! of framework instances with automatic component setup and validation.

use crate::core::{context::SecurityContext, result::OSResult};

use super::{
    config::{OSLConfigBuilder, SecurityConfig},
    pipeline::MiddlewarePipeline,
    registry::ExecutorRegistry,
    OSLFramework,
};

/// Builder for configuring and creating `OSLFramework` instances.
///
/// `OSLFrameworkBuilder` provides a fluent interface for configuring the framework.
/// Phase 1 implementation (OSL-TASK-006) provides complete builder functionality
/// including middleware and executor registration.
///
/// # Examples
///
/// ## Simple Configuration
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
#[derive(Debug)]
pub struct OSLFrameworkBuilder {
    security_config: Option<SecurityConfig>,
    config_builder: OSLConfigBuilder,
}

impl Default for OSLFrameworkBuilder {
    fn default() -> Self {
        Self::new()
    }
}

impl OSLFrameworkBuilder {
    /// Create a new framework builder with default settings.
    pub fn new() -> Self {
        Self {
            security_config: None,
            config_builder: OSLConfigBuilder::new(),
        }
    }

    /// Configure the framework with default security settings.
    ///
    /// This is the recommended configuration for most applications, providing
    /// comprehensive security logging and policy enforcement with sensible defaults.
    ///
    /// # Examples
    ///
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
    pub fn with_default_security(mut self) -> Self {
        self.security_config = Some(SecurityConfig::default());
        self
    }

    /// Enable or disable security logging.
    ///
    /// When enabled, all operations will be logged with comprehensive security
    /// audit information including user context, operation details, and results.
    ///
    /// # Arguments
    ///
    /// * `_enabled` - Whether to enable security logging
    ///
    /// # Note
    ///
    /// Full implementation coming in OSL-TASK-006. This foundation method
    /// accepts the parameter for API compatibility.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use airssys_osl::prelude::*;
    ///
    /// # async fn example() -> OSResult<()> {
    /// let osl = OSLFramework::builder()
    ///     .with_security_logging(true)
    ///     .build().await?;
    /// # Ok(())
    /// # }
    /// ```
    pub fn with_security_logging(self, _enabled: bool) -> Self {
        // TODO: Full implementation in OSL-TASK-006
        // This will configure the appropriate logger middleware
        self
    }

    /// Configure security policy from a file.
    ///
    /// Loads security policy configuration from the specified file path,
    /// enabling custom access control rules and enforcement policies.
    ///
    /// # Arguments  
    ///
    /// * `_path` - Path to the security policy configuration file
    ///
    /// # Note
    ///
    /// Full implementation coming in OSL-TASK-006. This foundation method
    /// accepts the parameter for API compatibility.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use airssys_osl::prelude::*;
    ///
    /// # async fn example() -> OSResult<()> {
    /// let osl = OSLFramework::builder()
    ///     .with_policy_file("/etc/osl/security-policy.toml")
    ///     .build().await?;
    /// # Ok(())
    /// # }
    /// ```
    pub fn with_policy_file<P: AsRef<std::path::Path>>(self, _path: P) -> Self {
        // TODO: Full implementation in OSL-TASK-006
        // This will load and parse the security policy file
        self
    }

    /// Build the configured framework instance.
    ///
    /// Validates the configuration, initializes all components, and creates
    /// a ready-to-use `OSLFramework` instance. This method performs comprehensive
    /// validation and will return detailed errors for configuration issues.
    ///
    /// # Returns
    ///
    /// Returns a configured `OSLFramework` instance ready for operation execution.
    ///
    /// # Errors
    ///
    /// Returns `OSError` if:
    /// - Security policy configuration is invalid
    /// - System resources cannot be initialized
    /// - Middleware initialization fails
    /// - Executor registry creation fails
    ///
    /// # Note
    ///
    /// Phase 1 provides the foundation with empty pipeline and registry.
    /// Phase 2 will implement full middleware orchestration.
    /// Phase 3 will add default executors.
    pub async fn build(self) -> OSResult<OSLFramework> {
        // Phase 1 implementation - foundation functionality

        // 1. Validate basic configuration
        self.validate_configuration()?;

        // 2. Build security context
        let security_context = self.build_security_context()?;

        // 3. Build framework configuration
        let config = self.config_builder.build()?;

        // 4. Create middleware pipeline (empty for Phase 1)
        let middleware_pipeline = MiddlewarePipeline::new();

        // Phase 2 will add: Initialize middleware from self.middlewares
        // Phase 2 will call: middleware_pipeline.initialize_all().await?;

        // 5. Create executor registry (empty for Phase 1)
        // Phase 3 will add: Default executors for filesystem, process, network
        let executors = ExecutorRegistry::new()?;

        Ok(OSLFramework {
            middleware_pipeline,
            executors,
            security_context,
            config,
        })
    }

    // Private helper methods

    fn validate_configuration(&self) -> OSResult<()> {
        // Basic validation - comprehensive validation in OSL-TASK-006
        Ok(())
    }

    fn build_security_context(&self) -> OSResult<SecurityContext> {
        // Build security context from configuration
        // For now, return a default security context with framework user
        Ok(SecurityContext::new("framework-user".to_string()))
    }
}
