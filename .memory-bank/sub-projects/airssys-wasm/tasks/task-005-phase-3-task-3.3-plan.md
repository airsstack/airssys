# WASM-TASK-005 Phase 3 Task 3.3 - Implementation Plan

**Task:** Audit Logging Integration  
**Status:** Ready for Implementation  
**Created:** 2025-12-19  
**Estimated Effort:** 1-2 days (8-16 hours)  
**Priority:** HIGH - Critical Path (Phase 3 completion)

---

## Executive Summary

**Objective:** Integrate capability checks with airssys-osl audit logging for security monitoring and compliance.

**What:** Add structured audit logging to the `check_capability()` function using airssys-osl's `SecurityAuditLogger` trait. Log ALL capability checks (granted AND denied) with full context (component ID, resource, permission, timestamp, trust level, result).

**Why:** Security compliance requires comprehensive audit trails. Every capability check must be logged for forensic analysis, compliance reporting (GDPR, SOC2), and security monitoring.

**How:** 
1. Create WASM-specific audit logger implementation (`WasmAuditLogger`)
2. Integrate logging into `check_capability()` function (line 831-839 in enforcement.rs)
3. Build WASM-specific audit log format with component context
4. Implement async logging with <100ns overhead target
5. Test ALL capability checks are logged with correct context

**Success Criteria:**
- ✅ ALL capability checks logged (granted + denied)
- ✅ <100ns logging overhead (async, non-blocking)
- ✅ 30+ tests passing (log format, context completeness, performance)
- ✅ JSON audit log format for machine parsing
- ✅ Zero warnings (clippy + rustdoc + compiler)

---

## Context & Prerequisites

### Completed Prerequisites ✅

**Task 3.1 (Complete):** Capability Check API
- ✅ `check_capability()` function implemented (line 831-839)
- ✅ DashMap-based capability checker (<5μs performance)
- ✅ 3-parameter API (component_id, resource, permission)
- ✅ 29 tests passing

**Task 3.2 (Complete):** Host Function Integration Points
- ✅ `require_capability!` macro
- ✅ Thread-local component context management
- ✅ WIT error types (4 variants)
- ✅ 13 integration patterns
- ✅ 36 tests passing (29 enforcement + 7 host_integration)

**airssys-osl (Available):**
- ✅ `SecurityAuditLogger` trait (async logging interface)
- ✅ `SecurityAuditLog` struct (timestamp, event_type, decision, metadata)
- ✅ `ConsoleSecurityAuditLogger` (console output for dev/test)
- ✅ `SecurityEventType` enum (AccessGranted, AccessDenied, etc.)

### Integration Point

**Primary:** `check_capability()` function in `enforcement.rs` (line 831-839):

```rust
pub fn check_capability(
    component_id: &str,
    resource: &str,
    permission: &str,
) -> Result<(), CapabilityCheckError> {
    let result = global_checker()
        .check(component_id, resource, permission)
        .to_result();
    
    // NEW: Add audit logging HERE (Task 3.3)
    // audit_log_capability_check(component_id, resource, permission, &result)?;
    
    result
}
```

**Secondary:** `CapabilityChecker::check()` method (line 698-756) - internal logging point.

---

## Technical Design

### Architecture Overview

```
┌─────────────────────────────────────────────────────────────┐
│ Host Function (e.g., filesystem_read)                        │
│   └─> require_capability!(path, "read")?                     │
│         └─> check_capability(component_id, resource, perm)   │
│               ├─> CapabilityChecker::check() [enforcement]   │
│               │     ├─> Registry lookup (DashMap)            │
│               │     ├─> ACL evaluation (airssys-osl)         │
│               │     └─> Returns CapabilityCheckResult        │
│               │                                               │
│               └─> WasmAuditLogger::log_check() [NEW]         │
│                     ├─> Build WasmCapabilityAuditLog         │
│                     ├─> Convert to SecurityAuditLog (OSL)    │
│                     └─> Async log (SecurityAuditLogger)      │
└─────────────────────────────────────────────────────────────┘
```

### Component Design

#### 1. WASM-Specific Audit Log Type

**File:** `src/security/audit.rs` (NEW)

**Purpose:** WASM-specific audit log format with component context.

```rust
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

/// WASM capability check audit log entry.
///
/// Records all capability checks (granted and denied) with full context
/// for security monitoring, compliance, and forensic analysis.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WasmCapabilityAuditLog {
    /// Timestamp (§3.2 - chrono DateTime<Utc>)
    pub timestamp: DateTime<Utc>,
    
    /// Component ID
    pub component_id: String,
    
    /// Resource path/pattern
    pub resource: String,
    
    /// Permission requested (read, write, execute, etc.)
    pub permission: String,
    
    /// Check result (granted or denied)
    pub result: CapabilityCheckResultType,
    
    /// Trust level of component (if available)
    pub trust_level: Option<String>,
    
    /// Denial reason (if denied)
    pub denial_reason: Option<String>,
    
    /// Additional metadata (JSON)
    pub metadata: serde_json::Value,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum CapabilityCheckResultType {
    Granted,
    Denied,
}

impl WasmCapabilityAuditLog {
    /// Convert to airssys-osl SecurityAuditLog for unified logging.
    pub fn to_osl_audit_log(&self) -> SecurityAuditLog {
        // Implementation: Map WASM log to OSL format
    }
}
```

