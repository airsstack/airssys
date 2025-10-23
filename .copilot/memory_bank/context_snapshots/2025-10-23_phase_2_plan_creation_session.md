# Context Snapshot: Phase 2 Implementation Plan Creation Session
**Date**: 2025-10-23  
**Sub-Project**: airssys-wasm  
**Task**: WASM-TASK-002 Phase 2 Implementation Plan Creation  
**Status**: Technical issue preventing file write - ready to retry

---

## Session Overview

### What We Accomplished
1. **Resumed from previous session** with comprehensive summary
2. **Completed all prerequisites** for Phase 2 plan creation:
   - ✅ Read ADR-WASM-002 (WASM Runtime Engine Selection - 1,100 lines)
   - ✅ Read ADR-WASM-006 (Component Isolation and Sandboxing - 494 lines)
   - ✅ Read workspace standards (shared_patterns.md, microsoft_rust_guidelines.md)
   - ✅ Read Phase 1 plan (1,155 lines - serves as template)
   - ✅ Extracted Phase 2 requirements from task_002_block_1_wasm_runtime_layer.md

3. **Identified technical issue**: Write tool experiencing JSON parsing error preventing file creation

### Current State

**Project Status**:
- **Current sub-project**: airssys-wasm (25% complete)
- **Foundation**: WASM-TASK-000 complete (9,283 lines, 363 tests, zero warnings)
- **Phase 1 status**: Implementation plan complete (ready to reference as template)
- **Dependencies**: airssys-osl (100% complete), airssys-rt (100% complete)

**Next Immediate Action**:
- **Primary deliverable**: Create `task_002_phase_2_implementation_plan.md`
- **Location**: `.copilot/memory_bank/sub_projects/airssys-wasm/tasks/`
- **Estimated size**: ~1,000-1,500 lines (following Phase 1 format)
- **Status**: Ready to write (all research complete, structure defined)

---

## Phase 2 Implementation Plan Structure (READY TO WRITE)

### Complete Outline

