//! Directory listing operation.

// Layer 1: Standard library imports
use std::fmt;

// Layer 2: Third-party crate imports
use chrono::{DateTime, Utc};
use uuid::Uuid;

// Layer 3: Internal module imports
use crate::core::operation::{Operation, OperationType, Permission};

/// Operation to list directory contents.
///
/// Requires read permission for the specified path.
///
/// # Examples
///
/// ```rust
/// use airssys_osl::operations::DirectoryListOperation;
///
/// let op = DirectoryListOperation::new("/tmp");
/// ```
#[derive(Debug, Clone)]
pub struct DirectoryListOperation {
    /// Path to the directory to list
    pub path: String,
    
    /// When this operation was created
    pub created_at: DateTime<Utc>,
    
    /// Optional operation ID
    pub operation_id: Option<String>,
}

impl DirectoryListOperation {
    /// Create a new directory list operation.
    ///
    /// # Arguments
    ///
    /// * `path` - Path to the directory to list
    ///
    /// # Examples
    ///
    /// ```rust
    /// use airssys_osl::operations::DirectoryListOperation;
    ///
    /// let op = DirectoryListOperation::new("/tmp");
    /// assert_eq!(op.path, "/tmp");
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

impl Operation for DirectoryListOperation {
    fn operation_type(&self) -> OperationType {
        OperationType::Filesystem
    }
    
    fn required_permissions(&self) -> Vec<Permission> {
        vec![Permission::FilesystemRead(self.path.clone())]
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

impl fmt::Display for DirectoryListOperation {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "DirectoryList({})", self.path)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_directory_list_operation_creation() {
        let op = DirectoryListOperation::new("/tmp");
        assert_eq!(op.path, "/tmp");
        assert_eq!(op.operation_type(), OperationType::Filesystem);
    }

    #[test]
    fn test_directory_list_operation_permissions() {
        let op = DirectoryListOperation::new("/var/log");
        let permissions = op.required_permissions();
        assert_eq!(permissions.len(), 1);
        assert_eq!(permissions[0], Permission::FilesystemRead("/var/log".to_string()));
    }
}