**Design Rationale:**
- **WASM-specific fields:** component_id, resource, permission, trust_level
- **Compatibility:** Converts to airssys-osl SecurityAuditLog for unified logging
- **Structured format:** JSON-serializable for machine parsing
- **Complete context:** All information needed for forensic analysis

#### 2. WASM Audit Logger Implementation

**File:** `src/security/audit.rs` (NEW)

**Purpose:** WASM-specific audit logger wrapping airssys-osl SecurityAuditLogger.

```rust
use std::sync::Arc;
use airssys_osl::middleware::security::audit::{SecurityAuditLogger, SecurityAuditLog};

/// WASM capability audit logger.
///
/// Logs all capability checks using airssys-osl SecurityAuditLogger
/// with WASM-specific context (component ID, resource, permission).
#[derive(Debug, Clone)]
pub struct WasmAuditLogger {
    /// Underlying OSL audit logger
    logger: Arc<dyn SecurityAuditLogger>,
}

impl WasmAuditLogger {
    /// Create new WASM audit logger wrapping an OSL logger.
    pub fn new(logger: Arc<dyn SecurityAuditLogger>) -> Self {
        Self { logger }
    }
    
    /// Log a capability check (granted or denied).
    pub async fn log_capability_check(
        &self,
        log: WasmCapabilityAuditLog,
    ) -> Result<(), CapabilityCheckError> {
        // Convert to OSL SecurityAuditLog
        let osl_log = log.to_osl_audit_log();
        
        // Async log (non-blocking)
        self.logger
            .log_security_event(osl_log)
            .await
            .map_err(|e| CapabilityCheckError::AuditLogError {
                reason: format!("Failed to log capability check: {}", e),
            })
    }
}
```

**Design Rationale:**
- **Composition over inheritance:** Wraps airssys-osl logger, doesn't reimplement
- **Async logging:** Non-blocking to meet <100ns overhead target
- **Unified logging:** Reuses airssys-osl infrastructure for consistency
- **Type-safe:** Strong typing prevents logging errors

#### 3. Global Audit Logger Management

**File:** `src/security/enforcement.rs` (MODIFY)

**Purpose:** Global audit logger instance (similar to global_checker pattern).

```rust
use std::sync::OnceLock;
use crate::security::audit::WasmAuditLogger;

// Global audit logger instance
static GLOBAL_AUDIT_LOGGER: OnceLock<WasmAuditLogger> = OnceLock::new();

/// Get global audit logger instance.
fn global_audit_logger() -> &'static WasmAuditLogger {
    GLOBAL_AUDIT_LOGGER.get_or_init(|| {
        // Default to console logger for development
        let console_logger = Arc::new(ConsoleSecurityAuditLogger::new());
        WasmAuditLogger::new(console_logger)
    })
}

/// Set global audit logger (for testing or custom implementations).
pub fn set_global_audit_logger(logger: WasmAuditLogger) -> Result<(), String> {
    GLOBAL_AUDIT_LOGGER.set(logger)
        .map_err(|_| "Global audit logger already set".to_string())
}
```

**Design Rationale:**
- **OnceLock pattern:** Same as global_checker (proven pattern)
- **Default logger:** Console logger for dev/test (easy to replace)
- **Configurable:** Can inject custom logger for production
- **Thread-safe:** OnceLock ensures single initialization

#### 4. Integration into check_capability()

**File:** `src/security/enforcement.rs` (MODIFY line 831-839)

**Purpose:** Add audit logging to capability check function.

```rust
pub fn check_capability(
    component_id: &str,
    resource: &str,
    permission: &str,
) -> Result<(), CapabilityCheckError> {
    // Step 1: Perform capability check
    let check_result = global_checker()
        .check(component_id, resource, permission);
    
    // Step 2: Build audit log
    let log = WasmCapabilityAuditLog {
        timestamp: Utc::now(),
        component_id: component_id.to_string(),
        resource: resource.to_string(),
        permission: permission.to_string(),
        result: match &check_result {
            CapabilityCheckResult::Granted => CapabilityCheckResultType::Granted,
            CapabilityCheckResult::Denied(_) => CapabilityCheckResultType::Denied,
        },
        trust_level: get_trust_level(component_id).ok(),  // Optional
        denial_reason: match &check_result {
            CapabilityCheckResult::Denied(reason) => Some(reason.clone()),
            _ => None,
        },
        metadata: serde_json::Value::Null,  // Extensible
    };
    
    // Step 3: Async log (non-blocking spawn)
    let logger = global_audit_logger().clone();
    tokio::spawn(async move {
        if let Err(e) = logger.log_capability_check(log).await {
            eprintln!("AUDIT LOG ERROR: {}", e);
        }
    });
    
    // Step 4: Return original check result
    check_result.to_result()
}
```

