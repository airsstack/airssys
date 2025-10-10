//! Integration tests for SecurityMiddleware with policy evaluation.

#![allow(clippy::expect_used)]
#![allow(clippy::unwrap_used)]

use airssys_osl::core::context::{ExecutionContext, SecurityContext};
use airssys_osl::core::middleware::Middleware;
use airssys_osl::core::security::SecurityConfig;
use airssys_osl::middleware::security::acl::{AccessControlList, AclEntry, AclPolicy};
use airssys_osl::middleware::security::middleware::SecurityMiddlewareBuilder;
use airssys_osl::middleware::security::rbac::RoleBasedAccessControl;
use airssys_osl::operations::filesystem::read::FileReadOperation;
use airssys_osl::operations::process::spawn::ProcessSpawnOperation;

#[tokio::test]
async fn test_security_middleware_deny_by_default() {
    // No policies configured - should deny by default
    let middleware = SecurityMiddlewareBuilder::new()
        .with_config(SecurityConfig::default())
        .build()
        .expect("Failed to build middleware");

    let operation = FileReadOperation::new("/etc/passwd".to_string());
    let context = ExecutionContext::new(SecurityContext::new("testuser".to_string()));

    let result = middleware.before_execution(operation, &context).await;

    assert!(result.is_err());
    if let Err(e) = result {
        let error_msg = format!("{e:?}");
        assert!(error_msg.contains("No security policies configured"));
    }
}

#[tokio::test]
async fn test_security_middleware_with_acl_allow() {
    // Create ACL that allows testuser
    let acl = AccessControlList::new().add_entry(AclEntry::new(
        "testuser".to_string(),
        "*".to_string(),
        vec!["*".to_string()], // Allow all permissions
        AclPolicy::Allow,
    ));

    let middleware = SecurityMiddlewareBuilder::new()
        .with_config(SecurityConfig::default())
        .add_policy(Box::new(acl))
        .build()
        .expect("Failed to build middleware");

    let operation = FileReadOperation::new("/etc/passwd".to_string());
    let context = ExecutionContext::new(SecurityContext::new("testuser".to_string()));

    let result = middleware.before_execution(operation, &context).await;

    assert!(result.is_ok());
    assert!(result.unwrap().is_some());
}

#[tokio::test]
async fn test_security_middleware_with_acl_deny() {
    // Create ACL that denies testuser
    let acl = AccessControlList::new().add_entry(AclEntry::new(
        "testuser".to_string(),
        "*".to_string(),
        vec!["*".to_string()], // Deny all permissions
        AclPolicy::Deny,
    ));

    let middleware = SecurityMiddlewareBuilder::new()
        .with_config(SecurityConfig::default())
        .add_policy(Box::new(acl))
        .build()
        .expect("Failed to build middleware");

    let operation = FileReadOperation::new("/etc/passwd".to_string());
    let context = ExecutionContext::new(SecurityContext::new("testuser".to_string()));

    let result = middleware.before_execution(operation, &context).await;

    assert!(result.is_err());
    if let Err(e) = result {
        let error_msg = format!("{e:?}");
        assert!(error_msg.contains("ACL policy denies"));
    }
}

#[tokio::test]
async fn test_security_middleware_with_rbac_allow() {
    // Create RBAC that allows users with roles
    let mut rbac = RoleBasedAccessControl::new();
    rbac = rbac.assign_roles("testuser".to_string(), vec!["user".to_string()]);

    let middleware = SecurityMiddlewareBuilder::new()
        .with_config(SecurityConfig::default())
        .add_policy(Box::new(rbac))
        .build()
        .expect("Failed to build middleware");

    let operation = ProcessSpawnOperation::new("ls".to_string());
    let context = ExecutionContext::new(SecurityContext::new("testuser".to_string()));

    let result = middleware.before_execution(operation, &context).await;

    assert!(result.is_ok());
    assert!(result.unwrap().is_some());
}

#[tokio::test]
async fn test_security_middleware_with_rbac_deny() {
    // Create RBAC but don't assign any roles to testuser
    let rbac = RoleBasedAccessControl::new();

    let middleware = SecurityMiddlewareBuilder::new()
        .with_config(SecurityConfig::default())
        .add_policy(Box::new(rbac))
        .build()
        .expect("Failed to build middleware");

    let operation = ProcessSpawnOperation::new("ls".to_string());
    let context = ExecutionContext::new(SecurityContext::new("testuser".to_string()));

    let result = middleware.before_execution(operation, &context).await;

    assert!(result.is_err());
    if let Err(e) = result {
        let error_msg = format!("{e:?}");
        assert!(error_msg.contains("No roles assigned"));
    }
}

