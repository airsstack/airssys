//! Access Control Lists (ACL) implementation.
//!
//! This module provides ACL-based security policy implementation with glob pattern matching
//! support for flexible and powerful access control.
//!
//! # Features
//!
//! - **Glob Pattern Matching**: Use wildcard patterns (`*`, `?`, `[...]`) for resources and permissions
//! - **String-Based Permissions**: Flexible permission model with glob support (e.g., `"read"`, `"file:*"`)
//! - **Context Attributes**: Extract resource and permission from [`SecurityContext`] attributes
//! - **First-Match Policy**: First matching entry determines the access decision
//! - **Explicit Deny**: Support for explicit deny policies
//! - **Default Deny**: Deny by default when no matching entries found
//!
//! # Context Attribute Keys
//!
//! ACL evaluation uses these standardized context attribute keys:
//! - [`ATTR_ACL_RESOURCE`]: The resource being accessed (e.g., file path, API endpoint)
//! - [`ATTR_ACL_PERMISSION`]: The permission being requested (e.g., "read", "write", "execute")
//!
//! # Examples
//!
//! ## Basic ACL with Glob Patterns
//!
//! ```rust
//! use airssys_osl::middleware::security::acl::{
//!     AccessControlList, AclEntry, AclPolicy, ATTR_ACL_RESOURCE, ATTR_ACL_PERMISSION
//! };
//! use airssys_osl::middleware::security::policy::SecurityPolicy;
//! use airssys_osl::core::context::SecurityContext;
//! use std::collections::HashMap;
//! use uuid::Uuid;
//! use chrono::Utc;
//!
//! // Create ACL entry allowing alice to read files in /home/alice/
//! let entry = AclEntry::new(
//!     "alice".to_string(),
//!     "/home/alice/*".to_string(),  // Glob pattern for resources
//!     vec!["read".to_string()],      // Allowed permissions
//!     AclPolicy::Allow,
//! );
//!
//! let acl = AccessControlList::new().add_entry(entry);
//!
//! // Create security context with resource and permission attributes
//! let mut attributes = HashMap::new();
//! attributes.insert(ATTR_ACL_RESOURCE.to_string(), "/home/alice/file.txt".to_string());
//! attributes.insert(ATTR_ACL_PERMISSION.to_string(), "read".to_string());
//!
//! let context = SecurityContext {
//!     principal: "alice".to_string(),
//!     session_id: Uuid::new_v4(),
//!     established_at: Utc::now(),
//!     attributes,
//! };
//!
//! // Evaluate - should allow access
//! let decision = acl.evaluate(&context);
//! ```
//!
//! ## Wildcard Permissions
//!
//! ```rust
//! use airssys_osl::middleware::security::acl::{AclEntry, AclPolicy};
//!
//! // Allow all permissions on admin resources
//! let admin_entry = AclEntry::new(
//!     "admin".to_string(),
//!     "/admin/*".to_string(),
//!     vec!["*".to_string()],  // Wildcard: all permissions
//!     AclPolicy::Allow,
//! );
//!
//! // Allow specific namespace permissions using glob patterns
//! let file_entry = AclEntry::new(
//!     "user".to_string(),
//!     "/data/*.txt".to_string(),
//!     vec!["file:*".to_string()],  // All file:* permissions (file:read, file:write, etc.)
//!     AclPolicy::Allow,
//! );
//! ```
//!
//! ## Explicit Deny Policy
//!
//! ```rust
//! use airssys_osl::middleware::security::acl::{AclEntry, AclPolicy};
//!
//! // Explicitly deny access to sensitive resources
//! let deny_entry = AclEntry::new(
//!     "guest".to_string(),
//!     "/etc/shadow".to_string(),
//!     vec!["*".to_string()],
//!     AclPolicy::Deny,  // Explicit deny for all permissions
//! );
//! ```

// Layer 1: Standard library imports
use std::fmt::Debug;

// Layer 2: Third-party crate imports
use glob::Pattern;
use serde::{Deserialize, Serialize};

// Layer 3: Internal module imports
use crate::core::context::SecurityContext;
use crate::middleware::security::policy::{PolicyDecision, PolicyScope, SecurityPolicy};

