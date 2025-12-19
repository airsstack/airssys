# WASM-TASK-005 Phase 3 Task 3.2: Host Function Integration Points - IMPLEMENTATION PLAN

**Task:** Host Function Integration Points  
**Status:** 📋 PLANNED (Ready to Start)  
**Date Created:** 2025-12-19  
**Estimated Duration:** 2-3 days (16-20 hours)  
**Prerequisites:** ✅ Phase 1 complete (Tasks 1.1-1.3), ✅ Phase 2 complete (Tasks 2.1-2.3), ✅ Task 3.1 complete (Capability Check API)

---

## Executive Summary

**What**: Define integration points and patterns for host functions to check capabilities before granting resource access. This includes a capability check macro to reduce boilerplate, integration patterns for all host function categories (filesystem, network, storage), and WIT error types for capability violations.

**Why**: Host functions need a consistent, ergonomic way to enforce capability checks. Without standardized integration points, each host function would implement checks differently, leading to security inconsistencies and maintenance burden. The integration patterns ensure all host functions enforce capabilities uniformly.

**How**: Create:
1. `require_capability!()` macro for ergonomic capability checks in host functions
2. Integration patterns for filesystem host functions (read, write, execute)
3. Integration patterns for network host functions (connect, bind, listen)
4. Integration patterns for storage host functions (read, write, delete)
5. WIT error types for capability check failures
6. Comprehensive integration tests with mock host functions

**Architecture Position**: This task bridges capability enforcement (Task 3.1) and actual host function implementations (Block 8, future work), providing the standardized patterns all host functions will follow.

---

## Implementation Strategy

### Core Design Principles

1. **Zero Boilerplate**: Macro reduces capability checks to one line
2. **Type Safety**: Compile-time errors for incorrect usage
3. **Clear Errors**: WIT error types with detailed capability violation messages
4. **Consistent Patterns**: All host function categories follow same pattern
5. **Security First**: Deny-by-default, fail-closed on errors

### Integration Architecture

```text
┌─────────────────────────────────────────────────────────────┐
│ 1. Host Function Declaration (WIT)                         │
│    filesystem-read: func(path: string) -> result<list<u8>> │
└────────────────────┬────────────────────────────────────────┘
                     │
                     ▼
┌─────────────────────────────────────────────────────────────┐
│ 2. Rust Host Function Implementation                        │
│    fn filesystem_read(path: &str) -> Result<Vec<u8>>       │
│    - require_capability!(Filesystem, path, "read")?         │
│    - Actual implementation (airssys-osl FileSystemOp)      │
└────────────────────┬────────────────────────────────────────┘
                     │
                     ▼
┌─────────────────────────────────────────────────────────────┐
│ 3. require_capability! Macro Expansion                      │
│    check_capability(&REGISTRY, component_id, path, "read")  │
│    .into_result()                                           │
│    .map_err(|reason| CapabilityError::Denied(reason))?     │
└────────────────────┬────────────────────────────────────────┘
                     │
                     ▼
┌─────────────────────────────────────────────────────────────┐
│ 4. Task 3.1: check_capability()                            │
│    - Registry lookup                                        │
│    - ACL evaluation                                         │
│    - Granted or Denied                                      │
└────────────────────┬────────────────────────────────────────┘
                     │
                     ▼
┌─────────────────────────────────────────────────────────────┐
│ 5. Result Handling                                          │
│    - Granted: Proceed with operation                        │
│    - Denied: Return WIT error to component                  │
└─────────────────────────────────────────────────────────────┘
```

---

## Data Structure Specifications

### 1. WIT Error Types

```wit
// File: wit/core/errors.wit

/// Capability check failure errors.
variant capability-error {
    /// Component lacks required capability for the operation.
    access-denied(string),
    
    /// Component not found in security registry.
    component-not-found(string),
    
    /// Invalid resource pattern in capability check.
    invalid-resource(string),
    
    /// Internal security system error.
    security-error(string),
}

/// Result type for operations requiring capability checks.
type capability-result<T> = result<T, capability-error>;
```

### 2. Rust Error Types

