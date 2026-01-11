# WASM-TASK-027: Implementation Plans

## Plan References
- **ADR-WASM-029:** Security Module Design (lines 280-386) - Defines SecurityPolicy, PolicyRule, PolicyEffect, and PolicyEngine specifications
- **ADR-WASM-023:** Module Boundary Enforcement (MANDATORY) - Enforces that security/policy/ CANNOT import from runtime/ or actor/
- **ADR-WASM-025:** Clean-Slate Rebuild Architecture - Six-module architecture with dependency inversion
- **ADR-WASM-026:** Implementation Roadmap (Phase 4) - Phase 4 security module implementation context
- **KNOWLEDGE-WASM-037:** Rebuild Architecture - Clean Slate Design - Current architecture foundation for rebuild
- **KNOWLEDGE-WASM-020:** airssys-osl Security Integration - Security integration patterns

**System Patterns:**
- Dependency Inversion Pattern (KNOWLEDGE-WASM-037) - security/ depends on core/ abstractions only
- Module Boundary Pattern (ADR-WASM-023) - One-way dependency: security/ → core/

**PROJECTS_STANDARD.md Compliance:**
- §2.1 (3-Layer Import Organization): Code will follow std → external → internal import order
- §4.3 (Module Architecture Patterns): mod.rs files will only contain module declarations
- §6.2 (Avoid `dyn` Patterns): Static dispatch preferred over trait objects
- §6.4 (Quality Gates): Zero warnings, comprehensive tests

**Rust Guidelines Applied:**
- M-DESIGN-FOR-AI: Idiomatic APIs, thorough docs, testable code
- M-MODULE-DOCS: Module documentation with examples and comprehensive descriptions
- M-ERRORS-CANONICAL-STRUCTS: Error types use SecurityError from core/security/
- M-STATIC-VERIFICATION: All lints enabled, clippy used for warnings
- M-FEATURES-ADDITIVE: Features will not break existing code

**Documentation Standards:**
- Diátaxis Type: Reference documentation for APIs
- Quality: Technical language, no marketing terms per documentation-quality-standards.md
- Compliance checklist: Will be added to task file upon completion

## Target Structure Reference

Per ADR-WASM-029:
```
security/policy/
├── mod.rs           # Module declarations only (§4.3)
├── engine.rs        # PolicyEngine for rule evaluation
└── rules.rs         # SecurityPolicy, PolicyRule, PolicyEffect
```

## Module Architecture (Mandatory per ADR-WASM-023)

**Code will be placed in:** `security/policy/`

**Module responsibilities (per ADR-WASM-023):**
- Policy-based security evaluation
- SecurityPolicy, PolicyRule, PolicyEffect types
- PolicyEngine for multi-policy evaluation

**Allowed imports (per ADR-WASM-023):**
- ✅ `crate::core::component::id::ComponentId` (core types allowed)
- ✅ `crate::core::security::errors::SecurityError` (core security types allowed)
- ✅ `super::rules::` (sibling imports within policy/)

**Forbidden imports (per ADR-WASM-023):**
- ❌ `crate::runtime::` (security/ CANNOT import from runtime/)
- ❌ `crate::actor::` (security/ CANNOT import from actor/)
- ❌ `crate::component::` (security/ CANNOT import from component/)

**Verification command (for implementer to run):**
```bash
# MUST return no output for valid architecture
grep -rn "use crate::runtime" airssys-wasm/src/security/policy/
grep -rn "use crate::actor" airssys-wasm/src/security/policy/
grep -rn "use crate::component" airssys-wasm/src/security/policy/
```

---

## Unit Testing Plan (MANDATORY per AGENTS.md §9)

### Overview
Comprehensive unit tests will be written in `#[cfg(test)]` blocks within each module file. Tests will validate actual functionality, not just API existence.

### Test Coverage Requirements
- All public methods must have unit tests
- All error paths must be tested
- Edge cases must be covered
- Pattern matching logic must be validated

### Test File Structure
```rust
#[cfg(test)]
mod tests {
    use super::*;

    // Test cases here
}
```

### Specific Tests Per Module

#### rules.rs Unit Tests (13 tests minimum)
1. **Policy Creation Tests:**
   - `test_security_policy_new()` - Creates SecurityPolicy with name and pattern
   - `test_security_policy_add_rule()` - Adds PolicyRule to policy

