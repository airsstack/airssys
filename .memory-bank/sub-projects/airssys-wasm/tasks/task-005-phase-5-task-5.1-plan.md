# Task 5.1: Security Integration Testing - Implementation Plan

**Task ID:** WASM-TASK-005-5.1  
**Phase:** Phase 5 - Testing & Documentation  
**Parent Task:** WASM-TASK-005 (Block 4 - Security & Isolation Layer)  
**Status:** 📋 PLANNING COMPLETE - READY FOR APPROVAL (Resource-Conscious Revision)  
**Created:** 2025-12-19  
**Revised:** 2025-12-19 (Scope adjustment for resource constraints)  
**Estimated Effort:** 2 days (16 hours)  

---

## Executive Summary

This plan defines **pragmatic, resource-conscious security testing** for the WASM-OSL Security Bridge. With Phases 1-4 complete (12,500+ lines, 362 tests, 96.8% quality), this task validates the security implementation through focused testing that applies the **80/20 rule** (20% effort = 80% security coverage):

- **~50 focused test cases** covering CRITICAL and COMMON security issues
- **10 bypass attempt tests** simulating high-impact attack vectors
- **5 lightweight performance benchmarks** (<2 minute runtime, 1,000 iterations)
- **Basic penetration scanner** with 5 attack scenarios (OWASP Top 3)

**Success Criteria:** Zero critical/high vulnerabilities in scope, all performance targets validated, production-ready security with pragmatic resource usage.

**Scope Philosophy:** This revision focuses on "good enough for production" rather than "perfect." It covers all CRITICAL security issues and COMMON attack patterns while deferring advanced/edge cases to future iterations. This is still production-ready security validation, just resource-optimized.

---

## Resource Constraints & Scope Philosophy

### Rationale for Resource-Conscious Approach

**Development Environment Constraints:**
- Limited CPU/memory resources on local development machine
- Need for fast iteration cycles (<2 minutes total test runtime)
- Pragmatic approach to initial production validation

**80/20 Security Principle:**
- 20% of test effort covers 80% of real-world security issues
- Focus on CRITICAL and COMMON attack vectors
- Defer advanced/edge case scenarios to future iterations
- **This is still production-ready security validation**

### Scope Adjustments Summary

| Category | Original Scope | Revised Scope | Reduction |
|----------|----------------|---------------|-----------|
| **Test Cases** | 100+ tests | ~50 tests | 50% |
| **Bypass Tests** | 20+ scenarios | 10 scenarios | 50% |
| **Benchmarks** | 10 benchmarks (5-10 min) | 5 benchmarks (<2 min) | 50% |
| **Benchmark Iterations** | 10,000-100,000 ops | 1,000 ops | 90% |
| **Penetration Framework** | Full framework, OWASP Top 10 | Basic scanner, OWASP Top 3 | 70% |
| **Estimated Effort** | 3 days (24 hours) | 2 days (16 hours) | 33% |
| **Code Volume** | ~5,600 lines | ~3,000 lines | 46% |

### What's Included (CRITICAL + COMMON)

**✅ Covered in This Task:**
1. **Filesystem security basics** (read, write, glob patterns)
2. **Network validation basics** (domain matching, port validation)
3. **Storage isolation basics** (namespace enforcement)
4. **Path traversal attacks** (../../../etc/passwd) - CRITICAL
5. **Privilege escalation** (capability override) - CRITICAL
6. **Quota bypass** (integer overflow) - COMMON
7. **Pattern bypass** (glob injection) - COMMON
8. **Trust level bypass** (DevMode abuse) - COMMON
9. **Performance target validation** (baseline verification, not profiling)
10. **OWASP Top 3** (Broken Access Control, Injection, Security Misconfiguration)

### What's Deferred (Advanced/Edge Cases)

**⏸️ Deferred to Future Iterations:**
1. **Advanced attack scenarios** (timing attacks, side-channels, cryptographic attacks)
2. **Fuzzing infrastructure** (automated input generation)
3. **Comprehensive penetration framework** (full OWASP Top 10, 20+ scenarios)
4. **Stress testing** (1M+ operations, multi-threaded concurrency)
5. **Memory profiling** (heap allocation tracking, leak detection)
6. **Regression suite** (historical performance comparison)
7. **Complex exploit chains** (multi-stage attacks)
8. **DoS/DDoS simulation** (resource exhaustion under load)
9. **Supply chain attack scenarios** (dependency poisoning)
10. **Percentile analysis** (p50/p95/p99 latency distribution)

**Justification:** These deferred items provide diminishing returns for initial production deployment. They're valuable for mature systems with real-world threat data but not essential for initial security validation.

---

## Context & Prerequisites

### Completed Implementation (Phases 1-4)

**Phase 1: WASM-OSL Security Bridge (✅ COMPLETE)**
- Files: `bridge.rs` (WasmCapability types), `parser.rs` (Component.toml), `capability.rs` (SecurityContext)
- Tests: 102 passing (`capability_mapping_tests.rs`)
- Deliverable: Capability mapping to airssys-osl ACL/RBAC

**Phase 2: Trust-Level System (✅ COMPLETE)**
- Files: `trust.rs` (TrustLevel, TrustRegistry), `approval.rs` (ApprovalWorkflow), `config.rs` (TrustConfig)
- Tests: 231 passing (71 trust + 96 approval + 64 config)
- Deliverable: Trust-based auto-approval workflows (Trusted/Unknown/DevMode)

**Phase 3: Capability Enforcement (✅ COMPLETE)**
- Files: `enforcement.rs` (CapabilityChecker), `host_integration.rs` (macros), `audit.rs` (SecurityAuditLogger)
- Tests: 47 passing (29 enforcement + 36 host + 11 audit)
- Deliverable: check_capability() API with audit logging

**Phase 4: ComponentActor Security Integration (✅ COMPLETE)**
- Files: `actor/security.rs` (context attachment), `security/quota.rs` (ResourceQuota)
- Tests: 100 passing (21 context + 63 quota)
- Deliverable: Per-component resource quotas (5 types: storage, message rate, network, CPU, memory)

### Architecture References

