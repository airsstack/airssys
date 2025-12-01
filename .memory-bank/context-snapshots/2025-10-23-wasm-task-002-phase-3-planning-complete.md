# Context Snapshot: WASM-TASK-002 Phase 3 Planning Complete - CPU Limiting Design

**Timestamp:** 2025-10-23  
**Active Sub-Project:** airssys-wasm  
**Session Type:** Planning Session Completion  
**Task Context:** WASM-TASK-002 Phase 3 - CPU Limiting and Resource Control

---

## Session Overview

### What Was Accomplished

This planning session created comprehensive implementation documentation for WASM-TASK-002 Phase 3 (CPU Limiting and Resource Control). Two major planning documents were produced:

1. **Comprehensive Implementation Plan** (`task_002_phase_3_implementation_plan.md`)
   - **Size**: 2,787 lines (85KB)
   - **Content**: Complete day-by-day implementation guide with:
     - Detailed code examples for every implementation step
     - Complete test specifications with 64+ new tests
     - Validation procedures and success criteria
     - Integration patterns and error handling strategies
     - Day-by-day timeline with hour estimates

2. **Structured Task Breakdown** (`task_002_phase_3_task_breakdown.md`)
   - **Size**: 1,118 lines
   - **Content**: Actionable subtasks with:
     - Granular file-by-file change specifications
     - Time estimates for each subtask
     - Clear dependencies and sequencing
     - Validation checkpoints
     - Confirmed by task-plans subagent

### Session Purpose

Save the complete planning state so implementation can begin with full context about:
- Phase 3 architectural objectives and design decisions
- File-level changes required (modify 6, create 4 new files)
- Test coverage requirements (64+ new tests)
- Validation procedures and success criteria
- Timeline and effort estimates (4-7 days, 22-27 hours)

---

## Workspace Context

### Active Sub-Project Status

**Project:** airssys-wasm (WASM Component Framework for Pluggable Systems)  
**Status:** Foundation Complete - Block 1 Implementation In Progress  
**Overall Progress:** 30% complete (Phases 1-2 done, Phase 3 planned)  
**Current Phase:** WASM-TASK-002 Phase 3 Planning COMPLETE - Ready for Implementation

### Sub-Project Hierarchy

```
AirsSys Workspace
├── airssys-osl (100% COMPLETE ✅) - OS Layer Framework
├── airssys-rt (100% COMPLETE ✅) - Actor Runtime System
├── airssys-wasm (30% COMPLETE 🔄) - WASM Component Framework
│   ├── WASM-TASK-000: Core Abstractions (100% COMPLETE ✅)
│   ├── WASM-TASK-001: Planning (SKIPPED/NOT_NEEDED ⏭️)
│   └── WASM-TASK-002: Block 1 Implementation (IN PROGRESS 🔄)
│       ├── Phase 1: Memory Limits Foundation (100% COMPLETE ✅)
│       ├── Phase 2: Memory Management & Sandboxing (100% COMPLETE ✅)
│       └── Phase 3: CPU Limiting & Resource Control (0% - PLANNING COMPLETE 📋)
├── airssys-wasm-component (25% COMPLETE 🔄) - WASM Component Macros
└── airssys-wasm-cli (10% COMPLETE 🔄) - CLI Tool for WASM Components
```

### Workspace Standards Compliance