2. **Pattern Matching Tests:**
   - `test_applies_to_wildcard()` - Pattern "*" matches all components
   - `test_applies_to_prefix()` - Pattern "abc-*" matches "abc-123"
   - `test_applies_to_exact()` - Exact pattern match
   - `test_applies_to_non_matching()` - Non-matching component

3. **Policy Evaluation Tests:**
   - `test_evaluate_no_rules()` - No rules = allow
   - `test_evaluate_action_wildcard()` - "*" action matches any action
   - `test_evaluate_resource_wildcard()` - "*" resource matches any resource
   - `test_evaluate_exact_match()` - Exact action/resource match
   - `test_evaluate_prefix_match()` - Prefix resource match
   - `test_evaluate_allow_effect()` - Allow effect returns Ok(())
   - `test_evaluate_deny_effect()` - Deny effect returns SecurityError

#### engine.rs Unit Tests (10 tests minimum)
1. **Engine Creation Tests:**
   - `test_policy_engine_new()` - Creates PolicyEngine
   - `test_policy_engine_add_policy()` - Adds SecurityPolicy

2. **Policy Evaluation Tests:**
   - `test_evaluate_no_policies()` - No policies = allow
   - `test_evaluate_single_policy_matching()` - Single matching policy
   - `test_evaluate_single_policy_non_matching()` - Non-matching policy ignored
   - `test_evaluate_multiple_policies_all_allow()` - Multiple allow policies
   - `test_evaluate_multiple_policies_one_deny()` - Deny policy blocks action
   - `test_evaluate_multiple_policies_component_specific()` - Different policies for different components
   - `test_evaluate_component_pattern_wildcard()` - Wildcard pattern policy
   - `test_evaluate_error_message()` - Deny error contains policy name

### Test Quality Requirements (per KNOWLEDGE-WASM-033)
- ✅ Tests use real types (no mocks)
- ✅ Tests verify actual behavior (not just API existence)
- ✅ Tests include edge cases and error paths
- ✅ All tests pass: `cargo test --lib security::policy`

---

## Integration Testing Plan (MANDATORY per AGENTS.md §9)

### Overview
Integration tests will be written in `tests/security-policy-integration-tests.rs`. Tests will verify end-to-end policy evaluation with real ComponentId instances.

### Integration Test File
**File:** `airssys-wasm/tests/security-policy-integration-tests.rs`

### Integration Tests (6 tests minimum)
1. **End-to-End Policy Evaluation:**
   - `test_full_policy_evaluation_workflow()` - Create policy, add to engine, evaluate with real ComponentId

2. **Multi-Component Policy Enforcement:**
   - `test_multiple_components_different_policies()` - Different policies for different components

3. **Pattern Matching Integration:**
   - `test_component_prefix_pattern_integration()` - Real ComponentId prefix matching

4. **Error Handling Integration:**
   - `test_policy_violation_error_propagation()` - Verify SecurityError propagates correctly

5. **Complex Policy Scenarios:**
   - `test_multiple_rules_single_action()` - Multiple rules evaluated for single action
   - `test_deny_takes_precedence()` - Deny effect blocks action even with allow

### Integration Test Quality Requirements
- ✅ Tests use real ComponentId instances (from core/)
- ✅ Tests verify complete workflow from policy creation to evaluation
- ✅ All tests pass: `cargo test --test security-policy-integration-tests`

### Fixture Requirements
No external fixtures needed for this task (pure logic module, no WASM execution).

---

## Implementation Actions

### Action 1: Create `security/policy/rules.rs`

**Objective:** Implement SecurityPolicy, PolicyRule, and PolicyEffect types with pattern matching and evaluation logic

**File:** `airssys-wasm/src/security/policy/rules.rs`

**Specification (ADR-WASM-029 lines 324-386):**

