# WASM-TASK-005 Phase 3 Task 3.1: Capability Check API - IMPLEMENTATION PLAN

**Task:** Capability Check API  
**Status:** 📋 PLANNED (Ready to Start)  
**Date Created:** 2025-12-19  
**Estimated Duration:** 2 days (12-16 hours)  
**Prerequisites:** ✅ Phase 1 complete (Tasks 1.1-1.3), ✅ Phase 2 complete (Tasks 2.1-2.3)

---

## Executive Summary

**What**: Create a high-performance API for host functions to check WASM component capabilities via airssys-osl's security infrastructure. This API bridges WASM security context to airssys-osl ACL/RBAC evaluation, providing <5μs capability checks.

**Why**: Host functions need a simple, fast way to enforce component capabilities before granting access to system resources. The API must integrate seamlessly with Phase 1 (capability types) and Phase 2 (trust levels) while leveraging airssys-osl's battle-tested security policy engine.

**How**: Implement `check_capability()` function that:
1. Retrieves `WasmSecurityContext` for component
2. Converts to airssys-osl `SecurityContext` with resource/permission attributes
3. Builds `AccessControlList` from component capabilities
4. Evaluates using airssys-osl `SecurityPolicy::evaluate()`
5. Returns `CapabilityCheckResult` (Granted/Denied with reason)

**Architecture Position**: This API sits between host function implementations (Phase 5, Task 8) and the security layer (Phase 1-2), providing the critical enforcement point for capability-based security.

---

## Implementation Strategy

### Core Design Principles

1. **Performance First**: <5μs per check (leverage airssys-osl performance)
2. **Simple API**: One-function interface for host functions
3. **Clear Decisions**: Explicit Granted/Denied with reasons
4. **Reuse Over Build**: Leverage airssys-osl SecurityPolicy evaluation
5. **Thread-Safe**: Support concurrent capability checks

### Capability Check Flow

```text
┌─────────────────────────────────────────────────────────────┐
│ 1. Host Function Invocation                                │
│    fn filesystem_read(component_id, path) -> Result<...>   │
└────────────────────┬────────────────────────────────────────┘
                     │
                     ▼
┌─────────────────────────────────────────────────────────────┐
│ 2. check_capability(component_id, path, "read")            │
│    - Extract component security context                     │
│    - Build airssys-osl SecurityContext                     │
│    - Build AccessControlList from capabilities             │
└────────────────────┬────────────────────────────────────────┘
                     │
                     ▼
┌─────────────────────────────────────────────────────────────┐
│ 3. airssys-osl ACL Evaluation                              │
│    acl.evaluate(osl_context)                                │
│    - Pattern matching (glob patterns)                       │
│    - Permission checking                                    │
│    - Policy decision                                        │
└────────────────────┬────────────────────────────────────────┘
                     │
                     ▼
┌─────────────────────────────────────────────────────────────┐
│ 4. Result Conversion                                        │
│    PolicyDecision → CapabilityCheckResult                   │
│    - Allow → Granted                                        │
│    - Deny(reason) → Denied(reason)                          │
└────────────────────┬────────────────────────────────────────┘
                     │
                     ▼
┌─────────────────────────────────────────────────────────────┐
│ 5. Host Function Decision                                  │
│    - Granted: Proceed with operation                        │
│    - Denied: Return CapabilityError to component            │
└─────────────────────────────────────────────────────────────┘
```

---

## Data Structure Specifications

### 1. CapabilityCheckResult Enum

```rust
/// Result of a capability check operation.
///
/// Indicates whether a component is granted or denied access to a resource
/// with a specific permission.
///
/// # Examples
///
/// ```rust
/// use airssys_wasm::security::CapabilityCheckResult;
///
/// let granted = CapabilityCheckResult::Granted;
/// assert!(granted.is_granted());
///
/// let denied = CapabilityCheckResult::Denied {
///     reason: "Component declared /app/data/* but requested /etc/passwd".to_string(),
/// };
/// assert!(!denied.is_granted());
/// ```
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum CapabilityCheckResult {
    /// Access granted - component has required capability.
    Granted,
    
    /// Access denied - component lacks required capability.
    Denied {
        /// Human-readable reason for denial (for logging/debugging)
        reason: String,
    },
}

