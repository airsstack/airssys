#![allow(clippy::expect_used)]
#![allow(clippy::unwrap_used)]

//! # Security Threat Model Testing
//!
//! This test suite validates the security middleware against common threat scenarios
//! and attack patterns. Each test represents a realistic security threat.

use airssys_osl::core::context::{ExecutionContext, SecurityContext};
use airssys_osl::core::middleware::Middleware;
use airssys_osl::core::security::SecurityConfig;
use airssys_osl::middleware::security::acl::{AccessControlList, AclEntry, AclPolicy};
use airssys_osl::middleware::security::middleware::SecurityMiddlewareBuilder;
use airssys_osl::middleware::security::rbac::{Role, RoleBasedAccessControl};
use airssys_osl::operations::filesystem::{read::FileReadOperation, write::FileWriteOperation};
use airssys_osl::operations::network::listen::NetworkListenOperation;
use airssys_osl::operations::process::spawn::ProcessSpawnOperation;

// ================================================================================================
// Test 1: Permission Escalation - User Attempting Admin Resources
// ================================================================================================

#[tokio::test]
async fn threat_permission_escalation_attempt() {
    // Scenario: Regular user attempting to access admin-only resource
    // Expected: Deny with audit trail

    let acl = AccessControlList::new().add_entry(AclEntry::new(
        "admin".to_string(),
        "/admin/*".to_string(),
        vec!["file:read".to_string()],
        AclPolicy::Allow,
    ));

    let middleware = SecurityMiddlewareBuilder::new()
        .with_config(SecurityConfig::default())
        .add_policy(Box::new(acl))
        .build()
        .expect("Failed to build middleware");

    // Attack: Regular user trying to read admin file
    let operation = FileReadOperation::new("/admin/secrets.txt".to_string());
    let mut context = ExecutionContext::new(SecurityContext::new("regular_user".to_string()));
    context
        .security_context
        .attributes
        .insert("acl.resource".to_string(), "/admin/secrets.txt".to_string());
    context
        .security_context
        .attributes
        .insert("acl.permission".to_string(), "file:read".to_string());

    let result = middleware.before_execution(operation, &context).await;

    // Validation: Should be denied (identity mismatch: regular_user vs admin)
    assert!(result.is_err(), "Permission escalation should be blocked");
}

// ================================================================================================
// Test 2: Resource Bypass - Accessing Resources Without Proper ACL Entry
// ================================================================================================

#[tokio::test]
async fn threat_resource_access_bypass() {
    // Scenario: Attacker attempting to access resources not covered by ACL
    // Expected: Deny-by-default enforcement

    let acl = AccessControlList::new().add_entry(AclEntry::new(
        "user".to_string(),
        "/public/*".to_string(),
        vec!["file:read".to_string()],
        AclPolicy::Allow,
    ));

    let middleware = SecurityMiddlewareBuilder::new()
        .with_config(SecurityConfig::default())
        .add_policy(Box::new(acl))
        .build()
        .expect("Failed to build middleware");

    // Attack: Trying to access resource outside ACL scope
    let operation = FileReadOperation::new("/private/data.txt".to_string());
    let mut context = ExecutionContext::new(SecurityContext::new("user".to_string()));
    context
        .security_context
        .attributes
        .insert("acl.resource".to_string(), "/private/data.txt".to_string());
    context
        .security_context
        .attributes
        .insert("acl.permission".to_string(), "file:read".to_string());

    let result = middleware.before_execution(operation, &context).await;

    // Validation: Deny-by-default should block access
    assert!(
        result.is_err(),
        "Resource bypass should be blocked by deny-by-default"
    );
}

// ================================================================================================
// Test 3: Role Bypass - User Without Required Role Attempting Privileged Operation
// ================================================================================================

