//! Network socket operation.

// Layer 1: Standard library imports
use std::fmt;

// Layer 2: Third-party crate imports
use chrono::{DateTime, Utc};
use uuid::Uuid;

// Layer 3: Internal module imports
use crate::core::operation::{Operation, OperationType, Permission};

/// Operation to create a network socket.
///
/// Requires NetworkSocket permission, which is an elevated privilege. This operation
/// supports creating different types of network sockets (TCP, UDP, etc.).
///
/// # Security
///
/// **This operation always requires elevated privileges** as it involves creating
/// network sockets. The framework's security middleware will validate permissions
/// before execution.
///
/// # Examples
///
/// ```rust
/// use airssys_osl::operations::NetworkSocketOperation;
///
/// // Create TCP socket
/// let op = NetworkSocketOperation::new("tcp");
///
/// // Create UDP socket
/// let op = NetworkSocketOperation::new("udp");
///
/// // Create Unix domain socket
/// let op = NetworkSocketOperation::new("unix");
/// ```
#[derive(Debug, Clone)]
pub struct NetworkSocketOperation {
    /// Socket type (e.g., "tcp", "udp", "unix")
    pub socket_type: String,

    /// When this operation was created
    pub created_at: DateTime<Utc>,

    /// Optional operation ID for tracking
    pub operation_id: Option<String>,
}

impl NetworkSocketOperation {
    /// Create a new network socket operation.
    ///
    /// # Arguments
    ///
    /// * `socket_type` - Type of socket to create (e.g., "tcp", "udp", "unix")
    ///
    /// # Examples
    ///
    /// ```rust
    /// use airssys_osl::operations::NetworkSocketOperation;
    ///
    /// let op = NetworkSocketOperation::new("tcp");
    /// assert_eq!(op.socket_type, "tcp");
    /// ```
    pub fn new(socket_type: impl Into<String>) -> Self {
        Self {
            socket_type: socket_type.into(),
            created_at: Utc::now(),
            operation_id: None,
        }
    }

    /// Create a TCP socket operation.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use airssys_osl::operations::NetworkSocketOperation;
    ///
    /// let op = NetworkSocketOperation::tcp();
    /// assert_eq!(op.socket_type, "tcp");
    /// ```
    pub fn tcp() -> Self {
        Self::new("tcp")
    }

    /// Create a UDP socket operation.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use airssys_osl::operations::NetworkSocketOperation;
    ///
    /// let op = NetworkSocketOperation::udp();
    /// assert_eq!(op.socket_type, "udp");
    /// ```
    pub fn udp() -> Self {
        Self::new("udp")
    }

    /// Create a Unix domain socket operation.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use airssys_osl::operations::NetworkSocketOperation;
    ///
    /// let op = NetworkSocketOperation::unix();
    /// assert_eq!(op.socket_type, "unix");
    /// ```
    pub fn unix() -> Self {
        Self::new("unix")
    }

    /// Set a unique operation ID for tracking.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use airssys_osl::operations::NetworkSocketOperation;
    ///
    /// let op = NetworkSocketOperation::new("tcp")
    ///     .with_operation_id("socket-123");
    /// assert_eq!(op.operation_id, Some("socket-123".to_string()));
    /// ```
    pub fn with_operation_id(mut self, id: impl Into<String>) -> Self {
        self.operation_id = Some(id.into());
        self
    }
}

impl Operation for NetworkSocketOperation {
    fn operation_type(&self) -> OperationType {
        OperationType::Network
    }

    fn required_permissions(&self) -> Vec<Permission> {
        vec![Permission::NetworkSocket]
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

impl fmt::Display for NetworkSocketOperation {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "NetworkSocket(type={})", self.socket_type)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new_creates_operation() {
        let op = NetworkSocketOperation::new("tcp");
        assert_eq!(op.socket_type, "tcp");
        assert!(op.operation_id.is_none());
    }

    #[test]
    fn test_tcp_constructor() {
        let op = NetworkSocketOperation::tcp();
        assert_eq!(op.socket_type, "tcp");
    }

    #[test]
    fn test_udp_constructor() {
        let op = NetworkSocketOperation::udp();
        assert_eq!(op.socket_type, "udp");
    }

    #[test]
    fn test_unix_constructor() {
        let op = NetworkSocketOperation::unix();
        assert_eq!(op.socket_type, "unix");
    }

    #[test]
    fn test_with_operation_id_sets_id() {
        let op = NetworkSocketOperation::new("tcp").with_operation_id("test-123");
        assert_eq!(op.operation_id, Some("test-123".to_string()));
    }

    #[test]
    fn test_operation_type_is_network() {
        let op = NetworkSocketOperation::new("tcp");
        assert_eq!(op.operation_type(), OperationType::Network);
    }

    #[test]
    fn test_required_permissions() {
        let op = NetworkSocketOperation::new("tcp");
        let perms = op.required_permissions();
        assert_eq!(perms.len(), 1);
        assert_eq!(perms[0], Permission::NetworkSocket);
    }

    #[test]
    fn test_requires_elevated_privileges() {
        let op = NetworkSocketOperation::new("tcp");
        assert!(op.requires_elevated_privileges());
    }

    #[test]
    fn test_operation_id_custom() {
        let op = NetworkSocketOperation::new("tcp").with_operation_id("custom-id");
        assert_eq!(op.operation_id(), "custom-id");
    }

    #[test]
    fn test_display_implementation() {
        let op = NetworkSocketOperation::new("tcp");
        assert_eq!(format!("{op}"), "NetworkSocket(type=tcp)");
    }

    #[test]
    fn test_clone_preserves_data() {
        let op = NetworkSocketOperation::new("tcp").with_operation_id("test-clone");
        let cloned = op.clone();
        assert_eq!(cloned.socket_type, op.socket_type);
        assert_eq!(cloned.operation_id, op.operation_id);
    }
}
