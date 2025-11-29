//! Permission types for Component.toml manifest declarations.
//!
//! This module provides types for parsing and representing component permissions
//! declared in Component.toml manifests. These permissions are checked at runtime
//! against capability-based security policies.
//!
//! # Permission Model
//!
//! Follows ADR-WASM-005 (Capability-Based Security Model):
//! - Components declare required permissions in Component.toml
//! - Host parses and validates permissions at load time
//! - Runtime enforces permissions at every host function call
//! - Deny-by-default: no permission means no access
//!
//! # Examples
//!
//! ## Component.toml Declaration
//!
//! ```toml
//! [permissions.filesystem]
//! read = ["/data/**", "/config/*.json"]
//! write = ["/output/**"]
//!
//! [permissions.network]
//! outbound = [
//!     { host = "api.example.com", port = 443 },
//!     { host = "*.cdn.example.com", port = 443 }
//! ]
//!
//! [permissions.storage]
//! namespaces = ["myapp:cache", "myapp:config"]
//! max_size_mb = 100
//! ```
//!
//! ## Parsing Permissions
//!
//! ```rust
//! use airssys_wasm::core::permission::{PermissionManifest, FilesystemPermissions};
//!
//! let toml_content = r#"
//! [permissions.filesystem]
//! read = ["/data/**"]
//! write = ["/output/**"]
//! "#;
//!
//! let manifest: PermissionManifest = toml::from_str(toml_content).unwrap();
//! assert_eq!(manifest.filesystem.read.len(), 1);
//! ```
//!
//! # References
//!
//! - **ADR-WASM-005**: Capability-Based Security Model
//! - **KNOWLEDGE-WASM-009**: Component Installation Architecture

// Layer 1: Standard library imports
use std::collections::HashSet;

// Layer 2: Third-party crate imports
use serde::{Deserialize, Serialize};

/// Complete permission manifest from Component.toml.
///
/// This structure represents all permissions a component requests.
/// The host validates these permissions before loading the component.
///
/// # Structure
///
/// ```toml
/// [permissions]
/// # All permission categories nested under this section
///
/// [permissions.filesystem]
/// read = ["/data/**"]
/// write = ["/output/**"]
/// delete = ["/tmp/cache/*"]
/// list = ["/data"]
///
/// [permissions.network]
/// outbound = [
///     { host = "api.example.com", port = 443 }
/// ]
///
/// [permissions.storage]
/// namespaces = ["myapp:cache"]
/// max_size_mb = 100
/// ```
#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq)]
#[serde(deny_unknown_fields)]
pub struct PermissionManifest {
    /// Filesystem access permissions (read, write, delete, list)
    #[serde(default)]
    pub filesystem: FilesystemPermissions,

    /// Network access permissions (outbound, inbound)
    #[serde(default)]
    pub network: NetworkPermissions,

    /// Storage access permissions (namespaces, quotas)
    #[serde(default)]
    pub storage: StoragePermissions,
}

/// Filesystem permission declarations.
///
/// Uses glob pattern matching for flexible path-based access control.
///
/// # Pattern Syntax
///
/// - `*` - Matches any characters except `/` (single directory level)
/// - `**` - Matches any characters including `/` (recursive)
/// - `?` - Matches single character
/// - `[abc]` - Matches one of the characters
///
/// # Examples
///
/// ```rust
/// use airssys_wasm::core::permission::FilesystemPermissions;
///
/// let perms = FilesystemPermissions {
///     read: vec!["/data/**".to_string(), "/config/*.json".to_string()],
///     write: vec!["/output/**".to_string()],
///     delete: vec!["/tmp/cache/*".to_string()],
///     list: vec!["/data".to_string()],
/// };
///
/// assert_eq!(perms.read.len(), 2);
/// ```
#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq)]
#[serde(deny_unknown_fields)]
pub struct FilesystemPermissions {
    /// File read permissions (glob patterns)
    ///
    /// Examples:
    /// - `"/etc/myapp/config.toml"` - Specific file
    /// - `"/etc/myapp/*.toml"` - All .toml files in directory
    /// - `"/var/data/myapp/**"` - All files recursively
    #[serde(default)]
    pub read: Vec<String>,

    /// File write permissions (glob patterns)
    ///
    /// Examples:
    /// - `"/var/data/myapp/output/**"`
    /// - `"/tmp/myapp/cache/*"`
    #[serde(default)]
    pub write: Vec<String>,