```rust
/// Capability enforcement errors.
///
/// These errors are returned when capability checks fail in host functions.
///
/// # WIT Mapping
///
/// This enum maps to `capability-error` in WIT, allowing WASM components
/// to receive detailed capability violation information.
#[derive(Debug, Clone, thiserror::Error)]
pub enum CapabilityError {
    /// Component lacks required capability.
    ///
    /// # Example
    ///
    /// ```text
    /// Component 'component-123' attempted to read '/etc/passwd' but only
    /// has capability for '/app/data/*'
    /// ```
    #[error("Access denied: {0}")]
    AccessDenied(String),
    
    /// Component not found in security registry.
    #[error("Component not found: {0}")]
    ComponentNotFound(String),
    
    /// Invalid resource pattern in capability check.
    #[error("Invalid resource: {0}")]
    InvalidResource(String),
    
    /// Internal security system error.
    #[error("Security error: {0}")]
    SecurityError(String),
}

impl CapabilityError {
    /// Creates an access denied error with detailed reason.
    pub fn access_denied(reason: impl Into<String>) -> Self {
        Self::AccessDenied(reason.into())
    }
    
    /// Creates a component not found error.
    pub fn component_not_found(component_id: &str) -> Self {
        Self::ComponentNotFound(component_id.to_string())
    }
}

// Conversion from CapabilityCheckResult to CapabilityError
impl From<CapabilityCheckResult> for Result<(), CapabilityError> {
    fn from(result: CapabilityCheckResult) -> Self {
        match result {
            CapabilityCheckResult::Granted => Ok(()),
            CapabilityCheckResult::Denied { reason } => {
                Err(CapabilityError::AccessDenied(reason))
            }
        }
    }
}
```

---

## Macro Implementation

### require_capability! Macro

```rust
/// Require capability check in host functions (macro).
///
/// This macro provides ergonomic capability checking for host functions,
/// reducing boilerplate from 5+ lines to a single line.
///
/// # Syntax
///
/// ```rust,ignore
/// require_capability!(capability_type, resource, permission)?;
/// ```
///
/// # Arguments
///
/// - `capability_type`: Capability category (Filesystem, Network, Storage)
/// - `resource`: Resource being accessed (path, endpoint, namespace)
/// - `permission`: Permission requested ("read", "write", "connect", etc.)
///
/// # Returns
///
/// Returns `Result<(), CapabilityError>`. Use `?` operator to propagate errors.
///
/// # Examples
///
/// ## Filesystem Host Function
///
/// ```rust,ignore
/// fn filesystem_read(path: &str) -> Result<Vec<u8>, HostError> {
///     // Single-line capability check
///     require_capability!(Filesystem, path, "read")?;
///     
///     // Proceed with actual file read
///     let bytes = std::fs::read(path)?;
///     Ok(bytes)
/// }
/// ```
///
/// ## Network Host Function
///
/// ```rust,ignore
/// fn network_connect(endpoint: &str) -> Result<TcpStream, HostError> {
///     require_capability!(Network, endpoint, "connect")?;
///     
///     let stream = TcpStream::connect(endpoint)?;
///     Ok(stream)
/// }
/// ```
///
/// ## Storage Host Function
///
/// ```rust,ignore
/// fn storage_write(key: &str, value: &[u8]) -> Result<(), HostError> {
///     require_capability!(Storage, key, "write")?;
///     
///     STORAGE.insert(key, value)?;
///     Ok(())
/// }
/// ```
///
/// # Macro Expansion
///
/// The macro expands to:
///
/// ```rust,ignore
/// // require_capability!(Filesystem, path, "read")?;
/// // expands to:
/// {
///     let component_id = get_current_component_id()?;
///     let result = check_capability(
///         &COMPONENT_SECURITY_REGISTRY,
///         component_id,
///         path,
///         "read",
///     );
///     result.into_result().map_err(CapabilityError::access_denied)?;
/// }
/// ```
///
/// # Performance
///
/// - Zero runtime overhead compared to manual check_capability() call
/// - Compile-time macro expansion (no runtime cost)
/// - Inlined for maximum performance
#[macro_export]
macro_rules! require_capability {
    ($capability_type:ident, $resource:expr, $permission:expr) => {{
        // Get current component ID from execution context
        let component_id = $crate::security::enforcement::get_current_component_id()
            .map_err(|e| $crate::security::enforcement::CapabilityError::SecurityError(
                format!("Failed to get component ID: {}", e)
            ))?;
        
        // Perform capability check
        let result = $crate::security::enforcement::check_capability(
            &$crate::security::enforcement::COMPONENT_SECURITY_REGISTRY,
            &component_id,
            $resource,
            $permission,
        );
        
        // Convert to Result and propagate error
        result.into_result().map_err($crate::security::enforcement::CapabilityError::access_denied)?;
    }};
}
```

### Component Context Management

```rust
/// Thread-local storage for current component ID.
///
/// This allows host functions to retrieve the component ID without
/// passing it through all function arguments.
thread_local! {
    static CURRENT_COMPONENT_ID: RefCell<Option<String>> = RefCell::new(None);
}

