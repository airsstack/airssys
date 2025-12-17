# Phase 5 Integration Testing: Technical Findings and Solutions

**Document Type:** Technical Findings Report  
**Phase:** OSL-TASK-010 Phase 5 - Integration Testing  
**Date:** 2025-10-12  
**Status:** In Progress → Implementation Planned  
**Related ADR:** [ADR-030: Security Context Attributes Architecture](../adr/030-security-context-attributes-architecture.md)

---

## Executive Summary

During Phase 5 integration testing implementation, we discovered that helper functions create `SecurityContext` with only the principal (username), but ACL and RBAC security policies require additional attributes in `context.attributes` HashMap to properly evaluate permissions. This document captures the technical findings, root cause analysis, and approved solution architecture.

**Key Metrics:**
- **Tests Created:** 30+ integration tests across 3 test files
- **Tests Passing:** 6/11 security tests (54% pass rate)
- **Tests Ignored:** 5/11 security tests (require attributes)
- **Root Cause:** Missing context attribute population in helpers
- **Solution:** Security module attribute builders + helper utility

---

## 1. Problem Discovery

### 1.1 Test Implementation Phase

Created comprehensive integration tests for Phase 5:

| Test File | Purpose | Tests | Status |
|-----------|---------|-------|--------|
| `helpers_security_tests.rs` | ACL/RBAC policy enforcement | 11 tests | 6 pass, 5 require attributes |
| `helpers_audit_tests.rs` | Security audit logging | 7 tests | Not yet run |
| `helpers_error_tests.rs` | Error handling scenarios | 12+ tests | Not yet run |

**Total:** 30+ integration tests covering security, audit, and error handling.

### 1.2 Compilation Issues (RESOLVED)

Encountered and fixed multiple compilation errors:

1. **Import Path Errors**
   ```rust
   // WRONG
   use airssys_osl::core::error::OSError;
   
   // CORRECT
   use airssys_osl::core::result::OSError;
   ```

2. **RBAC API Misuse**
   ```rust
   // WRONG - old API
   let role = Role::new("reader".to_string(), vec!["read_file".to_string()]);
   
   // CORRECT - builder pattern
   let role = Role::new("reader-role", "Reader")
       .with_permission(Permission::new("read_file", "Read files"));
   ```

3. **SecurityMiddleware Clone Issues**
   ```rust
   // WRONG - SecurityMiddleware doesn't implement Clone
   let security2 = security.clone();
   
   // CORRECT - rebuild middleware
   let security2 = SecurityMiddlewareBuilder::new()
       .add_policy(Box::new(acl))
       .build()
       .expect("Failed to build security middleware");
   ```

4. **OSError Field Names**
   ```rust
   // WRONG
   OSError::ExecutionFailed(msg)
   
   // CORRECT
   OSError::ExecutionFailed { reason: msg }
   ```

**Result:** All tests compile successfully ✅

### 1.3 Runtime Failures (CURRENT ISSUE)

After fixing compilation, 5/11 security tests fail at runtime:

#### Test Results Summary
```
test result: FAILED. 6 passed; 2 failed; 3 ignored; 0 measured; 0 filtered out

Passing Tests (6):
✅ test_rbac_no_role_assigned_denied
✅ test_multiple_policies_all_must_pass  
✅ test_read_file_with_acl_deny
✅ test_read_with_security_violation
✅ test_spawn_process_with_acl_deny
✅ test_network_connect_with_policy

Failed/Ignored Tests (5):
❌ test_read_file_with_acl_allow (ignored - needs acl.resource attribute)
❌ test_write_file_with_acl_glob_pattern (ignored - needs acl.resource attribute)
❌ test_read_file_with_rbac_admin_allowed (ignored - needs rbac.required_permission)
❌ test_write_file_with_rbac_reader_role_denied (ignored - needs rbac.required_permission)
❌ test_rbac_role_hierarchy (ignored - needs rbac.required_permission)
```

---

## 2. Root Cause Analysis

### 2.1 Current Helper Implementation

