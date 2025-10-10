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