/// Context attribute key for resource path/identifier.
///
/// Uses `acl.` prefix to prevent conflicts with other security modules.
/// Used to extract the resource being accessed from `SecurityContext.attributes`.
///
/// # Example
/// ```
/// use std::collections::HashMap;
/// use airssys_osl::middleware::security::acl::ATTR_ACL_RESOURCE;
///
/// let mut attributes = HashMap::new();
/// attributes.insert(ATTR_ACL_RESOURCE.to_string(), "/etc/passwd".to_string());
/// ```
pub const ATTR_ACL_RESOURCE: &str = "acl.resource";

/// Context attribute key for required permission/action.
///
/// Uses `acl.` prefix to prevent conflicts with other security modules.
/// Used to extract the required permission from `SecurityContext.attributes`.
///
/// # Example
/// ```
/// use std::collections::HashMap;
/// use airssys_osl::middleware::security::acl::ATTR_ACL_PERMISSION;
///
/// let mut attributes = HashMap::new();
/// attributes.insert(ATTR_ACL_PERMISSION.to_string(), "read".to_string());
/// ```
pub const ATTR_ACL_PERMISSION: &str = "acl.permission";

/// ACL policy action for entries.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum AclPolicy {
    /// Allow the operation
    Allow,

    /// Deny the operation
    Deny,
}

/// Access Control List entry.
///
/// Represents a single ACL rule that maps an identity to a resource
/// pattern with specific permissions and policy action.
///
/// # Breaking Changes (Phase 3)
///
/// **Version 0.1.0**: Updated to use string-based permissions with glob pattern support.
/// - Added `permissions: Vec<String>` field (replaces enum-based permissions)
/// - Constructor `new()` now requires `permissions` parameter
/// - Permissions support glob patterns (e.g., `"read*"`, `"file:*"`, `"*"`)
///
/// # Permission Semantics
///
/// - **Empty permissions** (`permissions = []`): No permissions granted, entry won't match
/// - **Wildcard permissions** (`permissions = ["*"]`): All permissions granted
/// - **Specific permissions** (`permissions = ["read", "write"]`): Only listed permissions
/// - **Glob patterns** (`permissions = ["read*"]`): Pattern matching (e.g., matches "read", "read_metadata")
///
/// # Examples
///
/// ```
/// use airssys_osl::middleware::security::acl::{AclEntry, AclPolicy};
///
/// // Allow read and write permissions
/// let entry = AclEntry::new(
///     "alice".to_string(),
///     "/home/alice/*".to_string(),
///     vec!["read".to_string(), "write".to_string()],
///     AclPolicy::Allow
/// );
///
/// // Allow all permissions with wildcard
/// let admin_entry = AclEntry::new(
///     "admin".to_string(),
///     "*".to_string(),
///     vec!["*".to_string()],
///     AclPolicy::Allow
/// );
///
/// // Use glob patterns for namespaced permissions
/// let api_entry = AclEntry::new(
///     "service".to_string(),
///     "/api/*".to_string(),
///     vec!["api:read*".to_string()],  // Matches api:read, api:read_user, etc.
///     AclPolicy::Allow
/// );
/// ```
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AclEntry {
    /// Identity this entry applies to (user, group, or service)
    pub identity: String,

    /// Resource pattern this entry applies to (supports glob patterns)
    pub resource_pattern: String,

    /// Permissions granted/denied by this entry (supports glob patterns)
    pub permissions: Vec<String>,

    /// Policy action (allow or deny)
    pub policy: AclPolicy,
}

impl AclEntry {
    /// Create a new ACL entry.
    ///
    /// # Arguments
    ///
    /// * `identity` - User, group, or service identity (supports "*" wildcard)
    /// * `resource_pattern` - Resource pattern (supports glob patterns like "/path/*")
    /// * `permissions` - List of permissions (supports glob patterns like "read*", or "*" for all)
    /// * `policy` - Allow or Deny action
    ///
    /// # Examples
    ///
    /// ```
    /// use airssys_osl::middleware::security::acl::{AclEntry, AclPolicy};
    ///
    /// // Specific permissions
    /// let entry = AclEntry::new(
    ///     "alice".to_string(),
    ///     "/data/*".to_string(),
    ///     vec!["read".to_string(), "write".to_string()],
    ///     AclPolicy::Allow
    /// );
    ///
    /// // Wildcard permissions
    /// let admin = AclEntry::new(
    ///     "admin".to_string(),
    ///     "*".to_string(),
    ///     vec!["*".to_string()],
    ///     AclPolicy::Allow
    /// );
    /// ```
    pub fn new(
        identity: String,
        resource_pattern: String,
        permissions: Vec<String>,
        policy: AclPolicy,
    ) -> Self {
        Self {
            identity,
            resource_pattern,
            permissions,
            policy,
        }
    }

