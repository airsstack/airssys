# WASM-TASK-029: Implementation Plans

## Plan References
- **ADR-WASM-029:** Security Module Design (lines 455-490)
- **ADR-WASM-025:** Clean-Slate Rebuild Architecture
- **ADR-WASM-026:** Implementation Roadmap (Phase 4)
- **KNOWLEDGE-WASM-020:** OSL Security Integration
- **ADR-WASM-023:** Module Boundary Enforcement (MANDATORY for architecture compliance)

**PROJECTS_STANDARD.md Compliance:**
- §2.1 (3-Layer Imports): Code will follow import organization
- §2.2 (No FQN): Types will be imported and used by simple name
- §3.2 (DateTime<Utc>): Time operations will use Utc (if applicable)
- §4.3 (Module Architecture): mod.rs files will only contain declarations
- §5.1 (Dependency Management): Dependencies from workspace will be correctly referenced
- §6.1 (YAGNI): Simple, direct implementation without over-engineering
- §6.2 (Avoid `dyn`): Static dispatch preferred, avoid trait objects where possible
- §6.4 (Quality Gates): Zero warnings, comprehensive tests

**Rust Guidelines Applied:**
- M-DESIGN-FOR-AI: Idiomatic APIs, thorough docs, testable
- M-MODULE-DOCS: Module documentation with examples will be added
- M-ERRORS-CANONICAL-STRUCTS: Error types follow canonical structure
- M-STATIC-VERIFICATION: All lints enabled, clippy used
- M-PUBLIC-DEBUG: All public types will implement Debug
- M-FIRST-DOC-SENTENCE: Documentation first sentences under 15 words

**Documentation Standards:**
- Diátaxis Type: Reference documentation for APIs
- Quality: Professional tone, no marketing hyperbole per documentation-quality-standards.md
- Compliance: Standards Compliance Checklist will be included in task file

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

**Objective:** Implement bridge to airssys-osl SecurityContext using correct API patterns

**File:** `airssys-wasm/src/security/mod.rs`

**Specification (Corrected API Usage):**

