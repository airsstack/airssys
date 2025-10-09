//! Primary API imports for AirsSys OSL.
//!
//! This module provides the main entry point for using AirsSys OSL, re-exporting
//! the most commonly used types and functions.
//!
//! # Three Usage Levels
//!
//! ## 1. Helper Functions (Most Ergonomic) - Coming in OSL-TASK-009
//!
//! ```rust,ignore
//! use airssys_osl::helpers::*;
//!
//! // One-line operations
//! let data = read_file("/etc/hosts", "admin").await?;
//! write_file("/tmp/test.txt", b"data".to_vec(), "admin").await?;
//! ```
//!
//! ## 2. Direct API (Current - Maximum Control)
//!
//! ```rust
//! use airssys_osl::prelude::*;
//!
//! # async fn example() -> OSResult<()> {
//! // Direct executor usage
//! let executor = crate::executors::FilesystemExecutor::new();
//! let operation = FileReadOperation::new("/etc/hosts".into());
//! let context = ExecutionContext::new(
//!     SecurityContext::new("admin".to_string())
//! );
//! let result = executor.execute(operation, &context).await?;
//! # Ok(())
//! # }
//! ```
//!
//! ## 3. Custom Executors with Macro (requires `macros` feature)
//!
//! ```rust,ignore
//! use airssys_osl::prelude::*;
//!
//! #[derive(Debug)]
//! struct MyExecutor;
//!
//! #[executor]
//! impl MyExecutor {
//!     async fn file_read(
//!         &self,
//!         operation: FileReadOperation,
//!         context: &ExecutionContext,
//!     ) -> OSResult<ExecutionResult> {
//!         // Custom implementation
//!         Ok(ExecutionResult::success(vec![]))
//!     }
//! }
//! ```

// Core result types - used across all levels
pub use crate::core::result::{OSError, OSResult};

// Core executor types - needed for custom executor implementations
pub use crate::core::executor::ExecutionResult;

// Core context types - needed for all operation execution
pub use crate::core::context::{ExecutionContext, SecurityContext};

// Core operation types - foundation for all operations
pub use crate::core::operation::{Operation, OperationType};

// Security configuration - extracted from framework layer
pub use crate::core::security::{AuditConfig, EnforcementLevel, SecurityConfig};

// Concrete operation types - for operation execution
pub use crate::operations::{
    // Filesystem operations
    DirectoryCreateOperation,
    DirectoryListOperation,
    FileDeleteOperation,
    FileReadOperation,
    FileWriteOperation,
    // Network operations
    NetworkConnectOperation,
    NetworkListenOperation,
    NetworkSocketOperation,
    // Process operations
    ProcessKillOperation,
    ProcessSignalOperation,
    ProcessSpawnOperation,
};

// Middleware configuration - for middleware usage
pub use crate::middleware::logger::{LogFormat, LogLevel};

// Procedural macros for ergonomic implementations (optional feature)
#[cfg(feature = "macros")]
pub use airssys_osl_macros::executor;

// Standard library re-exports for convenience
pub use chrono::{DateTime, Utc};
pub use std::time::Duration;
