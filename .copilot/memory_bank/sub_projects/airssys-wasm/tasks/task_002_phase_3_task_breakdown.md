# WASM-TASK-002 Phase 3: Task Breakdown
## CPU Limiting and Resource Control

**Status:** Ready for Implementation  
**Created:** 2025-10-23  
**Duration:** 4-7 days (22-27 hours)  
**Priority:** CRITICAL PATH - CPU Protection Layer  
**Reference:** `task_002_phase_3_implementation_plan.md` (2,787 lines, complete implementation guide)

---

## Executive Summary

This task breakdown provides actionable, day-by-day implementation guidance for WASM-TASK-002 Phase 3, implementing dual-layer CPU limiting using:

1. **Fuel Metering** (deterministic instruction counting via Wasmtime)
2. **Wall-Clock Timeout** (guaranteed termination via Tokio)

**Phase Context:**
- **Phase 2 Status**: ✅ COMPLETE (239 tests passing, zero warnings, 100% memory isolation)
- **Current Progress**: 30% overall (moving to 40% after Phase 3)
- **Dependencies**: All prerequisites met (Phases 1-2 complete)

**Success Metrics:**
- 64+ new tests (exceeds 30+ target)
- Zero compiler/clippy warnings
- 100% rustdoc coverage for new APIs
- Dual-layer CPU protection operational

---

## Architecture Overview

### Hybrid CPU Limiting (ADR-WASM-002 Decision 3b)

```
┌─────────────────────────────────────────────────────────┐
│                 Component Execution                      │
├─────────────────────────────────────────────────────────┤
│                                                          │
│  ┌────────────────────────────────────────────────┐    │
│  │ Layer 1: Fuel Metering (Deterministic)        │    │
│  │ - Wasmtime instruction counting                │    │
│  │ - store.add_fuel(1_000_000)                   │    │
│  │ - Prevents CPU-bound infinite loops            │    │
│  │ - Cannot be bypassed by slow I/O               │    │
│  └────────────────────────────────────────────────┘    │
│                        ▼                                 │
│  ┌────────────────────────────────────────────────┐    │
│  │ Layer 2: Wall-Clock Timeout (Guaranteed)       │    │
│  │ - Tokio timeout wrapper (100ms default)        │    │
│  │ - Real-time execution limit                    │    │
│  │ - Protects against slow operations             │    │
│  │ - Absolute guarantee of termination            │    │
│  └────────────────────────────────────────────────┘    │
│                        ▼                                 │
│           ┌─────────────────────────┐                   │
│           │ Execution Result        │                   │
│           │ - Success + fuel used   │                   │
│           │ - OutOfFuel trap        │                   │
│           │ - Timeout elapsed       │                   │
│           └─────────────────────────┘                   │
└─────────────────────────────────────────────────────────┘
```

**Key Design Decisions:**
- CPU limits have defaults (unlike memory): 1M fuel, 100ms timeout
- `[resources.cpu]` section is OPTIONAL in Component.toml
- Dual protection: fuel limits instructions, timeout limits real time
- Clear error differentiation: `OutOfFuel` vs `ExecutionTimeout`

---

## Task 3.1: Fuel Metering Implementation

**Duration:** Days 1-2 (8-10 hours)  
**Objective:** Implement deterministic CPU limiting via Wasmtime fuel metering

### Subtask 3.1.1: Extend ResourceLimits with CPU Fields

**Duration:** Day 1, 1-2 hours  
**File:** `airssys-wasm/src/runtime/limits.rs` (currently 1,435 lines from Phase 2)

**Changes Required:**

1. **Add CPU fields to ResourceLimits struct** (lines ~50-60):
```rust
pub struct ResourceLimits {
    // Memory limits (MANDATORY - no defaults)
    max_memory_bytes: u64,
    
    // CPU limits (OPTIONAL - have defaults)
    max_fuel_per_execution: u64,
    max_execution_timeout_ms: u64,
}
```

2. **Add default constants** (lines ~65-85):
```rust
impl ResourceLimits {
    /// Default maximum fuel per execution: 1,000,000 units
    pub const DEFAULT_MAX_FUEL: u64 = 1_000_000;
    
    /// Default execution timeout: 100ms wall-clock time
    pub const DEFAULT_TIMEOUT_MS: u64 = 100;
}
```

3. **Update ResourceLimitsBuilder** (lines ~90-150):
   - Add `max_fuel: Option<u64>` field
   - Add `timeout_ms: Option<u64>` field
   - Add `max_fuel(mut self, fuel: u64) -> Self` method
   - Add `execution_timeout_ms(mut self, timeout_ms: u64) -> Self` method
   - Update `build()` to apply defaults using `unwrap_or()`