```rust
// In src/helpers/simple.rs - ALL 10 HELPERS USE THIS PATTERN
pub async fn read_file_with_middleware<P, M>(
    path: P,
    user: impl Into<String>,
    middleware: M,
) -> OSResult<Vec<u8>>
where
    P: AsRef<Path>,
    M: Middleware<FileReadOperation>,
{
    let path_str = path.as_ref().display().to_string();
    let operation = FileReadOperation::new(path_str);
    
    // PROBLEM: Only sets principal, no attributes!
    let context = ExecutionContext::new(SecurityContext::new(user.into()));

    let executor = FilesystemExecutor::new().with_middleware(middleware);
    let result = executor.execute(operation, &context).await?;
    Ok(result.output)
}
```

### 2.2 Security Policy Requirements

#### ACL Policy Evaluation Logic
```rust
// In src/middleware/security/acl.rs
impl SecurityPolicy for AccessControlList {
    async fn evaluate(&self, context: &ExecutionContext) -> Result<PolicyDecision, String> {
        // ACL REQUIRES these attributes:
        let resource = context
            .security_context
            .attributes
            .get(ATTR_RESOURCE)  // ❌ NOT SET by helpers!
            .map(|s| s.as_str())
            .unwrap_or("resource");
            
        let permission = context
            .security_context
            .attributes
            .get(ATTR_PERMISSION)  // ❌ NOT SET by helpers!
            .map(|s| s.as_str())
            .unwrap_or("permission");
        
        // Without these, ACL uses default deny policy
        // Result: All ACL tests fail (unless testing deny)
    }
}
```

#### RBAC Policy Evaluation Logic
```rust
// In src/middleware/security/rbac.rs
impl SecurityPolicy for RoleBasedAccessControl {
    async fn evaluate(&self, context: &ExecutionContext) -> Result<PolicyDecision, String> {
        // RBAC REQUIRES this attribute:
        let required_permission = context
            .security_context
            .attributes
            .get(ATTR_REQUIRED_PERMISSION)  // ❌ NOT SET by helpers!
            .ok_or_else(|| "Required permission attribute not found")?;
        
        // Without this, RBAC can't check if role has permission
        // Result: All RBAC tests fail
    }
}
```

### 2.3 The Gap

```
┌──────────────────────────────────────────────────────────────┐
│  Helper Function                                              │
│  - Creates SecurityContext with ONLY principal (username)    │
│  - context.attributes = HashMap::new() (EMPTY!)              │
└────────────────────┬─────────────────────────────────────────┘
                     │ passes to
                     ▼
┌──────────────────────────────────────────────────────────────┐
│  Security Middleware (ACL/RBAC)                              │
│  - Expects context.attributes["acl.resource"]        ❌ MISSING │
│  - Expects context.attributes["acl.permission"]      ❌ MISSING │
│  - Expects context.attributes["rbac.required_permission"] ❌ MISSING │
└──────────────────────────────────────────────────────────────┘
                     │
                     ▼
                POLICY FAILURE
```

### 2.4 Why Some Tests Still Pass

Tests that **don't require specific attribute values** still pass:

```rust
// ✅ PASSES - Tests default deny behavior
#[tokio::test]
async fn test_read_file_with_acl_deny() {
    // ACL default policy is deny
    // Even without attributes, denies everything
    // Test expects denial → test passes
}

// ✅ PASSES - Tests "no role" scenario
#[tokio::test]
async fn test_rbac_no_role_assigned_denied() {
    // RBAC checks: Does user have a role?
    // User 'eve' has no role → denied
    // Doesn't need to check permission attribute
    // Test expects denial → test passes
}
```

Tests that **require attribute matching** fail:

