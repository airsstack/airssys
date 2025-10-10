//! Comprehensive Security Middleware Example
//!
//! This example demonstrates the complete security middleware implementation including:
//! - SecurityMiddleware setup and configuration
//! - ACL (Access Control Lists) for resource-based access control
//! - RBAC (Role-Based Access Control) with role hierarchy
//! - Multiple policies working together (deny-wins semantics)
//! - Security audit logging
//! - Middleware pipeline integration
//!
//! Run with: cargo run --example security_middleware_comprehensive

use airssys_osl::core::context::{ExecutionContext, SecurityContext};
use airssys_osl::core::middleware::Middleware;
use airssys_osl::core::security::SecurityConfig;
use airssys_osl::middleware::security::acl::{AccessControlList, AclEntry, AclPolicy};
use airssys_osl::middleware::security::middleware::SecurityMiddlewareBuilder;
use airssys_osl::middleware::security::rbac::{Role, RoleBasedAccessControl};
use airssys_osl::operations::filesystem::read::FileReadOperation;
use airssys_osl::operations::filesystem::write::FileWriteOperation;
use airssys_osl::operations::process::spawn::ProcessSpawnOperation;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("=== Comprehensive Security Middleware Example ===\n");

    // Example 1: Basic ACL Setup
    println!("Example 1: Basic ACL for File Access Control");
    example_basic_acl().await?;
    println!();

    // Example 2: RBAC with Role Hierarchy
    println!("Example 2: RBAC with Role Hierarchy");
    example_rbac_hierarchy().await?;
    println!();

    // Example 3: Combined ACL + RBAC (Deny-Wins)
    println!("Example 3: Combined ACL + RBAC Policies");
    example_combined_policies().await?;
    println!();

    // Example 4: Security Audit Logging
    println!("Example 4: Security Audit Logging");
    example_audit_logging().await?;
    println!();

    // Example 5: Real-World Scenario - Multi-User File System
    println!("Example 5: Real-World Multi-User File System");
    example_real_world_scenario().await?;
    println!();

    println!("=== All Examples Completed Successfully ===");
    Ok(())
}

/// Example 1: Basic ACL for file access control
async fn example_basic_acl() -> Result<(), Box<dyn std::error::Error>> {
    // Configure ACL with file access rules
    let acl = AccessControlList::new()
        .add_entry(AclEntry::new(
            "alice".to_string(),
            "/home/alice/*".to_string(),
            vec!["file:read".to_string(), "file:write".to_string()],
            AclPolicy::Allow,
        ))
        .add_entry(AclEntry::new(
            "bob".to_string(),
            "/home/bob/*".to_string(),
            vec!["file:read".to_string()],
            AclPolicy::Allow,
        ))
        .add_entry(AclEntry::new(
            "*".to_string(), // All users
            "/etc/*".to_string(),
            vec!["*".to_string()], // All permissions
            AclPolicy::Deny,       // Explicit deny for /etc
        ));

    let middleware = SecurityMiddlewareBuilder::new()
        .with_config(SecurityConfig::default())
        .add_policy(Box::new(acl))
        .build()?;

    // Test 1: Alice can read her own files
    let operation = FileReadOperation::new("/home/alice/document.txt".to_string());
    let mut context = ExecutionContext::new(SecurityContext::new("alice".to_string()));
    context.security_context.attributes.insert(
        "resource".to_string(),
        "/home/alice/document.txt".to_string(),
    );
    context
        .security_context
        .attributes
        .insert("permission".to_string(), "file:read".to_string());

    match middleware.before_execution(operation, &context).await {
        Ok(_) => println!("  ✓ Alice can read /home/alice/document.txt"),
        Err(e) => println!("  ✗ Alice denied: {e:?}"),
    }

    // Test 2: Bob cannot write to his files (only read permission)
    let operation =
        FileWriteOperation::new("/home/bob/data.txt".to_string(), b"test data".to_vec());
    context.security_context.principal = "bob".to_string();
    context
        .security_context
        .attributes
        .insert("resource".to_string(), "/home/bob/data.txt".to_string());
    context
        .security_context
        .attributes
        .insert("permission".to_string(), "file:write".to_string());

    match middleware.before_execution(operation, &context).await {
        Ok(_) => println!("  ✗ Bob should NOT be able to write"),
        Err(_) => println!("  ✓ Bob correctly denied write to /home/bob/data.txt"),
    }

    // Test 3: No one can access /etc (explicit deny)
    let operation = FileReadOperation::new("/etc/passwd".to_string());
    context.security_context.principal = "alice".to_string();
    context
        .security_context
        .attributes
        .insert("resource".to_string(), "/etc/passwd".to_string());
    context
        .security_context
        .attributes
        .insert("permission".to_string(), "file:read".to_string());

    match middleware.before_execution(operation, &context).await {
        Ok(_) => println!("  ✗ Should NOT access /etc/passwd"),
        Err(_) => println!("  ✓ /etc/passwd correctly protected (explicit deny)"),
    }

    Ok(())
}