**ADR-WASM-005: Capability-Based Security Model**
- Fine-grained pattern matching (filesystem globs, network domains, storage namespaces)
- Trust-level system for approval workflows
- Layered integration with airssys-osl RBAC/ACL

**ADR-WASM-006: Component Isolation and Sandboxing**
- 4-layer defense in depth: Capability → WASM → Actor → Supervision
- ComponentActor dual-trait design with WASM lifecycle

**System Patterns (system-patterns.md)**
- Capability-Based Security Pattern (lines 79-135)
- Deny-by-default security enforcement

### Performance Targets

| Metric | Target | Current (Phase 3/4) |
|--------|--------|---------------------|
| Capability check latency | <5μs | 3-5μs ✅ |
| Quota check latency | <10μs | 3-5μs ✅ |
| Quota update latency | <5μs | 1-2μs ✅ |
| End-to-end permission check | <15μs | TBD (Task 5.1) |

---

## Goal

**Primary Objective:** Validate the security implementation through **pragmatic, focused testing** that covers CRITICAL and COMMON security issues, verifies performance targets with lightweight benchmarks, and confirms defense-in-depth layers are functioning correctly.

**Secondary Objectives:**
1. Identify and document critical/high security vulnerabilities before production
2. Establish baseline performance metrics for security operations (validation, not profiling)
3. Create simple, reusable security scanner for future iterations
4. Validate trust-level workflow effectiveness for essential use cases
5. **Optimize for fast iteration cycles and resource-conscious development**

---

## Implementation Plan

### Overview

The plan is organized into **6 deliverable streams** executed in **2 implementation days**:

**Day 1 (8 hours):** Security Test Suite (15 tests) + Bypass Part 1 (5 tests) = **20 tests**  
**Day 2 (8 hours):** Bypass Part 2 (5 tests) + Trust Workflows (10 tests) + Capability Mapping (10 tests) + Benchmarks (5) + Scanner (5 scenarios) = **30 tests + 5 benchmarks**

**Total Deliverables:**
- 5 test files (~2,000 lines)
- **~50 focused test cases** (CRITICAL + COMMON issues)
- 1 basic penetration scanner (~1,000 lines)
- 5 lightweight performance benchmarks (<2 min runtime)
- Updated documentation

**Resource-Conscious Approach:**
- Small iteration counts (1,000 ops, not 10,000+)
- Fast test execution (<5 minutes total)
- Focus on baseline validation (not exhaustive profiling)
- Basic scanner (not full framework)

---

## Deliverable 1: Security Test Suite (Focused on Essentials)

### Objective
Focused testing of essential capability types with basic positive and negative patterns. Covers CRITICAL use cases only.

### Test Scenarios (15 tests - REDUCED from 25)

#### Positive Tests (7 tests - Essential patterns only)
1. **Filesystem capabilities** (5 tests)
   - Single path exact match: `/app/data/config.json`
   - Glob pattern match: `/app/data/*.json`
   - Recursive wildcard: `/app/data/**/*.log`
   - Read permission validation
   - Write permission validation

2. **Network capabilities** (1 test)
   - Domain exact match: `api.example.com:443`

3. **Storage capabilities** (1 test)
   - Namespace exact match: `component:test-id:data:*`

#### Negative Tests (8 tests - Critical denials only)
1. **Filesystem denials** (4 tests)
   - Path outside pattern: `/etc/passwd` with `/app/*`
   - Path traversal attempt: `/app/../etc/passwd`
   - Empty path pattern
   - Invalid glob syntax: `/app/[unclosed`

2. **Network denials** (2 tests)
   - Endpoint not in whitelist
   - Port mismatch: `api.example.com:80` vs `:443`

3. **Storage denials** (1 test)
   - Namespace not in whitelist

4. **Permission denials** (1 test)
   - Read-only capability with write operation

**Removed/Deferred:**
- ❌ Complex network patterns (wildcard subdomain, port ranges)
- ❌ Multiple path/endpoint scenarios
- ❌ Cross-component access edge cases
- ❌ Execute permission testing

### Implementation

**File:** `tests/security_test_suite.rs` (~400 lines - REDUCED from 600)

**Structure:**
```rust
//! Security Test Suite - Focused Essentials (Task 5.1 - Deliverable 1)
//!
//! Covers CRITICAL and COMMON capability patterns:
//! - Filesystem (basic paths, globs, permissions)
//! - Network (basic endpoint validation)
//! - Storage (basic namespace isolation)
//! 
//! DEFERRED: Complex patterns, edge cases, multi-capability scenarios

mod positive_tests {
    mod filesystem_capabilities;  // 5 tests
    mod network_capabilities;     // 1 test
    mod storage_capabilities;     // 1 test
}

mod negative_tests {
    mod filesystem_denials;       // 4 tests
    mod network_denials;          // 2 tests
    mod storage_denials;          // 1 test
    mod permission_denials;       // 1 test
}
```

**Test Utilities:**
```rust
/// Helper to create test security context
fn create_test_context(id: &str, capabilities: WasmCapabilitySet) -> WasmSecurityContext {
    WasmSecurityContext::new(id.to_string(), capabilities)
}

/// Assert capability is granted
fn assert_granted(result: CapabilityCheckResult) {
    assert!(result.is_granted(), "Expected granted, got: {:?}", result);
}

/// Assert capability is denied
fn assert_denied(result: CapabilityCheckResult) {
    assert!(result.is_denied(), "Expected denied, got: {:?}", result);
}
```

**Estimated Effort:** 3 hours (Day 1 morning - REDUCED from 4 hours)

---

## Deliverable 2: Bypass Attempt Tests (CRITICAL + COMMON Attacks)

### Objective
Simulate adversarial components attempting CRITICAL and COMMON bypass attacks. Focus on high-impact, likely threat vectors.

### Attack Scenarios (10 tests - REDUCED from 20+)

#### Category 1: Path Traversal Attacks (2 tests - CRITICAL)
1. **Classic path traversal:** `/app/../../../etc/passwd`
2. **Absolute path injection:** Capability for `/app/*`, attempt `/etc/passwd`

**Removed/Deferred:**
- ❌ URL-encoded traversal
- ❌ Double-encoded traversal
- ❌ Null byte injection
- ❌ Symlink following

