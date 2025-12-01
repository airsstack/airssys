# ADR-030: Security Context Attributes Architecture for Helper Functions

**Status:** Accepted  
**Date:** 2025-10-12  
**Deciders:** Architecture Team  
**Tags:** #architecture #security #middleware #helpers #acl #rbac #phase5

---

## Context and Problem Statement

During Phase 5 (Integration Testing) of OSL-TASK-010, integration tests revealed a critical limitation: helper functions create `SecurityContext` with only the principal (username), but ACL and RBAC policies require additional attributes in `context.attributes` HashMap to evaluate permissions properly.

**The Problem:**
```rust
// Current helper implementation - BROKEN
let context = ExecutionContext::new(SecurityContext::new(user.into()));
// Missing: context.attributes["resource"]       - ACL needs this
// Missing: context.attributes["permission"]     - ACL needs this  
// Missing: context.attributes["required_permission"] - RBAC needs this
```

**Test Failures Observed:**
- ACL tests: 3/4 tests ignored - ACL uses default deny policy (can't match resource patterns)
- RBAC tests: 2/4 tests failing - RBAC can't check if role has required permission
- 6/11 security integration tests passing (only tests not requiring attributes)

**Core Question:**
How should helper functions populate SecurityContext attributes while maintaining separation of concerns between operations domain and security domain?

---

## Decision Drivers

### Technical Drivers
- **Separation of Concerns:** Operations should only declare permissions, not security implementation details
- **Domain Expertise:** Security modules (ACL, RBAC) understand how to interpret permissions
- **Extensibility:** New security policies should be able to add their own attribute requirements
- **Type Safety:** Leverage Rust's type system and Operation trait's `required_permissions()`

### Security Requirements
- **ACL Needs:** Resource path (file, network address) and permission type (read, write, execute)
- **RBAC Needs:** Required permission name (read_file, write_file, spawn_process)
- **Both Policies:** Must work simultaneously without attribute conflicts
- **Namespace Isolation:** Each security module owns its attribute keys

### Implementation Constraints
- **10 Helper Functions:** All helpers need consistent attribute building
- **Performance:** Attribute building should be zero-cost abstraction
- **Maintainability:** Adding new security policies shouldn't require changing helpers
- **Backward Compatibility:** Changes shouldn't break existing middleware

---

## Considered Options

### Option 1: Operations Provide Security Attributes (Rejected)
**Approach:** Extend `Operation` trait with `security_attributes()` method

```rust
trait Operation {
    fn security_attributes(&self) -> HashMap<String, String>;
}

impl Operation for FileReadOperation {
    fn security_attributes(&self) -> HashMap<String, String> {
        // Operations know ACL and RBAC requirements
        let mut attrs = HashMap::new();
        attrs.insert("resource", self.path.clone());
        attrs.insert("permission", "read");
        attrs.insert("required_permission", "read_file");
        attrs
    }
}
```

**Pros:**
- Operations self-contained
- Each operation controls its metadata

**Cons:**
- ❌ **Violates separation of concerns** - operations know security implementation
- ❌ **Tight coupling** - operations depend on ACL/RBAC attribute names
- ❌ **Not extensible** - new security policies require modifying all operations
- ❌ **Domain pollution** - operations domain contains security logic

### Option 2: Helper Functions Hardcode Attributes (Rejected)
**Approach:** Each helper manually builds attributes

```rust
pub async fn read_file_with_middleware(...) {
    let mut attrs = HashMap::new();
    attrs.insert("resource", path_str.clone());
    attrs.insert("permission", "read");
    attrs.insert("required_permission", "read_file");
    // ... 10x duplication
}
```

**Cons:**
- ❌ **Massive duplication** - same code in 10 helpers
- ❌ **Maintenance burden** - adding new policy requires 10 file changes
- ❌ **Error prone** - easy to forget attributes or use wrong permission names
- ❌ **Not DRY** - violates don't repeat yourself principle

### Option 3: Security Modules Build Attributes from Permissions (Selected)
**Approach:** Each security module provides helper to extract attributes from `Operation::required_permissions()`

```rust
// acl.rs - ACL domain owns attribute building
pub fn build_acl_attributes(permissions: &[Permission]) -> HashMap<String, String> {
    match permissions.first() {
        Permission::FilesystemRead(path) => {
            ("acl.resource" => path, "acl.permission" => "read")
        }
        // ACL knows how to interpret permissions
    }
}

// rbac.rs - RBAC domain owns attribute building  
pub fn build_rbac_attributes(permissions: &[Permission]) -> HashMap<String, String> {
    match permissions.first() {
        Permission::FilesystemRead(_) => {
            ("rbac.required_permission" => "read_file")
        }
        // RBAC knows how to map permissions to roles
    }
}

// helpers/context.rs - Combines all security attributes
pub fn build_security_context(user: impl Into<String>, operation: &impl Operation) {
    let permissions = operation.required_permissions();
    let mut attrs = HashMap::new();
    attrs.extend(build_acl_attributes(&permissions));
    attrs.extend(build_rbac_attributes(&permissions));
    SecurityContext::new(user).with_attributes(attrs)
}
```

**Pros:**
- ✅ **Separation of concerns** - operations declare permissions, security interprets them
- ✅ **Domain expertise** - ACL/RBAC modules own their attribute logic
- ✅ **Extensible** - new security policies add their own builder functions
- ✅ **DRY** - helpers use single `build_security_context()` utility
- ✅ **Type safe** - leverages existing `Permission` enum
- ✅ **Testable** - can unit test attribute builders independently

**Cons:**
- Requires creating new helper module (`helpers/context.rs`)
- Need to update ACL/RBAC evaluation to use prefixed keys

---

## Decision Outcome

**Selected Option:** Option 3 - Security Modules Build Attributes from Permissions

### Implementation Architecture

```
┌─────────────────────────────────────────────────────────────────┐
│                        Helper Functions                         │
│  (read_file, write_file, spawn_process, etc.)                  │
└────────────────────────┬────────────────────────────────────────┘
                         │ calls
                         ▼
┌─────────────────────────────────────────────────────────────────┐
│           helpers/context.rs::build_security_context()         │
│  - Gets operation.required_permissions()                       │
│  - Calls build_acl_attributes(permissions)                     │
│  - Calls build_rbac_attributes(permissions)                    │
│  - Merges attributes and builds SecurityContext                │
└───────────┬──────────────────────────────┬──────────────────────┘
            │                              │
            ▼                              ▼
┌───────────────────────┐    ┌────────────────────────────────────┐
│  acl.rs               │    │  rbac.rs                           │
│  build_acl_attributes │    │  build_rbac_attributes             │
│  - Extracts resource  │    │  - Maps permission to role req     │
│  - Extracts perm type │    │  - Returns "rbac.*" attributes     │
│  - Returns "acl.*"    │    └────────────────────────────────────┘
│    attributes         │
└───────────────────────┘
```

### Key Design Decisions

#### Decision 1: Permission Priority Strategy
**Question:** If an operation has multiple permissions, which one to use?

**Decision:** **Use first permission (Option A)**

**Rationale:**
- Current operations only have single permission (FileRead, ProcessSpawn, etc.)
- Simple and predictable behavior
- Can be refined later when compound operations are introduced

**Code:**
```rust
pub fn build_rbac_attributes(permissions: &[Permission]) -> HashMap<String, String> {
    if permissions.is_empty() {
        return HashMap::new();
    }
    
    // Use first permission - works for current single-permission operations
    // TODO: Revisit when compound operations (e.g., CopyFile) are introduced
    let required_perm = match &permissions[0] {
        Permission::FilesystemRead(_) => "read_file",
        // ...
    };
    // ...
}
```

**Future Considerations:**
- Compound operations (CopyFile = read source + write dest)
- May need "most restrictive" or "combine all" strategy
- Documented in TODO for future enhancement

#### Decision 2: Attribute Key Namespacing
**Question:** How to prevent attribute key conflicts between ACL and RBAC?

**Decision:** **Use module prefixes (Option A)**

**Rationale:**
- Prevents namespace collisions between security modules
- Makes attribute ownership explicit
- Enables future security policies without conflicts
- Self-documenting (key name shows which module uses it)

**Attribute Keys:**
```rust
// ACL module owns these attributes
pub const ATTR_ACL_RESOURCE: &str = "acl.resource";
pub const ATTR_ACL_PERMISSION: &str = "acl.permission";

// RBAC module owns these attributes  
pub const ATTR_RBAC_REQUIRED_PERMISSION: &str = "rbac.required_permission";

// Future policies can add their own namespaces
// pub const ATTR_RATE_LIMIT_QUOTA: &str = "rate_limit.quota";
```

**Alternative Considered:**
- Keep current keys (`resource`, `permission`, `required_permission`)
- No conflicts exist today between ACL and RBAC
- **Rejected:** Prefixes provide future-proofing and clarity

#### Decision 3: Module Organization
**Question:** Where should `build_security_context()` utility live?

**Decision:** `src/helpers/context.rs`

**Rationale:**
- Semantic clarity - building SecurityContext objects
- Separates concerns from business logic helpers
- Clear import path: `use airssys_osl::helpers::build_security_context;`

#### Decision 4: Export Strategy
**Question:** Should we re-export `build_acl_attributes` and `build_rbac_attributes` from `security/mod.rs`?

**Decision:** **No re-export**

**Rationale:**
- Clear module ownership (ACL owns its builder, RBAC owns its builder)
- Prevents namespace pollution in security module
- Most users will use `build_security_context()` (combined helper)
- Advanced users can import directly: `use airssys_osl::middleware::security::acl::build_acl_attributes;`
- Can add re-exports later based on user feedback

---

## Consequences

### Positive Consequences

1. **Clean Domain Separation**
   - Operations: Declare `required_permissions()` only
   - Security: Interpret permissions into attributes
   - Helpers: Orchestrate using utilities

2. **Extensibility**
   - New security policies (rate limiting, quotas, etc.) add their own builders
   - No changes needed to operations or existing helpers
   - Example: Add `build_rate_limit_attributes()` → update `build_security_context()` → done

3. **Testability**
   - Unit test `build_acl_attributes()` with mock permissions
   - Unit test `build_rbac_attributes()` independently
   - Integration test `build_security_context()` with real operations

4. **Maintainability**
   - Single source of truth for attribute building per policy
   - Helpers use single `build_security_context()` call
   - Adding new helper = 2 line change

5. **Type Safety**
   - Leverages existing `Permission` enum
   - Pattern matching ensures all permission types handled
   - Compiler errors if new permissions added without handling

### Negative Consequences

1. **Migration Work Required**
   - Update ACL evaluation to use `ATTR_ACL_RESOURCE`, `ATTR_ACL_PERMISSION`
   - Update RBAC evaluation to use `ATTR_RBAC_REQUIRED_PERMISSION`
   - Update all 10 helper functions to use `build_security_context()`
   - Add `SecurityContext::with_attributes()` builder method

2. **Attribute Key Length**
   - `"acl.resource"` vs `"resource"` (11 vs 8 chars)
   - Minor runtime overhead (string allocation)
   - Acceptable tradeoff for clarity and extensibility

3. **Future Compound Operations**
   - Current "first permission" strategy insufficient
   - Will need enhancement when CopyFile, MoveFile introduced
   - Documented in TODO, not blocking current work

### Risks and Mitigation

| Risk | Impact | Mitigation |
|------|--------|------------|
| ACL/RBAC attribute reading breaks | High | Update evaluation code in same PR, comprehensive tests |
| Forgot to update a helper | Medium | Compiler error (build_security_context() required), test coverage |
| Permission enum missing case | Medium | Pattern matching ensures compiler errors, add tests |
| Compound operations unsupported | Low | Documented in TODO, revisit in Phase 6/7 |

---

## Implementation Plan

### Phase 1: ACL Attribute Builder
- [ ] Add `ATTR_ACL_RESOURCE`, `ATTR_ACL_PERMISSION` constants with prefix
- [ ] Implement `build_acl_attributes()` function
- [ ] Update ACL evaluation code to use prefixed constants
- [ ] Add unit tests for all Permission variants

### Phase 2: RBAC Attribute Builder  
- [ ] Add `ATTR_RBAC_REQUIRED_PERMISSION` constant with prefix
- [ ] Implement `build_rbac_attributes()` function
- [ ] Update RBAC evaluation code to use prefixed constant
- [ ] Add unit tests for all Permission variants

### Phase 3: Helper Utility
- [ ] Create `src/helpers/context.rs`
- [ ] Implement `build_security_context()` combining ACL + RBAC
- [ ] Export via `src/helpers/mod.rs`
- [ ] Add integration tests

### Phase 4: SecurityContext Builder
- [ ] Add `SecurityContext::with_attributes()` method
- [ ] Ensure builder pattern chaining works
- [ ] Add documentation and examples

### Phase 5: Update Helpers
- [ ] Update `read_file_with_middleware()`
- [ ] Update `write_file_with_middleware()`
- [ ] Update `delete_file_with_middleware()`
- [ ] Update `create_directory_with_middleware()`
- [ ] Update `spawn_process_with_middleware()`
- [ ] Update `kill_process_with_middleware()`
- [ ] Update `send_signal_with_middleware()`
- [ ] Update `connect_network_with_middleware()`
- [ ] Update `listen_network_with_middleware()`
- [ ] Update `accept_connection_with_middleware()`

### Phase 6: Test Fixes
- [ ] Remove `#[ignore]` from ACL tests (3 tests)
- [ ] Remove `#[ignore]` from RBAC tests (2 tests)
- [ ] Run `helpers_security_tests.rs` - expect 11/11 passing
- [ ] Run `helpers_audit_tests.rs` - verify audit logging works
- [ ] Run `helpers_error_tests.rs` - verify error handling works

### Phase 7: Documentation
- [ ] Add rustdoc to `build_acl_attributes()`
- [ ] Add rustdoc to `build_rbac_attributes()`
- [ ] Add rustdoc to `build_security_context()`
- [ ] Update helper function examples showing attribute usage
- [ ] Document attribute key contracts in each module

---

## Validation and Testing

### Unit Tests Required

**`build_acl_attributes()` tests:**
- [ ] FilesystemRead → `{"acl.resource": path, "acl.permission": "read"}`
- [ ] FilesystemWrite → `{"acl.resource": path, "acl.permission": "write"}`
- [ ] FilesystemDelete → `{"acl.resource": path, "acl.permission": "delete"}`
- [ ] FilesystemExecute → `{"acl.resource": path, "acl.permission": "execute"}`
- [ ] ProcessSpawn → `{"acl.resource": "process", "acl.permission": "spawn"}`
- [ ] NetworkConnect → `{"acl.resource": addr, "acl.permission": "connect"}`
- [ ] Empty permissions → `{}`

**`build_rbac_attributes()` tests:**
- [ ] FilesystemRead → `{"rbac.required_permission": "read_file"}`
- [ ] FilesystemWrite → `{"rbac.required_permission": "write_file"}`
- [ ] ProcessSpawn → `{"rbac.required_permission": "spawn_process"}`
- [ ] NetworkConnect → `{"rbac.required_permission": "connect_network"}`
- [ ] Empty permissions → `{}`

**`build_security_context()` tests:**
- [ ] Combines ACL and RBAC attributes correctly
- [ ] Sets principal correctly
- [ ] No attribute key conflicts (ACL vs RBAC)

### Integration Tests (Existing)

All existing integration tests in `helpers_security_tests.rs` should pass:
- [ ] `test_read_file_with_acl_allow` - ACL allows read
- [ ] `test_read_file_with_acl_deny` - ACL denies read
- [ ] `test_write_file_with_acl_glob_pattern` - ACL glob matching
- [ ] `test_read_file_with_rbac_admin_allowed` - RBAC admin role
- [ ] `test_write_file_with_rbac_reader_role_denied` - RBAC reader denied
- [ ] `test_rbac_no_role_assigned_denied` - RBAC no role
- [ ] `test_rbac_role_hierarchy` - RBAC hierarchy
- [ ] `test_multiple_policies_all_must_pass` - ACL + RBAC together
- [ ] `test_read_with_security_violation` - Error handling
- [ ] `test_spawn_process_with_acl_deny` - Process ACL
- [ ] `test_network_connect_with_policy` - Network policy

---

## References

### Related ADRs
- **ADR-028:** ACL Permission Model and Glob Matching (attribute requirements)
- **ADR-027:** Builder Pattern Architecture (SecurityMiddlewareBuilder)
- **ADR-029:** OSL-TASK-010 creation (helper middleware integration)

### Related Tasks
- **OSL-TASK-010:** Helper Function Middleware Integration
  - **Phase 5:** Integration Testing (current phase - revealed this issue)
  - **Phase 6:** Custom Middleware Documentation (next phase)
  - **Phase 7:** Advanced Usage Documentation (final phase)

### Code References
- `src/core/operation.rs` - Operation trait and Permission enum
- `src/middleware/security/acl.rs` - ACL policy and evaluation
- `src/middleware/security/rbac.rs` - RBAC policy and evaluation
- `src/helpers/simple.rs` - 10 helper functions to update
- `airssys-osl/tests/helpers_security_tests.rs` - Integration tests

### External Context
- Security Context Pattern: https://martinfowler.com/articles/patterns-of-distributed-systems/request-pipeline.html
- Middleware Pattern: https://www.oreilly.com/library/view/learning-rust/9781788390637/ch09s02.html

---

## Future Considerations

### 1. Compound Operations Support
When operations like `CopyFile` require multiple permissions:
```rust
// Future: CopyFileOperation
impl Operation for CopyFileOperation {
    fn required_permissions() -> Vec<Permission> {
        vec![
            Permission::FilesystemRead(source),
            Permission::FilesystemWrite(dest),
        ]
    }
}
```

**Enhancement needed:**
- RBAC: Combine permissions → `"rbac.required_permission": "read_file,write_file"`
- ACL: Multiple evaluations → check both source read AND dest write
- Document in Phase 6/7

### 2. Additional Security Policies
Future policies can follow same pattern:
```rust
// Future: Rate limiting policy
pub fn build_rate_limit_attributes(permissions: &[Permission]) -> HashMap<String, String> {
    // "rate_limit.operation_type": "filesystem_read"
    // "rate_limit.quota_key": user_id
}
```

### 3. Dynamic Attribute Requirements
Some policies may need runtime-specific attributes:
```rust
// Future: Time-based access control
pub fn build_time_policy_attributes(
    permissions: &[Permission],
    current_time: DateTime<Utc>,
) -> HashMap<String, String> {
    // "time_policy.hour": "14"
    // "time_policy.day_of_week": "Monday"
}
```

Would require extending `build_security_context()` signature.

### 4. Performance Optimization
Current implementation creates HashMap per call. Potential optimizations:
- Cache attribute building results
- Use `&'static str` for constant keys
- Arena allocation for short-lived attributes

Measure before optimizing - likely not a bottleneck.

---

## Notes

**Key Insight:** This architecture emerged from test-driven development. Integration tests revealed the missing functionality, forcing us to design a proper solution rather than implementing features without validation.

**Lesson Learned:** Separation of concerns is critical in security architecture. Operations should declare intent (permissions), security modules should interpret that intent (attributes), and helpers should orchestrate (combine).

**Success Criteria:** All 11 security integration tests pass without `#[ignore]` attributes, demonstrating that ACL and RBAC policies work correctly through helper functions.
