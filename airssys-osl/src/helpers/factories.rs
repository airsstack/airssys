//! Middleware factory functions for helper functions.
//!
//! This module provides default security middleware configurations for
//! development and testing. Production deployments should configure
//! custom policies.

// Layer 1: Standard library imports
use std::sync::Arc;

// Layer 2: No third-party imports needed

// Layer 3: Internal module imports
use crate::core::security::SecurityConfig;
use crate::middleware::security::acl::{AccessControlList, AclEntry, AclPolicy};
use crate::middleware::security::audit::ConsoleSecurityAuditLogger;
use crate::middleware::security::rbac::RoleBasedAccessControl;
use crate::middleware::security::{SecurityMiddleware, SecurityMiddlewareBuilder};

// ============================================================================
// Middleware Factory Functions
// ============================================================================

/// Default security middleware for helper functions.
///
/// Provides a deny-by-default security model with:
/// - ACL policy enforcement (glob pattern matching)
/// - RBAC policy enforcement (role-based permissions)
/// - Security audit logging (console output)
///
/// # Security Model
///
/// **Deny-by-default:** Operations are denied unless explicitly allowed by ALL policies.
///
/// **Policy Evaluation:** Both ACL and RBAC must allow the operation:
/// - ACL checks resource patterns (e.g., `/etc/hosts`)
/// - RBAC checks user roles and permissions (e.g., `file:read`)
/// - If ANY policy denies, the operation is denied
///
/// **Default Policies:**
/// - Admin user (`"admin"`) has full access via both ACL and RBAC
/// - ACL: Admin can access all resources (`*`)
/// - RBAC: Admin has all permissions (filesystem, process, network)
/// - All operations are logged to console for audit trail
///
/// # Production Use
///
/// **⚠️ WARNING:** The default policies are permissive for development convenience.
/// **For production deployments**, you MUST configure your own ACL/RBAC policies
/// using the `*_with_middleware` variants.
///
/// # Example
///
/// ```rust,no_run
/// use airssys_osl::helpers::*;
///
/// # async fn example() -> airssys_osl::core::result::OSResult<()> {
/// // Uses default_security_middleware() internally
/// let data = read_file("/etc/hosts", "admin").await?;
/// # Ok(())
/// # }
/// ```
///
/// # Custom Policies
///
/// For custom security policies, use the `*_with_middleware` variants:
///
/// ```rust,no_run
/// use airssys_osl::helpers::*;
/// use airssys_osl::middleware::security::*;
///
/// # async fn example() -> airssys_osl::core::result::OSResult<()> {
/// let acl = AccessControlList::new()
///     .add_entry(AclEntry::new(
///         "alice".to_string(),
///         "/data/*".to_string(),
///         vec!["read".to_string()],
///         AclPolicy::Allow
///     ));
///
/// let security = SecurityMiddlewareBuilder::new()
///     .add_policy(Box::new(acl))
///     .build();
///
/// // Phase 2-4: Implementation pending
/// // let data = read_file_with_middleware("/data/file.txt", "alice", security).await?;
/// # Ok(())
/// # }
/// ```
///
/// # See Also
///
/// - [`default_acl_policy()`]: Default ACL policy configuration
/// - [`default_rbac_policy()`]: Default RBAC policy configuration
/// - [`SecurityMiddleware`]: Security middleware implementation
pub(crate) fn default_security_middleware() -> SecurityMiddleware {
    SecurityMiddlewareBuilder::new()
        .with_config(SecurityConfig::default())
        .with_audit_logger(Arc::new(ConsoleSecurityAuditLogger::new()))
        .add_policy(Box::new(default_acl_policy()))
        .add_policy(Box::new(default_rbac_policy()))
        .build()
        .expect("Failed to build default security middleware")
}

