//! Process kill operation.

// Layer 1: Standard library imports
use std::fmt;

// Layer 2: Third-party crate imports
use chrono::{DateTime, Utc};
use uuid::Uuid;

// Layer 3: Internal module imports
use crate::core::operation::{Operation, OperationType, Permission};

/// Operation to terminate a process.
///
/// Requires ProcessManage permission, which is an elevated privilege. This operation
/// sends a SIGKILL signal to forcefully terminate the target process.
///
/// # Security
///
/// **This operation always requires elevated privileges** as it involves terminating
/// system processes. The framework's security middleware will validate permissions
/// before execution.
///
/// # Examples
///
/// ```rust
/// use airssys_osl::operations::ProcessKillOperation;
/// use airssys_osl::core::operation::Operation;
///
/// // Kill a process by PID
/// let op = ProcessKillOperation::new(12345);
/// assert_eq!(op.pid, 12345);
/// assert!(op.requires_elevated_privileges());
/// ```
#[derive(Debug, Clone)]
pub struct ProcessKillOperation {
    /// Process ID to kill
    pub pid: u32,

    /// When this operation was created
    pub created_at: DateTime<Utc>,

    /// Optional operation ID
    pub operation_id: Option<String>,
}

impl ProcessKillOperation {
    /// Create a new process kill operation.
    ///
    /// # Arguments
    ///
    /// * `pid` - Process ID to terminate
    ///
    /// # Examples
    ///
    /// ```rust
    /// use airssys_osl::operations::ProcessKillOperation;
    ///
    /// let op = ProcessKillOperation::new(12345);
    /// assert_eq!(op.pid, 12345);
    /// ```
    pub fn new(pid: u32) -> Self {
        Self {
            pid,
            created_at: Utc::now(),
            operation_id: None,
        }
    }

    /// Create with explicit timestamp (for testing).
    ///
    /// # Arguments
    ///
    /// * `pid` - Process ID to terminate
    /// * `created_at` - Timestamp when the operation was created
    ///
    /// # Examples
    ///
    /// ```rust
    /// use airssys_osl::operations::ProcessKillOperation;
    /// use chrono::Utc;
    ///
    /// let timestamp = Utc::now();
    /// let op = ProcessKillOperation::with_timestamp(12345, timestamp);
    /// assert_eq!(op.created_at, timestamp);
    /// ```
    pub fn with_timestamp(pid: u32, created_at: DateTime<Utc>) -> Self {
        Self {
            pid,
            created_at,
            operation_id: None,
        }
    }

    /// Set custom operation ID.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use airssys_osl::operations::ProcessKillOperation;
    /// use airssys_osl::core::operation::Operation;
    ///
    /// let op = ProcessKillOperation::new(12345)
    ///     .with_operation_id("my-kill-op");
    /// assert_eq!(op.operation_id(), "my-kill-op");
    /// ```
    pub fn with_operation_id(mut self, id: impl Into<String>) -> Self {
        self.operation_id = Some(id.into());
        self
    }
}

impl Operation for ProcessKillOperation {
    fn operation_type(&self) -> OperationType {
        OperationType::Process
    }

    fn required_permissions(&self) -> Vec<Permission> {
        vec![Permission::ProcessManage]
    }

    fn created_at(&self) -> DateTime<Utc> {
        self.created_at
    }

    fn operation_id(&self) -> String {
        self.operation_id.clone().unwrap_or_else(|| {
            format!("{}:{}", self.operation_type().as_str(), Uuid::new_v4())
        })
    }

    fn requires_elevated_privileges(&self) -> bool {
        true // Process killing always requires elevation
    }
}

impl fmt::Display for ProcessKillOperation {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "ProcessKill(pid={})", self.pid)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_process_kill_operation_creation() {
        let op = ProcessKillOperation::new(12345);
        assert_eq!(op.pid, 12345);
        assert_eq!(op.operation_type(), OperationType::Process);
    }

    #[test]
    fn test_process_kill_permissions() {
        let op = ProcessKillOperation::new(12345);
        let permissions = op.required_permissions();
        assert_eq!(permissions.len(), 1);
        assert_eq!(permissions[0], Permission::ProcessManage);
    }

    #[test]
    fn test_process_kill_requires_elevation() {
        let op = ProcessKillOperation::new(12345);
        assert!(op.requires_elevated_privileges());
    }

    #[test]
    fn test_process_kill_with_custom_id() {
        let op = ProcessKillOperation::new(12345)
            .with_operation_id("custom-kill");
        assert_eq!(op.operation_id(), "custom-kill");
    }

    #[test]
    fn test_process_kill_generated_id() {
        let op = ProcessKillOperation::new(12345);
        let id = op.operation_id();
        assert!(id.starts_with("process:"));
    }
}
