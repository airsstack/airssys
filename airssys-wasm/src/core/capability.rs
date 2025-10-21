//! Capability-based security primitives for fine-grained component permissions.
//!
//! This module defines the capability types used throughout airssys-wasm for
//! enforcing fine-grained security policies. Components declare required capabilities
//! in their metadata, and the runtime enforces these permissions at execution time.
//!
//! # Capability Model
//!
//! The capability model follows ADR-WASM-005 (Capability-Based Security Model):
//! - **Least Privilege**: Components get only capabilities they explicitly request
//! - **Pattern-Based**: Capabilities support patterns (globs, wildcards, namespaces)
//! - **Composable**: Multiple capabilities can be combined via CapabilitySet
//! - **Extensible**: Custom capabilities for domain-specific permissions
//!
//! # Examples
//!
//! ```
//! use airssys_wasm::core::capability::{Capability, PathPattern, CapabilitySet};
//!
//! // Create filesystem capabilities
//! let read_cap = Capability::FileRead(PathPattern::new("/data/*.json"));
//! let write_cap = Capability::FileWrite(PathPattern::new("/output/*"));
//!
//! // Create a capability set
//! let mut caps = CapabilitySet::new();
//! caps.grant(read_cap);
//! caps.grant(write_cap);
//!
//! // Check capabilities
//! let check = Capability::FileRead(PathPattern::new("/data/*.json"));
//! assert!(caps.has(&check));
//! ```
//!
//! # Security Considerations
//!
//! - Pattern matching implementation is in the `security/` module (Phase 7)
//! - Wildcards and globs require careful validation to prevent privilege escalation
//! - Custom capabilities should be used sparingly and documented thoroughly

// Standard library imports
use std::collections::HashSet;
use std::path::Path;

// External crate imports
use serde::{Deserialize, Serialize};
use serde_json;

/// Fine-grained capability for component permissions.
///
/// Capabilities define what operations a component is allowed to perform.
/// Each capability type uses pattern matching to allow flexible but secure
/// permission specifications.
///
/// # Capability Types
///
/// - **FileRead/FileWrite**: Filesystem access with glob patterns
/// - **NetworkOutbound**: Outbound connections to specific domains
/// - **NetworkInbound**: Listening on specific ports
/// - **Storage**: Access to storage namespaces
/// - **ProcessSpawn**: Ability to spawn external processes
/// - **Messaging**: Inter-component communication on topics
/// - **Custom**: Extensible for domain-specific permissions
///
/// # Examples
///
/// ```
/// use airssys_wasm::core::capability::{Capability, PathPattern, DomainPattern};
///
/// // Filesystem capabilities
/// let read = Capability::FileRead(PathPattern::new("/data/*.json"));
/// let write = Capability::FileWrite(PathPattern::new("/output/results.txt"));
///
/// // Network capabilities
/// let http = Capability::NetworkOutbound(DomainPattern::new("*.example.com"));
/// let listen = Capability::NetworkInbound(8080);
///
/// // Process capability
/// let spawn = Capability::ProcessSpawn;
/// ```
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum Capability {
    /// Filesystem read access with glob pattern support.
    ///
    /// Examples:
    /// - `"/data/config.json"` - Specific file
    /// - `"/data/*.json"` - All JSON files in directory
    /// - `"/data/**/*.json"` - All JSON files recursively
    FileRead(PathPattern),

    /// Filesystem write access with glob pattern support.
    ///
    /// Examples:
    /// - `"/output/results.txt"` - Specific file
    /// - `"/output/*.log"` - All log files in directory
    /// - `"/tmp/**/*"` - All files in tmp recursively
    FileWrite(PathPattern),

    /// Network outbound connection with domain pattern support.
    ///
    /// Examples:
    /// - `"api.example.com"` - Specific domain
    /// - `"*.example.com"` - All subdomains
    /// - `"*"` - Any domain (use with caution!)
    NetworkOutbound(DomainPattern),

    /// Network inbound listener on specific port.
    ///
    /// Example: `NetworkInbound(8080)` allows listening on port 8080
    NetworkInbound(u16),

    /// Storage access with namespace pattern support.
    ///
    /// Examples:
    /// - `"user.settings"` - Specific namespace
    /// - `"cache.*"` - All cache namespaces
    Storage(NamespacePattern),

    /// Process spawn capability (no parameters).
    ///
    /// Allows spawning external processes. This is a high-privilege
    /// capability and should be granted with caution.
    ProcessSpawn,

    /// Inter-component messaging with topic pattern support.
    ///
    /// Examples:
    /// - `"events.user"` - Specific topic
    /// - `"events.*"` - All event topics
    /// - `"**"` - All topics (use with caution!)
    Messaging(TopicPattern),

    /// Custom capability for domain-specific permissions.
    ///
    /// # Example
    ///
    /// ```
    /// use airssys_wasm::core::capability::Capability;
    /// use serde_json::json;
    ///
    /// let gpu_cap = Capability::Custom {
    ///     name: "gpu.compute".to_string(),
    ///     parameters: json!({
    ///         "device_id": 0,
    ///         "memory_limit": "2GB"
    ///     }),
    /// };
    /// ```
    Custom {
        /// Capability name (typically namespaced, e.g., "gpu.compute")
        name: String,
        /// Arbitrary parameters for the capability
        parameters: serde_json::Value,
    },
}