4. **Add accessor methods** (lines ~160-180):
```rust
impl ResourceLimits {
    pub fn max_fuel_per_execution(&self) -> u64 { ... }
    pub fn execution_timeout_ms(&self) -> u64 { ... }
}
```

**Unit Tests Required** (add to existing test module, ~100 lines):
- `test_resource_limits_with_defaults()` - Memory only, CPU uses defaults
- `test_resource_limits_custom_cpu()` - Memory + custom CPU
- `test_default_fuel_constant()` - Verify DEFAULT_MAX_FUEL = 1M
- `test_default_timeout_constant()` - Verify DEFAULT_TIMEOUT_MS = 100

**Validation:**
```bash
cargo test --package airssys-wasm --lib limits::tests
cargo clippy --package airssys-wasm
```

**Implementation Reference:** Lines 247-496 in implementation plan

---

### Subtask 3.1.2: Add FuelConfig and FuelMetrics Structs

**Duration:** Day 1, 2-3 hours  
**File:** `airssys-wasm/src/runtime/limits.rs` (continue in same file)

**Changes Required:**

1. **Add FuelConfig struct** (lines ~500-590):
```rust
/// Fuel metering configuration for deterministic CPU limiting.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct FuelConfig {
    max_fuel_per_execution: u64,
}

impl FuelConfig {
    pub fn new(max_fuel: u64) -> Self { ... }
    pub fn max_fuel(&self) -> u64 { ... }
}

impl Default for FuelConfig {
    fn default() -> Self {
        Self::new(ResourceLimits::DEFAULT_MAX_FUEL)
    }
}
```

2. **Add FuelMetrics struct** (lines ~595-700):
```rust
/// Real-time fuel consumption metrics.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct FuelMetrics {
    max_fuel: u64,
    consumed: u64,
}

impl FuelMetrics {
    pub fn new(max_fuel: u64, consumed: u64) -> Self { ... }
    pub fn max_fuel(&self) -> u64 { ... }
    pub fn consumed(&self) -> u64 { ... }
    pub fn remaining(&self) -> u64 { ... }
    pub fn usage_percentage(&self) -> f64 { ... }
    pub fn is_exhausted(&self) -> bool { ... }
    pub fn exceeds_threshold(&self, percentage: f64) -> bool { ... }
}
```

**Unit Tests Required** (add new test module, ~70 lines):
- `test_fuel_config_new()` - Basic creation
- `test_fuel_config_default()` - Default value
- `test_fuel_metrics_new()` - Metrics creation
- `test_fuel_metrics_usage_percentage()` - Percentage calculation (0%, 75%, 100%)
- `test_fuel_metrics_zero_max()` - Edge case: zero max fuel
- `test_fuel_metrics_is_exhausted()` - Exhaustion detection
- `test_fuel_metrics_exceeds_threshold()` - Threshold detection
- `test_fuel_metrics_remaining_saturating()` - Saturating subtraction

**Validation:**
```bash
cargo test --package airssys-wasm --lib limits::fuel_tests
```

**Implementation Reference:** Lines 499-768 in implementation plan

---

### Subtask 3.1.3: Enable Fuel Metering in Wasmtime Engine

**Duration:** Day 1, 1-2 hours  
**File:** `airssys-wasm/src/runtime/engine.rs`

**Changes Required:**

1. **Add fuel metering to engine configuration** (line ~35):
```rust
pub fn create_engine() -> Result<Engine, WasmError> {
    let mut config = Config::new();
    
    // Enable Component Model support
    config.wasm_component_model(true);
    
    // Enable async support for non-blocking execution
    config.async_support(true);
    
    // Enable fuel metering for deterministic CPU limiting (NEW)
    config.consume_fuel(true);
    
    // Create engine with configuration
    Engine::new(&config)
        .map_err(|e| WasmError::engine_creation_failed(e))
}
```

2. **Update module documentation** (lines 1-30):
   - Add fuel metering to feature list
   - Add usage examples with `store.add_fuel()`
   - Reference ADR-WASM-002 decision

**Unit Tests Required** (add to existing test module, ~25 lines):
- `test_create_engine()` - Basic creation (already exists)
- `test_engine_fuel_enabled()` - Verify fuel can be added
- `test_engine_fuel_consumed_zero_initially()` - Initial fuel state

**Validation:**
```bash
cargo test --package airssys-wasm --lib engine::tests
```

**Implementation Reference:** Lines 770-924 in implementation plan

---

### Subtask 3.1.4: Update WasmError with Fuel-Related Variants

**Duration:** Day 2, 1 hour  
**File:** `airssys-wasm/src/core/error.rs`

**Changes Required:**