#[tokio::test]
async fn test_security_middleware_multiple_policies() {
    // Create both ACL and RBAC policies
    let acl = AccessControlList::new().add_entry(AclEntry::new(
        "testuser".to_string(),
        "*".to_string(),
        vec!["*".to_string()], // Allow all permissions
        AclPolicy::Allow,
    ));

    let mut rbac = RoleBasedAccessControl::new();
    rbac = rbac.assign_roles("testuser".to_string(), vec!["user".to_string()]);

    let middleware = SecurityMiddlewareBuilder::new()
        .with_config(SecurityConfig::default())
        .add_policy(Box::new(acl))
        .add_policy(Box::new(rbac))
        .build()
        .expect("Failed to build middleware");

    let operation = FileReadOperation::new("/etc/passwd".to_string());
    let context = ExecutionContext::new(SecurityContext::new("testuser".to_string()));

    let result = middleware.before_execution(operation, &context).await;

    // Both policies should allow
    assert!(result.is_ok());
    assert!(result.unwrap().is_some());
}

#[tokio::test]
async fn test_security_middleware_any_deny_blocks() {
    // ACL allows but RBAC denies - should deny overall
    let acl = AccessControlList::new().add_entry(AclEntry::new(
        "testuser".to_string(),
        "*".to_string(),
        vec!["*".to_string()], // Allow all permissions
        AclPolicy::Allow,
    ));

    let rbac = RoleBasedAccessControl::new(); // No roles = deny

    let middleware = SecurityMiddlewareBuilder::new()
        .with_config(SecurityConfig::default())
        .add_policy(Box::new(acl))
        .add_policy(Box::new(rbac))
        .build()
        .expect("Failed to build middleware");

    let operation = FileReadOperation::new("/etc/passwd".to_string());
    let context = ExecutionContext::new(SecurityContext::new("testuser".to_string()));

    let result = middleware.before_execution(operation, &context).await;

    // RBAC denies - should block even though ACL allows
    assert!(result.is_err());
    if let Err(e) = result {
        let error_msg = format!("{e:?}");
        assert!(error_msg.contains("No roles assigned"));
    }
}

#[tokio::test]
async fn test_security_middleware_policy_count() {
    let acl = AccessControlList::new();
    let rbac = RoleBasedAccessControl::new();

    let middleware = SecurityMiddlewareBuilder::new()
        .with_config(SecurityConfig::default())
        .add_policy(Box::new(acl))
        .add_policy(Box::new(rbac))
        .build()
        .expect("Failed to build middleware");

    assert_eq!(middleware.policy_count(), 2);
}

// ========== PHASE 6: COMPREHENSIVE INTEGRATION TESTS ==========

#[tokio::test]
async fn test_acl_with_specific_resource_path() {
    use airssys_osl::middleware::security::acl::ATTR_RESOURCE;
    use std::collections::HashMap;

    // ACL that only allows access to /tmp/* files
    let acl = AccessControlList::new().add_entry(AclEntry::new(
        "testuser".to_string(),
        "/tmp/*".to_string(),
        vec!["read".to_string()],
        AclPolicy::Allow,
    ));

    let middleware = SecurityMiddlewareBuilder::new()
        .with_config(SecurityConfig::default())
        .add_policy(Box::new(acl))
        .build()
        .expect("Failed to build middleware");

    // Test with matching resource
    let mut attributes = HashMap::new();
    attributes.insert(ATTR_RESOURCE.to_string(), "/tmp/test.txt".to_string());
    attributes.insert("permission".to_string(), "read".to_string());

    let mut security_context = SecurityContext::new("testuser".to_string());
    security_context.attributes = attributes;
    let context = ExecutionContext::new(security_context);

    let operation = FileReadOperation::new("/tmp/test.txt".to_string());
    let result = middleware.before_execution(operation, &context).await;

    assert!(result.is_ok(), "Should allow access to /tmp/* resources");
}

#[tokio::test]
async fn test_acl_with_non_matching_resource() {
    use airssys_osl::middleware::security::acl::ATTR_RESOURCE;
    use std::collections::HashMap;

    // ACL that only allows access to /tmp/* files
    let acl = AccessControlList::new().add_entry(AclEntry::new(
        "testuser".to_string(),
        "/tmp/*".to_string(),
        vec!["read".to_string()],
        AclPolicy::Allow,
    ));

    let middleware = SecurityMiddlewareBuilder::new()
        .with_config(SecurityConfig::default())
        .add_policy(Box::new(acl))
        .build()
        .expect("Failed to build middleware");

    // Test with non-matching resource
    let mut attributes = HashMap::new();
    attributes.insert(ATTR_RESOURCE.to_string(), "/etc/passwd".to_string());
    attributes.insert("permission".to_string(), "read".to_string());

    let mut security_context = SecurityContext::new("testuser".to_string());
    security_context.attributes = attributes;
    let context = ExecutionContext::new(security_context);

    let operation = FileReadOperation::new("/etc/passwd".to_string());
    let result = middleware.before_execution(operation, &context).await;

    assert!(result.is_err(), "Should deny access to /etc/* resources");
}

