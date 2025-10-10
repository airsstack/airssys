//! Access Control Lists (ACL) implementation.
//!
//! This module provides ACL-based security policy implementation for
//! fine-grained access control to operations.

// Layer 1: Standard library imports
use std::fmt::Debug;

// Layer 2: Third-party crate imports
use serde::{Deserialize, Serialize};

// Layer 3: Internal module imports
use crate::core::context::SecurityContext;
use crate::core::operation::Operation;
use crate::middleware::security::policy::{PolicyDecision, PolicyScope, SecurityPolicy};

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
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AclEntry {
    /// Identity this entry applies to (user, group, or service)
    pub identity: String,

    /// Resource pattern this entry applies to
    pub resource_pattern: String,

    /// Policy action (allow or deny)
    pub policy: AclPolicy,
}

impl AclEntry {
    /// Create a new ACL entry.
    pub fn new(identity: String, resource_pattern: String, policy: AclPolicy) -> Self {
        Self {
            identity,
            resource_pattern,
            policy,
        }
    }

    /// Check if this entry matches the given identity.
    pub fn matches_identity(&self, identity: &str) -> bool {
        self.identity == identity || self.identity == "*"
    }

    /// Check if this entry matches the given resource pattern.
    pub fn matches_resource(&self, _resource: &str) -> bool {
        // TODO: Implement glob pattern matching in future phases
        // For now, exact match or wildcard
        self.resource_pattern == "*"
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

impl<O: Operation> SecurityPolicy<O> for AccessControlList {
    fn evaluate(&self, _operation: &O, context: &SecurityContext) -> PolicyDecision {
        let principal = &context.principal;

        // Evaluate entries in order
        for entry in &self.entries {
            if entry.matches_identity(principal) {
                return match entry.policy {
                    AclPolicy::Allow => PolicyDecision::Allow,
                    AclPolicy::Deny => {
                        PolicyDecision::Deny(format!("ACL policy denies access for '{principal}'"))
                    }
                };
            }
        }

        // No matching entry - apply default policy
        match self.default_policy {
            AclPolicy::Allow => PolicyDecision::Allow,
            AclPolicy::Deny => PolicyDecision::Deny(format!(
                "ACL default policy denies access for '{principal}'"
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

/// Implementation of SecurityPolicyDispatcher for AccessControlList.
///
/// This allows ACL policies to be used in the SecurityMiddleware's
/// type-erased policy storage.
impl crate::middleware::security::policy::SecurityPolicyDispatcher for AccessControlList {
    fn evaluate_any(
        &self,
        _operation: &dyn std::any::Any,
        context: &SecurityContext,
    ) -> PolicyDecision {
        // ACL policies work with any operation type, so we can evaluate directly
        // without type downcasting (we only need the security context)
        
        let principal = &context.principal;

        // Evaluate entries in order
        for entry in &self.entries {
            if entry.matches_identity(principal) {
                return match entry.policy {
                    AclPolicy::Allow => PolicyDecision::Allow,
                    AclPolicy::Deny => {
                        PolicyDecision::Deny(format!("ACL policy denies access for '{principal}'"))
                    }
                };
            }
        }

        // No matching entry - apply default policy
        match self.default_policy {
            AclPolicy::Allow => PolicyDecision::Allow,
            AclPolicy::Deny => PolicyDecision::Deny(format!(
                "ACL default policy denies access for '{principal}'"
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_acl_entry_creation() {
        let entry = AclEntry::new("user1".to_string(), "/path/*".to_string(), AclPolicy::Allow);
        assert_eq!(entry.identity, "user1");
        assert_eq!(entry.resource_pattern, "/path/*");
        assert_eq!(entry.policy, AclPolicy::Allow);
    }

    #[test]
    fn test_acl_entry_identity_matching() {
        let entry = AclEntry::new("user1".to_string(), "*".to_string(), AclPolicy::Allow);
        assert!(entry.matches_identity("user1"));
        assert!(!entry.matches_identity("user2"));

        let wildcard_entry = AclEntry::new("*".to_string(), "*".to_string(), AclPolicy::Allow);
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
        let entry = AclEntry::new("user1".to_string(), "*".to_string(), AclPolicy::Allow);
        let acl = AccessControlList::new().add_entry(entry);
        assert_eq!(acl.entries.len(), 1);
    }
}