1. **Add new error variants** (lines ~40-100):
```rust
#[derive(Debug, thiserror::Error)]
pub enum WasmError {
    // ... existing variants ...
    
    /// Component ran out of fuel (CPU instruction limit exceeded).
    #[error("Component ran out of fuel: {consumed}/{max_fuel} fuel units consumed ({usage:.1}%)")]
    OutOfFuel {
        max_fuel: u64,
        consumed: u64,
        usage: f64,
    },
    
    /// Component execution exceeded wall-clock timeout.
    #[error("Component execution timeout: exceeded {timeout_ms}ms limit (fuel consumed: {fuel_consumed:?})")]
    ExecutionTimeout {
        timeout_ms: u64,
        fuel_consumed: Option<u64>,
    },
    
    // Keep existing variant for backward compatibility
    #[deprecated(since = "0.2.0", note = "Use OutOfFuel or ExecutionTimeout instead")]
    #[error("Resource limit exceeded: {0}")]
    ResourceLimitExceeded(String),
}
```

2. **Add helper constructors** (lines ~110-150):
```rust
impl WasmError {
    /// Create OutOfFuel error from fuel metrics.
    pub fn out_of_fuel(metrics: crate::runtime::limits::FuelMetrics) -> Self {
        Self::OutOfFuel {
            max_fuel: metrics.max_fuel(),
            consumed: metrics.consumed(),
            usage: metrics.usage_percentage(),
        }
    }
    
    /// Create ExecutionTimeout error with optional fuel info.
    pub fn execution_timeout(timeout_ms: u64, fuel_consumed: Option<u64>) -> Self {
        Self::ExecutionTimeout {
            timeout_ms,
            fuel_consumed,
        }
    }
}
```

**Unit Tests Required** (add new test module, ~40 lines):
- `test_out_of_fuel_error()` - Error creation and formatting
- `test_execution_timeout_with_fuel()` - Timeout with fuel info
- `test_execution_timeout_without_fuel()` - Timeout without fuel info

**Validation:**
```bash
cargo test --package airssys-wasm --lib error::error_tests
```

**Implementation Reference:** Lines 926-1095 in implementation plan

---

### Subtask 3.1.5: Integration Tests for Fuel Metering

**Duration:** Day 2, 2-3 hours  
**File:** `airssys-wasm/tests/fuel_metering_test.rs` (NEW FILE)

**Create Integration Test Suite:**

Structure (11+ tests, ~130 lines):

**Fuel Metering Tests:**
1. `test_fuel_metering_enabled()` - Engine has fuel enabled
2. `test_fuel_consumed_tracking()` - Fuel tracking works
3. `test_fuel_metrics_from_store()` - Create metrics from store

**Configuration Tests:**
4. `test_fuel_config_defaults()` - Default values
5. `test_resource_limits_with_custom_fuel()` - Custom fuel
6. `test_resource_limits_with_default_fuel()` - Default fuel

**Metrics Tests:**
7. `test_fuel_metrics_percentage()` - Percentage calculation
8. `test_fuel_metrics_exhaustion()` - Exhaustion detection
9. `test_fuel_metrics_threshold()` - Threshold detection

**Error Tests:**
10. `test_out_of_fuel_error_creation()` - Error creation

**Validation:**
```bash
cargo test --test fuel_metering_test
```

**Expected Output:**
```
running 11 tests
test test_fuel_metering_enabled ... ok
test test_fuel_consumed_tracking ... ok
...
test result: ok. 11 passed; 0 failed
```

**Implementation Reference:** Lines 1097-1252 in implementation plan

---

## Task 3.2: Wall-Clock Timeout Protection

**Duration:** Days 2-3 (8-10 hours)  
**Objective:** Implement guaranteed termination via Tokio timeout wrapper

### Subtask 3.2.1: Create runtime/executor.rs with Timeout Wrapper

**Duration:** Day 2, 3-4 hours  
**File:** `airssys-wasm/src/runtime/executor.rs` (NEW FILE, ~500 lines)

**Module Structure:**

1. **Module documentation** (lines 1-70):
   - Architecture diagram (timeout wrapper + fuel execution)
   - Usage examples
   - Reference to ADR-WASM-002

2. **ComponentExecutor struct** (lines 80-400):
```rust
pub struct ComponentExecutor {
    engine: Engine,
}

impl ComponentExecutor {
    pub fn new(engine: Engine) -> WasmResult<Self> { ... }
    
    pub async fn execute_with_limits<T, R>(
        &self,
        component: Component,
        limits: ResourceLimits,
        args: T,
    ) -> WasmResult<ExecutionResult<R>> {
        // 1. Create Store with resource limiter
        // 2. Add fuel for this execution
        // 3. Wrap execution in tokio::time::timeout
        // 4. Handle result: success, fuel exhaustion, timeout
    }
    
    async fn execute_component<T, R>(...) -> Result<R, anyhow::Error> {
        // TODO: Stub for Phase 3
        unimplemented!("Component execution in future phases")
    }
}
```

