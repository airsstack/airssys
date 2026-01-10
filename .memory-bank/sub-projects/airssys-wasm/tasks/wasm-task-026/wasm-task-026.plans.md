# WASM-TASK-026: Implementation Plans

## Plan References
- **ADR-WASM-029:** Security Module Design (lines 168-277)
- **ADR-WASM-025:** Clean-Slate Rebuild Architecture
- **ADR-WASM-026:** Implementation Roadmap (Phase 4)
- **KNOWLEDGE-WASM-020:** OSL Security Integration

## Target Structure Reference

Per ADR-WASM-029:
```
security/capability/
├── ...
└── validator.rs     # CapabilityValidator implementation
```

---

## Implementation Actions

### Action 1: Create `security/capability/validator.rs`

**Objective:** Implement CapabilityValidator that implements SecurityValidator trait

**File:** `airssys-wasm/src/security/capability/validator.rs`

**Specification (ADR-WASM-029 lines 168-277):**

```rust
//! Capability validator implementation.

use std::collections::HashMap;
use std::sync::RwLock;

use crate::core::component::id::ComponentId;
use crate::core::security::capability::Capability;
use crate::core::security::errors::SecurityError;
use crate::core::security::traits::SecurityValidator;

use super::set::CapabilitySet;

/// Implementation of SecurityValidator trait.
pub struct CapabilityValidator {
    /// Registered capabilities per component.
    capabilities: RwLock<HashMap<ComponentId, CapabilitySet>>,
}

impl CapabilityValidator {
    pub fn new() -> Self {
        Self {
            capabilities: RwLock::new(HashMap::new()),
        }
    }

    /// Register capabilities for a component.
    pub fn register_component(&self, id: ComponentId, capabilities: CapabilitySet) {
        let mut caps = self.capabilities.write().unwrap();
        caps.insert(id, capabilities);
    }

    /// Unregister a component.
    pub fn unregister_component(&self, id: &ComponentId) {
        let mut caps = self.capabilities.write().unwrap();
        caps.remove(id);
    }
}

impl SecurityValidator for CapabilityValidator {
    fn validate_capability(
        &self,
        component: &ComponentId,
        capability: &Capability,
    ) -> Result<(), SecurityError> {
        // Implementation per ADR-WASM-029
    }

    fn can_send_to(
        &self,
        sender: &ComponentId,
        target: &ComponentId,
    ) -> Result<(), SecurityError> {
        // Implementation per ADR-WASM-029
    }
}
```

**Tests:** 10 unit tests
- Create validator
- Register/unregister component
- Validate messaging capability (granted)
- Validate messaging capability (denied)
- Validate storage capability
- can_send_to granted
- can_send_to denied
- Unregistered component handling
- Thread-safety test

---

### Action 2: Update `security/capability/mod.rs`

**Objective:** Add validator module declaration

Add `pub mod validator;` to module declarations.

---

## Verification Commands

```bash
# 1. Build check
cargo build -p airssys-wasm

# 2. Lint check
cargo clippy -p airssys-wasm --all-targets -- -D warnings

# 3. Run validator tests
cargo test -p airssys-wasm --lib security::capability::validator

# 4. Verify trait implementation
cargo test -p airssys-wasm --lib SecurityValidator
```

---

## Success Criteria

- [ ] CapabilityValidator implements SecurityValidator
- [ ] Build passes with zero warnings
- [ ] Thread-safe operations verified
- [ ] All unit tests pass
- [ ] Proper error handling with SecurityError
