//! Generic logger middleware implementation.
//!
//! This module contains the core LoggerMiddleware implementation that
//! integrates with the middleware pipeline to provide activity logging.

// Layer 1: Standard library imports
use std::sync::Arc;

// Layer 2: Third-party crate imports
// (imports will be added in Phase 3)

// Layer 3: Internal module imports
use super::activity::ActivityLogger;
use super::config::LoggerConfig;

/// Generic logger middleware for activity logging and audit trails.
///
/// This middleware logs operation execution before and after processing,
/// providing comprehensive audit trails for security and debugging.
/// Uses generic constraints instead of dynamic dispatch for zero-cost abstractions.
///
/// # Type Parameters
///
/// - `L`: The concrete logger implementation that handles log output
///
/// # Examples
///
/// ```rust,ignore
/// use airssys_osl::middleware::logger::{LoggerMiddleware, ConsoleActivityLogger};
///
/// let logger = ConsoleActivityLogger::new();
/// let middleware = LoggerMiddleware::new(logger, LoggerConfig::default());
/// ```
///
/// Implementation will be completed in Phase 3.
#[derive(Debug)]
pub struct LoggerMiddleware<L: ActivityLogger> {
    logger: Arc<L>,
    config: LoggerConfig,
}

impl<L: ActivityLogger> LoggerMiddleware<L> {
    /// Create a new logger middleware with the specified logger and configuration.
    ///
    /// # Examples
    ///
    /// ```rust,ignore
    /// use airssys_osl::middleware::logger::{LoggerMiddleware, ConsoleActivityLogger, LoggerConfig};
    ///
    /// let logger = ConsoleActivityLogger::new();
    /// let config = LoggerConfig::development();
    /// let middleware = LoggerMiddleware::new(logger, config);
    /// ```
    pub fn new(logger: L, config: LoggerConfig) -> Self {
        Self {
            logger: Arc::new(logger),
            config,
        }
    }

    /// Create a new logger middleware with default configuration.
    ///
    /// # Examples
    ///
    /// ```rust,ignore
    /// use airssys_osl::middleware::logger::{LoggerMiddleware, ConsoleActivityLogger};
    ///
    /// let logger = ConsoleActivityLogger::new();
    /// let middleware = LoggerMiddleware::with_default_config(logger);
    /// ```
    pub fn with_default_config(logger: L) -> Self {
        Self::new(logger, LoggerConfig::default())
    }

    /// Get a reference to the logger configuration.
    pub fn config(&self) -> &LoggerConfig {
        &self.config
    }

    /// Get a reference to the underlying logger.
    pub fn logger(&self) -> &Arc<L> {
        &self.logger
    }
}

// Implementation of Middleware<O> trait will be added in Phase 3
// when we integrate with the core middleware system
