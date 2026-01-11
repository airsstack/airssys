//! Policy evaluation engine.
//!
//! This module provides [`PolicyEngine`] for evaluating security policies
//! across multiple components and policies.
//!
//! # Examples
//!
//! Creating and using a policy engine:
//!
//! ```
//! use airssys_wasm::security::policy::engine::PolicyEngine;
//! use airssys_wasm::security::policy::rules::{SecurityPolicy, PolicyRule, PolicyEffect};
//! use airssys_wasm::core::component::id::ComponentId;
//!
//! let mut engine = PolicyEngine::new();
//!
//! let mut policy = SecurityPolicy::new("storage-policy", "comp-*");
//! policy.add_rule(PolicyRule {
//!     action: "write".to_string(),
//!     resource_pattern: "/sensitive/*".to_string(),
//!     effect: PolicyEffect::Deny,
//! });
//!
//! engine.add_policy(policy);
//!
//! let component = ComponentId::new("comp-123".to_string(), "comp", "1");
//! let result = engine.evaluate(&component, "write", "/sensitive/file");
//! ```

// Layer 1: Standard library imports (per PROJECTS_STANDARD.md §2.1)
use std::fmt;

// Layer 2: Third-party crate imports
// None required

// Layer 3: Internal module imports
use crate::core::component::id::ComponentId;
use crate::core::security::errors::SecurityError;

use super::rules::SecurityPolicy;

/// Policy evaluation engine for managing and evaluating security policies.
///
/// The engine maintains a collection of policies and evaluates them
/// against component actions, enforcing security constraints.
pub struct PolicyEngine {
    /// Registered security policies
    policies: Vec<SecurityPolicy>,
}

impl PolicyEngine {
    /// Creates a new policy engine with no policies registered.
    ///
    /// # Examples
    ///
    /// ```
    /// use airssys_wasm::security::policy::engine::PolicyEngine;
    ///
    /// let engine = PolicyEngine::new();
    /// ```
    pub fn new() -> Self {
        Self {
            policies: Vec::new(),
        }
    }

    /// Adds a security policy to the engine.
    ///
    /// Policies are evaluated in the order they were added.
    ///
    /// # Arguments
    ///
    /// * `policy` - SecurityPolicy to add to the engine
    pub fn add_policy(&mut self, policy: SecurityPolicy) {
        self.policies.push(policy);
    }

    /// Evaluates an action on a resource against all applicable policies.
    ///
    /// Checks all policies that apply to the component, returning an error
    /// if any policy denies the action.
    ///
    /// # Arguments
    ///
    /// * `component` - ComponentId of the component performing the action
    /// * `action` - Action to evaluate (e.g., "read", "write")
    /// * `resource` - Resource being accessed (e.g., "/path/to/file")
    ///
    /// # Returns
    ///
    /// `Ok(())` if the action is allowed by all policies,
    /// `Err(SecurityError::PolicyViolation)` if any policy denies the action
    ///
    /// # Errors
    ///
    /// Returns `SecurityError::PolicyViolation` if a Deny rule matches.
    ///
    /// # Examples
    ///
    /// ```
    /// use airssys_wasm::security::policy::engine::PolicyEngine;
    /// use airssys_wasm::security::policy::rules::{SecurityPolicy, PolicyRule, PolicyEffect};
    /// use airssys_wasm::core::component::id::ComponentId;
    ///
    /// let mut engine = PolicyEngine::new();
    ///
    /// let mut policy = SecurityPolicy::new("test-policy", "comp-*");
    /// policy.add_rule(PolicyRule {
    ///     action: "write".to_string(),
    ///     resource_pattern: "/secret/*".to_string(),
    ///     effect: PolicyEffect::Deny,
    /// });
    ///
    /// engine.add_policy(policy);
    ///
    /// let component = ComponentId::new("comp-123".to_string(), "comp", "1");
    /// let result = engine.evaluate(&component, "write", "/secret/file");
    /// ```
    pub fn evaluate(
        &self,
        component: &ComponentId,
        action: &str,
        resource: &str,
    ) -> Result<(), SecurityError> {
        for policy in &self.policies {
            if policy.applies_to(component) {
                policy.evaluate(action, resource)?;
            }
        }
        Ok(())
    }
}

