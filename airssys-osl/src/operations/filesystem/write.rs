//! File write operation.

// Layer 1: Standard library imports
use std::fmt;

// Layer 2: Third-party crate imports
use chrono::{DateTime, Utc};
use uuid::Uuid;

// Layer 3: Internal module imports
use crate::core::operation::{Operation, OperationType, Permission};

/// Operation to write data to a file.
///
/// Requires write permission for the specified path. Supports both overwrite
/// and append modes.
///
/// # Examples
///
/// ```rust
/// use airssys_osl::operations::FileWriteOperation;
///
/// // Overwrite mode (default)
/// let op = FileWriteOperation::new("/tmp/output.txt", b"Hello, World!".to_vec());
///
/// // Append mode
/// let op = FileWriteOperation::append("/tmp/output.txt", b"More data".to_vec());
/// ```
#[derive(Debug, Clone)]
pub struct FileWriteOperation {
    /// Path to the file to write
    pub path: String,

    /// Content to write to the file
    pub content: Vec<u8>,

    /// Whether to append or overwrite
    pub append: bool,

    /// When this operation was created
    pub created_at: DateTime<Utc>,

    /// Optional operation ID
    pub operation_id: Option<String>,
}

impl FileWriteOperation {
    /// Create a new file write operation (overwrite mode).
    ///
    /// # Arguments
    ///
    /// * `path` - Path to the file to write
    /// * `content` - Content to write to the file
    ///
    /// # Examples
    ///
    /// ```rust
    /// use airssys_osl::operations::FileWriteOperation;
    ///
    /// let op = FileWriteOperation::new("/tmp/output.txt", b"Hello, World!".to_vec());
    /// assert_eq!(op.append, false);
    /// ```
    pub fn new(path: impl Into<String>, content: Vec<u8>) -> Self {
        Self {
            path: path.into(),
            content,
            append: false,
            created_at: Utc::now(),
            operation_id: None,
        }
    }

    /// Create a new file write operation in append mode.
    ///
    /// # Arguments
    ///
    /// * `path` - Path to the file to append to
    /// * `content` - Content to append to the file
    ///
    /// # Examples
    ///
    /// ```rust
    /// use airssys_osl::operations::FileWriteOperation;
    ///
    /// let op = FileWriteOperation::append("/tmp/output.txt", b"More data".to_vec());
    /// assert_eq!(op.append, true);
    /// ```
    pub fn append(path: impl Into<String>, content: Vec<u8>) -> Self {
        Self {
            path: path.into(),
            content,
            append: true,
            created_at: Utc::now(),
            operation_id: None,
        }
    }

    /// Create with explicit timestamp (for testing).
    pub fn with_timestamp(
        path: impl Into<String>,
        content: Vec<u8>,
        append: bool,
        created_at: DateTime<Utc>,
    ) -> Self {
        Self {
            path: path.into(),
            content,
            append,
            created_at,
            operation_id: None,
        }
    }

    /// Set custom operation ID.
    pub fn with_operation_id(mut self, id: impl Into<String>) -> Self {
        self.operation_id = Some(id.into());
        self
    }
}

impl Operation for FileWriteOperation {
    fn operation_type(&self) -> OperationType {
        OperationType::Filesystem
    }

    fn required_permissions(&self) -> Vec<Permission> {
        vec![Permission::FilesystemWrite(self.path.clone())]
    }

    fn created_at(&self) -> DateTime<Utc> {
        self.created_at
    }

    fn operation_id(&self) -> String {
        self.operation_id
            .clone()
            .unwrap_or_else(|| format!("{}:{}", self.operation_type().as_str(), Uuid::new_v4()))
    }
}

impl fmt::Display for FileWriteOperation {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let mode = if self.append { "append" } else { "write" };
        write!(
            f,
            "FileWrite({}, mode={}, {} bytes)",
            self.path,
            mode,
            self.content.len()
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_file_write_operation_new() {
        let content = b"Hello, World!".to_vec();
        let op = FileWriteOperation::new("/tmp/output.txt", content.clone());
        assert_eq!(op.path, "/tmp/output.txt");
        assert_eq!(op.content, content);
        assert!(!op.append);
    }

    #[test]
    fn test_file_write_operation_append() {
        let content = b"More data".to_vec();
        let op = FileWriteOperation::append("/tmp/output.txt", content.clone());
        assert_eq!(op.path, "/tmp/output.txt");
        assert_eq!(op.content, content);
        assert!(op.append);
    }

    #[test]
    fn test_file_write_operation_permissions() {
        let op = FileWriteOperation::new("/tmp/test.txt", vec![1, 2, 3]);
        let permissions = op.required_permissions();
        assert_eq!(permissions.len(), 1);
        assert_eq!(
            permissions[0],
            Permission::FilesystemWrite("/tmp/test.txt".to_string())
        );
    }
}