/// Example 2: RBAC with role hierarchy and inheritance
async fn example_rbac_hierarchy() -> Result<(), Box<dyn std::error::Error>> {
    // Configure RBAC with role hierarchy
    let mut rbac = RoleBasedAccessControl::new();

    // Define roles with inheritance
    rbac = rbac.add_role(
        Role::new("user".to_string(), "Regular User".to_string())
            .with_permission("file:read".to_string())
            .with_permission("file:write".to_string()),
    );

    rbac = rbac.add_role(
        Role::new("operator".to_string(), "System Operator".to_string())
            .with_permission("process:spawn".to_string())
            .with_permission("process:kill".to_string())
            .inherits_from("user".to_string()), // Inherits user permissions
    );

    rbac = rbac.add_role(
        Role::new("admin".to_string(), "Administrator".to_string())
            .with_permission("system:*".to_string()) // All system permissions
            .inherits_from("operator".to_string()), // Inherits operator + user permissions
    );

    // Assign roles to users
    rbac = rbac.assign_roles("alice".to_string(), vec!["user".to_string()]);
    rbac = rbac.assign_roles("bob".to_string(), vec!["operator".to_string()]);
    rbac = rbac.assign_roles("charlie".to_string(), vec!["admin".to_string()]);

    let middleware = SecurityMiddlewareBuilder::new()
        .with_config(SecurityConfig::default())
        .add_policy(Box::new(rbac))
        .build()?;

    // Test 1: Alice (user) can spawn processes? No
    let operation = ProcessSpawnOperation::new("/bin/echo".to_string());
    let mut context = ExecutionContext::new(SecurityContext::new("alice".to_string()));
    context.security_context.attributes.insert(
        "required_permission".to_string(),
        "process:spawn".to_string(),
    );

    match middleware.before_execution(operation, &context).await {
        Ok(_) => println!("  ✗ Alice should NOT spawn processes"),
        Err(_) => println!("  ✓ Alice (user) correctly denied process:spawn"),
    }

    // Test 2: Bob (operator) can spawn processes via inheritance
    let operation = ProcessSpawnOperation::new("/bin/echo".to_string());
    context.security_context.principal = "bob".to_string();

    match middleware.before_execution(operation, &context).await {
        Ok(_) => println!("  ✓ Bob (operator) can spawn processes"),
        Err(e) => println!("  ✗ Bob denied: {e:?}"),
    }

    // Test 3: Charlie (admin) has all permissions via inheritance chain
    let operation = ProcessSpawnOperation::new("/usr/bin/systemctl".to_string());
    context.security_context.principal = "charlie".to_string();
    context.security_context.attributes.insert(
        "required_permission".to_string(),
        "system:admin".to_string(),
    );

    match middleware.before_execution(operation, &context).await {
        Ok(_) => println!("  ✓ Charlie (admin) has system:* permissions"),
        Err(e) => println!("  ✗ Charlie denied: {e:?}"),
    }

    Ok(())
}

