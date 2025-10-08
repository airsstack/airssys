//! Process signal operation.

// Layer 1: Standard library imports
use std::fmt;

// Layer 2: Third-party crate imports
use chrono::{DateTime, Utc};
use uuid::Uuid;

// Layer 3: Internal module imports
use crate::core::operation::{Operation, OperationType, Permission};

/// Operation to send a signal to a process.
///
/// Requires ProcessManage permission, which is an elevated privilege. This operation
/// allows sending arbitrary signals (SIGTERM, SIGKILL, SIGHUP, etc.) to target processes.
///
/// # Signal Numbers
///
/// Common Unix signal numbers:
/// - `1` (SIGHUP) - Hangup
/// - `2` (SIGINT) - Interrupt (Ctrl+C)
/// - `9` (SIGKILL) - Force kill (cannot be caught)
/// - `15` (SIGTERM) - Graceful termination request
/// - `18` (SIGCONT) - Continue if stopped
/// - `19` (SIGSTOP) - Stop process (cannot be caught)
///
/// # Security
///
/// **This operation always requires elevated privileges** as it involves sending
/// signals to system processes. The framework's security middleware will validate
/// permissions before execution.
///
/// # Examples
///
/// ```rust
/// use airssys_osl::operations::ProcessSignalOperation;
/// use airssys_osl::core::operation::Operation;
///
/// // Send SIGTERM (graceful termination)
/// let op = ProcessSignalOperation::new(12345, 15);
/// assert_eq!(op.pid, 12345);
/// assert_eq!(op.signal, 15);
///
/// // Send SIGKILL (force kill)
/// let op = ProcessSignalOperation::new(12345, 9);
/// assert!(op.requires_elevated_privileges());
/// ```
#[derive(Debug, Clone)]
pub struct ProcessSignalOperation {
    /// Process ID to send signal to
    pub pid: u32,

    /// Signal number to send
    pub signal: i32,

    /// When this operation was created
    pub created_at: DateTime<Utc>,

    /// Optional operation ID
    pub operation_id: Option<String>,
}

impl ProcessSignalOperation {
    /// Create a new process signal operation.
    ///
    /// # Arguments
    ///
    /// * `pid` - Process ID to send signal to
    /// * `signal` - Signal number to send (e.g., 15 for SIGTERM, 9 for SIGKILL)
    ///
    /// # Examples
    ///
    /// ```rust
    /// use airssys_osl::operations::ProcessSignalOperation;
    ///
    /// // Send SIGTERM
    /// let op = ProcessSignalOperation::new(12345, 15);
    /// assert_eq!(op.pid, 12345);
    /// assert_eq!(op.signal, 15);
    /// ```
    pub fn new(pid: u32, signal: i32) -> Self {
        Self {
            pid,
            signal,
            created_at: Utc::now(),
            operation_id: None,
        }
    }

    /// Create a SIGTERM operation (graceful termination).
    ///
    /// # Examples
    ///
    /// ```rust
    /// use airssys_osl::operations::ProcessSignalOperation;
    ///
    /// let op = ProcessSignalOperation::terminate(12345);
    /// assert_eq!(op.signal, 15);
    /// ```
    pub fn terminate(pid: u32) -> Self {
        Self::new(pid, 15) // SIGTERM
    }

    /// Create a SIGKILL operation (force kill).
    ///
    /// # Examples
    ///
    /// ```rust
    /// use airssys_osl::operations::ProcessSignalOperation;
    ///
    /// let op = ProcessSignalOperation::kill(12345);
    /// assert_eq!(op.signal, 9);
    /// ```
    pub fn kill(pid: u32) -> Self {
        Self::new(pid, 9) // SIGKILL
    }

    /// Create a SIGHUP operation (hangup).
    ///
    /// # Examples
    ///
    /// ```rust
    /// use airssys_osl::operations::ProcessSignalOperation;
    ///
    /// let op = ProcessSignalOperation::hangup(12345);
    /// assert_eq!(op.signal, 1);
    /// ```
    pub fn hangup(pid: u32) -> Self {
        Self::new(pid, 1) // SIGHUP
    }

    /// Create with explicit timestamp (for testing).
    ///
    /// # Arguments
    ///
    /// * `pid` - Process ID to send signal to
    /// * `signal` - Signal number to send
    /// * `created_at` - Timestamp when the operation was created
    ///
    /// # Examples
    ///
    /// ```rust
    /// use airssys_osl::operations::ProcessSignalOperation;
    /// use chrono::Utc;
    ///
    /// let timestamp = Utc::now();
    /// let op = ProcessSignalOperation::with_timestamp(12345, 15, timestamp);
    /// assert_eq!(op.created_at, timestamp);
    /// ```
    pub fn with_timestamp(pid: u32, signal: i32, created_at: DateTime<Utc>) -> Self {
        Self {
            pid,
            signal,
            created_at,
            operation_id: None,
        }
    }

    /// Set custom operation ID.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use airssys_osl::operations::ProcessSignalOperation;
    /// use airssys_osl::core::operation::Operation;
    ///
    /// let op = ProcessSignalOperation::new(12345, 15)
    ///     .with_operation_id("my-signal-op");
    /// assert_eq!(op.operation_id(), "my-signal-op");
    /// ```
    pub fn with_operation_id(mut self, id: impl Into<String>) -> Self {
        self.operation_id = Some(id.into());
        self
    }
}

impl Operation for ProcessSignalOperation {
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
        true // Process signaling always requires elevation
    }
}

impl fmt::Display for ProcessSignalOperation {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "ProcessSignal(pid={}, signal={})", self.pid, self.signal)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_process_signal_operation_creation() {
        let op = ProcessSignalOperation::new(12345, 15);
        assert_eq!(op.pid, 12345);
        assert_eq!(op.signal, 15);
        assert_eq!(op.operation_type(), OperationType::Process);
    }

    #[test]
    fn test_process_signal_terminate() {
        let op = ProcessSignalOperation::terminate(12345);
        assert_eq!(op.signal, 15); // SIGTERM
    }

    #[test]
    fn test_process_signal_kill() {
        let op = ProcessSignalOperation::kill(12345);
        assert_eq!(op.signal, 9); // SIGKILL
    }

    #[test]
    fn test_process_signal_hangup() {
        let op = ProcessSignalOperation::hangup(12345);
        assert_eq!(op.signal, 1); // SIGHUP
    }

    #[test]
    fn test_process_signal_permissions() {
        let op = ProcessSignalOperation::new(12345, 15);
        let permissions = op.required_permissions();
        assert_eq!(permissions.len(), 1);
        assert_eq!(permissions[0], Permission::ProcessManage);
    }

    #[test]
    fn test_process_signal_requires_elevation() {
        let op = ProcessSignalOperation::new(12345, 15);
        assert!(op.requires_elevated_privileges());
    }

    #[test]
    fn test_process_signal_with_custom_id() {
        let op = ProcessSignalOperation::new(12345, 15)
            .with_operation_id("custom-signal");
        assert_eq!(op.operation_id(), "custom-signal");
    }

    #[test]
    fn test_process_signal_generated_id() {
        let op = ProcessSignalOperation::new(12345, 15);
        let id = op.operation_id();
        assert!(id.starts_with("process:"));
    }
}