**Design Rationale:**
- **Non-blocking:** `tokio::spawn` ensures logging doesn't slow capability checks
- **Complete context:** All check details logged (granted + denied)
- **Error isolation:** Logging errors don't fail capability checks
- **Consistent:** Same pattern for all checks

#### 5. Trust Level Integration (Optional Enhancement)

**File:** `src/security/enforcement.rs` (MODIFY)

**Purpose:** Include trust level in audit logs (if available).

```rust
/// Get trust level for component (if registered in TrustRegistry).
fn get_trust_level(component_id: &str) -> Result<String, String> {
    // Query TrustRegistry (from Phase 2 Task 2.1)
    // Return "Trusted", "Unknown", or "DevMode"
    // If not available, return Err (optional field)
    
    // Placeholder for now (Task 2.1 integration)
    Err("Trust level not implemented".to_string())
}
```

**Design Rationale:**
- **Optional:** Trust level enriches logs but isn't required for checks
- **Phase 2 integration:** Uses TrustRegistry from Task 2.1
- **Graceful degradation:** Missing trust level doesn't break logging

---

## Implementation Steps

### Step 1: Create Audit Module (2 hours)

**File:** `src/security/audit.rs` (NEW, ~200 lines)

**Tasks:**
1. Create `WasmCapabilityAuditLog` struct
   - Fields: timestamp, component_id, resource, permission, result, trust_level, denial_reason, metadata
   - Derive: Debug, Clone, Serialize, Deserialize
   - Add rustdoc with examples

2. Create `CapabilityCheckResultType` enum
   - Variants: Granted, Denied
   - Derive: Debug, Clone, Serialize, Deserialize

3. Implement `to_osl_audit_log()` converter
   - Map WASM log to airssys-osl SecurityAuditLog
   - Set event_type based on result (AccessGranted/AccessDenied)
   - Build operation_id from component_id + resource
   - Include all context in metadata field

4. Create `WasmAuditLogger` struct
   - Field: `logger: Arc<dyn SecurityAuditLogger>`
   - Method: `new(logger: Arc<dyn SecurityAuditLogger>) -> Self`
   - Method: `log_capability_check(&self, log: WasmCapabilityAuditLog) -> Result<()>`
   - Add comprehensive rustdoc

5. Add module-level documentation
   - Explain audit logging purpose
   - Link to compliance requirements
   - Provide usage examples

**Acceptance Criteria:**
- ✅ Compiles without errors
- ✅ All types derive necessary traits
- ✅ Comprehensive rustdoc (>50 lines)
- ✅ Examples compile and demonstrate usage

**Estimated Time:** 2 hours

---

### Step 2: Integrate Global Audit Logger (1 hour)

**File:** `src/security/enforcement.rs` (MODIFY, +50 lines)

**Tasks:**
1. Add imports:
   ```rust
   use crate::security::audit::{WasmAuditLogger, WasmCapabilityAuditLog, CapabilityCheckResultType};
   use airssys_osl::middleware::security::audit::ConsoleSecurityAuditLogger;
   use tokio;
   ```

2. Add global audit logger:
   ```rust
   static GLOBAL_AUDIT_LOGGER: OnceLock<WasmAuditLogger> = OnceLock::new();
   ```

3. Implement `global_audit_logger()` function
   - Return &'static WasmAuditLogger
   - Initialize with ConsoleSecurityAuditLogger by default

4. Implement `set_global_audit_logger()` function
   - Allow custom logger injection
   - Return error if already set

5. Add rustdoc for global functions
   - Explain default logger (console)
   - Show how to inject custom logger
   - Provide testing examples

**Acceptance Criteria:**
- ✅ Compiles without errors
- ✅ Global logger initializes correctly
- ✅ Custom logger injection works
- ✅ Rustdoc complete with examples

**Estimated Time:** 1 hour

---

### Step 3: Modify check_capability() Function (1.5 hours)

**File:** `src/security/enforcement.rs` (MODIFY line 831-839, +30 lines)

**Tasks:**
1. Save original check result:
   ```rust
   let check_result = global_checker().check(component_id, resource, permission);
   ```

2. Build WasmCapabilityAuditLog:
   - Set timestamp: `Utc::now()`
   - Set component_id, resource, permission
   - Set result based on check_result
   - Set trust_level (optional, may be None for now)
   - Set denial_reason if denied

3. Spawn async logging task:
   ```rust
   let logger = global_audit_logger().clone();
   tokio::spawn(async move {
       if let Err(e) = logger.log_capability_check(log).await {
           eprintln!("AUDIT LOG ERROR: {}", e);
       }
   });
   ```

4. Return original check result:
   ```rust
   check_result.to_result()
   ```

5. Update rustdoc:
   - Document audit logging behavior
   - Explain async logging (non-blocking)
   - Show logged fields in examples

**Acceptance Criteria:**
- ✅ ALL capability checks logged (granted + denied)
- ✅ Original check result unchanged
- ✅ Async logging doesn't block
- ✅ Rustdoc updated with logging details

**Estimated Time:** 1.5 hours

---

### Step 4: Add Module Exports (15 minutes)