/// Path pattern for filesystem capabilities supporting glob-style wildcards.
///
/// This newtype wrapper provides type safety and will support glob matching
/// when the security module is implemented (Phase 7).
///
/// # Pattern Syntax
///
/// - `*` - Matches any characters except path separator
/// - `**` - Matches any characters including path separators (recursive)
/// - `?` - Matches exactly one character
/// - `[abc]` - Matches one of the characters in brackets
/// - `{a,b,c}` - Matches one of the alternatives
///
/// # Examples
///
/// ```
/// use airssys_wasm::core::capability::PathPattern;
///
/// let pattern = PathPattern::new("/data/*.json");
/// let pattern2 = PathPattern::new("/logs/**/error.log");
/// ```
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct PathPattern(String);

impl PathPattern {
    /// Create a new path pattern.
    ///
    /// # Examples
    ///
    /// ```
    /// use airssys_wasm::core::capability::PathPattern;
    ///
    /// let pattern = PathPattern::new("/data/*.json");
    /// let pattern2 = PathPattern::new(String::from("/logs/*.log"));
    /// ```
    pub fn new(pattern: impl Into<String>) -> Self {
        Self(pattern.into())
    }

    /// Get the pattern as a string slice.
    ///
    /// # Examples
    ///
    /// ```
    /// use airssys_wasm::core::capability::PathPattern;
    ///
    /// let pattern = PathPattern::new("/data/*.json");
    /// assert_eq!(pattern.as_str(), "/data/*.json");
    /// ```
    pub fn as_str(&self) -> &str {
        &self.0
    }

    /// Check if a path matches this pattern.
    ///
    /// # Note
    ///
    /// This is a placeholder that will be implemented in the security module (Phase 7).
    /// For now, it uses simple string equality.
    ///
    /// # Examples
    ///
    /// ```
    /// use airssys_wasm::core::capability::PathPattern;
    /// use std::path::Path;
    ///
    /// let pattern = PathPattern::new("/data/config.json");
    /// let path = Path::new("/data/config.json");
    ///
    /// // Note: Full glob matching will be implemented in Phase 7
    /// // For now, this does simple string comparison
    /// ```
    pub fn matches(&self, path: &Path) -> bool {
        // TODO(Phase 7): Implement proper glob matching in security/ module
        // For now, just compare strings directly
        path.to_str().map(|p| p == self.0).unwrap_or(false)
    }
}

/// Domain pattern for network capabilities supporting wildcard matching.
///
/// This newtype wrapper provides type safety and will support wildcard matching
/// when the security module is implemented (Phase 7).
///
/// # Pattern Syntax
///
/// - `*` - Matches any subdomain level
/// - `*.example.com` - Matches all subdomains of example.com
/// - `api.*.example.com` - Matches specific subdomain patterns
///
/// # Examples
///
/// ```
/// use airssys_wasm::core::capability::DomainPattern;
///
/// let pattern = DomainPattern::new("*.example.com");
/// let pattern2 = DomainPattern::new("api.production.example.com");
/// ```
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct DomainPattern(String);

impl DomainPattern {
    /// Create a new domain pattern.
    ///
    /// # Examples
    ///
    /// ```
    /// use airssys_wasm::core::capability::DomainPattern;
    ///
    /// let pattern = DomainPattern::new("*.example.com");
    /// let pattern2 = DomainPattern::new(String::from("api.example.com"));
    /// ```
    pub fn new(pattern: impl Into<String>) -> Self {
        Self(pattern.into())
    }