/// Example 3: Combined ACL + RBAC with deny-wins semantics
async fn example_combined_policies() -> Result<(), Box<dyn std::error::Error>> {
    // Setup ACL with explicit deny
    let acl = AccessControlList::new().add_entry(AclEntry::new(
        "alice".to_string(),
        "/sensitive/*".to_string(),
        vec!["*".to_string()],
        AclPolicy::Deny, // Explicit deny
    ));

    // Setup RBAC that would allow
    let mut rbac = RoleBasedAccessControl::new();
    rbac = rbac.add_role(
        Role::new("admin".to_string(), "Administrator".to_string())
            .with_permission("file:read".to_string())
            .with_permission("file:write".to_string()),
    );
    rbac = rbac.assign_roles("alice".to_string(), vec!["admin".to_string()]);

    // Build with BOTH policies
    let middleware = SecurityMiddlewareBuilder::new()
        .with_config(SecurityConfig::default())
        .add_policy(Box::new(acl)) // Evaluated first
        .add_policy(Box::new(rbac)) // Evaluated second
        .build()?;

    // Test: Even though Alice has admin role (RBAC allows), ACL denies
    let operation = FileReadOperation::new("/sensitive/secret.txt".to_string());
    let mut context = ExecutionContext::new(SecurityContext::new("alice".to_string()));
    context
        .security_context
        .attributes
        .insert("resource".to_string(), "/sensitive/secret.txt".to_string());
    context
        .security_context
        .attributes
        .insert("permission".to_string(), "file:read".to_string());
    context
        .security_context
        .attributes
        .insert("required_permission".to_string(), "file:read".to_string());

    match middleware.before_execution(operation, &context).await {
        Ok(_) => println!("  ✗ Should be denied (ACL deny wins)"),
        Err(_) => println!("  ✓ Deny-wins: ACL deny overrides RBAC allow"),
    }

    println!("  → Even admin role cannot override explicit ACL deny");
    println!("  → This demonstrates defense-in-depth security");

    Ok(())
}

/// Example 4: Security audit logging demonstration
async fn example_audit_logging() -> Result<(), Box<dyn std::error::Error>> {
    let acl = AccessControlList::new().add_entry(AclEntry::new(
        "alice".to_string(),
        "/logs/*".to_string(),
        vec!["file:read".to_string()],
        AclPolicy::Allow,
    ));

    let middleware = SecurityMiddlewareBuilder::new()
        .with_config(SecurityConfig::default())
        .add_policy(Box::new(acl))
        .build()?;

    println!("  → Security events are automatically logged:");
    println!("  → Format: JSON with timestamp, principal, resource, decision");
    println!("  → Fields: event_type, permission, reason, policies_evaluated");

    // Test some operations (they will be logged)
    let operation = FileReadOperation::new("/logs/app.log".to_string());
    let mut context = ExecutionContext::new(SecurityContext::new("alice".to_string()));
    context
        .security_context
        .attributes
        .insert("resource".to_string(), "/logs/app.log".to_string());
    context
        .security_context
        .attributes
        .insert("permission".to_string(), "file:read".to_string());

    match middleware.before_execution(operation, &context).await {
        Ok(_) => println!("  ✓ Access granted - logged as AccessGranted"),
        Err(_) => println!("  ✗ Access denied - logged as AccessDenied"),
    }

    // Denied access (also logged)
    let operation = FileReadOperation::new("/logs/secret.log".to_string());
    context
        .security_context
        .attributes
        .insert("resource".to_string(), "/logs/secret.log".to_string());
    context.security_context.principal = "bob".to_string();

    match middleware.before_execution(operation, &context).await {
        Ok(_) => println!("  ✗ Should be denied"),
        Err(_) => println!("  ✓ Access denied - logged as AccessDenied"),
    }

    println!("\n  Sample audit log entry:");
    println!(
        r#"  {{
    "timestamp": "2025-10-10T10:30:45.123456Z",
    "event_type": "AccessDenied",
    "principal": "bob",
    "resource": "/logs/secret.log",
    "permission": "file:read",
    "decision": "Deny",
    "reason": "No matching ACL entry",
    "policies_evaluated": ["ACL"]
  }}"#
    );

    // Show how to query logs
    println!("\n  → Query logs: logger.query_events(start_time, event_type)");
    println!("  → Use for: compliance, forensics, anomaly detection");

    Ok(())
}

