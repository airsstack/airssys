//! Security policy rule types.
//!
//! This module defines the core types for policy-based security evaluation:
//! - [`SecurityPolicy`]: Container for policy rules with component pattern matching
//! - [`PolicyRule`]: Individual rule with action, resource pattern, and effect
//! - [`PolicyEffect`]: Allow or Deny effect for rule evaluation
//!
//! # Examples
//!
//! Creating a policy with deny rules:
//!
//! ```
//! use airssys_wasm::security::policy::rules::{SecurityPolicy, PolicyRule, PolicyEffect};
//!
//! let mut policy = SecurityPolicy::new("filesystem-policy", "comp-*");
//!
//! policy.add_rule(PolicyRule {
//!     action: "write".to_string(),
//!     resource_pattern: "/sensitive/*".to_string(),
//!     effect: PolicyEffect::Deny,
//! });
//! ```

// Layer 1: Standard library imports (per PROJECTS_STANDARD.md §2.1)
use std::fmt;

// Layer 2: Third-party crate imports
// None required

// Layer 3: Internal module imports
use crate::core::component::id::ComponentId;
use crate::core::security::errors::SecurityError;

/// Security policy containing rules for component actions.
///
/// A policy applies to components matching its component_pattern and contains
/// rules that evaluate specific actions on resources.
#[derive(Debug, Clone)]
pub struct SecurityPolicy {
    /// Human-readable policy name for error messages
    pub name: String,
    /// Pattern for matching components (supports "*" wildcard and prefixes like "abc-*")
    pub component_pattern: String,
    /// Rules that define allowed/denied actions
    pub rules: Vec<PolicyRule>,
}

/// Individual policy rule with action, resource pattern, and effect.
///
/// Rules are evaluated in order, with Deny effects taking precedence.
#[derive(Debug, Clone)]
pub struct PolicyRule {
    /// Action to match (e.g., "read", "write", "*" for all actions)
    pub action: String,
    /// Resource pattern to match (supports "*" wildcard and prefixes)
    pub resource_pattern: String,
    /// Effect when rule matches: Allow or Deny
    pub effect: PolicyEffect,
}

/// Policy effect determining whether an action is allowed or denied.
///
/// Deny effects take precedence over Allow effects in evaluation.
#[derive(Debug, Clone, PartialEq)]
pub enum PolicyEffect {
    /// Allow the action if no Deny rules match
    Allow,
    /// Deny the action, preventing execution
    Deny,
}

impl SecurityPolicy {
    /// Creates a new security policy with a name and component pattern.
    ///
    /// # Arguments
    ///
    /// * `name` - Human-readable name for error messages
    /// * `component_pattern` - Pattern for matching components (supports "*" and "abc-*")
    ///
    /// # Examples
    ///
    /// ```
    /// use airssys_wasm::security::policy::rules::SecurityPolicy;
    ///
    /// let policy = SecurityPolicy::new("storage-policy", "storage-*");
    /// ```
    pub fn new(name: &str, component_pattern: &str) -> Self {
        Self {
            name: name.to_string(),
            component_pattern: component_pattern.to_string(),
            rules: Vec::new(),
        }
    }

    /// Adds a rule to the policy.
    ///
    /// # Arguments
    ///
    /// * `rule` - PolicyRule to add to the policy
    pub fn add_rule(&mut self, rule: PolicyRule) {
        self.rules.push(rule);
    }

    /// Checks if this policy applies to a given component.
    ///
    /// Uses pattern matching with support for:
    /// - "*" wildcard (matches all components)
    /// - Prefix patterns like "abc-*" (matches components starting with "abc-")
    /// - Exact string match
    ///
    /// # Arguments
    ///
    /// * `component` - ComponentId to check against pattern
    ///
    /// # Returns
    ///
    /// `true` if the policy applies to the component, `false` otherwise
    pub fn applies_to(&self, component: &ComponentId) -> bool {
        let component_str = component.to_string_id();

        if self.component_pattern == "*" {
            return true;
        }

        if self.component_pattern.ends_with("/*") {
            let prefix = &self.component_pattern[..self.component_pattern.len() - 2];
            return component_str.starts_with(prefix);
        }

        if self.component_pattern.ends_with('*') {
            let prefix = &self.component_pattern[..self.component_pattern.len() - 1];
            return component_str.starts_with(prefix);
        }

        component_str.starts_with(&self.component_pattern)
    }

    /// Evaluates an action on a resource against this policy's rules.
    ///
    /// Checks all rules in order, returning an error if any Deny rule matches.
    /// Allow rules are informational and do not affect the result.
    ///
    /// # Arguments
    ///
    /// * `action` - Action to evaluate (e.g., "read", "write")
    /// * `resource` - Resource to evaluate (e.g., "/path/to/file")
    ///
    /// # Returns
    ///
    /// `Ok(())` if allowed, `Err(SecurityError::PolicyViolation)` if denied
    ///
    /// # Errors
    ///
    /// Returns `SecurityError::PolicyViolation` if a Deny rule matches.
    pub fn evaluate(&self, action: &str, resource: &str) -> Result<(), SecurityError> {
        for rule in &self.rules {
            let action_matches = rule.action == "*" || rule.action == action;

            let resource_matches = if rule.resource_pattern == "*" {
                true
            } else if rule.resource_pattern.ends_with("/*") {
                let prefix = &rule.resource_pattern[..rule.resource_pattern.len() - 2];
                resource.starts_with(prefix)
            } else if rule.resource_pattern.ends_with('*') {
                let prefix = &rule.resource_pattern[..rule.resource_pattern.len() - 1];
                resource.starts_with(prefix)
            } else {
                resource.starts_with(&rule.resource_pattern)
            };

            if action_matches && resource_matches && rule.effect == PolicyEffect::Deny {
                return Err(SecurityError::PolicyViolation(format!(
                    "Policy {} denies {} on {}",
                    self.name, action, resource
                )));
            }
            // Allow rules don't affect result - they're informational
        }

        Ok(())
    }
}