    /// Get the pattern as a string slice.
    ///
    /// # Examples
    ///
    /// ```
    /// use airssys_wasm::core::capability::DomainPattern;
    ///
    /// let pattern = DomainPattern::new("*.example.com");
    /// assert_eq!(pattern.as_str(), "*.example.com");
    /// ```
    pub fn as_str(&self) -> &str {
        &self.0
    }

    /// Check if a domain matches this pattern.
    ///
    /// # Note
    ///
    /// This is a placeholder that will be implemented in the security module (Phase 7).
    /// For now, it uses simple string equality.
    ///
    /// # Examples
    ///
    /// ```
    /// use airssys_wasm::core::capability::DomainPattern;
    ///
    /// let pattern = DomainPattern::new("api.example.com");
    ///
    /// // Note: Full wildcard matching will be implemented in Phase 7
    /// // For now, this does simple string comparison
    /// ```
    pub fn matches(&self, domain: &str) -> bool {
        // TODO(Phase 7): Implement proper wildcard matching in security/ module
        // For now, just compare strings directly
        self.0 == domain
    }
}

/// Namespace pattern for storage capabilities.
///
/// This newtype wrapper provides type safety and will support namespace matching
/// when the security module is implemented (Phase 7).
///
/// # Pattern Syntax
///
/// - `*` - Matches any single namespace component
/// - `cache.*` - Matches all cache namespaces
/// - `user.*.settings` - Matches specific namespace patterns
///
/// # Examples
///
/// ```
/// use airssys_wasm::core::capability::NamespacePattern;
///
/// let pattern = NamespacePattern::new("cache.*");
/// let pattern2 = NamespacePattern::new("user.settings");
/// ```
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct NamespacePattern(String);

impl NamespacePattern {
    /// Create a new namespace pattern.
    ///
    /// # Examples
    ///
    /// ```
    /// use airssys_wasm::core::capability::NamespacePattern;
    ///
    /// let pattern = NamespacePattern::new("cache.*");
    /// let pattern2 = NamespacePattern::new(String::from("user.settings"));
    /// ```
    pub fn new(pattern: impl Into<String>) -> Self {
        Self(pattern.into())
    }

    /// Get the pattern as a string slice.
    ///
    /// # Examples
    ///
    /// ```
    /// use airssys_wasm::core::capability::NamespacePattern;
    ///
    /// let pattern = NamespacePattern::new("cache.*");
    /// assert_eq!(pattern.as_str(), "cache.*");
    /// ```
    pub fn as_str(&self) -> &str {
        &self.0
    }

    /// Check if a namespace matches this pattern.
    ///
    /// # Note
    ///
    /// This is a placeholder that will be implemented in the security module (Phase 7).
    /// For now, it uses simple string equality.
    ///
    /// # Examples
    ///
    /// ```
    /// use airssys_wasm::core::capability::NamespacePattern;
    ///
    /// let pattern = NamespacePattern::new("user.settings");
    ///
    /// // Note: Full pattern matching will be implemented in Phase 7
    /// // For now, this does simple string comparison
    /// ```
    pub fn matches(&self, namespace: &str) -> bool {
        // TODO(Phase 7): Implement proper namespace matching in security/ module
        // For now, just compare strings directly
        self.0 == namespace
    }
}

/// Topic pattern for messaging capabilities.
///
/// This newtype wrapper provides type safety and will support topic matching
/// when the security module is implemented (Phase 7).
///
/// # Pattern Syntax
///
/// - `*` - Matches any single topic component
/// - `events.*` - Matches all event topics
/// - `**` - Matches all topics (hierarchical wildcard)
///
/// # Examples
///
/// ```
/// use airssys_wasm::core::capability::TopicPattern;
///
/// let pattern = TopicPattern::new("events.*");
/// let pattern2 = TopicPattern::new("user.activity");
/// ```
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct TopicPattern(String);

impl TopicPattern {
    /// Create a new topic pattern.
    ///
    /// # Examples
    ///
    /// ```
    /// use airssys_wasm::core::capability::TopicPattern;
    ///
    /// let pattern = TopicPattern::new("events.*");
    /// let pattern2 = TopicPattern::new(String::from("user.activity"));
    /// ```
    pub fn new(pattern: impl Into<String>) -> Self {
        Self(pattern.into())
    }

    /// Get the pattern as a string slice.
    ///
    /// # Examples
    ///
    /// ```
    /// use airssys_wasm::core::capability::TopicPattern;
    ///
    /// let pattern = TopicPattern::new("events.*");
    /// assert_eq!(pattern.as_str(), "events.*");
    /// ```
    pub fn as_str(&self) -> &str {
        &self.0
    }