    /// Check if this entry matches the given identity.
    ///
    /// Supports exact matching and wildcard ("*") for all identities.
    pub fn matches_identity(&self, identity: &str) -> bool {
        self.identity == identity || self.identity == "*"
    }

    /// Check if this entry matches the given resource pattern.
    ///
    /// Uses glob pattern matching to support flexible resource patterns.
    ///
    /// # Supported Patterns
    ///
    /// - `*` - matches any sequence (e.g., `/path/*` matches `/path/file.txt`)
    /// - `?` - matches single character (e.g., `file?.txt` matches `file1.txt`)
    /// - `**` - matches any depth (e.g., `/etc/**/*.conf` matches nested configs)
    /// - `[abc]` - matches character class
    /// - `{a,b}` - matches alternatives
    ///
    /// # Examples
    ///
    /// ```
    /// use airssys_osl::middleware::security::acl::{AclEntry, AclPolicy};
    ///
    /// let entry = AclEntry::new(
    ///     "alice".to_string(),
    ///     "/home/alice/*".to_string(),
    ///     vec!["read".to_string()],
    ///     AclPolicy::Allow
    /// );
    ///
    /// assert!(entry.matches_resource("/home/alice/file.txt"));
    /// assert!(!entry.matches_resource("/home/bob/file.txt"));
    /// ```
    pub fn matches_resource(&self, resource: &str) -> bool {
        // Wildcard matches everything
        if self.resource_pattern == "*" {
            return true;
        }

        // Try glob pattern matching
        Pattern::new(&self.resource_pattern)
            .map(|pattern| pattern.matches(resource))
            .unwrap_or(false) // Invalid patterns don't match
    }

    /// Check if this entry grants the required permission.
    ///
    /// Uses glob pattern matching to support flexible permission patterns.
    ///
    /// # Permission Semantics
    ///
    /// - **Empty permissions** (`[]`): No permissions granted, returns `false`
    /// - **Wildcard** (`["*"]`): All permissions granted, returns `true`
    /// - **Specific**: Checks if any permission pattern matches the required permission
    ///
    /// # Examples
    ///
    /// ```
    /// use airssys_osl::middleware::security::acl::{AclEntry, AclPolicy};
    ///
    /// // Wildcard permissions
    /// let admin = AclEntry::new(
    ///     "admin".to_string(),
    ///     "*".to_string(),
    ///     vec!["*".to_string()],
    ///     AclPolicy::Allow
    /// );
    /// assert!(admin.matches_permission("read"));
    /// assert!(admin.matches_permission("write"));
    ///
    /// // Specific permissions
    /// let user = AclEntry::new(
    ///     "user".to_string(),
    ///     "*".to_string(),
    ///     vec!["read".to_string()],
    ///     AclPolicy::Allow
    /// );
    /// assert!(user.matches_permission("read"));
    /// assert!(!user.matches_permission("write"));
    ///
    /// // Glob pattern permissions
    /// let reader = AclEntry::new(
    ///     "reader".to_string(),
    ///     "*".to_string(),
    ///     vec!["read*".to_string()],
    ///     AclPolicy::Allow
    /// );
    /// assert!(reader.matches_permission("read"));
    /// assert!(reader.matches_permission("read_metadata"));
    /// assert!(!reader.matches_permission("write"));
    /// ```
    pub fn matches_permission(&self, required: &str) -> bool {
        // Empty permissions = no permissions granted
        if self.permissions.is_empty() {
            return false;
        }

        // Check if any permission pattern matches the required permission
        self.permissions.iter().any(|perm_pattern| {
            // Wildcard grants all permissions
            if perm_pattern == "*" {
                return true;
            }

            // Try glob pattern matching
            Pattern::new(perm_pattern)
                .map(|pattern| pattern.matches(required))
                .unwrap_or(false) // Invalid patterns don't match
        })
    }
}

