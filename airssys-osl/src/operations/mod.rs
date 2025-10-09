//! Concrete operation type implementations.
//!
//! This module provides concrete implementations of the `Operation` trait for
//! filesystem, process, and network operations. These types bridge the framework
//! builder API with the core execution architecture.
//!
//! # Architecture
//!
//! Operations are concrete types that implement the `Operation` trait:
//!
//! ```text
//! User Code
//!     ↓ (uses helper functions)
//! read_file("/path", "user")
//!     ↓ (creates)
//! FileReadOperation
//!     ↓ (passed to)
//! FilesystemExecutor::execute(operation, context)
//!     ↓ (produces)
//! ExecutionResult
//! ```
//!
//! # Operation Categories
//!
//! - **Filesystem Operations**: File and directory operations (read, write, create, delete)
//! - **Process Operations**: Process management (spawn, kill, signal)
//! - **Network Operations**: Network connectivity (connect, listen, socket)
//!
//! # Design Principles
//!
//! 1. **Stateless**: Operations contain all data needed for execution
//! 2. **Cloneable**: Operations can be duplicated for retry logic
//! 3. **Type-Safe**: Strong typing prevents misuse
//! 4. **Auditable**: All operations have timestamps and IDs for security auditing
//!
//! # Example
//!
//! ```rust,ignore
//! use airssys_osl::operations::FileReadOperation;
//! use airssys_osl::core::Operation;
//!
//! let operation = FileReadOperation::new("/etc/config.toml");
//! assert_eq!(operation.operation_type(), OperationType::Filesystem);
//! assert!(operation.required_permissions().contains(&Permission::FilesystemRead("/etc/config.toml".to_string())));
//! ```

// Filesystem operation types (modular structure)
pub mod filesystem;

// Process operation types (modular structure)
pub mod process;

// Network operation types
pub mod network;

// Re-export all operation types for convenient access
pub use filesystem::{
    DirectoryCreateOperation, DirectoryListOperation, FileDeleteOperation, FileReadOperation,
    FileWriteOperation,
};
pub use network::{NetworkConnectOperation, NetworkListenOperation, NetworkSocketOperation};
pub use process::{ProcessKillOperation, ProcessSignalOperation, ProcessSpawnOperation};