/// Default ACL policy for development/testing.
///
/// **⚠️ Development Only:** This policy is permissive for convenience.
/// Configure your own policies for production use.
///
/// # Default Rules
///
/// - Admin user has full access to all resources (`*`)
/// - All permissions granted to admin (`*`)
///
/// # Security Implications
///
/// This policy allows the `admin` user to perform ANY operation on ANY resource.
/// In a production environment, you should:
/// - Define specific resource patterns (e.g., `/app/data/*` instead of `*`)
/// - Limit permissions to required operations (e.g., `["read", "write"]` instead of `["*"]`)
/// - Use explicit deny rules for sensitive resources
/// - Implement least-privilege access control
///
/// # Production Configuration
///
/// ```rust
/// use airssys_osl::middleware::security::*;
///
/// // Example: Application-specific ACL policy
/// let production_acl = AccessControlList::new()
///     // Allow app user to read/write application data
///     .add_entry(AclEntry::new(
///         "app_user".to_string(),
///         "/app/data/*".to_string(),
///         vec!["read".to_string(), "write".to_string()],
///         AclPolicy::Allow
///     ))
///     // Deny app user from modifying configuration
///     .add_entry(AclEntry::new(
///         "app_user".to_string(),
///         "/app/config/*".to_string(),
///         vec!["write".to_string()],
///         AclPolicy::Deny
///     ))
///     // Allow admin full access to application directory
///     .add_entry(AclEntry::new(
///         "admin".to_string(),
///         "/app/*".to_string(),
///         vec!["*".to_string()],
///         AclPolicy::Allow
///     ))
///     // Deny everyone from accessing system files
///     .add_entry(AclEntry::new(
///         "*".to_string(),
///         "/etc/*".to_string(),
///         vec!["*".to_string()],
///         AclPolicy::Deny
///     ))
///     .add_entry(AclEntry::new(
///         "*".to_string(),
///         "/sys/*".to_string(),
///         vec!["*".to_string()],
///         AclPolicy::Deny
///     ));
/// ```
///
/// # Pattern Matching
///
/// ACL entries support glob pattern matching:
/// - `*` matches any sequence of characters
/// - `?` matches a single character
/// - `**` matches multiple directory levels
///
/// Examples:
/// - `/app/data/*` matches `/app/data/file.txt` but not `/app/data/sub/file.txt`
/// - `/app/data/**` matches all files under `/app/data/` including subdirectories
/// - `/tmp/*.log` matches log files in `/tmp/` directory
///
/// # See Also
///
/// - [`AccessControlList`]: ACL policy configuration
/// - [`AclEntry`]: Individual ACL entry
/// - [`default_security_middleware()`]: Factory using this policy
pub(crate) fn default_acl_policy() -> AccessControlList {
    AccessControlList::new().add_entry(AclEntry::new(
        "admin".to_string(),
        "*".to_string(),
        vec!["*".to_string()],
        AclPolicy::Allow,
    ))
}