/// Access Control List security policy.
///
/// Implements ACL-based access control with configurable default policy.
/// ACL entries are evaluated in order, with the first matching entry
/// determining the policy decision.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AccessControlList {
    /// ACL entries to evaluate
    entries: Vec<AclEntry>,

    /// Default policy when no entries match
    default_policy: AclPolicy,
}

impl AccessControlList {
    /// Create a new empty Access Control List.
    pub fn new() -> Self {
        Self {
            entries: Vec::new(),
            default_policy: AclPolicy::Deny, // Deny-by-default
        }
    }

    /// Set the default policy for this ACL.
    pub fn with_default_policy(mut self, policy: AclPolicy) -> Self {
        self.default_policy = policy;
        self
    }

    /// Add an ACL entry.
    pub fn add_entry(mut self, entry: AclEntry) -> Self {
        self.entries.push(entry);
        self
    }

    /// Add multiple ACL entries.
    pub fn with_entries(mut self, entries: Vec<AclEntry>) -> Self {
        self.entries.extend(entries);
        self
    }
}

impl Default for AccessControlList {
    fn default() -> Self {
        Self::new()
    }
}

impl SecurityPolicy for AccessControlList {
    fn evaluate(&self, context: &SecurityContext) -> PolicyDecision {
        // 1. Extract resource from context.attributes
        let resource = context
            .attributes
            .get(ATTR_ACL_RESOURCE)
            .map(|s| s.as_str())
            .unwrap_or("");

        // 2. Extract required permission from context.attributes
        let permission = context
            .attributes
            .get(ATTR_ACL_PERMISSION)
            .map(|s| s.as_str())
            .unwrap_or("");

        let principal = &context.principal;

        // 3. Evaluate entries in order - check identity AND resource AND permission
        for entry in &self.entries {
            // Check if identity matches
            if !entry.matches_identity(principal) {
                continue; // Skip this entry, check next
            }

            // Check if resource matches
            if !entry.matches_resource(resource) {
                continue; // Skip this entry, check next
            }

            // Check if permission matches (skip if no permission required)
            if !permission.is_empty() && !entry.matches_permission(permission) {
                continue; // Skip this entry, check next
            }

            // All checks passed - apply this entry's policy
            return match entry.policy {
                AclPolicy::Allow => PolicyDecision::Allow,
                AclPolicy::Deny => PolicyDecision::Deny(format!(
                    "ACL policy denies '{}' access to '{}' for '{}'",
                    if permission.is_empty() {
                        "any"
                    } else {
                        permission
                    },
                    if resource.is_empty() {
                        "any resource"
                    } else {
                        resource
                    },
                    principal
                )),
            };
        }

        // No matching entry - apply default policy
        match self.default_policy {
            AclPolicy::Allow => PolicyDecision::Allow,
            AclPolicy::Deny => PolicyDecision::Deny(format!(
                "ACL default policy denies access to '{}' for '{}'",
                if resource.is_empty() {
                    "resource"
                } else {
                    resource
                },
                principal
            )),
        }
    }

    fn description(&self) -> &str {
        "Access Control List (ACL) Policy"
    }

    fn scope(&self) -> PolicyScope {
        PolicyScope::All
    }
}