3. **ExecutionResult enum** (lines 420-480):
```rust
#[derive(Debug)]
pub enum ExecutionResult<T> {
    Success {
        output: T,
        fuel_metrics: FuelMetrics,
    },
}

impl<T> ExecutionResult<T> {
    pub fn fuel_metrics(&self) -> &FuelMetrics { ... }
}
```

4. **Update runtime/mod.rs** (add line):
```rust
pub mod executor;  // NEW
```

**Unit Tests Required** (add to file, ~30 lines):
- `test_executor_creation()` - Basic executor creation
- `test_execution_result_fuel_metrics()` - Metrics from result

**Validation:**
```bash
cargo test --package airssys-wasm --lib executor::tests
```

**Implementation Reference:** Lines 1254-1605 in implementation plan

---

### Subtask 3.2.2: Parse [resources.cpu] in Component.toml

**Duration:** Day 3, 2-3 hours  
**File:** `airssys-wasm/src/core/config.rs`

**Changes Required:**

1. **Add CpuConfig struct** (lines ~100-160):
```rust
/// CPU configuration from [resources.cpu] (OPTIONAL).
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct CpuConfig {
    /// Maximum fuel per execution (defaults to 1M).
    #[serde(default = "default_max_fuel")]
    pub max_fuel: u64,
    
    /// Execution timeout in milliseconds (defaults to 100ms).
    #[serde(default = "default_timeout_ms")]
    pub timeout_ms: u64,
}

impl Default for CpuConfig {
    fn default() -> Self {
        Self {
            max_fuel: default_max_fuel(),
            timeout_ms: default_timeout_ms(),
        }
    }
}

fn default_max_fuel() -> u64 {
    ResourceLimits::DEFAULT_MAX_FUEL
}

fn default_timeout_ms() -> u64 {
    ResourceLimits::DEFAULT_TIMEOUT_MS
}
```

2. **Update ResourcesConfig** (line ~50):
```rust
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct ResourcesConfig {
    pub memory: MemoryConfig,
    
    /// CPU configuration (OPTIONAL - has defaults)
    #[serde(default)]
    pub cpu: CpuConfig,  // NEW
}
```

3. **Add TryFrom conversion** (lines ~170-190):
```rust
impl TryFrom<ComponentConfig> for ResourceLimits {
    type Error = crate::core::error::WasmError;
    
    fn try_from(config: ComponentConfig) -> Result<Self, Self::Error> {
        ResourceLimits::builder()
            .memory_bytes(config.resources.memory.max_bytes)
            .max_fuel(config.resources.cpu.max_fuel)
            .execution_timeout_ms(config.resources.cpu.timeout_ms)
            .build()
            .map_err(|e| WasmError::InvalidConfiguration(e.to_string()))
    }
}
```

**Unit Tests Required** (add to existing test module, ~100 lines):
- `test_cpu_config_defaults()` - Default values
- `test_parse_toml_with_cpu_defaults()` - Omitted [resources.cpu]
- `test_parse_toml_with_custom_cpu()` - Full [resources.cpu]
- `test_config_to_resource_limits()` - Conversion works

**Validation:**
```bash
cargo test --package airssys-wasm --lib config::tests
```

**Component.toml Example:**
```toml
[component]
name = "my-component"
version = "0.1.0"

[resources.memory]
max_bytes = 1048576

[resources.cpu]  # OPTIONAL
max_fuel = 5000000
timeout_ms = 500
```

**Implementation Reference:** Lines 1607-1809 in implementation plan

---

### Subtask 3.2.3: Integration Tests for Timeout Protection

**Duration:** Day 3, 2 hours  
**File:** `airssys-wasm/tests/timeout_protection_test.rs` (NEW FILE)

**Create Timeout Test Suite:**

Structure (5+ tests, ~80 lines):

**Executor Tests:**
1. `test_executor_creation()` - Executor creation

**Configuration Tests:**
2. `test_resource_limits_with_timeout()` - Custom timeout
3. `test_default_timeout_value()` - Default timeout

**Parsing Tests:**
4. `test_component_config_cpu_parsing()` - TOML parsing
5. `test_config_to_limits_conversion()` - Config conversion

**Validation:**
```bash
cargo test --test timeout_protection_test
```

**Expected Output:**
```
running 5 tests
test test_executor_creation ... ok
test test_resource_limits_with_timeout ... ok
...
test result: ok. 5 passed; 0 failed
```

**Implementation Reference:** Lines 1811-1905 in implementation plan

---

## Task 3.3: CPU Limit Testing and Tuning