/// Sets the current component ID for this thread.
///
/// # Safety
///
/// This function is called by the WASM runtime before invoking any host
/// function. It must be called exactly once per host function invocation.
///
/// # Examples
///
/// ```rust,ignore
/// // Called by WASM runtime before host function
/// set_current_component_id("component-123".to_string());
///
/// // Host function can now use require_capability! macro
/// fn my_host_function() -> Result<(), HostError> {
///     require_capability!(Filesystem, "/app/data", "read")?;
///     // ...
/// }
/// ```
pub fn set_current_component_id(component_id: String) {
    CURRENT_COMPONENT_ID.with(|id| {
        *id.borrow_mut() = Some(component_id);
    });
}

/// Gets the current component ID for this thread.
///
/// # Returns
///
/// Returns the component ID set by `set_current_component_id()`, or an error
/// if no component ID is set (indicating a programming error).
///
/// # Errors
///
/// Returns error if called outside a host function invocation context.
pub fn get_current_component_id() -> Result<String, String> {
    CURRENT_COMPONENT_ID.with(|id| {
        id.borrow()
            .as_ref()
            .cloned()
            .ok_or_else(|| "No component ID set for current thread".to_string())
    })
}

/// Clears the current component ID after host function returns.
///
/// # Safety
///
/// This function is called by the WASM runtime after a host function returns.
/// It must be called exactly once per host function invocation.
pub fn clear_current_component_id() {
    CURRENT_COMPONENT_ID.with(|id| {
        *id.borrow_mut() = None;
    });
}
```

---

## Integration Patterns

### Pattern 1: Filesystem Host Functions

```rust
/// Filesystem read operation with capability check.
///
/// # WIT Declaration
///
/// ```wit
/// filesystem-read: func(path: string) -> capability-result<list<u8>>
/// ```
///
/// # Examples
///
/// ```rust,ignore
/// // Component calls filesystem-read
/// let bytes = filesystem_read("/app/config/settings.json")?;
/// ```
#[host_function]
pub fn filesystem_read(path: String) -> Result<Vec<u8>, CapabilityError> {
    // Step 1: Capability check (via macro)
    require_capability!(Filesystem, path.as_str(), "read")?;
    
    // Step 2: Actual implementation (delegated to Block 8)
    // For now, return placeholder
    todo!("Actual filesystem read implementation in Block 8")
}

/// Filesystem write operation with capability check.
#[host_function]
pub fn filesystem_write(path: String, data: Vec<u8>) -> Result<(), CapabilityError> {
    require_capability!(Filesystem, path.as_str(), "write")?;
    
    todo!("Actual filesystem write implementation in Block 8")
}

/// Filesystem execute operation with capability check.
#[host_function]
pub fn filesystem_execute(path: String) -> Result<i32, CapabilityError> {
    require_capability!(Filesystem, path.as_str(), "execute")?;
    
    todo!("Actual filesystem execute implementation in Block 8")
}
```

### Pattern 2: Network Host Functions

```rust
/// Network connect operation with capability check.
///
/// # WIT Declaration
///
/// ```wit
/// network-connect: func(endpoint: string) -> capability-result<tcp-stream>
/// ```
#[host_function]
pub fn network_connect(endpoint: String) -> Result<TcpStreamHandle, CapabilityError> {
    require_capability!(Network, endpoint.as_str(), "connect")?;
    
    todo!("Actual network connect implementation in Block 8")
}

/// Network bind operation with capability check.
#[host_function]
pub fn network_bind(endpoint: String) -> Result<TcpListenerHandle, CapabilityError> {
    require_capability!(Network, endpoint.as_str(), "bind")?;
    
    todo!("Actual network bind implementation in Block 8")
}