/// Build ACL-specific attributes from operation permissions.
///
/// Extracts resource path and permission type from Permission enum variants
/// to populate SecurityContext attributes required by ACL policy evaluation.
///
/// This function maintains separation of concerns: operations declare what
/// permissions they need, and this function interprets those permissions into
/// ACL-specific attributes.
///
/// # Arguments
///
/// * `permissions` - Slice of Permission enum variants from an Operation
///
/// # Returns
///
/// HashMap with ACL attribute keys:
/// - [`ATTR_ACL_RESOURCE`]: The resource being accessed (path, endpoint, etc.)
/// - [`ATTR_ACL_PERMISSION`]: The permission type (read, write, execute, etc.)
///
/// # Examples
///
/// ```
/// use airssys_osl::middleware::security::acl::build_acl_attributes;
/// use airssys_osl::core::operation::Permission;
///
/// let perms = vec![Permission::FilesystemRead("/tmp/file.txt".to_string())];
/// let attrs = build_acl_attributes(&perms);
///
/// assert_eq!(attrs.get("acl.resource"), Some(&"/tmp/file.txt".to_string()));
/// assert_eq!(attrs.get("acl.permission"), Some(&"read".to_string()));
/// ```
///
/// # Permission Mapping
///
/// - `FilesystemRead(path)` → `{ "acl.resource": path, "acl.permission": "read" }`
/// - `FilesystemWrite(path)` → `{ "acl.resource": path, "acl.permission": "write" }`
/// - `FilesystemExecute(path)` → `{ "acl.resource": path, "acl.permission": "execute" }`
/// - `ProcessSpawn` → `{ "acl.resource": "process", "acl.permission": "spawn" }`
/// - `ProcessManage` → `{ "acl.resource": "process", "acl.permission": "manage" }`
/// - `NetworkSocket` → `{ "acl.resource": "network", "acl.permission": "socket" }`
/// - `NetworkConnect(endpoint)` → `{ "acl.resource": endpoint, "acl.permission": "connect" }`
/// - `UtilityExecute(utility)` → `{ "acl.resource": utility, "acl.permission": "execute" }`
///
/// # Notes
///
/// - Returns empty HashMap if permissions slice is empty
/// - For multiple permissions, last one wins (current operations have single permission)
/// - Future compound operations may require different strategy (documented in ADR-030)
pub fn build_acl_attributes(
    permissions: &[crate::core::operation::Permission],
) -> std::collections::HashMap<String, String> {
    use crate::core::operation::Permission;
    use std::collections::HashMap;

    let mut attributes = HashMap::new();

    for perm in permissions {
        match perm {
            Permission::FilesystemRead(path) => {
                attributes.insert(ATTR_ACL_RESOURCE.to_string(), path.clone());
                attributes.insert(ATTR_ACL_PERMISSION.to_string(), "read".to_string());
            }
            Permission::FilesystemWrite(path) => {
                attributes.insert(ATTR_ACL_RESOURCE.to_string(), path.clone());
                attributes.insert(ATTR_ACL_PERMISSION.to_string(), "write".to_string());
            }
            Permission::FilesystemExecute(path) => {
                attributes.insert(ATTR_ACL_RESOURCE.to_string(), path.clone());
                attributes.insert(ATTR_ACL_PERMISSION.to_string(), "execute".to_string());
            }
            Permission::ProcessSpawn => {
                attributes.insert(ATTR_ACL_RESOURCE.to_string(), "process".to_string());
                attributes.insert(ATTR_ACL_PERMISSION.to_string(), "spawn".to_string());
            }
            Permission::ProcessManage => {
                attributes.insert(ATTR_ACL_RESOURCE.to_string(), "process".to_string());
                attributes.insert(ATTR_ACL_PERMISSION.to_string(), "manage".to_string());
            }
            Permission::NetworkSocket => {
                attributes.insert(ATTR_ACL_RESOURCE.to_string(), "network".to_string());
                attributes.insert(ATTR_ACL_PERMISSION.to_string(), "socket".to_string());
            }
            Permission::NetworkConnect(endpoint) => {
                attributes.insert(ATTR_ACL_RESOURCE.to_string(), endpoint.clone());
                attributes.insert(ATTR_ACL_PERMISSION.to_string(), "connect".to_string());
            }
            Permission::UtilityExecute(utility) => {
                attributes.insert(ATTR_ACL_RESOURCE.to_string(), utility.clone());
                attributes.insert(ATTR_ACL_PERMISSION.to_string(), "execute".to_string());
            }
        }
    }

    attributes
}

#[cfg(test)]
mod tests {
    #![allow(clippy::panic)]
    #![allow(clippy::unwrap_used)]
    #![allow(clippy::uninlined_format_args)]

    use super::*;