**File:** `src/security/mod.rs` (MODIFY, +10 lines)

**Tasks:**
1. Add module declaration:
   ```rust
   pub mod audit;
   ```

2. Add re-exports:
   ```rust
   pub use audit::{
       WasmAuditLogger,
       WasmCapabilityAuditLog,
       CapabilityCheckResultType,
   };
   
   pub use enforcement::{
       global_audit_logger,
       set_global_audit_logger,
   };
   ```

3. Update module-level rustdoc:
   - Add audit logging section
   - Link to audit module
   - Provide audit configuration examples

**Acceptance Criteria:**
- ✅ Exports accessible from `airssys_wasm::security`
- ✅ Rustdoc updated
- ✅ Examples compile

**Estimated Time:** 15 minutes

---

### Step 5: Add Cargo.toml Dependencies (15 minutes)

**File:** `airssys-wasm/Cargo.toml` (MODIFY)

**Tasks:**
1. Verify existing dependencies:
   - ✅ `tokio` (already present, needed for async logging)
   - ✅ `chrono` (already present, needed for timestamps)
   - ✅ `serde_json` (already present, needed for metadata)

2. Add if missing:
   ```toml
   [dependencies]
   tokio = { workspace = true, features = ["rt", "macros"] }
   ```

3. Verify airssys-osl dependency includes audit module:
   ```toml
   airssys-osl = { workspace = true }  # Already present
   ```

**Acceptance Criteria:**
- ✅ All dependencies available
- ✅ No version conflicts
- ✅ Workspace dependencies respected (§6.1)

**Estimated Time:** 15 minutes

---

### Step 6: Write Comprehensive Tests (3 hours)

**File:** `src/security/audit.rs` (MODIFY, +400 lines tests)

#### Test Categories

**A. Audit Log Creation Tests (5 tests)**

1. `test_audit_log_creation_granted` - Create log for granted check
2. `test_audit_log_creation_denied` - Create log for denied check
3. `test_audit_log_with_trust_level` - Include trust level
4. `test_audit_log_with_metadata` - Include custom metadata
5. `test_audit_log_timestamp` - Verify timestamp accuracy

**B. OSL Conversion Tests (5 tests)**

6. `test_to_osl_audit_log_granted` - Convert granted check to OSL format
7. `test_to_osl_audit_log_denied` - Convert denied check to OSL format
8. `test_osl_event_type_mapping` - Verify event type mapping
9. `test_osl_metadata_inclusion` - Verify metadata in OSL log
10. `test_osl_denial_reason` - Verify denial reason in OSL log

**C. WasmAuditLogger Tests (5 tests)**

11. `test_wasm_audit_logger_creation` - Create logger
12. `test_wasm_audit_logger_log_granted` - Log granted check
13. `test_wasm_audit_logger_log_denied` - Log denied check
14. `test_wasm_audit_logger_async` - Verify async logging
15. `test_wasm_audit_logger_error_handling` - Handle logging errors

**D. Integration Tests (10 tests)**

16. `test_check_capability_logs_granted` - Verify granted check logged
17. `test_check_capability_logs_denied` - Verify denied check logged
18. `test_check_capability_logs_all_fields` - Verify all fields present
19. `test_check_capability_logs_timestamp` - Verify timestamp accuracy
20. `test_check_capability_logs_component_id` - Verify component ID
21. `test_check_capability_logs_resource` - Verify resource path
22. `test_check_capability_logs_permission` - Verify permission
23. `test_check_capability_logs_denial_reason` - Verify denial reason
24. `test_check_capability_async_logging` - Verify non-blocking
25. `test_check_capability_logging_error_isolation` - Verify errors don't break checks

**E. Custom Logger Tests (5 tests)**

26. `test_set_global_audit_logger` - Set custom logger
27. `test_global_audit_logger_default` - Verify default logger
28. `test_custom_logger_receives_logs` - Verify custom logger called
29. `test_custom_logger_log_format` - Verify log format in custom logger
30. `test_custom_logger_error_handling` - Handle custom logger errors

**F. Performance Tests (2 tests)**

31. `test_audit_logging_overhead` - Measure <100ns overhead
32. `test_audit_logging_throughput` - Measure high-throughput scenario

**Total:** 32 tests (exceeds 30+ requirement)

**Test Infrastructure:**

```rust
#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::{Arc, Mutex};
    
    // Mock audit logger for testing
    #[derive(Debug, Default)]
    struct MockAuditLogger {
        logs: Arc<Mutex<Vec<SecurityAuditLog>>>,
    }
    
    impl MockAuditLogger {
        fn new() -> Self {
            Self {
                logs: Arc::new(Mutex::new(Vec::new())),
            }
        }
        
        fn get_logs(&self) -> Vec<SecurityAuditLog> {
            self.logs.lock().unwrap().clone()
        }
    }
    
    #[async_trait::async_trait]
    impl SecurityAuditLogger for MockAuditLogger {
        async fn log_security_event(&self, event: SecurityAuditLog) -> Result<(), AuditError> {
            self.logs.lock().unwrap().push(event);
            Ok(())
        }
    }
}
```