```markdown
task_002_phase_2_implementation_plan.md (~1,000-1,500 lines)

1. Executive Summary
   - Phase 2 overview (Memory Management and Sandboxing)
   - Deliverables summary (Tasks 2.1, 2.2, 2.3)
   - Duration estimate (5-10 days)
   - Key technical constraints

2. Context and Prerequisites
   - Current project state (25% complete, Phase 1 foundation)
   - Reference documents (ADR-002, ADR-006, workspace standards)
   - Technical requirements (memory limits, isolation, performance)
   - Foundation capabilities (from Phase 1)

3. Phase 2 Objectives
   - Primary objective: Memory management and isolation enforcement
   - Specific deliverables for each task:
     * Task 2.1: Linear Memory Limit Enforcement
     * Task 2.2: Component.toml Memory Configuration
     * Task 2.3: Memory Isolation Verification
   - Success criteria (10-12 checkpoints)

4. Implementation Details (Day-by-Day Breakdown)

   **Task 2.1: Linear Memory Limit Enforcement (Days 1-3)**
   
   Subtask 2.1.1: Create runtime/limits.rs module (Day 1, 2-3 hours)
   - File: airssys-wasm/src/runtime/limits.rs
   - Module declaration in runtime/mod.rs
   - Basic structure setup
   - Code example: module structure with imports
   
   Subtask 2.1.2: Implement ResourceLimits struct (Day 1, 2-3 hours)
   - Core struct definition with memory limits
   - Builder pattern for configuration
   - Validation logic
   - Code example: Complete ResourceLimits implementation
   
   Subtask 2.1.3: Implement MemoryConfig struct (Day 1, 2-3 hours)
   - Memory configuration struct
   - Parse from Component.toml
   - Validation (512KB-4MB range)
   - Code example: MemoryConfig with validation
   
   Subtask 2.1.4: Implement ComponentResourceLimiter (Day 2, 4-6 hours)
   - Wasmtime ResourceLimiter trait implementation
   - Memory allocation tracking
   - Limit enforcement logic
   - Code example: Full ComponentResourceLimiter trait impl
   
   Subtask 2.1.5: Memory usage monitoring (Day 2-3, 3-4 hours)
   - Real-time memory tracking
   - Usage metrics collection
   - Monitoring API
   - Code example: MemoryMonitor implementation
   
   Subtask 2.1.6: OOM error handling (Day 3, 2-3 hours)
   - Graceful OOM detection
   - Error propagation
   - Component shutdown on OOM
   - Code example: OOM handling with proper errors

   **Task 2.2: Component.toml Memory Configuration (Days 4-6)**
   
   Subtask 2.2.1: Update Component.toml parsing (Day 4, 3-4 hours)
   - Add [resources.memory] section parsing
   - MANDATORY field validation
   - Code example: Updated ComponentConfig struct
   
   Subtask 2.2.2: Validation logic (Day 4-5, 4-5 hours)
   - Enforce MANDATORY memory limits
   - Range validation (512KB-4MB)
   - Clear error messages
   - Code example: Validation with detailed errors
   
   Subtask 2.2.3: Error messages for missing limits (Day 5, 2-3 hours)
   - User-friendly error messages
   - Documentation references
   - Configuration examples
   - Code example: Error types and messages
   
   Subtask 2.2.4: Memory limit enforcement pipeline (Day 5-6, 4-5 hours)
   - Integration with ComponentResourceLimiter
   - Runtime enforcement
   - End-to-end validation
   - Code example: Complete enforcement pipeline

   **Task 2.3: Memory Isolation Verification (Days 7-9)**
   
   Subtask 2.3.1: Component memory boundary tests (Day 7, 3-4 hours)
   - Single component memory limit tests
   - OOM behavior validation
   - Code example: Unit tests for memory boundaries
   
   Subtask 2.3.2: Cross-component isolation tests (Day 7-8, 4-5 hours)
   - Multi-component memory independence
   - Verify no cross-contamination
   - Code example: Integration tests for isolation
   
   Subtask 2.3.3: Memory leak detection tests (Day 8, 3-4 hours)
   - Long-running component tests
   - Memory growth monitoring
   - Code example: Leak detection tests
   
   Subtask 2.3.4: Stress testing (Day 8-9, 4-5 hours)
   - High memory usage scenarios
   - Concurrent component stress tests
   - Code example: Stress test suite
   
   Subtask 2.3.5: 100% isolation verification (Day 9, 3-4 hours)
   - Comprehensive isolation validation
   - Security boundary verification
   - Code example: Complete isolation test suite

5. Testing Strategy
   
   **Unit Tests** (~50 tests):
   - ResourceLimits configuration (8 tests)
   - MemoryConfig validation (10 tests)
   - ComponentResourceLimiter logic (12 tests)
   - Memory monitoring (8 tests)
   - OOM handling (12 tests)
   
   **Integration Tests** (~30 tests):
   - End-to-end memory enforcement (10 tests)
   - Component.toml parsing pipeline (8 tests)
   - Cross-component isolation (12 tests)
   
   **Security Tests** (~15 tests):
   - Memory isolation breaches (5 tests)
   - OOM attack scenarios (5 tests)
   - Resource exhaustion (5 tests)
   
   **Performance Tests** (~10 tests):
   - Memory tracking overhead (<5% target)
   - Enforcement performance impact
   - Monitoring efficiency

6. Standards Compliance Checklist
   - [ ] §2.1: 3-layer imports (std → external → internal)
   - [ ] §3.2: chrono DateTime<Utc> for all timestamps
   - [ ] §4.3: mod.rs declaration-only (no implementation)
   - [ ] §5.1: Workspace dependencies
   - [ ] §6.1: YAGNI principles (no speculative features)
   - [ ] §6.2: Avoid dyn patterns (prefer generics)
   - [ ] §6.3: Microsoft Rust Guidelines compliance
   - [ ] M-DESIGN-FOR-AI: Idiomatic APIs, thorough docs
   - [ ] M-DI-HIERARCHY: Concrete > generics > dyn
   - [ ] M-ERRORS-CANONICAL-STRUCTS: Structured errors with Backtrace
   - [ ] M-MOCKABLE-SYSCALLS: All I/O mockable

7. Success Validation Criteria
   - [ ] Memory limits enforced at runtime
   - [ ] Components cannot exceed configured limits
   - [ ] OOM handling graceful and informative
   - [ ] Component.toml validation rejects missing limits
   - [ ] 100% memory isolation verified
   - [ ] All unit tests passing (~50 tests)
   - [ ] All integration tests passing (~30 tests)
   - [ ] All security tests passing (~15 tests)
   - [ ] Performance overhead <5%
   - [ ] Zero compiler warnings
   - [ ] Zero clippy warnings
   - [ ] Documentation complete (rustdoc + mdBook)
```

