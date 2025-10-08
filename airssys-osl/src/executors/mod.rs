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
//! use airssys_osl::executors::{FilesystemExecutor, ProcessExecutor, NetworkExecutor};
//! use airssys_osl::core::executor::OSExecutor;
//! use airssys_osl::core::context::{ExecutionContext, SecurityContext};
//! use airssys_osl::operations::{FileReadOperation, ProcessSpawnOperation, NetworkConnectOperation};
//!
//! # async fn example() -> airssys_osl::core::result::OSResult<()> {
//! // Create execution context
//! let context = ExecutionContext::new(SecurityContext::new("user".to_string()));
//!
//! // Filesystem executor
//! let fs_executor = FilesystemExecutor::new();
//! let read_op = FileReadOperation::new("/etc/hosts");
//! let result = fs_executor.execute(read_op, &context).await?;
//!
//! // Process executor
//! let proc_executor = ProcessExecutor::new("proc-executor");
//! let spawn_op = ProcessSpawnOperation::new("echo").arg("hello");
//! let result = proc_executor.execute(spawn_op, &context).await?;
//!
//! // Network executor
//! let net_executor = NetworkExecutor::new("net-executor");
//! let connect_op = NetworkConnectOperation::new("localhost:8080");
//! let result = net_executor.execute(connect_op, &context).await?;
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
//! - `process/` - Process operations (spawn, kill, signal)
//! - `network/` - Network operations (connect, listen, socket)

// Re-export executor implementations
pub mod filesystem;
pub mod network;
pub mod process;

// Re-export main types for convenience
pub use filesystem::FilesystemExecutor;
pub use network::NetworkExecutor;
pub use process::ProcessExecutor;