/// Network listen operation with capability check.
#[host_function]
pub fn network_listen(endpoint: String) -> Result<TcpListenerHandle, CapabilityError> {
    require_capability!(Network, endpoint.as_str(), "listen")?;
    
    todo!("Actual network listen implementation in Block 8")
}
```

### Pattern 3: Storage Host Functions

```rust
/// Storage read operation with capability check.
///
/// # WIT Declaration
///
/// ```wit
/// storage-read: func(key: string) -> capability-result<list<u8>>
/// ```
#[host_function]
pub fn storage_read(key: String) -> Result<Vec<u8>, CapabilityError> {
    require_capability!(Storage, key.as_str(), "read")?;
    
    todo!("Actual storage read implementation in Block 6")
}

/// Storage write operation with capability check.
#[host_function]
pub fn storage_write(key: String, value: Vec<u8>) -> Result<(), CapabilityError> {
    require_capability!(Storage, key.as_str(), "write")?;
    
    todo!("Actual storage write implementation in Block 6")
}

/// Storage delete operation with capability check.
#[host_function]
pub fn storage_delete(key: String) -> Result<(), CapabilityError> {
    require_capability!(Storage, key.as_str(), "delete")?;
    
    todo!("Actual storage delete implementation in Block 6")
}
```

---

## Implementation Steps (15 Steps, ~16-20 hours)

### Step 1: Create Host Integration Module (30 min)
- Create `airssys-wasm/src/security/host_integration.rs`
- Add module declaration to `security/mod.rs`
- Add 3-layer imports (§2.1)
- Define module-level rustdoc
- **Checkpoint**: `cargo check` passes

### Step 2: Implement CapabilityError Types (1 hour)
- `CapabilityError` enum with 4 variants
- Error message formatting
- Conversion from CapabilityCheckResult
- 8 unit tests
- **Checkpoint**: CapabilityError tests pass

### Step 3: Create WIT Error Definitions (1 hour)
- Create `wit/core/errors.wit`
- Define `capability-error` variant
- Define `capability-result<T>` type
- Documentation comments
- **Checkpoint**: WIT file validates

### Step 4: Implement Component Context Management (1.5 hours)
- Thread-local storage for component ID
- `set_current_component_id()`
- `get_current_component_id()`
- `clear_current_component_id()`
- 8 unit tests (set, get, clear, thread-safety)
- **Checkpoint**: Context management tests pass

### Step 5: Implement require_capability! Macro (2 hours)
- Macro definition with hygiene
- Macro expansion logic
- Error handling
- 10 macro tests (positive, negative, edge cases)
- **Checkpoint**: Macro tests pass

### Step 6: Filesystem Integration Pattern (2 hours)
- `filesystem_read()` stub with capability check
- `filesystem_write()` stub with capability check
- `filesystem_execute()` stub with capability check
- 9 integration tests (3 per function)
- **Checkpoint**: Filesystem integration tests pass

### Step 7: Network Integration Pattern (2 hours)
- `network_connect()` stub with capability check
- `network_bind()` stub with capability check
- `network_listen()` stub with capability check
- 9 integration tests
- **Checkpoint**: Network integration tests pass

### Step 8: Storage Integration Pattern (1.5 hours)
- `storage_read()` stub with capability check
- `storage_write()` stub with capability check
- `storage_delete()` stub with capability check
- 9 integration tests
- **Checkpoint**: Storage integration tests pass

### Step 9: Mock Host Function Testing Framework (2 hours)
- Create mock host function framework
- Simulate component context
- Test capability check enforcement
- 6 framework tests
- **Checkpoint**: Mock framework operational

### Step 10: Comprehensive Integration Tests (2 hours)
- 10 positive tests (capability granted scenarios)
- 10 negative tests (capability denied scenarios)
- 10 edge case tests (missing context, concurrent access)
- **Checkpoint**: 30+ integration tests pass

### Step 11: Error Message Validation (1 hour)
- Test error message clarity
- Test error message detail (includes resource, permission)
- Test WIT error conversion
- 8 error validation tests
- **Checkpoint**: Error messages validated

### Step 12: Documentation (2 hours)
- Module-level rustdoc with integration examples
- Function rustdoc for all host function patterns
- Macro documentation with examples
- Integration guide for future host functions
- **Checkpoint**: Zero rustdoc warnings

### Step 13: Examples (1.5 hours)
- `examples/security_host_function_filesystem.rs`
- `examples/security_host_function_network.rs`
- `examples/security_host_function_storage.rs`
- **Checkpoint**: All examples compile and run

### Step 14: WIT Integration Testing (1 hour)
- Test WIT error type definitions
- Test WIT bindings generation
- Test component error handling
- 5 WIT integration tests
- **Checkpoint**: WIT integration validated

### Step 15: Final Quality Gates (30 min)
- `cargo clippy --all-targets` (zero warnings)
- `cargo test --all-targets` (all pass)
- `cargo doc --no-deps` (zero warnings)
- WIT validation (`wit-bindgen validate`)
- **Checkpoint**: All quality gates pass

---

## Test Plan (30+ Test Scenarios)

### Positive Tests (10 tests)

| Test ID | Scenario | Expected Output |
|---------|----------|-----------------|
| P01 | Filesystem read with valid capability | Ok(bytes) |
| P02 | Network connect with valid capability | Ok(stream) |
| P03 | Storage write with valid capability | Ok(()) |
| P04 | Macro expansion correct | Compiles and runs |
| P05 | Component context set/get | Correct component ID |
| P06 | Multiple capabilities (all granted) | Ok for all operations |
| P07 | Concurrent host function calls | Thread-safe |
| P08 | Error conversion to WIT type | Correct WIT error |
| P09 | Mock host function with capability | Ok(result) |
| P10 | Host function pattern reuse | Consistent across categories |

### Negative Tests (10 tests)

| Test ID | Scenario | Expected Output |
|---------|----------|----------------|
| N01 | Filesystem read without capability | Err(AccessDenied) |
| N02 | Network connect without capability | Err(AccessDenied) |
| N03 | Storage write without capability | Err(AccessDenied) |
| N04 | Missing component context | Err(SecurityError) |
| N05 | Invalid resource pattern | Err(InvalidResource) |
| N06 | Component not registered | Err(ComponentNotFound) |
| N07 | Empty component ID | Err(SecurityError) |
| N08 | Null resource string | Err(InvalidResource) |
| N09 | Permission mismatch | Err(AccessDenied) |
| N10 | Capability check bypass attempt | Err(AccessDenied) |

### Edge Case Tests (10 tests)

| Test ID | Scenario | Expected Behavior |
|---------|----------|-------------------|
| E01 | Very long resource path (1000 chars) | Handle correctly |
| E02 | Unicode in resource path | Handle correctly |
| E03 | Concurrent component context changes | Thread-isolated |
| E04 | Component context not cleared | Leak detection |
| E05 | Macro used outside host function | Compile error or runtime error |
| E06 | Multiple macro invocations same function | All succeed |
| E07 | Error message contains full context | Detailed error |
| E08 | WIT error serialization | Correct format |
| E09 | Host function called recursively | Thread-safe context |
| E10 | Mock framework with 100+ host functions | Scalable |

---

## Quality Gates

### Cargo Clippy Requirements
- **Command**: `cargo clippy --all-targets --all-features -- -D warnings`
- **Target**: Zero warnings (deny warnings)
- **Enforced Lints**: `unwrap_used`, `expect_used`, `panic` (deny)

### Rustdoc Requirements
- **Command**: `cargo doc --no-deps --document-private-items`
- **Target**: Zero rustdoc warnings
- **Standards**: Microsoft Rust Guidelines (M-MODULE-DOCS, M-CANONICAL-DOCS)

### Test Coverage Targets
- **Unit Test Coverage**: >90% (all integration logic)
- **Integration Test Coverage**: 30+ integration tests
- **Macro Test Coverage**: 10+ macro expansion tests
- **Total Tests**: 50+ test cases

### WIT Validation
- **Command**: `wit-bindgen validate wit/`
- **Target**: All WIT files valid
- **Error Types**: Correctly defined and documented

---

## Timeline Estimate

| Step | Description | Time | Cumulative |
|------|-------------|------|------------|
| 1 | Host integration module | 30 min | 30 min |
| 2 | CapabilityError types | 1 hour | 1.5 hours |
| 3 | WIT error definitions | 1 hour | 2.5 hours |
| 4 | Component context management | 1.5 hours | 4 hours |
| 5 | require_capability! macro | 2 hours | 6 hours |
| 6 | Filesystem integration | 2 hours | 8 hours |
| 7 | Network integration | 2 hours | 10 hours |
| 8 | Storage integration | 1.5 hours | 11.5 hours |
| 9 | Mock framework | 2 hours | 13.5 hours |
| 10 | Integration tests | 2 hours | 15.5 hours |
| 11 | Error message validation | 1 hour | 16.5 hours |
| 12 | Documentation | 2 hours | 18.5 hours |
| 13 | Examples | 1.5 hours | 20 hours |
| 14 | WIT integration testing | 1 hour | 21 hours |
| 15 | Final quality gates | 30 min | **21.5 hours** |

**Total Duration**: 21.5 hours ≈ **2-3 days** (8 hour workdays)

**Breakdown by Activity**:
- Core implementation: 12 hours (56%)
- Testing: 5.5 hours (26%)
- Documentation: 3.5 hours (16%)
- Quality assurance: 0.5 hours (2%)

---

## Integration Points

### Task 3.1 Integration (Capability Check API)

```rust
// Task 3.2 uses Task 3.1's check_capability() function via macro

