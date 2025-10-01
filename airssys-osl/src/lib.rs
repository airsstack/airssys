//! AirssysOSL - Operating System Layer for Airssys
//!
//! This crate provides the core operating system abstraction layer for the Airssys platform.
//! It defines essential types, traits, and functionality for secure, cross-platform
//! operating system interactions.
//!
//! # Architecture
//!
//! The OSL is built around several core concepts:
//!
//! - **Operations**: Abstract representations of OS-level operations
//! - **Executors**: Components that can execute specific types of operations  
//! - **Middleware**: Cross-cutting concerns like logging, security, and validation
//! - **Context**: Execution context including security and metadata
//! - **Results**: Standardized result types with comprehensive error handling
//!
//! # Quick Start
//!
//! ```rust
//! use airssys_osl::core::context::{SecurityContext, ExecutionContext};
//! use airssys_osl::core::operation::OperationType;
//!
//! let security_context = SecurityContext::new("user123".to_string());
//! let execution_context = ExecutionContext::new(security_context);
//!
//! assert_eq!(execution_context.principal(), "user123");
//! ```
//!
//! # Logger Middleware Examples
//!
//! ## Console Logging
//!
//! ```rust
//! use airssys_osl::middleware::logger::{ActivityLog, ConsoleActivityLogger, ActivityLogger, LogFormat};
//!
//! # #[tokio::main]
//! # async fn main() -> Result<(), Box<dyn std::error::Error>> {
//! // Create a console logger with JSON format
//! let logger = ConsoleActivityLogger::new()
//!     .with_format(LogFormat::Json);
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
//! // Create a file logger
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
//! ).with_metadata("query".to_string(), serde_json::Value::String("SELECT * FROM users".to_string()));
//!
//! logger.log_activity(activity).await?;
//! logger.flush().await?;
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
//! // Create logger middleware for pipeline integration
//! let logger = ConsoleActivityLogger::new();
//! let config = LoggerConfig::default();
//! let middleware = LoggerMiddleware::new(logger, config);
//!
//! // Access the underlying logger for direct usage
//! let logger_ref = middleware.logger();
//! # }
//! ```
//!
//! # Complete Examples
//!
//! For comprehensive usage examples, see the executable examples in the
//! `examples/` directory. Run them with:
//!
//! ```bash
//! cargo run --example basic_usage
//! cargo run --example middleware_pipeline
//! ```
//!
//! # Core Modules
//!
//! ## [`core`] - Foundational Framework Abstractions
//! **Primary Module**: Contains all essential traits, types, and abstractions for the OSL framework
//!
//! - **[`core::context`]** - Execution and security context management
//!   - Manages security boundaries and execution metadata
//!   - Provides audit trail and permission enforcement
//!
//! - **[`core::executor`]** - Operation executor framework
//!   - Defines contracts for OS operation execution
//!   - Handles standardized result processing
//!
//! - **[`core::middleware`]** - Cross-cutting concerns pipeline
//!   - Interceptor patterns for logging, validation, monitoring
//!   - Composable request/response processing
//!
//! - **[`core::operation`]** - Operation modeling and permissions
//!   - Abstract representations of system operations
//!   - Type-safe permission and capability system
//!
//! - **[`core::result`]** - Comprehensive error handling
//!   - Structured error types with context
//!   - Consistent result propagation patterns
//!
//! ## [`middleware`] - Concrete Middleware Implementations
//!
//! - **[`middleware::logger`]** - Activity logging and audit trail middleware
//!   - Multiple output formats (JSON, Pretty, Compact)
//!   - Console, file, and tracing ecosystem integration
//!   - Thread-safe concurrent logging support
//!   - Comprehensive metadata and error handling
//!
//! ## Module Integration Philosophy
//!
//! This library uses **explicit module imports** instead of crate-level re-exports
//! to maintain clear architectural boundaries. Import specific types from their modules:
//!
//! ```rust
//! use airssys_osl::core::context::ExecutionContext;
//! use airssys_osl::core::operation::OperationType;
//! use airssys_osl::middleware::logger::{ActivityLog, ConsoleActivityLogger};
//! ```
//!
//! This approach provides:
//! - **Clear dependency tracking**: Easy to understand what each component uses
//! - **Better IDE support**: Precise navigation and completion
//! - **Maintainable architecture**: Explicit module boundaries prevent coupling

pub mod core;
pub mod middleware;
