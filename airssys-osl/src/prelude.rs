//! Primary API imports for AirsSys OSL framework usage.
//!
//! This module provides the main entry point for using AirsSys OSL, re-exporting
//! the most commonly used types and functions for ergonomic framework usage.
//!
//! This prelude follows the established pattern of providing the framework API
//! (80% use cases) as the primary interface while maintaining access to core
//! primitives for advanced usage (20% use cases).
//!
//! # Framework-First API (Recommended)
//!
//! ```rust
//! use airssys_osl::prelude::*;
//!
//! # async fn example() -> OSResult<()> {
//! // Primary framework usage pattern
//! let osl = OSLFramework::builder()
//!     .with_default_security()
//!     .with_security_logging(true)
//!     .build().await?;
//! # Ok(())
//! # }
//! ```
//!
//! # Primitive API (Advanced)
//!
//! ```rust
//! use airssys_osl::{prelude::*, core::*};
//!
//! # async fn example() -> OSResult<()> {
//! // Direct primitive usage for advanced cases
//! let context = ExecutionContext::new(
//!     SecurityContext::new("advanced-user".to_string())
//! );
//! # Ok(())
//! # }
//! ```
//!
//! # Custom Executor with Macro (requires `macros` feature)
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

// Framework layer - primary API (80% of use cases)
pub use crate::framework::{OSLFramework, OSLFrameworkBuilder};

// Configuration system
pub use crate::framework::config::{OSLConfig, OSLConfigBuilder, SecurityConfig};

// Core result types - used across all levels
pub use crate::core::result::{OSError, OSResult};

// Core executor types - needed for custom executor implementations
pub use crate::core::executor::ExecutionResult;

// Core context types - needed for both framework and primitive usage
pub use crate::core::context::{ExecutionContext, SecurityContext};

// Core operation types - foundation for all operations
pub use crate::core::operation::{Operation, OperationType};

// Concrete operation types - for advanced usage and testing
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

// Middleware configuration - for Level 2 usage
pub use crate::middleware::logger::{LogFormat, LogLevel};

// Procedural macros for ergonomic implementations (optional feature)
#[cfg(feature = "macros")]
pub use airssys_osl_macros::executor;

// Standard library re-exports for convenience
pub use chrono::{DateTime, Utc};
pub use std::time::Duration;

// TODO: The following will be added in OSL-TASK-006:
// - Operation builders (FilesystemBuilder, ProcessBuilder, NetworkBuilder)
// - Middleware orchestration helpers
// - Executor registry access