    #[test]
    fn test_acl_entry_creation() {
        let entry = AclEntry::new(
            "user1".to_string(),
            "/path/*".to_string(),
            vec!["read".to_string(), "write".to_string()],
            AclPolicy::Allow,
        );
        assert_eq!(entry.identity, "user1");
        assert_eq!(entry.resource_pattern, "/path/*");
        assert_eq!(entry.permissions.len(), 2);
        assert_eq!(entry.policy, AclPolicy::Allow);
    }

    #[test]
    fn test_acl_entry_identity_matching() {
        let entry = AclEntry::new(
            "user1".to_string(),
            "*".to_string(),
            vec!["*".to_string()],
            AclPolicy::Allow,
        );
        assert!(entry.matches_identity("user1"));
        assert!(!entry.matches_identity("user2"));

        let wildcard_entry = AclEntry::new(
            "*".to_string(),
            "*".to_string(),
            vec!["*".to_string()],
            AclPolicy::Allow,
        );
        assert!(wildcard_entry.matches_identity("anyone"));
    }

    #[test]
    fn test_acl_default_deny() {
        let acl = AccessControlList::new();
        assert_eq!(acl.default_policy, AclPolicy::Deny);
    }

    #[test]
    fn test_acl_with_default_policy() {
        let acl = AccessControlList::new().with_default_policy(AclPolicy::Allow);
        assert_eq!(acl.default_policy, AclPolicy::Allow);
    }

    #[test]
    fn test_acl_add_entry() {
        let entry = AclEntry::new(
            "user1".to_string(),
            "*".to_string(),
            vec!["read".to_string()],
            AclPolicy::Allow,
        );
        let acl = AccessControlList::new().add_entry(entry);
        assert_eq!(acl.entries.len(), 1);
    }

    // ========== NEW COMPREHENSIVE TESTS ==========

    #[test]
    fn test_resource_glob_wildcard() {
        let entry = AclEntry::new(
            "alice".to_string(),
            "*".to_string(),
            vec!["read".to_string()],
            AclPolicy::Allow,
        );
        assert!(entry.matches_resource("/any/path"));
        assert!(entry.matches_resource("/etc/passwd"));
        assert!(entry.matches_resource("anything"));
    }

    #[test]
    fn test_resource_glob_prefix() {
        let entry = AclEntry::new(
            "alice".to_string(),
            "/home/alice/*".to_string(),
            vec!["read".to_string()],
            AclPolicy::Allow,
        );
        assert!(entry.matches_resource("/home/alice/file.txt"));
        assert!(entry.matches_resource("/home/alice/documents"));
        assert!(!entry.matches_resource("/home/bob/file.txt"));
        assert!(!entry.matches_resource("/etc/passwd"));
    }

    #[test]
    fn test_resource_glob_exact_match() {
        let entry = AclEntry::new(
            "alice".to_string(),
            "/etc/passwd".to_string(),
            vec!["read".to_string()],
            AclPolicy::Allow,
        );
        assert!(entry.matches_resource("/etc/passwd"));
        assert!(!entry.matches_resource("/etc/shadow"));
        assert!(!entry.matches_resource("/etc/passwd.bak"));
    }

    #[test]
    fn test_permission_empty_denies_all() {
        let entry = AclEntry::new(
            "alice".to_string(),
            "*".to_string(),
            vec![], // Empty permissions
            AclPolicy::Allow,
        );
        assert!(!entry.matches_permission("read"));
        assert!(!entry.matches_permission("write"));
        assert!(!entry.matches_permission("anything"));
    }

    #[test]
    fn test_permission_wildcard_allows_all() {
        let entry = AclEntry::new(
            "admin".to_string(),
            "*".to_string(),
            vec!["*".to_string()],
            AclPolicy::Allow,
        );
        assert!(entry.matches_permission("read"));
        assert!(entry.matches_permission("write"));
        assert!(entry.matches_permission("delete"));
        assert!(entry.matches_permission("anything"));
    }

    #[test]
    fn test_permission_specific_exact_match() {
        let entry = AclEntry::new(
            "reader".to_string(),
            "*".to_string(),
            vec!["read".to_string()],
            AclPolicy::Allow,
        );
        assert!(entry.matches_permission("read"));
        assert!(!entry.matches_permission("write"));
        assert!(!entry.matches_permission("read_metadata"));
    }