#### Category 2: Privilege Escalation Attempts (2 tests - CRITICAL)
1. **Capability inflation:** Register with `["read"]`, attempt `write`
2. **Cross-component access:** Component A accessing Component B's storage

**Removed/Deferred:**
- ❌ Trust level bypass (covered in Deliverable 3)
- ❌ Approval workflow skip (covered in Deliverable 3)
- ❌ DevMode exploitation (covered in Deliverable 3)

#### Category 3: Quota Manipulation (2 tests - COMMON)
1. **Quota exhaustion:** Rapid requests to exhaust rate limits
2. **Integer overflow:** Large values to cause overflow

**Removed/Deferred:**
- ❌ Quota reset bypass
- ❌ Negative quota values
- ❌ Concurrent quota drain (stress testing)

#### Category 4: Capability Pattern Vulnerabilities (2 tests - COMMON)
1. **Wildcard expansion:** `**/*` attempting system-wide access
2. **Empty pattern bypass:** Empty string as capability pattern

**Removed/Deferred:**
- ❌ Regex injection (low priority)
- ❌ Pattern collision (edge case)

#### Category 5: Trust Level Bypass (2 tests - COMMON)
1. **Trust source spoofing:** Unknown component claiming Trusted status
2. **DevMode abuse:** DevMode bypasses without proper warnings

### Implementation

**File:** `tests/security_bypass_tests.rs` (~450 lines - REDUCED from 800)

**Structure:**
```rust
//! Security Bypass Attempt Tests - CRITICAL + COMMON (Task 5.1 - Deliverable 2)
//!
//! Simulates high-impact malicious attacks. All tests MUST verify attacks are blocked.
//! 
//! Focus: CRITICAL (path traversal, privilege escalation) + COMMON (quota, patterns, trust)
//! DEFERRED: Advanced encoding, stress testing, complex exploit chains

mod path_traversal_attacks {
    // Classic traversal, absolute path injection
}

mod privilege_escalation {
    // Capability inflation, cross-component access
}

mod quota_manipulation {
    // Exhaustion, integer overflow
}

mod pattern_vulnerabilities {
    // Wildcard expansion, empty pattern bypass
}

mod trust_bypass {
    // Trust source spoofing, DevMode abuse
}
```

**Test Pattern:**
```rust
#[test]
fn test_path_traversal_blocked() {
    let checker = CapabilityChecker::new();
    
    // Grant limited filesystem access
    let capabilities = WasmCapabilitySet::new()
        .grant(WasmCapability::Filesystem {
            paths: vec!["/app/data/*".to_string()],
            permissions: vec!["read".to_string()],
        });
    
    let ctx = WasmSecurityContext::new("malicious-component", capabilities);
    checker.register_component(ctx).unwrap();
    
    // Attempt path traversal attack
    let result = checker.check("malicious-component", "/app/data/../../../etc/passwd", "read");
    
    // MUST be denied
    assert!(result.is_denied(), "Path traversal attack was not blocked!");
    
    // Verify audit log entry
    // (audit logging integration from Phase 3)
}
```

**Estimated Effort:** 3 hours (Day 1 afternoon + Day 2 morning - REDUCED from 5 hours)

---

## Deliverable 3: Trust Level Workflow Tests

### Objective
Validate trust-level system behavior for Trusted, Unknown, and DevMode components across approval workflows.

### Test Scenarios (10 tests - REDUCED from 15)

#### Trusted Component Workflows (5 tests)
1. **Instant approval:** Trusted source gets immediate approval
2. **Auto-renewal:** Approval renewal without user interaction
3. **Capability expansion:** Trusted component requesting new capabilities
4. **Trust revocation:** Component removed from trusted sources
5. **Trust inheritance:** Child components inheriting trust from parent

#### Unknown Component Approval Flows (5 tests)
1. **Pending state:** Unknown component enters pending approval
2. **Manual approval:** User grants approval, state transitions to approved
3. **Manual rejection:** User rejects, state transitions to rejected
4. **Timeout expiration:** Approval request expires after timeout
5. **Retry after rejection:** Rejected component requests approval again

#### DevMode Bypass Testing (5 tests)
1. **Warning emission:** DevMode bypasses approval but logs warning
2. **Temporary trust:** DevMode trust expires after session
3. **Capability warnings:** Each capability use logs warning in DevMode
4. **Production prevention:** DevMode disabled in production builds
5. **Audit trail:** All DevMode bypasses logged for security review

### Implementation

**File:** `tests/trust_level_workflow_tests.rs` (~500 lines)

**Structure:**
```rust
//! Trust Level Workflow Tests (Task 5.1 - Deliverable 3)
//!
//! Validates trust-level system (Trusted/Unknown/DevMode) across approval workflows.

mod trusted_workflows {
    // Instant approval, auto-renewal, capability expansion, revocation, inheritance
}

mod unknown_workflows {
    // Pending state, manual approval/rejection, timeout, retry
}

mod devmode_workflows {
    // Warning emission, temporary trust, capability warnings, production prevention, audit
}
```

**Test Utilities:**
```rust
/// Create TrustRegistry with test configuration
fn create_test_trust_registry() -> TrustRegistry {
    let config = TrustConfig::builder()
        .add_trusted_source("github.com/trusted-org/*")
        .approval_timeout(Duration::from_secs(300))
        .build();
    TrustRegistry::new(config)
}

/// Simulate approval workflow state transitions
async fn simulate_approval(
    workflow: &mut ApprovalWorkflow,
    component_id: &str,
    action: ApprovalAction,
) -> ApprovalResult {
    workflow.process_approval(component_id, action).await
}
```

**Integration Points:**
- Uses `trust.rs` (TrustLevel, TrustRegistry)
- Uses `approval.rs` (ApprovalWorkflow, ApprovalState)
- Uses `config.rs` (TrustConfig parser)

**Estimated Effort:** 3 hours (Day 2 morning)

---

## Deliverable 4: Capability Mapping Tests (WASM → ACL/RBAC)

### Objective
Verify correctness of WasmCapability → AclEntry conversion and pattern matching validation.

### Test Scenarios (10 tests - REDUCED from 20)

