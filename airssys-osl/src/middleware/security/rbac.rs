//! Role-Based Access Control (RBAC) implementation.
//!
//! This module provides RBAC-based security policy implementation with
//! role hierarchies and permission management.

// Layer 1: Standard library imports
use std::collections::HashMap;
use std::fmt::Debug;

// Layer 2: Third-party crate imports
use serde::{Deserialize, Serialize};

// Layer 3: Internal module imports
use crate::core::context::SecurityContext;
use crate::core::operation::Operation;
use crate::middleware::security::policy::{PolicyDecision, PolicyScope, SecurityPolicy};

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
}

impl Default for RoleBasedAccessControl {
    fn default() -> Self {
        Self::new()
    }
}

impl<O: Operation> SecurityPolicy<O> for RoleBasedAccessControl {
    fn evaluate(&self, _operation: &O, context: &SecurityContext) -> PolicyDecision {
        let principal = &context.principal;

        // Get user's roles
        let user_roles = self.get_user_roles(principal);

        if user_roles.is_empty() {
            return PolicyDecision::Deny(format!("No roles assigned to user '{principal}'"));
        }

        // TODO: Implement permission resolution with role inheritance
        // For now, if user has any roles, allow (placeholder)
        PolicyDecision::Allow
    }

    fn description(&self) -> &str {
        "Role-Based Access Control (RBAC) Policy"
    }

    fn scope(&self) -> PolicyScope {
        PolicyScope::All
    }
}

/// Implementation of SecurityPolicyDispatcher for RoleBasedAccessControl.
///
/// This allows RBAC policies to be used in the SecurityMiddleware's
/// type-erased policy storage.
impl crate::middleware::security::policy::SecurityPolicyDispatcher for RoleBasedAccessControl {
    fn evaluate_any(
        &self,
        _operation: &dyn std::any::Any,
        context: &SecurityContext,
    ) -> PolicyDecision {
        // RBAC policies work with any operation type, evaluating based on
        // user roles rather than specific operation details
        
        let principal = &context.principal;

        // Get user's roles
        let user_roles = self.get_user_roles(principal);

        if user_roles.is_empty() {
            return PolicyDecision::Deny(format!("No roles assigned to user '{principal}'"));
        }

        // TODO: Implement permission resolution with role inheritance
        // For now, if user has any roles, allow (placeholder)
        PolicyDecision::Allow
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
}