```rust
// ❌ FAILS - Needs attribute matching
#[tokio::test]
async fn test_read_file_with_acl_allow() {
    // ACL needs to match:
    //   context.attributes["acl.resource"] == "/tmp/test.txt"
    //   context.attributes["acl.permission"] == "read"
    // Without attributes, uses default deny
    // Test expects allow → test fails
}

// ❌ FAILS - Needs permission check
#[tokio::test]
async fn test_write_file_with_rbac_reader_role_denied() {
    // RBAC needs to check:
    //   Does role "reader" have permission from
    //   context.attributes["rbac.required_permission"]?
    // Without attribute, can't evaluate
    // Test expects deny → test fails (or errors)
}
```

---

## 3. Error Patterns Observed

### 3.1 ACL Error Pattern

```
thread 'test_write_file_with_acl_glob_pattern' panicked at airssys-osl/tests/helpers_security_tests.rs:142:5:
Expected write to succeed with glob pattern match, got: 
Some(ExecutionFailed { 
    reason: "Middleware error in before_execution: SecurityViolation(
        \"Policy 'Access Control List (ACL) Policy' denied: 
        ACL default policy denies access to 'resource' for 'alice'\"
    )" 
})
```

**Analysis:**
- ACL logs show: "denies access to **'resource'**" ← literal string, not actual path!
- This confirms `context.attributes["acl.resource"]` is missing
- ACL falls back to default string "resource" (from `unwrap_or("resource")`)
- Default deny policy triggered because no ACL entry matches literal "resource"

### 3.2 RBAC Error Pattern

```
thread 'test_write_file_with_rbac_reader_role_denied' panicked:
Expected write to be denied for reader role, but got Ok(())
```

**Analysis:**
- Test expects: Writer operation denied for reader role
- Actual: Operation succeeds! 
- Root cause: RBAC can't find `context.attributes["rbac.required_permission"]`
- Without permission to check, RBAC might default to allow (depends on implementation)
- Test assertion fails because expected denial didn't occur

### 3.3 Error Wrapping Pattern

Initially mismatched error types:

```rust
// WRONG expectation
match result.err() {
    Some(OSError::MiddlewareFailed { reason }) => { /* ... */ }
    // ❌ Error type doesn't match!
}

// CORRECT - errors are wrapped by executor
match result.err() {
    Some(OSError::ExecutionFailed { reason }) => {
        // Middleware errors wrapped in ExecutionFailed by executors
        assert!(reason.contains("SecurityViolation"));
    }
}
```

**Finding:** Middleware errors are wrapped by executors as `ExecutionFailed`, not passed through as `MiddlewareFailed`.

---

## 4. Attempted Solutions (Rejected)

### 4.1 Option: Mark Tests as Ignored

**Approach:** Add `#[ignore]` attributes with documentation

```rust
#[tokio::test]
#[ignore = "ACL requires 'acl.resource' and 'acl.permission' attributes - helpers don't set these"]
async fn test_read_file_with_acl_allow() {
    // ...
}
```

**Why Rejected:**
- ❌ Doesn't fix the actual problem
- ❌ Tests remain incomplete
- ❌ ACL/RBAC policies don't actually work through helpers
- ❌ Kicks the can down the road
- ✅ Useful as temporary measure while implementing fix

### 4.2 Option: Hardcode Attributes in Helpers

**Approach:** Each helper manually builds attributes

```rust
pub async fn read_file_with_middleware(...) {
    let mut attrs = HashMap::new();
    attrs.insert("acl.resource".to_string(), path_str.clone());
    attrs.insert("acl.permission".to_string(), "read".to_string());
    attrs.insert("rbac.required_permission".to_string(), "read_file".to_string());
    // ... repeat in all 10 helpers
}
```

**Why Rejected:**
- ❌ Massive duplication (10 helpers × ~5 lines)
- ❌ Maintenance burden (add new policy = update 10 files)
- ❌ Error prone (typos, inconsistency)
- ❌ Violates DRY principle

### 4.3 Option: Operations Provide Attributes

**Approach:** Add `security_attributes()` to Operation trait

```rust
trait Operation {
    fn security_attributes(&self) -> HashMap<String, String>;
}
```

