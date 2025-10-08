//! Filesystem operation types.
//!
//! This module provides concrete implementations of filesystem operations that
//! implement the `Operation` trait. These types are used by the framework's
//! filesystem builder API.
//!
//! # Operations
//!
//! - [`FileReadOperation`] - Read file contents
//! - [`FileWriteOperation`] - Write or append to files
//! - [`DirectoryCreateOperation`] - Create directories (single or recursive)
//! - [`DirectoryListOperation`] - List directory contents
//! - [`FileDeleteOperation`] - Delete files
//!
//! # Examples
//!
//! ```rust
//! use airssys_osl::operations::{FileReadOperation, FileWriteOperation};
//! use airssys_osl::core::operation::{Operation, OperationType, Permission};
//!
//! // Read operation
//! let read_op = FileReadOperation::new("/etc/config.toml");
//! assert_eq!(read_op.operation_type(), OperationType::Filesystem);
//! assert_eq!(read_op.required_permissions(), vec![Permission::FilesystemRead("/etc/config.toml".to_string())]);
//!
//! // Write operation
//! let write_op = FileWriteOperation::new("/tmp/output.txt", b"Hello, World!".to_vec());
//! assert_eq!(write_op.required_permissions(), vec![Permission::FilesystemWrite("/tmp/output.txt".to_string())]);
//! ```

// Operation modules
pub mod create_dir;
pub mod delete;
pub mod list_dir;
pub mod read;
pub mod write;

// Re-export all operation types
pub use create_dir::DirectoryCreateOperation;
pub use delete::FileDeleteOperation;
pub use list_dir::DirectoryListOperation;
pub use read::FileReadOperation;
pub use write::FileWriteOperation;
