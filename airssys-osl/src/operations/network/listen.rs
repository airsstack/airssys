//! Network listen operation.

// Layer 1: Standard library imports
use std::fmt;

// Layer 2: Third-party crate imports
use chrono::{DateTime, Utc};
use uuid::Uuid;

// Layer 3: Internal module imports
use crate::core::operation::{Operation, OperationType, Permission};

/// Operation to listen on a network address.
///
/// Requires NetworkSocket permission, which is an elevated privilege. This operation
/// supports creating a TCP listener on a specified address with configurable backlog.
///
/// # Security
///
/// **This operation always requires elevated privileges** as it involves binding
/// to network ports. The framework's security middleware will validate permissions
/// before execution.
///
/// # Examples
///
/// ```rust
/// use airssys_osl::operations::NetworkListenOperation;
///
/// // Basic TCP listener
/// let op = NetworkListenOperation::new("0.0.0.0:8080");
///
/// // Listener with custom backlog
/// let op = NetworkListenOperation::new("127.0.0.1:3000")
///     .with_backlog(128);
///
/// // Unix domain socket listener
/// let op = NetworkListenOperation::new("unix-socket")
///     .with_socket_path("/tmp/my.sock")
///     .with_backlog(64);
/// ```
#[derive(Debug, Clone)]
pub struct NetworkListenOperation {
    /// Address to listen on (e.g., "0.0.0.0:8080" or "127.0.0.1:3000")
    /// For Unix domain sockets, this can be empty or descriptive
    pub address: String,

    /// Socket file path for Unix domain sockets (e.g., "/tmp/my.sock")
    /// When set, this operation will create a Unix domain socket at this path
    pub socket_path: Option<String>,

    /// Connection backlog size (None = use system default)
    pub backlog: Option<i32>,

    /// When this operation was created
    pub created_at: DateTime<Utc>,

    /// Optional operation ID for tracking
    pub operation_id: Option<String>,
}

impl NetworkListenOperation {
    /// Create a new network listen operation.
    ///
    /// # Arguments
    ///
    /// * `address` - Network address to listen on (host:port format)
    ///
    /// # Examples
    ///
    /// ```rust
    /// use airssys_osl::operations::NetworkListenOperation;
    ///
    /// let op = NetworkListenOperation::new("0.0.0.0:8080");
    /// assert_eq!(op.address, "0.0.0.0:8080");
    /// assert!(op.backlog.is_none());
    /// ```
    pub fn new(address: impl Into<String>) -> Self {
        Self {
            address: address.into(),
            socket_path: None,
            backlog: None,
            created_at: Utc::now(),
            operation_id: None,
        }
    }

    /// Set the socket file path for Unix domain sockets.
    ///
    /// When set, this operation will create a Unix domain socket at the specified
    /// path instead of a TCP listener. This requires both NetworkSocket and
    /// FilesystemWrite permissions.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use airssys_osl::operations::NetworkListenOperation;
    ///
    /// let op = NetworkListenOperation::new("unix-socket")
    ///     .with_socket_path("/tmp/my.sock");
    /// assert_eq!(op.socket_path, Some("/tmp/my.sock".to_string()));
    /// ```
    pub fn with_socket_path(mut self, path: impl Into<String>) -> Self {
        self.socket_path = Some(path.into());
        self
    }

    /// Set the connection backlog size.
    ///
    /// The backlog defines the maximum number of pending connections
    /// that can be queued.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use airssys_osl::operations::NetworkListenOperation;
    ///
    /// let op = NetworkListenOperation::new("0.0.0.0:8080")
    ///     .with_backlog(128);
    /// assert_eq!(op.backlog, Some(128));
    /// ```
    pub fn with_backlog(mut self, backlog: i32) -> Self {
        self.backlog = Some(backlog);
        self
    }

    /// Set a unique operation ID for tracking.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use airssys_osl::operations::NetworkListenOperation;
    ///
    /// let op = NetworkListenOperation::new("0.0.0.0:8080")
    ///     .with_operation_id("listen-123");
    /// assert_eq!(op.operation_id, Some("listen-123".to_string()));
    /// ```
    pub fn with_operation_id(mut self, id: impl Into<String>) -> Self {
        self.operation_id = Some(id.into());
        self
    }
}

impl Operation for NetworkListenOperation {
    fn operation_type(&self) -> OperationType {
        OperationType::Network
    }

    fn required_permissions(&self) -> Vec<Permission> {
        let mut perms = vec![Permission::NetworkSocket];

        // Unix domain sockets need filesystem write permission for the socket file
        if let Some(path) = &self.socket_path {
            perms.push(Permission::FilesystemWrite(path.clone()));
        }

        perms
    }

    fn created_at(&self) -> DateTime<Utc> {
        self.created_at
    }

    fn operation_id(&self) -> String {
        self.operation_id
            .clone()
            .unwrap_or_else(|| Uuid::new_v4().to_string())
    }

    fn requires_elevated_privileges(&self) -> bool {
        true // Network operations are privileged
    }
}