    /// File delete permissions (glob patterns)
    ///
    /// Examples:
    /// - `"/tmp/myapp/cache/*.tmp"`
    /// - `"/var/data/myapp/temp/**"`
    #[serde(default)]
    pub delete: Vec<String>,

    /// Directory list permissions (glob patterns)
    ///
    /// Examples:
    /// - `"/var/data/myapp"`
    /// - `"/etc/myapp"`
    #[serde(default)]
    pub list: Vec<String>,
}

/// Network permission declarations.
///
/// Uses wildcard domain matching and port specifications for network access control.
///
/// # Domain Pattern Syntax
///
/// - Exact domain: `"api.example.com"`
/// - Wildcard subdomain: `"*.example.com"` (matches `a.example.com`, `b.c.example.com`)
/// - IP addresses: `"192.168.1.100"`, `"[2001:db8::1]"` (IPv6)
///
/// # Examples
///
/// ```rust
/// use airssys_wasm::core::permission::{NetworkPermissions, NetworkEndpoint};
///
/// let perms = NetworkPermissions {
///     outbound: vec![
///         NetworkEndpoint {
///             host: "api.example.com".to_string(),
///             port: 443,
///         },
///         NetworkEndpoint {
///             host: "*.cdn.example.com".to_string(),
///             port: 443,
///         },
///     ],
///     inbound: vec![8080],
/// };
///
/// assert_eq!(perms.outbound.len(), 2);
/// ```
#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq)]
#[serde(deny_unknown_fields)]
pub struct NetworkPermissions {
    /// Outbound connection permissions (host + port)
    ///
    /// Components can make outbound connections only to specified endpoints.
    ///
    /// Examples:
    /// - `{ host = "api.example.com", port = 443 }`
    /// - `{ host = "*.cdn.example.com", port = 443 }`
    /// - `{ host = "192.168.1.100", port = 8080 }`
    #[serde(default)]
    pub outbound: Vec<NetworkEndpoint>,

    /// Inbound listening permissions (ports only)
    ///
    /// Components can listen on specified ports only.
    ///
    /// Examples:
    /// - `8080` - HTTP
    /// - `9000` - Custom service
    #[serde(default)]
    pub inbound: Vec<u16>,
}

/// Network endpoint specification (host + port).
///
/// # Examples
///
/// ```rust
/// use airssys_wasm::core::permission::NetworkEndpoint;
///
/// let endpoint = NetworkEndpoint {
///     host: "api.example.com".to_string(),
///     port: 443,
/// };
///
/// assert_eq!(endpoint.host, "api.example.com");
/// assert_eq!(endpoint.port, 443);
/// ```
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub struct NetworkEndpoint {
    /// Host domain or IP address
    ///
    /// Supports:
    /// - Exact domains: `"api.example.com"`
    /// - Wildcard subdomains: `"*.example.com"`
    /// - IPv4: `"192.168.1.100"`
    /// - IPv6: `"[2001:db8::1]"`
    pub host: String,

    /// Port number (1-65535)
    pub port: u16,
}

/// Storage permission declarations.
///
/// Uses namespace-based access control with quota enforcement.
///
/// # Namespace Format
///
/// - Component-isolated: `"myapp:config"`, `"myapp:cache"`
/// - Shared namespaces: `"shared:public-data"` (requires explicit approval)
///
/// # Examples
///
/// ```rust
/// use airssys_wasm::core::permission::StoragePermissions;
///
/// let perms = StoragePermissions {
///     namespaces: vec![
///         "myapp:config".to_string(),
///         "myapp:cache".to_string(),
///     ],
///     max_size_mb: 100,
/// };
///
/// assert_eq!(perms.namespaces.len(), 2);
/// assert_eq!(perms.max_size_mb, 100);
/// ```
#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq)]
#[serde(deny_unknown_fields)]
pub struct StoragePermissions {
    /// Storage namespace permissions
    ///
    /// Format: `"<prefix>:<name>"`
    /// - Component-isolated: `"myapp:*"`
    /// - Shared: `"shared:*"` (requires approval)
    ///
    /// Examples:
    /// - `"myapp:config"` - Component configuration storage
    /// - `"myapp:cache"` - Component cache storage
    /// - `"shared:public-data"` - Shared namespace (explicit approval)
    #[serde(default)]
    pub namespaces: Vec<String>,