impl Default for PolicyEngine {
    /// Creates a new PolicyEngine with default configuration.
    fn default() -> Self {
        Self::new()
    }
}

impl fmt::Debug for PolicyEngine {
    /// Formats the PolicyEngine for debug output.
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("PolicyEngine")
            .field("policy_count", &self.policies.len())
            .finish()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::security::policy::rules::{PolicyEffect, PolicyRule, SecurityPolicy};

    fn create_test_component(id: &str) -> ComponentId {
        // Create component with format "{id}/" to match expected test patterns
        ComponentId::new(id, "", "")
    }

    #[test]
    fn test_policy_engine_new() {
        let engine = PolicyEngine::new();
        assert!(engine.policies.is_empty());
    }

    #[test]
    fn test_policy_engine_default() {
        let engine = PolicyEngine::default();
        assert!(engine.policies.is_empty());
    }

    #[test]
    fn test_policy_engine_add_policy() {
        let mut engine = PolicyEngine::new();
        let policy = SecurityPolicy::new("test-policy", "comp-*");

        engine.add_policy(policy);
        assert_eq!(engine.policies.len(), 1);
    }

    #[test]
    fn test_evaluate_no_policies() {
        let engine = PolicyEngine::new();
        let component = create_test_component("comp-123");

        let result = engine.evaluate(&component, "read", "/data/file");
        assert!(result.is_ok());
    }

    #[test]
    fn test_evaluate_single_policy_matching() {
        let mut engine = PolicyEngine::new();

        let mut policy = SecurityPolicy::new("test-policy", "comp-*");
        policy.add_rule(PolicyRule {
            action: "read".to_string(),
            resource_pattern: "/public/*".to_string(),
            effect: PolicyEffect::Allow,
        });

        engine.add_policy(policy);

        let component = create_test_component("comp-123");
        let result = engine.evaluate(&component, "read", "/public/file");
        assert!(result.is_ok());
    }

    #[test]
    fn test_evaluate_single_policy_non_matching() {
        let mut engine = PolicyEngine::new();

        let mut policy = SecurityPolicy::new("test-policy", "storage-*");
        policy.add_rule(PolicyRule {
            action: "read".to_string(),
            resource_pattern: "/public/*".to_string(),
            effect: PolicyEffect::Allow,
        });

        engine.add_policy(policy);

        let component = create_test_component("comp-123"); // Doesn't match "storage-*"
        let result = engine.evaluate(&component, "read", "/public/file");
        assert!(result.is_ok());
    }

    #[test]
    fn test_evaluate_multiple_policies_all_allow() {
        let mut engine = PolicyEngine::new();

        let mut policy1 = SecurityPolicy::new("policy-1", "comp-*");
        policy1.add_rule(PolicyRule {
            action: "read".to_string(),
            resource_pattern: "/data/*".to_string(),
            effect: PolicyEffect::Allow,
        });

        let mut policy2 = SecurityPolicy::new("policy-2", "comp-*");
        policy2.add_rule(PolicyRule {
            action: "write".to_string(),
            resource_pattern: "/data/*".to_string(),
            effect: PolicyEffect::Allow,
        });

        engine.add_policy(policy1);
        engine.add_policy(policy2);

        let component = create_test_component("comp-123");
        let result = engine.evaluate(&component, "read", "/data/file");
        assert!(result.is_ok());
    }

