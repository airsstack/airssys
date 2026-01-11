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
├── set.rs           # CapabilitySet, permission structs, Builder
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
        if let Some(prefix) = pattern.strip_suffix("/*") {
            return target.starts_with(prefix) && target.len() > prefix.len();
        }
        if let Some(suffix) = pattern.strip_prefix("*.") {
            return target.ends_with(suffix) && target.len() > suffix.len();
        }
        pattern == target
    }
}
```

**Tests:** 6 unit tests
- Wildcard matching test
- Prefix pattern matching test
- Suffix pattern matching test
- Exact match test
- Non-matching pattern test
- Edge cases (empty strings)

---

### Action 2: Create `security/capability/set.rs`

**Objective:** Implement CapabilitySet, permission structs, and builder pattern

**File:** `airssys-wasm/src/security/capability/set.rs`

**Specification (ADR-WASM-029 lines 93-164):**

- CapabilitySet struct with permission vectors
- MessagingPermission, StoragePermission, FilesystemPermission, NetworkPermission structs
- Add methods for each permission type
- Has permission check methods
- **CapabilitySetBuilder for fluent permission construction**

**Tests:** 12 unit tests
- Create empty CapabilitySet
- Add and check messaging permission
- Add and check storage permission
- Pattern-based permission matching
- Permission denial tests
- Multiple permissions
- Network permissions
- Wildcard permission
- **Builder pattern tests (4 new)**:
  - Builder with single messaging permission
  - Builder with multiple permissions
  - Builder chaining all permission types
  - Builder empty set

---

### Action 3: Create `security/capability/grant.rs`

**Objective:** Implement CapabilityGrant for permission grants

**File:** `airssys-wasm/src/security/capability/grant.rs`

```rust
//! Capability grant management.

use super::set::CapabilitySet;
use crate::core::component::id::ComponentId;

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

    /// Create a new capability grant with expiration.
    pub fn with_expiration(
        component: ComponentId,
        capabilities: CapabilitySet,
        expires_at: u64,
    ) -> Self {
        Self {
            component,
            capabilities,
            expires_at: Some(expires_at),
        }
    }

    /// Check if the grant has expired.
    pub fn is_expired(&self, current_time_ms: u64) -> bool {
        self.expires_at.is_some_and(|exp| current_time_ms > exp)
    }

    /// Get the component ID.
    pub fn component(&self) -> &ComponentId {
        &self.component
    }

    /// Get the capability set.
    pub fn capabilities(&self) -> &CapabilitySet {
        &self.capabilities
    }
}
```

**Tests:** 4 unit tests
- Create grant
- Check expiration
- Grant with capabilities
- Grant with expiration

---

### Action 4: Create `security/capability/mod.rs`

**Objective:** Module declarations following §4.3 pattern

**File:** `airssys-wasm/src/security/capability/mod.rs`

```rust
//! # Capability Submodule
//!
//! Capability management for security validation.
//!
//! ## Modules
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
grep -rn "use crate::runtime\|use crate::actor\|use crate::component\|use crate::messaging\|use crate::system" src/security/
# Should return empty (no forbidden imports)
```

---

## Success Criteria

- [ ] All types from ADR-WASM-029 implemented
- [ ] CapabilitySetBuilder implemented for fluent API
- [ ] Build passes with zero warnings
- [ ] Clippy passes with zero warnings
- [ ] All unit tests pass (22 tests: 6 + 12 + 4)
- [ ] Only imports from core/ (Layer 1)
- [ ] mod.rs files contain only declarations
- [ ] Builder pattern provides fluent API

---

## Enhancement Notes

### Builder Pattern Addition

**Why:**
The builder pattern provides a fluent API for constructing complex CapabilitySets, making code more readable and reducing verbosity when creating permission sets.

**Implementation:**
- `CapabilitySetBuilder` struct with optional permission fields
- Chained setter methods returning `&mut self` for fluent API
- `build()` method to construct final `CapabilitySet`
- Default empty CapabilitySet for no permissions

**Example Usage:**
```rust
let capabilities = CapabilitySet::builder()
    .messaging(MessagingPermission {
        can_send_to: vec!["comp-a/*".to_string()],
        can_receive_from: vec![],
    })
    .storage(StoragePermission {
        can_write_keys: vec!["user/*".to_string()],
        can_read_keys: vec!["*".to_string()],
    })
    .build();
```

**Benefits:**
- More readable code
- Method chaining for complex permission sets
- Clearer intent when creating permissions
- Consistent with Rust builder pattern conventions