#[tokio::test]
async fn threat_role_bypass_attempt() {
    // Scenario: User without required role trying to perform admin operation
    // Expected: Deny with RBAC policy enforcement

    let mut rbac = RoleBasedAccessControl::new();
    rbac = rbac.add_role(Role::new("admin".to_string(), "Administrator".to_string()));
    rbac = rbac.assign_roles("admin_user".to_string(), vec!["admin".to_string()]);

    let middleware = SecurityMiddlewareBuilder::new()
        .with_config(SecurityConfig::default())
        .add_policy(Box::new(rbac))
        .build()
        .expect("Failed to build middleware");

    // Attack: Regular user without admin role attempting system operation
    let operation = ProcessSpawnOperation::new("/usr/bin/systemctl".to_string());
    let mut context = ExecutionContext::new(SecurityContext::new("regular_user".to_string()));
    context.security_context.attributes.insert(
        "required_permission".to_string(),
        "system:admin".to_string(),
    );

    let result = middleware.before_execution(operation, &context).await;

    // Validation: Should be denied (user has no roles)
    assert!(result.is_err(), "Role bypass should be blocked");
}

// ================================================================================================
// Test 4: Identity Spoofing - Empty or Invalid Principal
// ================================================================================================

#[tokio::test]
async fn threat_identity_spoofing_empty_principal() {
    // Scenario: Attacker providing empty principal to bypass identity checks
    // Expected: Deny (no match with ACL entries)

    let acl = AccessControlList::new().add_entry(AclEntry::new(
        "valid_user".to_string(),
        "/data/*".to_string(),
        vec!["file:read".to_string()],
        AclPolicy::Allow,
    ));

    let middleware = SecurityMiddlewareBuilder::new()
        .with_config(SecurityConfig::default())
        .add_policy(Box::new(acl))
        .build()
        .expect("Failed to build middleware");

    // Attack: Empty principal
    let operation = FileReadOperation::new("/data/file.txt".to_string());
    let mut context = ExecutionContext::new(SecurityContext::new("".to_string()));
    context
        .security_context
        .attributes
        .insert("acl.resource".to_string(), "/data/file.txt".to_string());
    context
        .security_context
        .attributes
        .insert("acl.permission".to_string(), "file:read".to_string());

    let result = middleware.before_execution(operation, &context).await;

    // Validation: Should be denied (empty principal doesn't match "valid_user")
    assert!(
        result.is_err(),
        "Empty principal should not bypass security"
    );
}

// ================================================================================================
// Test 5: Wildcard Exploitation - Attempting to Abuse Glob Patterns
// ================================================================================================

#[tokio::test]
async fn threat_wildcard_pattern_exploitation() {
    // Scenario: Attacker trying to exploit glob patterns to access unintended resources
    // Expected: Pattern matching follows glob semantics strictly

    let acl = AccessControlList::new().add_entry(AclEntry::new(
        "user".to_string(),
        "/public/[!s]*.txt".to_string(), // Only .txt files NOT starting with 's' in /public/
        vec!["file:read".to_string()],
        AclPolicy::Allow,
    ));

    let middleware = SecurityMiddlewareBuilder::new()
        .with_config(SecurityConfig::default())
        .add_policy(Box::new(acl))
        .build()
        .expect("Failed to build middleware");

    // Attack 1: Trying to access file starting with 's' (pattern excludes it)
    let operation = FileReadOperation::new("/public/secret.txt".to_string());
    let mut context = ExecutionContext::new(SecurityContext::new("user".to_string()));
    context
        .security_context
        .attributes
        .insert("acl.resource".to_string(), "/public/secret.txt".to_string());
    context
        .security_context
        .attributes
        .insert("acl.permission".to_string(), "file:read".to_string());

    let result = middleware.before_execution(operation, &context).await;
    assert!(result.is_err(), "File starting with 's' should be blocked");

    // Attack 2: Trying to access non-.txt file
    let operation2 = FileReadOperation::new("/public/binary.exe".to_string());
    context
        .security_context
        .attributes
        .insert("acl.resource".to_string(), "/public/binary.exe".to_string());

    let result2 = middleware.before_execution(operation2, &context).await;
    assert!(result2.is_err(), "Non-.txt file should be blocked");
}