    /// Total storage quota in megabytes
    ///
    /// Host enforces this limit across all namespaces.
    ///
    /// Default: 0 (no storage access)
    ///
    /// Example: `100` = 100 MB maximum storage
    #[serde(default)]
    pub max_size_mb: u64,
}

impl PermissionManifest {
    /// Create an empty permission manifest (deny-all).
    ///
    /// # Examples
    ///
    /// ```rust
    /// use airssys_wasm::core::permission::PermissionManifest;
    ///
    /// let manifest = PermissionManifest::new();
    /// assert!(manifest.filesystem.read.is_empty());
    /// assert!(manifest.network.outbound.is_empty());
    /// ```
    pub fn new() -> Self {
        Self::default()
    }

    /// Check if manifest has any permissions at all.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use airssys_wasm::core::permission::PermissionManifest;
    ///
    /// let empty = PermissionManifest::new();
    /// assert!(!empty.has_any_permissions());
    ///
    /// let mut with_perms = PermissionManifest::new();
    /// with_perms.filesystem.read.push("/data/**".to_string());
    /// assert!(with_perms.has_any_permissions());
    /// ```
    pub fn has_any_permissions(&self) -> bool {
        !self.filesystem.read.is_empty()
            || !self.filesystem.write.is_empty()
            || !self.filesystem.delete.is_empty()
            || !self.filesystem.list.is_empty()
            || !self.network.outbound.is_empty()
            || !self.network.inbound.is_empty()
            || !self.storage.namespaces.is_empty()
    }

    /// Get total count of all declared permissions.
    ///
    /// Useful for logging and metrics.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use airssys_wasm::core::permission::{PermissionManifest, NetworkEndpoint};
    ///
    /// let mut manifest = PermissionManifest::new();
    /// manifest.filesystem.read.push("/data/**".to_string());
    /// manifest.filesystem.write.push("/output/**".to_string());
    /// manifest.network.outbound.push(NetworkEndpoint {
    ///     host: "api.example.com".to_string(),
    ///     port: 443,
    /// });
    ///
    /// assert_eq!(manifest.total_permission_count(), 3);
    /// ```
    pub fn total_permission_count(&self) -> usize {
        self.filesystem.read.len()
            + self.filesystem.write.len()
            + self.filesystem.delete.len()
            + self.filesystem.list.len()
            + self.network.outbound.len()
            + self.network.inbound.len()
            + self.storage.namespaces.len()
    }
}

impl FilesystemPermissions {
    /// Create empty filesystem permissions (no access).
    ///
    /// # Examples
    ///
    /// ```rust
    /// use airssys_wasm::core::permission::FilesystemPermissions;
    ///
    /// let perms = FilesystemPermissions::new();
    /// assert!(perms.read.is_empty());
    /// ```
    pub fn new() -> Self {
        Self::default()
    }

    /// Get all unique patterns across all filesystem actions.
    ///
    /// Useful for validation and pattern compilation.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use airssys_wasm::core::permission::FilesystemPermissions;
    ///
    /// let mut perms = FilesystemPermissions::new();
    /// perms.read.push("/data/**".to_string());
    /// perms.write.push("/data/**".to_string()); // Duplicate pattern
    /// perms.write.push("/output/**".to_string());
    ///
    /// let patterns = perms.all_patterns();
    /// assert_eq!(patterns.len(), 2); // Deduplicated
    /// ```
    pub fn all_patterns(&self) -> HashSet<String> {
        let mut patterns = HashSet::new();
        patterns.extend(self.read.iter().cloned());
        patterns.extend(self.write.iter().cloned());
        patterns.extend(self.delete.iter().cloned());
        patterns.extend(self.list.iter().cloned());
        patterns
    }
}

impl NetworkPermissions {
    /// Create empty network permissions (no access).
    ///
    /// # Examples
    ///
    /// ```rust
    /// use airssys_wasm::core::permission::NetworkPermissions;
    ///
    /// let perms = NetworkPermissions::new();
    /// assert!(perms.outbound.is_empty());
    /// ```
    pub fn new() -> Self {
        Self::default()
    }