    #[test]
    fn test_evaluate_multiple_policies_one_deny() {
        let mut engine = PolicyEngine::new();

        let mut policy1 = SecurityPolicy::new("allow-policy", "comp-*");
        policy1.add_rule(PolicyRule {
            action: "write".to_string(),
            resource_pattern: "/data/*".to_string(),
            effect: PolicyEffect::Allow,
        });

        let mut policy2 = SecurityPolicy::new("deny-policy", "comp-*");
        policy2.add_rule(PolicyRule {
            action: "write".to_string(),
            resource_pattern: "/data/secret/*".to_string(),
            effect: PolicyEffect::Deny,
        });

        engine.add_policy(policy1);
        engine.add_policy(policy2);

        let component = create_test_component("comp-123");
        let result = engine.evaluate(&component, "write", "/data/secret/file");
        assert!(result.is_err());

        if let Err(SecurityError::PolicyViolation(msg)) = result {
            assert!(msg.contains("deny-policy"));
        } else {
            panic!("Expected PolicyViolation error");
        }
    }

    #[test]
    fn test_evaluate_multiple_policies_component_specific() {
        let mut engine = PolicyEngine::new();

        let mut policy1 = SecurityPolicy::new("storage-policy", "storage-*");
        policy1.add_rule(PolicyRule {
            action: "delete".to_string(),
            resource_pattern: "/*".to_string(),
            effect: PolicyEffect::Deny,
        });

        let mut policy2 = SecurityPolicy::new("compute-policy", "compute-*");
        policy2.add_rule(PolicyRule {
            action: "execute".to_string(),
            resource_pattern: "/*".to_string(),
            effect: PolicyEffect::Deny,
        });

        engine.add_policy(policy1);
        engine.add_policy(policy2);

        let storage_component = create_test_component("storage-123");
        let compute_component = create_test_component("compute-456");

        // Storage component should be denied delete
        let storage_result = engine.evaluate(&storage_component, "delete", "/any/file");
        assert!(storage_result.is_err());

        // Compute component should not be denied delete (policy doesn't apply)
        let compute_result = engine.evaluate(&compute_component, "delete", "/any/file");
        assert!(compute_result.is_ok());
    }

    #[test]
    fn test_evaluate_component_pattern_wildcard() {
        let mut engine = PolicyEngine::new();

        let mut policy = SecurityPolicy::new("global-policy", "*");
        policy.add_rule(PolicyRule {
            action: "admin".to_string(),
            resource_pattern: "*".to_string(),
            effect: PolicyEffect::Deny,
        });

        engine.add_policy(policy);

        let component1 = create_test_component("any-component-1");
        let component2 = create_test_component("any-component-2");

        let result1 = engine.evaluate(&component1, "admin", "/any/resource");
        assert!(result1.is_err());

        let result2 = engine.evaluate(&component2, "admin", "/any/resource");
        assert!(result2.is_err());
    }

    #[test]
    fn test_evaluate_error_message() {
        let mut engine = PolicyEngine::new();

        let mut policy = SecurityPolicy::new("test-deny-policy", "comp-*");
        policy.add_rule(PolicyRule {
            action: "write".to_string(),
            resource_pattern: "/secret/*".to_string(),
            effect: PolicyEffect::Deny,
        });

        let component = create_test_component("comp-123");
        engine.add_policy(policy);

        let result = engine.evaluate(&component, "write", "/secret/file");

        assert!(result.is_err());
        if let Err(SecurityError::PolicyViolation(msg)) = result {
            assert!(msg.contains("test-deny-policy"));
            assert!(msg.contains("write"));
            assert!(msg.contains("/secret/file"));
        } else {
            panic!("Expected PolicyViolation error");
        }
    }

    #[test]
    fn test_policy_engine_debug_format() {
        let mut engine = PolicyEngine::new();
        let policy = SecurityPolicy::new("test-policy", "comp-*");
        engine.add_policy(policy);

        let debug_str = format!("{:?}", engine);
        assert!(debug_str.contains("PolicyEngine"));
        assert!(debug_str.contains("1")); // policy count
    }
}