    /// Check if a topic matches this pattern.
    ///
    /// # Note
    ///
    /// This is a placeholder that will be implemented in the security module (Phase 7).
    /// For now, it uses simple string equality.
    ///
    /// # Examples
    ///
    /// ```
    /// use airssys_wasm::core::capability::TopicPattern;
    ///
    /// let pattern = TopicPattern::new("user.activity");
    ///
    /// // Note: Full pattern matching will be implemented in Phase 7
    /// // For now, this does simple string comparison
    /// ```
    pub fn matches(&self, topic: &str) -> bool {
        // TODO(Phase 7): Implement proper topic matching in security/ module
        // For now, just compare strings directly
        self.0 == topic
    }
}

/// Set of capabilities granted to a component.
///
/// CapabilitySet provides an ergonomic API for managing component permissions.
/// It supports adding (granting), removing (revoking), and checking capabilities.
///
/// # Examples
///
/// ```
/// use airssys_wasm::core::capability::{Capability, CapabilitySet, PathPattern};
///
/// let mut caps = CapabilitySet::new();
///
/// // Grant capabilities
/// caps.grant(Capability::FileRead(PathPattern::new("/data/*.json")));
/// caps.grant(Capability::FileWrite(PathPattern::new("/output/*")));
/// caps.grant(Capability::NetworkOutbound(airssys_wasm::core::capability::DomainPattern::new("api.example.com")));
///
/// // Check capabilities
/// let read_cap = Capability::FileRead(PathPattern::new("/data/*.json"));
/// assert!(caps.has(&read_cap));
///
/// // Revoke capabilities
/// caps.revoke(&read_cap);
/// assert!(!caps.has(&read_cap));
/// ```
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct CapabilitySet {
    capabilities: HashSet<Capability>,
}

impl CapabilitySet {
    /// Create an empty capability set.
    ///
    /// # Examples
    ///
    /// ```
    /// use airssys_wasm::core::capability::CapabilitySet;
    ///
    /// let caps = CapabilitySet::new();
    /// assert_eq!(caps.len(), 0);
    /// ```
    pub fn new() -> Self {
        Self::default()
    }

    /// Create a capability set from a vector of capabilities.
    ///
    /// # Examples
    ///
    /// ```
    /// use airssys_wasm::core::capability::{Capability, CapabilitySet, PathPattern};
    ///
    /// let caps = CapabilitySet::from_vec(vec![
    ///     Capability::FileRead(PathPattern::new("/data/*")),
    ///     Capability::ProcessSpawn,
    /// ]);
    ///
    /// assert_eq!(caps.len(), 2);
    /// ```
    pub fn from_vec(capabilities: Vec<Capability>) -> Self {
        Self {
            capabilities: capabilities.into_iter().collect(),
        }
    }

    /// Add a capability to the set (grant permission).
    ///
    /// If the capability already exists, this has no effect.
    ///
    /// # Examples
    ///
    /// ```
    /// use airssys_wasm::core::capability::{Capability, CapabilitySet};
    ///
    /// let mut caps = CapabilitySet::new();
    /// caps.grant(Capability::ProcessSpawn);
    /// assert_eq!(caps.len(), 1);
    /// ```
    pub fn grant(&mut self, capability: Capability) {
        self.capabilities.insert(capability);
    }

    /// Remove a capability from the set (revoke permission).
    ///
    /// If the capability doesn't exist, this has no effect.
    ///
    /// # Examples
    ///
    /// ```
    /// use airssys_wasm::core::capability::{Capability, CapabilitySet};
    ///
    /// let mut caps = CapabilitySet::new();
    /// let spawn_cap = Capability::ProcessSpawn;
    ///
    /// caps.grant(spawn_cap.clone());
    /// assert!(caps.has(&spawn_cap));
    ///
    /// caps.revoke(&spawn_cap);
    /// assert!(!caps.has(&spawn_cap));
    /// ```
    pub fn revoke(&mut self, capability: &Capability) {
        self.capabilities.remove(capability);
    }

    /// Check if a capability is exactly present in the set.
    ///
    /// # Examples
    ///
    /// ```
    /// use airssys_wasm::core::capability::{Capability, CapabilitySet, PathPattern};
    ///
    /// let mut caps = CapabilitySet::new();
    /// let read_cap = Capability::FileRead(PathPattern::new("/data/*.json"));
    ///
    /// caps.grant(read_cap.clone());
    /// assert!(caps.has(&read_cap));
    /// ```
    pub fn has(&self, capability: &Capability) -> bool {
        self.capabilities.contains(capability)
    }

