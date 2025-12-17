# WASM-TASK-005 Phase 1 Task 1.2: Component.toml Capability Parser - IMPLEMENTATION PLAN

**Task:** Component.toml Capability Parser  
**Status:** 📋 PLANNED (Ready to Start)  
**Date Created:** 2025-12-17  
**Estimated Duration:** 2-3 days (17.25 hours)  
**Prerequisites:** ✅ Task 1.1 complete

---

## Executive Summary

**What**: Build a TOML parser that reads `Component.toml` manifest files, parses the `[capabilities]` section, validates capability declarations, and constructs `WasmCapabilitySet` instances for security enforcement.

**Why**: Components must explicitly declare required host resource access in Component.toml before spawn. This parser is the **entry point** for the entire capability-based security system - without it, the WasmCapabilitySet (Task 1.1) and WasmSecurityContext (Task 1.3) have no input data.

**How**: Use `serde` + `toml` crate to deserialize TOML into intermediate structs, validate patterns and permissions, transform into `WasmCapability` enum instances, and build `WasmCapabilitySet`. The parser enforces strict validation to prevent security bypasses from malformed declarations.

**Architecture Position**: This parser sits between Component.toml (static manifest) and WasmCapabilitySet (runtime security), bridging the gap from developer intent to runtime enforcement.

---

## TOML Schema Specification

### Complete Schema Definition

```toml
# Component.toml - WASM Component Manifest
# Schema Version: 1.0

[component]
name = "example-component"
version = "1.0.0"
description = "Example component demonstrating capability syntax"

[capabilities]
# Filesystem capabilities (read, write, execute)
filesystem.read = ["/app/config/*", "/app/data/*.json"]
filesystem.write = ["/app/data/*", "/tmp/component-*"]
filesystem.execute = ["/app/bin/tool"]

# Network capabilities (connect, bind, listen)
network.connect = ["api.example.com:443", "*.cdn.example.com:80"]
network.bind = ["127.0.0.1:8080"]
network.listen = ["0.0.0.0:9000"]

# Storage capabilities (read, write, delete)
storage.read = ["component:<id>:config:*", "shared:cache:*"]
storage.write = ["component:<id>:data:*"]
storage.delete = ["component:<id>:temp:*"]

# Boolean flags for dangerous capabilities (default: false)
process.spawn = false  # Deny process spawning (explicit denial)
```

### Capability Type Definitions

| Capability Type | Permission Values | Pattern Format | Example |
|-----------------|-------------------|----------------|---------|
| **Filesystem** | `read`, `write`, `execute` | Filesystem glob patterns | `/app/data/*.json`, `/tmp/**/*.log` |
| **Network** | `connect`, `bind`, `listen` | `domain:port` with wildcard subdomains | `api.example.com:443`, `*.cdn.example.com:80` |
| **Storage** | `read`, `write`, `delete` | Namespace hierarchy (`:` separator) | `component:<id>:data:*`, `shared:config:*` |
| **Process** | Boolean (`true`/`false`) | N/A (boolean flag) | `process.spawn = false` |

### Pattern Syntax Rules

**Filesystem Patterns:**
- **Glob Syntax**: Reuse airssys-osl glob patterns (`*`, `**`, `?`, `[abc]`, `{a,b}`)
- **Absolute Paths Only**: Must start with `/` (reject relative paths like `../etc/passwd`)
- **No Parent Directory Escapes**: Reject patterns with `..` sequences
- **Examples**:
  - ✅ `/app/data/*.json` (all JSON files in /app/data)
  - ✅ `/app/**/*.log` (all log files recursively)
  - ❌ `../etc/passwd` (relative path - security risk)
  - ❌ `/app/../etc/passwd` (parent escape - security risk)

**Network Patterns:**
- **Format**: `domain:port` or `ip:port`
- **Wildcard Subdomains**: Support `*.example.com` for subdomain matching
- **Port Ranges**: NOT supported in v1.0 (require exact port)
- **Examples**:
  - ✅ `api.example.com:443` (exact domain + port)
  - ✅ `*.cdn.example.com:80` (wildcard subdomain)
  - ✅ `192.168.1.100:8080` (IP address + port)
  - ❌ `example.com:*` (wildcard port - not supported)
  - ❌ `example.com:80-90` (port range - not supported)

