//! Role-Based Access Control (RBAC) implementation.
//!
//! This module provides RBAC-based security policy implementation with
//! role hierarchies and permission management.
//!
//! # Features
//!
//! - **Role Hierarchies**: Roles can inherit permissions from other roles
//! - **Permission Resolution**: Automatic resolution of permissions through role inheritance
//! - **Circular Dependency Detection**: Prevents infinite loops in role hierarchies
//! - **Context Attributes**: Extracts required permission from `SecurityContext` attributes
//!
//! # Context Attribute Keys
//!
//! RBAC evaluation uses these standardized context attribute keys:
//! - [`ATTR_RBAC_REQUIRED_PERMISSION`]: The permission being requested (e.g., "read_file", "write_file")
//!
//! # Examples
//!
//! ```rust
//! use airssys_osl::middleware::security::rbac::{
//!     RoleBasedAccessControl, Role, Permission, ATTR_RBAC_REQUIRED_PERMISSION
//! };
//! use airssys_osl::middleware::security::policy::SecurityPolicy;
//! use airssys_osl::core::context::SecurityContext;
//! use std::collections::HashMap;
//! use uuid::Uuid;
//! use chrono::Utc;
//!
//! // Define permissions
//! let read_perm = Permission::new(
//!     "read_file".to_string(),
//!     "Read File".to_string(),
//!     "Allows reading files".to_string(),
//! );
//!
//! // Define roles
//! let user_role = Role::new("user".to_string(), "User".to_string())
//!     .with_permission("read_file".to_string());
//!
//! // Build RBAC system
//! let rbac = RoleBasedAccessControl::new()
//!     .add_permission(read_perm)
//!     .add_role(user_role)
//!     .assign_roles("alice".to_string(), vec!["user".to_string()]);
//!
//! // Create security context with required permission
//! let mut attributes = HashMap::new();
//! attributes.insert(ATTR_RBAC_REQUIRED_PERMISSION.to_string(), "read_file".to_string());
//!
//! let context = SecurityContext {
//!     principal: "alice".to_string(),
//!     session_id: Uuid::new_v4(),
//!     established_at: Utc::now(),
//!     attributes,
//! };
//!
//! // Evaluate - should allow access
//! let decision = rbac.evaluate(&context);
//! ```

// Layer 1: Standard library imports
use std::collections::{HashMap, HashSet};
use std::fmt::Debug;

// Layer 2: Third-party crate imports
use serde::{Deserialize, Serialize};

// Layer 3: Internal module imports
use crate::core::context::SecurityContext;
use crate::middleware::security::policy::{PolicyDecision, PolicyScope, SecurityPolicy};

/// Context attribute key for required permission.
///
/// Uses `rbac.` prefix to prevent conflicts with other security modules.
/// Used to extract the required permission from `SecurityContext.attributes`.
///
/// # Example
/// ```
/// use std::collections::HashMap;
/// use airssys_osl::middleware::security::rbac::ATTR_RBAC_REQUIRED_PERMISSION;
///
/// let mut attributes = HashMap::new();
/// attributes.insert(ATTR_RBAC_REQUIRED_PERMISSION.to_string(), "read_file".to_string());
/// ```
pub const ATTR_RBAC_REQUIRED_PERMISSION: &str = "rbac.required_permission";