    /// Check if any capability in the set matches the given capability.
    ///
    /// Uses pattern matching to check if any granted capability would allow
    /// the requested capability. For example, `FileRead("/data/*")` would match
    /// a request for `FileRead("/data/file.txt")`.
    ///
    /// # Note
    ///
    /// This is a placeholder that will be implemented in the security module (Phase 7).
    /// For now, it falls back to exact matching via `has()`.
    ///
    /// # Examples
    ///
    /// ```
    /// use airssys_wasm::core::capability::{Capability, CapabilitySet, PathPattern};
    ///
    /// let mut caps = CapabilitySet::new();
    /// caps.grant(Capability::FileRead(PathPattern::new("/data/*.json")));
    ///
    /// // Note: Full pattern matching will be implemented in Phase 7
    /// // For now, this requires exact match
    /// let check = Capability::FileRead(PathPattern::new("/data/*.json"));
    /// assert!(caps.matches(&check));
    /// ```
    pub fn matches(&self, capability: &Capability) -> bool {
        // TODO(Phase 7): Implement proper pattern matching in security/ module
        // For now, fall back to exact matching
        self.has(capability)
    }

    /// Iterate over all capabilities in the set.
    ///
    /// # Examples
    ///
    /// ```
    /// use airssys_wasm::core::capability::{Capability, CapabilitySet};
    ///
    /// let mut caps = CapabilitySet::new();
    /// caps.grant(Capability::ProcessSpawn);
    ///
    /// for cap in caps.iter() {
    ///     println!("Capability: {:?}", cap);
    /// }
    /// ```
    pub fn iter(&self) -> impl Iterator<Item = &Capability> {
        self.capabilities.iter()
    }

    /// Get the number of capabilities in the set.
    ///
    /// # Examples
    ///
    /// ```
    /// use airssys_wasm::core::capability::{Capability, CapabilitySet};
    ///
    /// let mut caps = CapabilitySet::new();
    /// assert_eq!(caps.len(), 0);
    ///
    /// caps.grant(Capability::ProcessSpawn);
    /// assert_eq!(caps.len(), 1);
    /// ```
    pub fn len(&self) -> usize {
        self.capabilities.len()
    }

