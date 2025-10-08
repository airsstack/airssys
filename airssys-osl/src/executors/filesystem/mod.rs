//! Filesystem executor implementation using tokio::fs.
//!
//! This module provides the `FilesystemExecutor` which handles filesystem operations
//! such as reading files, writing files, and creating directories using tokio's
//! async filesystem APIs.
//!
//! # Module Structure
//!
//! - `executor` - FilesystemExecutor struct definition
//! - `read` - FileReadOperation executor implementation
//! - `write` - FileWriteOperation executor implementation
//! - `create_dir` - DirectoryCreateOperation executor implementation
//! - `delete` - FileDeleteOperation executor implementation
//!
//! # Example
//!
//! ```rust,no_run
//! use airssys_osl::executors::FilesystemExecutor;
//! use airssys_osl::core::executor::OSExecutor;
//! use airssys_osl::operations::filesystem::FileReadOperation;
//! use airssys_osl::core::context::{ExecutionContext, SecurityContext};
//!
//! # async fn example() -> airssys_osl::core::result::OSResult<()> {
//! let executor = FilesystemExecutor::new();
//! let operation = FileReadOperation::new("/etc/hosts");
//! let security_context = SecurityContext::new("test-user".to_string());
//! let context = ExecutionContext::new(security_context);
//!
//! let result = executor.execute(operation, &context).await?;
//! println!("Read {} bytes", result.output.len());
//! # Ok(())
//! # }
//! ```

// Module declarations (private - internal implementation)
mod create_dir;
mod delete;
mod executor;
mod read;
mod write;

// Public re-exports
pub use executor::FilesystemExecutor;