**Duration:** Days 3-4 (6-8 hours)  
**Objective:** Comprehensive validation of dual-layer CPU limiting

### Subtask 3.3.1: Create Comprehensive CPU Limit Test Suite

**Duration:** Days 3-4, 4-6 hours  
**File:** `airssys-wasm/tests/cpu_limits_test.rs` (NEW FILE, ~380 lines)

**Test Categories (31+ tests):**

**1. Fuel Metering Tests (5 tests):**
- `test_fuel_config_various_limits()` - 100K to 100M fuel
- `test_fuel_metrics_edge_cases()` - Zero, overconsumption, exact exhaustion
- `test_fuel_warning_thresholds()` - 50%, 75%, 90%, 95%
- `test_fuel_percentage_accuracy()` - 0%, 25%, 50%, 75%, 100%
- `test_resource_limits_fuel_variations()` - Default, low, high

**2. Timeout Configuration Tests (3 tests):**
- `test_timeout_various_durations()` - 50ms, 100ms, 500ms, 5000ms
- `test_default_timeout()` - Verify 100ms default
- `test_timeout_and_fuel_combination()` - Both configured

**3. Component.toml CPU Parsing Tests (4 tests):**
- `test_component_toml_default_cpu()` - Omitted [resources.cpu]
- `test_component_toml_custom_cpu()` - Full configuration
- `test_component_toml_partial_cpu()` - Only timeout_ms set
- `test_component_config_to_limits()` - TryFrom conversion

**4. Error Handling Tests (4 tests):**
- `test_out_of_fuel_error()` - Full exhaustion (100%)
- `test_out_of_fuel_error_partial()` - Partial exhaustion (95%)
- `test_execution_timeout_error_with_fuel()` - Timeout with metrics
- `test_execution_timeout_error_no_fuel()` - Timeout without metrics

**5. Wasmtime Integration Tests (3 tests):**
- `test_engine_fuel_metering_enabled()` - Fuel addition works
- `test_fuel_addition_and_tracking()` - Consumption tracking
- `test_multiple_fuel_additions()` - Multiple add_fuel() calls

**6. Executor Tests (1 test):**
- `test_executor_creation()` - Basic executor creation

**7. Security Tests (4 tests):**
- `test_zero_fuel_limit_protection()` - Zero fuel behavior
- `test_zero_timeout_protection()` - Zero timeout behavior
- `test_extremely_large_fuel_limits()` - u64::MAX fuel
- `test_extremely_large_timeout()` - u64::MAX timeout

**Validation:**
```bash
cargo test --test cpu_limits_test
```

**Expected Output:**
```
running 31 tests
test test_fuel_config_various_limits ... ok
test test_fuel_metrics_edge_cases ... ok
...
test result: ok. 31 passed; 0 failed
```

**Implementation Reference:** Lines 1907-2355 in implementation plan

---

## File-by-File Change Summary

### Files to Modify (7 files)

1. **`airssys-wasm/src/runtime/limits.rs`** (1,435 lines → ~1,700 lines)
   - Add CPU fields to ResourceLimits
   - Add FuelConfig and FuelMetrics structs
   - Update builder with fuel and timeout methods
   - Add ~200 lines of new code + 100 lines tests

2. **`airssys-wasm/src/runtime/engine.rs`** (~80 lines)
   - Add `config.consume_fuel(true)` line
   - Update module documentation
   - Add 3 unit tests (~25 lines)

3. **`airssys-wasm/src/runtime/executor.rs`** (NEW FILE, ~500 lines)
   - ComponentExecutor struct with timeout wrapper
   - ExecutionResult enum
   - Module documentation
   - 2 unit tests

4. **`airssys-wasm/src/runtime/mod.rs`** (~20 lines)
   - Add `pub mod executor;` line

5. **`airssys-wasm/src/core/error.rs`** (~200 lines → ~280 lines)
   - Add OutOfFuel variant
   - Add ExecutionTimeout variant
   - Add helper constructors
   - Add 3 unit tests (~40 lines)

6. **`airssys-wasm/src/core/config.rs`** (~150 lines → ~280 lines)
   - Add CpuConfig struct
   - Update ResourcesConfig with cpu field
   - Add TryFrom<ComponentConfig> for ResourceLimits
   - Add 4 unit tests (~100 lines)

7. **`airssys-wasm/src/runtime/mod.rs`** (~15 lines)
   - Export executor module

### Files to Create (3 new test files)

8. **`airssys-wasm/tests/fuel_metering_test.rs`** (NEW FILE, ~130 lines)
   - 11+ integration tests for fuel metering

9. **`airssys-wasm/tests/timeout_protection_test.rs`** (NEW FILE, ~80 lines)
   - 5+ integration tests for timeout protection

