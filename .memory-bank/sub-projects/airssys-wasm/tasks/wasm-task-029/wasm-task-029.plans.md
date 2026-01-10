# WASM-TASK-029: Implementation Plans

## Plan References
- **ADR-WASM-029:** Security Module Design (lines 455-490)
- **ADR-WASM-025:** Clean-Slate Rebuild Architecture
- **ADR-WASM-026:** Implementation Roadmap (Phase 4)
- **KNOWLEDGE-WASM-020:** OSL Security Integration

## Target Structure Reference

Per ADR-WASM-029:
```
security/
├── mod.rs           # Contains OslSecurityBridge
├── capability/
├── policy/
└── audit.rs
```

---

## Implementation Actions

### Action 1: Add OslSecurityBridge to `security/mod.rs`

**Objective:** Implement bridge to airssys-osl SecurityContext

**File:** `airssys-wasm/src/security/mod.rs`

**Specification (ADR-WASM-029 lines 457-490):**

```rust
//! # Security Module
//!
//! Security implementation for capability-based access control.
//!
//! This module is **Layer 2A** of the architecture.

use airssys_osl::SecurityContext;
use crate::core::security::errors::SecurityError;

pub mod audit;
pub mod capability;
pub mod policy;

/// Bridge to airssys-osl SecurityContext.
pub struct OslSecurityBridge {
    security_context: SecurityContext,
}

impl OslSecurityBridge {
    pub fn new(security_context: SecurityContext) -> Self {
        Self { security_context }
    }

    /// Check OSL-level permissions before capability check.
    pub fn check_osl_permission(
        &self,
        principal: &str,
        resource: &str,
        action: &str,
    ) -> Result<(), SecurityError> {
        if self.security_context.is_permitted(principal, resource, action) {
            Ok(())
        } else {
            Err(SecurityError::PermissionDenied(format!(
                "OSL denied: {} cannot {} on {}",
                principal, action, resource
            )))
        }
    }
}
```

**Tests:** 5 unit tests
- Create bridge with SecurityContext
- Check permitted action
- Check denied action
- Error message formatting
- Integration with CapabilityValidator (if applicable)

---

### Action 2: Verify airssys-osl Dependency

**Objective:** Ensure Cargo.toml has airssys-osl dependency

Check that `airssys-wasm/Cargo.toml` includes airssys-osl from workspace.

---

## Verification Commands

```bash
# 1. Build check
cargo build -p airssys-wasm

# 2. Lint check
cargo clippy -p airssys-wasm --all-targets -- -D warnings

# 3. Run security module tests
cargo test -p airssys-wasm --lib security

# 4. Verify import compliance
grep -rn "use crate::runtime\|use crate::component\|use crate::messaging\|use crate::system" src/security/
# Should return empty (no forbidden imports)
```

---

## Success Criteria

- [ ] OslSecurityBridge wraps SecurityContext
- [ ] Build passes with zero warnings
- [ ] Integration with airssys-osl works
- [ ] All unit tests pass
- [ ] Module boundary compliance verified
