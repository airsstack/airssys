//! File deletion operation.

// Layer 1: Standard library imports
use std::fmt;

// Layer 2: Third-party crate imports
use chrono::{DateTime, Utc};
use uuid::Uuid;

// Layer 3: Internal module imports
use crate::core::operation::{Operation, OperationType, Permission};

/// Operation to delete a file.
///
/// Requires write permission for the specified path.
///
/// # Examples
///
/// ```rust
/// use airssys_osl::operations::FileDeleteOperation;
///
/// let op = FileDeleteOperation::new("/tmp/file-to-delete.txt");
/// ```
#[derive(Debug, Clone)]
pub struct FileDeleteOperation {
    /// Path to the file to delete
    pub path: String,
    
    /// When this operation was created
    pub created_at: DateTime<Utc>,
    
    /// Optional operation ID
    pub operation_id: Option<String>,
}

impl FileDeleteOperation {
    /// Create a new file delete operation.
    ///
    /// # Arguments
    ///
    /// * `path` - Path to the file to delete
    ///
    /// # Examples
    ///
    /// ```rust
    /// use airssys_osl::operations::FileDeleteOperation;
    ///
    /// let op = FileDeleteOperation::new("/tmp/file-to-delete.txt");
    /// assert_eq!(op.path, "/tmp/file-to-delete.txt");
    /// ```
    pub fn new(path: impl Into<String>) -> Self {
        Self {
            path: path.into(),
            created_at: Utc::now(),
            operation_id: None,
        }
    }
    
    /// Create with explicit timestamp (for testing).
    pub fn with_timestamp(path: impl Into<String>, created_at: DateTime<Utc>) -> Self {
        Self {
            path: path.into(),
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

impl Operation for FileDeleteOperation {
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
        self.operation_id.clone().unwrap_or_else(|| {
            format!("{}:{}", self.operation_type().as_str(), Uuid::new_v4())
        })
    }
}

impl fmt::Display for FileDeleteOperation {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "FileDelete({})", self.path)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_file_delete_operation_creation() {
        let op = FileDeleteOperation::new("/tmp/delete-me.txt");
        assert_eq!(op.path, "/tmp/delete-me.txt");
        assert_eq!(op.operation_type(), OperationType::Filesystem);
    }

    #[test]
    fn test_file_delete_operation_permissions() {
        let op = FileDeleteOperation::new("/tmp/file.txt");
        let permissions = op.required_permissions();
        assert_eq!(permissions.len(), 1);
        assert_eq!(permissions[0], Permission::FilesystemWrite("/tmp/file.txt".to_string()));
    }
}