10. **`airssys-wasm/tests/cpu_limits_test.rs`** (NEW FILE, ~380 lines)
    - 31+ comprehensive CPU limit tests

---

## Validation Steps

### Step 1: Code Quality Checks (MANDATORY)

```bash
# Zero warnings required
cargo check --workspace

# Zero clippy warnings
cargo clippy --workspace --all-targets --all-features

# Format check
cargo fmt --check
```

**Expected:** No warnings, no errors

---

### Step 2: Unit Test Validation

```bash
# Test modified modules
cargo test --package airssys-wasm --lib limits
cargo test --package airssys-wasm --lib engine
cargo test --package airssys-wasm --lib executor
cargo test --package airssys-wasm --lib error
cargo test --package airssys-wasm --lib config
```

**Expected:** All unit tests passing

---

### Step 3: Integration Test Validation

```bash
# Test new integration test files
cargo test --test fuel_metering_test
cargo test --test timeout_protection_test
cargo test --test cpu_limits_test

# Run all tests
cargo test --package airssys-wasm
```

**Expected Test Count:**
- Existing tests: 239 (203 unit + 36 integration) from Phase 2
- New tests: 64+ (20+ unit + 47+ integration) from Phase 3
- **Total: 286+ tests passing**

---

### Step 4: Documentation Validation

```bash
# Zero doc warnings
cargo doc --no-deps --package airssys-wasm

# Open and verify
cargo doc --open --package airssys-wasm
```

**Checklist:**
- [ ] FuelConfig fully documented
- [ ] FuelMetrics fully documented
- [ ] ComponentExecutor fully documented
- [ ] ExecutionResult fully documented
- [ ] CpuConfig fully documented
- [ ] Error variants documented
- [ ] Usage examples included

---

### Step 5: Workspace Standards Compliance

**Verify compliance with:**
- ✅ §2.1: 3-Layer Import Organization
- ✅ §3.2: chrono DateTime<Utc> (if applicable)
- ✅ §4.3: mod.rs Declaration-Only Pattern
- ✅ §5.1: Workspace Dependencies
- ✅ §6.3: Microsoft Rust Guidelines

---

### Step 6: Final Validation

```bash
# Complete workspace validation
cargo test --workspace
cargo clippy --workspace --all-targets --all-features
cargo doc --no-deps --workspace

# Count tests
cargo test --package airssys-wasm -- --list | wc -l
```

**Expected:**
- ✅ 286+ tests passing
- ✅ Zero warnings
- ✅ 100% rustdoc coverage
- ✅ All validation steps green

---

## Timeline and Dependencies

### Day-by-Day Schedule

**Day 1: Fuel Metering Foundation (6-7 hours)**
- 🕐 Hours 1-2: Subtask 3.1.1 - Extend ResourceLimits
- 🕐 Hours 3-5: Subtask 3.1.2 - Add FuelConfig/FuelMetrics
- 🕐 Hours 6-7: Subtask 3.1.3 - Enable fuel in engine
- 🕐 Hour 8: Subtask 3.1.4 - Update WasmError

**Day 2: Fuel Integration + Timeout Start (6-7 hours)**
- 🕐 Hours 1-3: Subtask 3.1.5 - Fuel integration tests
- 🕐 Hours 4-7: Subtask 3.2.1 - Create executor.rs

**Day 3: Timeout Completion (6-7 hours)**
- 🕐 Hours 1-3: Subtask 3.2.2 - Parse [resources.cpu]
- 🕐 Hours 4-5: Subtask 3.2.3 - Timeout integration tests
- 🕐 Hours 6-7: Subtask 3.3.1 - Start CPU limit tests

**Day 4: Testing and Validation (4-6 hours)**
- 🕐 Hours 1-3: Subtask 3.3.1 - Complete CPU limit tests
- 🕐 Hours 4-6: Final validation and cleanup

**Total: 22-27 hours (4-7 days depending on pace)**

---

### Dependency Graph

```
Task 3.1.1 (Extend ResourceLimits)
    ↓
Task 3.1.2 (FuelConfig/FuelMetrics) ─→ Task 3.1.4 (Update WasmError)
    ↓                                       ↓
Task 3.1.3 (Enable fuel) ────────────→ Task 3.1.5 (Fuel tests)
    ↓
Task 3.2.1 (Create executor.rs) ←─────── Task 3.1.2 (needs FuelMetrics)
    ↓
Task 3.2.2 (Parse [resources.cpu]) ←─── Task 3.1.1 (needs ResourceLimits)
    ↓
Task 3.2.3 (Timeout tests)
    ↓
Task 3.3.1 (CPU limit tests) ←───────── ALL previous tasks
```

**Critical Path:**
- 3.1.1 → 3.1.2 → 3.2.1 → 3.2.2 → 3.3.1

