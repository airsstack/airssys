# ADR-028: ACL Permission Model and Glob Pattern Matching

**Status**: Accepted  
**Date**: 2025-10-10  
**Deciders**: Development Team  
**Category**: Security Architecture  
**Related**: OSL-TASK-003 Phase 3 - ACL Implementation  

## Context

The Access Control List (ACL) security policy implementation requires decisions on:

1. **Permission Type System**: How to represent and store permissions in ACL entries
2. **Pattern Matching**: How to match resource paths and permissions (exact match vs patterns)
3. **Context Attributes**: How to pass resource and permission information from middleware to policies
4. **Permission Semantics**: How to interpret empty vs wildcard vs specific permissions
5. **Breaking Changes**: Whether to accept breaking API changes in Phase 3

### Current State (Phase 1-2)
```rust
pub struct AclEntry {
    pub identity: String,
    pub resource_pattern: String,
    pub policy: AclPolicy,  // ❌ Missing permissions field
}

impl AclEntry {
    pub fn matches_resource(&self, _resource: &str) -> bool {
        self.resource_pattern == "*"  // ❌ Only wildcard, ignores parameter
    }
}
```

**Problems**:
- No permission granularity (cannot distinguish read vs write)
- No glob pattern matching for resources
- Context attributes not extracted or used
- Incomplete evaluation logic (checks identity only)

### Requirements from OSL-TASK-003
- Fine-grained access control to operations
- Resource pattern matching for file paths, network endpoints, etc.
- Permission-based access control (read, write, execute, delete)
- Integration with SecurityContext.attributes HashMap

## Decision

### 1. Permission Model: String-Based with Glob Support

**Decision**: Use `Vec<String>` for permissions with glob pattern matching.

**Rationale**:
- **Flexibility**: String-based allows arbitrary permission names (not limited to predefined enum)
- **Extensibility**: Third-party code can define custom permissions without modifying core
- **Glob Support**: Enables patterns like `"read*"` to match `"read"`, `"read_metadata"`, `"read_content"`
- **Simplicity**: No need to maintain synchronized Permission enum across modules

**Implementation**:
```rust
pub struct AclEntry {
    pub identity: String,
    pub resource_pattern: String,
    pub permissions: Vec<String>,  // ✅ String-based with glob
    pub policy: AclPolicy,
}
```

**Examples**:
```rust
// Specific permissions
AclEntry {
    permissions: vec!["read".to_string(), "write".to_string()],
    ...
}

// Glob patterns
AclEntry {
    permissions: vec!["read*".to_string()],  // Matches read, read_metadata, etc.
    ...
}

// Wildcard (all permissions)
AclEntry {
    permissions: vec!["*".to_string()],
    ...
}

// No permissions
AclEntry {
    permissions: vec![],  // Denies all access
    ...
}
```

### 2. Pattern Matching: Add glob Crate Dependency

**Decision**: Use `glob` crate (v0.3) for resource and permission pattern matching.

**Rationale**:
- **Industry Standard**: `glob` crate is mature, well-tested, widely used
- **Rich Patterns**: Supports `*`, `?`, `**`, `[...]`, `{...}` patterns
- **Performance**: Compiled patterns are efficient
- **Professional ACLs**: Production ACL systems require pattern matching

**Dependency Addition**:
```toml
# Cargo.toml (workspace)
[workspace.dependencies]
glob = "0.3"

# airssys-osl/Cargo.toml
[dependencies]
glob = { workspace = true }
```

**Implementation**:
```rust
use glob::Pattern;

impl AclEntry {
    pub fn matches_resource(&self, resource: &str) -> bool {
        Pattern::new(&self.resource_pattern)
            .map(|pattern| pattern.matches(resource))
            .unwrap_or(false)
    }
    
    pub fn matches_permission(&self, required: &str) -> bool {
        if self.permissions.is_empty() {
            return false;  // Empty = no permissions
        }
        
        self.permissions.iter().any(|perm_pattern| {
            Pattern::new(perm_pattern)
                .map(|pattern| pattern.matches(required))
                .unwrap_or(false)
        })
    }
}
```

**Supported Patterns**:
- `*` - matches any sequence (e.g., `/path/*` matches `/path/file.txt`)
- `?` - matches single character (e.g., `file?.txt` matches `file1.txt`)
- `**` - matches any depth (e.g., `/etc/**/*.conf` matches nested configs)
- `[abc]` - matches character class (e.g., `file[123].txt`)
- `{a,b}` - matches alternatives (e.g., `*.{rs,toml}`)

