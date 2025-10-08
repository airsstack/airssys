//! Platform-specific executor implementations for OS operations.
//!
//! This module provides concrete implementations of the `OSExecutor` trait for
//! filesystem, process, and network operations. All executors use tokio for
//! async I/O operations and follow the Microsoft Rust Guidelines for
//! production-quality code.
//!
//! # Architecture
//!
//! The executor module follows a component-based architecture:
//! - **FilesystemExecutor**: Handles file and directory operations using tokio::fs
//! - **ProcessExecutor**: Manages process spawning and control using tokio::process
//! - **NetworkExecutor**: Handles network connections using tokio::net
//!
//! # Usage
//!
//! Executors are automatically initialized and managed by the `ExecutorRegistry`:
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
//! # Ok(())
//! # }
//! ```
//!
//! # Module Structure
//!
//! Each executor is organized into a submodule with implementation files
//! for each operation type, mirroring the structure of the operations module:
//!
//! - `filesystem/` - Filesystem operations (read, write, create_dir, delete)
//! - `process/` - Process operations (spawn, kill, signal) - TODO
//! - `network/` - Network operations (connect, listen) - TODO

// Re-export executor implementations
pub mod filesystem;
// TODO(OSL-TASK-008): Implement process executor for ProcessSpawnOperation, etc.
// pub mod process;
// TODO(OSL-TASK-008): Implement network executor for NetworkConnectOperation, etc.
// pub mod network;

// Re-export main types for convenience
pub use filesystem::FilesystemExecutor;
// pub use process::ProcessExecutor;
// pub use network::NetworkExecutor;