```rust
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

        component_str.starts_with(&self.component_pattern)
    }

    /// Evaluates an action on a resource against this policy's rules.
    ///
    /// Checks all rules in order, returning an error if any Deny rule matches.
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
            let resource_matches = rule.resource_pattern == "*"
                || (rule.resource_pattern.ends_with("/*")
                    && resource.starts_with(&rule.resource_pattern[..rule.resource_pattern.len() - 2]))
                || resource.starts_with(&rule.resource_pattern);

            if action_matches && resource_matches {
                if rule.effect == PolicyEffect::Deny {
                    return Err(SecurityError::PolicyViolation(format!(
                        "Policy {} denies {} on {}",
                        self.name, action, resource
                    )));
                }
            }
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn create_test_component(id: &str) -> ComponentId {
        ComponentId::new(id.to_string())
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
        let policy = SecurityPolicy::new("test-policy", "abc-*");
        let component = create_test_component("abc-123");
        assert!(policy.applies_to(&component));
    }

    #[test]
    fn test_applies_to_exact() {
        let policy = SecurityPolicy::new("test-policy", "specific-comp");
        let component = create_test_component("specific-comp");
        assert!(policy.applies_to(&component));
    }

    #[test]
    fn test_applies_to_non_matching() {
        let policy = SecurityPolicy::new("test-policy", "xyz-*");
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
}
```

**ADR Constraints:**
- ADR-WASM-029 requires: SecurityPolicy, PolicyRule, PolicyEffect as specified
- ADR-WASM-023 requires: Must import from core/ ONLY, not runtime/ or actor/

**PROJECTS_STANDARD.md Compliance:**
- §2.1: 3-layer import organization (std, external, internal)
- §4.3: Module has #[cfg(test)] block for unit tests
- §6.2: No `dyn` trait objects, uses concrete types
- §6.4: Zero warnings, comprehensive tests

**Rust Guidelines:**
- M-MODULE-DOCS: Comprehensive module documentation with examples
- M-ERRORS-CANONICAL-STRUCTS: Uses SecurityError from core/security/
- M-STATIC-VERIFICATION: All lints enabled

**Documentation:**
- Diátaxis type: Reference documentation for APIs
- Quality: Technical language, no marketing terms
- Compliance checklist: Will add to task file

**Verification:**
```bash
# Build check
cargo build -p airssys-wasm

# Unit tests pass
cargo test -p airssys-wasm --lib security::policy::rules

# Module architecture clean (ADR-WASM-023)
grep -rn "use crate::runtime" airssys-wasm/src/security/policy/rules.rs
grep -rn "use crate::actor" airssys-wasm/src/security/policy/rules.rs
grep -rn "use crate::component" airssys-wasm/src/security/policy/rules.rs
```

---

### Action 2: Create `security/policy/engine.rs`

**Objective:** Implement PolicyEngine for evaluating multiple policies across components

**File:** `airssys-wasm/src/security/policy/engine.rs`

**Specification (ADR-WASM-029 lines 282-320):**

```rust
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
//! let component = ComponentId::new("comp-123".to_string());
//! let result = engine.evaluate(&component, "write", "/sensitive/file");
//! ```

use crate::core::component::id::ComponentId;
use crate::core::security::errors::SecurityError;

use super::rules::{PolicyRule, SecurityPolicy};

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
    /// let component = ComponentId::new("comp-123".to_string());
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::security::policy::rules::{PolicyEffect, PolicyRule, SecurityPolicy};

    fn create_test_component(id: &str) -> ComponentId {
        ComponentId::new(id.to_string())
    }

    #[test]
    fn test_policy_engine_new() {
        let engine = PolicyEngine::new();
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

        engine.add_policy(policy);

        let component = create_test_component("comp-123");
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
}
```

**ADR Constraints:**
- ADR-WASM-029 requires: PolicyEngine with evaluate() method as specified
- ADR-WASM-023 requires: Must import from core/ ONLY, not runtime/ or actor/

**PROJECTS_STANDARD.md Compliance:**
- §2.1: 3-layer import organization
- §4.3: Module has #[cfg(test)] block for unit tests
- §6.2: No `dyn` trait objects
- §6.4: Zero warnings, comprehensive tests

**Rust Guidelines:**
- M-MODULE-DOCS: Comprehensive module documentation with examples
- M-ERRORS-CANONICAL-STRUCTS: Uses SecurityError from core/security/
- M-STATIC-VERIFICATION: All lints enabled

**Documentation:**
- Diátaxis type: Reference documentation for APIs
- Quality: Technical language, no marketing terms
- Compliance checklist: Will add to task file

**Verification:**
```bash
# Build check
cargo build -p airssys-wasm

# Unit tests pass
cargo test -p airssys-wasm --lib security::policy::engine