---

## Key Technical Decisions (From ADRs)

### Memory Management Architecture (ADR-WASM-002)

**MANDATORY Memory Declaration**:
- **NO defaults**: Memory limits must be explicitly declared in Component.toml
- **Rationale**: Forces engineers to think about resource usage
- **Philosophy**: Prevent "works on my machine" production surprises
- **Range**: 512KB (minimum) to 4MB (maximum)

**Enforcement Mechanism**:
- **Primary**: Wasmtime ResourceLimiter trait implementation
- **Tracking**: Real-time memory allocation monitoring
- **Validation**: Pre-instantiation configuration check
- **Error handling**: Graceful OOM with clear error messages

**Performance Targets**:
- **Overhead**: <5% for memory tracking and enforcement
- **Instantiation**: <10ms component instantiation time
- **Memory**: <512KB baseline memory per component

### Isolation Architecture (ADR-WASM-006)

**4-Layer Defense-in-Depth**:
1. **Capability Layer**: WASI-preview2 capability-based security
2. **WASM Layer**: Linear memory isolation (this phase's focus)
3. **Actor Layer**: airssys-rt lightweight process isolation
4. **Supervision Layer**: Erlang-style supervisor trees

**Memory Isolation Requirements**:
- **100% isolation**: Critical security boundary (MANDATORY)
- **No shared memory**: Components cannot access each other's memory
- **Enforcement**: WASM linear memory + ResourceLimiter
- **Verification**: Comprehensive test suite required

**Integration with airssys-rt**:
- **ComponentActor**: Dual-trait design (Actor + Child)
- **Spawn overhead**: 625ns per actor (extremely lightweight)
- **Supervision**: Automatic restart on OOM failures
- **Message passing**: Zero-copy where possible

---

## Code Examples to Include in Plan

### 1. ResourceLimits Struct (Complete Implementation)

```rust
// airssys-wasm/src/runtime/limits.rs

use std::time::Duration;
use thiserror::Error;

#[derive(Debug, Clone, Error)]
pub enum ResourceLimitError {
    #[error("Memory limit {0} bytes is below minimum of 524288 bytes (512KB)")]
    MemoryTooLow(usize),
    
    #[error("Memory limit {0} bytes exceeds maximum of 4194304 bytes (4MB)")]
    MemoryTooHigh(usize),
    
    #[error("Memory limit not specified in Component.toml [resources.memory] section")]
    MemoryNotConfigured,
}

#[derive(Debug, Clone)]
pub struct ResourceLimits {
    memory_bytes: usize,
    cpu_timeout: Duration,
}

impl ResourceLimits {
    const MIN_MEMORY: usize = 512 * 1024; // 512KB
    const MAX_MEMORY: usize = 4 * 1024 * 1024; // 4MB
    const DEFAULT_CPU_TIMEOUT: Duration = Duration::from_secs(30);

    pub fn builder() -> ResourceLimitsBuilder {
        ResourceLimitsBuilder::default()
    }

    pub fn memory_bytes(&self) -> usize {
        self.memory_bytes
    }

    pub fn cpu_timeout(&self) -> Duration {
        self.cpu_timeout
    }

    pub fn validate(&self) -> Result<(), ResourceLimitError> {
        if self.memory_bytes < Self::MIN_MEMORY {
            return Err(ResourceLimitError::MemoryTooLow(self.memory_bytes));
        }
        if self.memory_bytes > Self::MAX_MEMORY {
            return Err(ResourceLimitError::MemoryTooHigh(self.memory_bytes));
        }
        Ok(())
    }
}

#[derive(Debug, Default)]
pub struct ResourceLimitsBuilder {
    memory_bytes: Option<usize>,
    cpu_timeout: Option<Duration>,
}

impl ResourceLimitsBuilder {
    pub fn memory_bytes(mut self, bytes: usize) -> Self {
        self.memory_bytes = Some(bytes);
        self
    }

    pub fn cpu_timeout(mut self, timeout: Duration) -> Self {
        self.cpu_timeout = Some(timeout);
        self
    }

    pub fn build(self) -> Result<ResourceLimits, ResourceLimitError> {
        let memory_bytes = self
            .memory_bytes
            .ok_or(ResourceLimitError::MemoryNotConfigured)?;

        let limits = ResourceLimits {
            memory_bytes,
            cpu_timeout: self.cpu_timeout.unwrap_or(ResourceLimits::DEFAULT_CPU_TIMEOUT),
        };

        limits.validate()?;
        Ok(limits)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_valid_memory_limits() {
        let limits = ResourceLimits::builder()
            .memory_bytes(1024 * 1024) // 1MB
            .build()
            .unwrap();
        assert_eq!(limits.memory_bytes(), 1024 * 1024);
    }

    #[test]
    fn test_memory_too_low() {
        let result = ResourceLimits::builder()
            .memory_bytes(256 * 1024) // 256KB - below minimum
            .build();
        assert!(matches!(result, Err(ResourceLimitError::MemoryTooLow(_))));
    }

    #[test]
    fn test_memory_too_high() {
        let result = ResourceLimits::builder()
            .memory_bytes(8 * 1024 * 1024) // 8MB - above maximum
            .build();
        assert!(matches!(result, Err(ResourceLimitError::MemoryTooHigh(_))));
    }

    #[test]
    fn test_memory_not_configured() {
        let result = ResourceLimits::builder().build();
        assert!(matches!(result, Err(ResourceLimitError::MemoryNotConfigured)));
    }
}
```

### 2. ComponentResourceLimiter (Wasmtime Trait Implementation)

```rust
// airssys-wasm/src/runtime/limits.rs (continued)

use wasmtime::{ResourceLimiter, ResourceLimiterAsync};
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::Arc;

pub struct ComponentResourceLimiter {
    limits: ResourceLimits,
    current_memory: Arc<AtomicUsize>,
}

impl ComponentResourceLimiter {
    pub fn new(limits: ResourceLimits) -> Self {
        Self {
            limits,
            current_memory: Arc::new(AtomicUsize::new(0)),
        }
    }

    pub fn current_memory_usage(&self) -> usize {
        self.current_memory.load(Ordering::SeqCst)
    }

    pub fn memory_limit(&self) -> usize {
        self.limits.memory_bytes()
    }
}

impl ResourceLimiter for ComponentResourceLimiter {
    fn memory_growing(
        &mut self,
        current: usize,
        desired: usize,
        _maximum: Option<usize>,
    ) -> anyhow::Result<bool> {
        // Check if new size would exceed our configured limit
        if desired > self.limits.memory_bytes() {
            return Ok(false); // Deny allocation
        }

        // Update current usage tracking
        self.current_memory.store(desired, Ordering::SeqCst);
        Ok(true) // Allow allocation
    }

    fn table_growing(
        &mut self,
        _current: u32,
        _desired: u32,
        _maximum: Option<u32>,
    ) -> anyhow::Result<bool> {
        // Allow table growth (separate from memory limits)
        Ok(true)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_memory_within_limits() {
        let limits = ResourceLimits::builder()
            .memory_bytes(1024 * 1024)
            .build()
            .unwrap();
        let mut limiter = ComponentResourceLimiter::new(limits);

        let result = limiter.memory_growing(0, 512 * 1024, None);
        assert!(result.unwrap());
        assert_eq!(limiter.current_memory_usage(), 512 * 1024);
    }

    #[test]
    fn test_memory_exceeds_limits() {
        let limits = ResourceLimits::builder()
            .memory_bytes(1024 * 1024)
            .build()
            .unwrap();
        let mut limiter = ComponentResourceLimiter::new(limits);

        let result = limiter.memory_growing(0, 2 * 1024 * 1024, None);
        assert!(!result.unwrap()); // Should deny
    }
}
```

### 3. Component.toml Memory Configuration

```toml
# Example Component.toml with MANDATORY memory configuration

[component]
name = "my-component"
version = "0.1.0"

[resources.memory]
# MANDATORY: Must specify memory limits explicitly
# Range: 512KB (524288) to 4MB (4194304)
max_bytes = 1048576  # 1MB

[resources.cpu]
# Optional: CPU timeout (defaults to 30s)
timeout_seconds = 60
```

### 4. MemoryConfig Struct with Validation

```rust
// airssys-wasm/src/core/component.rs (additions)

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ComponentConfig {
    pub component: ComponentMetadata,
    #[serde(default)]
    pub resources: ResourcesConfig,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct ResourcesConfig {
    pub memory: Option<MemoryConfig>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MemoryConfig {
    pub max_bytes: usize,
}

impl ComponentConfig {
    pub fn validate(&self) -> Result<(), ConfigError> {
        // MANDATORY: Memory configuration must be present
        let memory_config = self.resources.memory
            .as_ref()
            .ok_or(ConfigError::MissingMemoryConfig)?;

        // Validate memory limits
        let limits = ResourceLimits::builder()
            .memory_bytes(memory_config.max_bytes)
            .build()
            .map_err(ConfigError::InvalidResourceLimits)?;

        Ok(())
    }
}

#[derive(Debug, Error)]
pub enum ConfigError {
    #[error("Missing [resources.memory] section in Component.toml - memory limits are MANDATORY")]
    MissingMemoryConfig,
    
    #[error("Invalid resource limits: {0}")]
    InvalidResourceLimits(#[from] ResourceLimitError),
}
```

---

## New Files Phase 2 Will Create

```
airssys-wasm/src/runtime/
├── limits.rs              # NEW: ResourceLimits, MemoryConfig, ComponentResourceLimiter
└── instance.rs            # NEW: WasmInstance with memory tracking

airssys-wasm/tests/
├── memory_limits_tests.rs          # NEW: Memory limit enforcement tests
├── memory_isolation_tests.rs       # NEW: Cross-component isolation tests
└── memory_stress_tests.rs          # NEW: Stress and leak detection tests
```

---

## Testing Strategy Details

### Unit Test Categories (~50 tests)

**ResourceLimits Configuration (8 tests)**:
- Valid memory limits (512KB, 1MB, 4MB)
- Below minimum (256KB rejection)
- Above maximum (8MB rejection)
- Missing memory configuration
- Default CPU timeout
- Custom CPU timeout
- Builder pattern validation
- Edge case: exactly 512KB, exactly 4MB

**MemoryConfig Validation (10 tests)**:
- Valid Component.toml parsing
- Missing [resources.memory] section
- Invalid memory values
- Memory too low error message
- Memory too high error message
- TOML syntax errors
- Default values handling
- Optional fields parsing
- Multiple components configuration
- Nested configuration structures

**ComponentResourceLimiter Logic (12 tests)**:
- Memory allocation within limits
- Memory allocation exceeding limits
- Multiple allocation attempts
- Memory deallocation tracking
- Current usage reporting
- Concurrent allocation attempts
- Table growth (should allow)
- Edge case: allocation at exact limit
- Edge case: allocation 1 byte over limit
- Atomic usage tracking
- Limiter cloning behavior
- Thread safety verification

**Memory Monitoring (8 tests)**:
- Current usage tracking
- Usage over time monitoring
- Peak usage recording
- Usage metrics collection
- Monitoring API correctness
- Metrics reset functionality
- Concurrent monitoring
- Performance overhead measurement

**OOM Handling (12 tests)**:
- OOM detection accuracy
- Graceful component shutdown
- Error propagation correctness
- Error message clarity
- Component state cleanup
- Resource deallocation on OOM
- Supervisor notification
- Restart behavior
- Multiple concurrent OOM scenarios
- OOM during initialization
- OOM during execution
- OOM recovery testing

### Integration Test Categories (~30 tests)

**End-to-End Memory Enforcement (10 tests)**:
- Component instantiation with limits
- Memory allocation during execution
- OOM triggering during runtime
- Multiple components with different limits
- Limit enforcement across restarts
- Configuration reload scenarios
- Runtime limit updates (if supported)
- Component migration scenarios
- Persistence of usage metrics
- Full lifecycle enforcement

**Component.toml Parsing Pipeline (8 tests)**:
- Valid configuration loading
- Invalid configuration rejection
- Missing configuration handling
- Configuration validation pipeline
- Error message generation
- Configuration inheritance
- Multiple configuration sources
- Default value merging

**Cross-Component Isolation (12 tests)**:
- Two components memory independence
- Multiple components memory independence
- Component A OOM doesn't affect Component B
- Concurrent memory allocation
- Isolation under stress
- Memory boundaries enforcement
- No cross-contamination verification
- Isolation during restart
- Isolation during failure
- Isolation during shutdown
- 100% isolation verification
- Security boundary validation

### Security Test Categories (~15 tests)

**Memory Isolation Breaches (5 tests)**:
- Attempt to access other component memory
- Attempt to exceed memory limits via tricks
- Buffer overflow prevention
- Pointer manipulation prevention
- Memory safety verification

**OOM Attack Scenarios (5 tests)**:
- Rapid allocation attack
- Gradual memory exhaustion
- Coordinated multi-component attack
- Recursive allocation attack
- Denial of service prevention

**Resource Exhaustion (5 tests)**:
- System-wide memory exhaustion
- Component starvation scenarios
- Fair resource allocation
- Priority-based allocation (if supported)
- Graceful degradation under pressure

### Performance Test Categories (~10 tests)

**Memory Tracking Overhead (<5% target)**:
- Baseline allocation performance
- Allocation with tracking enabled
- Overhead measurement
- Performance regression detection

**Enforcement Performance Impact**:
- Instantiation time impact
- Execution time impact
- Throughput impact
- Latency impact

**Monitoring Efficiency**:
- Metrics collection overhead
- Monitoring API performance
- Concurrent monitoring performance
- Scalability testing

---

## Standards Compliance Verification

### Code Quality Standards

**§2.1 - 3-Layer Import Organization**:
```rust
// Layer 1: Standard library
use std::sync::Arc;
use std::time::Duration;

// Layer 2: External crates
use wasmtime::{ResourceLimiter, Store};
use thiserror::Error;

// Layer 3: Internal modules
use crate::core::component::ComponentConfig;
use crate::runtime::engine::WasmEngine;
```

**§3.2 - chrono DateTime<Utc> Standard**:
```rust
use chrono::{DateTime, Utc};

pub struct MemoryMetrics {
    recorded_at: DateTime<Utc>,
    current_usage: usize,
}
```

**§4.3 - Module Architecture (mod.rs declaration-only)**:
```rust
// airssys-wasm/src/runtime/mod.rs
pub mod engine;
pub mod limits;    // NEW
pub mod instance;  // NEW

pub use limits::{ResourceLimits, ComponentResourceLimiter, MemoryConfig};
pub use instance::WasmInstance;
```

**§6.1 - YAGNI Principles**:
- Implement only Task 2.1, 2.2, 2.3 requirements
- No speculative features (e.g., dynamic limit updates)
- No premature optimization (measure first)
- Simple solutions first

**§6.2 - Avoid dyn Patterns**:
```rust
// ✅ CORRECT - Use generic constraints
pub fn create_limiter<C: Into<ResourceLimits>>(config: C) -> ComponentResourceLimiter {
    ComponentResourceLimiter::new(config.into())
}

// ❌ FORBIDDEN - Avoid dyn trait objects
pub fn create_limiter(config: Box<dyn ResourceConfig>) -> ComponentResourceLimiter {
    // Don't do this
}
```

**§6.3 - Microsoft Rust Guidelines**:
- M-DESIGN-FOR-AI: Clear, idiomatic APIs with thorough documentation
- M-DI-HIERARCHY: Concrete types > generics > dyn traits
- M-ERRORS-CANONICAL-STRUCTS: Structured errors with Backtrace
- M-MOCKABLE-SYSCALLS: All Wasmtime interactions mockable for testing

---

## Success Validation Checklist

### Functional Requirements
- [ ] Memory limits enforced at runtime via Wasmtime ResourceLimiter
- [ ] Components cannot exceed configured limits (verified with tests)
- [ ] OOM handling graceful with informative error messages
- [ ] Component.toml validation rejects missing [resources.memory] section
- [ ] Memory range validation (512KB-4MB) working correctly
- [ ] 100% memory isolation between components verified

### Testing Requirements
- [ ] All unit tests passing (~50 tests total)
  - [ ] ResourceLimits tests (8 tests)
  - [ ] MemoryConfig tests (10 tests)
  - [ ] ComponentResourceLimiter tests (12 tests)
  - [ ] Memory monitoring tests (8 tests)
  - [ ] OOM handling tests (12 tests)
- [ ] All integration tests passing (~30 tests total)
  - [ ] End-to-end enforcement (10 tests)
  - [ ] Configuration pipeline (8 tests)
  - [ ] Cross-component isolation (12 tests)
- [ ] All security tests passing (~15 tests total)
  - [ ] Isolation breach prevention (5 tests)
  - [ ] OOM attack prevention (5 tests)
  - [ ] Resource exhaustion prevention (5 tests)
- [ ] All performance tests passing (~10 tests total)
  - [ ] Memory tracking overhead <5%
  - [ ] Enforcement performance acceptable

### Quality Requirements
- [ ] Zero compiler warnings
- [ ] Zero clippy warnings with `--all-targets --all-features`
- [ ] Documentation complete:
  - [ ] All public APIs have rustdoc comments
  - [ ] Code examples in documentation tested
  - [ ] Architecture documentation updated
- [ ] Standards compliance verified:
  - [ ] §2.1-§6.3 workspace standards followed
  - [ ] Microsoft Rust Guidelines compliance verified

### Integration Requirements
- [ ] Component.toml parsing updated and tested
- [ ] WasmEngine integration with ComponentResourceLimiter
- [ ] Error handling integrated with existing error types
- [ ] Monitoring API accessible from public interface

---

## Risk Assessment and Mitigation

### Technical Risks

**Risk 1: Wasmtime ResourceLimiter API changes**
- **Likelihood**: Low (stable API)
- **Impact**: High (core functionality)
- **Mitigation**: Pin Wasmtime version, monitor changelog

**Risk 2: Performance overhead exceeds 5% target**
- **Likelihood**: Medium
- **Impact**: Medium
- **Mitigation**: Benchmark early, optimize hotspots, consider caching

**Risk 3: 100% isolation difficult to verify**
- **Likelihood**: Medium
- **Impact**: High (security requirement)
- **Mitigation**: Comprehensive test suite, property-based testing, security review

**Risk 4: OOM handling edge cases**
- **Likelihood**: Medium
- **Impact**: Medium
- **Mitigation**: Extensive testing, fuzzing, error injection tests

### Schedule Risks

**Risk 1: Implementation takes longer than 9 days**
- **Likelihood**: Medium
- **Impact**: Low (no hard deadline)
- **Mitigation**: Break into smaller increments, parallel testing development

**Risk 2: Testing reveals architectural issues**
- **Likelihood**: Low (well-researched)
- **Impact**: High
- **Mitigation**: Early integration testing, continuous validation

---

## Post-Phase 2 Integration Plan

### Integration with Phase 3 (CPU Limiting)
- ResourceLimits struct already includes CPU timeout field
- ComponentResourceLimiter can be extended for CPU metering
- Monitoring infrastructure reusable for CPU metrics

### Integration with airssys-rt
- ComponentActor will use ComponentResourceLimiter
- Supervisor trees will monitor OOM failures
- Automatic restart on OOM with exponential backoff

### Integration with Security Middleware
- Memory limits as security policy enforcement
- Audit logging for limit violations
- Security context integration

---

## Documentation Updates Required

### API Documentation (rustdoc)
- [ ] Document ResourceLimits public API
- [ ] Document ComponentResourceLimiter trait implementation
- [ ] Document MemoryConfig struct
- [ ] Add usage examples to module-level docs
- [ ] Document error types and handling

### Architecture Documentation (mdBook)
- [ ] Update WASM runtime architecture chapter
- [ ] Add memory management section
- [ ] Document isolation guarantees
- [ ] Add configuration guide for Component.toml

### Memory Bank Updates
- [ ] Update progress.md (Phase 2 → 35% complete)
- [ ] Create knowledge doc for ResourceLimiter pattern
- [ ] Update tech_context.md with memory targets achieved

---

## Appendix: Key Reference Documents

### ADRs Read
- **ADR-WASM-002**: WASM Runtime Engine Selection (1,100 lines)
  - Runtime: Wasmtime v24.0+ with Component Model
  - Memory limits MANDATORY (no defaults - intentional design)
  - Memory range: 512KB-4MB configurable
  - Enforcement: ResourceLimiter trait implementation

- **ADR-WASM-006**: Component Isolation and Sandboxing (494 lines)
  - 4-layer defense-in-depth
  - 100% memory isolation requirement
  - ComponentActor dual-trait design
  - Integration with airssys-rt supervisor trees

### Templates Used
- **task_002_phase_1_implementation_plan.md** (1,155 lines)
  - Structure template for this plan
  - Code example patterns
  - Testing strategy format
  - Standards checklist approach

### Standards References
- **shared_patterns.md**: §2.1-§6.3 mandatory patterns
- **microsoft_rust_guidelines.md**: Complete M-* standards
- **documentation_terminology_standards.md**: Professional documentation standards

---

## Session End Notes

### Technical Issue Encountered
- Write tool experiencing JSON parsing error
- Unable to create `task_002_phase_2_implementation_plan.md` file
- All research and planning complete
- Ready to retry in fresh session

### Restart Instructions for Next Session
1. Read this snapshot: `2025-10-23_phase_2_plan_creation_session.md`
2. Use complete outline above to create implementation plan file
3. Location: `.copilot/memory_bank/sub_projects/airssys-wasm/tasks/task_002_phase_2_implementation_plan.md`
4. Estimated size: ~1,000-1,500 lines
5. Follow Phase 1 format (reference: `task_002_phase_1_implementation_plan.md`)
6. Include all code examples documented above
7. After creation: STOP and wait for user review

### Context Preserved
- All prerequisite research completed (ADRs, standards, Phase 1 template)
- Complete plan outline documented above (ready to write)
- All code examples prepared (ResourceLimits, ComponentResourceLimiter, etc.)
- Testing strategy fully defined (~105 tests across 4 categories)
- Standards compliance checklist ready
- Success criteria clearly defined

**Status**: Ready to create implementation plan file in next session.