    #[test]
    fn test_permission_glob_pattern() {
        let entry = AclEntry::new(
            "reader".to_string(),
            "*".to_string(),
            vec!["read*".to_string()],
            AclPolicy::Allow,
        );
        assert!(entry.matches_permission("read"));
        assert!(entry.matches_permission("read_metadata"));
        assert!(entry.matches_permission("read_content"));
        assert!(!entry.matches_permission("write"));
    }

    #[test]
    fn test_permission_multiple_specific() {
        let entry = AclEntry::new(
            "user".to_string(),
            "*".to_string(),
            vec!["read".to_string(), "write".to_string()],
            AclPolicy::Allow,
        );
        assert!(entry.matches_permission("read"));
        assert!(entry.matches_permission("write"));
        assert!(!entry.matches_permission("delete"));
    }

    #[test]
    fn test_acl_evaluate_with_context_attributes() {
        use chrono::Utc;
        use std::collections::HashMap;
        use uuid::Uuid;

        let mut attributes = HashMap::new();
        attributes.insert(
            ATTR_ACL_RESOURCE.to_string(),
            "/home/alice/file.txt".to_string(),
        );
        attributes.insert(ATTR_ACL_PERMISSION.to_string(), "read".to_string());

        let context = SecurityContext {
            principal: "alice".to_string(),
            session_id: Uuid::new_v4(),
            established_at: Utc::now(),
            attributes,
        };

        let entry = AclEntry::new(
            "alice".to_string(),
            "/home/alice/*".to_string(),
            vec!["read".to_string()],
            AclPolicy::Allow,
        );

        let acl = AccessControlList::new().add_entry(entry);

        match acl.evaluate(&context) {
            PolicyDecision::Allow => {
                // Success - all conditions matched
            }
            other => panic!("Expected Allow, got {:?}", other),
        }
    }

    #[test]
    fn test_acl_evaluate_deny_wrong_permission() {
        use chrono::Utc;
        use std::collections::HashMap;
        use uuid::Uuid;

        let mut attributes = HashMap::new();
        attributes.insert(
            ATTR_ACL_RESOURCE.to_string(),
            "/home/alice/file.txt".to_string(),
        );
        attributes.insert(ATTR_ACL_PERMISSION.to_string(), "write".to_string());

        let context = SecurityContext {
            principal: "alice".to_string(),
            session_id: Uuid::new_v4(),
            established_at: Utc::now(),
            attributes,
        };

        let entry = AclEntry::new(
            "alice".to_string(),
            "/home/alice/*".to_string(),
            vec!["read".to_string()], // Only read, not write
            AclPolicy::Allow,
        );

        let acl = AccessControlList::new().add_entry(entry);

        match acl.evaluate(&context) {
            PolicyDecision::Deny(_) => {
                // Success - denied due to wrong permission
            }
            other => panic!("Expected Deny, got {:?}", other),
        }
    }

    #[test]
    fn test_acl_evaluate_deny_wrong_resource() {
        use chrono::Utc;
        use std::collections::HashMap;
        use uuid::Uuid;

        let mut attributes = HashMap::new();
        attributes.insert(
            ATTR_ACL_RESOURCE.to_string(),
            "/home/bob/file.txt".to_string(),
        );
        attributes.insert(ATTR_ACL_PERMISSION.to_string(), "read".to_string());

        let context = SecurityContext {
            principal: "alice".to_string(),
            session_id: Uuid::new_v4(),
            established_at: Utc::now(),
            attributes,
        };

        let entry = AclEntry::new(
            "alice".to_string(),
            "/home/alice/*".to_string(),
            vec!["read".to_string()],
            AclPolicy::Allow,
        );

        let acl = AccessControlList::new().add_entry(entry);

        match acl.evaluate(&context) {
            PolicyDecision::Deny(_) => {
                // Success - denied due to wrong resource
            }
            other => panic!("Expected Deny, got {:?}", other),
        }
    }