impl fmt::Display for NetworkListenOperation {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match (&self.socket_path, self.backlog) {
            (Some(path), Some(backlog)) => {
                write!(f, "NetworkListen(socket_path={path}, backlog={backlog})")
            }
            (Some(path), None) => {
                write!(f, "NetworkListen(socket_path={path})")
            }
            (None, Some(backlog)) => {
                write!(
                    f,
                    "NetworkListen(address={}, backlog={backlog})",
                    self.address
                )
            }
            (None, None) => {
                write!(f, "NetworkListen(address={})", self.address)
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new_creates_operation_with_defaults() {
        let op = NetworkListenOperation::new("0.0.0.0:8080");
        assert_eq!(op.address, "0.0.0.0:8080");
        assert!(op.socket_path.is_none());
        assert!(op.backlog.is_none());
        assert!(op.operation_id.is_none());
    }

    #[test]
    fn test_with_socket_path_sets_path() {
        let op = NetworkListenOperation::new("unix-socket").with_socket_path("/tmp/my.sock");
        assert_eq!(op.socket_path, Some("/tmp/my.sock".to_string()));
    }

    #[test]
    fn test_with_backlog_sets_backlog() {
        let op = NetworkListenOperation::new("0.0.0.0:8080").with_backlog(128);
        assert_eq!(op.backlog, Some(128));
    }

    #[test]
    fn test_with_operation_id_sets_id() {
        let op = NetworkListenOperation::new("0.0.0.0:8080").with_operation_id("test-123");
        assert_eq!(op.operation_id, Some("test-123".to_string()));
    }

    #[test]
    fn test_operation_type_is_network() {
        let op = NetworkListenOperation::new("0.0.0.0:8080");
        assert_eq!(op.operation_type(), OperationType::Network);
    }

    #[test]
    fn test_required_permissions() {
        let op = NetworkListenOperation::new("0.0.0.0:8080");
        let perms = op.required_permissions();
        assert_eq!(perms.len(), 1);
        assert_eq!(perms[0], Permission::NetworkSocket);
    }

    #[test]
    fn test_required_permissions_with_unix_socket() {
        let op = NetworkListenOperation::new("unix-socket").with_socket_path("/tmp/my.sock");
        let perms = op.required_permissions();
        assert_eq!(perms.len(), 2);
        assert_eq!(perms[0], Permission::NetworkSocket);
        assert_eq!(
            perms[1],
            Permission::FilesystemWrite("/tmp/my.sock".to_string())
        );
    }

    #[test]
    fn test_requires_elevated_privileges() {
        let op = NetworkListenOperation::new("0.0.0.0:8080");
        assert!(op.requires_elevated_privileges());
    }

    #[test]
    fn test_operation_id_custom() {
        let op = NetworkListenOperation::new("0.0.0.0:8080").with_operation_id("custom-id");
        assert_eq!(op.operation_id(), "custom-id");
    }

    #[test]
    fn test_display_implementation_without_backlog() {
        let op = NetworkListenOperation::new("0.0.0.0:8080");
        assert_eq!(format!("{op}"), "NetworkListen(address=0.0.0.0:8080)");
    }

    #[test]
    fn test_display_implementation_with_backlog() {
        let op = NetworkListenOperation::new("0.0.0.0:8080").with_backlog(128);
        assert_eq!(
            format!("{op}"),
            "NetworkListen(address=0.0.0.0:8080, backlog=128)"
        );
    }

    #[test]
    fn test_display_implementation_unix_socket() {
        let op = NetworkListenOperation::new("unix-socket").with_socket_path("/tmp/my.sock");
        assert_eq!(format!("{op}"), "NetworkListen(socket_path=/tmp/my.sock)");
    }

    #[test]
    fn test_display_implementation_unix_socket_with_backlog() {
        let op = NetworkListenOperation::new("unix-socket")
            .with_socket_path("/tmp/my.sock")
            .with_backlog(64);
        assert_eq!(
            format!("{op}"),
            "NetworkListen(socket_path=/tmp/my.sock, backlog=64)"
        );
    }

    #[test]
    fn test_clone_preserves_data() {
        let op = NetworkListenOperation::new("0.0.0.0:8080")
            .with_backlog(128)
            .with_operation_id("test-clone");
        let cloned = op.clone();
        assert_eq!(cloned.address, op.address);
        assert_eq!(cloned.backlog, op.backlog);
        assert_eq!(cloned.operation_id, op.operation_id);
    }

    #[test]
    fn test_clone_preserves_unix_socket_data() {
        let op = NetworkListenOperation::new("unix-socket")
            .with_socket_path("/tmp/test.sock")
            .with_backlog(64)
            .with_operation_id("test-clone");
        let cloned = op.clone();
        assert_eq!(cloned.address, op.address);
        assert_eq!(cloned.socket_path, op.socket_path);
        assert_eq!(cloned.backlog, op.backlog);
        assert_eq!(cloned.operation_id, op.operation_id);
    }
}