// ================================================================================================
// Test 6: Permission String Manipulation
// ================================================================================================

#[tokio::test]
async fn threat_permission_string_manipulation() {
    // Scenario: Attacker trying to manipulate permission strings
    // Expected: Exact permission matching enforced

    let acl = AccessControlList::new().add_entry(AclEntry::new(
        "user".to_string(),
        "/data/*".to_string(),
        vec!["file:read".to_string()],
        AclPolicy::Allow,
    ));

    let middleware = SecurityMiddlewareBuilder::new()
        .with_config(SecurityConfig::default())
        .add_policy(Box::new(acl))
        .build()
        .expect("Failed to build middleware");

    // Attack: Trying to use modified permission string
    let operation =
        FileWriteOperation::new("/data/file.txt".to_string(), "data".as_bytes().to_vec());
    let mut context = ExecutionContext::new(SecurityContext::new("user".to_string()));
    context
        .security_context
        .attributes
        .insert("acl.resource".to_string(), "/data/file.txt".to_string());
    context
        .security_context
        .attributes
        .insert("acl.permission".to_string(), "file:write".to_string());

    let result = middleware.before_execution(operation, &context).await;

    // Validation: Should be denied (permission mismatch: write vs read)
    assert!(
        result.is_err(),
        "Permission string manipulation should be blocked"
    );
}

// ================================================================================================
// Test 7: Multi-Policy Conflict Exploitation
// ================================================================================================

#[tokio::test]
async fn threat_multi_policy_conflict_exploitation() {
    // Scenario: Attacker hoping one policy allows when another denies
    // Expected: ANY deny blocks (deny-wins semantics)

    // Setup: ACL allows, RBAC denies
    let acl = AccessControlList::new().add_entry(AclEntry::new(
        "user".to_string(),
        "/data/*".to_string(),
        vec!["file:read".to_string()],
        AclPolicy::Allow,
    ));

    let mut rbac = RoleBasedAccessControl::new();
    rbac = rbac.add_role(Role::new("user_role".to_string(), "User Role".to_string()));
    rbac = rbac.assign_roles("user".to_string(), vec!["user_role".to_string()]);

    let middleware = SecurityMiddlewareBuilder::new()
        .with_config(SecurityConfig::default())
        .add_policy(Box::new(acl))
        .add_policy(Box::new(rbac))
        .build()
        .expect("Failed to build middleware");

    // Attack: Hoping ACL allow overrides RBAC deny
    let operation = FileReadOperation::new("/data/file.txt".to_string());
    let mut context = ExecutionContext::new(SecurityContext::new("user".to_string()));
    context
        .security_context
        .attributes
        .insert("acl.resource".to_string(), "/data/file.txt".to_string());
    context
        .security_context
        .attributes
        .insert("acl.permission".to_string(), "file:read".to_string());
    context.security_context.attributes.insert(
        "rbac.required_permission".to_string(),
        "file:read".to_string(),
    );

    let result = middleware.before_execution(operation, &context).await;

    // Validation: Should be denied (RBAC denies due to missing permission, ANY deny blocks)
    assert!(
        result.is_err(),
        "Any deny should block even if one policy allows"
    );
}

// ================================================================================================
// Test 8: Circular Role Dependency Exploitation
// ================================================================================================

