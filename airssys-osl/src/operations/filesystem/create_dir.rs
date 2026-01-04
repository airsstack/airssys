//! Directory creation operation.

// Layer 1: Standard library imports
use std::fmt;

// Layer 2: Third-party crate imports
use chrono::{DateTime, Utc};
use uuid::Uuid;

// Layer 3: Internal module imports
use crate::core::operation::{Operation, OperationType, Permission};

/// Operation to create a directory.
///
/// Requires write permission for the specified path. Supports both single
/// directory creation and recursive parent directory creation.
///
/// # Examples
///
/// ```rust
/// use airssys_osl::operations::DirectoryCreateOperation;
///
/// // Single directory
/// let op = DirectoryCreateOperation::new("/tmp/mydir");
///
/// // Recursive creation (creates parent directories)
/// let op = DirectoryCreateOperation::new("/tmp/parent/child/grandchild")
///     .recursive();
/// ```
#[derive(Debug, Clone)]
pub struct DirectoryCreateOperation {
    /// Path to the directory to create
    pub path: String,

    /// Whether to create parent directories recursively
    pub recursive: bool,

    /// When this operation was created
    pub created_at: DateTime<Utc>,

    /// Optional operation ID
    pub operation_id: Option<String>,
}

impl DirectoryCreateOperation {
    /// Create a new directory creation operation.
    ///
    /// # Arguments
    ///
    /// * `path` - Path to the directory to create
    ///
    /// # Examples
    ///
    /// ```rust
    /// use airssys_osl::operations::DirectoryCreateOperation;
    ///
    /// let op = DirectoryCreateOperation::new("/tmp/mydir");
    /// assert_eq!(op.recursive, false);
    /// ```
    pub fn new(path: impl Into<String>) -> Self {
        Self {
            path: path.into(),
            recursive: false,
            created_at: Utc::now(),
            operation_id: None,
        }
    }

    /// Enable recursive directory creation (create parent directories).
    ///
    /// # Examples
    ///
    /// ```rust
    /// use airssys_osl::operations::DirectoryCreateOperation;
    ///
    /// let op = DirectoryCreateOperation::new("/tmp/parent/child")
    ///     .recursive();
    /// assert_eq!(op.recursive, true);
    /// ```
    pub fn recursive(mut self) -> Self {
        self.recursive = true;
        self
    }

    /// Create with explicit timestamp (for testing).
    pub fn with_timestamp(
        path: impl Into<String>,
        recursive: bool,
        created_at: DateTime<Utc>,
    ) -> Self {
        Self {
            path: path.into(),
            recursive,
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

impl Operation for DirectoryCreateOperation {
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

impl fmt::Display for DirectoryCreateOperation {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let mode = if self.recursive {
            "recursive"
        } else {
            "single"
        };
        write!(f, "DirectoryCreate({}, mode={})", self.path, mode)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_directory_create_operation_new() {
        let op = DirectoryCreateOperation::new("/tmp/mydir");
        assert_eq!(op.path, "/tmp/mydir");
        assert!(!op.recursive);
    }

    #[test]
    fn test_directory_create_operation_recursive() {
        let op = DirectoryCreateOperation::new("/tmp/parent/child").recursive();
        assert_eq!(op.path, "/tmp/parent/child");
        assert!(op.recursive);
    }

    #[test]
    fn test_directory_create_operation_permissions() {
        let op = DirectoryCreateOperation::new("/tmp/newdir");
        let permissions = op.required_permissions();
        assert_eq!(permissions.len(), 1);
        assert_eq!(
            permissions[0],
            Permission::FilesystemWrite("/tmp/newdir".to_string())
        );
    }
}