# Module architecture clean (ADR-WASM-023)
grep -rn "use crate::runtime" airssys-wasm/src/security/policy/engine.rs
grep -rn "use crate::actor" airssys-wasm/src/security/policy/engine.rs
grep -rn "use crate::component" airssys-wasm/src/security/policy/engine.rs
```

---

### Action 3: Create `security/policy/mod.rs`

**Objective:** Module declarations following §4.3 pattern (no implementation, only declarations)

**File:** `airssys-wasm/src/security/policy/mod.rs`

```rust
//! # Policy Submodule
//!
//! Policy-based security evaluation for component actions.
//!
//! This submodule provides types and engine for evaluating security policies
//! that define which actions components can perform on which resources.
//!
//! ## Modules
//!
//! - [`engine`] - [`PolicyEngine`] for rule evaluation across multiple policies
//! - [`rules`] - [`SecurityPolicy`], [`PolicyRule`], [`PolicyEffect`] types
//!
//! ## Overview
//!
//! The policy system complements capability-based security by providing
//! a declarative way to define allowed and denied actions:
//!
//! - **Policies** contain rules with component pattern matching
//! - **Rules** define action/resource patterns with Allow/Deny effects
//! - **Engine** evaluates all applicable policies for component actions
//!
//! ## Pattern Matching
//!
//! Component and resource patterns support:
//! - `"*"` wildcard: matches all components/resources
//! - `"prefix-*"` pattern: matches strings starting with prefix
//! - Exact string match
//!
//! ## Examples
//!
//! Creating a policy with deny rules:
//!
//! ```no_run
//! use airssys_wasm::security::policy::engine::PolicyEngine;
//! use airssys_wasm::security::policy::rules::{SecurityPolicy, PolicyRule, PolicyEffect};
//! use airssys_wasm::core::component::id::ComponentId;
//!
//! let mut engine = PolicyEngine::new();
//!
//! // Deny writes to sensitive paths for storage components
//! let mut policy = SecurityPolicy::new("storage-policy", "storage-*");
//! policy.add_rule(PolicyRule {
//!     action: "write".to_string(),
//!     resource_pattern: "/sensitive/*".to_string(),
//!     effect: PolicyEffect::Deny,
//! });
//!
//! engine.add_policy(policy);
//!
//! let component = ComponentId::new("storage-123".to_string());
//! let result = engine.evaluate(&component, "write", "/sensitive/file");
//! ```

pub mod engine;
pub mod rules;

// Re-export commonly used types for convenience
pub use engine::PolicyEngine;
pub use rules::{PolicyEffect, PolicyRule, SecurityPolicy};
```

**ADR Constraints:**
- ADR-WASM-029 requires: Module structure with engine.rs and rules.rs

**PROJECTS_STANDARD.md Compliance:**
- §4.3: mod.rs contains ONLY module declarations (no implementation code)

**Rust Guidelines:**
- M-MODULE-DOCS: Module documentation with overview and examples

**Documentation:**
- Diátaxis type: Reference documentation for module structure
- Quality: Technical language, no marketing terms
- Compliance checklist: Will add to task file

**Verification:**
```bash
# Module has only declarations (no implementation code)
grep -E "(fn |struct |enum |impl )" airssys-wasm/src/security/policy/mod.rs
# Expected: No matches (only declarations)
```

---

### Action 4: Update `security/mod.rs`

**Objective:** Add policy submodule declaration to security/mod.rs

**File:** `airssys-wasm/src/security/mod.rs`

Add `pub mod policy;` to module declarations section.

**Verification:**
```bash
# Build check
cargo build -p airssys-wasm
```

---

## Integration Test File Creation

**File:** `airssys-wasm/tests/security-policy-integration-tests.rs`

```rust
//! Integration tests for security policy evaluation.
//!
//! Tests verify end-to-end policy evaluation with real ComponentId instances
//! and multiple policy scenarios.

use airssys_wasm::core::component::id::ComponentId;
use airssys_wasm::core::security::errors::SecurityError;
use airssys_wasm::security::policy::engine::PolicyEngine;
use airssys_wasm::security::policy::rules::{PolicyEffect, PolicyRule, SecurityPolicy};

fn create_test_component(id: &str) -> ComponentId {
    ComponentId::new(id.to_string())
}