#### Mapping Correctness (8 tests)
1. **Filesystem → AclEntry:**
   - WasmCapability::Filesystem → AclEntry::FileRead/FileWrite
   - Path pattern → ACL resource pattern
   - Permission list → ACL permission flags

2. **Network → AclEntry:**
   - WasmCapability::Network → AclEntry::NetworkConnect/NetworkBind
   - Endpoint pattern → ACL network resource
   - Port ranges → ACL port restrictions

3. **Storage → AclEntry:**
   - WasmCapability::Storage → AclEntry::Custom("storage")
   - Namespace pattern → ACL storage key
   - CRUD operations → ACL permission flags

4. **Custom → AclEntry:**
   - WasmCapability::Custom → AclEntry::Custom
   - Arbitrary data → JSON metadata
   - Extension points → ACL extensibility

#### Pattern Matching Validation (6 tests)
1. **Glob patterns:**
   - Single wildcard: `*.json`
   - Recursive wildcard: `**/*.log`
   - Character classes: `[a-z]*.txt`

2. **Regex patterns:**
   - Anchored patterns: `^/app/.*$`
   - Optional components: `/app/(data|logs)/.*`
   - Negation: `^(?!/etc/).*`

3. **Exact matches:**
   - Case sensitivity
   - Special character escaping
   - Unicode support

#### Edge Cases (6 tests)
1. **Empty patterns:** Empty string, null, whitespace-only
2. **Invalid patterns:** Unclosed brackets, invalid regex, malformed globs
3. **Circular dependencies:** Pattern A depends on Pattern B depends on Pattern A
4. **Large patterns:** 10,000+ character patterns (DoS prevention)
5. **Special characters:** Spaces, tabs, newlines, control characters
6. **Unicode edge cases:** Emoji, RTL text, combining characters

### Implementation

**File:** `tests/capability_mapping_validation_tests.rs` (~700 lines)

**Structure:**
```rust
//! Capability Mapping Validation Tests (Task 5.1 - Deliverable 4)
//!
//! Verifies WasmCapability → AclEntry conversion and pattern matching.

mod mapping_correctness {
    mod filesystem_to_acl;
    mod network_to_acl;
    mod storage_to_acl;
    mod custom_to_acl;
}

mod pattern_matching {
    mod glob_patterns;
    mod regex_patterns;
    mod exact_matches;
}

mod edge_cases {
    mod empty_patterns;
    mod invalid_patterns;
    mod circular_dependencies;
    mod large_patterns;
    mod special_characters;
    mod unicode_edge_cases;
}
```

**Test Example:**
```rust
#[test]
fn test_filesystem_capability_to_acl_entry() {
    use airssys_wasm::security::bridge::capability_to_acl_entry;
    use airssys_osl::security::{AclEntry, Permission};
    
    let wasm_cap = WasmCapability::Filesystem {
        paths: vec!["/app/data/*.json".to_string()],
        permissions: vec!["read".to_string(), "write".to_string()],
    };
    
    let acl_entries = capability_to_acl_entry(&wasm_cap);
    
    assert_eq!(acl_entries.len(), 2); // read + write
    assert!(acl_entries.iter().any(|e| matches!(e, AclEntry::FileRead { .. })));
    assert!(acl_entries.iter().any(|e| matches!(e, AclEntry::FileWrite { .. })));
    
    // Verify pattern conversion
    for entry in &acl_entries {
        match entry {
            AclEntry::FileRead { pattern, .. } | AclEntry::FileWrite { pattern, .. } => {
                assert_eq!(pattern, "/app/data/*.json");
            }
            _ => panic!("Unexpected ACL entry type"),
        }
    }
}
```

**Estimated Effort:** 4 hours (Day 2 afternoon)

---

## Deliverable 5: Performance Benchmarks (Resource-Conscious)

### Objective
**Baseline validation** of security operation performance with lightweight benchmarks (<2 minute total runtime). Focus on verifying targets are met, not exhaustive profiling.

### Resource-Conscious Approach
- **Small iterations:** 1,000 ops (not 10,000-100,000)
- **Quick validation:** Verify targets met (not percentile analysis)
- **Fast execution:** <2 minutes total runtime
- **Baseline only:** No stress testing, concurrency, or memory profiling

### Benchmark Categories (5 benchmarks - REDUCED from 10)

#### 1. Capability Check Latency
**Target:** <5μs  
**Scenario:** Check one filesystem capability against pattern  
**Iterations:** 1,000  
**Expected:** 3-5μs (based on Phase 3 results)

```rust
fn benchmark_capability_check(c: &mut Criterion) {
    let checker = setup_capability_checker();
    c.bench_function("capability_check_single", |b| {
        b.iter_batched(
            || ("/app/data/config.json", "read"),
            |(path, perm)| checker.check("test-component", path, perm),
            BatchSize::SmallInput,
        );
    });
}
```

#### 2. Quota Check Overhead
**Target:** <10μs  
**Scenario:** Check single storage quota  
**Iterations:** 1,000  
**Expected:** 3-5μs (based on Phase 4 Task 4.3 results)

```rust
fn benchmark_quota_check(c: &mut Criterion) {
    let tracker = setup_quota_tracker();
    c.bench_function("quota_check_storage", |b| {
        b.iter(|| tracker.check_storage("test-component", 1024));
    });
}
```

#### 3. End-to-End Permission Check
**Target:** <15μs  
**Scenario:** Capability check + Quota check + Audit log  
**Iterations:** 1,000  
**Expected:** 10-15μs (sum of capability + quota + audit overhead)

```rust
fn benchmark_end_to_end_permission(c: &mut Criterion) {
    let security_ctx = setup_security_context();
    c.bench_function("end_to_end_permission", |b| {
        b.iter(|| {
            security_ctx.check_permission(
                "test-component",
                "/app/data/file.json",
                "write",
                1024, // bytes
            )
        });
    });
}
```

#### 4. Security Context Lookup
**Target:** <1μs  
**Scenario:** Lookup ComponentActor security context from registry  
**Iterations:** 1,000  
**Expected:** <1μs (DashMap O(1) lookup from Phase 3)

```rust
fn benchmark_security_context_lookup(c: &mut Criterion) {
    let registry = setup_security_registry();
    c.bench_function("security_context_lookup", |b| {
        b.iter(|| registry.get_context("test-component"));
    });
}
```