/// Builds RBAC attributes from operation permissions.
///
/// This function maps operation permissions to RBAC permission names,
/// returning attributes suitable for populating a `SecurityContext`.
///
/// # Permission Mapping
///
/// The function maps `Permission` enum variants to RBAC permission names:
/// - `FilesystemRead(_)` → `"file:read"`
/// - `FilesystemWrite(_)` → `"file:write"`
/// - `FilesystemExecute(_)` → `"file:execute"`
/// - `ProcessSpawn` → `"process:spawn"`
/// - `ProcessManage` → `"process:manage"`
/// - `NetworkSocket` → `"network:socket"`
/// - `NetworkConnect(_)` → `"network:connect"`
/// - `UtilityExecute(_)` → `"utility:execute"`
///
/// # Arguments
///
/// * `permissions` - Slice of `Permission` enum values from an operation
///
/// # Returns
///
/// HashMap containing RBAC attributes:
/// - Key: [`ATTR_RBAC_REQUIRED_PERMISSION`] = `"rbac.required_permission"`
/// - Value: The RBAC permission name (e.g., `"file:read"`)
///
/// # Priority
///
/// When multiple permissions are provided, the **first permission** is used.
/// This follows the architectural decision (ADR-030) that operations should
/// declare their primary permission first.
///
/// # Example
///
/// ```
/// use airssys_osl::core::operation::Permission;
/// use airssys_osl::middleware::security::rbac::{build_rbac_attributes, ATTR_RBAC_REQUIRED_PERMISSION};
///
/// let permissions = vec![Permission::FilesystemRead("/etc/passwd".to_string())];
/// let attrs = build_rbac_attributes(&permissions);
///
/// assert_eq!(attrs.get(ATTR_RBAC_REQUIRED_PERMISSION), Some(&"file:read".to_string()));
/// ```
pub fn build_rbac_attributes(
    permissions: &[crate::core::operation::Permission],
) -> std::collections::HashMap<String, String> {
    use crate::core::operation::Permission;
    let mut attributes = std::collections::HashMap::new();

    if let Some(first_permission) = permissions.first() {
        let permission_name = match first_permission {
            Permission::FilesystemRead(_) => "file:read",
            Permission::FilesystemWrite(_) => "file:write",
            Permission::FilesystemExecute(_) => "file:execute",
            Permission::ProcessSpawn => "process:spawn",
            Permission::ProcessManage => "process:manage",
            Permission::NetworkSocket => "network:socket",
            Permission::NetworkConnect(_) => "network:connect",
            Permission::UtilityExecute(_) => "utility:execute",
        };

        attributes.insert(
            ATTR_RBAC_REQUIRED_PERMISSION.to_string(),
            permission_name.to_string(),
        );
    }

    attributes
}

/// Type alias for user identifiers.
pub type UserId = String;

/// Type alias for role identifiers.
pub type RoleId = String;

/// Type alias for permission identifiers.
pub type PermissionId = String;

/// Permission definition for RBAC.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Permission {
    /// Unique identifier for this permission
    pub id: PermissionId,

    /// Human-readable permission name
    pub name: String,

    /// Description of what this permission grants
    pub description: String,
}

impl Permission {
    /// Create a new permission.
    pub fn new(id: PermissionId, name: String, description: String) -> Self {
        Self {
            id,
            name,
            description,
        }
    }
}

/// Role definition for RBAC.
///
/// Roles group permissions together and can inherit from other roles
/// to create role hierarchies.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Role {
    /// Unique identifier for this role
    pub id: RoleId,

    /// Human-readable role name
    pub name: String,

    /// Permissions granted by this role
    pub permissions: Vec<PermissionId>,

    /// Roles this role inherits from
    pub inherits_from: Vec<RoleId>,
}

impl Role {
    /// Create a new role.
    pub fn new(id: RoleId, name: String) -> Self {
        Self {
            id,
            name,
            permissions: Vec::new(),
            inherits_from: Vec::new(),
        }
    }

    /// Add a permission to this role.
    pub fn with_permission(mut self, permission_id: PermissionId) -> Self {
        self.permissions.push(permission_id);
        self
    }

    /// Add multiple permissions to this role.
    pub fn with_permissions(mut self, permission_ids: Vec<PermissionId>) -> Self {
        self.permissions.extend(permission_ids);
        self
    }

    /// Make this role inherit from another role.
    pub fn inherits_from(mut self, role_id: RoleId) -> Self {
        self.inherits_from.push(role_id);
        self
    }
}

/// Role-Based Access Control security policy.
///
/// Implements RBAC with support for role hierarchies and permission
/// inheritance. Users are assigned roles, and roles grant permissions.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RoleBasedAccessControl {
    /// Role definitions
    roles: HashMap<RoleId, Role>,

    /// User to role assignments
    role_assignments: HashMap<UserId, Vec<RoleId>>,

    /// Permission definitions
    permissions: HashMap<PermissionId, Permission>,
}

impl RoleBasedAccessControl {
    /// Create a new empty RBAC system.
    pub fn new() -> Self {
        Self {
            roles: HashMap::new(),
            role_assignments: HashMap::new(),
            permissions: HashMap::new(),
        }
    }

    /// Add a role definition.
    pub fn add_role(mut self, role: Role) -> Self {
        self.roles.insert(role.id.clone(), role);
        self
    }

    /// Add a permission definition.
    pub fn add_permission(mut self, permission: Permission) -> Self {
        self.permissions.insert(permission.id.clone(), permission);
        self
    }

    /// Assign roles to a user.
    pub fn assign_roles(mut self, user_id: UserId, role_ids: Vec<RoleId>) -> Self {
        self.role_assignments.insert(user_id, role_ids);
        self
    }