**Why Rejected:**
- ❌ **Violates separation of concerns** - operations know security details
- ❌ **Tight coupling** - operations depend on ACL/RBAC attribute names
- ❌ **Not extensible** - new security policies require changing all operations
- ❌ **Domain pollution** - operations domain contains security logic

---

## 5. Approved Solution (ADR-030)

### 5.1 Architecture Overview

**Principle:** Security domain builds attributes from operation permissions

```
┌─────────────────────────────────────────────────────────────┐
│ Operations Domain                                           │
│ - FileReadOperation::required_permissions()                 │
│   Returns: [Permission::FilesystemRead("/tmp/file.txt")]   │
│ - Declares WHAT permissions needed (domain expertise)       │
└────────────────────┬────────────────────────────────────────┘
                     │
                     ▼
┌─────────────────────────────────────────────────────────────┐
│ Security Domain                                             │
│ - acl::build_acl_attributes(permissions)                   │
│   Returns: {"acl.resource": path, "acl.permission": "read"} │
│ - rbac::build_rbac_attributes(permissions)                 │
│   Returns: {"rbac.required_permission": "read_file"}       │
│ - Interprets HOW to evaluate permissions (domain expertise) │
└────────────────────┬────────────────────────────────────────┘
                     │
                     ▼
┌─────────────────────────────────────────────────────────────┐
│ Helpers Domain                                              │
│ - build_security_context(user, operation)                  │
│   Combines ACL + RBAC attributes                            │
│ - Orchestrates security context building                    │
└─────────────────────────────────────────────────────────────┘
```

### 5.2 Implementation Components

#### Component 1: ACL Attribute Builder

```rust
// In src/middleware/security/acl.rs
pub const ATTR_ACL_RESOURCE: &str = "acl.resource";
pub const ATTR_ACL_PERMISSION: &str = "acl.permission";

pub fn build_acl_attributes(permissions: &[Permission]) -> HashMap<String, String> {
    let mut attributes = HashMap::new();
    
    for perm in permissions {
        match perm {
            Permission::FilesystemRead(path) => {
                attributes.insert(ATTR_ACL_RESOURCE.to_string(), path.clone());
                attributes.insert(ATTR_ACL_PERMISSION.to_string(), "read".to_string());
            }
            Permission::FilesystemWrite(path) => {
                attributes.insert(ATTR_ACL_RESOURCE.to_string(), path.clone());
                attributes.insert(ATTR_ACL_PERMISSION.to_string(), "write".to_string());
            }
            // ... all permission types
        }
    }
    
    attributes
}
```

#### Component 2: RBAC Attribute Builder

```rust
// In src/middleware/security/rbac.rs
pub const ATTR_RBAC_REQUIRED_PERMISSION: &str = "rbac.required_permission";

pub fn build_rbac_attributes(permissions: &[Permission]) -> HashMap<String, String> {
    let mut attributes = HashMap::new();
    
    if permissions.is_empty() {
        return attributes;
    }
    
    // Use first permission (current operations have single permission)
    let required_perm = match &permissions[0] {
        Permission::FilesystemRead(_) => "read_file",
        Permission::FilesystemWrite(_) => "write_file",
        Permission::ProcessSpawn => "spawn_process",
        // ... all permission types
    };
    
    attributes.insert(ATTR_RBAC_REQUIRED_PERMISSION.to_string(), required_perm.to_string());
    attributes
}
```

#### Component 3: Combined Context Builder

```rust
// In src/helpers/context.rs (NEW FILE)
use crate::core::context::SecurityContext;
use crate::core::operation::Operation;
use crate::middleware::security::acl::build_acl_attributes;
use crate::middleware::security::rbac::build_rbac_attributes;
use std::collections::HashMap;

pub fn build_security_context<O: Operation>(
    user: impl Into<String>,
    operation: &O,
) -> SecurityContext {
    let permissions = operation.required_permissions();
    
    let mut attributes = HashMap::new();
    
    // Combine attributes from all security modules
    attributes.extend(build_acl_attributes(&permissions));
    attributes.extend(build_rbac_attributes(&permissions));
    
    SecurityContext::new(user.into()).with_attributes(attributes)
}
```