#### 5. Trust Level Determination
**Target:** <5μs  
**Scenario:** Determine trust level from component source  
**Iterations:** 1,000  
**Expected:** <5μs (Phase 2 TrustRegistry lookup)

```rust
fn benchmark_trust_level_determination(c: &mut Criterion) {
    let trust_registry = setup_trust_registry();
    c.bench_function("trust_level_determination", |b| {
        b.iter(|| {
            trust_registry.determine_trust_level(
                "https://github.com/trusted-org/component"
            )
        });
    });
}
```

### Removed/Deferred Benchmarks
**❌ Deferred to future iterations:**
- Multiple capability checks (not critical for baseline)
- Worst-case checks (10 capabilities, complex globs)
- Quota update latency (already validated in Phase 4)
- Multi-quota checks (all 5 types)
- Approval workflow lookup (not performance-critical)
- ACL evaluation deep-dive (covered by airssys-osl)
- Stress testing (1M+ operations)
- Concurrency benchmarks (multi-threaded)
- Memory profiling (heap allocations)
- Percentile analysis (p50/p95/p99)

### Implementation

**File:** `benches/security_integration_benchmarks.rs` (~250 lines - REDUCED from 400)

**Criterion Configuration (Resource-Conscious):**
```rust
use criterion::{criterion_group, criterion_main, Criterion, BenchmarkId, BatchSize};

fn configure_criterion() -> Criterion {
    Criterion::default()
        .sample_size(100)              // Small sample (not 1000)
        .measurement_time(Duration::from_secs(10))  // 10 sec per benchmark
        .warm_up_time(Duration::from_secs(2))       // 2 sec warm-up
        .without_plots()               // Skip plot generation
}

criterion_group! {
    name = security_benchmarks;
    config = configure_criterion();
    targets = 
        benchmark_capability_check,
        benchmark_quota_check,
        benchmark_end_to_end_permission,
        benchmark_security_context_lookup,
        benchmark_trust_level_determination
}

criterion_main!(security_benchmarks);
```

**Expected Runtime:**
- Warm-up: 2 sec × 5 = 10 sec
- Measurement: 10 sec × 5 = 50 sec
- **Total: ~60 seconds (<2 minutes)** ✅

### Success Criteria
- ✅ All 5 benchmarks run without errors
- ✅ All targets met (capability <5μs, quota <10μs, end-to-end <15μs, context <1μs, trust <5μs)
- ✅ Total runtime <2 minutes
- ✅ Results documented in completion summary

**Estimated Effort:** 2 hours (Day 2 afternoon - REDUCED from 3 hours)

---

## Deliverable 6: Penetration Testing Framework

### Objective
Create automated security vulnerability scanning framework for continuous security testing.

### Framework Architecture

```
penetration-testing/
├── framework/
│   ├── scanner.rs          // Vulnerability scanner engine
│   ├── attacks.rs          // Attack scenario library
│   ├── reporter.rs         // Vulnerability report generator
│   └── config.rs           // Framework configuration
├── scenarios/
│   ├── path_traversal.rs   // Path traversal attack scenarios
│   ├── privilege_esc.rs    // Privilege escalation scenarios
│   ├── injection.rs        // Injection attack scenarios
│   ├── dos.rs              // Denial-of-service scenarios
│   └── custom.rs           // Custom attack scenarios
├── reports/
│   └── vulnerability_report.json  // Generated reports
└── tests/
    └── framework_tests.rs  // Framework unit tests
```

### Components

#### 1. Vulnerability Scanner Engine (`scanner.rs`)

**Responsibilities:**
- Execute attack scenarios against security system
- Track successful/blocked attacks
- Generate vulnerability reports
- Support parallel scenario execution

**API:**
```rust
pub struct VulnerabilityScanner {
    scenarios: Vec<Box<dyn AttackScenario>>,
    config: ScannerConfig,
}

pub trait AttackScenario: Send + Sync {
    fn name(&self) -> &str;
    fn category(&self) -> AttackCategory;
    fn execute(&self, target: &SecuritySystem) -> AttackResult;
    fn severity(&self) -> Severity;
}

pub enum AttackResult {
    Blocked,           // Attack successfully blocked ✅
    Vulnerability(VulnerabilityReport),  // Security issue found ❌
    Error(String),     // Test execution error
}

pub enum Severity {
    Critical,  // Immediate fix required
    High,      // Fix before production
    Medium,    // Fix in next sprint
    Low,       // Document and monitor
}
```

#### 2. Attack Scenario Library (`attacks.rs`)

**Scenarios (20+ built-in):**

1. **Path Traversal Attacks:**
   - Classic traversal: `../../../etc/passwd`
   - URL-encoded: `%2e%2e%2f`
   - Null byte: `\0`
   - Symlink following

2. **Privilege Escalation:**
   - Capability inflation
   - Cross-component access
   - Trust level bypass
   - Quota manipulation

3. **Injection Attacks:**
   - Regex injection
   - Glob pattern injection
   - Command injection (if applicable)
   - SQL injection (storage layer)

4. **Denial-of-Service:**
   - Quota exhaustion
   - Pattern complexity explosion
   - Memory exhaustion
   - CPU exhaustion

5. **Authentication/Authorization:**
   - Session hijacking
   - Token forgery
   - Approval bypass
   - Trust source spoofing

**Example Scenario:**
```rust
pub struct PathTraversalAttack {
    variants: Vec<String>,  // Different traversal patterns
}

impl AttackScenario for PathTraversalAttack {
    fn name(&self) -> &str {
        "Path Traversal Attack"
    }
    
    fn category(&self) -> AttackCategory {
        AttackCategory::PathTraversal
    }
    
    fn execute(&self, target: &SecuritySystem) -> AttackResult {
        for variant in &self.variants {
            let result = target.check_filesystem_access(
                "malicious-component",
                variant,
                "read",
            );
            
            if result.is_granted() {
                return AttackResult::Vulnerability(VulnerabilityReport {
                    name: self.name().to_string(),
                    severity: Severity::Critical,
                    description: format!("Path traversal succeeded: {}", variant),
                    recommendation: "Strengthen path normalization and validation",
                });
            }
        }
        
        AttackResult::Blocked
    }
    
    fn severity(&self) -> Severity {
        Severity::Critical
    }
}
```