    /// Get all roles assigned to a user.
    pub fn get_user_roles(&self, user_id: &str) -> Vec<&RoleId> {
        self.role_assignments
            .get(user_id)
            .map(|roles| roles.iter().collect())
            .unwrap_or_default()
    }

    /// Resolve all effective roles for a user including inherited roles.
    ///
    /// Uses depth-first traversal to collect all roles including inherited ones.
    /// Detects circular dependencies and returns an error if found.
    ///
    /// # Arguments
    ///
    /// * `role_ids` - Direct role IDs assigned to the user
    ///
    /// # Returns
    ///
    /// * `Ok(HashSet<RoleId>)` - All effective role IDs (direct + inherited)
    /// * `Err(String)` - Error message if circular dependency detected
    fn resolve_roles(&self, role_ids: &[&RoleId]) -> Result<HashSet<RoleId>, String> {
        let mut effective_roles = HashSet::new();
        let mut visited = HashSet::new();
        let mut stack = Vec::new();

        for &role_id in role_ids {
            self.resolve_role_recursive(role_id, &mut effective_roles, &mut visited, &mut stack)?;
        }

        Ok(effective_roles)
    }

    /// Recursive helper for role resolution with circular dependency detection.
    fn resolve_role_recursive(
        &self,
        role_id: &RoleId,
        effective_roles: &mut HashSet<RoleId>,
        visited: &mut HashSet<RoleId>,
        stack: &mut Vec<RoleId>,
    ) -> Result<(), String> {
        // Check for circular dependency
        if stack.contains(role_id) {
            return Err(format!(
                "Circular role dependency detected: {}",
                stack.join(" -> ")
            ));
        }

        // Skip if already visited (diamond dependency is OK)
        if visited.contains(role_id) {
            return Ok(());
        }

        // Mark as visited and add to stack
        visited.insert(role_id.clone());
        stack.push(role_id.clone());

        // Add this role to effective roles
        effective_roles.insert(role_id.clone());

        // Recursively resolve inherited roles
        if let Some(role) = self.roles.get(role_id) {
            for inherited_role_id in &role.inherits_from {
                self.resolve_role_recursive(inherited_role_id, effective_roles, visited, stack)?;
            }
        }

        // Remove from stack (backtrack)
        stack.pop();

        Ok(())
    }

    /// Collect all permissions from a set of roles.
    ///
    /// Resolves permission IDs to actual permissions and returns them.
    fn collect_permissions(&self, role_ids: &HashSet<RoleId>) -> HashSet<PermissionId> {
        let mut permissions = HashSet::new();

        for role_id in role_ids {
            if let Some(role) = self.roles.get(role_id) {
                for perm_id in &role.permissions {
                    permissions.insert(perm_id.clone());
                }
            }
        }

        permissions
    }
}

impl Default for RoleBasedAccessControl {
    fn default() -> Self {
        Self::new()
    }
}

impl SecurityPolicy for RoleBasedAccessControl {
    fn evaluate(&self, context: &SecurityContext) -> PolicyDecision {
        let principal = &context.principal;

        // 1. Get user's directly assigned roles
        let user_roles = self.get_user_roles(principal);

        if user_roles.is_empty() {
            return PolicyDecision::Deny(format!("No roles assigned to user '{principal}'"));
        }

        // 2. Resolve role hierarchy (include inherited roles)
        let effective_roles = match self.resolve_roles(&user_roles) {
            Ok(roles) => roles,
            Err(err) => {
                return PolicyDecision::Deny(format!(
                    "Role hierarchy error for user '{principal}': {err}"
                ))
            }
        };

        // 3. Extract required permission from context attributes
        let required_permission = context
            .attributes
            .get(ATTR_RBAC_REQUIRED_PERMISSION)
            .map(|s| s.as_str());

        // If no permission is required, allow (user has valid roles)
        let Some(required_perm) = required_permission else {
            return PolicyDecision::Allow;
        };

        // 4. Collect all permissions from effective roles
        let user_permissions = self.collect_permissions(&effective_roles);

        // 5. Check if user has the required permission
        if user_permissions.contains(required_perm) {
            PolicyDecision::Allow
        } else {
            PolicyDecision::Deny(format!(
                "User '{principal}' with roles {effective_roles:?} does not have required permission '{required_perm}'"
            ))
        }
    }

    fn description(&self) -> &str {
        "Role-Based Access Control (RBAC) Policy"
    }

    fn scope(&self) -> PolicyScope {
        PolicyScope::All
    }
}