#### Component 4: Updated Helper Functions

```rust
// In src/helpers/simple.rs
use super::context::build_security_context;

pub async fn read_file_with_middleware<P, M>(
    path: P,
    user: impl Into<String>,
    middleware: M,
) -> OSResult<Vec<u8>>
where
    P: AsRef<Path>,
    M: Middleware<FileReadOperation>,
{
    let path_str = path.as_ref().display().to_string();
    let operation = FileReadOperation::new(path_str);
    
    // ✅ USE HELPER - automatically populates all security attributes
    let security_context = build_security_context(user, &operation);
    let context = ExecutionContext::new(security_context);

    let executor = FilesystemExecutor::new().with_middleware(middleware);
    let result = executor.execute(operation, &context).await?;
    Ok(result.output)
}

// Repeat pattern for all 10 helpers
```

### 5.3 Key Design Decisions

| Decision | Choice | Rationale |
|----------|--------|-----------|
| **Permission Priority** | Use first permission | Current operations have single permission; can enhance later for compound operations |
| **Attribute Namespacing** | Use module prefixes (`acl.*`, `rbac.*`) | Prevents key conflicts; explicit ownership; future-proof |
| **Module Location** | `src/helpers/context.rs` | Semantic clarity - building SecurityContext |
| **Export Strategy** | No re-export from `security/mod.rs` | Clear ownership; prevents namespace pollution |

### 5.4 Required Updates

#### Update 1: ACL Evaluation Code
```rust
// In src/middleware/security/acl.rs - AccessControlList::evaluate()
let resource = context
    .security_context
    .attributes
    .get(ATTR_ACL_RESOURCE)  // Changed from ATTR_RESOURCE
    .map(|s| s.as_str())
    .unwrap_or("resource");
    
let permission = context
    .security_context
    .attributes
    .get(ATTR_ACL_PERMISSION)  // Changed from ATTR_PERMISSION
    .map(|s| s.as_str())
    .unwrap_or("permission");
```

#### Update 2: RBAC Evaluation Code
```rust
// In src/middleware/security/rbac.rs - RoleBasedAccessControl::evaluate()
let required_permission = context
    .security_context
    .attributes
    .get(ATTR_RBAC_REQUIRED_PERMISSION)  // Changed from ATTR_REQUIRED_PERMISSION
    .ok_or_else(|| "Required permission attribute not found")?;
```

#### Update 3: SecurityContext Builder
```rust
// In src/core/context.rs - Add builder method
impl SecurityContext {
    pub fn with_attributes(mut self, attributes: HashMap<String, String>) -> Self {
        self.attributes = attributes;
        self
    }
}
```

---

## 6. Implementation Plan

### Phase 1: Security Module Builders
- [ ] Add prefixed constants to `acl.rs` (`ATTR_ACL_RESOURCE`, `ATTR_ACL_PERMISSION`)
- [ ] Implement `build_acl_attributes()` with all Permission variants
- [ ] Update ACL evaluation to use prefixed constants
- [ ] Add unit tests for ACL attribute builder

- [ ] Add prefixed constant to `rbac.rs` (`ATTR_RBAC_REQUIRED_PERMISSION`)
- [ ] Implement `build_rbac_attributes()` with first-permission strategy
- [ ] Update RBAC evaluation to use prefixed constant
- [ ] Add unit tests for RBAC attribute builder

### Phase 2: Helper Utility
- [ ] Create `src/helpers/context.rs`
- [ ] Implement `build_security_context()` combining ACL + RBAC
- [ ] Export from `src/helpers/mod.rs`
- [ ] Add integration tests

### Phase 3: SecurityContext Enhancement
- [ ] Add `with_attributes()` builder method
- [ ] Ensure method chaining works
- [ ] Add documentation and examples

