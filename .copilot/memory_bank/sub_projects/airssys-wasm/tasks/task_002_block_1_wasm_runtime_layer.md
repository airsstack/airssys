# [WASM-TASK-002] - Block 1: WASM Runtime Layer

**Status:** not-started  
**Added:** 2025-10-20  
**Updated:** 2025-10-20  
**Priority:** Critical Path - Foundation Layer  
**Layer:** 1 - Foundation  
**Block:** 1 of 11  
**Estimated Effort:** 4-6 weeks  

## Overview

Implement the foundational WASM runtime layer that executes WebAssembly Component Model modules with security and resource control. This block provides the core execution engine for all WASM components, including memory sandboxing, CPU limiting, async execution, and crash isolation.

## Context

**Current State:**
- Architecture complete: ADR-WASM-002 (WASM Runtime Engine Selection)
- Technology selected: Wasmtime with Component Model support
- Integration requirements: Tokio async runtime for airssys-rt compatibility
- Resource limiting strategy: Hybrid fuel metering + wall-clock timeout

**Problem Statement:**
The framework needs a robust WASM execution engine that:
1. Executes Component Model WASM modules (not core WASM)
2. Enforces strict memory limits (512KB-4MB configurable)
3. Prevents CPU abuse with dual-layer protection
4. Integrates seamlessly with Tokio async ecosystem
5. Isolates component crashes from host runtime
6. Provides fast instantiation (<10ms cold start target)

**Why This Block Matters:**
The WASM runtime layer is the absolute foundation of the framework. Without this:
- No WASM code can execute
- Security boundaries cannot be enforced
- Resource limiting is impossible
- Integration with airssys-rt fails

This block must complete before ANY other components can be implemented.

## Objectives

### Primary Objective
Build a production-ready WASM runtime engine using Wasmtime that executes Component Model modules with enforced memory/CPU limits, async support, and isolated crash handling.

### Secondary Objectives
- Achieve <10ms cold start instantiation time
- Implement comprehensive error handling and diagnostics
- Create reusable runtime configuration patterns
- Establish performance benchmarking baseline
- Document runtime behavior and limitations

## Scope

### In Scope
1. **Wasmtime Integration** - Component Model runtime setup
2. **Memory Management** - Linear memory sandbox with configurable limits
3. **CPU Limiting** - Hybrid fuel metering + wall-clock timeout
4. **Async Execution** - Tokio integration for async WASM functions
5. **Crash Isolation** - Component failure doesn't terminate host
6. **Configuration System** - Runtime limits from Component.toml
7. **Error Handling** - Comprehensive error types and recovery
8. **Performance Benchmarking** - Baseline metrics collection

### Out of Scope
- WIT interface bindings (Block 2)
- Actor system integration (Block 3)
- Capability-based security (Block 4)
- Multi-instance management (handled by actor system)
- AOT compilation (Phase 2 optimization)

## Implementation Plan

### Phase 1: Wasmtime Setup and Basic Execution (Week 1-2)

#### Task 1.1: Wasmtime Dependency and Engine Setup
**Deliverables:**
- Add wasmtime dependencies to Cargo.toml
- Create `runtime/` module structure
- Implement `WasmEngine` struct with `Config` setup
- Enable Component Model support
- Configure Cranelift JIT compiler
- Basic engine instantiation tests

**Success Criteria:**
- Engine initializes successfully
- Component Model features enabled
- Unit tests pass for engine setup

#### Task 1.2: Component Loading and Instantiation
**Deliverables:**
- `ComponentLoader` struct for .wasm file loading
- Component validation and verification
- Basic instantiation pipeline
- Module caching strategy
- Instantiation timing measurements

**Success Criteria:**
- Load valid Component Model .wasm files
- Reject invalid or malformed components
- Basic "hello world" component executes
- Instantiation < 10ms (cold start)

#### Task 1.3: Error Handling Foundation
**Deliverables:**
- `RuntimeError` enum with comprehensive variants
- Error context propagation
- Wasmtime error translation layer
- User-friendly error messages
- Error recovery strategies

**Success Criteria:**
- All Wasmtime errors properly translated
- Error messages are actionable
- Stack traces preserved where possible

---

### Phase 2: Memory Management and Sandboxing (Week 2-3)

#### Task 2.1: Linear Memory Limit Enforcement
**Deliverables:**
- Memory limit configuration system
- `MemoryConfig` struct (min, max, guard pages)
- Runtime memory limit validation
- Out-of-memory error handling
- Memory usage monitoring

**Success Criteria:**
- Memory limits enforced at runtime
- Components cannot exceed configured limits
- OOM errors caught and handled gracefully
- Memory usage queryable

#### Task 2.2: Component.toml Memory Configuration
**Deliverables:**
- Parse memory limits from Component.toml
- Validation of memory configuration values
- Default limit enforcement (no unlimited memory)
- Memory configuration documentation
- Configuration error handling

