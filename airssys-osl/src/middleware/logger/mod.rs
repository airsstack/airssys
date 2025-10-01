//! Logger middleware for comprehensive activity logging and audit trails.
//!
//! This module provides a generic logger middleware that can work with any
//! implementation of the `ActivityLogger` trait. It supports multiple output
//! formats and destinations for comprehensive audit trails.
//!
//! # Design Principles
//!
//! - **Generic-first design**: Uses `LoggerMiddleware<L: ActivityLogger>`
//!   instead of dynamic dispatch for zero-cost abstractions
//! - **Separated concerns**: Pure logging trait with specific implementations
//! - **User-controlled composition**: Library doesn't impose usage patterns
//!
//! # Quick Start
//!
//! ```rust,ignore
//! use airssys_osl::middleware::logger::{ConsoleActivityLogger, LoggerMiddleware};
//!
//! // Create a console logger with pretty printing
//! let logger = ConsoleActivityLogger::new().with_pretty_print(true);
//! let middleware = LoggerMiddleware::with_default_config(logger);
//! ```
//!
//! # Available Loggers
//!
//! - **[`ConsoleActivityLogger`]** - Console output with optional pretty printing
//! - **[`FileActivityLogger`]** - Async file-based logging with rotation support
//! - **[`TracingActivityLogger`]** - Integration with the tracing ecosystem
//!
//! # Core Types
//!
//! - **[`ActivityLog`]** - Structured log entry with operation metadata
//! - **[`ActivityLogger`]** - Core trait for pluggable log destinations
//! - **[`LoggerMiddleware`]** - Generic middleware implementation
//! - **[`LoggerConfig`]** - Configuration for logging behavior

// Layer 1: Standard library imports
// (imports will be added in Phase 2)

// Layer 2: Third-party crate imports
// (imports will be added in Phase 2)

// Layer 3: Internal module imports
// (imports will be added in Phase 2)

// Public API exports
pub use activity::{ActivityLog, ActivityLogger};
pub use config::{LogFormat, LogLevel, LoggerConfig};
pub use error::LogError;
pub use formatter::LogFormatter;
pub use middleware::LoggerMiddleware;

// Concrete logger implementations (will be uncommented in Phase 4)
// pub use loggers::{ConsoleActivityLogger, FileActivityLogger, TracingActivityLogger};

// Internal modules (following §4.3 - mod.rs only has declarations and re-exports)
mod activity;
mod config;
mod error;
mod formatter;
pub mod loggers;
mod middleware;