#[tokio::test]
async fn threat_circular_role_dependency_dos() {
    // Scenario: Attacker creates circular role dependencies to cause DoS
    // Expected: Circular dependency detected and denied

    let mut rbac = RoleBasedAccessControl::new();

    // Create circular dependency: role_a -> role_c -> role_b -> role_a
    rbac = rbac.add_role(
        Role::new("role_a".to_string(), "Role A".to_string()).inherits_from("role_c".to_string()),
    );
    rbac = rbac.add_role(
        Role::new("role_b".to_string(), "Role B".to_string()).inherits_from("role_a".to_string()),
    );
    rbac = rbac.add_role(
        Role::new("role_c".to_string(), "Role C".to_string()).inherits_from("role_b".to_string()),
    );

    rbac = rbac.assign_roles("user".to_string(), vec!["role_a".to_string()]);

    let middleware = SecurityMiddlewareBuilder::new()
        .with_config(SecurityConfig::default())
        .add_policy(Box::new(rbac))
        .build()
        .expect("Failed to build middleware");

    // Attack: Triggering circular dependency resolution
    let operation = ProcessSpawnOperation::new("/bin/test".to_string());
    let mut context = ExecutionContext::new(SecurityContext::new("user".to_string()));
    context
        .security_context
        .attributes
        .insert("rbac.required_permission".to_string(), "perm_a".to_string());

    let result = middleware.before_execution(operation, &context).await;

    // Validation: Should be denied (circular dependency detected)
    assert!(
        result.is_err(),
        "Circular role dependency should be detected and denied"
    );
}

// ================================================================================================
// Test 9: Default Policy Bypass Attempt
// ================================================================================================

#[tokio::test]
async fn threat_default_policy_bypass() {
    // Scenario: No policies configured, attacker hoping for allow-by-default
    // Expected: Deny-by-default enforcement

    let middleware = SecurityMiddlewareBuilder::new()
        .with_config(SecurityConfig::default())
        .build()
        .expect("Failed to build middleware");

    // Attack: Attempting operation with no policies
    let operation = FileReadOperation::new("/etc/passwd".to_string());
    let mut context = ExecutionContext::new(SecurityContext::new("attacker".to_string()));
    context
        .security_context
        .attributes
        .insert("acl.resource".to_string(), "/etc/passwd".to_string());
    context
        .security_context
        .attributes
        .insert("acl.permission".to_string(), "file:read".to_string());

    let result = middleware.before_execution(operation, &context).await;

    // Validation: Should be denied (deny-by-default)
    assert!(
        result.is_err(),
        "No policies should result in deny-by-default"
    );
}

// ================================================================================================
// Test 10: Network Socket Type Confusion
// ================================================================================================

#[tokio::test]
async fn threat_network_socket_type_confusion() {
    // Scenario: Attacker trying to use network operations without proper authorization
    // Expected: Network operations denied without proper policy

    let acl = AccessControlList::new().add_entry(AclEntry::new(
        "web_service".to_string(),
        "127.0.0.1:8080".to_string(),
        vec!["network:listen".to_string()],
        AclPolicy::Allow,
    ));

    let middleware = SecurityMiddlewareBuilder::new()
        .with_config(SecurityConfig::default())
        .add_policy(Box::new(acl))
        .build()
        .expect("Failed to build middleware");

    // Attack: Different user trying to bind to allowed port
    let operation = NetworkListenOperation::new("127.0.0.1:8080".to_string());
    let mut context = ExecutionContext::new(SecurityContext::new("attacker".to_string()));
    context
        .security_context
        .attributes
        .insert("acl.resource".to_string(), "127.0.0.1:8080".to_string());
    context
        .security_context
        .attributes
        .insert("acl.permission".to_string(), "network:listen".to_string());

    let result = middleware.before_execution(operation, &context).await;

    // Validation: Should be denied (identity mismatch: attacker vs web_service)
    assert!(
        result.is_err(),
        "Network operation should be blocked for wrong identity"
    );
}

// ================================================================================================
// Test 11: Process Spawning Privilege Escalation
// ================================================================================================

