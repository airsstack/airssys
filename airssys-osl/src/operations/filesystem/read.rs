//! File read operation.

// Layer 1: Standard library imports
use std::fmt;

// Layer 2: Third-party crate imports
use chrono::{DateTime, Utc};
use uuid::Uuid;

// Layer 3: Internal module imports
use crate::core::operation::{Operation, OperationType, Permission};

/// Operation to read a file from the filesystem.
///
/// Requires read permission for the specified path. This operation is used
/// to read file contents through the OSL framework with proper security
/// validation and audit logging.
///
/// # Examples
///
/// ```rust
/// use airssys_osl::operations::FileReadOperation;
/// use chrono::Utc;
///
/// // Basic usage
/// let op = FileReadOperation::new("/etc/config.toml");
///
/// // With custom timestamp (for testing)
/// let timestamp = Utc::now();
/// let op = FileReadOperation::with_timestamp("/etc/config.toml", timestamp);
///
/// // With custom operation ID
/// let op = FileReadOperation::new("/etc/config.toml")
///     .with_operation_id("my-custom-id");
/// ```
#[derive(Debug, Clone)]
pub struct FileReadOperation {
    /// Path to the file to read
    pub path: String,

    /// When this operation was created
    pub created_at: DateTime<Utc>,

    /// Optional operation ID (generated if None)
    pub operation_id: Option<String>,
}

impl FileReadOperation {
    /// Create a new file read operation.
    ///
    /// # Arguments
    ///
    /// * `path` - Path to the file to read
    ///
    /// # Examples
    ///
    /// ```rust
    /// use airssys_osl::operations::FileReadOperation;
    ///
    /// let op = FileReadOperation::new("/etc/config.toml");
    /// assert_eq!(op.path, "/etc/config.toml");
    /// ```
    pub fn new(path: impl Into<String>) -> Self {
        Self {
            path: path.into(),
            created_at: Utc::now(),
            operation_id: None,
        }
    }

    /// Create with explicit timestamp (for testing).
    ///
    /// # Arguments
    ///
    /// * `path` - Path to the file to read
    /// * `created_at` - Timestamp when the operation was created
    ///
    /// # Examples
    ///
    /// ```rust
    /// use airssys_osl::operations::FileReadOperation;
    /// use chrono::Utc;
    ///
    /// let timestamp = Utc::now();
    /// let op = FileReadOperation::with_timestamp("/etc/config.toml", timestamp);
    /// assert_eq!(op.created_at, timestamp);
    /// ```
    pub fn with_timestamp(path: impl Into<String>, created_at: DateTime<Utc>) -> Self {
        Self {
            path: path.into(),
            created_at,
            operation_id: None,
        }
    }

    /// Set custom operation ID.
    ///
    /// # Arguments
    ///
    /// * `id` - Custom operation ID to use
    ///
    /// # Examples
    ///
    /// ```rust
    /// use airssys_osl::operations::FileReadOperation;
    /// use airssys_osl::core::operation::Operation;
    ///
    /// let op = FileReadOperation::new("/etc/config.toml")
    ///     .with_operation_id("my-custom-id");
    /// assert_eq!(op.operation_id(), "my-custom-id");
    /// ```
    pub fn with_operation_id(mut self, id: impl Into<String>) -> Self {
        self.operation_id = Some(id.into());
        self
    }
}

impl Operation for FileReadOperation {
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
        self.operation_id
            .clone()
            .unwrap_or_else(|| format!("{}:{}", self.operation_type().as_str(), Uuid::new_v4()))
    }
}

impl fmt::Display for FileReadOperation {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "FileRead({})", self.path)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_file_read_operation_creation() {
        let op = FileReadOperation::new("/etc/config.toml");
        assert_eq!(op.path, "/etc/config.toml");
        assert_eq!(op.operation_type(), OperationType::Filesystem);
        assert!(!op.requires_elevated_privileges());
    }

    #[test]
    fn test_file_read_operation_permissions() {
        let op = FileReadOperation::new("/etc/passwd");
        let permissions = op.required_permissions();
        assert_eq!(permissions.len(), 1);
        assert_eq!(
            permissions[0],
            Permission::FilesystemRead("/etc/passwd".to_string())
        );
    }

    #[test]
    fn test_file_read_operation_with_custom_id() {
        let op = FileReadOperation::new("/tmp/test.txt").with_operation_id("custom-123");
        assert_eq!(op.operation_id(), "custom-123");
    }

    #[test]
    fn test_file_read_operation_generated_id() {
        let op = FileReadOperation::new("/tmp/test.txt");
        let id = op.operation_id();
        assert!(id.starts_with("filesystem:"));
    }
}