#### 3. Report Generator (`reporter.rs`)

**Output Formats:**
- JSON (machine-readable)
- Markdown (human-readable)
- JUnit XML (CI/CD integration)

**Report Structure:**
```rust
pub struct PenetrationTestReport {
    pub timestamp: DateTime<Utc>,
    pub duration: Duration,
    pub total_scenarios: usize,
    pub blocked: usize,
    pub vulnerabilities: Vec<VulnerabilityReport>,
    pub errors: Vec<String>,
    pub summary: ReportSummary,
}

pub struct VulnerabilityReport {
    pub name: String,
    pub category: AttackCategory,
    pub severity: Severity,
    pub description: String,
    pub recommendation: String,
    pub cve_references: Vec<String>,  // If applicable
}

pub struct ReportSummary {
    pub pass: bool,  // true if zero critical/high vulnerabilities
    pub critical_count: usize,
    pub high_count: usize,
    pub medium_count: usize,
    pub low_count: usize,
}
```

#### 4. Framework Configuration (`config.rs`)

**Configuration Options:**
```rust
pub struct ScannerConfig {
    pub parallel_execution: bool,
    pub max_concurrent: usize,
    pub timeout_per_scenario: Duration,
    pub fail_fast: bool,  // Stop on first vulnerability
    pub report_format: ReportFormat,
    pub severity_threshold: Severity,  // Min severity to report
}
```

### Usage Example

```rust
// tests/penetration_testing_integration.rs

#[test]
fn run_security_penetration_test() {
    let scanner = VulnerabilityScanner::builder()
        .add_scenario(PathTraversalAttack::new())
        .add_scenario(PrivilegeEscalationAttack::new())
        .add_scenario(QuotaManipulationAttack::new())
        .config(ScannerConfig {
            parallel_execution: true,
            max_concurrent: 10,
            timeout_per_scenario: Duration::from_secs(30),
            fail_fast: false,
            report_format: ReportFormat::Json,
            severity_threshold: Severity::Low,
        })
        .build();
    
    let security_system = create_test_security_system();
    let report = scanner.scan(&security_system);
    
    // Assert no critical or high vulnerabilities
    assert_eq!(report.summary.critical_count, 0, "Critical vulnerabilities found!");
    assert_eq!(report.summary.high_count, 0, "High vulnerabilities found!");
    
    // Generate report
    report.save_to_file("target/penetration-test-report.json").unwrap();
    
    println!("Penetration test complete: {} scenarios, {} blocked, {} vulnerabilities",
             report.total_scenarios, report.blocked, report.vulnerabilities.len());
}
```

### Implementation

**Files:**
- `tests/penetration_testing/framework/scanner.rs` (~300 lines)
- `tests/penetration_testing/framework/attacks.rs` (~400 lines)
- `tests/penetration_testing/framework/reporter.rs` (~200 lines)
- `tests/penetration_testing/framework/config.rs` (~100 lines)
- `tests/penetration_testing/scenarios/*.rs` (~500 lines total)
- `tests/penetration_testing_integration.rs` (~200 lines)

**Total:** ~1,700 lines

**Acceptance Criteria:**
- Framework executes all 20+ attack scenarios
- Zero false positives (blocked attacks reported as vulnerabilities)
- Report generation works in all formats (JSON, Markdown, JUnit)
- Framework is reusable for future security testing

**Estimated Effort:** 5 hours (Day 3 afternoon + buffer)

---

## Implementation Schedule

### Day 1 (8 hours): Positive/Negative Tests + Capability Mapping

**Morning (4 hours):**
- ✅ Deliverable 1: Security Test Suite (25 tests, 600 lines)
  - Positive tests: 12 tests (filesystem, network, storage)
  - Negative tests: 13 tests (denials, edge cases)

**Afternoon (4 hours):**
- ✅ Deliverable 2: Bypass Attempt Tests - Part 1 (10 tests, 400 lines)
  - Path traversal attacks: 6 tests
  - Privilege escalation: 4 tests

**Deliverables:** 2 test files, 35 tests, 1,000 lines

### Day 2 (8 hours): Bypass Tests + Trust Workflows + Mapping

**Morning (3 hours):**
- ✅ Deliverable 2: Bypass Attempt Tests - Part 2 (10 tests, 400 lines)
  - Quota manipulation: 5 tests
  - Capability vulnerabilities: 5 tests
- ✅ Deliverable 3: Trust Level Workflow Tests (15 tests, 500 lines)
  - Trusted workflows: 5 tests
  - Unknown workflows: 5 tests
  - DevMode workflows: 5 tests

**Afternoon (4 hours):**
- ✅ Deliverable 4: Capability Mapping Tests (20 tests, 700 lines)
  - Mapping correctness: 8 tests
  - Pattern matching: 6 tests
  - Edge cases: 6 tests

**Buffer (1 hour):** Testing, debugging, documentation

**Deliverables:** 2 test files, 45 tests, 1,600 lines

### Day 3 (8 hours): Performance + Penetration Framework

**Morning (3 hours):**
- ✅ Deliverable 5: Performance Benchmarks (10 benchmarks, 400 lines)
  - Capability check latency: 3 benchmarks
  - Quota check performance: 3 benchmarks
  - End-to-end permission: 4 benchmarks

**Afternoon (4 hours):**
- ✅ Deliverable 6: Penetration Testing Framework (1,700 lines)
  - Framework core: scanner, attacks, reporter, config (1,000 lines)
  - Attack scenarios: 20+ scenarios (500 lines)
  - Integration tests: framework validation (200 lines)

**Buffer (1 hour):** Final testing, documentation, report generation

**Deliverables:** 1 benchmark suite, 1 penetration framework, 2,100 lines

### Total Deliverables

- **Test Files:** 5 new test files (~3,500 lines)
- **Benchmark Suite:** 1 file (400 lines)
- **Penetration Framework:** 5 framework files + scenarios (1,700 lines)
- **Total Code:** ~5,600 lines
- **Total Tests:** 100+ test cases + 10 benchmarks + 20+ attack scenarios