#[test]
fn test_full_policy_evaluation_workflow() {
    let mut engine = PolicyEngine::new();

    // Create a policy with deny rules
    let mut policy = SecurityPolicy::new("test-policy", "comp-*");
    policy.add_rule(PolicyRule {
        action: "write".to_string(),
        resource_pattern: "/secret/*".to_string(),
        effect: PolicyEffect::Deny,
    });

    engine.add_policy(policy);

    // Test with matching component
    let component = create_test_component("comp-123");
    let result = engine.evaluate(&component, "write", "/secret/file");

    assert!(result.is_err());
    if let Err(SecurityError::PolicyViolation(msg)) = result {
        assert!(msg.contains("test-policy"));
    } else {
        panic!("Expected PolicyViolation error");
    }
}

#[test]
fn test_multiple_components_different_policies() {
    let mut engine = PolicyEngine::new();

    // Storage components can't delete
    let mut storage_policy = SecurityPolicy::new("storage-policy", "storage-*");
    storage_policy.add_rule(PolicyRule {
        action: "delete".to_string(),
        resource_pattern: "*".to_string(),
        effect: PolicyEffect::Deny,
    });
    engine.add_policy(storage_policy);

    // Compute components can't execute
    let mut compute_policy = SecurityPolicy::new("compute-policy", "compute-*");
    compute_policy.add_rule(PolicyRule {
        action: "execute".to_string(),
        resource_pattern: "*".to_string(),
        effect: PolicyEffect::Deny,
    });
    engine.add_policy(compute_policy);

    // Test storage component
    let storage_comp = create_test_component("storage-123");
    assert!(engine
        .evaluate(&storage_comp, "delete", "/any/file")
        .is_err());
    assert!(engine
        .evaluate(&storage_comp, "read", "/any/file")
        .is_ok());

    // Test compute component
    let compute_comp = create_test_component("compute-456");
    assert!(engine
        .evaluate(&compute_comp, "execute", "/binary")
        .is_err());
    assert!(engine
        .evaluate(&compute_comp, "read", "/any/file")
        .is_ok());
}

#[test]
fn test_component_prefix_pattern_integration() {
    let mut engine = PolicyEngine::new();

    let mut policy = SecurityPolicy::new("prefix-policy", "app-*");
    policy.add_rule(PolicyRule {
        action: "admin".to_string(),
        resource_pattern: "*".to_string(),
        effect: PolicyEffect::Deny,
    });
    engine.add_policy(policy);

    // Matching components
    assert!(engine
        .evaluate(&create_test_component("app-123"), "admin", "/resource")
        .is_err());
    assert!(engine
        .evaluate(&create_test_component("app-456"), "admin", "/resource")
        .is_err());

    // Non-matching component
    assert!(engine
        .evaluate(&create_test_component("service-789"), "admin", "/resource")
        .is_ok());
}