    #[test]
    fn test_acl_evaluate_multiple_entries_first_match() {
        use chrono::Utc;
        use std::collections::HashMap;
        use uuid::Uuid;

        let mut attributes = HashMap::new();
        attributes.insert(ATTR_ACL_RESOURCE.to_string(), "/data/file.txt".to_string());
        attributes.insert(ATTR_ACL_PERMISSION.to_string(), "read".to_string());

        let context = SecurityContext {
            principal: "alice".to_string(),
            session_id: Uuid::new_v4(),
            established_at: Utc::now(),
            attributes,
        };

        let entry1 = AclEntry::new(
            "alice".to_string(),
            "/data/*".to_string(),
            vec!["read".to_string()],
            AclPolicy::Allow,
        );

        let entry2 = AclEntry::new(
            "alice".to_string(),
            "/data/*".to_string(),
            vec!["write".to_string()],
            AclPolicy::Deny,
        );

        let acl = AccessControlList::new().add_entry(entry1).add_entry(entry2);

        // First matching entry should win (Allow with read permission)
        match acl.evaluate(&context) {
            PolicyDecision::Allow => {
                // Success - first entry matched
            }
            other => panic!("Expected Allow from first entry, got {:?}", other),
        }
    }

    #[test]
    fn test_acl_evaluate_explicit_deny() {
        use chrono::Utc;
        use std::collections::HashMap;
        use uuid::Uuid;

        let mut attributes = HashMap::new();
        attributes.insert(ATTR_ACL_RESOURCE.to_string(), "/etc/shadow".to_string());
        attributes.insert(ATTR_ACL_PERMISSION.to_string(), "read".to_string());

        let context = SecurityContext {
            principal: "alice".to_string(),
            session_id: Uuid::new_v4(),
            established_at: Utc::now(),
            attributes,
        };

        let entry = AclEntry::new(
            "alice".to_string(),
            "/etc/shadow".to_string(),
            vec!["*".to_string()],
            AclPolicy::Deny, // Explicit deny
        );

        let acl = AccessControlList::new().add_entry(entry);

        match acl.evaluate(&context) {
            PolicyDecision::Deny(reason) => {
                assert!(reason.contains("denies"));
            }
            other => panic!("Expected Deny, got {:?}", other),
        }
    }

    #[test]
    fn test_acl_evaluate_default_policy_deny() {
        use chrono::Utc;
        use std::collections::HashMap;
        use uuid::Uuid;

        let mut attributes = HashMap::new();
        attributes.insert(
            ATTR_ACL_RESOURCE.to_string(),
            "/random/file.txt".to_string(),
        );
        attributes.insert(ATTR_ACL_PERMISSION.to_string(), "read".to_string());

        let context = SecurityContext {
            principal: "bob".to_string(), // No entries for bob
            session_id: Uuid::new_v4(),
            established_at: Utc::now(),
            attributes,
        };

        let entry = AclEntry::new(
            "alice".to_string(),
            "/home/alice/*".to_string(),
            vec!["read".to_string()],
            AclPolicy::Allow,
        );

        let acl = AccessControlList::new().add_entry(entry);
        // Default policy is Deny

        match acl.evaluate(&context) {
            PolicyDecision::Deny(reason) => {
                assert!(reason.contains("default"));
            }
            other => panic!("Expected default Deny, got {:?}", other),
        }
    }

    #[test]
    fn test_acl_evaluate_no_permission_required() {
        use chrono::Utc;
        use std::collections::HashMap;
        use uuid::Uuid;

        let mut attributes = HashMap::new();
        attributes.insert(
            ATTR_ACL_RESOURCE.to_string(),
            "/public/file.txt".to_string(),
        );
        // No permission attribute - skip permission check

        let context = SecurityContext {
            principal: "alice".to_string(),
            session_id: Uuid::new_v4(),
            established_at: Utc::now(),
            attributes,
        };

        let entry = AclEntry::new(
            "alice".to_string(),
            "/public/*".to_string(),
            vec!["read".to_string()], // Has read, but no permission required
            AclPolicy::Allow,
        );

        let acl = AccessControlList::new().add_entry(entry);

        match acl.evaluate(&context) {
            PolicyDecision::Allow => {
                // Success - permission check skipped when not required
            }
            other => panic!("Expected Allow (no permission check), got {:?}", other),
        }
    }
}
