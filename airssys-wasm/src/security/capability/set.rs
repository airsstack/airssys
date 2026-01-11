//! Capability set management for components.

use super::types::PatternMatcher;

/// Messaging permission configuration.
#[derive(Debug, Clone, PartialEq)]
pub struct MessagingPermission {
    /// Component patterns that can be sent to.
    pub can_send_to: Vec<String>,
    /// Component patterns that can be received from.
    pub can_receive_from: Vec<String>,
}

/// Storage permission configuration.
#[derive(Debug, Clone, PartialEq)]
pub struct StoragePermission {
    /// Key patterns that can be written.
    pub can_write_keys: Vec<String>,
    /// Key patterns that can be read.
    pub can_read_keys: Vec<String>,
}

/// Filesystem permission configuration.
#[derive(Debug, Clone, PartialEq)]
pub struct FilesystemPermission {
    /// Path patterns that can be read.
    pub can_read_paths: Vec<String>,
    /// Path patterns that can be written.
    pub can_write_paths: Vec<String>,
}

/// Network permission configuration.
#[derive(Debug, Clone, PartialEq)]
pub struct NetworkPermission {
    /// Host patterns that can be connected to.
    pub can_connect_to: Vec<String>,
    /// Ports that can be bound to.
    pub can_bind_ports: Vec<u16>,
}

/// Set of capabilities granted to a component.
///
/// Manages component permissions across messaging, storage, filesystem, and network.
#[derive(Debug, Clone, Default)]
pub struct CapabilitySet {
    messaging: Vec<MessagingPermission>,
    storage: Vec<StoragePermission>,
    filesystem: Vec<FilesystemPermission>,
    network: Vec<NetworkPermission>,
}

impl CapabilitySet {
    /// Create a new empty capability set.
    ///
    /// # Examples
    ///
    /// ```
    /// use airssys_wasm::security::capability::set::CapabilitySet;
    ///
    /// let set = CapabilitySet::new();
    /// assert!(!set.can_send_to("any"));
    /// ```
    pub fn new() -> Self {
        Self::default()
    }

    /// Create a new builder for constructing a CapabilitySet.
    ///
    /// # Examples
    ///
    /// ```
    /// use airssys_wasm::security::capability::set::{CapabilitySet, MessagingPermission};
    ///
    /// let capabilities = CapabilitySet::builder()
    ///     .messaging(MessagingPermission {
    ///         can_send_to: vec!["comp-a/*".to_string()],
    ///         can_receive_from: vec![],
    ///     })
    ///     .build();
    ///
    /// assert!(capabilities.can_send_to("comp-a/1"));
    /// ```
    pub fn builder() -> CapabilitySetBuilder {
        CapabilitySetBuilder::new()
    }

    /// Add a messaging permission.
    pub fn add_messaging(&mut self, perm: MessagingPermission) {
        self.messaging.push(perm);
    }

    /// Add a storage permission.
    pub fn add_storage(&mut self, perm: StoragePermission) {
        self.storage.push(perm);
    }

    /// Add a filesystem permission.
    pub fn add_filesystem(&mut self, perm: FilesystemPermission) {
        self.filesystem.push(perm);
    }

    /// Add a network permission.
    pub fn add_network(&mut self, perm: NetworkPermission) {
        self.network.push(perm);
    }

    /// Check if messaging to target is allowed.
    pub fn can_send_to(&self, target: &str) -> bool {
        for perm in &self.messaging {
            for pattern in &perm.can_send_to {
                if PatternMatcher::matches(pattern, target) {
                    return true;
                }
            }
        }
        false
    }

    /// Check if receiving from source is allowed.
    pub fn can_receive_from(&self, source: &str) -> bool {
        for perm in &self.messaging {
            for pattern in &perm.can_receive_from {
                if PatternMatcher::matches(pattern, source) {
                    return true;
                }
            }
        }
        false
    }

    /// Check if storage key can be read.
    pub fn can_read_key(&self, key: &str) -> bool {
        for perm in &self.storage {
            for pattern in &perm.can_read_keys {
                if PatternMatcher::matches(pattern, key) {
                    return true;
                }
            }
        }
        false
    }

    /// Check if storage key can be written.
    pub fn can_write_key(&self, key: &str) -> bool {
        for perm in &self.storage {
            for pattern in &perm.can_write_keys {
                if PatternMatcher::matches(pattern, key) {
                    return true;
                }
            }
        }
        false
    }

    /// Check if filesystem path can be read.
    pub fn can_read_path(&self, path: &str) -> bool {
        for perm in &self.filesystem {
            for pattern in &perm.can_read_paths {
                if PatternMatcher::matches(pattern, path) {
                    return true;
                }
            }
        }
        false
    }