```rust
//! Security module for capability-based access control.
//!
//! This module is **Layer 2A** of the architecture and provides
//! integration with airssys-osl's security infrastructure.

// Layer 1: Standard library imports
use std::collections::HashMap;

// Layer 2: Third-party crate imports
use airssys_osl::core::context::SecurityContext;
use airssys_osl::middleware::security::policy::{PolicyDecision, SecurityPolicy};
use airssys_osl::middleware::security::acl::{ATTR_ACL_PERMISSION, ATTR_ACL_RESOURCE};

// Layer 3: Internal module imports
use crate::core::security::errors::SecurityError;

pub mod audit;
pub mod capability;
pub mod policy;

/// Bridge to airssys-osl security infrastructure.
///
/// Wraps an airssys-osl security policy and provides convenient methods
/// for checking permissions in the WASM component context.
#[derive(Debug)]
pub struct OslSecurityBridge {
    /// Underlying security policy (e.g., AccessControlList)
    policy: Box<dyn SecurityPolicy>,
}

impl OslSecurityBridge {
    /// Creates a new security bridge with the given policy.
    ///
    /// # Arguments
    /// * `policy` - Security policy implementation (e.g., AccessControlList)
    pub fn new(policy: Box<dyn SecurityPolicy>) -> Self {
        Self { policy }
    }

    /// Checks if a principal has permission to perform an action on a resource.
    ///
    /// # Arguments
    /// * `principal` - The identity requesting access (e.g., component ID)
    /// * `resource` - The resource being accessed (e.g., file path, API endpoint)
    /// * `permission` - The permission being requested (e.g., "read", "write")
    ///
    /// # Errors
    /// Returns `SecurityError::PermissionDenied` if the policy denies access.
    ///
    /// # Examples
    /// ```ignore
    /// use airssys_osl::middleware::security::acl::AccessControlList;
    /// use airssys_wasm::security::OslSecurityBridge;
    ///
    /// let acl = AccessControlList::new();
    /// let bridge = OslSecurityBridge::new(Box::new(acl));
    ///
    /// match bridge.check_permission("component-123", "/data/file.txt", "read") {
    ///     Ok(()) => println!("Access granted"),
    ///     Err(e) => println!("Access denied: {}", e),
    /// }
    /// ```
    pub fn check_permission(
        &self,
        principal: &str,
        resource: &str,
        permission: &str,
    ) -> Result<(), SecurityError> {
        // Build security context with ACL attributes
        let mut attributes = HashMap::new();
        attributes.insert(ATTR_ACL_RESOURCE.to_string(), resource.to_string());
        attributes.insert(ATTR_ACL_PERMISSION.to_string(), permission.to_string());

        let context = SecurityContext {
            principal: principal.to_string(),
            session_id: None,
            established_at: chrono::Utc::now(),
            attributes,
        };

        // Evaluate using airssys-osl SecurityPolicy trait
        match self.policy.evaluate(&context) {
            PolicyDecision::Allow => Ok(()),
            PolicyDecision::Deny(reason) => Err(SecurityError::PermissionDenied(format!(
                "OSL denied: {} cannot {} on {}: {}",
                principal, permission, resource, reason
            ))),
            PolicyDecision::RequireAdditionalAuth(auth) => Err(SecurityError::PermissionDenied(format!(
                "OSL requires additional authentication: {:?}", auth
            ))),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use airssys_osl::middleware::security::acl::{AccessControlList, AclEntry, AclPolicy};

    #[test]
    fn test_bridge_creation() {
        let acl = AccessControlList::new();
        let bridge = OslSecurityBridge::new(Box::new(acl));
        
        // Verify bridge holds the policy
        assert_eq!(bridge.policy.evaluate(&context_for_test("user", "/file", "read")), PolicyDecision::Deny("No matching ACL entry".to_string()));
    }

    #[test]
    fn test_permitted_action() {
        // Create ACL with allow entry
        let mut acl = AccessControlList::new();
        acl = acl.add_entry(AclEntry::new(
            "component-123".to_string(),
            "/data/*".to_string(),
            vec!["read".to_string()],
            AclPolicy::Allow,
        ));

        let bridge = OslSecurityBridge::new(Box::new(acl));

        // Should succeed
        let result = bridge.check_permission("component-123", "/data/file.txt", "read");
        assert!(result.is_ok(), "Expected success, got: {:?}", result);
    }

    #[test]
    fn test_denied_action() {
        // Empty ACL - deny by default
        let acl = AccessControlList::new();
        let bridge = OslSecurityBridge::new(Box::new(acl));

        // Should fail
        let result = bridge.check_permission("component-456", "/secret/file.txt", "write");
        assert!(result.is_err());
        assert!(matches!(result, Err(SecurityError::PermissionDenied(_))));
    }

    #[test]
    fn test_error_message_formatting() {
        let acl = AccessControlList::new();
        let bridge = OslSecurityBridge::new(Box::new(acl));

        let result = bridge.check_permission("user-123", "/path", "action");
        let err = result.unwrap_err();
        
        // Verify error message contains key information
        let err_msg = format!("{:?}", err);
        assert!(err_msg.contains("OSL denied"));
        assert!(err_msg.contains("user-123"));
        assert!(err_msg.contains("path"));
        assert!(err_msg.contains("action"));
    }

    #[test]
    fn test_principal_mismatch() {
        let mut acl = AccessControlList::new();
        acl = acl.add_entry(AclEntry::new(
            "component-allowed".to_string(),
            "/data/*".to_string(),
            vec!["read".to_string()],
            AclPolicy::Allow,
        ));

        let bridge = OslSecurityBridge::new(Box::new(acl));

        // Different principal - should be denied
        let result = bridge.check_permission("component-different", "/data/file.txt", "read");
        assert!(result.is_err());
    }

    // Helper function for building test contexts
    fn context_for_test(principal: &str, resource: &str, permission: &str) -> SecurityContext {
        let mut attributes = HashMap::new();
        attributes.insert(ATTR_ACL_RESOURCE.to_string(), resource.to_string());
        attributes.insert(ATTR_ACL_PERMISSION.to_string(), permission.to_string());

        SecurityContext {
            principal: principal.to_string(),
            session_id: None,
            established_at: chrono::Utc::now(),
            attributes,
        }
    }
}
```

**Key Changes from Original Plan:**
1. ✅ Uses correct API: `SecurityPolicy::evaluate()` instead of non-existent `is_permitted()`
2. ✅ Holds `Box<dyn SecurityPolicy>` (AccessControlList or custom policy)
3. ✅ Uses `ATTR_ACL_RESOURCE` and `ATTR_ACL_PERMISSION` constants
4. ✅ Handles all `PolicyDecision` variants (Allow, Deny, RequireAdditionalAuth)
5. ✅ Follows 3-layer import organization (§2.1)
6. ✅ Comprehensive documentation with examples (M-MODULE-DOCS, M-FIRST-DOC-SENTENCE)
7. ✅ Detailed unit tests with actual assertions

---

### Action 2: Add Integration Tests for OSL Security

**Objective:** Create end-to-end integration tests for OSL security integration

**File:** `airssys-wasm/tests/osl-security-integration-tests.rs`

**Integration Test Specification:**

```rust
//! Integration tests for airssys-osl security integration.
//!
//! Tests end-to-end permission checks through OslSecurityBridge
//! with realistic ACL scenarios.

use airssys_wasm::security::OslSecurityBridge;
use airssys_osl::middleware::security::acl::{
    AccessControlList, AclEntry, AclPolicy,
    ATTR_ACL_RESOURCE, ATTR_ACL_PERMISSION,
};
use airssys_osl::core::context::SecurityContext;
use std::collections::HashMap;

#[test]
fn test_filesystem_access_control() {
    // Scenario: Component with filesystem read capability
    let mut acl = AccessControlList::new();
    
    // Grant read access to /app/data/*
    acl = acl.add_entry(AclEntry::new(
        "component-fs-001".to_string(),
        "/app/data/*".to_string(),
        vec!["read".to_string()],
        AclPolicy::Allow,
    ));

    let bridge = OslSecurityBridge::new(Box::new(acl));

    // Test: Allowed paths
    assert!(bridge.check_permission("component-fs-001", "/app/data/config.txt", "read").is_ok());
    assert!(bridge.check_permission("component-fs-001", "/app/data/subdir/file.txt", "read").is_ok());

    // Test: Denied operations
    assert!(bridge.check_permission("component-fs-001", "/app/data/secret.txt", "write").is_err());
    assert!(bridge.check_permission("component-fs-001", "/app/config/settings.txt", "read").is_err());
}

#[test]
fn test_network_access_control() {
    // Scenario: Component with network connect capability
    let mut acl = AccessControlList::new();
    
    acl = acl.add_entry(AclEntry::new(
        "component-net-002".to_string(),
        "api.example.com:443".to_string(),
        vec!["connect".to_string()],
        AclPolicy::Allow,
    ));

    let bridge = OslSecurityBridge::new(Box::new(acl));

    // Test: Allowed connections
    assert!(bridge.check_permission("component-net-002", "api.example.com:443", "connect").is_ok());

    // Test: Denied connections
    assert!(bridge.check_permission("component-net-002", "other-api.com:443", "connect").is_err());
    assert!(bridge.check_permission("component-net-002", "api.example.com:80", "connect").is_err());
}

#[test]
fn test_component_isolation() {
    // Scenario: Multiple components with different access levels
    let mut acl = AccessControlList::new();
    
    // Component A: Can read /app/public/*
    acl = acl.add_entry(AclEntry::new(
        "component-a".to_string(),
        "/app/public/*".to_string(),
        vec!["read".to_string()],
        AclPolicy::Allow,
    ));

    // Component B: Can read/write /app/private/*
    acl = acl.add_entry(AclEntry::new(
        "component-b".to_string(),
        "/app/private/*".to_string(),
        vec!["read".to_string(), "write".to_string()],
        AclPolicy::Allow,
    ));

    let bridge = OslSecurityBridge::new(Box::new(acl));

    // Test: Component A access
    assert!(bridge.check_permission("component-a", "/app/public/file.txt", "read").is_ok());
    assert!(bridge.check_permission("component-a", "/app/private/secret.txt", "read").is_err());

    // Test: Component B access
    assert!(bridge.check_permission("component-b", "/app/private/data.txt", "read").is_ok());
    assert!(bridge.check_permission("component-b", "/app/private/data.txt", "write").is_ok());
    assert!(bridge.check_permission("component-b", "/app/public/file.txt", "read").is_err());
}

#[test]
fn test_deny_by_default_behavior() {
    // Scenario: New component with no ACL entries
    let acl = AccessControlList::new();
    let bridge = OslSecurityBridge::new(Box::new(acl));

    // All operations should be denied
    assert!(bridge.check_permission("new-component", "/any/resource", "any-action").is_err());
    assert!(bridge.check_permission("new-component", "/app/data", "read").is_err());
    assert!(bridge.check_permission("new-component", "localhost:8080", "connect").is_err());
}

#[test]
fn test_pattern_matching_glob_patterns() {
    // Scenario: Component with wildcard patterns
    let mut acl = AccessControlList::new();
    
    // Grant access to all logs
    acl = acl.add_entry(AclEntry::new(
        "component-logger".to_string(),
        "/var/log/**/*.log".to_string(),
        vec!["read".to_string()],
        AclPolicy::Allow,
    ));

    let bridge = OslSecurityBridge::new(Box::new(acl));

    // Test: Pattern matching
    assert!(bridge.check_permission("component-logger", "/var/log/app.log", "read").is_ok());
    assert!(bridge.check_permission("component-logger", "/var/log/subdir/error.log", "read").is_ok());
    assert!(bridge.check_permission("component-logger", "/var/log/subdir/nested/debug.log", "read").is_ok());
    
    // Test: Non-matching paths
    assert!(bridge.check_permission("component-logger", "/var/log/config.txt", "read").is_err());
    assert!(bridge.check_permission("component-logger", "/var/data/file.log", "read").is_err());
}

#[test]
fn test_multiple_permissions() {
    // Scenario: Component with multiple permissions on same resource
    let mut acl = AccessControlList::new();
    
    acl = acl.add_entry(AclEntry::new(
        "component-multi".to_string(),
        "/app/shared/*".to_string(),
        vec!["read".to_string(), "write".to_string(), "delete".to_string()],
        AclPolicy::Allow,
    ));

    let bridge = OslSecurityBridge::new(Box::new(acl));

    // Test: All allowed permissions
    assert!(bridge.check_permission("component-multi", "/app/shared/data.txt", "read").is_ok());
    assert!(bridge.check_permission("component-multi", "/app/shared/data.txt", "write").is_ok());
    assert!(bridge.check_permission("component-multi", "/app/shared/data.txt", "delete").is_ok());
    
    // Test: Other permissions denied
    assert!(bridge.check_permission("component-multi", "/app/shared/data.txt", "execute").is_err());
}

#[test]
fn test_security_context_attributes() {
    // Scenario: Verify context attributes are passed correctly
    let mut acl = AccessControlList::new();
    
    acl = acl.add_entry(AclEntry::new(
        "component-ctx".to_string(),
        "/resource".to_string(),
        vec!["action".to_string()],
        AclPolicy::Allow,
    ));

    let bridge = OslSecurityBridge::new(Box::new(acl));

    // Build context manually and verify
    let mut attributes = HashMap::new();
    attributes.insert(ATTR_ACL_RESOURCE.to_string(), "/resource".to_string());
    attributes.insert(ATTR_ACL_PERMISSION.to_string(), "action".to_string());

    let context = SecurityContext {
        principal: "component-ctx".to_string(),
        session_id: None,
        established_at: chrono::Utc::now(),
        attributes,
    };

    // Verify context attributes are used in evaluation
    let decision = bridge.policy.evaluate(&context);
    assert!(matches!(decision, PolicyDecision::Allow));
}
```

**Key Changes from Original Plan:**
1. ✅ New integration test file: `tests/osl-security-integration-tests.rs`
2. ✅ End-to-end scenarios: filesystem, network, isolation, patterns
3. ✅ Realistic ACL configurations with actual assertions
4. ✅ Tests for deny-by-default behavior
5. ✅ Glob pattern matching validation
6. ✅ Multiple permissions testing
7. ✅ Security context attribute verification

---

### Action 3: Verify airssys-osl Dependency

**Objective:** Ensure Cargo.toml has airssys-osl dependency

**File to Check:** `airssys-wasm/Cargo.toml`

**Expected Content:**
```toml
[dependencies]
airssys-osl = { workspace = true }
```

**Verification:**
```bash
grep "airssys-osl" airssys-wasm/Cargo.toml
```

**Expected Output:** Should show the dependency is present.

---

## Verification Commands

Run after ALL actions complete:

```bash
# 1. Build check
cargo build -p airssys-wasm

# 2. Module architecture verification (ADR-WASM-023)
# security/ CANNOT import from runtime/ or actor/
grep -rn "use crate::runtime" airssys-wasm/src/security/
grep -rn "use crate::actor" airssys-wasm/src/security/
# Expected: No output (clean architecture)

# 3. Lint check
cargo clippy -p airssys-wasm --all-targets -- -D warnings

# 4. Run security module unit tests
cargo test -p airssys-wasm --lib security

# 5. Run integration tests
cargo test -p airssys-wasm --test osl-security-integration-tests
```

---

## Success Criteria

- [ ] OslSecurityBridge wraps SecurityPolicy correctly
- [ ] Uses correct API: `policy.evaluate(&SecurityContext)` (not is_permitted)
- [ ] Uses ATTR_ACL_RESOURCE and ATTR_ACL_PERMISSION constants
- [ ] Build passes with zero warnings
- [ ] Integration with airssys-osl works end-to-end
- [ ] All unit tests pass (6 tests with detailed assertions)
- [ ] All integration tests pass (8 scenarios)
- [ ] Module boundary compliance verified (no forbidden imports)
- [ ] Documentation follows M-MODULE-DOCS and M-FIRST-DOC-SENTENCE
- [ ] Code follows 3-layer import organization (§2.1)
- [ ] Public types implement Debug (M-PUBLIC-DEBUG)