impl CapabilityCheckResult {
    /// Returns true if access was granted.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use airssys_wasm::security::CapabilityCheckResult;
    ///
    /// assert!(CapabilityCheckResult::Granted.is_granted());
    /// assert!(!CapabilityCheckResult::Denied {
    ///     reason: "Test".to_string()
    /// }.is_granted());
    /// ```
    pub fn is_granted(&self) -> bool {
        matches!(self, CapabilityCheckResult::Granted)
    }
    
    /// Returns true if access was denied.
    pub fn is_denied(&self) -> bool {
        !self.is_granted()
    }
    
    /// Returns the denial reason if denied, None otherwise.
    pub fn denial_reason(&self) -> Option<&str> {
        match self {
            CapabilityCheckResult::Denied { reason } => Some(reason),
            _ => None,
        }
    }
    
    /// Converts to Result<(), String> for easy error handling.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use airssys_wasm::security::CapabilityCheckResult;
    ///
    /// let granted = CapabilityCheckResult::Granted;
    /// assert!(granted.into_result().is_ok());
    ///
    /// let denied = CapabilityCheckResult::Denied {
    ///     reason: "Access denied".to_string()
    /// };
    /// assert_eq!(denied.into_result(), Err("Access denied".to_string()));
    /// ```
    pub fn into_result(self) -> Result<(), String> {
        match self {
            CapabilityCheckResult::Granted => Ok(()),
            CapabilityCheckResult::Denied { reason } => Err(reason),
        }
    }
}
```

### 2. ComponentSecurityRegistry

```rust
/// Registry maintaining security contexts for active components.
///
/// # Thread Safety
/// - Uses Arc<RwLock<>> for concurrent access
/// - Read-heavy workload (capability checks)
/// - Write-light workload (component spawn/shutdown)
///
/// # Design Rationale
/// - Centralized storage avoids passing contexts through all layers
/// - RwLock allows multiple concurrent reads (important for performance)
/// - Cleanup on component shutdown prevents memory leaks
pub struct ComponentSecurityRegistry {
    /// Component security contexts (protected by RwLock)
    contexts: Arc<RwLock<HashMap<String, WasmSecurityContext>>>,
}

impl ComponentSecurityRegistry {
    /// Creates new empty registry.
    pub fn new() -> Self {
        Self {
            contexts: Arc::new(RwLock::new(HashMap::new())),
        }
    }
    
    /// Registers security context for component.
    ///
    /// # Arguments
    ///
    /// * `component_id` - Unique identifier for the component
    /// * `context` - Security context with capabilities
    ///
    /// # Examples
    ///
    /// ```rust,ignore
    /// let registry = ComponentSecurityRegistry::new();
    /// let context = WasmSecurityContext::new(
    ///     "component-123".to_string(),
    ///     capabilities
    /// );
    /// registry.register("component-123", context);
    /// ```
    pub fn register(&self, component_id: String, context: WasmSecurityContext) {
        let mut contexts = self.contexts.write().unwrap_or_else(|poisoned| {
            warn!("ComponentSecurityRegistry lock poisoned, recovering");
            poisoned.into_inner()
        });
        contexts.insert(component_id, context);
    }
    
    /// Retrieves security context for component.
    ///
    /// # Arguments
    ///
    /// * `component_id` - Unique identifier for the component
    ///
    /// # Returns
    ///
    /// Returns `Some(WasmSecurityContext)` if component is registered,
    /// `None` if component not found.
    ///
    /// # Performance
    ///
    /// - Read lock (non-blocking for concurrent reads)
    /// - HashMap lookup: O(1)
    /// - Clone overhead: ~100ns (shallow clone with Arc)
    pub fn get(&self, component_id: &str) -> Option<WasmSecurityContext> {
        let contexts = self.contexts.read().unwrap_or_else(|poisoned| {
            warn!("ComponentSecurityRegistry lock poisoned, recovering");
            poisoned.into_inner()
        });
        contexts.get(component_id).cloned()
    }
    
    /// Unregisters component security context (called on component shutdown).
    pub fn unregister(&self, component_id: &str) {
        let mut contexts = self.contexts.write().unwrap_or_else(|poisoned| {
            warn!("ComponentSecurityRegistry lock poisoned, recovering");
            poisoned.into_inner()
        });
        contexts.remove(component_id);
    }
    