#[tokio::test]
async fn test_rbac_with_role_inheritance() {
    use airssys_osl::middleware::security::rbac::{Permission, Role, ATTR_REQUIRED_PERMISSION};
    use std::collections::HashMap;

    // Create role hierarchy: admin -> user
    let read_perm = Permission::new(
        "read_file".to_string(),
        "Read File".to_string(),
        "Read files".to_string(),
    );

    let user_role =
        Role::new("user".to_string(), "User".to_string()).with_permission("read_file".to_string());

    let admin_role =
        Role::new("admin".to_string(), "Admin".to_string()).inherits_from("user".to_string());

    let rbac = RoleBasedAccessControl::new()
        .add_permission(read_perm)
        .add_role(user_role)
        .add_role(admin_role)
        .assign_roles("alice".to_string(), vec!["admin".to_string()]);

    let middleware = SecurityMiddlewareBuilder::new()
        .with_config(SecurityConfig::default())
        .add_policy(Box::new(rbac))
        .build()
        .expect("Failed to build middleware");

    // Alice has admin role, which inherits read_file from user role
    let mut attributes = HashMap::new();
    attributes.insert(
        ATTR_REQUIRED_PERMISSION.to_string(),
        "read_file".to_string(),
    );

    let mut security_context = SecurityContext::new("alice".to_string());
    security_context.attributes = attributes;
    let context = ExecutionContext::new(security_context);

    let operation = FileReadOperation::new("/etc/passwd".to_string());
    let result = middleware.before_execution(operation, &context).await;

    assert!(
        result.is_ok(),
        "Admin should inherit read_file permission from user role"
    );
}

#[tokio::test]
async fn test_rbac_without_required_permission() {
    use airssys_osl::middleware::security::rbac::{Permission, Role, ATTR_REQUIRED_PERMISSION};
    use std::collections::HashMap;

    // User has read permission but not write
    let read_perm = Permission::new(
        "read_file".to_string(),
        "Read File".to_string(),
        "Read files".to_string(),
    );

    let user_role =
        Role::new("user".to_string(), "User".to_string()).with_permission("read_file".to_string());

    let rbac = RoleBasedAccessControl::new()
        .add_permission(read_perm)
        .add_role(user_role)
        .assign_roles("alice".to_string(), vec!["user".to_string()]);

    let middleware = SecurityMiddlewareBuilder::new()
        .with_config(SecurityConfig::default())
        .add_policy(Box::new(rbac))
        .build()
        .expect("Failed to build middleware");

    // Request write permission (not granted)
    let mut attributes = HashMap::new();
    attributes.insert(
        ATTR_REQUIRED_PERMISSION.to_string(),
        "write_file".to_string(),
    );

    let mut security_context = SecurityContext::new("alice".to_string());
    security_context.attributes = attributes;
    let context = ExecutionContext::new(security_context);

    let operation = FileReadOperation::new("/etc/passwd".to_string());
    let result = middleware.before_execution(operation, &context).await;

    assert!(
        result.is_err(),
        "Should deny when user lacks required permission"
    );
}

#[tokio::test]
async fn test_combined_acl_and_rbac_both_allow() {
    use airssys_osl::middleware::security::acl::{ATTR_PERMISSION, ATTR_RESOURCE};
    use airssys_osl::middleware::security::rbac::{Permission, Role, ATTR_REQUIRED_PERMISSION};
    use std::collections::HashMap;

    // ACL allows access to /tmp/* for testuser
    let acl = AccessControlList::new().add_entry(AclEntry::new(
        "testuser".to_string(),
        "/tmp/*".to_string(),
        vec!["read".to_string()],
        AclPolicy::Allow,
    ));

    // RBAC grants read_file permission to user role
    let read_perm = Permission::new(
        "read_file".to_string(),
        "Read File".to_string(),
        "Read files".to_string(),
    );

    let user_role =
        Role::new("user".to_string(), "User".to_string()).with_permission("read_file".to_string());

    let rbac = RoleBasedAccessControl::new()
        .add_permission(read_perm)
        .add_role(user_role)
        .assign_roles("testuser".to_string(), vec!["user".to_string()]);

    let middleware = SecurityMiddlewareBuilder::new()
        .with_config(SecurityConfig::default())
        .add_policy(Box::new(acl))
        .add_policy(Box::new(rbac))
        .build()
        .expect("Failed to build middleware");

    // Populate context attributes for both policies
    let mut attributes = HashMap::new();
    attributes.insert(ATTR_RESOURCE.to_string(), "/tmp/test.txt".to_string());
    attributes.insert(ATTR_PERMISSION.to_string(), "read".to_string());
    attributes.insert(
        ATTR_REQUIRED_PERMISSION.to_string(),
        "read_file".to_string(),
    );

    let mut security_context = SecurityContext::new("testuser".to_string());
    security_context.attributes = attributes;
    let context = ExecutionContext::new(security_context);

    let operation = FileReadOperation::new("/tmp/test.txt".to_string());
    let result = middleware.before_execution(operation, &context).await;

    assert!(
        result.is_ok(),
        "Both ACL and RBAC allow - operation should succeed"
    );
}

