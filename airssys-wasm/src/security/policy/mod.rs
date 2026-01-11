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
//! let component = ComponentId::new("storage-123".to_string(), "storage", "123");
//! let result = engine.evaluate(&component, "write", "/sensitive/file");
//! ```

pub mod engine;
pub mod rules;

// Re-export commonly used types for convenience
pub use engine::PolicyEngine;
pub use rules::{PolicyEffect, PolicyRule, SecurityPolicy};