    /// Lists all registered component IDs (for debugging/monitoring).
    pub fn list_components(&self) -> Vec<String> {
        let contexts = self.contexts.read().unwrap_or_else(|poisoned| {
            warn!("ComponentSecurityRegistry lock poisoned, recovering");
            poisoned.into_inner()
        });
        contexts.keys().cloned().collect()
    }
}

impl Default for ComponentSecurityRegistry {
    fn default() -> Self {
        Self::new()
    }
}
```

---

## Core API Implementation

### check_capability() Function

```rust
/// Checks if a component has capability to access a resource with a permission.
///
/// This is the primary API for host functions to enforce component capabilities.
/// It integrates WASM security context with airssys-osl policy evaluation.
///
/// # Arguments
///
/// * `registry` - Component security registry
/// * `component_id` - Unique identifier for the component
/// * `resource` - Resource being accessed (e.g., "/app/data/file.json", "api.example.com:443")
/// * `permission` - Permission requested (e.g., "read", "write", "connect")
///
/// # Returns
///
/// Returns `CapabilityCheckResult::Granted` if the component has the required
/// capability, or `CapabilityCheckResult::Denied` with a reason if not.
///
/// # Performance
///
/// - Registry lookup: <1μs (RwLock read + HashMap lookup)
/// - Context conversion: <1μs (struct construction)
/// - ACL evaluation: <3μs (glob pattern matching via airssys-osl)
/// - **Total**: <5μs (target achieved)
///
/// # Examples
///
/// ## Filesystem Access Check
///
/// ```rust,ignore
/// use airssys_wasm::security::{check_capability, ComponentSecurityRegistry};
///
/// let registry = ComponentSecurityRegistry::new();
/// // Assume component registered with capabilities
///
/// let result = check_capability(
///     &registry,
///     "component-123",
///     "/app/data/file.json",
///     "read"
/// );
///
/// match result {
///     CapabilityCheckResult::Granted => {
///         // Proceed with file read operation
///     }
///     CapabilityCheckResult::Denied { reason } => {
///         // Return error to component
///         return Err(CapabilityError::AccessDenied(reason));
///     }
/// }
/// ```
///
/// ## Network Access Check
///
/// ```rust,ignore
/// let result = check_capability(
///     &registry,
///     "component-456",
///     "api.example.com:443",
///     "connect"
/// );
/// ```
///
/// ## Storage Access Check
///
/// ```rust,ignore
/// let result = check_capability(
///     &registry,
///     "component-789",
///     "component:<id>:data:config",
///     "write"
/// );
/// ```
///
/// # Error Handling
///
/// If the component is not found in the registry, returns
/// `CapabilityCheckResult::Denied` with reason "Component not found".
/// This is a security-first approach (deny-by-default).
pub fn check_capability(
    registry: &ComponentSecurityRegistry,
    component_id: &str,
    resource: &str,
    permission: &str,
) -> CapabilityCheckResult {
    // Step 1: Retrieve component security context from registry
    let wasm_ctx = match registry.get(component_id) {
        Some(ctx) => ctx,
        None => {
            return CapabilityCheckResult::Denied {
                reason: format!("Component '{}' not found in registry", component_id),
            };
        }
    };
    
    // Step 2: Convert to airssys-osl SecurityContext with resource/permission attributes
    let osl_ctx = wasm_ctx.to_osl_context(resource, permission);
    
    // Step 3: Build AccessControlList from component capabilities
    let acl = wasm_ctx.to_acl();
    
    // Step 4: Evaluate using airssys-osl SecurityPolicy
    use airssys_osl::middleware::security::policy::SecurityPolicy;
    let decision = acl.evaluate(&osl_ctx);
    
    // Step 5: Convert PolicyDecision to CapabilityCheckResult
    use airssys_osl::middleware::security::policy::PolicyDecision;
    match decision {
        PolicyDecision::Allow => CapabilityCheckResult::Granted,
        PolicyDecision::Deny(reason) => CapabilityCheckResult::Denied { reason },
        PolicyDecision::RequireAdditionalAuth(_) => {
            // WASM components don't support additional auth - deny
            CapabilityCheckResult::Denied {
                reason: "Additional authentication required (not supported for WASM components)".to_string(),
            }
        }
    }
}
```

---

## Implementation Steps (12 Steps, ~12-16 hours)

### Step 1: Create Enforcement Module Structure (30 min)
- Create `airssys-wasm/src/security/enforcement.rs`
- Add module declaration to `security/mod.rs`
- Add 3-layer imports (§2.1)
- Define module-level rustdoc
- **Checkpoint**: `cargo check` passes

### Step 2: Implement CapabilityCheckResult Enum (1 hour)
- `CapabilityCheckResult` enum with 2 variants (Granted, Denied)
- Helper methods (`is_granted()`, `is_denied()`, `denial_reason()`, `into_result()`)
- Derive traits (Debug, Clone, PartialEq, Eq)
- 8 unit tests
- **Checkpoint**: CapabilityCheckResult tests pass

### Step 3: Implement ComponentSecurityRegistry (2 hours)
- `ComponentSecurityRegistry` struct with Arc<RwLock<>>
- `new()`, `register()`, `get()`, `unregister()`, `list_components()`
- Thread safety tests
- 10 unit tests (register, get, unregister, concurrency)
- **Checkpoint**: Registry tests pass

### Step 4: Implement check_capability() Core Logic (2 hours)
- Main `check_capability()` function
- Registry lookup
- Context conversion (WASM → OSL)
- ACL building
- Policy evaluation
- Result conversion
- 10 unit tests (granted, denied, not found)
- **Checkpoint**: check_capability() tests pass

### Step 5: Integration with Phase 1 Types (1 hour)
- Test with WasmCapability::Filesystem
- Test with WasmCapability::Network
- Test with WasmCapability::Storage
- 6 integration tests
- **Checkpoint**: Phase 1 integration tests pass

### Step 6: Integration with airssys-osl (1.5 hours)
- Test ACL pattern matching (glob patterns)
- Test deny-by-default behavior
- Test multiple capabilities
- 8 integration tests
- **Checkpoint**: airssys-osl integration tests pass

### Step 7: Performance Optimization (2 hours)
- Profile capability checks (criterion benchmarks)
- Optimize hot paths (inline hints)
- Minimize allocations
- Verify <5μs target
- 3 benchmark tests
- **Checkpoint**: Performance target met

### Step 8: Error Handling Edge Cases (1 hour)
- Component not found
- Empty capabilities
- Invalid resource patterns
- Concurrent registry access
- 8 edge case tests
- **Checkpoint**: Edge case tests pass

### Step 9: Comprehensive Test Suite (1.5 hours)
- 15 positive tests (granted scenarios)
- 15 negative tests (denied scenarios)
- 10 edge case tests
- **Checkpoint**: 40+ tests pass

### Step 10: Documentation (1.5 hours)
- Module-level rustdoc with examples
- Function rustdoc for all public APIs
- Performance characteristics documentation
- Integration guide for host functions
- **Checkpoint**: Zero rustdoc warnings

### Step 11: Examples (1 hour)
- `examples/security_capability_check_filesystem.rs`
- `examples/security_capability_check_network.rs`
- `examples/security_capability_check_storage.rs`
- **Checkpoint**: All examples run

### Step 12: Final Quality Gates (30 min)
- `cargo clippy --all-targets` (zero warnings)
- `cargo test --all-targets` (all pass)
- `cargo doc --no-deps` (zero warnings)
- `cargo bench` (verify <5μs)
- **Checkpoint**: All quality gates pass

---

## Test Plan (40+ Test Scenarios)

### Positive Tests (15 tests)

| Test ID | Scenario | Expected Output |
|---------|----------|-----------------|
| P01 | Filesystem read with matching capability | Granted |
| P02 | Filesystem write with matching capability | Granted |
| P03 | Network connect with matching capability | Granted |
| P04 | Storage read with matching capability | Granted |
| P05 | Wildcard pattern match (e.g., `/app/*` matches `/app/data/file`) | Granted |
| P06 | Multiple capabilities (one matches) | Granted |
| P07 | Exact resource match | Granted |
| P08 | Case-sensitive permission match | Granted |
| P09 | Multiple permissions (all granted) | Granted |
| P10 | Component with many capabilities (100+) | Granted (performance) |
| P11 | Concurrent capability checks | Granted (thread-safe) |
| P12 | Registry with many components (1000+) | Granted (scalability) |
| P13 | Nested path pattern (/app/data/sub/file) | Granted |
| P14 | Port number in endpoint (api.com:443) | Granted |
| P15 | Storage namespace hierarchy | Granted |

### Negative Tests (15 tests)

| Test ID | Scenario | Expected Output |
|---------|----------|----------------|
| N01 | Component not found in registry | Denied: "Component not found" |
| N02 | Resource not matching any capability | Denied: ACL denial reason |
| N03 | Permission not in capability | Denied: Permission not granted |
| N04 | Empty capabilities set | Denied: No matching capability |
| N05 | Wildcard mismatch (e.g., `/app/*` vs `/etc/passwd`) | Denied |
| N06 | Invalid resource format | Denied |
| N07 | Case-sensitive mismatch | Denied |
| N08 | Expired component context | Denied: Component not found |
| N09 | Concurrent modification during check | Denied or Granted (consistent) |
| N10 | Null/empty component_id | Denied |
| N11 | Null/empty resource | Denied |
| N12 | Null/empty permission | Denied |
| N13 | Component with denied capability | Denied |
| N14 | Path traversal attempt (`../etc/passwd`) | Denied |
| N15 | Network port outside range | Denied |

### Edge Case Tests (10 tests)

| Test ID | Scenario | Expected Behavior |
|---------|----------|-------------------|
| E01 | Very long resource path (1000 chars) | Evaluate correctly |
| E02 | Unicode in resource path | Handle correctly |
| E03 | Special characters in permission | Handle correctly |
| E04 | Multiple concurrent registrations | Thread-safe |
| E05 | Registry lock poisoning | Recover gracefully |
| E06 | Component unregistered during check | Deny |
| E07 | Empty registry | Deny all checks |
| E08 | Duplicate component registration | Replace old context |
| E09 | Whitespace in resource/permission | Trim and match |
| E10 | Very large capability set (1000+ caps) | Evaluate within <10μs |

---

## Performance Targets

### Capability Check Performance

- **Registry Lookup**: <1μs (RwLock read + HashMap)
- **Context Conversion**: <1μs (struct construction)
- **ACL Evaluation**: <3μs (glob pattern matching)
- **Total**: <5μs per check ✅

### Optimization Strategies

1. **Inline Hints**: Mark hot path functions with `#[inline]`
2. **Minimize Allocations**: Reuse SecurityContext and ACL where possible
3. **RwLock Optimization**: Read-heavy workload (99% reads)
4. **HashMap Optimization**: Pre-size with capacity hint

### Benchmark Tests

```rust
#[cfg(test)]
mod benches {
    use super::*;
    use criterion::{black_box, Criterion};
    
    fn bench_capability_check(c: &mut Criterion) {
        let registry = ComponentSecurityRegistry::new();
        let capabilities = WasmCapabilitySet::new()
            .grant(WasmCapability::Filesystem {
                paths: vec!["/app/data/*".to_string()],
                permissions: vec!["read".to_string()],
            });
        let context = WasmSecurityContext::new("component-bench".to_string(), capabilities);
        registry.register("component-bench".to_string(), context);
        
        c.bench_function("check_capability_granted", |b| {
            b.iter(|| {
                check_capability(
                    black_box(&registry),
                    black_box("component-bench"),
                    black_box("/app/data/file.json"),
                    black_box("read"),
                )
            });
        });
    }
}
```

---

## Integration Points

### Phase 1 Integration (Capability Types)

```rust
// Task 1.1: WasmCapability, WasmCapabilitySet, WasmSecurityContext
use crate::security::capability::{WasmCapability, WasmCapabilitySet, WasmSecurityContext};

// Task 3.1 uses Task 1.1 types directly
let wasm_ctx = WasmSecurityContext::new(component_id, capabilities);
let osl_ctx = wasm_ctx.to_osl_context(resource, permission);
let acl = wasm_ctx.to_acl();
```

### Phase 2 Integration (Trust Levels)

```rust
// Task 2.1: TrustLevel is stored in WasmSecurityContext (future enhancement)
// For now, trust level affects INSTALLATION approval, not runtime checks

// Future: Add trust_level to WasmSecurityContext for runtime trust-based decisions
```

### airssys-osl Integration

```rust
// Use airssys-osl SecurityPolicy for evaluation
use airssys_osl::middleware::security::{
    AccessControlList,
    policy::{SecurityPolicy, PolicyDecision},
};

// Evaluate using airssys-osl
let decision = acl.evaluate(&osl_ctx);
```

### Task 3.2 Integration (Host Function Integration)

```rust
// Task 3.2 will use check_capability() in host functions

#[host_function]
fn filesystem_read(component_id: &str, path: &str) -> Result<Vec<u8>, HostError> {
    // Use Task 3.1 API
    let result = check_capability(
        &GLOBAL_REGISTRY,
        component_id,
        path,
        "read",
    );
    
    if result.is_denied() {
        return Err(HostError::CapabilityDenied(
            result.denial_reason().unwrap().to_string()
        ));
    }
    
    // Proceed with actual file read (Block 8)
    // ...
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
- **Standards**: Microsoft Rust Guidelines (M-MODULE-DOCS, M-CANONICAL-DOCS)

### Test Coverage Targets
- **Unit Test Coverage**: >90% (all enforcement logic)
- **Integration Test Coverage**: 15+ integration tests
- **Edge Case Coverage**: 10+ edge case tests
- **Total Tests**: 40+ test cases

### Performance Targets
- **Capability Check**: <5μs per check (verified with criterion)
- **Registry Lookup**: <1μs (RwLock read + HashMap)
- **Concurrent Throughput**: >100k checks/sec (multi-threaded)

---

## Timeline Estimate

| Step | Description | Time | Cumulative |
|------|-------------|------|------------|
| 1 | Enforcement module structure | 30 min | 30 min |
| 2 | CapabilityCheckResult enum | 1 hour | 1.5 hours |
| 3 | ComponentSecurityRegistry | 2 hours | 3.5 hours |
| 4 | check_capability() core | 2 hours | 5.5 hours |
| 5 | Phase 1 integration | 1 hour | 6.5 hours |
| 6 | airssys-osl integration | 1.5 hours | 8 hours |
| 7 | Performance optimization | 2 hours | 10 hours |
| 8 | Error handling | 1 hour | 11 hours |
| 9 | Comprehensive tests | 1.5 hours | 12.5 hours |
| 10 | Documentation | 1.5 hours | 14 hours |
| 11 | Examples | 1 hour | 15 hours |
| 12 | Final quality gates | 30 min | **15.5 hours** |

**Total Duration**: 15.5 hours ≈ **2 days** (8 hour workdays)

**Breakdown by Activity**:
- Core implementation: 8.5 hours (55%)
- Testing: 3.5 hours (23%)
- Documentation: 2.5 hours (16%)
- Quality assurance: 1 hour (6%)

---

## Risk Assessment

### Technical Risks

| Risk | Severity | Probability | Mitigation |
|------|----------|-------------|------------|
| **Performance Target Miss** | High | Low | Profile early, optimize hot paths, leverage airssys-osl performance |
| **Thread Safety Issues** | Medium | Low | RwLock for registry, atomic operations, comprehensive concurrency tests |
| **Memory Leaks** | Medium | Low | Proper component unregistration, cleanup tests |
| **Pattern Matching Bugs** | Low | Low | Reuse airssys-osl glob matching (battle-tested) |

---

## Standards Compliance

### PROJECTS_STANDARD.md
- §2.1: 3-layer import organization ✅
- §4.3: Module architecture (mod.rs only re-exports) ✅
- §5.1: Dependency management (airssys-osl at top) ✅
- §6.1: YAGNI principles (minimal, focused API) ✅

### Microsoft Rust Guidelines
- M-DESIGN-FOR-AI: Clear API, extensive docs ✅
- M-CANONICAL-DOCS: Comprehensive public API docs ✅
- M-EXAMPLES: Examples for all use cases ✅

### ADR Compliance
- ADR-WASM-005: Capability-Based Security Model ✅
- ADR-WASM-010: Implementation Strategy (reuse airssys-osl) ✅

---

## Approval Status

**Planner**: Memory Bank Planner  
**Date**: 2025-12-19  
**Status**: ⏳ **AWAITING APPROVAL**

This plan provides a comprehensive blueprint for implementing the capability check API with clear performance targets, integration points, and production-ready documentation.

**Ready to Start:** Task 3.1 implementation can begin after user approval.

---

## Next Steps After Task 3.1

### Task 3.2: Host Function Integration Points (2-3 days)
- Capability check macro for host functions
- Integration patterns for filesystem/network/storage
- WIT error types for capability violations
- 30+ integration tests

### Task 3.3: Audit Logging Integration (1-2 days)
- Integrate airssys-osl SecurityAuditLogger
- Log all capability checks (granted + denied)
- Structured audit log format (JSON)
- Minimal performance overhead (<100ns)