#[cfg(test)]
mod tests {
    #![allow(clippy::panic)]
    #![allow(clippy::unwrap_used)]
    #![allow(clippy::uninlined_format_args)]

    use super::*;

    #[test]
    fn test_permission_creation() {
        let perm = Permission::new(
            "read_file".to_string(),
            "Read File".to_string(),
            "Allows reading files".to_string(),
        );
        assert_eq!(perm.id, "read_file");
        assert_eq!(perm.name, "Read File");
    }

    #[test]
    fn test_role_creation() {
        let role = Role::new("admin".to_string(), "Administrator".to_string())
            .with_permission("read_file".to_string())
            .with_permission("write_file".to_string());

        assert_eq!(role.id, "admin");
        assert_eq!(role.permissions.len(), 2);
    }

    #[test]
    fn test_role_inheritance() {
        let role = Role::new("superadmin".to_string(), "Super Administrator".to_string())
            .inherits_from("admin".to_string());

        assert_eq!(role.inherits_from.len(), 1);
        assert_eq!(role.inherits_from[0], "admin");
    }

    #[test]
    fn test_rbac_creation() {
        let rbac = RoleBasedAccessControl::new();
        assert_eq!(rbac.roles.len(), 0);
        assert_eq!(rbac.permissions.len(), 0);
    }

    #[test]
    fn test_rbac_add_role() {
        let role = Role::new("admin".to_string(), "Administrator".to_string());
        let rbac = RoleBasedAccessControl::new().add_role(role);
        assert_eq!(rbac.roles.len(), 1);
    }

    #[test]
    fn test_rbac_assign_roles() {
        let rbac = RoleBasedAccessControl::new()
            .assign_roles("user1".to_string(), vec!["admin".to_string()]);

        let user_roles = rbac.get_user_roles("user1");
        assert_eq!(user_roles.len(), 1);
        assert_eq!(user_roles[0], "admin");
    }

    // ========== COMPREHENSIVE RBAC EVALUATION TESTS ==========

    #[test]
    fn test_rbac_evaluate_user_with_no_roles() {
        use chrono::Utc;
        use std::collections::HashMap;
        use uuid::Uuid;

        let rbac = RoleBasedAccessControl::new();

        let context = SecurityContext {
            principal: "alice".to_string(),
            session_id: Uuid::new_v4(),
            established_at: Utc::now(),
            attributes: HashMap::new(),
        };

        match rbac.evaluate(&context) {
            PolicyDecision::Deny(reason) => {
                assert!(reason.contains("No roles assigned"));
            }
            other => panic!("Expected Deny, got {:?}", other),
        }
    }

    #[test]
    fn test_rbac_evaluate_single_role_with_permission_allow() {
        use chrono::Utc;
        use std::collections::HashMap;
        use uuid::Uuid;

        // Setup RBAC with role and permission
        let perm = Permission::new(
            "read_file".to_string(),
            "Read File".to_string(),
            "Allows reading files".to_string(),
        );

        let role = Role::new("user".to_string(), "User".to_string())
            .with_permission("read_file".to_string());

        let rbac = RoleBasedAccessControl::new()
            .add_permission(perm)
            .add_role(role)
            .assign_roles("alice".to_string(), vec!["user".to_string()]);

        // Create context with required permission
        let mut attributes = HashMap::new();
        attributes.insert(
            ATTR_RBAC_REQUIRED_PERMISSION.to_string(),
            "read_file".to_string(),
        );

        let context = SecurityContext {
            principal: "alice".to_string(),
            session_id: Uuid::new_v4(),
            established_at: Utc::now(),
            attributes,
        };

        match rbac.evaluate(&context) {
            PolicyDecision::Allow => {
                // Success
            }
            other => panic!("Expected Allow, got {:?}", other),
        }
    }

    #[test]
    fn test_rbac_evaluate_single_role_missing_permission() {
        use chrono::Utc;
        use std::collections::HashMap;
        use uuid::Uuid;

        // Setup RBAC with role that doesn't have the required permission
        let role = Role::new("user".to_string(), "User".to_string())
            .with_permission("read_file".to_string());

        let rbac = RoleBasedAccessControl::new()
            .add_role(role)
            .assign_roles("alice".to_string(), vec!["user".to_string()]);

        // Request write permission (not granted)
        let mut attributes = HashMap::new();
        attributes.insert(
            ATTR_RBAC_REQUIRED_PERMISSION.to_string(),
            "write_file".to_string(),
        );

        let context = SecurityContext {
            principal: "alice".to_string(),
            session_id: Uuid::new_v4(),
            established_at: Utc::now(),
            attributes,
        };

        match rbac.evaluate(&context) {
            PolicyDecision::Deny(reason) => {
                assert!(reason.contains("does not have required permission"));
            }
            other => panic!("Expected Deny, got {:?}", other),
        }
    }