**Storage Patterns:**
- **Format**: Hierarchical namespace with `:` separator
- **Component Namespace**: `component:<id>:*` (auto-substituted with actual component ID)
- **Shared Namespace**: `shared:*` for cross-component storage
- **Glob Support**: Support `*` wildcard for namespace segments
- **Examples**:
  - ✅ `component:<id>:data:*` (component-scoped data namespace)
  - ✅ `shared:cache:*` (shared cache namespace)
  - ✅ `component:<id>:*` (all component namespaces)
  - ❌ `admin:*` (reserved namespace - requires admin role)

### Validation Rules

| Rule | Description | Error Message |
|------|-------------|---------------|
| **No Empty Arrays** | Capability arrays must contain at least 1 pattern | "Capability '{capability}' has empty pattern array" |
| **No Duplicate Patterns** | Same pattern cannot appear twice in same capability | "Duplicate pattern '{pattern}' in '{capability}'" |
| **Valid Permissions** | Permissions must match allowed values for capability type | "Invalid permission '{perm}' for '{capability}' (expected: {allowed})" |
| **Filesystem: Absolute Paths** | Filesystem paths must start with `/` | "Filesystem path '{path}' must be absolute (start with /)" |
| **Filesystem: No Parent Escapes** | Reject `..` in paths | "Filesystem path '{path}' contains parent directory escape (..)" |
| **Network: Valid Format** | Must match `domain:port` or `ip:port` | "Network endpoint '{endpoint}' must be in 'domain:port' format" |
| **Storage: Valid Namespace** | Must follow `:` hierarchy | "Storage namespace '{ns}' must use ':' hierarchy separator" |
| **Process: Boolean Only** | Process capabilities must be boolean | "Process capability '{cap}' must be boolean (true/false)" |

---

## Implementation Steps (17 Steps, ~17.25 hours)

### Step 1: Create Parser Module Structure (30 min)
- Create `airssys-wasm/src/security/parser.rs`
- Add module declaration to `mod.rs`
- Add 3-layer imports (§2.1)
- Define module-level rustdoc
- **Checkpoint**: `cargo check` passes

### Step 2: Implement Data Structures (1 hour)
- `ComponentManifest` struct with `#[derive(Deserialize)]`
- `CapabilityDeclarations` (nested TOML structure)
- `FilesystemCapabilities`, `NetworkCapabilities`, `StorageCapabilities`, `ProcessCapabilities`
- **Checkpoint**: Structs serialize/deserialize correctly

### Step 3: Implement Error Types (45 min)
- `ParseError` enum with `#[derive(Error)]`
- Error variants for validation failures
- Detailed error messages with context
- **Checkpoint**: Errors compile correctly

### Step 4: Implement TOML Deserialization (1 hour)
- `ComponentManifestParser::parse()` method
- Use `toml::from_str()` for deserialization
- Basic error handling
- **Checkpoint**: Parse simple TOML successfully

### Step 5: Implement Filesystem Validation (1.5 hours)
- `validate_filesystem_paths()` method
- Absolute path check, parent directory escape check
- Duplicate detection
- 5 unit tests
- **Checkpoint**: Filesystem validation tests pass

### Step 6: Implement Network Validation (1.5 hours)
- `validate_network_endpoints()` method
- Domain:port format check, port validation (1-65535)
- 5 unit tests
- **Checkpoint**: Network validation tests pass

### Step 7: Implement Storage Validation (1 hour)
- `validate_storage_namespaces()` method
- Colon hierarchy check
- 3 unit tests
- **Checkpoint**: Storage validation tests pass

### Step 8: Implement WasmCapabilitySet Builder (1.5 hours)
- `build_capability_set()` method
- Map validated patterns to `WasmCapability` enum
- Integration test (TOML → WasmCapabilitySet)
- **Checkpoint**: End-to-end parsing works