impl fmt::Display for PolicyEffect {
    /// Formats the policy effect for display.
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Allow => write!(f, "Allow"),
            Self::Deny => write!(f, "Deny"),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn create_test_component(id: &str) -> ComponentId {
        // Create component in format matching test expectations
        // Result string: "{id}//" (namespace=id, name="", instance="")
        ComponentId::new(id, "", "")
    }

    #[test]
    fn test_security_policy_new() {
        let policy = SecurityPolicy::new("test-policy", "comp-*");
        assert_eq!(policy.name, "test-policy");
        assert_eq!(policy.component_pattern, "comp-*");
        assert!(policy.rules.is_empty());
    }

    #[test]
    fn test_security_policy_add_rule() {
        let mut policy = SecurityPolicy::new("test-policy", "comp-*");
        let rule = PolicyRule {
            action: "read".to_string(),
            resource_pattern: "/data/*".to_string(),
            effect: PolicyEffect::Allow,
        };

        policy.add_rule(rule);
        assert_eq!(policy.rules.len(), 1);
    }

    #[test]
    fn test_applies_to_wildcard() {
        let policy = SecurityPolicy::new("test-policy", "*");
        let component = create_test_component("any-component");
        assert!(policy.applies_to(&component));
    }

    #[test]
    fn test_applies_to_prefix() {
        let policy = SecurityPolicy::new("test-policy", "abc-");
        let component = create_test_component("abc-123");
        assert!(policy.applies_to(&component));
    }

    #[test]
    fn test_applies_to_exact() {
        let policy = SecurityPolicy::new("test-policy", "specific//");
        let component = create_test_component("specific");
        assert!(policy.applies_to(&component));
    }

    #[test]
    fn test_applies_to_non_matching() {
        let policy = SecurityPolicy::new("test-policy", "xyz-");
        let component = create_test_component("abc-123");
        assert!(!policy.applies_to(&component));
    }

    #[test]
    fn test_evaluate_no_rules() {
        let policy = SecurityPolicy::new("test-policy", "comp-*");
        let result = policy.evaluate("read", "/data/file");
        assert!(result.is_ok());
    }

    #[test]
    fn test_evaluate_action_wildcard() {
        let mut policy = SecurityPolicy::new("test-policy", "comp-*");
        policy.add_rule(PolicyRule {
            action: "*".to_string(),
            resource_pattern: "/secret/*".to_string(),
            effect: PolicyEffect::Deny,
        });

        let result = policy.evaluate("any-action", "/secret/file");
        assert!(result.is_err());
    }

    #[test]
    fn test_evaluate_resource_wildcard() {
        let mut policy = SecurityPolicy::new("test-policy", "comp-*");
        policy.add_rule(PolicyRule {
            action: "write".to_string(),
            resource_pattern: "*".to_string(),
            effect: PolicyEffect::Deny,
        });

        let result = policy.evaluate("write", "/any/path");
        assert!(result.is_err());
    }

    #[test]
    fn test_evaluate_exact_match() {
        let mut policy = SecurityPolicy::new("test-policy", "comp-*");
        policy.add_rule(PolicyRule {
            action: "read".to_string(),
            resource_pattern: "/data/file.txt".to_string(),
            effect: PolicyEffect::Allow,
        });

        let result = policy.evaluate("read", "/data/file.txt");
        assert!(result.is_ok());
    }

    #[test]
    fn test_evaluate_prefix_match() {
        let mut policy = SecurityPolicy::new("test-policy", "comp-*");
        policy.add_rule(PolicyRule {
            action: "write".to_string(),
            resource_pattern: "/data/*".to_string(),
            effect: PolicyEffect::Deny,
        });

        let result = policy.evaluate("write", "/data/subdir/file.txt");
        assert!(result.is_err());
    }

    #[test]
    fn test_evaluate_allow_effect() {
        let mut policy = SecurityPolicy::new("test-policy", "comp-*");
        policy.add_rule(PolicyRule {
            action: "read".to_string(),
            resource_pattern: "/public/*".to_string(),
            effect: PolicyEffect::Allow,
        });

        let result = policy.evaluate("read", "/public/file.txt");
        assert!(result.is_ok());
    }

    #[test]
    fn test_evaluate_deny_effect() {
        let mut policy = SecurityPolicy::new("test-policy", "comp-*");
        policy.add_rule(PolicyRule {
            action: "delete".to_string(),
            resource_pattern: "/important/*".to_string(),
            effect: PolicyEffect::Deny,
        });

        let result = policy.evaluate("delete", "/important/file.txt");
        assert!(result.is_err());
    }

    #[test]
    fn test_policy_effect_display() {
        assert_eq!(format!("{}", PolicyEffect::Allow), "Allow");
        assert_eq!(format!("{}", PolicyEffect::Deny), "Deny");
    }
}