**Acceptance Criteria:**
- ✅ 32/32 tests passing
- ✅ >95% code coverage
- ✅ All edge cases tested
- ✅ Performance validated (<100ns)
- ✅ Zero warnings

**Estimated Time:** 3 hours

---

### Step 7: Add Performance Benchmarks (1 hour)

**File:** `benches/audit_logging_benchmarks.rs` (NEW, ~150 lines)

**Benchmarks:**

1. **audit_log_creation** - Measure WasmCapabilityAuditLog creation time
2. **osl_conversion** - Measure to_osl_audit_log() conversion time
3. **async_logging** - Measure async logging overhead
4. **end_to_end_check_with_logging** - Measure total overhead

**Target:** <100ns async logging overhead (non-blocking spawn time)

**Example Benchmark:**

```rust
use criterion::{black_box, criterion_group, criterion_main, Criterion};

fn audit_logging_overhead(c: &mut Criterion) {
    let rt = tokio::runtime::Runtime::new().unwrap();
    
    c.bench_function("check_capability_with_logging", |b| {
        b.iter(|| {
            rt.block_on(async {
                check_capability(
                    black_box("test-component"),
                    black_box("/app/data/file.json"),
                    black_box("read"),
                )
            })
        })
    });
}

criterion_group!(benches, audit_logging_overhead);
criterion_main!(benches);
```

**Acceptance Criteria:**
- ✅ Benchmarks compile and run
- ✅ <100ns overhead verified
- ✅ Baseline comparison available
- ✅ Results documented

**Estimated Time:** 1 hour

---

### Step 8: Update Documentation (1.5 hours)

**Files to Update:**

#### A. Module Documentation (30 minutes)

**File:** `src/security/audit.rs`

- Add module-level rustdoc (>100 lines)
- Explain audit logging purpose
- Provide complete usage examples
- Link to compliance requirements (GDPR, SOC2)
- Show custom logger implementation example

#### B. Integration Guide (30 minutes)

**File:** `src/security/enforcement.rs`

- Update check_capability() rustdoc
- Add audit logging section
- Show log format examples
- Explain async logging behavior

#### C. README Updates (30 minutes)

**File:** `README.md` (if security section exists)

- Add audit logging feature
- Explain log format
- Show configuration examples
- Link to compliance documentation

**Acceptance Criteria:**
- ✅ >200 lines of rustdoc added
- ✅ All public APIs documented
- ✅ Examples compile and run
- ✅ Zero rustdoc warnings

**Estimated Time:** 1.5 hours

---

### Step 9: Code Review & Quality Assurance (1 hour)

**Tasks:**

1. Run quality checks:
   ```bash
   cargo clippy --all-targets --all-features
   cargo doc --no-deps
   cargo test --all-features
   cargo bench --no-run
   ```

2. Review checklist:
   - ✅ Zero compiler warnings
   - ✅ Zero clippy warnings
   - ✅ Zero rustdoc warnings
   - ✅ All tests passing (32/32)
   - ✅ Benchmarks compile
   - ✅ Code review score >9.0/10

3. Standards compliance (§PROJECT_STANDARD.md):
   - ✅ Import organization (§2.1)
   - ✅ Error handling (§4.3)
   - ✅ Module architecture (§5.1)
   - ✅ Dependency management (§6.1)
   - ✅ DateTime<Utc> usage (§3.2)

4. Final verification:
   - ✅ ALL capability checks logged
   - ✅ Performance target met (<100ns)
   - ✅ Documentation complete
   - ✅ No regressions (785+ tests still passing)

**Acceptance Criteria:**
- ✅ All quality gates passed
- ✅ Standards compliance verified
- ✅ Code review approved (>9.0/10)
- ✅ Ready for production

**Estimated Time:** 1 hour

---

## File Changes Summary

### New Files (2 files, ~350 lines)

| File | Lines | Purpose |
|------|-------|---------|
| `src/security/audit.rs` | ~250 | Audit log types, WasmAuditLogger, tests |
| `benches/audit_logging_benchmarks.rs` | ~100 | Performance benchmarks |

### Modified Files (3 files, ~100 lines added)

| File | Changes | Lines Added |
|------|---------|-------------|
| `src/security/enforcement.rs` | Add global logger, modify check_capability() | ~80 |
| `src/security/mod.rs` | Add audit module exports | ~10 |
| `Cargo.toml` | Verify dependencies (likely no changes) | ~0 |

**Total Code:** ~450 lines (250 new + 80 modified + 100 benchmarks + 20 tests)

---

## Test Plan

### Test Coverage Matrix

| Category | Tests | Coverage Target | Status |
|----------|-------|-----------------|--------|
| Audit Log Creation | 5 | 100% | Planned |
| OSL Conversion | 5 | 100% | Planned |
| WasmAuditLogger | 5 | 100% | Planned |
| Integration Tests | 10 | >95% | Planned |
| Custom Logger Tests | 5 | 100% | Planned |
| Performance Tests | 2 | Validate targets | Planned |
| **Total** | **32** | **>95%** | **Planned** |