### Step 9: Comprehensive Test Suite (2 hours)
- 10 positive tests (V01-V10)
- 10 negative tests (E01-E10)
- 10 edge case tests (EC01-EC10)
- **Checkpoint**: 30+ tests pass

### Step 10: Parser Documentation (1.5 hours)
- Module-level rustdoc
- Function rustdoc with examples
- Run `cargo doc --no-deps --open`
- **Checkpoint**: Zero rustdoc warnings

### Step 11: Component.toml Syntax Guide (2 hours)
- Create `docs/components/wasm/component-manifest-syntax.md`
- Capability syntax reference, validation rules
- 15+ examples
- **Checkpoint**: Documentation reviewed

### Step 12: Examples (1 hour)
- `examples/security_parsing_simple.rs`
- `examples/security_parsing_complex.rs`
- `examples/security_parsing_errors.rs`
- **Checkpoint**: All examples run

### Step 13: Integration Tests (1 hour)
- Parser + ACL conversion integration
- Error message clarity tests
- Performance benchmarks
- **Checkpoint**: Integration tests pass

### Step 14: Final Quality Gates (1 hour)
- `cargo clippy --all-targets` (zero warnings)
- `cargo test --all-targets` (all pass)
- `cargo doc --no-deps` (zero warnings)
- **Checkpoint**: All quality gates pass

---

## Test Plan (30+ Test Scenarios)

### Valid Declaration Tests (10 tests)

| Test ID | Scenario | Expected Output |
|---------|----------|-----------------|
| V01 | Single filesystem read | 1 Filesystem capability with read |
| V02 | Multiple filesystem patterns | 1 Filesystem capability with 2 paths |
| V03 | Multiple filesystem permissions | 2 Filesystem capabilities (read + write) |
| V04 | Network connect | 1 Network capability |
| V05 | Network wildcard subdomain | 1 Network capability with wildcard |
| V06 | Storage read | 1 Storage capability |
| V07 | All capability types | 3 capabilities |
| V08 | Process spawn denied | Process capability NOT added |
| V09 | Empty capabilities section | Empty WasmCapabilitySet |
| V10 | Glob patterns (recursive) | 1 Filesystem with recursive glob |

### Invalid Declaration Tests (10 tests)

| Test ID | Scenario | Expected Error |
|---------|----------|----------------|
| E01 | Empty pattern array | `ParseError::EmptyPatternArray` |
| E02 | Duplicate filesystem pattern | `ParseError::DuplicatePattern` |
| E03 | Relative filesystem path | `ParseError::RelativeFilesystemPath` |
| E04 | Parent directory escape | `ParseError::ParentDirectoryEscape` |
| E05 | Invalid network endpoint (no port) | `ParseError::InvalidNetworkEndpoint` |
| E06 | Invalid network port (0) | `ParseError::InvalidNetworkPort` |
| E07 | Invalid network port (string) | `ParseError::InvalidNetworkPort` |
| E08 | Invalid storage namespace (no colon) | `ParseError::InvalidStorageNamespace` |
| E09 | Malformed TOML syntax | `ParseError::TomlParseError` |
| E10 | Missing component name | `ParseError::MissingMetadataField` |

### Edge Case Tests (10 tests)

| Test ID | Scenario | Expected Behavior |
|---------|----------|-------------------|
| EC01 | Very long path (1000 chars) | Parse successfully |
| EC02 | IPv4 network endpoint | Parse successfully |
| EC03 | IPv6 network endpoint | Parse successfully |
| EC04 | High port number (65535) | Parse successfully |
| EC05 | Component namespace substitution | Parse successfully |
| EC06 | Mixed case permission | Fail (case-sensitive) |
| EC07 | UTF-8 paths | Parse successfully |
| EC08 | Whitespace in patterns | Trim + parse |
| EC09 | Comments in TOML | Ignore comments + parse |
| EC10 | Multiple capability sections | TOML parser error |

---

## Integration with airssys-osl