/// Default RBAC policy for development/testing.
///
/// **⚠️ Development Only:** This policy grants admin user full access.
/// Configure your own roles for production use.
///
/// # Default Configuration
///
/// Creates an RBAC policy with an "admin" role that has full permissions:
/// - Admin user has all filesystem, process, and network permissions
/// - Suitable for development and testing convenience
///
/// # Security Implications
///
/// This policy grants the `admin` user full access to all operations.
/// In a production environment, you should:
/// - Define roles based on actual job functions (e.g., `reader`, `writer`, `operator`)
/// - Assign minimum required permissions to each role
/// - Implement role hierarchies for delegation
/// - Assign users to appropriate roles based on their needs
///
/// # Production Configuration
///
/// ```rust
/// use airssys_osl::middleware::security::rbac::*;
///
/// // Example: Multi-role RBAC policy
/// let production_rbac = RoleBasedAccessControl::new()
///     // Define permissions
///     .add_permission(Permission::new(
///         "file:read".to_string(),
///         "Read files".to_string(),
///         "Allow reading file contents".to_string()
///     ))
///     .add_permission(Permission::new(
///         "file:write".to_string(),
///         "Write files".to_string(),
///         "Allow writing file contents".to_string()
///     ))
///     
///     // Define roles
///     .add_role(
///         Role::new("reader".to_string(), "Reader Role".to_string())
///             .with_permission("file:read".to_string())
///     )
///     .add_role(
///         Role::new("writer".to_string(), "Writer Role".to_string())
///             .with_permission("file:read".to_string())
///             .with_permission("file:write".to_string())
///     )
///     
///     // Assign users to roles
///     .assign_roles("alice".to_string(), vec!["reader".to_string()])
///     .assign_roles("bob".to_string(), vec!["writer".to_string()]);
/// ```
///
/// # Permission Format
///
/// Permissions follow the format `category:action`:
/// - **Filesystem**: `file:read`, `file:write`, `file:delete`, `file:create`
/// - **Process**: `process:spawn`, `process:kill`, `process:signal`
/// - **Network**: `network:connect`, `network:listen`, `network:socket`
///
/// # See Also
///
/// - [`RoleBasedAccessControl`]: RBAC policy configuration
/// - [`default_security_middleware()`]: Factory using this policy
pub(crate) fn default_rbac_policy() -> RoleBasedAccessControl {
    use crate::middleware::security::rbac::{Permission, Role};

    RoleBasedAccessControl::new()
        // Define admin permission (full access)
        .add_permission(Permission::new(
            "admin:all".to_string(),
            "Full administrative access".to_string(),
            "Grants all permissions for filesystem, process, and network operations".to_string(),
        ))
        // Define filesystem permissions
        .add_permission(Permission::new(
            "file:read".to_string(),
            "Read files".to_string(),
            "Allow reading file contents".to_string(),
        ))
        .add_permission(Permission::new(
            "file:write".to_string(),
            "Write files".to_string(),
            "Allow writing file contents".to_string(),
        ))
        .add_permission(Permission::new(
            "file:delete".to_string(),
            "Delete files".to_string(),
            "Allow deleting files".to_string(),
        ))
        .add_permission(Permission::new(
            "file:create".to_string(),
            "Create files/directories".to_string(),
            "Allow creating files and directories".to_string(),
        ))
        // Define process permissions
        .add_permission(Permission::new(
            "process:spawn".to_string(),
            "Spawn processes".to_string(),
            "Allow spawning new processes".to_string(),
        ))
        .add_permission(Permission::new(
            "process:kill".to_string(),
            "Kill processes".to_string(),
            "Allow killing processes".to_string(),
        ))
        .add_permission(Permission::new(
            "process:signal".to_string(),
            "Send signals".to_string(),
            "Allow sending signals to processes".to_string(),
        ))
        // Define network permissions
        .add_permission(Permission::new(
            "network:connect".to_string(),
            "Network connect".to_string(),
            "Allow network connections".to_string(),
        ))
        .add_permission(Permission::new(
            "network:listen".to_string(),
            "Network listen".to_string(),
            "Allow network listening".to_string(),
        ))
        .add_permission(Permission::new(
            "network:socket".to_string(),
            "Create sockets".to_string(),
            "Allow socket creation".to_string(),
        ))
        // Define admin role with all permissions
        .add_role(
            Role::new("admin".to_string(), "Administrator Role".to_string())
                .with_permission("admin:all".to_string())
                .with_permission("file:read".to_string())
                .with_permission("file:write".to_string())
                .with_permission("file:delete".to_string())
                .with_permission("file:create".to_string())
                .with_permission("process:spawn".to_string())
                .with_permission("process:kill".to_string())
                .with_permission("process:signal".to_string())
                .with_permission("network:connect".to_string())
                .with_permission("network:listen".to_string())
                .with_permission("network:socket".to_string()),
        )
        // Assign admin user to admin role
        .assign_roles("admin".to_string(), vec!["admin".to_string()])
}