require_capability!(Filesystem, path, "read")?;
// expands to:
check_capability(&REGISTRY, component_id, path, "read")
    .into_result()
    .map_err(CapabilityError::access_denied)?;
```

### Task 3.3 Integration (Audit Logging)

```rust
// Task 3.3 will log all capability checks performed by Task 3.2 host functions
// Integration happens at Task 3.1 level (check_capability logs to audit)
```

### Block 8 Integration (Host Functions Implementation)

```rust
// Future Block 8 will implement actual host function logic

#[host_function]
pub fn filesystem_read(path: String) -> Result<Vec<u8>, CapabilityError> {
    // Task 3.2: Capability check
    require_capability!(Filesystem, path.as_str(), "read")?;
    
    // Block 8: Actual implementation
    use airssys_osl::operations::filesystem::FileSystemOp;
    let operation = FileSystemOp::read_file(path);
    let result = operation.execute().await?;
    Ok(result)
}
```

---

## Risk Assessment

### Technical Risks

| Risk | Severity | Probability | Mitigation |
|------|----------|-------------|------------|
| **Macro Hygiene Issues** | Medium | Low | Use `$crate::` for all paths, comprehensive macro tests |
| **Thread-Local Context Leaks** | Medium | Medium | Clear context after each host function, leak detection tests |
| **WIT Error Mapping** | Low | Low | Clear error type definitions, bidirectional conversion tests |
| **Bypass Vulnerabilities** | High | Low | Comprehensive negative tests, security review |

---

## Standards Compliance

### PROJECTS_STANDARD.md
- §2.1: 3-layer import organization ✅
- §4.3: Module architecture (mod.rs only re-exports) ✅
- §5.1: Dependency management ✅
- §6.1: YAGNI principles (minimal API surface) ✅

### Microsoft Rust Guidelines
- M-DESIGN-FOR-AI: Clear API, extensive docs ✅
- M-CANONICAL-DOCS: Comprehensive public API docs ✅
- M-EXAMPLES: Examples for all integration patterns ✅

### ADR Compliance
- ADR-WASM-005: Capability-Based Security Model ✅
- ADR-WASM-010: Implementation Strategy ✅

---

## Approval Status

**Planner**: Memory Bank Planner  
**Date**: 2025-12-19  
**Status**: ⏳ **AWAITING APPROVAL**

This plan provides a comprehensive blueprint for implementing host function integration points with clear patterns, macro ergonomics, and production-ready documentation.

**Ready to Start:** Task 3.2 implementation can begin after Task 3.1 completion and user approval.

---

## Next Steps After Task 3.2

### Task 3.3: Audit Logging Integration (1-2 days)
- Integrate airssys-osl SecurityAuditLogger
- Log all capability checks (granted + denied)
- Include component context (ID, capability, resource, trust level)
- Structured audit log format (JSON)
- Minimal performance overhead (<100ns per log)
