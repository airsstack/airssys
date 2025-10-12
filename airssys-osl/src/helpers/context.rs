//! Security context building utilities for helper functions.
//!
//! This module provides helper functions for constructing `SecurityContext`
//! instances with proper security attributes for ACL and RBAC policies.

use std::collections::HashMap;

use crate::core::context::SecurityContext;
use crate::core::operation::Operation;
use crate::middleware::security::acl::build_acl_attributes;
use crate::middleware::security::rbac::build_rbac_attributes;

/// Builds a complete security context with ACL and RBAC attributes.
///
/// This function creates a `SecurityContext` for the given user and operation,
/// automatically populating security attributes required by both ACL and RBAC
/// security policies.
///
/// # Security Architecture
///
/// The function coordinates attribute building across security modules:
/// - **ACL module**: Extracts resource path and permission type from operation
/// - **RBAC module**: Maps operation permissions to role-based permission names
///
/// This separation of concerns ensures:
/// - Operations declare what permissions they need
/// - Security modules interpret those permissions for their domain
/// - Helper functions coordinate the integration seamlessly
///
/// # Arguments
///
/// * `operation` - The operation being authorized, containing permissions
/// * `user` - The principal (username/service) executing the operation
///
/// # Returns
///
/// A `SecurityContext` populated with:
/// - `principal`: The executing user/service
/// - `session_id`: Auto-generated UUID for this session
/// - `established_at`: Current timestamp
/// - `attributes`: Combined ACL and RBAC security attributes
///
/// # Implementation Note
///
/// This function uses the first permission from the operation's permission list,
/// following the established architectural decision (ADR-030) that operations
/// should declare their primary permission first.
pub fn build_security_context<O: Operation>(operation: &O, user: &str) -> SecurityContext {
    let mut attributes = HashMap::new();

    // Build ACL attributes from operation permissions
    let acl_attrs = build_acl_attributes(&operation.required_permissions());
    attributes.extend(acl_attrs);

    // Build RBAC attributes from operation permissions
    let rbac_attrs = build_rbac_attributes(&operation.required_permissions());
    attributes.extend(rbac_attrs);

    // Create SecurityContext with combined attributes
    SecurityContext::new(user.to_string()).with_attributes(attributes)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::middleware::security::acl::{ATTR_ACL_PERMISSION, ATTR_ACL_RESOURCE};
    use crate::middleware::security::rbac::ATTR_RBAC_REQUIRED_PERMISSION;
    use crate::operations::filesystem::{FileReadOperation, FileWriteOperation};
    use crate::operations::network::NetworkConnectOperation;
    use crate::operations::process::ProcessSpawnOperation;

    #[test]
    fn test_build_security_context_filesystem_read() {
        let operation = FileReadOperation::new("/etc/passwd".to_string());
        let context = build_security_context(&operation, "alice");

        assert_eq!(context.principal, "alice");
        assert_eq!(
            context.get_attribute(ATTR_ACL_RESOURCE),
            Some("/etc/passwd")
        );
        assert_eq!(context.get_attribute(ATTR_ACL_PERMISSION), Some("read"));
        assert_eq!(
            context.get_attribute(ATTR_RBAC_REQUIRED_PERMISSION),
            Some("file:read")
        );
    }

    #[test]
    fn test_build_security_context_filesystem_write() {
        let operation = FileWriteOperation::new("/var/log/app.log".to_string(), vec![]);
        let context = build_security_context(&operation, "bob");

        assert_eq!(context.principal, "bob");
        assert_eq!(
            context.get_attribute(ATTR_ACL_RESOURCE),
            Some("/var/log/app.log")
        );
        assert_eq!(context.get_attribute(ATTR_ACL_PERMISSION), Some("write"));
        assert_eq!(
            context.get_attribute(ATTR_RBAC_REQUIRED_PERMISSION),
            Some("file:write")
        );
    }

    #[test]
    fn test_build_security_context_process_spawn() {
        let operation = ProcessSpawnOperation::new("/bin/ls".to_string());
        let context = build_security_context(&operation, "charlie");

        assert_eq!(context.principal, "charlie");
        assert_eq!(context.get_attribute(ATTR_ACL_RESOURCE), Some("process"));
        assert_eq!(context.get_attribute(ATTR_ACL_PERMISSION), Some("spawn"));
        assert_eq!(
            context.get_attribute(ATTR_RBAC_REQUIRED_PERMISSION),
            Some("process:spawn")
        );
    }

    #[test]
    fn test_build_security_context_network_connect() {
        let operation = NetworkConnectOperation::new("https://api.example.com".to_string());
        let context = build_security_context(&operation, "dave");

        assert_eq!(context.principal, "dave");
        assert_eq!(
            context.get_attribute(ATTR_ACL_RESOURCE),
            Some("https://api.example.com")
        );
        assert_eq!(context.get_attribute(ATTR_ACL_PERMISSION), Some("connect"));
        assert_eq!(
            context.get_attribute(ATTR_RBAC_REQUIRED_PERMISSION),
            Some("network:connect")
        );
    }

    #[test]
    fn test_build_security_context_preserves_session() {
        let operation1 = FileReadOperation::new("/tmp/test".to_string());
        let operation2 = FileReadOperation::new("/tmp/test".to_string());
        let context1 = build_security_context(&operation1, "eve");
        let context2 = build_security_context(&operation2, "eve");

        // Each context gets a unique session ID
        assert_ne!(context1.session_id, context2.session_id);
    }

    #[test]
    fn test_build_security_context_combines_attributes() {
        let operation = FileWriteOperation::new("/etc/config".to_string(), vec![]);
        let context = build_security_context(&operation, "frank");

        // Should have attributes from both ACL and RBAC modules
        assert!(context.has_attribute(ATTR_ACL_RESOURCE));
        assert!(context.has_attribute(ATTR_ACL_PERMISSION));
        assert!(context.has_attribute(ATTR_RBAC_REQUIRED_PERMISSION));

        // Verify all expected attributes are present
        assert_eq!(context.attributes.len(), 3);
    }
}