/// Example 5: Real-world multi-user file system scenario
async fn example_real_world_scenario() -> Result<(), Box<dyn std::error::Error>> {
    println!("  Scenario: Multi-tenant file storage system");
    println!("  Users: alice (tenant1), bob (tenant2), admin (system)");
    println!();

    // Configure comprehensive security policies
    let acl = AccessControlList::new()
        // Tenant isolation
        .add_entry(AclEntry::new(
            "alice".to_string(),
            "/data/tenant1/*".to_string(),
            vec!["file:read".to_string(), "file:write".to_string()],
            AclPolicy::Allow,
        ))
        .add_entry(AclEntry::new(
            "bob".to_string(),
            "/data/tenant2/*".to_string(),
            vec!["file:read".to_string(), "file:write".to_string()],
            AclPolicy::Allow,
        ))
        // Shared read-only data
        .add_entry(AclEntry::new(
            "*".to_string(),
            "/data/shared/*".to_string(),
            vec!["file:read".to_string()],
            AclPolicy::Allow,
        ))
        // System files protected
        .add_entry(AclEntry::new(
            "*".to_string(),
            "/data/system/*".to_string(),
            vec!["*".to_string()],
            AclPolicy::Deny,
        ));

    let mut rbac = RoleBasedAccessControl::new();
    rbac = rbac.add_role(
        Role::new("admin".to_string(), "System Admin".to_string())
            .with_permission("system:*".to_string()),
    );
    rbac = rbac.assign_roles("admin".to_string(), vec!["admin".to_string()]);

    let middleware = SecurityMiddlewareBuilder::new()
        .with_config(SecurityConfig::default())
        .add_policy(Box::new(acl))
        .add_policy(Box::new(rbac))
        .build()?;

    // Test scenarios
    println!("  Test 1: Tenant isolation");
    let operation = FileReadOperation::new("/data/tenant2/secret.txt".to_string());
    let mut context = ExecutionContext::new(SecurityContext::new("alice".to_string()));
    context.security_context.attributes.insert(
        "resource".to_string(),
        "/data/tenant2/secret.txt".to_string(),
    );
    context
        .security_context
        .attributes
        .insert("permission".to_string(), "file:read".to_string());

    match middleware.before_execution(operation, &context).await {
        Ok(_) => println!("    ✗ Tenant isolation violated!"),
        Err(_) => println!("    ✓ Alice cannot access tenant2 data (isolation works)"),
    }

    println!("\n  Test 2: Shared data access");
    let operation = FileReadOperation::new("/data/shared/readme.txt".to_string());
    context.security_context.attributes.insert(
        "resource".to_string(),
        "/data/shared/readme.txt".to_string(),
    );

    match middleware.before_execution(operation, &context).await {
        Ok(_) => println!("    ✓ Alice can read shared data"),
        Err(e) => println!("    ✗ Shared access failed: {e:?}"),
    }

    println!("\n  Test 3: System protection");
    let operation = FileReadOperation::new("/data/system/config.json".to_string());
    context.security_context.principal = "admin".to_string();
    context.security_context.attributes.insert(
        "resource".to_string(),
        "/data/system/config.json".to_string(),
    );
    context.security_context.attributes.insert(
        "required_permission".to_string(),
        "system:admin".to_string(),
    );

    match middleware.before_execution(operation, &context).await {
        Ok(_) => println!("    ✗ System files should be protected"),
        Err(_) => println!("    ✓ System files protected even from admin (explicit deny wins)"),
    }

    println!("\n  → Complete security enforcement:");
    println!("    • Tenant isolation via ACL");
    println!("    • Shared resources with controlled permissions");
    println!("    • System protection with deny-wins semantics");
    println!("    • Full audit trail for compliance");

    Ok(())
}
