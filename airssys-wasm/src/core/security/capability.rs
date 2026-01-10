//! Capability types for security validation.
//!
//! This module defines the capability-based security model types.
//! Each capability represents a permission to perform specific actions
//! on specific resources.

/// Capability types for security validation.
///
/// # Example
///
/// ```rust
/// use airssys_wasm::core::security::capability::{
///     Capability, MessagingCapability, MessagingAction
/// };
///
/// let cap = Capability::Messaging(MessagingCapability {
///     action: MessagingAction::Send,
///     target_pattern: "org.example/*".to_string(),
/// });
/// ```
#[derive(Debug, Clone)]
pub enum Capability {
    /// Messaging-related capability.
    Messaging(MessagingCapability),
    /// Storage-related capability.
    Storage(StorageCapability),
    /// Filesystem-related capability.
    Filesystem(FilesystemCapability),
    /// Network-related capability.
    Network(NetworkCapability),
}

// --- Messaging ---

/// Messaging capability specification.
#[derive(Debug, Clone)]
pub struct MessagingCapability {
    /// The messaging action permitted.
    pub action: MessagingAction,
    /// Target component pattern (glob-style).
    pub target_pattern: String,
}

/// Messaging action types.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum MessagingAction {
    /// Send fire-and-forget messages.
    Send,
    /// Send request-response messages.
    Request,
    /// Broadcast to multiple targets.
    Broadcast,
}

// --- Storage ---

/// Storage capability specification.
#[derive(Debug, Clone)]
pub struct StorageCapability {
    /// The storage action permitted.
    pub action: StorageAction,
    /// Namespace pattern (glob-style).
    pub namespace_pattern: String,
}

/// Storage action types.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum StorageAction {
    /// Read from storage.
    Read,
    /// Write to storage.
    Write,
    /// Delete from storage.
    Delete,
}

// --- Filesystem ---

/// Filesystem capability specification.
#[derive(Debug, Clone)]
pub struct FilesystemCapability {
    /// The filesystem action permitted.
    pub action: FilesystemAction,
    /// Path pattern (glob-style).
    pub path_pattern: String,
}

/// Filesystem action types.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum FilesystemAction {
    /// Read files.
    Read,
    /// Write files.
    Write,
    /// Delete files.
    Delete,
    /// List directory contents.
    ListDir,
}

// --- Network ---

/// Network capability specification.
#[derive(Debug, Clone)]
pub struct NetworkCapability {
    /// The network action permitted.
    pub action: NetworkAction,
    /// Host pattern (glob-style).
    pub host_pattern: String,
    /// Optional specific port.
    pub port: Option<u16>,
}

/// Network action types.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum NetworkAction {
    /// Outbound network connections.
    Outbound,
    /// Inbound network connections.
    Inbound,
}

#[cfg(test)]
mod tests {
    use super::*;

    // Capability enum creation tests
    #[test]
    fn test_messaging_capability_creation() {
        let cap = Capability::Messaging(MessagingCapability {
            action: MessagingAction::Send,
            target_pattern: "org.example/*".to_string(),
        });
        assert!(matches!(cap, Capability::Messaging(_)));
    }

    #[test]
    fn test_storage_capability_creation() {
        let cap = Capability::Storage(StorageCapability {
            action: StorageAction::Read,
            namespace_pattern: "data/*".to_string(),
        });
        assert!(matches!(cap, Capability::Storage(_)));
    }

    #[test]
    fn test_filesystem_capability_creation() {
        let cap = Capability::Filesystem(FilesystemCapability {
            action: FilesystemAction::Write,
            path_pattern: "/tmp/*".to_string(),
        });
        assert!(matches!(cap, Capability::Filesystem(_)));
    }

    #[test]
    fn test_network_capability_creation() {
        let cap = Capability::Network(NetworkCapability {
            action: NetworkAction::Outbound,
            host_pattern: "api.example.com".to_string(),
            port: Some(443),
        });
        assert!(matches!(cap, Capability::Network(_)));
    }

    // Action enum equality tests
    #[test]
    fn test_messaging_action_equality() {
        assert_eq!(MessagingAction::Send, MessagingAction::Send);
        assert_ne!(MessagingAction::Send, MessagingAction::Request);
    }

    #[test]
    fn test_storage_action_equality() {
        assert_eq!(StorageAction::Read, StorageAction::Read);
        assert_ne!(StorageAction::Read, StorageAction::Write);
    }

    #[test]
    fn test_filesystem_action_equality() {
        assert_eq!(FilesystemAction::Read, FilesystemAction::Read);
        assert_ne!(FilesystemAction::Read, FilesystemAction::Write);
    }

    #[test]
    fn test_network_action_equality() {
        assert_eq!(NetworkAction::Outbound, NetworkAction::Outbound);
        assert_ne!(NetworkAction::Outbound, NetworkAction::Inbound);
    }

    // Struct field access tests
    #[test]
    fn test_messaging_capability_fields() {
        let cap = MessagingCapability {
            action: MessagingAction::Broadcast,
            target_pattern: "events/*".to_string(),
        };
        assert_eq!(cap.action, MessagingAction::Broadcast);
        assert_eq!(cap.target_pattern, "events/*");
    }

    #[test]
    fn test_storage_capability_fields() {
        let cap = StorageCapability {
            action: StorageAction::Delete,
            namespace_pattern: "cache/*".to_string(),
        };
        assert_eq!(cap.action, StorageAction::Delete);
        assert_eq!(cap.namespace_pattern, "cache/*");
    }

    #[test]
    fn test_filesystem_capability_fields() {
        let cap = FilesystemCapability {
            action: FilesystemAction::ListDir,
            path_pattern: "/logs/*".to_string(),
        };
        assert_eq!(cap.action, FilesystemAction::ListDir);
        assert_eq!(cap.path_pattern, "/logs/*");
    }

    #[test]
    fn test_network_capability_fields() {
        let cap = NetworkCapability {
            action: NetworkAction::Inbound,
            host_pattern: "localhost".to_string(),
            port: Some(8080),
        };
        assert_eq!(cap.action, NetworkAction::Inbound);
        assert_eq!(cap.host_pattern, "localhost");
        assert_eq!(cap.port, Some(8080));
    }

    // Gap analysis tests

    #[test]
    fn test_capability_debug_shows_inner_type() {
        let cap = Capability::Messaging(MessagingCapability {
            action: MessagingAction::Send,
            target_pattern: "test/*".to_string(),
        });
        let debug_str = format!("{:?}", cap);
        assert!(debug_str.contains("Messaging"));
        assert!(debug_str.contains("Send"));
    }

    #[test]
    fn test_capability_clone_creates_independent_copy() {
        let cap1 = Capability::Storage(StorageCapability {
            action: StorageAction::Write,
            namespace_pattern: "data/*".to_string(),
        });
        let cap2 = cap1.clone();

        assert!(matches!(cap2, Capability::Storage(_)));
    }
}