#[tokio::test]
async fn threat_process_spawn_privilege_escalation() {
    // Scenario: Regular user attempting to spawn privileged process
    // Expected: Deny based on RBAC permission requirements

    let mut rbac = RoleBasedAccessControl::new();
    rbac = rbac.add_role(Role::new("admin".to_string(), "Administrator".to_string()));
    rbac = rbac.add_role(Role::new("user".to_string(), "Regular User".to_string()));
    rbac = rbac.assign_roles("regular_user".to_string(), vec!["user".to_string()]);

    let middleware = SecurityMiddlewareBuilder::new()
        .with_config(SecurityConfig::default())
        .add_policy(Box::new(rbac))
        .build()
        .expect("Failed to build middleware");

    // Attack: Regular user trying to spawn privileged process
    let operation = ProcessSpawnOperation::new("/usr/bin/sudo".to_string());
    let mut context = ExecutionContext::new(SecurityContext::new("regular_user".to_string()));
    context.security_context.attributes.insert(
        "rbac.required_permission".to_string(),
        "process:spawn:privileged".to_string(),
    );

    let result = middleware.before_execution(operation, &context).await;

    // Validation: Should be denied (user doesn't have privileged permission)
    assert!(
        result.is_err(),
        "Privileged process spawn should be blocked for regular user"
    );
}

// ================================================================================================
// Test 12: ACL Default Policy Override Attempt
// ================================================================================================

#[tokio::test]
async fn threat_acl_default_policy_override() {
    // Scenario: Attacker hoping default Allow policy overrides explicit Deny
    // Expected: Explicit Deny always wins

    let acl = AccessControlList::new()
        .with_default_policy(AclPolicy::Allow)
        .add_entry(AclEntry::new(
            "user".to_string(),
            "/secret/*".to_string(),
            vec!["*".to_string()],
            AclPolicy::Deny, // Explicit deny
        ));
    let middleware = SecurityMiddlewareBuilder::new()
        .with_config(SecurityConfig::default())
        .add_policy(Box::new(acl))
        .build()
        .expect("Failed to build middleware");

    // Attack: Trying to access explicitly denied resource
    let operation = FileReadOperation::new("/secret/data.txt".to_string());
    let mut context = ExecutionContext::new(SecurityContext::new("user".to_string()));
    context
        .security_context
        .attributes
        .insert("acl.resource".to_string(), "/secret/data.txt".to_string());
    context
        .security_context
        .attributes
        .insert("acl.permission".to_string(), "file:read".to_string());

    let result = middleware.before_execution(operation, &context).await;

    // Validation: Should be denied (explicit deny wins over default allow)
    assert!(
        result.is_err(),
        "Explicit deny should override default allow policy"
    );
}

// ================================================================================================
// Test 13: Permission Wildcard Confusion
// ================================================================================================

#[tokio::test]
async fn threat_permission_wildcard_confusion() {
    // Scenario: Attacker using "*" permission expecting all access
    // Expected: Wildcard must be explicitly configured in ACL entry

    let acl = AccessControlList::new().add_entry(AclEntry::new(
        "user".to_string(),
        "/data/*".to_string(),
        vec!["file:read".to_string()], // Specific permission, not wildcard
        AclPolicy::Allow,
    ));

    let middleware = SecurityMiddlewareBuilder::new()
        .with_config(SecurityConfig::default())
        .add_policy(Box::new(acl))
        .build()
        .expect("Failed to build middleware");

    // Attack: Requesting with wildcard permission
    let operation =
        FileWriteOperation::new("/data/file.txt".to_string(), "data".as_bytes().to_vec());
    let mut context = ExecutionContext::new(SecurityContext::new("user".to_string()));
    context
        .security_context
        .attributes
        .insert("acl.resource".to_string(), "/data/file.txt".to_string());
    context
        .security_context
        .attributes
        .insert("acl.permission".to_string(), "*".to_string());

    let result = middleware.before_execution(operation, &context).await;

    // Validation: Should be denied (ACL entry has specific "file:read", not "*")
    assert!(
        result.is_err(),
        "Wildcard permission should not match specific permission in ACL"
    );
}