    /// Check if the capability set is empty.
    ///
    /// # Examples
    ///
    /// ```
    /// use airssys_wasm::core::capability::CapabilitySet;
    ///
    /// let caps = CapabilitySet::new();
    /// assert!(caps.is_empty());
    /// ```
    pub fn is_empty(&self) -> bool {
        self.capabilities.is_empty()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_path_pattern_creation() {
        let pattern = PathPattern::new("/data/*.json");
        assert_eq!(pattern.as_str(), "/data/*.json");

        let pattern2 = PathPattern::new(String::from("/logs/*.log"));
        assert_eq!(pattern2.as_str(), "/logs/*.log");
    }

    #[test]
    fn test_domain_pattern_creation() {
        let pattern = DomainPattern::new("*.example.com");
        assert_eq!(pattern.as_str(), "*.example.com");

        let pattern2 = DomainPattern::new(String::from("api.example.com"));
        assert_eq!(pattern2.as_str(), "api.example.com");
    }

    #[test]
    fn test_namespace_pattern_creation() {
        let pattern = NamespacePattern::new("cache.*");
        assert_eq!(pattern.as_str(), "cache.*");

        let pattern2 = NamespacePattern::new(String::from("user.settings"));
        assert_eq!(pattern2.as_str(), "user.settings");
    }

    #[test]
    fn test_topic_pattern_creation() {
        let pattern = TopicPattern::new("events.*");
        assert_eq!(pattern.as_str(), "events.*");

        let pattern2 = TopicPattern::new(String::from("user.activity"));
        assert_eq!(pattern2.as_str(), "user.activity");
    }

    #[test]
    fn test_capability_variants() {
        let file_read = Capability::FileRead(PathPattern::new("/data/*"));
        let file_write = Capability::FileWrite(PathPattern::new("/output/*"));
        let network_out = Capability::NetworkOutbound(DomainPattern::new("*.example.com"));
        let network_in = Capability::NetworkInbound(8080);
        let storage = Capability::Storage(NamespacePattern::new("cache.*"));
        let process = Capability::ProcessSpawn;
        let messaging = Capability::Messaging(TopicPattern::new("events.*"));
        let custom = Capability::Custom {
            name: "gpu.compute".to_string(),
            parameters: serde_json::json!({"device": 0}),
        };

        // Just ensure they compile and are constructed correctly
        assert!(matches!(file_read, Capability::FileRead(_)));
        assert!(matches!(file_write, Capability::FileWrite(_)));
        assert!(matches!(network_out, Capability::NetworkOutbound(_)));
        assert!(matches!(network_in, Capability::NetworkInbound(8080)));
        assert!(matches!(storage, Capability::Storage(_)));
        assert!(matches!(process, Capability::ProcessSpawn));
        assert!(matches!(messaging, Capability::Messaging(_)));
        assert!(matches!(custom, Capability::Custom { .. }));
    }

    #[test]
    fn test_capability_equality() {
        let cap1 = Capability::FileRead(PathPattern::new("/data/*"));
        let cap2 = Capability::FileRead(PathPattern::new("/data/*"));
        let cap3 = Capability::FileRead(PathPattern::new("/logs/*"));

        assert_eq!(cap1, cap2);
        assert_ne!(cap1, cap3);
    }

    #[test]
    fn test_capability_hashing() {
        use std::collections::HashSet;

        let mut set = HashSet::new();
        set.insert(Capability::ProcessSpawn);
        set.insert(Capability::ProcessSpawn); // Duplicate

        assert_eq!(set.len(), 1); // HashSet deduplicates
    }

    #[test]
    fn test_capability_set_new() {
        let caps = CapabilitySet::new();
        assert!(caps.is_empty());
        assert_eq!(caps.len(), 0);
    }

    #[test]
    fn test_capability_set_from_vec() {
        let caps = CapabilitySet::from_vec(vec![
            Capability::ProcessSpawn,
            Capability::FileRead(PathPattern::new("/data/*")),
        ]);

        assert_eq!(caps.len(), 2);
        assert!(!caps.is_empty());
    }

    #[test]
    fn test_capability_set_grant() {
        let mut caps = CapabilitySet::new();

        caps.grant(Capability::ProcessSpawn);
        assert_eq!(caps.len(), 1);

        // Granting same capability again doesn't increase count
        caps.grant(Capability::ProcessSpawn);
        assert_eq!(caps.len(), 1);
    }

    #[test]
    fn test_capability_set_revoke() {
        let mut caps = CapabilitySet::new();
        let spawn_cap = Capability::ProcessSpawn;

        caps.grant(spawn_cap.clone());
        assert!(caps.has(&spawn_cap));

        caps.revoke(&spawn_cap);
        assert!(!caps.has(&spawn_cap));
    }

    #[test]
    fn test_capability_set_has() {
        let mut caps = CapabilitySet::new();
        let read_cap = Capability::FileRead(PathPattern::new("/data/*.json"));

        assert!(!caps.has(&read_cap));

        caps.grant(read_cap.clone());
        assert!(caps.has(&read_cap));
    }

    #[test]
    fn test_capability_set_matches() {
        let mut caps = CapabilitySet::new();
        let read_cap = Capability::FileRead(PathPattern::new("/data/*.json"));

        caps.grant(read_cap.clone());

        // For now, matches() falls back to exact matching
        assert!(caps.matches(&read_cap));

        let other_cap = Capability::FileRead(PathPattern::new("/logs/*.log"));
        assert!(!caps.matches(&other_cap));
    }

    #[test]
    fn test_capability_set_iter() {
        let mut caps = CapabilitySet::new();
        caps.grant(Capability::ProcessSpawn);
        caps.grant(Capability::FileRead(PathPattern::new("/data/*")));

        let count = caps.iter().count();
        assert_eq!(count, 2);
    }

    #[test]
    fn test_capability_serialization() -> Result<(), Box<dyn std::error::Error>> {
        let cap = Capability::FileRead(PathPattern::new("/data/*.json"));
        let json = serde_json::to_string(&cap)?;
        let deserialized: Capability = serde_json::from_str(&json)?;

        assert_eq!(cap, deserialized);
        Ok(())
    }

    #[test]
    fn test_capability_set_serialization() -> Result<(), Box<dyn std::error::Error>> {
        let mut caps = CapabilitySet::new();
        caps.grant(Capability::ProcessSpawn);
        caps.grant(Capability::FileRead(PathPattern::new("/data/*")));

        let json = serde_json::to_string(&caps)?;
        let deserialized: CapabilitySet = serde_json::from_str(&json)?;

        assert_eq!(caps.len(), deserialized.len());
        Ok(())
    }
}