    /// Check if filesystem path can be written.
    pub fn can_write_path(&self, path: &str) -> bool {
        for perm in &self.filesystem {
            for pattern in &perm.can_write_paths {
                if PatternMatcher::matches(pattern, path) {
                    return true;
                }
            }
        }
        false
    }

    /// Check if network connection is allowed.
    pub fn can_connect_to(&self, host: &str) -> bool {
        for perm in &self.network {
            for pattern in &perm.can_connect_to {
                if PatternMatcher::matches(pattern, host) {
                    return true;
                }
            }
        }
        false
    }

    /// Check if port binding is allowed.
    pub fn can_bind_port(&self, port: u16) -> bool {
        for perm in &self.network {
            if perm.can_bind_ports.contains(&port) {
                return true;
            }
        }
        false
    }
}

/// Builder for constructing CapabilitySet instances.
///
/// Provides a fluent API for creating complex permission sets.
#[derive(Debug, Default)]
pub struct CapabilitySetBuilder {
    messaging: Vec<MessagingPermission>,
    storage: Vec<StoragePermission>,
    filesystem: Vec<FilesystemPermission>,
    network: Vec<NetworkPermission>,
}

impl CapabilitySetBuilder {
    /// Create a new CapabilitySetBuilder.
    pub fn new() -> Self {
        Self::default()
    }

    /// Add a messaging permission.
    ///
    /// # Examples
    ///
    /// ```
    /// use airssys_wasm::security::capability::set::{CapabilitySet, MessagingPermission};
    ///
    /// let capabilities = CapabilitySet::builder()
    ///     .messaging(MessagingPermission {
    ///         can_send_to: vec!["*".to_string()],
    ///         can_receive_from: vec![],
    ///     })
    ///     .build();
    ///
    /// assert!(capabilities.can_send_to("any"));
    /// ```
    pub fn messaging(mut self, perm: MessagingPermission) -> Self {
        self.messaging.push(perm);
        self
    }

    /// Add a storage permission.
    pub fn storage(mut self, perm: StoragePermission) -> Self {
        self.storage.push(perm);
        self
    }

    /// Add a filesystem permission.
    pub fn filesystem(mut self, perm: FilesystemPermission) -> Self {
        self.filesystem.push(perm);
        self
    }

    /// Add a network permission.
    pub fn network(mut self, perm: NetworkPermission) -> Self {
        self.network.push(perm);
        self
    }