### Integration Architecture

```text
┌──────────────────────────────────────────────────────────────┐
│ 1. Component.toml (TOML File)                               │
│    [capabilities]                                            │
│    filesystem.read = ["/app/data/*"]                         │
└────────────────┬─────────────────────────────────────────────┘
                 │ Task 1.2: Parser (THIS TASK)
                 ▼
┌──────────────────────────────────────────────────────────────┐
│ 2. WasmCapabilitySet (Task 1.1 - ALREADY DONE)              │
│    WasmCapability::Filesystem {                              │
│        paths: vec!["/app/data/*"],                           │
│        permissions: vec!["read"],                            │
│    }                                                         │
└────────────────┬─────────────────────────────────────────────┘
                 │ Task 1.1: to_acl_entry() (ALREADY DONE)
                 ▼
┌──────────────────────────────────────────────────────────────┐
│ 3. airssys-osl AclEntry (airssys-osl integration)           │
│    AclEntry {                                                │
│        identity: "component-123",                            │
│        resource_pattern: "/app/data/*",                      │
│        permissions: vec!["read"],                            │
│        policy: AclPolicy::Allow,                             │
│    }                                                         │
└────────────────┬─────────────────────────────────────────────┘
                 │ airssys-osl SecurityPolicy::evaluate()
                 ▼
┌──────────────────────────────────────────────────────────────┐
│ 4. airssys-osl PolicyDecision                               │
│    PolicyDecision::Allow or Deny(reason)                     │
└──────────────────────────────────────────────────────────────┘
```

### Integration Verification

✅ **Parser Dependencies:**
- `toml` - Workspace dependency ✅
- `serde` - Workspace dependency ✅
- `airssys-osl` - Already added in Task 1.1 ✅

✅ **Parser Output:**
- Produces `WasmCapabilitySet` (Task 1.1 type) ✅
- No direct airssys-osl dependencies in parser ✅
- Integration happens through Task 1.1's `to_acl_entry()` bridge ✅

✅ **Security Model Alignment:**
- airssys-osl: Deny-by-default ✅
- airssys-wasm: Deny-by-default ✅
- airssys-osl: Glob patterns ✅
- airssys-wasm: Glob patterns (delegated to airssys-osl) ✅

### Integration Flow Example

```rust
// Step 1: Parse Component.toml (Task 1.2 - THIS TASK)
let parser = ComponentManifestParser::new();
let capability_set = parser.parse(toml_content)?;
// Returns: WasmCapabilitySet

// Step 2: Convert to ACL (Task 1.1 - ALREADY DONE)
let acl_entries = capability_set.to_acl_entries("component-123");
// Returns: Vec<AclEntry> from airssys-osl

// Step 3: Build ACL Policy (Task 3.1 - FUTURE)
let acl = AccessControlList::new();  // airssys-osl type
for entry in acl_entries {
    acl = acl.add_entry(entry);  // airssys-osl method
}

// Step 4: Evaluate (Task 3.1 - FUTURE)
let decision = acl.evaluate(&security_context);  // airssys-osl evaluation
match decision {
    PolicyDecision::Allow => CapabilityCheckResult::Granted,
    PolicyDecision::Deny(reason) => CapabilityCheckResult::Denied(reason),
}
```

---

## Quality Gates

### Cargo Clippy Requirements
- **Command**: `cargo clippy --all-targets --all-features -- -D warnings`
- **Target**: Zero warnings (deny warnings)
- **Enforced Lints**: `unwrap_used`, `expect_used`, `panic` (deny)

### Rustdoc Requirements
- **Command**: `cargo doc --no-deps --document-private-items`
- **Target**: Zero rustdoc warnings
- **Standards**: Microsoft Rust Guidelines (M-MODULE-DOCS, M-CANONICAL-DOCS, M-EXAMPLES)

### Test Coverage Targets
- **Unit Test Coverage**: >95% (all validation logic)
- **Integration Test Coverage**: 4+ integration tests
- **Edge Case Coverage**: 10+ edge case tests
- **Total Tests**: 30+ test cases