**Success Criteria:**
- Component.toml memory limits parsed correctly
- Invalid configurations rejected with clear errors
- REQUIRED field validation (no optional memory limits)
- Engineers must explicitly define limits

#### Task 2.3: Memory Isolation Verification
**Deliverables:**
- Component memory boundary tests
- Cross-component memory isolation validation
- Memory leak detection tests
- Stress testing with high memory usage
- Memory safety documentation

**Success Criteria:**
- Components cannot access each other's memory
- Memory leaks detected and documented
- High memory usage handled safely
- 100% memory isolation verified

---

### Phase 3: CPU Limiting and Resource Control (Week 3-4)

#### Task 3.1: Fuel Metering Implementation
**Deliverables:**
- Enable Wasmtime fuel metering
- Fuel consumption configuration
- Fuel limit calculation from Component.toml
- Fuel exhaustion handling
- Fuel consumption metrics

**Success Criteria:**
- Fuel metering tracks instruction execution
- Fuel limits enforced reliably
- Fuel exhaustion returns controlled error
- Fuel consumption measurable

#### Task 3.2: Wall-Clock Timeout Protection
**Deliverables:**
- Tokio-based timeout wrapper
- Timeout configuration from Component.toml
- Timeout preemption mechanism
- Timeout vs fuel interaction handling
- Dual-layer protection verification

**Success Criteria:**
- Wall-clock timeout enforced (e.g., 5 seconds)
- Timeout preempts long-running computation
- Works alongside fuel metering (hybrid)
- No race conditions between mechanisms

#### Task 3.3: CPU Limit Testing and Tuning
**Deliverables:**
- Infinite loop test cases
- CPU-bound computation tests
- Fuel/timeout calibration tests
- CPU limit bypass attempt tests
- CPU limiting documentation

**Success Criteria:**
- Infinite loops terminated within timeout
- CPU-bound work respects fuel limits
- No bypass vulnerabilities found
- Clear documentation of CPU protection

---

### Phase 4: Async Execution and Tokio Integration (Week 4-5)

#### Task 4.1: Async WASM Function Support
**Deliverables:**
- Wasmtime async configuration
- Async function call handling
- Tokio runtime integration
- Async error propagation
- Async call examples

**Success Criteria:**
- WASM async functions execute correctly
- Integrates with Tokio runtime
- Async errors handled properly
- No blocking operations on async runtime

#### Task 4.2: Async Host Function Calls
**Deliverables:**
- Async host function interface
- Host function call suspension/resumption
- Async host function error handling
- Performance of async boundaries
- Async host function examples

**Success Criteria:**
- WASM can call async host functions
- Execution suspends/resumes correctly
- Errors propagate through async boundary
- Minimal performance overhead

#### Task 4.3: Async Integration Testing
**Deliverables:**
- Complex async workflow tests
- Concurrent async call tests
- Async cancellation handling
- Async performance benchmarks
- Async pattern documentation

**Success Criteria:**
- Complex async patterns work correctly
- Concurrent calls don't interfere
- Cancellation is graceful
- Performance meets targets (<5% overhead)

---

### Phase 5: Crash Isolation and Recovery (Week 5-6)

#### Task 5.1: Component Crash Handling
**Deliverables:**
- Trap handler implementation
- Panic boundary around WASM execution
- Crash error categorization
- Crash logging and diagnostics
- Crash recovery strategy

**Success Criteria:**
- Component traps don't crash host
- Panics contained and reported
- Clear crash diagnostics available
- Host remains stable after crashes

#### Task 5.2: Resource Cleanup on Failure
**Deliverables:**
- Proper Drop implementation
- Resource cleanup on trap/panic
- Memory cleanup verification
- Handle cleanup (file descriptors, etc.)
- Cleanup testing

**Success Criteria:**
- Resources cleaned up on crash
- No resource leaks after failure
- Memory fully reclaimed
- Handles closed properly

#### Task 5.3: Crash Isolation Testing
**Deliverables:**
- Deliberate crash test suite
- Multiple crash scenario tests
- Crash recovery validation
- Stress testing with crashes
- Crash isolation documentation

**Success Criteria:**
- All crash types handled safely
- Host stability maintained
- No cascading failures
- Recovery is reliable

---

### Phase 6: Performance Baseline Establishment (Week 6)

#### Task 6.1: Instantiation Performance Baseline
**Deliverables:**
- Cold start measurement benchmarks
- Warm start measurement benchmarks
- Module caching effectiveness measurement
- Instantiation timing documentation
- Performance baseline report

**Success Criteria:**
- Cold start time measured and documented (target: <10ms)
- Warm start time measured and documented (target: <1ms)
- Module caching impact quantified
- Baseline performance characteristics documented
- No optimization work performed (measure only)