---

## Success Criteria

### Functional Requirements ✅

1. **Test Coverage:**
   - ✅ 100+ test cases covering all capability types
   - ✅ 20+ bypass attempt scenarios
   - ✅ 15+ trust level workflow tests
   - ✅ 20+ capability mapping tests

2. **Security Validation:**
   - ✅ Zero security vulnerabilities found (Critical/High)
   - ✅ All bypass attempts successfully blocked
   - ✅ Trust level workflows function correctly
   - ✅ Capability mappings accurate and secure

3. **Performance:**
   - ✅ Capability checks: <5μs (current: 3-5μs)
   - ✅ Quota checks: <10μs (current: 3-5μs)
   - ✅ End-to-end permission: <15μs (to be measured)

4. **Framework:**
   - ✅ Penetration testing framework functional
   - ✅ 20+ attack scenarios executable
   - ✅ Reports generated in JSON/Markdown/JUnit formats

### Non-Functional Requirements ✅

1. **Code Quality:**
   - Zero compiler warnings
   - Zero clippy warnings
   - 100% rustdoc coverage for new public APIs
   - Microsoft Rust Guidelines compliance

2. **Documentation:**
   - All test files have module-level documentation
   - Complex tests have inline comments
   - Penetration framework has usage guide
   - Performance benchmark results documented

3. **Integration:**
   - All tests pass in CI/CD pipeline
   - Benchmarks run without errors
   - Penetration framework integrates with existing test suite

---

## Risk Assessment & Mitigation

### High-Risk Areas

**Risk 1: False Positives in Bypass Tests**
- **Impact:** High - May block legitimate use cases
- **Mitigation:** Carefully design attack scenarios with clear distinction between malicious and legitimate patterns
- **Validation:** Manual review of each bypass test scenario

**Risk 2: Performance Regression**
- **Impact:** Medium - Security checks may exceed performance targets
- **Mitigation:** Implement benchmarks early to catch regressions, use profiling tools
- **Contingency:** Optimize hot paths if targets not met

**Risk 3: Framework Complexity**
- **Impact:** Low - Penetration framework may be overcomplicated
- **Mitigation:** Start with simple scanner, add complexity incrementally
- **Validation:** Framework should be usable by developers without extensive documentation

### Medium-Risk Areas

**Risk 4: Edge Case Coverage**
- **Impact:** Medium - May miss obscure attack vectors
- **Mitigation:** Reference OWASP Top 10, CWE lists for common vulnerabilities
- **Validation:** Peer review of attack scenarios

**Risk 5: Integration with Existing Tests**
- **Impact:** Low - New tests may conflict with existing tests
- **Mitigation:** Use isolated test fixtures, avoid shared state
- **Validation:** Run full test suite after each deliverable

---

## Testing Strategy

### Unit Testing
- Each test scenario is self-contained
- Use test utilities for common setup/teardown
- Mock external dependencies (airssys-osl, airssys-rt)

### Integration Testing
- Test full security stack (capability → quota → audit)
- Use real CapabilityChecker, QuotaTracker instances
- Verify airssys-osl ACL integration

### Performance Testing
- Use `criterion` for statistical benchmarking
- Run benchmarks on dedicated hardware (no other processes)
- Compare results against baseline (Phase 3/4 metrics)

### Security Testing
- Penetration framework runs all attack scenarios
- Manual security review of critical paths
- Code audit for common vulnerabilities (OWASP Top 10)

---

## Documentation Requirements

### Test Documentation
1. **Module-level documentation** for each test file:
   - Purpose and scope
   - Test organization (categories)
   - Dependencies and prerequisites

2. **Test function documentation:**
   - Test objective (1 sentence)
   - Test scenario (steps)
   - Expected outcome

### Penetration Framework Documentation
1. **Framework architecture** (scanner, attacks, reporter)
2. **Usage guide** (creating custom scenarios)
3. **Report interpretation guide** (understanding severity levels)
4. **Integration guide** (CI/CD pipeline integration)

### Performance Documentation
1. **Benchmark results** (all 10 benchmarks)
2. **Performance analysis** (comparison to targets)
3. **Optimization notes** (if applicable)

---

## Dependencies

### External Dependencies
- **airssys-osl:** ACL/RBAC evaluation (Phase 1 integration)
- **airssys-rt:** Actor system for ComponentActor context (Phase 4)
- **criterion:** Benchmarking framework (existing)

### Internal Dependencies
- **Phase 1:** `security/bridge.rs`, `security/parser.rs`, `security/capability.rs`
- **Phase 2:** `security/trust.rs`, `security/approval.rs`, `security/config.rs`
- **Phase 3:** `security/enforcement.rs`, `security/host_integration.rs`, `security/audit.rs`
- **Phase 4:** `actor/security.rs`, `security/quota.rs`

### Test Utilities (Existing)
- Existing test files have reusable utilities (e.g., `create_test_context()`)
- Leverage existing test fixtures from Phases 1-4

---

## Verification & Validation

### Acceptance Testing

**Deliverable 1: Security Test Suite**
- [ ] All 25 tests pass
- [ ] Zero false positives (legitimate patterns not blocked)
- [ ] Zero false negatives (malicious patterns not caught)

**Deliverable 2: Bypass Attempt Tests**
- [ ] All 20+ bypass attempts blocked
- [ ] Audit logs generated for each attempt
- [ ] No security vulnerabilities found

**Deliverable 3: Trust Level Workflow Tests**
- [ ] All 15 trust workflow tests pass
- [ ] State transitions correct (Pending → Approved/Rejected)
- [ ] DevMode warnings logged

**Deliverable 4: Capability Mapping Tests**
- [ ] All 20 mapping tests pass
- [ ] WasmCapability → AclEntry conversion correct
- [ ] Pattern matching validation accurate

**Deliverable 5: Performance Benchmarks**
- [ ] All 10 benchmarks run successfully
- [ ] Performance targets met (or justified)
- [ ] No performance regressions vs. Phase 3/4

**Deliverable 6: Penetration Testing Framework**
- [ ] Framework executes all 20+ scenarios
- [ ] Reports generated in all formats
- [ ] Zero critical/high vulnerabilities found

