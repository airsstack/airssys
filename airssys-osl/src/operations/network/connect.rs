//! Network connect operation.

// Layer 1: Standard library imports
use std::fmt;
use std::time::Duration;

// Layer 2: Third-party crate imports
use chrono::{DateTime, Utc};
use uuid::Uuid;

// Layer 3: Internal module imports
use crate::core::operation::{Operation, OperationType, Permission};

/// Operation to connect to a network endpoint.
///
/// Requires NetworkConnect permission, which is an elevated privilege. This operation
/// supports TCP connections with configurable timeout.
///
/// # Security
///
/// **This operation always requires elevated privileges** as it involves network
/// socket operations. The framework's security middleware will validate permissions
/// before execution.
///
/// # Examples
///
/// ```rust
/// use airssys_osl::operations::NetworkConnectOperation;
/// use std::time::Duration;
///
/// // Basic TCP connection
/// let op = NetworkConnectOperation::new("localhost:8080");
///
/// // Connection with timeout
/// let op = NetworkConnectOperation::new("example.com:443")
///     .with_timeout(Duration::from_secs(10));
/// ```
#[derive(Debug, Clone)]
pub struct NetworkConnectOperation {
    /// Address to connect to (e.g., "localhost:8080" or "192.168.1.1:3000")
    pub address: String,

    /// Connection timeout (None = use system default)
    pub timeout: Option<Duration>,

    /// When this operation was created
    pub created_at: DateTime<Utc>,

    /// Optional operation ID for tracking
    pub operation_id: Option<String>,
}

impl NetworkConnectOperation {
    /// Create a new network connect operation.
    ///
    /// # Arguments
    ///
    /// * `address` - Network address to connect to (host:port format)
    ///
    /// # Examples
    ///
    /// ```rust
    /// use airssys_osl::operations::NetworkConnectOperation;
    ///
    /// let op = NetworkConnectOperation::new("localhost:8080");
    /// assert_eq!(op.address, "localhost:8080");
    /// assert!(op.timeout.is_none());
    /// ```
    pub fn new(address: impl Into<String>) -> Self {
        Self {
            address: address.into(),
            timeout: None,
            created_at: Utc::now(),
            operation_id: None,
        }
    }

    /// Set connection timeout.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use airssys_osl::operations::NetworkConnectOperation;
    /// use std::time::Duration;
    ///
    /// let op = NetworkConnectOperation::new("localhost:8080")
    ///     .with_timeout(Duration::from_secs(5));
    /// assert_eq!(op.timeout, Some(Duration::from_secs(5)));
    /// ```
    pub fn with_timeout(mut self, timeout: Duration) -> Self {
        self.timeout = Some(timeout);
        self
    }

    /// Set a unique operation ID for tracking.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use airssys_osl::operations::NetworkConnectOperation;
    ///
    /// let op = NetworkConnectOperation::new("localhost:8080")
    ///     .with_operation_id("connect-123");
    /// assert_eq!(op.operation_id, Some("connect-123".to_string()));
    /// ```
    pub fn with_operation_id(mut self, id: impl Into<String>) -> Self {
        self.operation_id = Some(id.into());
        self
    }
}

impl Operation for NetworkConnectOperation {
    fn operation_type(&self) -> OperationType {
        OperationType::Network
    }

    fn required_permissions(&self) -> Vec<Permission> {
        vec![Permission::NetworkConnect(self.address.clone())]
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

impl fmt::Display for NetworkConnectOperation {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "NetworkConnect(address={})", self.address)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new_creates_operation_with_defaults() {
        let op = NetworkConnectOperation::new("localhost:8080");
        assert_eq!(op.address, "localhost:8080");
        assert!(op.timeout.is_none());
        assert!(op.operation_id.is_none());
    }

    #[test]
    fn test_with_timeout_sets_timeout() {
        let timeout = Duration::from_secs(10);
        let op = NetworkConnectOperation::new("localhost:8080").with_timeout(timeout);
        assert_eq!(op.timeout, Some(timeout));
    }

    #[test]
    fn test_with_operation_id_sets_id() {
        let op = NetworkConnectOperation::new("localhost:8080").with_operation_id("test-123");
        assert_eq!(op.operation_id, Some("test-123".to_string()));
    }

    #[test]
    fn test_operation_type_is_network() {
        let op = NetworkConnectOperation::new("localhost:8080");
        assert_eq!(op.operation_type(), OperationType::Network);
    }

    #[test]
    fn test_required_permissions() {
        let op = NetworkConnectOperation::new("localhost:8080");
        let perms = op.required_permissions();
        assert_eq!(perms.len(), 1);
        assert_eq!(
            perms[0],
            Permission::NetworkConnect("localhost:8080".to_string())
        );
    }

    #[test]
    fn test_requires_elevated_privileges() {
        let op = NetworkConnectOperation::new("localhost:8080");
        assert!(op.requires_elevated_privileges());
    }

    #[test]
    fn test_operation_id_custom() {
        let op = NetworkConnectOperation::new("localhost:8080").with_operation_id("custom-id");
        assert_eq!(op.operation_id(), "custom-id");
    }

    #[test]
    fn test_display_implementation() {
        let op = NetworkConnectOperation::new("localhost:8080");
        assert_eq!(format!("{op}"), "NetworkConnect(address=localhost:8080)");
    }

    #[test]
    fn test_clone_preserves_data() {
        let op = NetworkConnectOperation::new("localhost:8080")
            .with_timeout(Duration::from_secs(5))
            .with_operation_id("test-clone");
        let cloned = op.clone();
        assert_eq!(cloned.address, op.address);
        assert_eq!(cloned.timeout, op.timeout);
        assert_eq!(cloned.operation_id, op.operation_id);
    }
}