**Parallel Opportunities:**
- 3.1.3 and 3.1.4 can be done in parallel with 3.1.2
- 3.1.5 can start while 3.2.1 is in progress (different files)

---

## Success Criteria

### Phase 3 Complete When All Checkboxes Pass:

**Fuel Metering (Task 3.1):**
- [ ] ResourceLimits extended with `max_fuel_per_execution` and `max_execution_timeout_ms`
- [ ] FuelConfig struct implemented with defaults
- [ ] FuelMetrics struct with usage tracking (consumed, remaining, percentage, exhausted)
- [ ] Wasmtime engine fuel metering enabled (`consume_fuel(true)`)
- [ ] WasmError::OutOfFuel variant with formatted metrics
- [ ] 10+ unit tests passing in limits.rs
- [ ] 11+ integration tests passing in fuel_metering_test.rs

**Timeout Protection (Task 3.2):**
- [ ] runtime/executor.rs created with ComponentExecutor
- [ ] Tokio timeout wrapper implemented
- [ ] ExecutionResult enum with fuel_metrics
- [ ] CpuConfig struct with serde defaults
- [ ] [resources.cpu] parsing in Component.toml
- [ ] WasmError::ExecutionTimeout variant with optional fuel info
- [ ] 10+ unit tests passing in executor.rs and config.rs
- [ ] 5+ integration tests passing in timeout_protection_test.rs

**Testing and Validation (Task 3.3):**
- [ ] cpu_limits_test.rs created with 31+ comprehensive tests
- [ ] Fuel configuration tests (various limits, edge cases, thresholds)
- [ ] Timeout configuration tests (various durations, defaults)
- [ ] Component.toml CPU parsing tests (default, custom, partial)
- [ ] Error handling tests (OutOfFuel, ExecutionTimeout formatting)
- [ ] Wasmtime integration tests (fuel enabled, tracking)
- [ ] Security tests (zero limits, extremely large limits)

**Quality Standards:**
- [ ] Zero compiler warnings (`cargo check --workspace`)
- [ ] Zero clippy warnings (`cargo clippy --workspace --all-targets --all-features`)
- [ ] 286+ total tests passing (239 existing + 64+ new)
- [ ] 100% rustdoc coverage for new APIs (`cargo doc`)
- [ ] Workspace standards compliance (§2.1-§6.3)
- [ ] Import organization (3-layer pattern)
- [ ] mod.rs declaration-only pattern

**Documentation:**
- [ ] Comprehensive rustdoc for FuelConfig, FuelMetrics
- [ ] Comprehensive rustdoc for ComponentExecutor, ExecutionResult
- [ ] Comprehensive rustdoc for CpuConfig
- [ ] Usage examples in module documentation
- [ ] Component.toml configuration examples
- [ ] Error handling patterns documented
- [ ] ADR-WASM-002 references included

---

## Key Implementation Notes

### Design Philosophy (CRITICAL)

**CPU Limits vs Memory Limits:**
- **Memory**: MANDATORY, no defaults (varies drastically between components)
- **CPU**: OPTIONAL, has defaults (similar patterns across components)
  - Default fuel: 1,000,000 units
  - Default timeout: 100ms

**Error Handling Priority:**
1. Check fuel exhaustion first (more specific)
2. Then check timeout
3. Then other trap reasons

**Fuel Calibration (Platform-Dependent):**
```
Simple arithmetic:   1-5 fuel per operation
Function call:      50-100 fuel
Memory access:       1-2 fuel per access
Branch:              1-3 fuel

Rough Estimates:
  100K fuel ≈   1-5ms CPU time
    1M fuel ≈  10-50ms CPU time
   10M fuel ≈ 100-500ms CPU time
```

**Component.toml Examples:**

```toml
# Minimal (uses CPU defaults)
[component]
name = "simple-component"
version = "0.1.0"

[resources.memory]
max_bytes = 1048576

# No [resources.cpu] needed - uses 1M fuel, 100ms timeout


# Custom CPU configuration
[component]
name = "heavy-component"
version = "0.1.0"

[resources.memory]
max_bytes = 4194304

[resources.cpu]
max_fuel = 50000000     # 50M fuel
timeout_ms = 1000       # 1 second
```

---

### Common Patterns from Implementation Plan

**Pattern 1: Builder with Defaults**
```rust
let limits = ResourceLimits::builder()
    .memory_bytes(1024 * 1024)  // MANDATORY
    // CPU fields optional - use defaults
    .build()?;

assert_eq!(limits.max_fuel_per_execution(), 1_000_000);
assert_eq!(limits.execution_timeout_ms(), 100);
```