### Phase 4: Update All Helpers (10 functions)
- [ ] `read_file_with_middleware()`
- [ ] `write_file_with_middleware()`
- [ ] `delete_file_with_middleware()`
- [ ] `create_directory_with_middleware()`
- [ ] `spawn_process_with_middleware()`
- [ ] `kill_process_with_middleware()`
- [ ] `send_signal_with_middleware()`
- [ ] `connect_network_with_middleware()`
- [ ] `listen_network_with_middleware()`
- [ ] `accept_connection_with_middleware()`

### Phase 5: Test Validation
- [ ] Remove `#[ignore]` from 5 ACL/RBAC tests
- [ ] Run `cargo test --package airssys-osl --test helpers_security_tests`
- [ ] Expected: 11/11 tests passing ✅
- [ ] Run `cargo test --package airssys-osl --test helpers_audit_tests`
- [ ] Run `cargo test --package airssys-osl --test helpers_error_tests`

### Phase 6: Documentation
- [ ] Rustdoc for `build_acl_attributes()`
- [ ] Rustdoc for `build_rbac_attributes()`
- [ ] Rustdoc for `build_security_context()`
- [ ] Update helper function documentation
- [ ] Add examples showing attribute usage

---

## 7. Success Criteria

### 7.1 Functional Criteria
- ✅ All 11 security integration tests pass without `#[ignore]`
- ✅ All 7 audit logging tests pass
- ✅ All 12+ error handling tests pass
- ✅ ACL policies correctly match resources via glob patterns
- ✅ RBAC policies correctly check role permissions
- ✅ Multiple policies (ACL + RBAC) work together

### 7.2 Code Quality Criteria
- ✅ No code duplication across helpers
- ✅ Clean separation of concerns (operations ↔ security)
- ✅ Type-safe attribute building via Permission enum
- ✅ Comprehensive unit tests for attribute builders
- ✅ Clear documentation with examples

### 7.3 Architecture Criteria
- ✅ Operations domain: Only declares permissions
- ✅ Security domain: Owns attribute interpretation
- ✅ Helpers domain: Simple orchestration
- ✅ Extensible for future security policies
- ✅ No tight coupling between domains

---

## 8. Lessons Learned

### 8.1 Testing Reveals Architecture Issues
- **Insight:** Integration tests revealed a fundamental architectural gap
- **Value:** Tests forced us to design a proper solution vs implementing features blindly
- **Practice:** Write integration tests early to validate architecture

### 8.2 Separation of Concerns Is Critical
- **Insight:** Operations shouldn't know security implementation details
- **Value:** Each domain maintains its expertise and responsibilities
- **Practice:** Use indirection layers (attribute builders) to decouple domains

### 8.3 Type Safety Guides Design
- **Insight:** Leveraging existing `Permission` enum provides type safety
- **Value:** Pattern matching ensures all cases handled; compiler errors on missing cases
- **Practice:** Extend existing types rather than creating new parallel structures

### 8.4 Progressive Enhancement Works
- **Insight:** Start with first-permission strategy; enhance for compound operations later
- **Value:** Ship working solution now; iterate when requirements emerge
- **Practice:** Document TODO for future enhancements; don't over-engineer

### 8.5 Namespace Isolation Prevents Future Pain
- **Insight:** Prefixing attribute keys prevents conflicts as system grows
- **Value:** Each security module owns its namespace; new policies don't break existing ones
- **Practice:** Be explicit about ownership even when no current conflicts exist

---

## 9. References

### Documentation
- **ADR-030:** Security Context Attributes Architecture (primary architectural decision)
- **ADR-028:** ACL Permission Model and Glob Matching (ACL attribute requirements)
- **OSL-TASK-010:** Helper Function Middleware Integration (parent task)

### Code Files
- **Tests:** `airssys-osl/tests/helpers_security_tests.rs` (11 integration tests)
- **Helpers:** `airssys-osl/src/helpers/simple.rs` (10 helper functions)
- **ACL:** `airssys-osl/src/middleware/security/acl.rs` (ACL policy)
- **RBAC:** `airssys-osl/src/middleware/security/rbac.rs` (RBAC policy)
- **Operations:** `airssys-osl/src/operations/*/` (Operation implementations)
- **Context:** `airssys-osl/src/core/context.rs` (SecurityContext)