### Test Execution Plan

1. **Unit tests** (Steps 6A-6C): Test individual components
2. **Integration tests** (Step 6D): Test check_capability() logging
3. **Custom logger tests** (Step 6E): Test logger injection
4. **Performance tests** (Step 6F + Step 7): Validate <100ns overhead
5. **Regression tests**: Verify existing 785 tests still pass

### Test Data

**Valid Test Cases:**
- Granted capability check with all fields
- Denied capability check with denial reason
- Check with trust level (trusted, unknown, devmode)
- Check with custom metadata
- High-throughput scenario (1000+ checks/sec)

**Edge Cases:**
- Missing trust level (optional field)
- Logging error (error isolation test)
- Concurrent logging (thread safety)
- Empty metadata
- Long resource paths (>1000 chars)

**Performance Cases:**
- Single check overhead (<100ns)
- Bulk checks (1000 checks, measure throughput)
- Concurrent checks (10 threads, verify no contention)

---

## Performance Targets

### Target Metrics

| Metric | Target | Measurement Method |
|--------|--------|-------------------|
| **Async logging overhead** | <100ns | tokio::spawn time |
| **Log creation time** | <50ns | WasmCapabilityAuditLog::new() |
| **OSL conversion time** | <50ns | to_osl_audit_log() |
| **Total overhead** | <200ns | End-to-end benchmark |
| **Throughput** | >10,000 checks/sec | Bulk check benchmark |

### Optimization Strategies

1. **Async spawn:** Use `tokio::spawn` for non-blocking logging
2. **Clone minimization:** Clone only when necessary (Arc for logger)
3. **Lazy evaluation:** Don't build log if logging disabled (future)
4. **Structured logging:** Pre-allocate strings where possible
5. **Batch logging:** (Future) Batch multiple logs for efficiency

### Performance Validation

**Benchmark Command:**
```bash
cargo bench --bench audit_logging_benchmarks
```

**Expected Output:**
```
check_capability_with_logging    time: [180.23 ns 185.67 ns 191.34 ns]
audit_log_creation               time: [42.15 ns 44.89 ns 47.56 ns]
osl_conversion                   time: [38.92 ns 41.23 ns 43.67 ns]
```

---

## Standards Compliance

### Workspace Standards (PROJECTS_STANDARD.md)

#### §2.1: Import Organization ✅

```rust
// Layer 1: Standard library
use std::sync::{Arc, OnceLock};

// Layer 2: Third-party crates (alphabetical)
use async_trait::async_trait;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use tokio;

// Layer 3: Internal modules (grouped by module)
use crate::security::enforcement::{CapabilityCheckError, CapabilityCheckResult};
use airssys_osl::middleware::security::audit::{SecurityAuditLogger, SecurityAuditLog};
```

#### §3.2: DateTime<Utc> Usage ✅

```rust
pub struct WasmCapabilityAuditLog {
    pub timestamp: DateTime<Utc>,  // ✅ Correct: chrono DateTime<Utc>
    // ...
}

// ✅ Correct: Use Utc::now()
let log = WasmCapabilityAuditLog {
    timestamp: Utc::now(),
    // ...
};
```

#### §4.3: Error Handling ✅

```rust
// ✅ Use CapabilityCheckError for audit log errors
pub enum CapabilityCheckError {
    // ... existing variants ...
    
    #[error("Audit log error: {reason}")]
    AuditLogError { reason: String },
}

// ✅ Propagate errors with ?
logger.log_capability_check(log).await?;
```

#### §5.1: Module Architecture ✅

```rust
// src/security/
// ├── mod.rs           (module declarations, exports)
// ├── enforcement.rs   (capability checking)
// ├── audit.rs         (audit logging) ✅ NEW
// └── host_integration.rs (host function patterns)
```

#### §6.1: Dependency Management ✅

```toml
[dependencies]
# Workspace dependencies (already present)
tokio = { workspace = true }
chrono = { workspace = true }
serde = { workspace = true }
airssys-osl = { workspace = true }
```

### Microsoft Rust Guidelines ✅

- **Error Handling:** Use thiserror for custom errors
- **Async Safety:** Use tokio::spawn for non-blocking
- **Documentation:** Comprehensive rustdoc with examples
- **Testing:** >95% code coverage
- **Performance:** Benchmark all critical paths

---

## Risk Assessment & Mitigation

### Technical Risks

#### Risk 1: Logging Overhead >100ns (MEDIUM)

**Mitigation:**
- Use `tokio::spawn` for async non-blocking logging
- Benchmark early and optimize if needed
- Consider lazy evaluation (build log only if logging enabled)

#### Risk 2: Logging Errors Break Capability Checks (LOW)

