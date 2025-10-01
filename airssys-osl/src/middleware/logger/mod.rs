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
//! ## Console Logging
//!
//! ```rust
//! use airssys_osl::middleware::logger::{ActivityLog, ConsoleActivityLogger, ActivityLogger};
//! use std::collections::HashMap;
//!
//! # #[tokio::main]
//! # async fn main() -> Result<(), Box<dyn std::error::Error>> {
//! // Create a console logger with JSON format
//! let logger = ConsoleActivityLogger::new()
//!     .with_format(airssys_osl::middleware::logger::LogFormat::Json);
//!
//! // Create an activity log
//! let activity = ActivityLog::new(
//!     "op_123".to_string(),
//!     "file_read".to_string(),
//!     Some("user123".to_string()),
//!     "Success".to_string(),
//!     150,
//! );
//!
//! // Log the activity
//! logger.log_activity(activity).await?;
//! # Ok(())
//! # }
//! ```
//!
//! ## File Logging
//!
//! ```rust,no_run
//! use airssys_osl::middleware::logger::{ActivityLog, FileActivityLogger, ActivityLogger, LogFormat};
//! use std::path::Path;
//!
//! # #[tokio::main]
//! # async fn main() -> Result<(), Box<dyn std::error::Error>> {
//! // Create a file logger with pretty format
//! let logger = FileActivityLogger::new(Path::new("audit.log"))
//!     .await?
//!     .with_format(LogFormat::Pretty);
//!
//! // Create and log activity
//! let activity = ActivityLog::new(
//!     "op_456".to_string(),
//!     "database_query".to_string(),
//!     Some("admin".to_string()),
//!     "Success".to_string(),
//!     300,
//! );
//!
//! logger.log_activity(activity).await?;
//! logger.flush().await?;
//! # Ok(())
//! # }
//! ```
//!
//! ## Tracing Integration
//!
//! ```rust
//! use airssys_osl::middleware::logger::{ActivityLog, TracingActivityLogger, ActivityLogger};
//!
//! # #[tokio::main]
//! # async fn main() -> Result<(), Box<dyn std::error::Error>> {
//! // Initialize tracing subscriber (usually done once at startup)
//! tracing_subscriber::fmt::init();
//!
//! // Create a tracing logger
//! let logger = TracingActivityLogger::new();
//!
//! // Create and log activity
//! let activity = ActivityLog::new(
//!     "op_789".to_string(),
//!     "api_call".to_string(),
//!     Some("service".to_string()),
//!     "Success".to_string(),
//!     100,
//! );
//!
//! logger.log_activity(activity).await?;
//! # Ok(())
//! # }
//! ```
//!
//! ## Middleware Integration
//!
//! ```rust
//! use airssys_osl::middleware::logger::{LoggerMiddleware, ConsoleActivityLogger, LoggerConfig};
//!
//! # fn main() {
//! // Create logger middleware for use in processing pipelines
//! let logger = ConsoleActivityLogger::new();
//! let config = LoggerConfig::default();
//! let middleware = LoggerMiddleware::new(logger, config);
//!
//! // Access the underlying logger
//! let logger_ref = middleware.logger();
//! # }
//! ```
//!
//! # Advanced Usage
//!
//! ## Custom Metadata
//!
//! ```rust
//! use airssys_osl::middleware::logger::ActivityLog;
//! use serde_json::Value;
//!
//! # fn main() {
//! let activity = ActivityLog::new(
//!     "op_123".to_string(),
//!     "file_operation".to_string(),
//!     Some("user123".to_string()),
//!     "Success".to_string(),
//!     200,
//! )
//! .with_metadata("file_path".to_string(), Value::String("/path/to/file".to_string()))
//! .with_metadata("file_size".to_string(), Value::Number(serde_json::Number::from(1024)))
//! .mark_security_relevant();
//! # }
//! ```
//!
//! ## Concurrent Logging
//!
//! ```rust,no_run
//! use airssys_osl::middleware::logger::{ActivityLog, FileActivityLogger, ActivityLogger};
//! use std::sync::Arc;
//! use std::path::Path;
//!
//! # #[tokio::main]
//! # async fn main() -> Result<(), Box<dyn std::error::Error>> {
//! // Create shared logger instance
//! let logger = Arc::new(
//!     FileActivityLogger::new(Path::new("concurrent.log")).await?
//! );
//!
//! // Spawn multiple concurrent logging tasks
//! let mut handles = Vec::new();
//! for i in 0..10 {
//!     let logger_clone = Arc::clone(&logger);
//!     let handle = tokio::spawn(async move {
//!         let activity = ActivityLog::new(
//!             format!("op_{}", i),
//!             "concurrent_operation".to_string(),
//!             Some("system".to_string()),
//!             "Success".to_string(),
//!             50,
//!         );
//!         logger_clone.log_activity(activity).await
//!     });
//!     handles.push(handle);
//! }
//!
//! // Wait for all tasks to complete
//! for handle in handles {
//!     handle.await??;
//! }
//!
//! logger.flush().await?;
//! # Ok(())
//! # }
//! ```
//!
//! # Available Loggers
//!
//! - **[`ConsoleActivityLogger`]** - Console output with multiple formats and color support
//! - **[`FileActivityLogger`]** - Async file-based logging with automatic directory creation
//! - **[`TracingActivityLogger`]** - Integration with the tracing ecosystem for structured logging
//!
//! # Core Types
//!
//! - **[`ActivityLog`]** - Structured log entry with operation metadata and timestamps
//! - **[`ActivityLogger`]** - Core trait for pluggable log destinations
//! - **[`LoggerMiddleware`]** - Generic middleware implementation for pipeline integration
//! - **[`LoggerConfig`]** - Configuration for logging behavior and performance tuning
//! - **[`LogFormat`]** - Output format options (JSON, Pretty, Compact)
//! - **[`LogLevel`]** - Log level filtering for importance-based logging
//!
//! # Error Handling
//!
//! All async operations return `Result<(), LogError>` for comprehensive error handling:
//!
//! ```rust
//! use airssys_osl::middleware::logger::{ActivityLog, ConsoleActivityLogger, ActivityLogger, LogError};
//!
//! # #[tokio::main]
//! # async fn main() {
//! let logger = ConsoleActivityLogger::new();
//! let activity = ActivityLog::new(
//!     "op_123".to_string(),
//!     "test".to_string(),
//!     None,
//!     "Success".to_string(),
//!     0,
//! );
//!
//! match logger.log_activity(activity).await {
//!     Ok(()) => println!("Logged successfully"),
//!     Err(LogError::Io { operation, path, source }) => {
//!         eprintln!("I/O error during {} on {}: {}", operation, path, source);
//!     },
//!     Err(LogError::Formatting { operation_id, message }) => {
//!         eprintln!("Format error for {}: {}", operation_id, message);
//!     },
//!     Err(LogError::Configuration { field, message }) => {
//!         eprintln!("Config error in {}: {}", field, message);
//!     },
//!     Err(e) => eprintln!("Other error: {}", e),
//! }
//! # }
//! ```

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

// Concrete logger implementations
pub use loggers::{ConsoleActivityLogger, FileActivityLogger, TracingActivityLogger};

// Internal modules (following §4.3 - mod.rs only has declarations and re-exports)
mod activity;
mod config;
mod error;
mod formatter;
pub mod loggers;
mod middleware;
