# WASM-TASK-027: Implementation Plans

## Plan References
- **ADR-WASM-029:** Security Module Design (lines 280-386)
- **ADR-WASM-025:** Clean-Slate Rebuild Architecture
- **ADR-WASM-026:** Implementation Roadmap (Phase 4)

## Target Structure Reference

Per ADR-WASM-029:
```
security/policy/
├── mod.rs           # Module declarations
├── engine.rs        # PolicyEngine for rule evaluation
└── rules.rs         # SecurityPolicy, PolicyRule types
```

---

## Implementation Actions

### Action 1: Create `security/policy/rules.rs`

**Objective:** Implement SecurityPolicy, PolicyRule, and PolicyEffect types

**File:** `airssys-wasm/src/security/policy/rules.rs`

**Specification (ADR-WASM-029 lines 324-386):**

```rust
//! Security policy rule types.

use crate::core::component::id::ComponentId;
use crate::core::security::errors::SecurityError;

/// Security policy containing rules.
#[derive(Debug, Clone)]
pub struct SecurityPolicy {
    pub name: String,
    pub component_pattern: String,
    pub rules: Vec<PolicyRule>,
}

/// Individual policy rule.
#[derive(Debug, Clone)]
pub struct PolicyRule {
    pub action: String,
    pub resource_pattern: String,
    pub effect: PolicyEffect,
}

#[derive(Debug, Clone, PartialEq)]
pub enum PolicyEffect {
    Allow,
    Deny,
}

impl SecurityPolicy {
    pub fn new(name: &str, component_pattern: &str) -> Self;
    pub fn add_rule(&mut self, rule: PolicyRule);
    pub fn applies_to(&self, component: &ComponentId) -> bool;
    pub fn evaluate(&self, action: &str, resource: &str) -> Result<(), SecurityError>;
}
```

**Tests:** 8 unit tests
- Create policy
- Add rules
- Check applies_to
- Evaluate Allow rule
- Evaluate Deny rule
- Pattern matching tests

---

### Action 2: Create `security/policy/engine.rs`

**Objective:** Implement PolicyEngine for evaluating multiple policies

**File:** `airssys-wasm/src/security/policy/engine.rs`

**Specification (ADR-WASM-029 lines 282-320):**

```rust
//! Policy evaluation engine.

use crate::core::component::id::ComponentId;
use crate::core::security::errors::SecurityError;

use super::rules::SecurityPolicy;

/// Policy evaluation engine.
pub struct PolicyEngine {
    policies: Vec<SecurityPolicy>,
}

impl PolicyEngine {
    pub fn new() -> Self;
    pub fn add_policy(&mut self, policy: SecurityPolicy);
    pub fn evaluate(
        &self,
        component: &ComponentId,
        action: &str,
        resource: &str,
    ) -> Result<(), SecurityError>;
}
```

**Tests:** 6 unit tests
- Create engine
- Add policies
- Evaluate with no policies
- Evaluate with matching policy
- Evaluate with denial
- Multiple policy evaluation

---

### Action 3: Create `security/policy/mod.rs`

**Objective:** Module declarations following §4.3 pattern

**File:** `airssys-wasm/src/security/policy/mod.rs`

```rust
//! # Policy Submodule
//!
//! Policy-based security evaluation.
//!
//! - [`engine`] - PolicyEngine for rule evaluation
//! - [`rules`] - SecurityPolicy, PolicyRule, PolicyEffect

pub mod engine;
pub mod rules;
```

---

### Action 4: Update `security/mod.rs`

**Objective:** Add policy submodule declaration

Add `pub mod policy;` to module declarations.

---

## Verification Commands

```bash
# 1. Build check
cargo build -p airssys-wasm

# 2. Lint check
cargo clippy -p airssys-wasm --all-targets -- -D warnings

# 3. Run policy tests
cargo test -p airssys-wasm --lib security::policy
```

---

## Success Criteria

- [ ] All types from ADR-WASM-029 implemented
- [ ] Build passes with zero warnings
- [ ] All unit tests pass
- [ ] Policy evaluation works correctly
- [ ] Proper error handling with SecurityError