#### Task 6.2: Execution Performance Baseline
**Deliverables:**
- Criterion benchmark suite for execution
- Compute-heavy operation benchmarks
- Memory-intensive operation benchmarks
- Async operation overhead benchmarks
- Performance baseline documentation

**Success Criteria:**
- Execution performance baseline established
- Benchmarks reproducible and documented
- Performance characteristics understood
- Baseline metrics for future comparison
- No optimization performed (establish baseline first)

#### Task 6.3: Resource Usage Baseline
**Deliverables:**
- Memory usage profiling
- CPU usage profiling
- Runtime overhead measurements
- Resource usage documentation
- Baseline performance report

**Success Criteria:**
- Memory footprint characterized and documented
- CPU overhead quantified and documented
- Resource usage patterns identified
- Performance baseline complete
- Foundation for Phase 2 optimization established

**Note on Performance:**
This phase focuses on **baseline measurement only**, not optimization. Following the "make it work, make it right, make it fast" principle, we first establish what the current performance characteristics are. Optimization work will be deferred to Phase 2+ based on actual measured needs, not assumptions.

---

## Success Criteria

### Definition of Done
This task is complete when:

1. ✅ **Component Model Execution**
   - Load and execute Component Model .wasm files
   - Proper module validation and verification
   - Support for component imports/exports

2. ✅ **Memory Management**
   - Memory limits enforced from Component.toml
   - 512KB-4MB configurable range working
   - REQUIRED field validation (no defaults)
   - 100% memory isolation verified

3. ✅ **CPU Limiting**
   - Hybrid fuel metering + wall-clock timeout working
   - Infinite loops terminated reliably
   - CPU-bound work respects limits
   - No bypass vulnerabilities

4. ✅ **Async Execution**
   - Async WASM functions execute
   - Tokio integration seamless
   - Async host functions supported
   - <5% async overhead

5. ✅ **Crash Isolation**
   - Component crashes don't terminate host
   - All resources cleaned up on failure
   - Clear crash diagnostics available
   - Host stability maintained

6. ✅ **Performance Baseline**
   - Cold start time measured and documented
   - Warm start time measured and documented
   - Execution overhead measured and documented
   - Resource usage baseline established
   - Benchmark suite created for future tracking

7. ✅ **Testing & Documentation**
   - Comprehensive test suite (>90% coverage)
   - All phases tested thoroughly
   - Complete API documentation
   - Usage examples provided

## Dependencies

### Upstream Dependencies
- ✅ ADR-WASM-002: WASM Runtime Engine Selection - **COMPLETE**
- ✅ KNOWLEDGE-WASM-001: Component Framework Architecture - **COMPLETE**
- ✅ Wasmtime crate availability - **AVAILABLE** (v24.0+)
- ✅ Tokio runtime - **AVAILABLE** (from airssys-rt)

### Downstream Dependencies (Blocks This Task)
- WASM-TASK-003: WIT Interface System (needs runtime to load interfaces)
- WASM-TASK-004: Actor System Integration (needs runtime to host components)
- All subsequent blocks depend on this foundational runtime

### External Dependencies
- Wasmtime crate (v24.0+ with Component Model)
- Tokio async runtime (v1.47+)
- Cranelift JIT compiler (bundled with Wasmtime)

## Risks and Mitigations

### Risk 1: Wasmtime API Instability
**Impact:** High - API changes could require significant refactoring  
**Probability:** Low - Component Model is mature (v1.0)  
**Mitigation:**
- Pin Wasmtime version initially
- Follow Wasmtime release notes closely
- Plan for upgrade path in Phase 2
- Abstract Wasmtime behind internal API

### Risk 2: Performance Not Meeting Targets
**Impact:** Medium - Could affect user experience  
**Probability:** Medium - 10ms cold start is aggressive  
**Mitigation:**
- Implement module caching early
- Profile and optimize continuously
- Consider AOT compilation for Phase 2
- Document actual performance achieved

### Risk 3: Fuel Metering Insufficient
**Impact:** High - CPU abuse could DoS the system  
**Probability:** Low - Dual protection (fuel + timeout)  
**Mitigation:**
- Implement wall-clock timeout as backup
- Test with adversarial components
- Monitor fuel consumption patterns
- Adjust fuel limits based on real-world usage

### Risk 4: Async Complexity
**Impact:** Medium - Could introduce subtle bugs  
**Probability:** Medium - Async boundaries are complex  
**Mitigation:**
- Extensive async testing
- Follow Wasmtime async examples closely
- Code review all async code
- Document async patterns clearly

### Risk 5: Crash Handling Edge Cases
**Impact:** High - Unhandled crashes could crash host  
**Probability:** Low - Wasmtime has robust trap handling  
**Mitigation:**
- Comprehensive crash testing
- Fuzzing with malicious components
- Multiple layers of panic boundaries
- Monitoring in production