#[tokio::test]
async fn test_combined_acl_allows_rbac_denies() {
    use airssys_osl::middleware::security::acl::{ATTR_PERMISSION, ATTR_RESOURCE};
    use airssys_osl::middleware::security::rbac::ATTR_REQUIRED_PERMISSION;
    use std::collections::HashMap;

    // ACL allows
    let acl = AccessControlList::new().add_entry(AclEntry::new(
        "testuser".to_string(),
        "*".to_string(),
        vec!["*".to_string()],
        AclPolicy::Allow,
    ));

    // RBAC denies (no roles assigned)
    let rbac = RoleBasedAccessControl::new();

    let middleware = SecurityMiddlewareBuilder::new()
        .with_config(SecurityConfig::default())
        .add_policy(Box::new(acl))
        .add_policy(Box::new(rbac))
        .build()
        .expect("Failed to build middleware");

    let mut attributes = HashMap::new();
    attributes.insert(ATTR_RESOURCE.to_string(), "/tmp/test.txt".to_string());
    attributes.insert(ATTR_PERMISSION.to_string(), "read".to_string());
    attributes.insert(
        ATTR_REQUIRED_PERMISSION.to_string(),
        "read_file".to_string(),
    );

    let mut security_context = SecurityContext::new("testuser".to_string());
    security_context.attributes = attributes;
    let context = ExecutionContext::new(security_context);

    let operation = FileReadOperation::new("/tmp/test.txt".to_string());
    let result = middleware.before_execution(operation, &context).await;

    assert!(
        result.is_err(),
        "ANY deny should block operation (RBAC denies)"
    );
}

#[tokio::test]
async fn test_process_operation_security() {
    // Test security middleware with process operations
    let acl = AccessControlList::new().add_entry(AclEntry::new(
        "testuser".to_string(),
        "*".to_string(),
        vec!["execute".to_string()],
        AclPolicy::Allow,
    ));

    let middleware = SecurityMiddlewareBuilder::new()
        .with_config(SecurityConfig::default())
        .add_policy(Box::new(acl))
        .build()
        .expect("Failed to build middleware");

    let operation = ProcessSpawnOperation::new("/bin/ls".to_string());
    let context = ExecutionContext::new(SecurityContext::new("testuser".to_string()));

    let result = middleware.before_execution(operation, &context).await;

    assert!(result.is_ok(), "Should allow process execution");
}

#[tokio::test]
async fn test_network_operation_security() {
    use airssys_osl::operations::network::connect::NetworkConnectOperation;

    // Test security middleware with network operations
    let acl = AccessControlList::new().add_entry(AclEntry::new(
        "testuser".to_string(),
        "*".to_string(),
        vec!["connect".to_string()],
        AclPolicy::Allow,
    ));

    let middleware = SecurityMiddlewareBuilder::new()
        .with_config(SecurityConfig::default())
        .add_policy(Box::new(acl))
        .build()
        .expect("Failed to build middleware");

    let operation = NetworkConnectOperation::new("127.0.0.1:8080".to_string());
    let context = ExecutionContext::new(SecurityContext::new("testuser".to_string()));

    let result = middleware.before_execution(operation, &context).await;

    assert!(result.is_ok(), "Should allow network connection");
}

#[tokio::test]
async fn test_middleware_with_disabled_logging() {
    // Test that middleware can be disabled via configuration
    let acl = AccessControlList::new().add_entry(AclEntry::new(
        "testuser".to_string(),
        "*".to_string(),
        vec!["*".to_string()],
        AclPolicy::Allow,
    ));

    let config = SecurityConfig::without_logging();

    let middleware = SecurityMiddlewareBuilder::new()
        .with_config(config)
        .add_policy(Box::new(acl))
        .build()
        .expect("Failed to build middleware");

    // Middleware should be disabled when logging is disabled
    assert!(
        !<airssys_osl::middleware::security::SecurityMiddleware as Middleware<FileReadOperation>>::is_enabled(&middleware)
    );
}