### 3. Context Attributes: Standardized Constants

**Decision**: Define public constants for context attribute keys.

**Constants**:
```rust
/// Context attribute key for resource path/identifier
pub const ATTR_RESOURCE: &str = "resource";

/// Context attribute key for required permission/action
pub const ATTR_PERMISSION: &str = "permission";
```

**Rationale**:
- **Consistency**: Prevents typos and inconsistencies across codebase
- **Documentation**: Self-documenting constant names
- **Refactoring**: Easy to change keys in one place
- **Type Safety**: String constants provide some level of type checking

**Usage in SecurityMiddleware**:
```rust
// Populate before policy evaluation
context.security_context.attributes.insert(
    acl::ATTR_RESOURCE.to_string(),
    "/etc/passwd".to_string()
);
context.security_context.attributes.insert(
    acl::ATTR_PERMISSION.to_string(),
    "read".to_string()
);
```

**Usage in ACL evaluate()**:
```rust
fn evaluate(&self, context: &SecurityContext) -> PolicyDecision {
    let resource = context.attributes
        .get(ATTR_RESOURCE)
        .map(|s| s.as_str())
        .unwrap_or("");
    
    let permission = context.attributes
        .get(ATTR_PERMISSION)
        .map(|s| s.as_str())
        .unwrap_or("");
    
    // ... evaluation logic
}
```

### 4. Permission Semantics

**Decision**: Define clear semantics for empty, wildcard, and specific permissions.

**Rules**:

| Permission Value | Meaning | Behavior |
|-----------------|---------|----------|
| `[]` (empty) | No permissions granted | Always denies access |
| `["*"]` | All permissions granted | Always allows (if identity and resource match) |
| `["read"]` | Specific permission | Allows only exact match |
| `["read*"]` | Glob pattern | Allows glob matches (read, read_metadata, etc.) |
| `["read", "write"]` | Multiple specific | Allows if ANY matches required permission |

**Matching Strategy**:
- **Single Permission Check**: Check if required permission matches ANY entry permission
- **Empty Required Permission**: If no permission specified in context, skip permission check
- **Case Sensitivity**: Exact match (case-sensitive by default)

**Examples**:
```rust
// Entry: permissions = []
required = "read" → ❌ DENY (no permissions granted)

// Entry: permissions = ["*"]
required = "read" → ✅ ALLOW (wildcard matches all)
required = "write" → ✅ ALLOW (wildcard matches all)

// Entry: permissions = ["read"]
required = "read" → ✅ ALLOW (exact match)
required = "write" → ❌ DENY (no match)

// Entry: permissions = ["read*"]
required = "read" → ✅ ALLOW (glob matches)
required = "read_metadata" → ✅ ALLOW (glob matches)
required = "write" → ❌ DENY (no match)

// Entry: permissions = ["read", "write"]
required = "read" → ✅ ALLOW (matches first)
required = "write" → ✅ ALLOW (matches second)
required = "delete" → ❌ DENY (no match)
```

### 5. Breaking Changes Acceptance

**Decision**: Accept breaking changes to `AclEntry` API in Phase 3.

**Rationale**:
- **Early Development**: Phase 3 is active development, not production
- **No External Consumers**: No published crates depending on current API
- **Better Now Than Later**: Fixing API before production release is cheaper
- **Correctness Over Compatibility**: Complete implementation more important than backward compatibility

**Breaking Changes**:
```rust
// OLD (Phase 1-2)
impl AclEntry {
    pub fn new(identity: String, resource_pattern: String, policy: AclPolicy) -> Self
}

// NEW (Phase 3)
impl AclEntry {
    pub fn new(
        identity: String, 
        resource_pattern: String, 
        permissions: Vec<String>,  // ← NEW PARAMETER
        policy: AclPolicy
    ) -> Self
}
```

**Migration Strategy**:
1. Update all existing tests (6 tests in acl.rs)
2. Update documentation examples
3. Add migration notes to CHANGELOG (when released)
4. Consider deprecation warnings in future if needed

## Consequences

### Positive

✅ **Flexible Permission System**: String-based permissions support arbitrary permission names  
✅ **Professional Pattern Matching**: glob crate provides industry-standard pattern matching  
✅ **Clear Semantics**: Well-defined rules for empty/wildcard/specific permissions  
✅ **Extensibility**: Third-party code can define custom permissions and patterns  
✅ **Standards Compliance**: Follows workspace standards §2.1, §6.1 (YAGNI), §6.2 (no dyn)  
✅ **Type Safety**: Constants prevent attribute key typos  

