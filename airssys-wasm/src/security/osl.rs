//! OSL (airssys-osl) security integration module.
//!
//! This module provides the bridge to airssys-osl's security infrastructure.

// Layer 1: Standard library imports
use std::collections::HashMap;

// Layer 2: Third-party crate imports
use airssys_osl::core::context::SecurityContext;
use airssys_osl::middleware::security::acl::{ATTR_ACL_PERMISSION, ATTR_ACL_RESOURCE};
use airssys_osl::middleware::security::policy::{PolicyDecision, SecurityPolicy};

// Layer 3: Internal module imports
use crate::core::security::errors::SecurityError;

/// Bridge to airssys-osl security infrastructure.
///
/// Wraps an airssys-osl security policy and provides convenient methods
/// for checking permissions in the WASM component context.
///
/// Generic parameter `P` allows any type implementing `SecurityPolicy`,
/// providing static dispatch and compile-time type safety (per §6.2).
#[derive(Debug)]
pub struct OslSecurityBridge<P: SecurityPolicy> {
    /// Underlying security policy (e.g., AccessControlList)
    policy: P,
}

impl<P: SecurityPolicy> OslSecurityBridge<P> {
    /// Creates a new security bridge with the given policy.
    ///
    /// # Arguments
    /// * `policy` - Security policy implementation (e.g., AccessControlList)
    pub fn new(policy: P) -> Self {
        Self { policy }
    }

    /// Checks if a principal has permission to perform an action on a resource.
    ///
    /// # Arguments
    /// * `principal` - The identity requesting access (e.g., component ID)
    /// * `resource` - The resource being accessed (e.g., file path, API endpoint)
    /// * `permission` - The permission being requested (e.g., "read", "write")
    ///
    /// # Errors
    /// Returns `SecurityError::PermissionDenied` if the policy denies access.
    pub fn check_permission(
        &self,
        principal: &str,
        resource: &str,
        permission: &str,
    ) -> Result<(), SecurityError> {
        // Build security context with ACL attributes
        let mut attributes = HashMap::new();
        attributes.insert(ATTR_ACL_RESOURCE.to_string(), resource.to_string());
        attributes.insert(ATTR_ACL_PERMISSION.to_string(), permission.to_string());

        let context = SecurityContext {
            principal: principal.to_string(),
            session_id: uuid::Uuid::new_v4(),
            established_at: chrono::Utc::now(),
            attributes,
        };

        // Evaluate using airssys-osl SecurityPolicy trait
        match self.policy.evaluate(&context) {
            PolicyDecision::Allow => Ok(()),
            PolicyDecision::Deny(reason) => Err(SecurityError::PermissionDenied(format!(
                "OSL denied: {} cannot {} on {}: {}",
                principal, permission, resource, reason
            ))),
            PolicyDecision::RequireAdditionalAuth(auth) => Err(SecurityError::PermissionDenied(
                format!("OSL requires additional authentication: {:?}", auth),
            )),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use airssys_osl::middleware::security::acl::{AccessControlList, AclEntry, AclPolicy};

    #[test]
    fn test_bridge_creation() {
        let acl = AccessControlList::new();
        let bridge = OslSecurityBridge::new(acl);

        // Verify bridge holds the policy
        let context = context_for_test("user", "/file", "read");
        assert!(matches!(
            bridge.policy.evaluate(&context),
            PolicyDecision::Deny(_)
        ));
    }

    #[test]
    fn test_permitted_action() {
        // Create ACL with allow entry
        let mut acl = AccessControlList::new();
        acl = acl.add_entry(AclEntry::new(
            "component-123".to_string(),
            "/data/*".to_string(),
            vec!["read".to_string()],
            AclPolicy::Allow,
        ));

        let bridge = OslSecurityBridge::new(acl);

        // Should succeed
        let result = bridge.check_permission("component-123", "/data/file.txt", "read");
        assert!(result.is_ok(), "Expected success, got: {:?}", result);
    }

    #[test]
    fn test_denied_action() {
        // Empty ACL - deny by default
        let acl = AccessControlList::new();
        let bridge = OslSecurityBridge::new(acl);

        // Should fail
        let result = bridge.check_permission("component-456", "/secret/file.txt", "write");
        assert!(result.is_err());
        assert!(matches!(result, Err(SecurityError::PermissionDenied(_))));
    }

    #[test]
    fn test_error_message_formatting() {
        let acl = AccessControlList::new();
        let bridge = OslSecurityBridge::new(acl);

        let result = bridge.check_permission("user-123", "/path", "action");
        let err = result.unwrap_err();

        // Verify error message contains key information
        let err_msg = format!("{:?}", err);
        assert!(err_msg.contains("OSL denied"));
        assert!(err_msg.contains("user-123"));
        assert!(err_msg.contains("path"));
        assert!(err_msg.contains("action"));
    }

    #[test]
    fn test_principal_mismatch() {
        let mut acl = AccessControlList::new();
        acl = acl.add_entry(AclEntry::new(
            "component-allowed".to_string(),
            "/data/*".to_string(),
            vec!["read".to_string()],
            AclPolicy::Allow,
        ));

        let bridge = OslSecurityBridge::new(acl);

        // Different principal - should be denied
        let result = bridge.check_permission("component-different", "/data/file.txt", "read");
        assert!(result.is_err());
    }

    // Helper function for building test contexts
    fn context_for_test(principal: &str, resource: &str, permission: &str) -> SecurityContext {
        let mut attributes = HashMap::new();
        attributes.insert(ATTR_ACL_RESOURCE.to_string(), resource.to_string());
        attributes.insert(ATTR_ACL_PERMISSION.to_string(), permission.to_string());

        SecurityContext {
            principal: principal.to_string(),
            session_id: uuid::Uuid::new_v4(),
            established_at: chrono::Utc::now(),
            attributes,
        }
    }
}