### Performance Considerations
- **Parser Performance Target**: <100μs per Component.toml
- **Optimization Strategies**:
  - Lazy validation (validate only declared capabilities)
  - HashSet for O(1) duplicate detection
  - No regex (simple string operations faster)
  - Minimal allocations

---

## Timeline Estimate

| Step | Description | Time | Cumulative |
|------|-------------|------|------------|
| 1 | Parser module structure | 30 min | 30 min |
| 2 | Data structures | 1 hour | 1.5 hours |
| 3 | Error types | 45 min | 2.25 hours |
| 4 | TOML deserialization | 1 hour | 3.25 hours |
| 5 | Filesystem validation | 1.5 hours | 4.75 hours |
| 6 | Network validation | 1.5 hours | 6.25 hours |
| 7 | Storage validation | 1 hour | 7.25 hours |
| 8 | WasmCapabilitySet builder | 1.5 hours | 8.75 hours |
| 9 | Comprehensive test suite | 2 hours | 10.75 hours |
| 10 | Parser documentation | 1.5 hours | 12.25 hours |
| 11 | Syntax guide | 2 hours | 14.25 hours |
| 12 | Examples | 1 hour | 15.25 hours |
| 13 | Integration tests | 1 hour | 16.25 hours |
| 14 | Final quality gates | 1 hour | **17.25 hours** |

**Total Duration**: 17.25 hours ≈ **2-3 days** (6-8 hour workdays)

**Breakdown by Activity**:
- Core implementation: 8.75 hours (51%)
- Testing: 3 hours (17%)
- Documentation: 4.5 hours (26%)
- Quality assurance: 1 hour (6%)

---

## Risk Assessment

### Technical Risks

| Risk | Severity | Probability | Mitigation |
|------|----------|-------------|------------|
| **Pattern Validation Bypass** | High | Low | 20+ negative tests, security review |
| **TOML Parsing Performance** | Medium | Low | Benchmark tests, lazy evaluation |
| **Error Message Clarity** | Medium | Medium | User testing, detailed error messages |
| **Future Schema Changes** | Medium | High | Version schema, backward compatibility |

### Security Considerations

**Critical Security Properties**:
1. **Fail-Closed**: Parser errors MUST deny all access
2. **No Bypass**: Validation logic MUST be comprehensive
3. **Clear Errors**: Error messages MUST NOT leak sensitive info
4. **Immutability**: Parsed capability set MUST be immutable

---

## Standards Compliance

### PROJECTS_STANDARD.md
- §2.1: 3-layer import organization ✅
- §4.3: Module architecture (mod.rs only re-exports) ✅
- §5.1: Dependency management ✅
- §6.1: YAGNI principles ✅

### Microsoft Rust Guidelines
- M-DESIGN-FOR-AI: Clear API, extensive docs ✅
- M-CANONICAL-DOCS: Comprehensive public API docs ✅
- M-EXAMPLES: Examples for all public functions ✅

### ADR Compliance
- ADR-WASM-005: Capability-Based Security Model ✅
- ADR-WASM-010: Implementation Strategy (reuse airssys-osl) ✅

---

## Next Steps After Task 1.2

### Task 1.3: SecurityContext Bridge (1-2 days)
- Convert `WasmSecurityContext` → `airssys_osl::core::context::SecurityContext`
- Attach security context to ComponentActor instances
- Restore context after supervisor restart

### Phase 2: Trust-Level System (Week 2)
- Task 2.1: TrustLevel enum (Trusted/Unknown/DevMode)
- Task 2.2: Approval workflow state machine
- Task 2.3: Trust configuration file format

---

## Approval Status

**Planner**: Memory Bank Planner  
**Date**: 2025-12-17  
**Status**: ✅ **APPROVED** - Ready for implementation

This plan provides a comprehensive blueprint for implementing the Component.toml parser with full airssys-osl integration, strict validation, comprehensive testing, and production-ready documentation.

**Ready to Start:** Task 1.2 implementation can begin immediately.

