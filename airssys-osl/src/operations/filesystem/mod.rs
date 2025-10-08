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

#[cfg(test)]
mod tests {
    use super::*;

    /// Test that all operations are cloneable (required by Operation trait)
    #[test]
    fn test_all_operations_are_cloneable() {
        let file_read = FileReadOperation::new("/tmp/test.txt");
        let _cloned = file_read.clone();

        let file_write = FileWriteOperation::new("/tmp/test.txt", vec![1, 2, 3]);
        let _cloned = file_write.clone();

        let dir_create = DirectoryCreateOperation::new("/tmp/dir");
        let _cloned = dir_create.clone();

        let dir_list = DirectoryListOperation::new("/tmp");
        let _cloned = dir_list.clone();

        let file_delete = FileDeleteOperation::new("/tmp/test.txt");
        let _cloned = file_delete.clone();
    }

    /// Test Display implementations for all operations
    #[test]
    fn test_operations_display() {
        let file_read = FileReadOperation::new("/tmp/test.txt");
        assert_eq!(format!("{}", file_read), "FileRead(/tmp/test.txt)");

        let file_write = FileWriteOperation::new("/tmp/test.txt", vec![1, 2, 3]);
        assert_eq!(
            format!("{}", file_write),
            "FileWrite(/tmp/test.txt, mode=write, 3 bytes)"
        );

        let dir_create = DirectoryCreateOperation::new("/tmp/dir").recursive();
        assert_eq!(
            format!("{}", dir_create),
            "DirectoryCreate(/tmp/dir, mode=recursive)"
        );

        let dir_list = DirectoryListOperation::new("/tmp");
        assert_eq!(format!("{}", dir_list), "DirectoryList(/tmp)");

        let file_delete = FileDeleteOperation::new("/tmp/test.txt");
        assert_eq!(format!("{}", file_delete), "FileDelete(/tmp/test.txt)");
    }
}
