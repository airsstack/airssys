//! Security policy framework and evaluation engine.
//!
//! This module provides the core security policy abstraction and policy
//! evaluation framework for the security middleware.

// Layer 1: Standard library imports
use std::fmt::Debug;

// Layer 2: Third-party crate imports
// (none yet - will add as needed)

// Layer 3: Internal module imports
use crate::core::context::SecurityContext;
use crate::core::operation::Operation;

/// Security policy evaluation result.
///
/// Represents the decision made by a security policy when evaluating
/// whether an operation should be allowed.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum PolicyDecision {
    /// Operation is allowed to proceed
    Allow,

    /// Operation is denied with a reason
    Deny(String),

    /// Operation requires additional authentication
    RequireAdditionalAuth(AuthRequirement),
}

/// Additional authentication requirement for operations.
///
/// Specifies what additional authentication is required before
/// an operation can proceed.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum AuthRequirement {
    /// Require multi-factor authentication
    MultiFactorAuth,

    /// Require elevated privileges
    ElevatedPrivileges,

    /// Require specific role
    SpecificRole(String),

    /// Custom authentication requirement
    Custom(String),
}

/// Scope of a security policy.
///
/// Defines which operation types a policy applies to.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PolicyScope {
    /// Policy applies to filesystem operations
    Filesystem,

    /// Policy applies to process operations
    Process,

    /// Policy applies to network operations
    Network,

    /// Policy applies to all operation types
    All,
}

/// Core trait for security policy implementations.
///
/// Security policies evaluate operations against security rules and return
/// a decision on whether the operation should be allowed, denied, or require
/// additional authentication.
///
/// # Generic Parameters
///
/// * `O` - The operation type this policy can evaluate
///
/// # Design Principles
///
/// - **Generic-first design** (§6.2): Uses generic constraints for type safety
/// - **Deny-by-default**: Policies should deny unless explicitly allowed
/// - **Stateless evaluation**: Policy evaluation should be deterministic
/// - **Clear decisions**: Return explicit Allow/Deny with reasons
///
/// # Example Implementation
///
/// ```rust
/// use airssys_osl::middleware::security::policy::{
///     SecurityPolicy, PolicyDecision, PolicyScope
/// };
/// use airssys_osl::core::{context::SecurityContext, operation::Operation};
///
/// #[derive(Debug)]
/// struct AlwaysAllowPolicy;
///
/// impl<O: Operation> SecurityPolicy<O> for AlwaysAllowPolicy {
///     fn evaluate(&self, _operation: &O, _context: &SecurityContext) -> PolicyDecision {
///         PolicyDecision::Allow
///     }
///
///     fn description(&self) -> &str {
///         "Always allow policy (for testing only)"
///     }
///
///     fn scope(&self) -> PolicyScope {
///         PolicyScope::All
///     }
/// }
/// ```
///
/// # Thread Safety
///
/// Implementations must be thread-safe (Send + Sync) as policies may be
/// evaluated concurrently from multiple threads.
pub trait SecurityPolicy<O>: Debug + Send + Sync + 'static
where
    O: Operation,
{
    /// Evaluate if the operation is permitted.
    ///
    /// # Arguments
    ///
    /// * `operation` - The operation to evaluate
    /// * `context` - The security context of the requester
    ///
    /// # Returns
    ///
    /// Returns a `PolicyDecision` indicating whether the operation should
    /// be allowed, denied, or requires additional authentication.
    fn evaluate(&self, operation: &O, context: &SecurityContext) -> PolicyDecision;

    /// Get human-readable policy description.
    ///
    /// This description is used in audit logs and error messages to explain
    /// which policy made a security decision.
    fn description(&self) -> &str;

    /// Get the scope of this policy.
    ///
    /// Defines which operation types this policy applies to. Policies with
    /// more specific scopes may be evaluated before broader policies.
    fn scope(&self) -> PolicyScope;
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_policy_decision_equality() {
        assert_eq!(PolicyDecision::Allow, PolicyDecision::Allow);
        assert_eq!(
            PolicyDecision::Deny("test".to_string()),
            PolicyDecision::Deny("test".to_string())
        );
        assert_ne!(
            PolicyDecision::Allow,
            PolicyDecision::Deny("test".to_string())
        );
    }

    #[test]
    fn test_auth_requirement_equality() {
        assert_eq!(
            AuthRequirement::MultiFactorAuth,
            AuthRequirement::MultiFactorAuth
        );
        assert_eq!(
            AuthRequirement::SpecificRole("admin".to_string()),
            AuthRequirement::SpecificRole("admin".to_string())
        );
    }

    #[test]
    fn test_policy_scope_equality() {
        assert_eq!(PolicyScope::Filesystem, PolicyScope::Filesystem);
        assert_eq!(PolicyScope::All, PolicyScope::All);
        assert_ne!(PolicyScope::Filesystem, PolicyScope::Process);
    }
}