**Mitigation:**
- Isolate logging errors (catch in async task, don't propagate)
- Log to stderr if audit logging fails
- Capability check result is independent of logging

#### Risk 3: Missing Trust Level Data (LOW)

**Mitigation:**
- Make trust_level optional (Option<String>)
- Gracefully handle missing data
- Document optional nature in rustdoc

#### Risk 4: Log Storage Exhaustion (MEDIUM - Production)

**Mitigation:**
- Use log rotation (airssys-osl responsibility)
- Document log retention policies
- Provide configuration for log storage limits

### Mitigation Summary

| Risk | Likelihood | Impact | Mitigation | Residual Risk |
|------|------------|--------|------------|---------------|
| Logging overhead | Medium | Medium | Async spawn | Low |
| Logging errors | Low | Medium | Error isolation | Low |
| Missing trust level | Low | Low | Optional field | None |
| Log exhaustion | Medium | High | Rotation + config | Medium |

---

## Dependencies & Integration Points

### Completed Dependencies ✅

1. **Task 3.1:** Capability Check API (complete)
2. **Task 3.2:** Host Function Integration Points (complete)
3. **airssys-osl:** SecurityAuditLogger trait (available)

### Integration Points

#### 1. airssys-osl SecurityAuditLogger

**Status:** ✅ Available  
**Location:** `airssys-osl/src/middleware/security/audit.rs`

**Interface:**
```rust
#[async_trait]
pub trait SecurityAuditLogger: Debug + Send + Sync + 'static {
    async fn log_security_event(&self, event: SecurityAuditLog) -> Result<(), AuditError>;
    async fn flush(&self) -> Result<(), AuditError>;
}
```

**Usage:** Wrap with WasmAuditLogger, call log_security_event()

#### 2. TrustRegistry (Phase 2 Task 2.1)

**Status:** ✅ Complete  
**Location:** `airssys-wasm/src/security/trust.rs`

**Integration:** Query trust level for component (optional field in audit log)

#### 3. check_capability() Function

**Status:** ✅ Available  
**Location:** `airssys-wasm/src/security/enforcement.rs` (line 831-839)

**Modification:** Add audit logging call (Step 3)

---

## Timeline & Estimates

### Total Estimated Effort: 12 hours (1.5 days)

| Step | Task | Duration | Dependencies |
|------|------|----------|--------------|
| 1 | Create Audit Module | 2 hours | None |
| 2 | Integrate Global Audit Logger | 1 hour | Step 1 |
| 3 | Modify check_capability() | 1.5 hours | Step 1, 2 |
| 4 | Add Module Exports | 15 min | Step 1 |
| 5 | Add Cargo Dependencies | 15 min | None |
| 6 | Write Comprehensive Tests | 3 hours | Step 1-5 |
| 7 | Add Performance Benchmarks | 1 hour | Step 1-5 |
| 8 | Update Documentation | 1.5 hours | Step 1-7 |
| 9 | Code Review & QA | 1 hour | Step 1-8 |

**Critical Path:** Steps 1 → 2 → 3 → 6 → 9 (8.5 hours minimum)

**Parallel Opportunities:**
- Steps 4, 5 can be done anytime
- Step 7 can be done after Step 3
- Step 8 can be done incrementally

**Recommended Schedule:**
- **Day 1 (6 hours):** Steps 1-5 (core implementation)
- **Day 2 (6 hours):** Steps 6-9 (testing, benchmarks, QA)

---

## Acceptance Criteria

### Functional Requirements ✅

- [ ] ALL capability checks logged (granted + denied)
- [ ] Audit logs include full context (component_id, resource, permission, timestamp, result)
- [ ] Trust level included in logs (if available)
- [ ] Denial reason included in logs (if denied)
- [ ] Async logging doesn't block capability checks
- [ ] Logging errors don't break capability checks

### Performance Requirements ✅

- [ ] <100ns logging overhead (async spawn time)
- [ ] >10,000 checks/sec throughput
- [ ] No regression in existing check_capability() performance

### Quality Requirements ✅

- [ ] 32+ tests passing (>95% coverage)
- [ ] Zero compiler warnings
- [ ] Zero clippy warnings
- [ ] Zero rustdoc warnings
- [ ] Code review score >9.0/10

### Documentation Requirements ✅

- [ ] >200 lines of rustdoc added
- [ ] All public APIs documented
- [ ] Examples compile and run
- [ ] Integration guide complete

### Standards Compliance ✅

- [ ] Import organization (§2.1)
- [ ] Error handling (§4.3)
- [ ] Module architecture (§5.1)
- [ ] Dependency management (§6.1)
- [ ] DateTime<Utc> usage (§3.2)

### Integration Requirements ✅

- [ ] airssys-osl SecurityAuditLogger integrated
- [ ] TrustRegistry integration (optional field)
- [ ] check_capability() modified correctly
- [ ] No breaking changes to existing APIs

---

## References

### Related Documentation

- **Main Task:** `task-005-block-4-security-and-isolation-layer.md` (Block 4 plan)
- **Task 3.1:** `task-005-phase-3-task-3.1-completion.md` (Capability Check API)
- **Task 3.2:** `task-005-phase-3-task-3.2-completion.md` (Host Function Integration)
- **DashMap Migration:** `knowledge-wasm-023-dashmap-migration-rationale.md` (Registry design)
- **ADR-WASM-005:** Capability-Based Security Model

### Code Locations

- **Enforcement:** `airssys-wasm/src/security/enforcement.rs` (check_capability at line 831)
- **OSL Audit:** `airssys-osl/src/middleware/security/audit.rs` (SecurityAuditLogger trait)
- **Trust Registry:** `airssys-wasm/src/security/trust.rs` (TrustRegistry from Phase 2)

### Standards References

- **PROJECTS_STANDARD.md:** §2.1 (imports), §3.2 (DateTime), §4.3 (errors), §5.1 (modules), §6.1 (deps)
- **Microsoft Rust Guidelines:** Error handling, async safety, documentation
- **Diátaxis Framework:** Documentation structure (explanations, how-to guides, reference)

---

## Next Steps After Completion

### Immediate (Task 3.3 Complete)

1. **Update _index.md:** Add Task 3.3 completion status
2. **Update progress.md:** Mark Phase 3 complete (3/3 tasks)
3. **Create completion summary:** Document results and lessons learned
4. **Start Task 4.1:** ComponentActor Security Context Attachment

### Phase 3 Completion Checklist

When Task 3.3 is complete, verify Phase 3 objectives:

- [ ] Task 3.1: Capability Check API ✅ COMPLETE
- [ ] Task 3.2: Host Function Integration Points ✅ COMPLETE
- [ ] Task 3.3: Audit Logging Integration ⏳ IN PROGRESS

**Phase 3 Success Criteria:**
- [ ] All capability checks enforced (<5μs)
- [ ] Host functions integrate via macro (one-line checks)
- [ ] All checks audited (granted + denied)
- [ ] >95% test coverage (Phase 3 tests)
- [ ] Zero warnings across Phase 3 code
- [ ] Documentation complete

### Phase 4 Preview

**Next Phase:** ComponentActor Security Integration

**Task 4.1:** ComponentActor Security Context Attachment
- Attach WasmSecurityContext to each ComponentActor
- Capability set isolation per component
- Security context restoration after restart

---

## Appendix A: Example Audit Log Output

### JSON Format (Console Logger)

```json
{
  "timestamp": "2025-12-19T10:30:45.123456Z",
  "event_type": "AccessGranted",
  "operation_id": "wasm-component-abc123::/app/data/file.json::read",
  "principal": "wasm-component-abc123",
  "session_id": "550e8400-e29b-41d4-a716-446655440000",
  "decision": "Allow",
  "policy_applied": "WasmCapabilityCheck",
  "metadata": {
    "wasm_component_id": "wasm-component-abc123",
    "resource": "/app/data/file.json",
    "permission": "read",
    "trust_level": "Trusted",
    "denial_reason": null
  }
}
```

### JSON Format (Denied Check)

```json
{
  "timestamp": "2025-12-19T10:30:46.789012Z",
  "event_type": "AccessDenied",
  "operation_id": "wasm-component-def456::/etc/passwd::read",
  "principal": "wasm-component-def456",
  "session_id": "550e8400-e29b-41d4-a716-446655440001",
  "decision": "Deny: Component declared /app/data/* but requested /etc/passwd",
  "policy_applied": "WasmCapabilityCheck",
  "metadata": {
    "wasm_component_id": "wasm-component-def456",
    "resource": "/etc/passwd",
    "permission": "read",
    "trust_level": "Unknown",
    "denial_reason": "Component declared /app/data/* but requested /etc/passwd"
  }
}
```

---

## Appendix B: Custom Logger Example

### Implementing a File-Based Audit Logger

```rust
use std::fs::OpenOptions;
use std::io::Write;
use std::sync::Mutex;
use airssys_osl::middleware::security::audit::{SecurityAuditLogger, SecurityAuditLog, AuditError};

#[derive(Debug)]
pub struct FileAuditLogger {
    file: Mutex<std::fs::File>,
}

impl FileAuditLogger {
    pub fn new(path: &str) -> Result<Self, std::io::Error> {
        let file = OpenOptions::new()
            .create(true)
            .append(true)
            .open(path)?;
        
        Ok(Self {
            file: Mutex::new(file),
        })
    }
}

#[async_trait::async_trait]
impl SecurityAuditLogger for FileAuditLogger {
    async fn log_security_event(&self, event: SecurityAuditLog) -> Result<(), AuditError> {
        let json = serde_json::to_string(&event)?;
        let mut file = self.file.lock().unwrap();
        writeln!(file, "{}", json)?;
        Ok(())
    }
    
    async fn flush(&self) -> Result<(), AuditError> {
        let mut file = self.file.lock().unwrap();
        file.flush()?;
        Ok(())
    }
}

// Usage
fn setup_file_audit_logging() -> Result<(), String> {
    let file_logger = Arc::new(FileAuditLogger::new("/var/log/wasm-audit.log")
        .map_err(|e| format!("Failed to create file logger: {}", e))?);
    
    let wasm_logger = WasmAuditLogger::new(file_logger);
    set_global_audit_logger(wasm_logger)?;
    
    Ok(())
}
```

---

**Document Status:** ✅ READY FOR IMPLEMENTATION  
**Last Updated:** 2025-12-19  
**Next Review:** After Task 3.3 completion
