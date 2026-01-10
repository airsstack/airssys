# WASM-TASK-025: Implementation Plans

## Plan References
- **ADR-WASM-029:** Security Module Design (primary specification)
- **ADR-WASM-025:** Clean-Slate Rebuild Architecture
- **ADR-WASM-026:** Implementation Roadmap (Phase 4)
- **KNOWLEDGE-WASM-037:** Rebuild Architecture Clean-Slate
- **KNOWLEDGE-WASM-020:** OSL Security Integration

## Target Structure Reference

Per ADR-WASM-029:
```
security/capability/
├── mod.rs           # Module declarations
├── types.rs         # PatternMatcher, core re-exports
├── set.rs           # CapabilitySet, permission structs
├── grant.rs         # CapabilityGrant
└── validator.rs     # (WASM-TASK-026)
```

---

## Implementation Actions

### Action 1: Create `security/capability/types.rs`

**Objective:** Implement PatternMatcher and re-export core capability types

**File:** `airssys-wasm/src/security/capability/types.rs`

**Specification (ADR-WASM-029 lines 64-89):**

```rust
//! Capability type utilities and re-exports.

use crate::core::security::capability::{
    Capability, MessagingCapability, StorageCapability,
    FilesystemCapability, NetworkCapability,
};

// Re-export from core
pub use crate::core::security::capability::*;

/// Pattern matcher for capability validation.
pub struct PatternMatcher;

impl PatternMatcher {
    /// Match a target against a pattern (glob-like)
    pub fn matches(pattern: &str, target: &str) -> bool {
        if pattern == "*" {
            return true;
        }
        if pattern.ends_with("/*") {
            let prefix = &pattern[..pattern.len() - 2];
            return target.starts_with(prefix);
        }
        pattern == target
    }
}
```

**Tests:** 5 unit tests
- Wildcard matching test
- Prefix pattern matching test
- Exact match test
- Non-matching pattern test
- Edge cases (empty strings)

---

### Action 2: Create `security/capability/set.rs`

**Objective:** Implement CapabilitySet for managing component permissions

**File:** `airssys-wasm/src/security/capability/set.rs`

**Specification (ADR-WASM-029 lines 93-164):**

- CapabilitySet struct with permission vectors
- MessagingPermission, StoragePermission, FilesystemPermission, NetworkPermission structs
- Add methods for each permission type
- Has permission check methods

**Tests:** 8 unit tests
- Create empty CapabilitySet
- Add and check messaging permission
- Add and check storage permission
- Pattern-based permission matching
- Permission denial tests

---

### Action 3: Create `security/capability/grant.rs`

**Objective:** Implement CapabilityGrant for permission grants

**File:** `airssys-wasm/src/security/capability/grant.rs`

```rust
//! Capability grant management.

use crate::core::component::id::ComponentId;
use super::set::CapabilitySet;

/// Represents a capability grant to a component.
#[derive(Debug, Clone)]
pub struct CapabilityGrant {
    /// The component receiving the grant.
    pub component: ComponentId,
    /// The set of capabilities granted.
    pub capabilities: CapabilitySet,
    /// Optional expiration timestamp (ms since epoch).
    pub expires_at: Option<u64>,
}

impl CapabilityGrant {
    /// Create a new capability grant.
    pub fn new(component: ComponentId, capabilities: CapabilitySet) -> Self {
        Self {
            component,
            capabilities,
            expires_at: None,
        }
    }

    /// Check if the grant has expired.
    pub fn is_expired(&self, current_time_ms: u64) -> bool {
        self.expires_at.map_or(false, |exp| current_time_ms > exp)
    }
}
```

**Tests:** 4 unit tests
- Create grant
- Check expiration
- Grant with capabilities

---

### Action 4: Create `security/capability/mod.rs`

**Objective:** Module declarations following §4.3 pattern

**File:** `airssys-wasm/src/security/capability/mod.rs`

```rust
//! # Capability Submodule
//!
//! Capability management for security validation.
//!
//! - [`types`] - PatternMatcher and core re-exports
//! - [`set`] - CapabilitySet for permission management
//! - [`grant`] - CapabilityGrant for permission grants

pub mod grant;
pub mod set;
pub mod types;
```

---

### Action 5: Update `security/mod.rs`

**Objective:** Add capability submodule declaration

**File:** `airssys-wasm/src/security/mod.rs`

Add `pub mod capability;` to module declarations.

---

## Verification Commands

```bash
# 1. Build check
cargo build -p airssys-wasm

# 2. Lint check
cargo clippy -p airssys-wasm --all-targets -- -D warnings

# 3. Run security tests
cargo test -p airssys-wasm --lib security::capability

# 4. Module boundary check
grep -rn "use crate::runtime\|use crate::component\|use crate::messaging\|use crate::system" src/security/
# Should return empty (no forbidden imports)
```

---

## Success Criteria

- [ ] All types from ADR-WASM-029 implemented
- [ ] Build passes with zero warnings
- [ ] Clippy passes with zero warnings
- [ ] All unit tests pass
- [ ] Only imports from core/ (Layer 1)
- [ ] mod.rs files contain only declarations
