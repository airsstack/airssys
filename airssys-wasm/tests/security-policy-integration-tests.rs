//! Integration tests for security policy evaluation.
//!
//! Tests verify end-to-end policy evaluation with real ComponentId instances
//! and multiple policy scenarios.

use airssys_wasm::core::component::id::ComponentId;
use airssys_wasm::core::security::errors::SecurityError;
use airssys_wasm::security::policy::engine::PolicyEngine;
use airssys_wasm::security::policy::rules::{PolicyEffect, PolicyRule, SecurityPolicy};

fn create_test_component(id: &str) -> ComponentId {
    // Create component in format matching test expectations
    // Result string: "{id}//" (namespace=id, name="", instance="")
    ComponentId::new(id, "", "")
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
    assert!(engine.evaluate(&storage_comp, "read", "/any/file").is_ok());

    // Test compute component
    let compute_comp = create_test_component("compute-456");
    assert!(engine
        .evaluate(&compute_comp, "execute", "/binary")
        .is_err());
    assert!(engine.evaluate(&compute_comp, "read", "/any/file").is_ok());
}

#[test]
fn test_component_prefix_pattern_integration() {
    let mut engine = PolicyEngine::new();

    let mut policy = SecurityPolicy::new("prefix-policy", "app-");
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
    let is_violation = matches!(&result, Err(SecurityError::PolicyViolation(_)));
    assert!(is_violation);
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
    assert!(engine.evaluate(&component, "write", "/public/file").is_ok());

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