## Progress Tracking

**Overall Status:** not-started - 0%

### Phase Breakdown
| Phase | Description | Status | Estimated Duration | Notes |
|-------|-------------|--------|-------------------|-------|
| 1 | Wasmtime Setup and Basic Execution | not-started | Week 1-2 | Foundation |
| 2 | Memory Management and Sandboxing | not-started | Week 2-3 | Security critical |
| 3 | CPU Limiting and Resource Control | not-started | Week 3-4 | Security critical |
| 4 | Async Execution and Tokio Integration | not-started | Week 4-5 | airssys-rt compat |
| 5 | Crash Isolation and Recovery | not-started | Week 5-6 | Production readiness |
| 6 | Performance Baseline Establishment | not-started | Week 6 | Measurement not optimization |

### Subtasks
| ID | Description | Status | Updated | Notes |
|----|-------------|--------|---------|-------|
| 1.1 | Wasmtime Dependency and Engine Setup | not-started | - | Foundation setup |
| 1.2 | Component Loading and Instantiation | not-started | - | Basic execution |
| 1.3 | Error Handling Foundation | not-started | - | Reliability |
| 2.1 | Linear Memory Limit Enforcement | not-started | - | Security critical |
| 2.2 | Component.toml Memory Configuration | not-started | - | Configuration |
| 2.3 | Memory Isolation Verification | not-started | - | Security validation |
| 3.1 | Fuel Metering Implementation | not-started | - | CPU protection layer 1 |
| 3.2 | Wall-Clock Timeout Protection | not-started | - | CPU protection layer 2 |
| 3.3 | CPU Limit Testing and Tuning | not-started | - | Security validation |
| 4.1 | Async WASM Function Support | not-started | - | Async foundation |
| 4.2 | Async Host Function Calls | not-started | - | Host integration |
| 4.3 | Async Integration Testing | not-started | - | Async validation |
| 5.1 | Component Crash Handling | not-started | - | Isolation foundation |
| 5.2 | Resource Cleanup on Failure | not-started | - | Reliability |
| 5.3 | Crash Isolation Testing | not-started | - | Production readiness |
| 6.1 | Instantiation Performance Baseline | not-started | - | Measure cold/warm start |
| 6.2 | Execution Performance Baseline | not-started | - | Measure execution overhead |
| 6.3 | Resource Usage Baseline | not-started | - | Measure resource footprint |

## Progress Log

*No progress yet - task just created*

## Related Documentation

### ADRs
- **ADR-WASM-002: WASM Runtime Engine Selection** - Primary reference for all decisions
- **ADR-WASM-006: Component Isolation and Sandboxing** - Memory isolation requirements
- **ADR-WASM-005: Capability-Based Security Model** - Security context integration (Phase 2)

### Knowledge Documentation
- **KNOWLEDGE-WASM-001: Component Framework Architecture** - Overall architecture context
- **KNOWLEDGE-WASM-003: Core Architecture Design** - Integration requirements
- **KNOWLEDGE-WASM-004: WIT Management Architecture** - Component interface patterns

### External References
- [Wasmtime Documentation](https://docs.wasmtime.dev/)
- [WebAssembly Component Model](https://github.com/WebAssembly/component-model)
- [Wasmtime Async Support](https://docs.wasmtime.dev/api/wasmtime/struct.Config.html#method.async_support)
- [Wasmtime Fuel Metering](https://docs.wasmtime.dev/api/wasmtime/struct.Config.html#method.consume_fuel)

## Notes

**Critical Foundation:**
This block is the absolute foundation. Nothing else can proceed until WASM code can execute safely.

**Module Structure:**
Following KNOWLEDGE-WASM-012 (Module Structure Architecture), runtime code lives in `src/runtime/`:
- `runtime/engine.rs` - Wasmtime engine wrapper
- `runtime/instance.rs` - Component instance management
- `runtime/limits.rs` - Resource limits (memory, CPU)
- `runtime/loader.rs` - Component loading
- `runtime/executor.rs` - Component execution

**Security First:**
Memory and CPU limits are REQUIRED, not optional. Components MUST declare limits in Component.toml.

**Performance Baseline First:**
Phase 6 focuses on **measurement**, not optimization. We establish baseline performance characteristics before attempting optimization. "Make it work, make it right, make it fast" - we're at phase 1.

**Async Complexity:**
Wasmtime async support is mature but complex. Follow examples closely and test extensively.

**Crash Isolation:**
Production systems will have buggy components. Crash isolation is not optional.

**Testing Strategy:**
Include adversarial testing (infinite loops, OOM attempts, malformed components).

**Phase 1 Simplicity:**
This is Phase 1 - focus on correct, safe implementation. Optimizations can come in Phase 2 based on measured performance data.
