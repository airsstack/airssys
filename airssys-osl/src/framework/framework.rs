//! Main framework entry point for OSL operations.
//!
//! This module provides the `OSLFramework` struct which serves as the primary
//! interface for all OSL operations through the framework API.

use crate::core::{
    context::SecurityContext,
};

use super::{
    builder::OSLFrameworkBuilder,
    config::OSLConfig,
};

/// Main framework entry point for high-level OSL operations.
///
/// `OSLFramework` provides an ergonomic interface over the core OSL primitives.
/// This is the foundation implementation (OSL-TASK-005) with full functionality
/// coming in OSL-TASK-006.
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
#[derive(Debug)]
pub struct OSLFramework {
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
}