### Code Review Checklist

- [ ] All tests follow existing test patterns
- [ ] No test code duplication (use utilities)
- [ ] Error messages are descriptive
- [ ] Performance benchmarks use `black_box()` correctly
- [ ] Penetration framework is extensible
- [ ] Documentation is complete and accurate

---

## Completion Checklist

### Implementation
- [ ] Deliverable 1: Security Test Suite (25 tests)
- [ ] Deliverable 2: Bypass Attempt Tests (20+ tests)
- [ ] Deliverable 3: Trust Level Workflow Tests (15 tests)
- [ ] Deliverable 4: Capability Mapping Tests (20 tests)
- [ ] Deliverable 5: Performance Benchmarks (10 benchmarks)
- [ ] Deliverable 6: Penetration Testing Framework (20+ scenarios)

### Testing & Validation
- [ ] All tests pass (100+ tests)
- [ ] All benchmarks run successfully
- [ ] Penetration framework completes without errors
- [ ] Zero security vulnerabilities found

### Code Quality
- [ ] Zero compiler warnings
- [ ] Zero clippy warnings
- [ ] 100% rustdoc coverage for new public APIs
- [ ] Microsoft Rust Guidelines compliance

### Documentation
- [ ] Test file module documentation complete
- [ ] Penetration framework usage guide written
- [ ] Performance benchmark results documented
- [ ] Security findings documented (if any)

### Integration
- [ ] All tests pass in CI/CD pipeline
- [ ] Benchmark suite integrated into CI/CD
- [ ] Penetration framework integrated into test suite

---

## References

### Architecture Decision Records
- **ADR-WASM-005:** Capability-Based Security Model (fine-grained patterns, trust-level system)
- **ADR-WASM-006:** Component Isolation and Sandboxing (4-layer defense)
- **ADR-WASM-010:** Implementation Strategy (security testing in Phase 5)

### System Patterns
- **system-patterns.md (lines 79-135):** Capability-Based Security Pattern
- **system-patterns.md (lines 199-228):** airssys-osl Integration Pattern

### Technical Context
- **tech-context.md (lines 163-185):** Security Architecture (sandbox, threat model)
- **tech-context.md (lines 333-337):** Security Testing (fuzzing, capability testing, penetration testing)

### Task Documentation
- **task-005-block-4-security-and-isolation-layer.md (lines 532-554):** Task 5.1 specification

### Standards
- **Microsoft Rust Guidelines:** Testing best practices, security guidelines
- **PROJECTS_STANDARD.md:** Workspace testing standards
- **OWASP Top 10:** Common web application security risks
- **CWE (Common Weakness Enumeration):** Software security weakness patterns

---

## Appendix A: Test File Structure

### Directory Layout
```
airssys-wasm/tests/
├── security_test_suite.rs                    # Deliverable 1 (600 lines)
├── security_bypass_tests.rs                  # Deliverable 2 (800 lines)
├── trust_level_workflow_tests.rs             # Deliverable 3 (500 lines)
├── capability_mapping_validation_tests.rs    # Deliverable 4 (700 lines)
└── penetration_testing/
    ├── framework/
    │   ├── scanner.rs                        # Deliverable 6 (300 lines)
    │   ├── attacks.rs                        # Deliverable 6 (400 lines)
    │   ├── reporter.rs                       # Deliverable 6 (200 lines)
    │   └── config.rs                         # Deliverable 6 (100 lines)
    ├── scenarios/
    │   ├── path_traversal.rs                 # Deliverable 6 (100 lines)
    │   ├── privilege_esc.rs                  # Deliverable 6 (100 lines)
    │   ├── injection.rs                      # Deliverable 6 (100 lines)
    │   ├── dos.rs                            # Deliverable 6 (100 lines)
    │   └── custom.rs                         # Deliverable 6 (100 lines)
    └── integration.rs                        # Deliverable 6 (200 lines)

airssys-wasm/benches/
└── security_integration_benchmarks.rs        # Deliverable 5 (400 lines)
```

---

## Appendix B: Performance Baseline

### Phase 3/4 Baseline Metrics

| Operation | Phase 3/4 Result | Task 5.1 Target | Status |
|-----------|------------------|-----------------|--------|
| Capability check (single) | 3-5μs | <5μs | ✅ On target |
| Quota check (storage) | 3-5μs | <10μs | ✅ Exceeds target |
| Quota update | 1-2μs | <5μs | ✅ Exceeds target |
| Capability check (multiple) | TBD | <10μs | 🎯 To measure |
| End-to-end permission | TBD | <15μs | 🎯 To measure |

**Note:** "TBD" metrics will be established in Task 5.1 benchmarks.

---

## Appendix C: Attack Scenario Checklist

### OWASP Top 10 Coverage

1. **A01:2021 – Broken Access Control**
   - ✅ Path traversal attacks
   - ✅ Privilege escalation attempts
   - ✅ Cross-component access

2. **A03:2021 – Injection**
   - ✅ Regex injection
   - ✅ Glob pattern injection
   - ✅ Command injection (if applicable)

3. **A04:2021 – Insecure Design**
   - ✅ Trust level bypass
   - ✅ Approval workflow skip
   - ✅ Capability inflation

4. **A05:2021 – Security Misconfiguration**
   - ✅ DevMode exploitation
   - ✅ Empty pattern bypass
   - ✅ Trust source spoofing

5. **A07:2021 – Identification and Authentication Failures**
   - ✅ Session hijacking
   - ✅ Token forgery

6. **A08:2021 – Software and Data Integrity Failures**
   - ✅ Quota manipulation
   - ✅ Integer overflow

7. **A09:2021 – Security Logging and Monitoring Failures**
   - ✅ Audit log bypass (verify all security events logged)

---

**END OF IMPLEMENTATION PLAN**

---

**Status:** 📋 PLANNING COMPLETE - AWAITING APPROVAL  
**Next Step:** User review and approval before implementation  
**Estimated Total Effort:** 3 days (24 hours)  
**Deliverables:** 5 test files, 1 benchmark suite, 1 penetration framework, 100+ tests, ~5,600 lines  
**Success Criteria:** Zero security vulnerabilities, all performance targets met, comprehensive coverage