### Negative

⚠️ **Breaking Changes**: Existing code using `AclEntry::new()` will break  
⚠️ **New Dependency**: Adds `glob` crate to dependency tree (~small, well-maintained)  
⚠️ **Runtime Pattern Compilation**: Glob patterns compiled at runtime (mitigated by caching potential)  
⚠️ **Error Handling**: Invalid glob patterns need proper error handling  

### Mitigations

1. **Breaking Changes**: Acceptable at current development stage, document in CHANGELOG
2. **Dependency Size**: `glob` crate is small (~15KB), no transitive dependencies
3. **Performance**: Pattern compilation happens once per ACL entry, consider caching if needed
4. **Error Handling**: Use `Pattern::new().unwrap_or(false)` for graceful degradation

## Implementation Plan

### Phase 3 Tasks (7.5 hours total)

1. **Add glob Dependency** (15 min)
   - Update root Cargo.toml workspace.dependencies
   - Update airssys-osl/Cargo.toml dependencies

2. **Define Constants and Update AclEntry** (30 min)
   - Add `ATTR_RESOURCE` and `ATTR_PERMISSION` constants
   - Add `permissions: Vec<String>` field to `AclEntry`
   - Update `AclEntry::new()` constructor

3. **Implement Glob-Based Resource Matching** (45 min)
   - Rewrite `matches_resource()` using `glob::Pattern`
   - Add error handling for invalid patterns

4. **Implement Permission Checking** (1 hour)
   - Add `matches_permission()` method with glob support
   - Implement empty/wildcard/specific permission logic

5. **Fix evaluate() Logic** (2 hours)
   - Extract resource and permission from context.attributes
   - Check identity AND resource AND permission
   - Update error messages with context

6. **Update Existing Tests** (30 min)
   - Update 6 existing tests for new `permissions` parameter

7. **Add Comprehensive New Tests** (2 hours)
   - Resource glob pattern matching tests (8 tests)
   - Permission glob matching tests (6 tests)
   - Combined identity+resource+permission tests (4 tests)
   - Total: ~18 new tests

8. **Documentation Updates** (1 hour)
   - Document constants
   - Update module docs with glob pattern examples
   - Add rustdoc examples

## Alternatives Considered

### Alternative 1: Enum-Based Permissions
```rust
pub enum Permission {
    Read,
    Write,
    Execute,
    Delete,
    Custom(String),
}
```

**Rejected**: 
- Less flexible than strings
- Requires enum updates for new permissions
- Custom variant defeats type safety anyway
- More complex serialization

### Alternative 2: Regex Instead of Glob
```rust
use regex::Regex;
```

**Rejected**:
- Overkill for resource path matching
- More complex syntax for users
- Higher performance cost
- `glob` is more intuitive for file paths

### Alternative 3: Keep Simple String Matching
```rust
pub fn matches_resource(&self, resource: &str) -> bool {
    self.resource_pattern == resource || self.resource_pattern == "*"
}
```

**Rejected**:
- Too limited for production ACL systems
- Users expect pattern matching (industry standard)
- Would need to implement it later anyway

## Cross-References

- **Main Task**: OSL-TASK-003 Security Middleware Module
- **Development Plan**: OSL-TASK-003-DEVELOPMENT-PLAN.md
- **Implementation File**: `src/middleware/security/acl.rs`
- **Related ADRs**: None (first security-related ADR)
- **Workspace Standards**: §2.1, §5.1, §6.1, §6.2, §6.3
- **Microsoft Guidelines**: M-SIMPLE-ABSTRACTIONS, M-DI-HIERARCHY

## Compliance

### Workspace Standards
- ✅ **§2.1**: 3-layer import organization maintained
- ✅ **§5.1**: glob added to workspace.dependencies (proper dependency management)
- ✅ **§6.1**: YAGNI principles (only essential ACL features, no over-engineering)
- ✅ **§6.2**: Avoid dyn patterns (no trait objects, string-based implementation)
- ✅ **§6.3**: Microsoft Rust Guidelines (M-SIMPLE-ABSTRACTIONS)

### Security Requirements
- ✅ Deny-by-default model maintained (empty permissions deny)
- ✅ No security policy bypasses
- ✅ Comprehensive audit trail (via SecurityMiddleware)
- ✅ Input validation (glob pattern error handling)

---

**Status**: ✅ **Accepted and Ready for Implementation**  
**Next Step**: Begin Phase 3 implementation following this ADR  
**Estimated Completion**: 2025-10-10 (1 day of focused work)