    #[test]
    fn test_rbac_evaluate_multiple_roles_no_inheritance() {
        use chrono::Utc;
        use std::collections::HashMap;
        use uuid::Uuid;

        // Setup RBAC with two roles
        let role1 = Role::new("reader".to_string(), "Reader".to_string())
            .with_permission("read_file".to_string());

        let role2 = Role::new("writer".to_string(), "Writer".to_string())
            .with_permission("write_file".to_string());

        let rbac = RoleBasedAccessControl::new()
            .add_role(role1)
            .add_role(role2)
            .assign_roles(
                "alice".to_string(),
                vec!["reader".to_string(), "writer".to_string()],
            );

        // Request write permission (granted by writer role)
        let mut attributes = HashMap::new();
        attributes.insert(
            ATTR_RBAC_REQUIRED_PERMISSION.to_string(),
            "write_file".to_string(),
        );

        let context = SecurityContext {
            principal: "alice".to_string(),
            session_id: Uuid::new_v4(),
            established_at: Utc::now(),
            attributes,
        };

        match rbac.evaluate(&context) {
            PolicyDecision::Allow => {
                // Success - write permission granted by writer role
            }
            other => panic!("Expected Allow, got {:?}", other),
        }
    }

    #[test]
    fn test_rbac_evaluate_role_inheritance_one_level() {
        use chrono::Utc;
        use std::collections::HashMap;
        use uuid::Uuid;

        // Setup role hierarchy: admin inherits from user
        let user_role = Role::new("user".to_string(), "User".to_string())
            .with_permission("read_file".to_string());

        let admin_role = Role::new("admin".to_string(), "Administrator".to_string())
            .with_permission("write_file".to_string())
            .inherits_from("user".to_string());

        let rbac = RoleBasedAccessControl::new()
            .add_role(user_role)
            .add_role(admin_role)
            .assign_roles("alice".to_string(), vec!["admin".to_string()]);

        // Request read permission (inherited from user role)
        let mut attributes = HashMap::new();
        attributes.insert(
            ATTR_RBAC_REQUIRED_PERMISSION.to_string(),
            "read_file".to_string(),
        );

        let context = SecurityContext {
            principal: "alice".to_string(),
            session_id: Uuid::new_v4(),
            established_at: Utc::now(),
            attributes,
        };

        match rbac.evaluate(&context) {
            PolicyDecision::Allow => {
                // Success - read permission inherited from user role
            }
            other => panic!("Expected Allow (inherited permission), got {:?}", other),
        }
    }

    #[test]
    fn test_rbac_evaluate_role_inheritance_multiple_levels() {
        use chrono::Utc;
        use std::collections::HashMap;
        use uuid::Uuid;

        // Setup 3-level hierarchy: superadmin -> admin -> user
        let user_role = Role::new("user".to_string(), "User".to_string())
            .with_permission("read_file".to_string());

        let admin_role = Role::new("admin".to_string(), "Administrator".to_string())
            .with_permission("write_file".to_string())
            .inherits_from("user".to_string());

        let superadmin_role =
            Role::new("superadmin".to_string(), "Super Administrator".to_string())
                .with_permission("delete_file".to_string())
                .inherits_from("admin".to_string());

        let rbac = RoleBasedAccessControl::new()
            .add_role(user_role)
            .add_role(admin_role)
            .add_role(superadmin_role)
            .assign_roles("alice".to_string(), vec!["superadmin".to_string()]);

        // Request read permission (inherited through admin from user)
        let mut attributes = HashMap::new();
        attributes.insert(
            ATTR_RBAC_REQUIRED_PERMISSION.to_string(),
            "read_file".to_string(),
        );

        let context = SecurityContext {
            principal: "alice".to_string(),
            session_id: Uuid::new_v4(),
            established_at: Utc::now(),
            attributes,
        };

        match rbac.evaluate(&context) {
            PolicyDecision::Allow => {
                // Success - read permission inherited through 2 levels
            }
            other => panic!(
                "Expected Allow (multi-level inherited permission), got {:?}",
                other
            ),
        }
    }