**Mandatory Standards (ALL ENFORCED):**
- ✅ **§2.1**: 3-Layer Import Organization (std → third-party → internal)
- ✅ **§3.2**: `chrono::DateTime<Utc>` for timestamps (not `std::time`)
- ✅ **§4.3**: Module Architecture (mod.rs exports only, no implementation)
- ✅ **§5.1**: Workspace dependency management with layer-based organization
- ✅ **§6.1**: YAGNI Principles (build only what's needed)
- ✅ **§6.2**: Avoid `dyn` patterns (prefer generic constraints)
- ✅ **§6.3**: Microsoft Rust Guidelines Integration (complete compliance)
- ✅ **§7.1**: mdBook Documentation Standards
- ✅ **§7.2**: Documentation Quality Standards (professional, objective, sourced)

**Reference Documents:**
- `.memory-bank/workspace/shared_patterns.md` (Code standards)
- `.memory-bank/workspace/microsoft_rust_guidelines.md` (Rust guidelines)
- `.memory-bank/workspace/documentation_terminology_standards.md` (Doc standards)

---

## Sub-Project Context: airssys-wasm

### Project Vision

**Framework**: WASM Component Framework for Pluggable Systems  
**Purpose**: General-purpose infrastructure platform for component-based software architectures  
**Novel Approach**: Combines WebAssembly + runtime deployment + composition patterns  
**Deployment Model**: Runtime component management inspired by smart contract systems

### Current Implementation State

**Phase 2 Status (Baseline for Phase 3):**
- ✅ **Memory Management**: Complete isolation and sandboxing
- ✅ **Test Coverage**: 239 tests passing (203 unit + 36 integration)
- ✅ **Code Quality**: Zero compiler/clippy warnings
- ✅ **Memory Isolation**: 100% verified with comprehensive test suite
- ✅ **Base Implementation**: 1,435 lines in `runtime/limits.rs`

**Architecture Foundation:**
- ✅ Core abstractions (WASM-TASK-000): 9,283 lines, 363 tests, 15 core modules
- ✅ Universal abstractions: Component, Capability, Error, Config
- ✅ Domain abstractions: Runtime, Interface, Actor, Security, Messaging, Storage, etc.
- ✅ 100% rustdoc coverage, full workspace standards compliance

### Technology Stack

**Core Technologies:**
- **Wasmtime**: WebAssembly runtime engine (fuel metering, memory management)
- **Component Model**: WASM Component Model for modular component development
- **WIT**: WebAssembly Interface Types for language-agnostic interfaces
- **WASI Preview 2**: WebAssembly System Interface for system access
- **Tokio**: Async runtime for timeout protection and concurrency

**Integration:**
- **airssys-osl**: Provides secure system access primitives
- **airssys-rt**: Provides actor-based component hosting capabilities

---

## Task Context: WASM-TASK-002 Phase 3

### Phase 3 Overview

**Objective:** Implement dual-layer CPU limiting (fuel metering + wall-clock timeout)

**Architecture Decision:** ADR-WASM-002 Decision 3b (Hybrid CPU Limiting Approach)

**Dual-Layer Design:**
1. **Layer 1: Fuel Metering** (Deterministic)
   - Instruction-level counting via Wasmtime's fuel system
   - Deterministic resource accounting
   - Predictable behavior across platforms
   - Configurable via `max_fuel` parameter

2. **Layer 2: Wall-Clock Timeout** (Guaranteed Termination)
   - Tokio-based timeout wrapper
   - Guarantees termination regardless of fuel behavior
   - Protects against infinite loops and non-terminating execution
   - Configurable via `timeout_ms` parameter

**Rationale for Dual-Layer Approach:**
- Fuel metering provides fine-grained, deterministic control
- Wall-clock timeout provides guaranteed termination safety net
- Combining both layers ensures robust CPU limiting with no gaps
- Follows proven patterns from production WASM runtimes

### Three Main Implementation Tasks

#### **Task 3.1: Fuel Metering Implementation**
**Timeline:** Days 1-2 (8-10 hours)  
**Objective:** Extend ResourceLimits with CPU fields and enable Wasmtime fuel metering

**Key Deliverables:**
- Extend `ResourceLimits` struct with CPU-related fields
- Add `FuelConfig` and `FuelMetrics` structs
- Enable Wasmtime fuel metering in engine configuration
- Update `WasmError` with `OutOfFuel` variant
- Create `tests/fuel_metering_test.rs` with 11+ integration tests

**Files to Modify:**
- `airssys-wasm/src/runtime/limits.rs` (+265 lines)
- `airssys-wasm/src/runtime/engine.rs` (+25 lines)
- `airssys-wasm/src/core/error.rs` (+40 lines for OutOfFuel)

**Files to Create:**
- `airssys-wasm/tests/fuel_metering_test.rs` (NEW - ~130 lines, 11+ tests)

**Test Coverage:**
- 11+ new tests: default config, exhaustion detection, trap handling, config variations
- Focus: Fuel metering accuracy, error handling, edge cases

#### **Task 3.2: Wall-Clock Timeout Protection**
**Timeline:** Days 2-3 (8-10 hours)  
**Objective:** Create timeout wrapper with Tokio and parse [resources.cpu] in Component.toml

**Key Deliverables:**
- Create `runtime/executor.rs` with `ComponentExecutor`
- Implement Tokio timeout wrapper for component execution
- Parse `[resources.cpu]` section in Component.toml
- Update `WasmError` with `ExecutionTimeout` variant
- Create `tests/timeout_protection_test.rs` with 5+ integration tests

**Files to Modify:**
- `airssys-wasm/src/runtime/mod.rs` (+1 line - export executor)
- `airssys-wasm/src/core/config.rs` (+130 lines - CpuConfig, parse [resources.cpu])
- `airssys-wasm/src/core/error.rs` (+40 lines for ExecutionTimeout)

**Files to Create:**
- `airssys-wasm/src/runtime/executor.rs` (NEW - ~500 lines - timeout wrapper)
- `airssys-wasm/tests/timeout_protection_test.rs` (NEW - ~80 lines, 5+ tests)

**Test Coverage:**
- 5+ new tests: timeout enforcement, long-running execution, timeout configuration, error context
- Focus: Guaranteed termination, timeout accuracy, error differentiation

#### **Task 3.3: CPU Limit Testing and Tuning**
**Timeline:** Days 3-4 (6-8 hours)  
**Objective:** Comprehensive testing and validation of dual-layer CPU limiting

**Key Deliverables:**
- Create `tests/cpu_limits_test.rs` with comprehensive test suite
- 31+ integration tests covering all CPU limiting scenarios
- Security and bypass attempt tests
- Final validation and performance tuning

**Files to Create:**
- `airssys-wasm/tests/cpu_limits_test.rs` (NEW - ~380 lines, 31+ tests)

**Test Categories:**
1. **Basic Functionality** (6 tests): Defaults, custom limits, within-limits execution
2. **Fuel Exhaustion** (5 tests): Exact exhaustion, mid-execution, loop exhaustion
3. **Timeout Protection** (6 tests): Timeout enforcement, long-running loops, timeout context
4. **Dual-Layer Coordination** (5 tests): Fuel-first priority, timeout fallback, race conditions
5. **Configuration** (4 tests): Component.toml parsing, defaults, overrides
6. **Error Handling** (3 tests): Clear error messages, error differentiation, context info
7. **Security** (2 tests): Bypass attempts, deterministic behavior

**Test Coverage:**
- 31+ new tests ensuring robust CPU limiting with no security gaps
- Focus: Dual-layer coordination, error handling, security validation

### Files Summary

**Files to Modify (6 files):**
1. `airssys-wasm/src/runtime/limits.rs` (+265 lines)
   - Extend with CPU config fields (`max_fuel`, `timeout_ms`)
   - Add `FuelConfig` and `FuelMetrics` structs
   - Implement fuel configuration helpers

2. `airssys-wasm/src/runtime/engine.rs` (+25 lines)
   - Enable Wasmtime fuel metering in engine config
   - Add fuel configuration to `WasmEngineBuilder`

3. `airssys-wasm/src/runtime/mod.rs` (+1 line)
   - Export new `executor` module

4. `airssys-wasm/src/core/error.rs` (+80 lines)
   - Add `OutOfFuel` variant with fuel consumption details
   - Add `ExecutionTimeout` variant with timeout context
   - Implement `Display` and helper methods

5. `airssys-wasm/src/core/config.rs` (+130 lines)
   - Add `CpuConfig` struct (max_fuel, timeout_ms)
   - Parse `[resources.cpu]` section from Component.toml
   - Implement default CPU configuration (1M fuel, 100ms timeout)

6. `airssys-wasm/Cargo.toml` (minor update)
   - Verify `tokio` dependency with `time` feature

**Files to Create (4 new files):**
1. `airssys-wasm/src/runtime/executor.rs` (NEW - ~500 lines)
   - `ComponentExecutor` struct with timeout wrapper
   - Tokio-based async execution with timeout
   - Integration with fuel metering
   - Error handling and context enrichment

2. `airssys-wasm/tests/fuel_metering_test.rs` (NEW - ~130 lines)
   - 11+ integration tests for fuel metering
   - Test default config, exhaustion, traps, config variations

3. `airssys-wasm/tests/timeout_protection_test.rs` (NEW - ~80 lines)
   - 5+ integration tests for timeout protection
   - Test timeout enforcement, long-running execution, error context

4. `airssys-wasm/tests/cpu_limits_test.rs` (NEW - ~380 lines)
   - 31+ comprehensive integration tests
   - Test all dual-layer coordination scenarios, security, edge cases

**Total Changes:**
- **Lines Added**: ~1,700+ lines (including tests)
- **New Tests**: 64+ tests (286+ total from current 239)
- **Test Files**: 3 new integration test files
- **Modified Files**: 6 existing files
- **New Files**: 4 new files (1 implementation + 3 test files)

### Success Criteria

**Functional Requirements:**
- ✅ Dual-layer CPU limiting operational (fuel + timeout)
- ✅ Fuel metering accurately tracks instruction execution
- ✅ Wall-clock timeout guarantees termination
- ✅ Component.toml `[resources.cpu]` parsing working
- ✅ Clear error differentiation (OutOfFuel vs ExecutionTimeout)
- ✅ Default CPU limits applied when section omitted

**Quality Requirements:**
- ✅ 64+ new tests passing (286+ total from current 239)
- ✅ Zero compiler/clippy warnings
- ✅ 100% rustdoc coverage for new public APIs
- ✅ Full workspace standards compliance (§2.1-§6.3)
- ✅ Professional, objective documentation (§7.2)

**Security Requirements:**
- ✅ No bypass opportunities for CPU limits
- ✅ Deterministic fuel behavior across platforms
- ✅ Guaranteed termination via timeout fallback
- ✅ Clear error context for debugging without leaking internals

**Performance Requirements:**
- ✅ Minimal overhead from fuel metering (<5% typical)
- ✅ Timeout accuracy within 10ms tolerance
- ✅ No performance regression in existing memory management

### Key Design Decisions

#### 1. CPU Limits Have Defaults (Unlike Memory)

**Decision:** CPU limits are OPTIONAL in Component.toml with sensible defaults

**Defaults:**
```rust
pub const DEFAULT_MAX_FUEL: u64 = 1_000_000;  // 1M fuel units
pub const DEFAULT_TIMEOUT_MS: u64 = 100;      // 100ms wall-clock timeout
```

**Rationale:**
- Memory limits are security-critical and must be explicit
- CPU limits are more about resource management and fairness
- Reasonable defaults prevent accidental resource exhaustion
- Users can override defaults in `[resources.cpu]` when needed

**Component.toml Example:**
```toml
# CPU limits are OPTIONAL - defaults applied if section omitted
[resources.cpu]
max_fuel = 5000000      # Override default 1M fuel
timeout_ms = 500        # Override default 100ms timeout
```

#### 2. Error Handling Priority

**Decision:** Check fuel exhaustion first, then timeout, then other trap reasons

**Priority Order:**
1. **OutOfFuel**: Fuel exhaustion detected (deterministic limit reached)
2. **ExecutionTimeout**: Wall-clock timeout exceeded (guaranteed termination)
3. **Trap**: Other WASM trap reasons (e.g., division by zero, unreachable)

**Rationale:**
- Fuel exhaustion is the primary, deterministic CPU limit
- Timeout is the safety net fallback
- Clear error differentiation aids debugging
- Consistent error priority prevents ambiguity

**Implementation Pattern:**
```rust
// Check fuel first
if let Some(fuel_info) = check_fuel_exhaustion(&trap) {
    return Err(WasmError::OutOfFuel { fuel_info });
}

// Then check timeout
if is_timeout(&error) {
    return Err(WasmError::ExecutionTimeout { 
        timeout_ms, 
        fuel_consumed 
    });
}

// Finally, other traps
Err(WasmError::Trap { reason })
```

#### 3. Dual-Layer Coordination

**Decision:** No race conditions between fuel metering and timeout mechanisms

**Coordination Strategy:**
- **Fuel metering**: Embedded in Wasmtime execution (synchronous)
- **Timeout wrapper**: Tokio task with timeout (asynchronous)
- **Error priority**: Fuel exhaustion checked before timeout
- **Context enrichment**: Timeout errors include fuel consumption info

**Race Condition Prevention:**
```rust
// Timeout wrapper includes fuel info regardless of error type
match tokio::time::timeout(timeout, execute_with_fuel(instance)).await {
    Ok(Ok(result)) => Ok(result),
    Ok(Err(e)) => {
        // Execution failed - check if fuel exhaustion
        if is_out_of_fuel(&e) {
            Err(WasmError::OutOfFuel { fuel_consumed, max_fuel })
        } else {
            Err(e)
        }
    }
    Err(_timeout) => {
        // Timeout exceeded - include fuel info in error
        let fuel_consumed = instance.get_fuel().unwrap_or(0);
        Err(WasmError::ExecutionTimeout { 
            timeout_ms, 
            fuel_consumed, // Context for debugging
            message: "Component execution exceeded wall-clock timeout"
        })
    }
}
```

**Benefits:**
- Clear error differentiation
- Fuel info always available for debugging
- No ambiguous error states
- Deterministic behavior in all scenarios

#### 4. Component.toml Configuration

**Decision:** Parse `[resources.cpu]` as OPTIONAL section with defaults

**Configuration Schema:**
```toml
[resources.cpu]
max_fuel = 1000000     # Optional: default 1M fuel units
timeout_ms = 100       # Optional: default 100ms wall-clock timeout
```

**Parsing Implementation:**
```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CpuConfig {
    #[serde(default = "default_max_fuel")]
    pub max_fuel: u64,
    
    #[serde(default = "default_timeout_ms")]
    pub timeout_ms: u64,
}

impl Default for CpuConfig {
    fn default() -> Self {
        Self {
            max_fuel: DEFAULT_MAX_FUEL,      // 1M fuel
            timeout_ms: DEFAULT_TIMEOUT_MS,  // 100ms
        }
    }
}
```

**Benefits:**
- Zero-config components get sensible CPU limits
- Explicit overrides when needed for specific use cases
- Consistent with serde's optional field handling
- Clear defaults documented in code and documentation

### Timeline and Effort Estimates

**Overall Timeline:** 4-7 days (22-27 hours total effort)

**Day 1: Fuel Metering Foundation** (6-7 hours)
- ⏱️ Task 3.1.1: Extend ResourceLimits with CPU fields (2-3h)
- ⏱️ Task 3.1.2: Add FuelConfig and FuelMetrics structs (1-2h)
- ⏱️ Task 3.1.3: Enable Wasmtime fuel metering (1h)
- ⏱️ Task 3.1.4: Update WasmError with OutOfFuel variant (1h)
- ⏱️ Task 3.1.5: Create fuel_metering_test.rs (1-2h)
- **Validation**: 11+ fuel metering tests passing, zero warnings

**Day 2: Fuel Integration + Timeout Start** (6-7 hours)
- ⏱️ Task 3.1.6: Final fuel metering integration and validation (2-3h)
- ⏱️ Task 3.2.1: Create runtime/executor.rs with ComponentExecutor (2-3h)
- ⏱️ Task 3.2.2: Implement Tokio timeout wrapper (1-2h)
- **Validation**: Fuel metering complete, executor foundation in place

**Day 3: Timeout Completion** (6-7 hours)
- ⏱️ Task 3.2.3: Parse [resources.cpu] in Component.toml (2-3h)
- ⏱️ Task 3.2.4: Update WasmError with ExecutionTimeout (1h)
- ⏱️ Task 3.2.5: Create timeout_protection_test.rs (2-3h)
- ⏱️ Task 3.2.6: Final timeout integration and validation (1h)
- **Validation**: Timeout protection operational, 5+ timeout tests passing

**Day 4: Testing and Validation** (4-6 hours)
- ⏱️ Task 3.3.1: Create cpu_limits_test.rs (3-4h)
- ⏱️ Task 3.3.2: Run comprehensive test suite validation (1h)
- ⏱️ Task 3.3.3: Performance tuning and optimization (1h)
- ⏱️ Task 3.3.4: Final documentation and completion (1h)
- **Validation**: 64+ new tests passing, zero warnings, dual-layer CPU limiting fully operational

**Effort Breakdown:**
- **Task 3.1** (Fuel Metering): 8-10 hours over Days 1-2
- **Task 3.2** (Timeout Protection): 8-10 hours over Days 2-3
- **Task 3.3** (Testing & Tuning): 6-8 hours on Days 3-4
- **Total**: 22-27 hours across 4-7 days

### Progress Tracking

**Current Status:**
- ✅ **Phase 2 Complete**: Memory management and sandboxing (239 tests, zero warnings)
- ✅ **Phase 3 Planning Complete**: Comprehensive implementation plan and task breakdown created
- 🔄 **Phase 3 Implementation**: Ready to begin - Day 1, Task 3.1.1

**Next Immediate Steps:**
1. **Begin Task 3.1.1**: Extend `ResourceLimits` with CPU fields
2. **Follow Day 1 Plan**: Complete fuel metering foundation (6-7 hours)
3. **Validate Incrementally**: Run tests after each subtask completion
4. **Update Progress**: Track completion in task_002_phase_3_task_breakdown.md

**Validation Checkpoints:**
- ✅ **Checkpoint 1 (Day 1)**: Fuel metering foundation complete, 11+ tests passing
- ⏳ **Checkpoint 2 (Day 2)**: Fuel integration + timeout foundation complete
- ⏳ **Checkpoint 3 (Day 3)**: Timeout protection operational, 5+ tests passing
- ⏳ **Checkpoint 4 (Day 4)**: Comprehensive testing complete, 64+ tests passing

---

## Reference Documentation

### Planning Documents Created

**1. Comprehensive Implementation Plan**
- **Path**: `.memory-bank/sub_projects/airssys-wasm/tasks/task_002_phase_3_implementation_plan.md`
- **Size**: 2,787 lines (85KB)
- **Content**: Day-by-day implementation guide with:
  - Complete code examples for all implementation steps
  - Detailed test specifications (64+ tests)
  - Validation procedures and success criteria
  - Integration patterns and error handling
  - Architecture decisions and rationale

**2. Structured Task Breakdown**
- **Path**: `.memory-bank/sub_projects/airssys-wasm/tasks/task_002_phase_3_task_breakdown.md`
- **Size**: 1,118 lines
- **Content**: Actionable subtasks with:
  - Granular file-by-file changes
  - Time estimates (22-27 hours total)
  - Clear dependencies and sequencing
  - Validation checkpoints
  - Confirmed by task-plans subagent

### Architecture Decision Records

**ADR-WASM-002: WASM Runtime Engine Selection**
- **Path**: `.memory-bank/sub_projects/airssys-wasm/docs/adr/adr_wasm_002_wasm_runtime_engine_selection.md`
- **Decision 3b**: Hybrid CPU limiting approach (fuel metering + wall-clock timeout)
- **Rationale**: Combines deterministic fuel metering with guaranteed termination via timeout
- **Status**: Accepted (2025-10-21)

### Phase 2 Success Patterns

**Phase 2 Completion Summary**
- **Path**: `.memory-bank/sub_projects/airssys-wasm/tasks/task_002_phase_2_completion_summary.md`
- **Lessons Learned**: Incremental implementation, comprehensive testing, workspace compliance
- **Success Metrics**: 239 tests passing, zero warnings, 100% memory isolation
- **Patterns to Replicate**: Day-by-day validation, integration test focus, rustdoc coverage

### Memory Bank Core Files

**airssys-wasm Sub-Project Context:**
- `.memory-bank/sub_projects/airssys-wasm/project_brief.md` - Project vision and objectives
- `.memory-bank/sub_projects/airssys-wasm/tech_context.md` - Technology stack and constraints
- `.memory-bank/sub_projects/airssys-wasm/progress.md` - Implementation status and milestones
- `.memory-bank/sub_projects/airssys-wasm/tasks/task_002_wasm_runtime_implementation.md` - WASM-TASK-002 master task file

**Workspace Standards:**
- `.memory-bank/workspace/shared_patterns.md` - §2.1-§6.2 mandatory code standards
- `.memory-bank/workspace/microsoft_rust_guidelines.md` - Complete Microsoft Rust Guidelines
- `.memory-bank/workspace/documentation_terminology_standards.md` - Professional documentation standards

---

## Key Technical Insights

### Architectural Patterns Established

**1. Dual-Layer Protection Pattern**
- **Layer 1 (Deterministic)**: Instruction-level fuel metering via Wasmtime
- **Layer 2 (Guaranteed)**: Wall-clock timeout via Tokio
- **Coordination**: Fuel-first error priority, timeout fallback with fuel context
- **Benefits**: Robust CPU limiting with no security gaps, clear error differentiation

**2. Default Configuration Pattern**
- **Memory limits**: REQUIRED explicit configuration (security-critical)
- **CPU limits**: OPTIONAL with sensible defaults (resource management)
- **Rationale**: Balance security requirements with usability
- **Implementation**: Serde defaults with optional Component.toml overrides

**3. Error Context Enrichment Pattern**
- **OutOfFuel errors**: Include fuel consumed, max fuel, clear message
- **ExecutionTimeout errors**: Include timeout value, fuel consumed (context), clear message
- **Benefits**: Rich debugging context without leaking internal state
- **Implementation**: Structured error variants with `Display` formatting

**4. Incremental Validation Pattern**
- **Per-Task Validation**: Run tests after each subtask completion
- **Day-End Validation**: Comprehensive test suite validation at end of each day
- **Final Validation**: Complete integration test suite (64+ tests) at phase completion
- **Benefits**: Early issue detection, prevents regression, ensures quality at each step

### Implementation Best Practices

**From Phase 2 Success:**
1. **Incremental Implementation**: Small, testable changes validated continuously
2. **Integration Test Focus**: Comprehensive integration tests over excessive unit tests
3. **Workspace Compliance**: Strict adherence to §2.1-§6.3 standards throughout
4. **Rustdoc Coverage**: 100% rustdoc coverage for all new public APIs
5. **Zero Warnings Policy**: Address all compiler/clippy warnings immediately

**Phase 3 Specific:**
1. **Dual-Layer Coordination**: Ensure fuel metering and timeout work together without race conditions
2. **Clear Error Priority**: Fuel exhaustion → Timeout → Other traps (consistent order)
3. **Default Configuration**: Sensible defaults for CPU limits (unlike memory)
4. **Security First**: Comprehensive bypass attempt testing in security test suite
5. **Performance Monitoring**: Track fuel metering overhead, timeout accuracy

### Technical Challenges and Solutions

**Challenge 1: Fuel Metering Configuration**
- **Issue**: Wasmtime fuel metering must be enabled before component instantiation
- **Solution**: Configure fuel in `WasmEngineBuilder`, set fuel on Store before execution
- **Pattern**: `engine.set_fuel_enabled(true)` → `store.add_fuel(max_fuel)` → execute

**Challenge 2: Timeout Coordination**
- **Issue**: Tokio timeout and fuel exhaustion can occur simultaneously
- **Solution**: Check fuel exhaustion first in error handling, then timeout
- **Pattern**: Error priority order prevents ambiguous error states

**Challenge 3: Error Context Without Leaking State**
- **Issue**: Provide useful debugging info without exposing internal implementation details
- **Solution**: Structured error variants with high-level context (fuel consumed, timeout value)
- **Pattern**: `OutOfFuel { fuel_consumed, max_fuel }`, `ExecutionTimeout { timeout_ms, fuel_consumed }`

**Challenge 4: Default vs Explicit Configuration**
- **Issue**: Balance security requirements with usability
- **Solution**: Memory limits REQUIRED (security), CPU limits OPTIONAL with defaults (resource mgmt)
- **Pattern**: Serde `#[serde(default)]` for CPU limits, explicit parsing for memory limits

---

## Next Steps

### Immediate Actions (Day 1)

**Start Implementation:**
1. **Read Phase 3 Planning Documents**
   - Review `task_002_phase_3_implementation_plan.md` (2,787 lines)
   - Review `task_002_phase_3_task_breakdown.md` (1,118 lines)
   - Understand complete implementation context

2. **Begin Task 3.1.1: Extend ResourceLimits with CPU Fields** (2-3 hours)
   - Add `max_fuel: Option<u64>` to `ResourceLimits`
   - Add `timeout_ms: Option<u64>` to `ResourceLimits`
   - Implement CPU config helpers (`with_fuel_limit()`, `with_timeout()`)
   - Update tests to cover new CPU fields
   - **Validation**: Compile without warnings, existing tests pass

3. **Continue Day 1 Tasks** (6-7 hours total)
   - Task 3.1.2: Add FuelConfig and FuelMetrics structs (1-2h)
   - Task 3.1.3: Enable Wasmtime fuel metering (1h)
   - Task 3.1.4: Update WasmError with OutOfFuel variant (1h)
   - Task 3.1.5: Create fuel_metering_test.rs (1-2h)
   - **Day 1 Validation**: 11+ fuel metering tests passing, zero warnings

### Week 1 Objectives

**Days 1-2: Fuel Metering Implementation** (8-10 hours)
- Complete Task 3.1 (all 6 subtasks)
- Fuel metering operational with 11+ tests passing
- Zero compiler/clippy warnings maintained

**Days 2-3: Timeout Protection Implementation** (8-10 hours)
- Complete Task 3.2 (all 6 subtasks)
- Timeout wrapper operational with 5+ tests passing
- Component.toml `[resources.cpu]` parsing working

**Days 3-4: Comprehensive Testing** (6-8 hours)
- Complete Task 3.3 (all 4 subtasks)
- 64+ new tests passing (286+ total)
- Dual-layer CPU limiting fully validated

**Week 1 Target:** Phase 3 implementation complete (40% overall project progress)

### Strategic Milestones

**After Phase 3 Completion:**
- **Progress**: 40% complete (Phases 1-3 done)
- **Capability**: Memory + CPU resource limiting operational
- **Quality**: 286+ tests passing, zero warnings, 100% rustdoc coverage
- **Readiness**: Ready for Phase 4 (Component Lifecycle - instantiation, execute, cleanup)

**Phase 4 Planning (Future):**
- Component instantiation from .wasm files
- Component execution with resource limits applied
- Component cleanup and state management
- Integration with airssys-rt actor system

**Block 1 Completion (WASM-TASK-002):**
- All 5 phases complete (memory, CPU, lifecycle, testing, docs)
- Runtime module fully operational
- Foundation for remaining 10 implementation blocks

---

## Historical Context

### Project Evolution

**Oct 17, 2025:** airssys-wasm project initiated after airssys-rt 100% completion  
**Oct 18, 2025:** Core abstractions foundation planning (WASM-TASK-000)  
**Oct 22, 2025:** WASM-TASK-000 complete (9,283 lines, 363 tests, 15 core modules)  
**Oct 22, 2025:** WASM-TASK-001 marked SKIPPED (redundant with Phase 12 validation)  
**Oct 22, 2025:** WASM-TASK-002 Block 1 implementation started  
**Oct 22-23, 2025:** WASM-TASK-002 Phase 1 complete (memory limits foundation)  
**Oct 23, 2025:** WASM-TASK-002 Phase 2 complete (memory management & sandboxing)  
**Oct 23, 2025:** WASM-TASK-002 Phase 3 planning complete (this snapshot)

### Dependencies Resolved

**airssys-osl (100% COMPLETE ✅):**
- Provides secure system access primitives
- Foundation for WASM component system calls
- Security middleware patterns referenced in WASM security model

**airssys-rt (100% COMPLETE ✅):**
- Provides actor-based component hosting
- Supervisor patterns for component lifecycle management
- Performance benchmarking patterns (sub-microsecond latency targets)

**WASM-TASK-000 (100% COMPLETE ✅):**
- Core abstractions foundation (15 modules, 9,283 lines)
- Universal abstractions: Component, Capability, Error, Config
- Domain abstractions: Runtime, Interface, Actor, Security, Messaging, Storage, etc.
- 363 tests passing, 100% rustdoc coverage

**All Prerequisites Met:** Ready for Block 1 Implementation (WASM-TASK-002) 🚀

---

## Snapshot Metadata

**Snapshot Type:** Planning Session Completion  
**Session Focus:** WASM-TASK-002 Phase 3 - CPU Limiting and Resource Control  
**Planning Documents Created:** 2 comprehensive documents (3,905 lines total)  
**Implementation Readiness:** 100% ready to begin Day 1, Task 3.1.1  
**Next Session Expected Activity:** Implementation of dual-layer CPU limiting (Days 1-4)

**Restoration Instructions:**
1. Read this snapshot for complete Phase 3 context
2. Review `task_002_phase_3_implementation_plan.md` for detailed implementation guide
3. Review `task_002_phase_3_task_breakdown.md` for actionable subtask list
4. Begin implementation at Day 1, Task 3.1.1: Extend ResourceLimits with CPU fields
5. Follow day-by-day validation checkpoints for quality assurance

**Related Snapshots:**
- `2025-10-23_phase_2_plan_creation_session.md` - Phase 2 planning session (predecessor)
- Future: `2025-10-XX_phase_3_completion.md` - Phase 3 implementation completion (successor)

---

**END OF SNAPSHOT**

**Status:** ✅ WASM-TASK-002 Phase 3 Planning Complete - Ready for Implementation  
**Next Action:** Begin Day 1, Task 3.1.1 - Extend ResourceLimits with CPU fields  
**Timeline:** 4-7 days (22-27 hours) to Phase 3 completion  
**Target:** 40% overall project progress (Phases 1-3 complete, dual-layer CPU limiting operational)