**Pattern 2: FuelMetrics Usage**
```rust
let consumed = store.fuel_consumed()?;
let metrics = FuelMetrics::new(max_fuel, consumed);

println!("Fuel: {}/{} ({:.1}%)",
    metrics.consumed(),
    metrics.max_fuel(),
    metrics.usage_percentage()
);

if metrics.exceeds_threshold(90.0) {
    warn!("Component approaching fuel limit");
}
```

**Pattern 3: Error Handling**
```rust
match executor.execute_with_limits(component, limits, args).await {
    Ok(ExecutionResult::Success { fuel_metrics, .. }) => {
        info!("Execution succeeded: {} fuel used", fuel_metrics.consumed());
    }
    Err(WasmError::OutOfFuel { consumed, max_fuel, usage }) => {
        error!("Component exhausted {max_fuel} fuel ({usage:.1}%)");
    }
    Err(WasmError::ExecutionTimeout { timeout_ms, fuel_consumed }) => {
        error!("Component timeout: {timeout_ms}ms (fuel: {fuel_consumed:?})");
    }
    Err(e) => {
        error!("Execution failed: {e}");
    }
}
```

---

## Risk Management

### Known Challenges

**Challenge 1: Component Execution Stub**
- **Issue:** Phase 3 doesn't implement actual component execution
- **Impact:** Cannot test fuel consumption with real WASM code
- **Mitigation:** 
  - Focus tests on configuration and infrastructure
  - Test error types and handling paths
  - Document stub implementation (`unimplemented!()`)
  - Defer real execution tests to future phases

**Challenge 2: Fuel Calibration Variability**
- **Issue:** Fuel consumption varies by platform
- **Impact:** Difficult to provide precise fuel-to-time estimates
- **Mitigation:**
  - Document fuel as approximate
  - Provide ranges instead of exact values
  - Recommend timeout as primary time guarantee

**Challenge 3: Dual-Layer Coordination**
- **Issue:** Race condition between fuel exhaustion and timeout
- **Impact:** Unclear which error to return
- **Mitigation:**
  - Check fuel exhaustion first
  - Include fuel info in timeout errors
  - Document error priority

### Rollback Procedure

If Phase 3 fails validation:

```bash
# Option 1: Git revert
git revert <phase-3-commits>

# Option 2: Manual rollback
# Remove new files:
rm src/runtime/executor.rs
rm tests/fuel_metering_test.rs
rm tests/timeout_protection_test.rs
rm tests/cpu_limits_test.rs

# Revert modified files:
git checkout HEAD -- src/runtime/limits.rs
git checkout HEAD -- src/runtime/engine.rs
git checkout HEAD -- src/core/error.rs
git checkout HEAD -- src/core/config.rs
git checkout HEAD -- src/runtime/mod.rs

# Validate Phase 2 still works
cargo test --workspace
```

---

## References

### Primary Documentation
- **Implementation Plan**: `task_002_phase_3_implementation_plan.md` (2,787 lines, complete guide)
- **ADR-WASM-002**: Hybrid CPU limiting architecture (fuel + timeout)
- **ADR-WASM-006**: Component isolation and sandboxing (4-layer defense)
- **Phase 2 Completion**: 239 tests, zero warnings, 100% memory isolation

### External References
- Wasmtime fuel metering: https://docs.wasmtime.dev/api/wasmtime/struct.Config.html#method.consume_fuel
- Tokio timeout: https://docs.rs/tokio/latest/tokio/time/fn.timeout.html
- Wasmtime Store fuel API: https://docs.wasmtime.dev/api/wasmtime/struct.Store.html#method.add_fuel

### Workspace Standards
- §2.1: 3-Layer Import Organization (MANDATORY)
- §3.2: chrono DateTime<Utc> Standard (MANDATORY)
- §4.3: Module Architecture (mod.rs declaration-only)
- §5.1: Dependency Management (workspace dependencies)
- §6.3: Microsoft Rust Guidelines (MANDATORY)

---

## Next Steps After Phase 3

**Immediate:**
1. Review this task breakdown for completeness
2. Clarify any unclear implementation details
3. Begin Day 1 implementation (Subtask 3.1.1)

**After Phase 3 Complete:**
- **Phase 4**: Component instantiation and actual execution (integrate CPU limiting with real components)
- **Phase 5**: Host function bridge (integrate with airssys-osl)
- **Phase 6**: Security policy enforcement (integrate with capability system)

**Progress Tracking:**
- Current: 30% (Phases 1-2 complete)
- After Phase 3: 40% (CPU protection layer complete)
- Target: 100% (complete WASM component framework)

---

**END OF TASK BREAKDOWN**

This structured breakdown provides day-by-day, file-by-file guidance for autonomous implementation of WASM-TASK-002 Phase 3, following the comprehensive 2,787-line implementation plan as the authoritative reference.