    #[test]
    fn test_rbac_evaluate_circular_dependency_detection() {
        use chrono::Utc;
        use std::collections::HashMap;
        use uuid::Uuid;

        // Setup circular dependency: role1 -> role2 -> role1
        let role1 = Role::new("role1".to_string(), "Role 1".to_string())
            .with_permission("perm1".to_string())
            .inherits_from("role2".to_string());

        let role2 = Role::new("role2".to_string(), "Role 2".to_string())
            .with_permission("perm2".to_string())
            .inherits_from("role1".to_string());

        let rbac = RoleBasedAccessControl::new()
            .add_role(role1)
            .add_role(role2)
            .assign_roles("alice".to_string(), vec!["role1".to_string()]);

        let context = SecurityContext {
            principal: "alice".to_string(),
            session_id: Uuid::new_v4(),
            established_at: Utc::now(),
            attributes: HashMap::new(),
        };

        match rbac.evaluate(&context) {
            PolicyDecision::Deny(reason) => {
                assert!(reason.contains("Circular role dependency detected"));
            }
            other => panic!("Expected Deny (circular dependency), got {:?}", other),
        }
    }

    #[test]
    fn test_rbac_evaluate_diamond_dependency() {
        use chrono::Utc;
        use std::collections::HashMap;
        use uuid::Uuid;

        // Setup diamond dependency (OK, not circular):
        //     top
        //    /   \
        //  left  right
        //    \   /
        //    bottom
        let top_role =
            Role::new("top".to_string(), "Top".to_string()).with_permission("top_perm".to_string());

        let left_role = Role::new("left".to_string(), "Left".to_string())
            .with_permission("left_perm".to_string())
            .inherits_from("top".to_string());

        let right_role = Role::new("right".to_string(), "Right".to_string())
            .with_permission("right_perm".to_string())
            .inherits_from("top".to_string());

        let bottom_role = Role::new("bottom".to_string(), "Bottom".to_string())
            .with_permission("bottom_perm".to_string())
            .inherits_from("left".to_string())
            .inherits_from("right".to_string());

        let rbac = RoleBasedAccessControl::new()
            .add_role(top_role)
            .add_role(left_role)
            .add_role(right_role)
            .add_role(bottom_role)
            .assign_roles("alice".to_string(), vec!["bottom".to_string()]);

        // Request top permission (should be available through both paths)
        let mut attributes = HashMap::new();
        attributes.insert(
            ATTR_RBAC_REQUIRED_PERMISSION.to_string(),
            "top_perm".to_string(),
        );

        let context = SecurityContext {
            principal: "alice".to_string(),
            session_id: Uuid::new_v4(),
            established_at: Utc::now(),
            attributes,
        };

        match rbac.evaluate(&context) {
            PolicyDecision::Allow => {
                // Success - diamond dependency handled correctly
            }
            other => panic!("Expected Allow (diamond dependency OK), got {:?}", other),
        }
    }

    #[test]
    fn test_rbac_evaluate_no_permission_required() {
        use chrono::Utc;
        use std::collections::HashMap;
        use uuid::Uuid;

        // Setup RBAC with role but no permission in context
        let role = Role::new("user".to_string(), "User".to_string())
            .with_permission("read_file".to_string());

        let rbac = RoleBasedAccessControl::new()
            .add_role(role)
            .assign_roles("alice".to_string(), vec!["user".to_string()]);

        // No permission attribute - should allow if user has valid roles
        let context = SecurityContext {
            principal: "alice".to_string(),
            session_id: Uuid::new_v4(),
            established_at: Utc::now(),
            attributes: HashMap::new(),
        };

        match rbac.evaluate(&context) {
            PolicyDecision::Allow => {
                // Success - user has valid roles, no permission check needed
            }
            other => panic!("Expected Allow (no permission required), got {:?}", other),
        }
    }

    #[test]
    fn test_rbac_evaluate_empty_rbac_system() {
        use chrono::Utc;
        use std::collections::HashMap;
        use uuid::Uuid;

        // Empty RBAC system - no roles or permissions defined
        let rbac = RoleBasedAccessControl::new();

        let context = SecurityContext {
            principal: "alice".to_string(),
            session_id: Uuid::new_v4(),
            established_at: Utc::now(),
            attributes: HashMap::new(),
        };

        match rbac.evaluate(&context) {
            PolicyDecision::Deny(reason) => {
                assert!(reason.contains("No roles assigned"));
            }
            other => panic!("Expected Deny (empty RBAC), got {:?}", other),
        }
    }
}