    /// Build the CapabilitySet.
    ///
    /// # Examples
    ///
    /// ```
    /// use airssys_wasm::security::capability::set::{CapabilitySet, MessagingPermission, StoragePermission};
    ///
    /// let capabilities = CapabilitySet::builder()
    ///     .messaging(MessagingPermission {
    ///         can_send_to: vec!["comp-a/*".to_string()],
    ///         can_receive_from: vec![],
    ///     })
    ///     .storage(StoragePermission {
    ///         can_write_keys: vec!["user/*".to_string()],
    ///         can_read_keys: vec!["*".to_string()],
    ///     })
    ///     .build();
    ///
    /// assert!(capabilities.can_send_to("comp-a/1"));
    /// assert!(capabilities.can_write_key("user/1"));
    /// ```
    pub fn build(self) -> CapabilitySet {
        CapabilitySet {
            messaging: self.messaging,
            storage: self.storage,
            filesystem: self.filesystem,
            network: self.network,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_create_empty_capability_set() {
        let set = CapabilitySet::new();
        assert!(!set.can_send_to("any"));
        assert!(!set.can_read_key("any"));
    }

    #[test]
    fn test_add_and_check_messaging_permission() {
        let mut set = CapabilitySet::new();
        set.add_messaging(MessagingPermission {
            can_send_to: vec!["comp-a/*".to_string()],
            can_receive_from: vec!["comp-b/*".to_string()],
        });

        assert!(set.can_send_to("comp-a/1"));
        assert!(!set.can_send_to("comp-b/1"));
        assert!(set.can_receive_from("comp-b/1"));
        assert!(!set.can_receive_from("comp-a/1"));
    }

    #[test]
    fn test_add_and_check_storage_permission() {
        let mut set = CapabilitySet::new();
        set.add_storage(StoragePermission {
            can_write_keys: vec!["user/*".to_string()],
            can_read_keys: vec!["*".to_string()],
        });

        assert!(set.can_write_key("user/1"));
        assert!(!set.can_write_key("system/1"));
        assert!(set.can_read_key("user/1"));
        assert!(set.can_read_key("system/1"));
    }

    #[test]
    fn test_pattern_based_permission_matching() {
        let mut set = CapabilitySet::new();
        set.add_filesystem(FilesystemPermission {
            can_read_paths: vec!["/safe/*".to_string()],
            can_write_paths: vec!["/data/*".to_string()],
        });

        assert!(set.can_read_path("/safe/file.txt"));
        assert!(!set.can_read_path("/unsafe/file.txt"));
        assert!(set.can_write_path("/data/file.txt"));
        assert!(!set.can_write_path("/safe/file.txt"));
    }

    #[test]
    fn test_permission_denial() {
        let set = CapabilitySet::new();
        assert!(!set.can_send_to("any"));
        assert!(!set.can_receive_from("any"));
        assert!(!set.can_read_key("any"));
        assert!(!set.can_write_key("any"));
        assert!(!set.can_read_path("/any"));
        assert!(!set.can_write_path("/any"));
        assert!(!set.can_connect_to("host"));
        assert!(!set.can_bind_port(8080));
    }

    #[test]
    fn test_multiple_permissions() {
        let mut set = CapabilitySet::new();
        set.add_messaging(MessagingPermission {
            can_send_to: vec!["comp-a/*".to_string()],
            can_receive_from: vec![],
        });
        set.add_messaging(MessagingPermission {
            can_send_to: vec!["comp-b/*".to_string()],
            can_receive_from: vec![],
        });

        assert!(set.can_send_to("comp-a/1"));
        assert!(set.can_send_to("comp-b/1"));
        assert!(!set.can_send_to("comp-c/1"));
    }

    #[test]
    fn test_network_permissions() {
        let mut set = CapabilitySet::new();
        set.add_network(NetworkPermission {
            can_connect_to: vec!["api.example.com".to_string(), "*.internal".to_string()],
            can_bind_ports: vec![8080, 3000],
        });

        assert!(set.can_connect_to("api.example.com"));
        assert!(set.can_connect_to("service.internal"));
        assert!(!set.can_connect_to("other.com"));
        assert!(set.can_bind_port(8080));
        assert!(set.can_bind_port(3000));
        assert!(!set.can_bind_port(9090));
    }

    #[test]
    fn test_wildcard_permission() {
        let mut set = CapabilitySet::new();
        set.add_messaging(MessagingPermission {
            can_send_to: vec!["*".to_string()],
            can_receive_from: vec![],
        });

        assert!(set.can_send_to("anything"));
        assert!(set.can_send_to(""));
    }

    // Builder pattern tests

    #[test]
    fn test_builder_single_messaging_permission() {
        let capabilities = CapabilitySet::builder()
            .messaging(MessagingPermission {
                can_send_to: vec!["comp-a/*".to_string()],
                can_receive_from: vec![],
            })
            .build();

        assert!(capabilities.can_send_to("comp-a/1"));
        assert!(!capabilities.can_send_to("comp-b/1"));
    }

    #[test]
    fn test_builder_multiple_permissions() {
        let capabilities = CapabilitySet::builder()
            .messaging(MessagingPermission {
                can_send_to: vec!["comp-a/*".to_string()],
                can_receive_from: vec![],
            })
            .messaging(MessagingPermission {
                can_send_to: vec!["comp-b/*".to_string()],
                can_receive_from: vec![],
            })
            .build();

        assert!(capabilities.can_send_to("comp-a/1"));
        assert!(capabilities.can_send_to("comp-b/1"));
        assert!(!capabilities.can_send_to("comp-c/1"));
    }

    #[test]
    fn test_builder_all_permission_types() {
        let capabilities = CapabilitySet::builder()
            .messaging(MessagingPermission {
                can_send_to: vec!["*".to_string()],
                can_receive_from: vec![],
            })
            .storage(StoragePermission {
                can_write_keys: vec!["user/*".to_string()],
                can_read_keys: vec!["*".to_string()],
            })
            .filesystem(FilesystemPermission {
                can_read_paths: vec!["/safe/*".to_string()],
                can_write_paths: vec![],
            })
            .network(NetworkPermission {
                can_connect_to: vec!["*.internal".to_string()],
                can_bind_ports: vec![8080],
            })
            .build();

        assert!(capabilities.can_send_to("any"));
        assert!(capabilities.can_write_key("user/1"));
        assert!(capabilities.can_read_key("system/1"));
        assert!(capabilities.can_read_path("/safe/file.txt"));
        assert!(capabilities.can_connect_to("service.internal"));
        assert!(capabilities.can_bind_port(8080));
    }

    #[test]
    fn test_builder_empty_set() {
        let capabilities = CapabilitySet::builder().build();

        assert!(!capabilities.can_send_to("any"));
        assert!(!capabilities.can_read_key("any"));
        assert!(!capabilities.can_read_path("/any"));
        assert!(!capabilities.can_connect_to("host"));
    }
}