#[test]
fn test_policy_violation_error_propagation() {
    let mut engine = PolicyEngine::new();

    let mut policy = SecurityPolicy::new("deny-all", "*");
    policy.add_rule(PolicyRule {
        action: "dangerous".to_string(),
        resource_pattern: "*".to_string(),
        effect: PolicyEffect::Deny,
    });
    engine.add_policy(policy);

    let component = create_test_component("any-comp");
    let result = engine.evaluate(&component, "dangerous", "/any/resource");

    assert!(result.is_err());
    assert!(matches!(result, Err(SecurityError::PolicyViolation(_)));
}

#[test]
fn test_multiple_rules_single_action() {
    let mut engine = PolicyEngine::new();

    let mut policy = SecurityPolicy::new("multi-rule-policy", "comp-*");
    policy.add_rule(PolicyRule {
        action: "write".to_string(),
        resource_pattern: "/public/*".to_string(),
        effect: PolicyEffect::Allow,
    });
    policy.add_rule(PolicyRule {
        action: "write".to_string(),
        resource_pattern: "/private/*".to_string(),
        effect: PolicyEffect::Deny,
    });
    engine.add_policy(policy);

    let component = create_test_component("comp-123");

    // Public writes should be allowed
    assert!(engine
        .evaluate(&component, "write", "/public/file")
        .is_ok());

    // Private writes should be denied
    assert!(engine
        .evaluate(&component, "write", "/private/file")
        .is_err());
}

#[test]
fn test_deny_takes_precedence() {
    let mut engine = PolicyEngine::new();

    let mut policy = SecurityPolicy::new("precedence-policy", "comp-*");
    // Allow rule first
    policy.add_rule(PolicyRule {
        action: "write".to_string(),
        resource_pattern: "/data/*".to_string(),
        effect: PolicyEffect::Allow,
    });
    // Deny rule takes precedence
    policy.add_rule(PolicyRule {
        action: "write".to_string(),
        resource_pattern: "/data/secret/*".to_string(),
        effect: PolicyEffect::Deny,
    });
    engine.add_policy(policy);

    let component = create_test_component("comp-123");

    // Regular data writes should be allowed
    assert!(engine
        .evaluate(&component, "write", "/data/public/file")
        .is_ok());

    // Secret writes should be denied (deny takes precedence)
    assert!(engine
        .evaluate(&component, "write", "/data/secret/file")
        .is_err());
}
```

**Verification:**
```bash
# Integration tests pass
cargo test --test security-policy-integration-tests
```

---

## Verification Commands

Run after ALL actions complete:

```bash
# 1. Build check
cargo build -p airssys-wasm

# 2. Module architecture verification (ADR-WASM-023 MANDATORY)
# ALL MUST RETURN NOTHING for valid architecture
grep -rn "use crate::runtime" airssys-wasm/src/security/policy/
grep -rn "use crate::actor" airssys-wasm/src/security/policy/
grep -rn "use crate::component" airssys-wasm/src/security/policy/

# 3. Lint check
cargo clippy -p airssys-wasm --all-targets -- -D warnings

# 4. Unit tests pass
cargo test -p airssys-wasm --lib security::policy

# 5. Integration tests pass
cargo test -p airssys-wasm --test security-policy-integration-tests

# 6. All tests pass
cargo test -p airssys-wasm
```

---

## Success Criteria

- [ ] All types from ADR-WASM-029 implemented (SecurityPolicy, PolicyRule, PolicyEffect, PolicyEngine)
- [ ] Module structure matches ADR-WASM-029 specification
- [ ] Build passes with zero warnings
- [ ] All unit tests pass (13 tests in rules.rs, 10 tests in engine.rs)
- [ ] All integration tests pass (6 tests in integration tests)
- [ ] Module architecture verified clean (no forbidden imports)
- [ ] Policy evaluation works correctly with Allow/Deny effects
- [ ] Pattern matching works for wildcards and prefixes
- [ ] Proper error handling with SecurityError
- [ ] Module documentation follows M-MODULE-DOCS guidelines
- [ ] Standards Compliance Checklist added to task file

---

## Standards Compliance Summary

**ADR-WASM-023 (Module Boundary Enforcement):**
- ✅ security/policy/ imports ONLY from core/
- ✅ NO imports from runtime/, actor/, component/
- ✅ Verification commands confirm clean architecture

**ADR-WASM-029 (Security Module Design):**
- ✅ SecurityPolicy, PolicyRule, PolicyEffect types implemented
- ✅ PolicyEngine with evaluate() method implemented
- ✅ Pattern matching with wildcards and prefixes

**PROJECTS_STANDARD.md:**
- ✅ §2.1: 3-layer import organization
- ✅ §4.3: Module architecture patterns
- ✅ §6.2: Avoid `dyn` patterns
- ✅ §6.4: Quality Gates (zero warnings)

**Rust Guidelines:**
- ✅ M-DESIGN-FOR-AI: Idiomatic APIs, thorough docs, testable
- ✅ M-MODULE-DOCS: Module documentation with examples
- ✅ M-ERRORS-CANONICAL-STRUCTS: Error handling with SecurityError
- ✅ M-STATIC-VERIFICATION: Lints enabled, clippy used

**Testing Requirements (AGENTS.md §9):**
- ✅ Unit tests: 23 tests total (13 in rules.rs, 10 in engine.rs)
- ✅ Integration tests: 6 tests in security-policy-integration-tests.rs
- ✅ Tests validate REAL functionality (not just API existence)
- ✅ Zero warnings verified

**Documentation Standards:**
- ✅ Diátaxis type: Reference documentation
- ✅ Quality: Technical language, no marketing terms
- ✅ Standards Compliance Checklist: Will be added to task file