    /// Get all unique host patterns from outbound permissions.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use airssys_wasm::core::permission::{NetworkPermissions, NetworkEndpoint};
    ///
    /// let mut perms = NetworkPermissions::new();
    /// perms.outbound.push(NetworkEndpoint {
    ///     host: "api.example.com".to_string(),
    ///     port: 443,
    /// });
    /// perms.outbound.push(NetworkEndpoint {
    ///     host: "*.cdn.example.com".to_string(),
    ///     port: 443,
    /// });
    ///
    /// let hosts = perms.outbound_hosts();
    /// assert_eq!(hosts.len(), 2);
    /// ```
    pub fn outbound_hosts(&self) -> HashSet<String> {
        self.outbound.iter().map(|e| e.host.clone()).collect()
    }
}

impl StoragePermissions {
    /// Create empty storage permissions (no access).
    ///
    /// # Examples
    ///
    /// ```rust
    /// use airssys_wasm::core::permission::StoragePermissions;
    ///
    /// let perms = StoragePermissions::new();
    /// assert!(perms.namespaces.is_empty());
    /// assert_eq!(perms.max_size_mb, 0);
    /// ```
    pub fn new() -> Self {
        Self::default()
    }

    /// Check if a namespace is allowed.
    ///
    /// Simple string equality check (pattern matching done by checker).
    ///
    /// # Examples
    ///
    /// ```rust
    /// use airssys_wasm::core::permission::StoragePermissions;
    ///
    /// let mut perms = StoragePermissions::new();
    /// perms.namespaces.push("myapp:cache".to_string());
    ///
    /// assert!(perms.has_namespace("myapp:cache"));
    /// assert!(!perms.has_namespace("other:cache"));
    /// ```
    pub fn has_namespace(&self, namespace: &str) -> bool {
        self.namespaces.iter().any(|ns| ns == namespace)
    }
}

#[cfg(test)]
#[allow(clippy::unwrap_used)] // Test code is allowed to use unwrap for clarity
mod tests {
    use super::*;

    #[test]
    fn test_permission_manifest_parse() {
        let toml_content = r#"
[filesystem]
read = ["/data/**", "/config/*.json"]
write = ["/output/**"]

[network]
outbound = [
    { host = "api.example.com", port = 443 }
]

[storage]
namespaces = ["myapp:cache"]
max_size_mb = 100
"#;

        let parsed: Result<PermissionManifest, _> = toml::from_str(toml_content);
        assert!(parsed.is_ok(), "Failed to parse: {:?}", parsed);

        let manifest = parsed.unwrap();
        assert_eq!(manifest.filesystem.read.len(), 2);
        assert_eq!(manifest.filesystem.write.len(), 1);
        assert_eq!(manifest.network.outbound.len(), 1);
        assert_eq!(manifest.storage.max_size_mb, 100);
    }

    #[test]
    fn test_empty_manifest() {
        let manifest = PermissionManifest::new();
        assert!(!manifest.has_any_permissions());
        assert_eq!(manifest.total_permission_count(), 0);
    }

    #[test]
    fn test_manifest_with_permissions() {
        let mut manifest = PermissionManifest::new();
        manifest.filesystem.read.push("/data/**".to_string());
        manifest.filesystem.write.push("/output/**".to_string());

        assert!(manifest.has_any_permissions());
        assert_eq!(manifest.total_permission_count(), 2);
    }

    #[test]
    fn test_filesystem_all_patterns() {
        let mut perms = FilesystemPermissions::new();
        perms.read.push("/data/**".to_string());
        perms.write.push("/data/**".to_string()); // Duplicate
        perms.write.push("/output/**".to_string());

        let patterns = perms.all_patterns();
        assert_eq!(patterns.len(), 2); // Deduplicated
    }

    #[test]
    fn test_network_outbound_hosts() {
        let mut perms = NetworkPermissions::new();
        perms.outbound.push(NetworkEndpoint {
            host: "api.example.com".to_string(),
            port: 443,
        });
        perms.outbound.push(NetworkEndpoint {
            host: "*.cdn.example.com".to_string(),
            port: 443,
        });

        let hosts = perms.outbound_hosts();
        assert_eq!(hosts.len(), 2);
        assert!(hosts.contains("api.example.com"));
        assert!(hosts.contains("*.cdn.example.com"));
    }

    #[test]
    fn test_storage_has_namespace() {
        let mut perms = StoragePermissions::new();
        perms.namespaces.push("myapp:cache".to_string());
        perms.namespaces.push("myapp:config".to_string());

        assert!(perms.has_namespace("myapp:cache"));
        assert!(perms.has_namespace("myapp:config"));
        assert!(!perms.has_namespace("other:cache"));
    }
}