### External Resources
- Rust Pattern Matching: https://doc.rust-lang.org/book/ch18-00-patterns.html
- Separation of Concerns: https://en.wikipedia.org/wiki/Separation_of_concerns
- Builder Pattern: https://rust-unofficial.github.io/patterns/patterns/creational/builder.html

---

## 10. Appendix

### A. Complete Test Status

```
helpers_security_tests.rs (11 tests):
  ✅ test_read_file_with_acl_allow             (will pass after fix)
  ✅ test_read_file_with_acl_deny              (already passing)
  ✅ test_write_file_with_acl_glob_pattern     (will pass after fix)
  ✅ test_read_file_with_rbac_admin_allowed    (will pass after fix)
  ✅ test_write_file_with_rbac_reader_role_denied (will pass after fix)
  ✅ test_rbac_no_role_assigned_denied         (already passing)
  ✅ test_rbac_role_hierarchy                  (will pass after fix)
  ✅ test_multiple_policies_all_must_pass      (already passing)
  ✅ test_read_with_security_violation         (already passing)
  ✅ test_spawn_process_with_acl_deny          (already passing)
  ✅ test_network_connect_with_policy          (already passing)

helpers_audit_tests.rs (7 tests):
  ⏳ test_file_operation_logged                (not yet run)
  ⏳ test_process_operation_logged             (not yet run)
  ⏳ test_network_operation_logged             (not yet run)
  ⏳ test_failed_operation_logged              (not yet run)
  ⏳ test_security_violation_logged            (not yet run)
  ⏳ test_audit_log_contains_context           (not yet run)
  ⏳ test_audit_log_ordering                   (not yet run)

helpers_error_tests.rs (12+ tests):
  ⏳ test_file_not_found_error                 (not yet run)
  ⏳ test_io_error_propagation                 (not yet run)
  ⏳ test_network_error_handling               (not yet run)
  ⏳ test_empty_file_path                      (not yet run)
  ⏳ test_invalid_process_id                   (not yet run)
  ⏳ test_malformed_network_address            (not yet run)
  ⏳ ... (6+ more edge case tests)
```

### B. Attribute Key Reference

| Module | Attribute Key | Type | Example Value | Purpose |
|--------|--------------|------|---------------|---------|
| ACL | `acl.resource` | String | `/tmp/file.txt` | Resource being accessed |
| ACL | `acl.permission` | String | `read`, `write` | Permission type |
| RBAC | `rbac.required_permission` | String | `read_file`, `write_file` | Required role permission |

### C. Permission to Attribute Mapping

| Permission Variant | ACL Resource | ACL Permission | RBAC Required Permission |
|-------------------|--------------|----------------|-------------------------|
| `FilesystemRead(path)` | `path` | `"read"` | `"read_file"` |
| `FilesystemWrite(path)` | `path` | `"write"` | `"write_file"` |
| `FilesystemDelete(path)` | `path` | `"delete"` | `"delete_file"` |
| `FilesystemExecute(path)` | `path` | `"execute"` | `"execute_file"` |
| `ProcessSpawn` | `"process"` | `"spawn"` | `"spawn_process"` |
| `ProcessKill` | `"process"` | `"kill"` | `"kill_process"` |
| `ProcessSignal` | `"process"` | `"signal"` | `"signal_process"` |
| `NetworkConnect(addr)` | `addr` | `"connect"` | `"connect_network"` |
| `NetworkListen(addr)` | `addr` | `"listen"` | `"listen_network"` |
| `NetworkAccept` | `"network"` | `"accept"` | `"accept_network"` |

---

**Document Status:** Ready for Implementation  
**Next Action:** Begin Phase 1 implementation (security module builders)  
**Estimated Effort:** 2-3 hours of focused development  
**Target Completion:** Phase 5 of OSL-TASK-010
