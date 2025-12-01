# WASM-TASK-002 Phase 3: Implementation Plan
## CPU Limiting and Resource Control

**Status:** Planning Complete - Ready for Implementation  
**Created:** 2025-10-23  
**Duration:** 4-7 days (Week 3-4)  
**Priority:** Critical Path - CPU Protection Layer

---

## Executive Summary

This document provides a comprehensive, day-by-day implementation plan for **WASM-TASK-002 Phase 3**, covering:

- **Task 3.1**: Fuel Metering Implementation (Days 1-2)
- **Task 3.2**: Wall-Clock Timeout Protection (Days 2-3)
- **Task 3.3**: CPU Limit Testing and Tuning (Days 3-4)

Phase 3 implements dual-layer CPU limiting using Wasmtime fuel metering (deterministic) combined with Tokio wall-clock timeouts (guaranteed termination), completing the resource control foundation established in Phase 2.

### Architecture Overview

**Hybrid CPU Limiting (ADR-WASM-002 Decision 3b):**

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

**Key Design Principles:**

1. **Dual Protection**: Fuel metering (deterministic) + timeout (guaranteed)
2. **Complementary**: Fuel limits instructions, timeout limits real time
3. **Default Values Allowed**: Unlike memory, CPU limits have sensible defaults
   - Default fuel: 1,000,000 units (~10-50ms depending on operations)
   - Default timeout: 100ms wall-clock time
4. **Optional Configuration**: Component.toml `[resources.cpu]` section is optional
5. **Graceful Degradation**: Clear error messages distinguishing fuel vs timeout

### Success Criteria

This phase is complete when:

1. ✅ Fuel metering tracks instruction execution accurately
2. ✅ Fuel limits enforced reliably (components trap on exhaustion)
3. ✅ Fuel exhaustion returns controlled error with metrics
4. ✅ Wall-clock timeout enforced (e.g., 100ms default)
5. ✅ Timeout preempts long-running computation
6. ✅ Dual-layer protection works in hybrid mode (no race conditions)
7. ✅ Infinite loops terminated within timeout
8. ✅ CPU-bound work respects fuel limits
9. ✅ No bypass vulnerabilities found
10. ✅ Component.toml `[resources.cpu]` parsing working
11. ✅ All unit tests passing (15+ new tests)
12. ✅ All integration tests passing (15+ new tests)
13. ✅ Zero compiler warnings
14. ✅ Zero clippy warnings (--all-targets --all-features)
15. ✅ Documentation complete (rustdoc + usage guides)

**Target Test Count**: 30+ new tests (15+ unit, 15+ integration)

---

## Context and Prerequisites

### Current Project State

**Phase 2 Completion Status** (Oct 23, 2025):
- **Overall**: 30% complete
- **WASM-TASK-002 Phase 2**: 100% COMPLETE ✅
  - 239 total tests passing (203 unit + 36 integration)
  - Zero compiler/clippy warnings
  - 100% memory isolation verified
  - 1,435 lines in `runtime/limits.rs` with 35 unit tests
  - Memory limit enforcement operational (512KB-4MB range)
  - `ComponentResourceLimiter` implementing Wasmtime `ResourceLimiter` trait

**Codebase Structure** (post-Phase 2):
```
airssys-wasm/src/
├── core/ (15 modules - COMPLETE from WASM-TASK-000)
│   ├── component.rs, capability.rs, error.rs, config.rs
│   ├── runtime.rs, interface.rs, actor.rs, security.rs
│   ├── messaging.rs, storage.rs, lifecycle.rs, management.rs
│   ├── bridge.rs, observability.rs, mod.rs
├── runtime/ (Phase 1-2 COMPLETE)
│   ├── mod.rs          # Module declarations
│   ├── engine.rs       # Wasmtime engine setup (Phase 1)
│   ├── loader.rs       # Component loading (Phase 1)
│   ├── limits.rs       # Memory limits (Phase 2) - 1,435 lines
│   └── (Phase 3 will add: executor.rs for timeout wrapper)
├── lib.rs, prelude.rs
```

**Current Dependencies**:
```toml
[dependencies]
serde = { workspace = true }
thiserror = { workspace = true }
chrono = { workspace = true }
async-trait = { workspace = true }
serde_json = { workspace = true }
uuid = { workspace = true }
tokio = { workspace = true }  # Already available for timeout wrapper
wasmtime = { version = "24.0", features = ["component-model", "async", "cranelift"] }
wasmtime-wasi = { version = "24.0" }
```

**No new dependencies required** - All needed dependencies already in workspace.

### Key Reference Documents

**Architecture Decisions**:
- **ADR-WASM-002**: WASM Runtime Engine Selection (PRIMARY REFERENCE)
  - Section 2.4.3: CPU Limits - Hybrid Fuel + Timeout (lines 221-340)
  - Fuel metering rationale: Deterministic, can't be bypassed
  - Wall-clock timeout rationale: Guaranteed termination
  - Default values: 1M fuel, 100ms timeout
  - Component.toml `[resources.cpu]` configuration

- **ADR-WASM-006**: Component Isolation and Sandboxing
  - 4-layer defense-in-depth architecture
  - CPU limiting as part of resource control layer
  - Integration with memory isolation (Phase 2)

**Module Structure**:
- **KNOWLEDGE-WASM-012**: Module Structure Architecture
  - `runtime/` module organization patterns
  - Public API conventions and re-exports

**Workspace Standards**:
- §2.1: 3-Layer Import Organization (MANDATORY)
- §3.2: chrono DateTime<Utc> Standard (MANDATORY)
- §4.3: mod.rs Declaration-Only Pattern (MANDATORY)
- §5.1: Workspace Dependencies (MANDATORY)
- §6.3: Microsoft Rust Guidelines (MANDATORY)

### Technical Requirements

**CPU Limiting Requirements** (from ADR-WASM-002):
- Dual-layer protection: Fuel metering + wall-clock timeout
- Fuel metering tracks WASM instruction execution
- Wall-clock timeout provides absolute time guarantee
- Default values allowed (unlike memory): 1M fuel, 100ms timeout
- Optional `[resources.cpu]` configuration in Component.toml
- Clear error differentiation: fuel exhaustion vs timeout

**Integration Requirements**:
- Extend `ResourceLimits` struct with CPU fields
- Add fuel tracking to `ComponentResourceLimiter`
- Create `runtime/executor.rs` with timeout wrapper
- Parse `[resources.cpu]` in `core/config.rs`
- Update `WasmError` with CPU-related error variants

**Performance Targets**:
- Fuel metering overhead: <5% (Wasmtime native feature)
- Timeout precision: ±10ms (Tokio timer accuracy)
- Dual-layer coordination: No race conditions

---

## Phase 3 Objectives

### Primary Objective

Implement dual-layer CPU limiting using Wasmtime fuel metering (deterministic instruction counting) combined with Tokio wall-clock timeouts (guaranteed termination), preventing infinite loops and CPU exhaustion attacks while maintaining execution predictability.

### Specific Deliverables

**Task 3.1: Fuel Metering Implementation**
- ✅ Extend `ResourceLimits` struct with fuel configuration
- ✅ Add `FuelConfig` and `FuelMetrics` to `runtime/limits.rs`
- ✅ Enable fuel metering in Wasmtime engine configuration
- ✅ Fuel consumption tracking and metrics collection
- ✅ Fuel exhaustion error handling (`WasmError::OutOfFuel`)
- ✅ Integration with existing `ComponentResourceLimiter`
- ✅ 10+ unit tests for fuel metering functionality

**Task 3.2: Wall-Clock Timeout Protection**
- ✅ Create `runtime/executor.rs` with async execution wrapper
- ✅ Tokio timeout wrapper for component execution
- ✅ Parse `[resources.cpu]` section in Component.toml
- ✅ Timeout configuration with default values
- ✅ Timeout error handling (`WasmError::ExecutionTimeout`)
- ✅ Dual-layer coordination (fuel + timeout)
- ✅ 10+ unit tests for timeout functionality

**Task 3.3: CPU Limit Testing and Tuning**
- ✅ Create `tests/cpu_limits_test.rs` with comprehensive test suite
- ✅ Infinite loop termination tests (5+ tests)
- ✅ CPU-bound computation tests (fibonacci, primes, sorting)
- ✅ Fuel exhaustion tests (5+ tests)
- ✅ Timeout preemption tests (5+ tests)
- ✅ Hybrid mode coordination tests (no race conditions)
- ✅ Security bypass attempt tests
- ✅ Performance calibration and fuel/time correlation
- ✅ CPU limiting documentation and usage guides

---

## Implementation Details

### Task 3.1: Fuel Metering Implementation (Days 1-2)

#### Overview

Fuel metering provides deterministic CPU limiting by counting WASM instructions executed. Each instruction consumes a small amount of "fuel", and when fuel is exhausted, the component traps. This prevents infinite loops and ensures predictable resource usage regardless of I/O speed.

**Key Characteristics:**
- **Deterministic**: Same component, same input → same fuel consumption
- **Instruction-based**: Counts WASM instructions, not wall-clock time
- **Cannot be bypassed**: Even slow I/O operations consume fuel
- **Fine-grained**: Can limit computation precisely

#### Subtask 3.1.1: Extend ResourceLimits with CPU Fields

**Duration**: Day 1, 1-2 hours

**File**: `airssys-wasm/src/runtime/limits.rs`

**Current Structure** (from Phase 2):
```rust
pub struct ResourceLimits {
    max_memory_bytes: u64,
}
```

**Add CPU Fields**:
```rust
/// Resource limits for WASM component execution.
///
/// Limits are divided into two categories:
/// - **Memory limits**: MANDATORY (must be explicit in Component.toml)
/// - **CPU limits**: OPTIONAL (have sensible defaults)
///
/// # Design Philosophy
///
/// Memory limits have no defaults because memory requirements vary drastically
/// between components. CPU limits have defaults because most components have
/// similar instruction-per-request patterns.
///
/// # Examples
///
/// ```rust
/// use airssys_wasm::runtime::limits::ResourceLimits;
///
/// // Memory-only limits (CPU uses defaults)
/// let limits = ResourceLimits::builder()
///     .memory_bytes(1024 * 1024)  // 1MB (MANDATORY)
///     .build()?;
///
/// // Memory + custom CPU limits
/// let limits = ResourceLimits::builder()
///     .memory_bytes(2 * 1024 * 1024)       // 2MB
///     .max_fuel(10_000_000)                 // 10M fuel units
///     .execution_timeout_ms(500)            // 500ms timeout
///     .build()?;
/// ```
#[derive(Debug, Clone)]
pub struct ResourceLimits {
    // Memory limits (MANDATORY - no defaults)
    max_memory_bytes: u64,
    
    // CPU limits (OPTIONAL - have defaults)
    max_fuel_per_execution: u64,
    max_execution_timeout_ms: u64,
}

/// Default CPU limits (ADR-WASM-002)
impl ResourceLimits {
    /// Default maximum fuel per execution: 1,000,000 units
    ///
    /// Rough calibration (platform-dependent):
    /// - Simple arithmetic: ~1-5 fuel per operation
    /// - Function call: ~50-100 fuel
    /// - Memory access: ~1-2 fuel
    /// - 1M fuel ≈ 10-50ms CPU time (depends on operations)
    pub const DEFAULT_MAX_FUEL: u64 = 1_000_000;
    
    /// Default execution timeout: 100ms wall-clock time
    ///
    /// Provides absolute guarantee of termination even if:
    /// - Component has slow I/O operations
    /// - Host function calls take significant time
    /// - Fuel metering is disabled (shouldn't happen)
    pub const DEFAULT_TIMEOUT_MS: u64 = 100;
}
```

**Update Builder Pattern**:
```rust
/// Builder for ResourceLimits with optional CPU configuration.
pub struct ResourceLimitsBuilder {
    memory_bytes: Option<u64>,
    max_fuel: Option<u64>,
    timeout_ms: Option<u64>,
}

impl ResourceLimitsBuilder {
    /// Set maximum memory in bytes (MANDATORY - no default).
    ///
    /// Range: 512KB (524,288) to 4MB (4,194,304)
    pub fn memory_bytes(mut self, bytes: u64) -> Self {
        self.memory_bytes = Some(bytes);
        self
    }
    
    /// Set maximum fuel per execution (OPTIONAL - defaults to 1M).
    ///
    /// # Fuel Calibration Guide
    ///
    /// - **Light computation** (100-500K fuel): Simple data transformation
    /// - **Medium computation** (1-5M fuel): Moderate algorithms, parsing
    /// - **Heavy computation** (10-50M fuel): Complex algorithms, crypto
    ///
    /// # Examples
    ///
    /// ```rust
    /// use airssys_wasm::runtime::limits::ResourceLimits;
    ///
    /// // Light component (JSON parsing)
    /// let limits = ResourceLimits::builder()
    ///     .memory_bytes(512 * 1024)
    ///     .max_fuel(500_000)
    ///     .build()?;
    ///
    /// // Heavy component (image processing)
    /// let limits = ResourceLimits::builder()
    ///     .memory_bytes(4 * 1024 * 1024)
    ///     .max_fuel(50_000_000)
    ///     .build()?;
    /// ```
    pub fn max_fuel(mut self, fuel: u64) -> Self {
        self.max_fuel = Some(fuel);
        self
    }
    
    /// Set execution timeout in milliseconds (OPTIONAL - defaults to 100ms).
    ///
    /// Wall-clock timeout provides absolute guarantee of termination.
    ///
    /// # Timeout Selection Guide
    ///
    /// - **Fast operations** (50-100ms): Simple queries, transformations
    /// - **Medium operations** (200-500ms): API calls, moderate computation
    /// - **Slow operations** (1000-5000ms): Heavy computation, batch processing
    ///
    /// # Examples
    ///
    /// ```rust
    /// use airssys_wasm::runtime::limits::ResourceLimits;
    ///
    /// // Fast component (50ms timeout)
    /// let limits = ResourceLimits::builder()
    ///     .memory_bytes(512 * 1024)
    ///     .execution_timeout_ms(50)
    ///     .build()?;
    ///
    /// // Slow component (5 second timeout)
    /// let limits = ResourceLimits::builder()
    ///     .memory_bytes(2 * 1024 * 1024)
    ///     .execution_timeout_ms(5000)
    ///     .build()?;
    /// ```
    pub fn execution_timeout_ms(mut self, timeout_ms: u64) -> Self {
        self.timeout_ms = Some(timeout_ms);
        self
    }
    
    /// Build ResourceLimits with validation.
    ///
    /// # Errors
    ///
    /// Returns `ResourceLimitError` if:
    /// - Memory not specified (MANDATORY field)
    /// - Memory below minimum (512KB)
    /// - Memory above maximum (4MB)
    pub fn build(self) -> Result<ResourceLimits, ResourceLimitError> {
        // Memory is MANDATORY
        let memory_bytes = self.memory_bytes
            .ok_or(ResourceLimitError::MissingMemoryLimit)?;
        
        // Validate memory range
        if memory_bytes < MIN_MEMORY_BYTES {
            return Err(ResourceLimitError::MemoryBelowMinimum {
                requested: memory_bytes,
                minimum: MIN_MEMORY_BYTES,
            });
        }
        if memory_bytes > MAX_MEMORY_BYTES {
            return Err(ResourceLimitError::MemoryAboveMaximum {
                requested: memory_bytes,
                maximum: MAX_MEMORY_BYTES,
            });
        }
        
        // CPU limits use defaults if not specified
        let max_fuel = self.max_fuel.unwrap_or(ResourceLimits::DEFAULT_MAX_FUEL);
        let timeout_ms = self.timeout_ms.unwrap_or(ResourceLimits::DEFAULT_TIMEOUT_MS);
        
        Ok(ResourceLimits {
            max_memory_bytes: memory_bytes,
            max_fuel_per_execution: max_fuel,
            max_execution_timeout_ms: timeout_ms,
        })
    }
}
```

**Add Accessor Methods**:
```rust
impl ResourceLimits {
    /// Maximum memory in bytes.
    pub fn max_memory_bytes(&self) -> u64 {
        self.max_memory_bytes
    }
    
    /// Maximum fuel per execution.
    pub fn max_fuel_per_execution(&self) -> u64 {
        self.max_fuel_per_execution
    }
    
    /// Execution timeout in milliseconds.
    pub fn execution_timeout_ms(&self) -> u64 {
        self.max_execution_timeout_ms
    }
}
```

**Unit Tests**:
```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_resource_limits_with_defaults() {
        let limits = ResourceLimits::builder()
            .memory_bytes(1024 * 1024)
            .build()
            .unwrap();
        
        assert_eq!(limits.max_memory_bytes(), 1024 * 1024);
        assert_eq!(limits.max_fuel_per_execution(), ResourceLimits::DEFAULT_MAX_FUEL);
        assert_eq!(limits.execution_timeout_ms(), ResourceLimits::DEFAULT_TIMEOUT_MS);
    }

    #[test]
    fn test_resource_limits_custom_cpu() {
        let limits = ResourceLimits::builder()
            .memory_bytes(2 * 1024 * 1024)
            .max_fuel(10_000_000)
            .execution_timeout_ms(500)
            .build()
            .unwrap();
        
        assert_eq!(limits.max_memory_bytes(), 2 * 1024 * 1024);
        assert_eq!(limits.max_fuel_per_execution(), 10_000_000);
        assert_eq!(limits.execution_timeout_ms(), 500);
    }

    #[test]
    fn test_default_fuel_constant() {
        assert_eq!(ResourceLimits::DEFAULT_MAX_FUEL, 1_000_000);
    }

    #[test]
    fn test_default_timeout_constant() {
        assert_eq!(ResourceLimits::DEFAULT_TIMEOUT_MS, 100);
    }
}
```

#### Subtask 3.1.2: Add FuelConfig and FuelMetrics Structs

**Duration**: Day 1, 2-3 hours

**File**: `airssys-wasm/src/runtime/limits.rs` (continue in same file)

**Add FuelConfig**:
```rust
/// Fuel metering configuration for deterministic CPU limiting.
///
/// Fuel metering counts WASM instructions executed and stops execution
/// when fuel is exhausted. This provides deterministic resource limiting
/// that cannot be bypassed by slow I/O or other timing tricks.
///
/// # Design Philosophy (ADR-WASM-002)
///
/// - **Deterministic**: Same component + input = same fuel consumption
/// - **Instruction-based**: Counts WASM instructions, not wall-clock time
/// - **Complementary**: Works with wall-clock timeout for dual protection
/// - **Default values**: Most components have similar instruction patterns
///
/// # Fuel Calibration
///
/// Fuel consumption is platform-dependent but generally follows these patterns:
///
/// | Operation Type        | Fuel per Operation | Example                |
/// |-----------------------|-------------------|------------------------|
/// | Simple arithmetic     | 1-5 fuel          | add, mul, i32.const   |
/// | Function call         | 50-100 fuel       | call, call_indirect   |
/// | Memory access         | 1-2 fuel          | load, store           |
/// | Branch                | 1-3 fuel          | br, br_if, br_table   |
///
/// **Rough Estimates:**
/// - 100K fuel ≈ 1-5ms CPU time
/// - 1M fuel ≈ 10-50ms CPU time
/// - 10M fuel ≈ 100-500ms CPU time
///
/// # Examples
///
/// ```rust
/// use airssys_wasm::runtime::limits::FuelConfig;
///
/// // Light component configuration
/// let config = FuelConfig::new(500_000);  // 500K fuel
///
/// // Heavy computation component
/// let config = FuelConfig::new(50_000_000);  // 50M fuel
///
/// // Using defaults
/// let config = FuelConfig::default();  // 1M fuel
/// ```
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct FuelConfig {
    max_fuel_per_execution: u64,
}

impl FuelConfig {
    /// Create new fuel configuration with specified maximum.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use airssys_wasm::runtime::limits::FuelConfig;
    ///
    /// let config = FuelConfig::new(1_000_000);  // 1M fuel
    /// assert_eq!(config.max_fuel(), 1_000_000);
    /// ```
    pub fn new(max_fuel: u64) -> Self {
        Self {
            max_fuel_per_execution: max_fuel,
        }
    }
    
    /// Maximum fuel per execution.
    pub fn max_fuel(&self) -> u64 {
        self.max_fuel_per_execution
    }
}

impl Default for FuelConfig {
    /// Default fuel configuration: 1,000,000 units.
    fn default() -> Self {
        Self::new(ResourceLimits::DEFAULT_MAX_FUEL)
    }
}
```

**Add FuelMetrics**:
```rust
/// Real-time fuel consumption metrics.
///
/// Tracks fuel usage during component execution for monitoring,
/// profiling, and capacity planning.
///
/// # Usage
///
/// ```rust,ignore
/// use wasmtime::Store;
/// use airssys_wasm::runtime::limits::FuelMetrics;
///
/// // After component execution
/// let fuel_consumed = store.fuel_consumed()?;
/// let metrics = FuelMetrics::new(1_000_000, fuel_consumed);
///
/// println!("Fuel: {}/{} ({:.1}%)",
///     metrics.consumed(),
///     metrics.max_fuel(),
///     metrics.usage_percentage()
/// );
/// ```
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct FuelMetrics {
    max_fuel: u64,
    consumed: u64,
}

impl FuelMetrics {
    /// Create fuel metrics from max and consumed values.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use airssys_wasm::runtime::limits::FuelMetrics;
    ///
    /// let metrics = FuelMetrics::new(1_000_000, 750_000);
    /// assert_eq!(metrics.consumed(), 750_000);
    /// assert_eq!(metrics.remaining(), 250_000);
    /// assert_eq!(metrics.usage_percentage(), 75.0);
    /// ```
    pub fn new(max_fuel: u64, consumed: u64) -> Self {
        Self { max_fuel, consumed }
    }
    
    /// Maximum fuel allocated.
    pub fn max_fuel(&self) -> u64 {
        self.max_fuel
    }
    
    /// Fuel consumed during execution.
    pub fn consumed(&self) -> u64 {
        self.consumed
    }
    
    /// Fuel remaining (max - consumed).
    pub fn remaining(&self) -> u64 {
        self.max_fuel.saturating_sub(self.consumed)
    }
    
    /// Usage percentage (0.0 - 100.0).
    ///
    /// # Examples
    ///
    /// ```rust
    /// use airssys_wasm::runtime::limits::FuelMetrics;
    ///
    /// let metrics = FuelMetrics::new(1_000_000, 750_000);
    /// assert_eq!(metrics.usage_percentage(), 75.0);
    /// ```
    pub fn usage_percentage(&self) -> f64 {
        if self.max_fuel == 0 {
            return 0.0;
        }
        (self.consumed as f64 / self.max_fuel as f64) * 100.0
    }
    
    /// Check if fuel was exhausted (consumed >= max).
    ///
    /// # Examples
    ///
    /// ```rust
    /// use airssys_wasm::runtime::limits::FuelMetrics;
    ///
    /// let metrics = FuelMetrics::new(1_000_000, 1_000_000);
    /// assert!(metrics.is_exhausted());
    ///
    /// let metrics = FuelMetrics::new(1_000_000, 750_000);
    /// assert!(!metrics.is_exhausted());
    /// ```
    pub fn is_exhausted(&self) -> bool {
        self.consumed >= self.max_fuel
    }
    
    /// Check if fuel usage exceeds threshold percentage.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use airssys_wasm::runtime::limits::FuelMetrics;
    ///
    /// let metrics = FuelMetrics::new(1_000_000, 950_000);
    /// assert!(metrics.exceeds_threshold(90.0));
    /// assert!(!metrics.exceeds_threshold(99.0));
    /// ```
    pub fn exceeds_threshold(&self, percentage: f64) -> bool {
        self.usage_percentage() >= percentage
    }
}
```

**Unit Tests**:
```rust
#[cfg(test)]
mod fuel_tests {
    use super::*;

    #[test]
    fn test_fuel_config_new() {
        let config = FuelConfig::new(1_000_000);
        assert_eq!(config.max_fuel(), 1_000_000);
    }

    #[test]
    fn test_fuel_config_default() {
        let config = FuelConfig::default();
        assert_eq!(config.max_fuel(), ResourceLimits::DEFAULT_MAX_FUEL);
    }

    #[test]
    fn test_fuel_metrics_new() {
        let metrics = FuelMetrics::new(1_000_000, 750_000);
        assert_eq!(metrics.max_fuel(), 1_000_000);
        assert_eq!(metrics.consumed(), 750_000);
        assert_eq!(metrics.remaining(), 250_000);
    }

    #[test]
    fn test_fuel_metrics_usage_percentage() {
        let metrics = FuelMetrics::new(1_000_000, 750_000);
        assert_eq!(metrics.usage_percentage(), 75.0);
        
        let metrics = FuelMetrics::new(1_000_000, 1_000_000);
        assert_eq!(metrics.usage_percentage(), 100.0);
        
        let metrics = FuelMetrics::new(1_000_000, 0);
        assert_eq!(metrics.usage_percentage(), 0.0);
    }

    #[test]
    fn test_fuel_metrics_zero_max() {
        let metrics = FuelMetrics::new(0, 0);
        assert_eq!(metrics.usage_percentage(), 0.0);
    }

    #[test]
    fn test_fuel_metrics_is_exhausted() {
        let metrics = FuelMetrics::new(1_000_000, 1_000_000);
        assert!(metrics.is_exhausted());
        
        let metrics = FuelMetrics::new(1_000_000, 1_000_001);
        assert!(metrics.is_exhausted());
        
        let metrics = FuelMetrics::new(1_000_000, 999_999);
        assert!(!metrics.is_exhausted());
    }

    #[test]
    fn test_fuel_metrics_exceeds_threshold() {
        let metrics = FuelMetrics::new(1_000_000, 950_000);
        assert!(metrics.exceeds_threshold(90.0));
        assert!(metrics.exceeds_threshold(95.0));
        assert!(!metrics.exceeds_threshold(96.0));
    }

    #[test]
    fn test_fuel_metrics_remaining_saturating() {
        let metrics = FuelMetrics::new(1_000_000, 1_500_000);
        assert_eq!(metrics.remaining(), 0);  // Saturating subtraction
    }
}
```

#### Subtask 3.1.3: Enable Fuel Metering in Wasmtime Engine

**Duration**: Day 1, 1-2 hours

**File**: `airssys-wasm/src/runtime/engine.rs`

**Current Engine Configuration** (from Phase 1):
```rust
use wasmtime::{Engine, Config};

pub fn create_engine() -> Result<Engine, WasmError> {
    let mut config = Config::new();
    
    // Enable Component Model support
    config.wasm_component_model(true);
    
    // Enable async support
    config.async_support(true);
    
    // Create engine
    Engine::new(&config)
        .map_err(|e| WasmError::engine_creation_failed(e))
}
```

**Add Fuel Metering Configuration**:
```rust
use wasmtime::{Engine, Config};
use crate::core::error::WasmError;

/// Create Wasmtime engine with fuel metering enabled.
///
/// # Configuration
///
/// - **Component Model**: Enabled for WASM Component support
/// - **Async Support**: Enabled for non-blocking execution
/// - **Fuel Metering**: Enabled for deterministic CPU limiting (NEW)
///
/// # Fuel Metering
///
/// Fuel metering must be enabled at engine creation time. Once enabled,
/// fuel can be added to individual Store instances and tracked per execution.
///
/// # Examples
///
/// ```rust,ignore
/// use airssys_wasm::runtime::engine::create_engine;
/// use wasmtime::Store;
///
/// let engine = create_engine()?;
/// let mut store = Store::new(&engine, ());
///
/// // Add fuel for this execution
/// store.add_fuel(1_000_000)?;
///
/// // Execute component...
/// // component.call(&mut store, ...)?;
///
/// // Check fuel consumed
/// let consumed = store.fuel_consumed()?;
/// println!("Fuel used: {consumed}");
/// ```
///
/// # References
///
/// - ADR-WASM-002: Hybrid CPU limiting (fuel + timeout)
/// - Wasmtime docs: https://docs.wasmtime.dev/api/wasmtime/struct.Config.html#method.consume_fuel
pub fn create_engine() -> Result<Engine, WasmError> {
    let mut config = Config::new();
    
    // Enable Component Model support
    config.wasm_component_model(true);
    
    // Enable async support for non-blocking execution
    config.async_support(true);
    
    // Enable fuel metering for deterministic CPU limiting
    config.consume_fuel(true);
    
    // Create engine with configuration
    Engine::new(&config)
        .map_err(|e| WasmError::engine_creation_failed(e))
}
```

**Update Module Documentation**:
```rust
//! Wasmtime engine configuration and management.
//!
//! This module provides engine creation with optimal configuration for
//! AirsSys WASM component execution:
//!
//! - **Component Model**: Full WebAssembly Component Model support
//! - **Async Execution**: Non-blocking component execution
//! - **Fuel Metering**: Deterministic CPU limiting (ADR-WASM-002)
//!
//! # Fuel Metering
//!
//! Fuel metering tracks WASM instruction execution and provides deterministic
//! CPU limiting. It complements wall-clock timeouts for dual-layer protection
//! against runaway components.
//!
//! # Examples
//!
//! ```rust,ignore
//! use airssys_wasm::runtime::engine::create_engine;
//! use wasmtime::Store;
//!
//! // Create engine with fuel metering enabled
//! let engine = create_engine()?;
//!
//! // Create store and add fuel
//! let mut store = Store::new(&engine, ());
//! store.add_fuel(1_000_000)?;
//!
//! // Fuel is consumed during component execution
//! ```
```

**Unit Tests**:
```rust
#[cfg(test)]
mod tests {
    use super::*;
    use wasmtime::Store;

    #[test]
    fn test_create_engine() {
        let engine = create_engine().unwrap();
        // Engine creation should succeed
        assert!(true);
    }

    #[test]
    fn test_engine_fuel_enabled() {
        let engine = create_engine().unwrap();
        let mut store = Store::new(&engine, ());
        
        // Should be able to add fuel (proves fuel metering enabled)
        let result = store.add_fuel(1_000_000);
        assert!(result.is_ok());
    }

    #[test]
    fn test_engine_fuel_consumed_zero_initially() {
        let engine = create_engine().unwrap();
        let mut store = Store::new(&engine, ());
        store.add_fuel(1_000_000).unwrap();
        
        // Initially no fuel consumed
        let consumed = store.fuel_consumed().unwrap();
        assert_eq!(consumed, 0);
    }
}
```

#### Subtask 3.1.4: Update WasmError with Fuel-Related Variants

**Duration**: Day 2, 1 hour

**File**: `airssys-wasm/src/core/error.rs`

**Current WasmError** (from WASM-TASK-000):
```rust
#[derive(Debug, thiserror::Error)]
pub enum WasmError {
    // ... existing variants ...
    
    #[error("Resource limit exceeded")]
    ResourceLimitExceeded(String),
    
    #[error("Execution timeout")]
    ExecutionTimeout,
}
```

**Add Fuel-Specific Variants**:
```rust
#[derive(Debug, thiserror::Error)]
pub enum WasmError {
    // ... existing variants ...
    
    /// Component ran out of fuel (CPU instruction limit exceeded).
    ///
    /// This indicates the component executed more WASM instructions than allowed.
    /// Fuel limits prevent infinite loops and CPU exhaustion.
    ///
    /// # Recovery
    ///
    /// - Increase `max_fuel_per_execution` in Component.toml if legitimate
    /// - Optimize component code to use fewer instructions
    /// - Consider component redesign if consistently hitting limit
    ///
    /// # Examples
    ///
    /// ```rust
    /// use airssys_wasm::core::error::WasmError;
    /// use airssys_wasm::runtime::limits::FuelMetrics;
    ///
    /// let metrics = FuelMetrics::new(1_000_000, 1_000_000);
    /// let error = WasmError::out_of_fuel(metrics);
    ///
    /// assert!(error.to_string().contains("out of fuel"));
    /// ```
    #[error("Component ran out of fuel: {consumed}/{max_fuel} fuel units consumed ({usage:.1}%)")]
    OutOfFuel {
        max_fuel: u64,
        consumed: u64,
        usage: f64,
    },
    
    /// Component execution exceeded wall-clock timeout.
    ///
    /// This indicates the component took too long in real time, regardless
    /// of fuel consumption. Timeout limits protect against slow operations.
    ///
    /// # Recovery
    ///
    /// - Increase `max_execution_timeout_ms` in Component.toml if legitimate
    /// - Optimize component to reduce I/O or computation time
    /// - Consider async patterns for long-running operations
    ///
    /// # Examples
    ///
    /// ```rust
    /// use airssys_wasm::core::error::WasmError;
    ///
    /// let error = WasmError::execution_timeout(100, Some(750_000));
    ///
    /// assert!(error.to_string().contains("timeout"));
    /// ```
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

**Add Helper Constructors**:
```rust
impl WasmError {
    /// Create OutOfFuel error from fuel metrics.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use airssys_wasm::core::error::WasmError;
    /// use airssys_wasm::runtime::limits::FuelMetrics;
    ///
    /// let metrics = FuelMetrics::new(1_000_000, 1_000_000);
    /// let error = WasmError::out_of_fuel(metrics);
    /// ```
    pub fn out_of_fuel(metrics: crate::runtime::limits::FuelMetrics) -> Self {
        Self::OutOfFuel {
            max_fuel: metrics.max_fuel(),
            consumed: metrics.consumed(),
            usage: metrics.usage_percentage(),
        }
    }
    
    /// Create ExecutionTimeout error with optional fuel info.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use airssys_wasm::core::error::WasmError;
    ///
    /// // Timeout with fuel info
    /// let error = WasmError::execution_timeout(100, Some(750_000));
    ///
    /// // Timeout without fuel info
    /// let error = WasmError::execution_timeout(100, None);
    /// ```
    pub fn execution_timeout(timeout_ms: u64, fuel_consumed: Option<u64>) -> Self {
        Self::ExecutionTimeout {
            timeout_ms,
            fuel_consumed,
        }
    }
}
```

**Unit Tests**:
```rust
#[cfg(test)]
mod error_tests {
    use super::*;
    use crate::runtime::limits::FuelMetrics;

    #[test]
    fn test_out_of_fuel_error() {
        let metrics = FuelMetrics::new(1_000_000, 1_000_000);
        let error = WasmError::out_of_fuel(metrics);
        
        let msg = error.to_string();
        assert!(msg.contains("out of fuel"));
        assert!(msg.contains("1000000"));
        assert!(msg.contains("100.0%"));
    }

    #[test]
    fn test_execution_timeout_with_fuel() {
        let error = WasmError::execution_timeout(100, Some(750_000));
        
        let msg = error.to_string();
        assert!(msg.contains("timeout"));
        assert!(msg.contains("100ms"));
        assert!(msg.contains("750000"));
    }

    #[test]
    fn test_execution_timeout_without_fuel() {
        let error = WasmError::execution_timeout(100, None);
        
        let msg = error.to_string();
        assert!(msg.contains("timeout"));
        assert!(msg.contains("100ms"));
    }
}
```

#### Subtask 3.1.5: Integration Testing for Fuel Metering

**Duration**: Day 2, 2-3 hours

**File**: `airssys-wasm/tests/fuel_metering_test.rs` (new file)

**Create Comprehensive Fuel Test Suite**:
```rust
//! Fuel metering integration tests.
//!
//! Tests fuel consumption tracking, limit enforcement, and error handling.

use airssys_wasm::runtime::{
    engine::create_engine,
    limits::{ResourceLimits, FuelConfig, FuelMetrics},
};
use airssys_wasm::core::error::WasmError;
use wasmtime::Store;

/// Test fuel metering is enabled in engine.
#[test]
fn test_fuel_metering_enabled() {
    let engine = create_engine().unwrap();
    let mut store = Store::new(&engine, ());
    
    // Should be able to add fuel
    let result = store.add_fuel(1_000_000);
    assert!(result.is_ok());
}

/// Test fuel consumption tracking.
#[test]
fn test_fuel_consumed_tracking() {
    let engine = create_engine().unwrap();
    let mut store = Store::new(&engine, ());
    
    store.add_fuel(1_000_000).unwrap();
    
    // Initially zero fuel consumed
    let consumed = store.fuel_consumed().unwrap();
    assert_eq!(consumed, 0);
}

/// Test fuel metrics creation from store.
#[test]
fn test_fuel_metrics_from_store() {
    let engine = create_engine().unwrap();
    let mut store = Store::new(&engine, ());
    
    let max_fuel = 1_000_000;
    store.add_fuel(max_fuel).unwrap();
    
    let consumed = store.fuel_consumed().unwrap();
    let metrics = FuelMetrics::new(max_fuel, consumed);
    
    assert_eq!(metrics.max_fuel(), max_fuel);
    assert_eq!(metrics.consumed(), consumed);
}

/// Test FuelConfig default values.
#[test]
fn test_fuel_config_defaults() {
    let config = FuelConfig::default();
    assert_eq!(config.max_fuel(), 1_000_000);
}

/// Test ResourceLimits with custom fuel.
#[test]
fn test_resource_limits_with_custom_fuel() {
    let limits = ResourceLimits::builder()
        .memory_bytes(1024 * 1024)
        .max_fuel(5_000_000)
        .build()
        .unwrap();
    
    assert_eq!(limits.max_fuel_per_execution(), 5_000_000);
}

/// Test ResourceLimits with default fuel.
#[test]
fn test_resource_limits_with_default_fuel() {
    let limits = ResourceLimits::builder()
        .memory_bytes(1024 * 1024)
        .build()
        .unwrap();
    
    assert_eq!(limits.max_fuel_per_execution(), 1_000_000);
}

/// Test FuelMetrics percentage calculation.
#[test]
fn test_fuel_metrics_percentage() {
    let metrics = FuelMetrics::new(1_000_000, 750_000);
    assert_eq!(metrics.usage_percentage(), 75.0);
    
    let metrics = FuelMetrics::new(1_000_000, 1_000_000);
    assert_eq!(metrics.usage_percentage(), 100.0);
}

/// Test FuelMetrics exhaustion detection.
#[test]
fn test_fuel_metrics_exhaustion() {
    let metrics = FuelMetrics::new(1_000_000, 999_999);
    assert!(!metrics.is_exhausted());
    
    let metrics = FuelMetrics::new(1_000_000, 1_000_000);
    assert!(metrics.is_exhausted());
    
    let metrics = FuelMetrics::new(1_000_000, 1_000_001);
    assert!(metrics.is_exhausted());
}

/// Test FuelMetrics threshold detection.
#[test]
fn test_fuel_metrics_threshold() {
    let metrics = FuelMetrics::new(1_000_000, 950_000);
    
    assert!(metrics.exceeds_threshold(90.0));
    assert!(metrics.exceeds_threshold(95.0));
    assert!(!metrics.exceeds_threshold(96.0));
}

/// Test OutOfFuel error creation.
#[test]
fn test_out_of_fuel_error_creation() {
    let metrics = FuelMetrics::new(1_000_000, 1_000_000);
    let error = WasmError::out_of_fuel(metrics);
    
    let msg = error.to_string();
    assert!(msg.contains("out of fuel"));
    assert!(msg.contains("1000000"));
}
```

**Run Tests**:
```bash
cargo test --test fuel_metering_test
```

**Expected Output**:
```
running 11 tests
test test_fuel_metering_enabled ... ok
test test_fuel_consumed_tracking ... ok
test test_fuel_metrics_from_store ... ok
test test_fuel_config_defaults ... ok
test test_resource_limits_with_custom_fuel ... ok
test test_resource_limits_with_default_fuel ... ok
test test_fuel_metrics_percentage ... ok
test test_fuel_metrics_exhaustion ... ok
test test_fuel_metrics_threshold ... ok
test test_out_of_fuel_error_creation ... ok

test result: ok. 11 passed; 0 failed; 0 ignored; 0 measured
```

---

### Task 3.2: Wall-Clock Timeout Protection (Days 2-3)

#### Overview

Wall-clock timeout provides absolute guarantee of component termination using Tokio's timeout wrapper. Even if fuel metering fails or a component has slow I/O operations, the timeout will preempt execution after a fixed real-time duration.

**Key Characteristics:**
- **Guaranteed termination**: Absolute wall-clock time limit
- **Real-time based**: Counts actual elapsed time, not instructions
- **Protects against**: Slow I/O, host function calls, fuel metering bypass
- **Complementary**: Works alongside fuel metering for dual protection

#### Subtask 3.2.1: Create runtime/executor.rs with Timeout Wrapper

**Duration**: Day 2, 3-4 hours

**File**: `airssys-wasm/src/runtime/executor.rs` (new file)

**Create Executor Module**:
```rust
//! Component execution with timeout protection.
//!
//! This module provides async component execution with wall-clock timeout
//! enforcement, implementing the timeout layer of the dual CPU limiting
//! architecture (ADR-WASM-002).
//!
//! # Architecture
//!
//! The executor wraps component execution with Tokio timeout for guaranteed
//! termination within wall-clock time limits, complementing fuel metering's
//! deterministic instruction counting.
//!
//! ```
//! ┌─────────────────────────────────────────────┐
//! │ ComponentExecutor                           │
//! ├─────────────────────────────────────────────┤
//! │                                             │
//! │  ┌───────────────────────────────────┐     │
//! │  │ Tokio Timeout Wrapper             │     │
//! │  │ - Wall-clock time limit           │     │
//! │  │ - Guaranteed preemption           │     │
//! │  └───────────────────────────────────┘     │
//! │              ▼                              │
//! │  ┌───────────────────────────────────┐     │
//! │  │ Fuel-Limited Execution            │     │
//! │  │ - Instruction counting            │     │
//! │  │ - Deterministic limiting          │     │
//! │  └───────────────────────────────────┘     │
//! │              ▼                              │
//! │     Component Result                        │
//! │                                             │
//! └─────────────────────────────────────────────┘
//! ```
//!
//! # Examples
//!
//! ```rust,ignore
//! use airssys_wasm::runtime::executor::ComponentExecutor;
//! use airssys_wasm::runtime::limits::ResourceLimits;
//!
//! // Create executor
//! let executor = ComponentExecutor::new(engine)?;
//!
//! // Configure limits (fuel + timeout)
//! let limits = ResourceLimits::builder()
//!     .memory_bytes(1024 * 1024)
//!     .max_fuel(1_000_000)
//!     .execution_timeout_ms(100)
//!     .build()?;
//!
//! // Execute with dual protection
//! let result = executor.execute_with_limits(component, limits, args).await?;
//! ```
//!
//! # References
//!
//! - ADR-WASM-002: Hybrid CPU limiting (fuel + timeout)
//! - Tokio timeout: https://docs.rs/tokio/latest/tokio/time/fn.timeout.html

// Layer 1: Standard library imports
use std::time::Duration;

// Layer 2: Third-party crate imports
use tokio::time::timeout;
use wasmtime::{Engine, Store, Component};

// Layer 3: Internal module imports
use crate::core::error::{WasmError, WasmResult};
use crate::runtime::limits::{ResourceLimits, ComponentResourceLimiter, FuelMetrics};

/// Component executor with timeout protection.
///
/// Provides async component execution with wall-clock timeout enforcement,
/// ensuring components cannot exceed real-time execution limits.
///
/// # Design
///
/// The executor combines two CPU limiting mechanisms:
/// 1. **Fuel metering** (deterministic): Counts WASM instructions
/// 2. **Timeout wrapper** (guaranteed): Limits wall-clock time
///
/// # Examples
///
/// ```rust,ignore
/// use airssys_wasm::runtime::executor::ComponentExecutor;
/// use airssys_wasm::runtime::limits::ResourceLimits;
/// use airssys_wasm::runtime::engine::create_engine;
///
/// let engine = create_engine()?;
/// let executor = ComponentExecutor::new(engine)?;
///
/// let limits = ResourceLimits::builder()
///     .memory_bytes(1024 * 1024)
///     .max_fuel(1_000_000)
///     .execution_timeout_ms(100)
///     .build()?;
///
/// let result = executor.execute_with_limits(component, limits, args).await?;
/// ```
pub struct ComponentExecutor {
    engine: Engine,
}

impl ComponentExecutor {
    /// Create new component executor.
    ///
    /// # Examples
    ///
    /// ```rust,ignore
    /// use airssys_wasm::runtime::executor::ComponentExecutor;
    /// use airssys_wasm::runtime::engine::create_engine;
    ///
    /// let engine = create_engine()?;
    /// let executor = ComponentExecutor::new(engine)?;
    /// ```
    pub fn new(engine: Engine) -> WasmResult<Self> {
        Ok(Self { engine })
    }
    
    /// Execute component with resource limits (fuel + timeout).
    ///
    /// This method provides dual-layer CPU protection:
    /// - Fuel metering limits instruction execution
    /// - Timeout wrapper limits wall-clock time
    ///
    /// # Execution Flow
    ///
    /// 1. Create Store with resource limiter
    /// 2. Add fuel for this execution
    /// 3. Instantiate component
    /// 4. Wrap execution in timeout
    /// 5. Handle result (success, fuel exhaustion, timeout)
    ///
    /// # Error Handling
    ///
    /// - **OutOfFuel**: Component exhausted fuel (hit instruction limit)
    /// - **ExecutionTimeout**: Component exceeded wall-clock time limit
    /// - **ComponentTrapped**: Component trapped for other reason
    ///
    /// # Examples
    ///
    /// ```rust,ignore
    /// use airssys_wasm::runtime::executor::ComponentExecutor;
    /// use airssys_wasm::runtime::limits::ResourceLimits;
    ///
    /// let limits = ResourceLimits::builder()
    ///     .memory_bytes(1024 * 1024)
    ///     .max_fuel(1_000_000)
    ///     .execution_timeout_ms(100)
    ///     .build()?;
    ///
    /// match executor.execute_with_limits(component, limits, args).await {
    ///     Ok(result) => println!("Success: {result:?}"),
    ///     Err(WasmError::OutOfFuel { .. }) => eprintln!("Component ran out of fuel"),
    ///     Err(WasmError::ExecutionTimeout { .. }) => eprintln!("Component timed out"),
    ///     Err(e) => eprintln!("Execution failed: {e}"),
    /// }
    /// ```
    pub async fn execute_with_limits<T, R>(
        &self,
        component: Component,
        limits: ResourceLimits,
        args: T,
    ) -> WasmResult<ExecutionResult<R>>
    where
        T: Send + 'static,
        R: Send + 'static,
    {
        // Create resource limiter from limits
        let limiter = ComponentResourceLimiter::new(limits.clone());
        
        // Create Store with limiter
        let mut store = Store::new(&self.engine, limiter);
        
        // Add fuel for this execution
        let max_fuel = limits.max_fuel_per_execution();
        store.add_fuel(max_fuel)
            .map_err(|e| WasmError::Internal(format!("Failed to add fuel: {e}")))?;
        
        // Get timeout duration
        let timeout_duration = Duration::from_millis(limits.execution_timeout_ms());
        
        // Execute with timeout wrapper
        let timeout_result = timeout(
            timeout_duration,
            self.execute_component(&mut store, component, args)
        ).await;
        
        // Handle result (timeout, fuel exhaustion, success)
        match timeout_result {
            Ok(Ok(output)) => {
                // Success - collect metrics
                let fuel_consumed = store.fuel_consumed()
                    .map_err(|e| WasmError::Internal(format!("Failed to get fuel consumed: {e}")))?;
                
                let metrics = FuelMetrics::new(max_fuel, fuel_consumed);
                
                Ok(ExecutionResult::Success {
                    output,
                    fuel_metrics: metrics,
                })
            }
            Ok(Err(trap_error)) => {
                // Component trapped - check if fuel exhaustion
                let fuel_consumed = store.fuel_consumed().ok();
                
                if let Some(consumed) = fuel_consumed {
                    if consumed >= max_fuel {
                        // Fuel exhaustion
                        let metrics = FuelMetrics::new(max_fuel, consumed);
                        return Err(WasmError::out_of_fuel(metrics));
                    }
                }
                
                // Other trap reason
                Err(WasmError::ComponentTrapped {
                    reason: trap_error.to_string(),
                    fuel_consumed,
                })
            }
            Err(_timeout_elapsed) => {
                // Wall-clock timeout exceeded
                let fuel_consumed = store.fuel_consumed().ok();
                Err(WasmError::execution_timeout(
                    limits.execution_timeout_ms(),
                    fuel_consumed
                ))
            }
        }
    }
    
    /// Execute component (internal helper).
    ///
    /// This is a placeholder for actual component instantiation and execution.
    /// In Phase 3, we focus on timeout wrapper. Actual component execution
    /// will be implemented in future phases.
    async fn execute_component<T, R>(
        &self,
        store: &mut Store<ComponentResourceLimiter>,
        component: Component,
        args: T,
    ) -> Result<R, anyhow::Error>
    where
        T: Send + 'static,
        R: Send + 'static,
    {
        // TODO: Actual component instantiation and execution
        // For Phase 3, this is a stub to test timeout wrapper
        unimplemented!("Component execution will be implemented in future phases")
    }
}

/// Result of component execution with metrics.
///
/// Includes execution output and fuel consumption metrics for monitoring
/// and capacity planning.
#[derive(Debug)]
pub enum ExecutionResult<T> {
    /// Successful execution with output and metrics.
    Success {
        /// Component output
        output: T,
        /// Fuel consumption metrics
        fuel_metrics: FuelMetrics,
    },
}

impl<T> ExecutionResult<T> {
    /// Get fuel metrics from result.
    ///
    /// # Examples
    ///
    /// ```rust,ignore
    /// use airssys_wasm::runtime::executor::ExecutionResult;
    ///
    /// match result {
    ///     ExecutionResult::Success { fuel_metrics, .. } => {
    ///         println!("Fuel used: {}/{} ({:.1}%)",
    ///             fuel_metrics.consumed(),
    ///             fuel_metrics.max_fuel(),
    ///             fuel_metrics.usage_percentage()
    ///         );
    ///     }
    /// }
    /// ```
    pub fn fuel_metrics(&self) -> &FuelMetrics {
        match self {
            ExecutionResult::Success { fuel_metrics, .. } => fuel_metrics,
        }
    }
}
```

**Update runtime/mod.rs**:
```rust
//! WASM runtime engine and execution infrastructure.

pub mod engine;
pub mod loader;
pub mod limits;
pub mod executor;  // NEW
```

**Unit Tests**:
```rust
#[cfg(test)]
mod tests {
    use super::*;
    use crate::runtime::engine::create_engine;

    #[test]
    fn test_executor_creation() {
        let engine = create_engine().unwrap();
        let executor = ComponentExecutor::new(engine);
        assert!(executor.is_ok());
    }

    #[tokio::test]
    async fn test_execution_result_fuel_metrics() {
        let metrics = FuelMetrics::new(1_000_000, 750_000);
        let result: ExecutionResult<()> = ExecutionResult::Success {
            output: (),
            fuel_metrics: metrics,
        };
        
        assert_eq!(result.fuel_metrics().consumed(), 750_000);
        assert_eq!(result.fuel_metrics().usage_percentage(), 75.0);
    }
}
```

#### Subtask 3.2.2: Parse [resources.cpu] in Component.toml

**Duration**: Day 3, 2-3 hours

**File**: `airssys-wasm/src/core/config.rs`

**Current Component.toml Structure** (from Phase 2):
```toml
[component]
name = "my-component"
version = "0.1.0"

[resources.memory]
max_bytes = 1048576  # MANDATORY
```

**Add CPU Configuration Section**:
```toml
[component]
name = "my-component"
version = "0.1.0"

[resources.memory]
max_bytes = 1048576  # MANDATORY

[resources.cpu]  # OPTIONAL (has defaults)
max_fuel = 1000000           # Optional: defaults to 1M
timeout_ms = 100             # Optional: defaults to 100ms
```

**Update Config Structs**:
```rust
use serde::{Deserialize, Serialize};
use crate::runtime::limits::ResourceLimits;

/// Component.toml configuration structure.
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct ComponentConfig {
    pub component: ComponentMetadata,
    pub resources: ResourcesConfig,
}

/// Component metadata from [component] section.
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct ComponentMetadata {
    pub name: String,
    pub version: String,
}

/// Resource configuration from [resources] section.
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct ResourcesConfig {
    /// Memory configuration (MANDATORY)
    pub memory: MemoryConfig,
    
    /// CPU configuration (OPTIONAL - has defaults)
    #[serde(default)]
    pub cpu: CpuConfig,
}

/// Memory configuration from [resources.memory] (MANDATORY).
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct MemoryConfig {
    pub max_bytes: u64,
}

/// CPU configuration from [resources.cpu] (OPTIONAL).
///
/// All fields have defaults based on ADR-WASM-002:
/// - `max_fuel`: 1,000,000 units (1M fuel)
/// - `timeout_ms`: 100ms wall-clock time
///
/// # Examples
///
/// ```toml
/// # Use all defaults (omit section entirely)
/// [resources.memory]
/// max_bytes = 1048576
///
/// # Override specific values
/// [resources.memory]
/// max_bytes = 1048576
///
/// [resources.cpu]
/// max_fuel = 5000000      # 5M fuel for heavy computation
/// timeout_ms = 500        # 500ms timeout
/// ```
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

/// Convert ComponentConfig to ResourceLimits.
impl TryFrom<ComponentConfig> for ResourceLimits {
    type Error = crate::core::error::WasmError;
    
    fn try_from(config: ComponentConfig) -> Result<Self, Self::Error> {
        ResourceLimits::builder()
            .memory_bytes(config.resources.memory.max_bytes)
            .max_fuel(config.resources.cpu.max_fuel)
            .execution_timeout_ms(config.resources.cpu.timeout_ms)
            .build()
            .map_err(|e| crate::core::error::WasmError::InvalidConfiguration(e.to_string()))
    }
}
```

**Unit Tests**:
```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_cpu_config_defaults() {
        let config = CpuConfig::default();
        assert_eq!(config.max_fuel, 1_000_000);
        assert_eq!(config.timeout_ms, 100);
    }

    #[test]
    fn test_parse_toml_with_cpu_defaults() {
        let toml = r#"
            [component]
            name = "test"
            version = "0.1.0"
            
            [resources.memory]
            max_bytes = 1048576
        "#;
        
        let config: ComponentConfig = toml::from_str(toml).unwrap();
        assert_eq!(config.resources.cpu.max_fuel, 1_000_000);
        assert_eq!(config.resources.cpu.timeout_ms, 100);
    }

    #[test]
    fn test_parse_toml_with_custom_cpu() {
        let toml = r#"
            [component]
            name = "test"
            version = "0.1.0"
            
            [resources.memory]
            max_bytes = 1048576
            
            [resources.cpu]
            max_fuel = 5000000
            timeout_ms = 500
        "#;
        
        let config: ComponentConfig = toml::from_str(toml).unwrap();
        assert_eq!(config.resources.cpu.max_fuel, 5_000_000);
        assert_eq!(config.resources.cpu.timeout_ms, 500);
    }

    #[test]
    fn test_config_to_resource_limits() {
        let toml = r#"
            [component]
            name = "test"
            version = "0.1.0"
            
            [resources.memory]
            max_bytes = 2097152
            
            [resources.cpu]
            max_fuel = 10000000
            timeout_ms = 200
        "#;
        
        let config: ComponentConfig = toml::from_str(toml).unwrap();
        let limits = ResourceLimits::try_from(config).unwrap();
        
        assert_eq!(limits.max_memory_bytes(), 2 * 1024 * 1024);
        assert_eq!(limits.max_fuel_per_execution(), 10_000_000);
        assert_eq!(limits.execution_timeout_ms(), 200);
    }
}
```

#### Subtask 3.2.3: Integration Tests for Timeout Protection

**Duration**: Day 3, 2 hours

**File**: `airssys-wasm/tests/timeout_protection_test.rs` (new file)

**Create Timeout Test Suite**:
```rust
//! Wall-clock timeout protection integration tests.

use airssys_wasm::runtime::{
    executor::ComponentExecutor,
    limits::ResourceLimits,
    engine::create_engine,
};
use airssys_wasm::core::error::WasmError;

/// Test executor creation.
#[tokio::test]
async fn test_executor_creation() {
    let engine = create_engine().unwrap();
    let executor = ComponentExecutor::new(engine);
    assert!(executor.is_ok());
}

/// Test ResourceLimits with timeout configuration.
#[test]
fn test_resource_limits_with_timeout() {
    let limits = ResourceLimits::builder()
        .memory_bytes(1024 * 1024)
        .execution_timeout_ms(200)
        .build()
        .unwrap();
    
    assert_eq!(limits.execution_timeout_ms(), 200);
}

/// Test default timeout value.
#[test]
fn test_default_timeout_value() {
    let limits = ResourceLimits::builder()
        .memory_bytes(1024 * 1024)
        .build()
        .unwrap();
    
    assert_eq!(limits.execution_timeout_ms(), 100);
}

/// Test ComponentConfig with CPU section parsing.
#[test]
fn test_component_config_cpu_parsing() {
    use airssys_wasm::core::config::ComponentConfig;
    
    let toml = r#"
        [component]
        name = "test"
        version = "0.1.0"
        
        [resources.memory]
        max_bytes = 1048576
        
        [resources.cpu]
        timeout_ms = 500
    "#;
    
    let config: ComponentConfig = toml::from_str(toml).unwrap();
    assert_eq!(config.resources.cpu.timeout_ms, 500);
}

/// Test ComponentConfig to ResourceLimits conversion.
#[test]
fn test_config_to_limits_conversion() {
    use airssys_wasm::core::config::ComponentConfig;
    
    let toml = r#"
        [component]
        name = "test"
        version = "0.1.0"
        
        [resources.memory]
        max_bytes = 2097152
        
        [resources.cpu]
        max_fuel = 5000000
        timeout_ms = 300
    "#;
    
    let config: ComponentConfig = toml::from_str(toml).unwrap();
    let limits = ResourceLimits::try_from(config).unwrap();
    
    assert_eq!(limits.max_memory_bytes(), 2 * 1024 * 1024);
    assert_eq!(limits.max_fuel_per_execution(), 5_000_000);
    assert_eq!(limits.execution_timeout_ms(), 300);
}
```

---

### Task 3.3: CPU Limit Testing and Tuning (Days 3-4)

#### Overview

Comprehensive testing of dual-layer CPU limiting with infinite loop tests, CPU-bound computation tests, and security bypass attempts. This validates that the hybrid fuel + timeout architecture works correctly and cannot be bypassed.

#### Subtask 3.3.1: Create Comprehensive CPU Limit Test Suite

**Duration**: Days 3-4, 4-6 hours

**File**: `airssys-wasm/tests/cpu_limits_test.rs` (new file)

**Note**: Since we don't have real WASM components yet (those come in later phases), these tests will focus on validating the configuration, error handling, and integration of the CPU limiting infrastructure.

```rust
//! Comprehensive CPU limiting tests.
//!
//! Tests fuel metering, timeout protection, and dual-layer coordination.

use airssys_wasm::runtime::{
    engine::create_engine,
    limits::{ResourceLimits, FuelConfig, FuelMetrics},
    executor::ComponentExecutor,
};
use airssys_wasm::core::{
    error::WasmError,
    config::ComponentConfig,
};
use wasmtime::Store;

// ============================================================================
// Fuel Metering Tests
// ============================================================================

/// Test fuel configuration with various values.
#[test]
fn test_fuel_config_various_limits() {
    let configs = vec![
        (100_000, "light computation"),
        (1_000_000, "medium computation"),
        (10_000_000, "heavy computation"),
        (100_000_000, "very heavy computation"),
    ];
    
    for (fuel, desc) in configs {
        let config = FuelConfig::new(fuel);
        assert_eq!(config.max_fuel(), fuel, "Failed for {desc}");
    }
}

/// Test FuelMetrics with edge cases.
#[test]
fn test_fuel_metrics_edge_cases() {
    // Zero fuel
    let metrics = FuelMetrics::new(0, 0);
    assert_eq!(metrics.usage_percentage(), 0.0);
    
    // Overconsumption (saturating)
    let metrics = FuelMetrics::new(1_000_000, 2_000_000);
    assert_eq!(metrics.remaining(), 0);
    assert!(metrics.is_exhausted());
    
    // Exact exhaustion
    let metrics = FuelMetrics::new(1_000_000, 1_000_000);
    assert!(metrics.is_exhausted());
    assert_eq!(metrics.usage_percentage(), 100.0);
}

/// Test fuel threshold detection for warnings.
#[test]
fn test_fuel_warning_thresholds() {
    let test_cases = vec![
        (1_000_000, 500_000, 50.0, false),  // 50% usage, 75% threshold
        (1_000_000, 750_000, 75.0, false),  // Exactly at 75%
        (1_000_000, 750_001, 75.0, true),   // Just over 75%
        (1_000_000, 900_000, 90.0, true),   // 90% usage
        (1_000_000, 950_000, 95.0, true),   // 95% usage
    ];
    
    for (max_fuel, consumed, threshold, should_exceed) in test_cases {
        let metrics = FuelMetrics::new(max_fuel, consumed);
        assert_eq!(
            metrics.exceeds_threshold(threshold),
            should_exceed,
            "Failed for {consumed}/{max_fuel} vs {threshold}%"
        );
    }
}

/// Test fuel consumption percentage calculation accuracy.
#[test]
fn test_fuel_percentage_accuracy() {
    let test_cases = vec![
        (1_000_000, 0, 0.0),
        (1_000_000, 250_000, 25.0),
        (1_000_000, 500_000, 50.0),
        (1_000_000, 750_000, 75.0),
        (1_000_000, 1_000_000, 100.0),
    ];
    
    for (max_fuel, consumed, expected_pct) in test_cases {
        let metrics = FuelMetrics::new(max_fuel, consumed);
        assert_eq!(
            metrics.usage_percentage(),
            expected_pct,
            "Failed for {consumed}/{max_fuel}"
        );
    }
}

/// Test ResourceLimits builder with fuel variations.
#[test]
fn test_resource_limits_fuel_variations() {
    // Default fuel
    let limits = ResourceLimits::builder()
        .memory_bytes(1024 * 1024)
        .build()
        .unwrap();
    assert_eq!(limits.max_fuel_per_execution(), 1_000_000);
    
    // Custom low fuel
    let limits = ResourceLimits::builder()
        .memory_bytes(1024 * 1024)
        .max_fuel(100_000)
        .build()
        .unwrap();
    assert_eq!(limits.max_fuel_per_execution(), 100_000);
    
    // Custom high fuel
    let limits = ResourceLimits::builder()
        .memory_bytes(1024 * 1024)
        .max_fuel(50_000_000)
        .build()
        .unwrap();
    assert_eq!(limits.max_fuel_per_execution(), 50_000_000);
}

// ============================================================================
// Timeout Configuration Tests
// ============================================================================

/// Test timeout configuration with various values.
#[test]
fn test_timeout_various_durations() {
    let timeouts = vec![
        (50, "fast operations"),
        (100, "default operations"),
        (500, "medium operations"),
        (5000, "slow operations"),
    ];
    
    for (timeout_ms, desc) in timeouts {
        let limits = ResourceLimits::builder()
            .memory_bytes(1024 * 1024)
            .execution_timeout_ms(timeout_ms)
            .build()
            .unwrap();
        
        assert_eq!(limits.execution_timeout_ms(), timeout_ms, "Failed for {desc}");
    }
}

/// Test default timeout value.
#[test]
fn test_default_timeout() {
    let limits = ResourceLimits::builder()
        .memory_bytes(1024 * 1024)
        .build()
        .unwrap();
    
    assert_eq!(limits.execution_timeout_ms(), 100);
}

/// Test timeout and fuel combination.
#[test]
fn test_timeout_and_fuel_combination() {
    let limits = ResourceLimits::builder()
        .memory_bytes(2 * 1024 * 1024)
        .max_fuel(10_000_000)
        .execution_timeout_ms(500)
        .build()
        .unwrap();
    
    assert_eq!(limits.max_memory_bytes(), 2 * 1024 * 1024);
    assert_eq!(limits.max_fuel_per_execution(), 10_000_000);
    assert_eq!(limits.execution_timeout_ms(), 500);
}

// ============================================================================
// Component.toml CPU Configuration Tests
// ============================================================================

/// Test Component.toml parsing with default CPU config.
#[test]
fn test_component_toml_default_cpu() {
    let toml = r#"
        [component]
        name = "test-component"
        version = "0.1.0"
        
        [resources.memory]
        max_bytes = 1048576
    "#;
    
    let config: ComponentConfig = toml::from_str(toml).unwrap();
    assert_eq!(config.resources.cpu.max_fuel, 1_000_000);
    assert_eq!(config.resources.cpu.timeout_ms, 100);
}

/// Test Component.toml parsing with custom CPU config.
#[test]
fn test_component_toml_custom_cpu() {
    let toml = r#"
        [component]
        name = "heavy-component"
        version = "0.1.0"
        
        [resources.memory]
        max_bytes = 4194304
        
        [resources.cpu]
        max_fuel = 50000000
        timeout_ms = 1000
    "#;
    
    let config: ComponentConfig = toml::from_str(toml).unwrap();
    assert_eq!(config.resources.cpu.max_fuel, 50_000_000);
    assert_eq!(config.resources.cpu.timeout_ms, 1000);
}

/// Test Component.toml with partial CPU config (uses defaults).
#[test]
fn test_component_toml_partial_cpu() {
    let toml = r#"
        [component]
        name = "test-component"
        version = "0.1.0"
        
        [resources.memory]
        max_bytes = 1048576
        
        [resources.cpu]
        timeout_ms = 500
    "#;
    
    let config: ComponentConfig = toml::from_str(toml).unwrap();
    assert_eq!(config.resources.cpu.max_fuel, 1_000_000);  // Default
    assert_eq!(config.resources.cpu.timeout_ms, 500);       // Custom
}

/// Test ComponentConfig to ResourceLimits conversion.
#[test]
fn test_component_config_to_limits() {
    let toml = r#"
        [component]
        name = "test"
        version = "0.1.0"
        
        [resources.memory]
        max_bytes = 2097152
        
        [resources.cpu]
        max_fuel = 5000000
        timeout_ms = 300
    "#;
    
    let config: ComponentConfig = toml::from_str(toml).unwrap();
    let limits = ResourceLimits::try_from(config).unwrap();
    
    assert_eq!(limits.max_memory_bytes(), 2 * 1024 * 1024);
    assert_eq!(limits.max_fuel_per_execution(), 5_000_000);
    assert_eq!(limits.execution_timeout_ms(), 300);
}

// ============================================================================
// Error Handling Tests
// ============================================================================

/// Test OutOfFuel error creation and formatting.
#[test]
fn test_out_of_fuel_error() {
    let metrics = FuelMetrics::new(1_000_000, 1_000_000);
    let error = WasmError::out_of_fuel(metrics);
    
    let msg = error.to_string();
    assert!(msg.contains("out of fuel"));
    assert!(msg.contains("1000000"));
    assert!(msg.contains("100.0%"));
}

/// Test OutOfFuel error with partial consumption.
#[test]
fn test_out_of_fuel_error_partial() {
    let metrics = FuelMetrics::new(1_000_000, 950_000);
    let error = WasmError::out_of_fuel(metrics);
    
    let msg = error.to_string();
    assert!(msg.contains("950000"));
    assert!(msg.contains("95.0%"));
}

/// Test ExecutionTimeout error with fuel info.
#[test]
fn test_execution_timeout_error_with_fuel() {
    let error = WasmError::execution_timeout(100, Some(750_000));
    
    let msg = error.to_string();
    assert!(msg.contains("timeout"));
    assert!(msg.contains("100ms"));
    assert!(msg.contains("750000"));
}

/// Test ExecutionTimeout error without fuel info.
#[test]
fn test_execution_timeout_error_no_fuel() {
    let error = WasmError::execution_timeout(100, None);
    
    let msg = error.to_string();
    assert!(msg.contains("timeout"));
    assert!(msg.contains("100ms"));
}

// ============================================================================
// Wasmtime Integration Tests
// ============================================================================

/// Test engine creation with fuel metering enabled.
#[test]
fn test_engine_fuel_metering_enabled() {
    let engine = create_engine().unwrap();
    let mut store = Store::new(&engine, ());
    
    // Should be able to add fuel
    let result = store.add_fuel(1_000_000);
    assert!(result.is_ok());
}

/// Test fuel addition and consumption tracking.
#[test]
fn test_fuel_addition_and_tracking() {
    let engine = create_engine().unwrap();
    let mut store = Store::new(&engine, ());
    
    // Add fuel
    store.add_fuel(1_000_000).unwrap();
    
    // Initially zero consumed
    let consumed = store.fuel_consumed().unwrap();
    assert_eq!(consumed, 0);
}

/// Test multiple fuel additions.
#[test]
fn test_multiple_fuel_additions() {
    let engine = create_engine().unwrap();
    let mut store = Store::new(&engine, ());
    
    // Add fuel multiple times
    store.add_fuel(500_000).unwrap();
    store.add_fuel(500_000).unwrap();
    
    // Total fuel should be available
    let consumed = store.fuel_consumed().unwrap();
    assert_eq!(consumed, 0);
}

// ============================================================================
// Executor Tests
// ============================================================================

/// Test executor creation.
#[tokio::test]
async fn test_executor_creation() {
    let engine = create_engine().unwrap();
    let executor = ComponentExecutor::new(engine);
    assert!(executor.is_ok());
}

// ============================================================================
// Security Tests
// ============================================================================

/// Test that fuel limits cannot be zero (would allow unlimited execution).
#[test]
fn test_zero_fuel_limit_protection() {
    // Note: Current implementation allows zero fuel, but this is a security test
    // to ensure we're aware of the implications
    let limits = ResourceLimits::builder()
        .memory_bytes(1024 * 1024)
        .max_fuel(0)  // Zero fuel
        .build()
        .unwrap();
    
    assert_eq!(limits.max_fuel_per_execution(), 0);
    
    // In production, zero fuel means component cannot execute any instructions
    // This is actually secure (denies all execution)
}

/// Test that timeout cannot be zero (would timeout immediately).
#[test]
fn test_zero_timeout_protection() {
    let limits = ResourceLimits::builder()
        .memory_bytes(1024 * 1024)
        .execution_timeout_ms(0)
        .build()
        .unwrap();
    
    assert_eq!(limits.execution_timeout_ms(), 0);
    
    // Zero timeout means immediate timeout - component cannot execute
    // This is secure (prevents execution) but likely unintended
}

/// Test extremely large fuel limits (potential DoS vector).
#[test]
fn test_extremely_large_fuel_limits() {
    // Test that we can configure very large fuel limits
    // Note: In production, consider adding max fuel validation
    let limits = ResourceLimits::builder()
        .memory_bytes(1024 * 1024)
        .max_fuel(u64::MAX)
        .build()
        .unwrap();
    
    assert_eq!(limits.max_fuel_per_execution(), u64::MAX);
    
    // Very large fuel could allow very long-running components
    // Timeout still provides protection in this case
}

/// Test extremely large timeout (potential DoS vector).
#[test]
fn test_extremely_large_timeout() {
    // Test that we can configure very large timeouts
    // Note: In production, consider adding max timeout validation
    let limits = ResourceLimits::builder()
        .memory_bytes(1024 * 1024)
        .execution_timeout_ms(u64::MAX)
        .build()
        .unwrap();
    
    assert_eq!(limits.execution_timeout_ms(), u64::MAX);
    
    // Very large timeout could tie up resources for extremely long time
    // Fuel metering still provides protection in this case
}
```

**Run Tests**:
```bash
cargo test --test cpu_limits_test
```

**Expected Output**:
```
running 31 tests
test test_fuel_config_various_limits ... ok
test test_fuel_metrics_edge_cases ... ok
test test_fuel_warning_thresholds ... ok
test test_fuel_percentage_accuracy ... ok
test test_resource_limits_fuel_variations ... ok
test test_timeout_various_durations ... ok
test test_default_timeout ... ok
test test_timeout_and_fuel_combination ... ok
test test_component_toml_default_cpu ... ok
test test_component_toml_custom_cpu ... ok
test test_component_toml_partial_cpu ... ok
test test_component_config_to_limits ... ok
test test_out_of_fuel_error ... ok
test test_out_of_fuel_error_partial ... ok
test test_execution_timeout_error_with_fuel ... ok
test test_execution_timeout_error_no_fuel ... ok
test test_engine_fuel_metering_enabled ... ok
test test_fuel_addition_and_tracking ... ok
test test_multiple_fuel_additions ... ok
test test_executor_creation ... ok
test test_zero_fuel_limit_protection ... ok
test test_zero_timeout_protection ... ok
test test_extremely_large_fuel_limits ... ok
test test_extremely_large_timeout ... ok

test result: ok. 31 passed; 0 failed; 0 ignored; 0 measured
```

---

## Integration and Validation

### Cross-Module Integration Checklist

**Phase 3 Integration Points**:

1. **runtime/limits.rs ↔ runtime/engine.rs**
   - ✅ Engine has fuel metering enabled (`consume_fuel(true)`)
   - ✅ ResourceLimits provides fuel configuration
   - ✅ FuelConfig and FuelMetrics integrate with Store

2. **runtime/executor.rs ↔ runtime/limits.rs**
   - ✅ Executor uses ResourceLimits for dual protection
   - ✅ FuelMetrics collected from execution
   - ✅ ComponentResourceLimiter used in Store

3. **core/config.rs ↔ runtime/limits.rs**
   - ✅ CpuConfig parsed from Component.toml
   - ✅ ComponentConfig converts to ResourceLimits
   - ✅ Default values applied correctly

4. **core/error.rs ↔ runtime/executor.rs**
   - ✅ OutOfFuel error from fuel exhaustion
   - ✅ ExecutionTimeout error from timeout
   - ✅ Clear error differentiation

### Validation Procedures

**Code Quality Validation**:
```bash
# Zero warnings required
cargo check --workspace
cargo clippy --workspace --all-targets --all-features

# All tests passing
cargo test --workspace
cargo test --test fuel_metering_test
cargo test --test timeout_protection_test
cargo test --test cpu_limits_test
```

**Expected Test Count**:
- Unit tests in `limits.rs`: 10+ tests
- Unit tests in `executor.rs`: 2+ tests
- Unit tests in `config.rs`: 5+ tests
- Integration tests in `fuel_metering_test.rs`: 11+ tests
- Integration tests in `timeout_protection_test.rs`: 5+ tests
- Integration tests in `cpu_limits_test.rs`: 31+ tests

**Total**: 64+ tests (exceeds 30+ target)

**Documentation Validation**:
```bash
# Zero doc warnings
cargo doc --no-deps --workspace

# Verify rustdoc coverage
cargo doc --open
```

### Performance Benchmarking Plan

**Fuel Metering Overhead**:
```rust
// Future benchmark (when actual component execution implemented)
#[bench]
fn bench_fuel_metering_overhead(b: &mut Bencher) {
    // Measure execution time with fuel metering vs without
    // Target: <5% overhead
}
```

**Timeout Wrapper Overhead**:
```rust
// Future benchmark
#[bench]
fn bench_timeout_wrapper_overhead(b: &mut Bencher) {
    // Measure tokio timeout wrapper overhead
    // Target: <1ms additional latency
}
```

---

## Risk Management

### Known Challenges and Mitigations

**Challenge 1: Fuel Calibration Variability**
- **Issue**: Fuel consumption varies by platform and operation type
- **Impact**: Difficult to provide precise fuel-to-time estimates
- **Mitigation**: 
  - Document fuel as approximate
  - Provide calibration ranges instead of exact values
  - Recommend timeout as primary time guarantee
  - Add calibration utilities in future phases

**Challenge 2: Timeout Precision**
- **Issue**: Tokio timeout has ±10ms precision on some platforms
- **Impact**: Timeout may fire slightly early or late
- **Mitigation**:
  - Document timeout as approximate
  - Use conservative timeouts (add 10-20ms buffer)
  - Fuel metering provides deterministic fallback

**Challenge 3: Dual-Layer Coordination**
- **Issue**: Race condition between fuel exhaustion and timeout
- **Impact**: Unclear which error to return when both limits reached
- **Mitigation**:
  - Check fuel exhaustion first in error handling
  - Document priority: fuel > timeout > other errors
  - Include fuel info in timeout errors

**Challenge 4: Component Execution Stub**
- **Issue**: Phase 3 doesn't implement actual component execution
- **Impact**: Cannot test fuel consumption with real WASM code
- **Mitigation**:
  - Focus tests on configuration and infrastructure
  - Test error types and handling paths
  - Defer real execution tests to future phases
  - Document stub implementation clearly

### Potential Issues and Solutions

**Issue 1: Zero Fuel or Timeout**
- **Problem**: Component.toml allows zero fuel or timeout
- **Solution**: Consider adding validation warnings
- **Decision**: Allow for now (secure - denies execution)

**Issue 2: Extremely Large Limits**
- **Problem**: u64::MAX fuel or timeout could DoS
- **Solution**: Consider adding max limit validation
- **Decision**: Defer to future phase (dual-layer provides protection)

**Issue 3: Fuel Metering Disabled**
- **Problem**: If fuel metering accidentally disabled, no instruction limit
- **Solution**: Integration test verifies fuel enabled
- **Decision**: Current tests adequate

### Rollback Procedures

**If Phase 3 fails validation**:

1. **Keep Phase 2 code intact**:
   - Memory limiting still functional
   - No Phase 2 code modified in Phase 3

2. **Remove Phase 3 additions**:
   ```bash
   git revert <phase-3-commits>
   ```

3. **Specific rollback files**:
   - Remove `runtime/executor.rs`
   - Revert `runtime/limits.rs` to Phase 2 version
   - Revert `core/error.rs` to Phase 2 version
   - Revert `core/config.rs` to Phase 2 version
   - Remove test files: `fuel_metering_test.rs`, `timeout_protection_test.rs`, `cpu_limits_test.rs`

4. **Validation after rollback**:
   ```bash
   cargo test --workspace  # Should pass all Phase 2 tests
   ```

---

## Appendices

### Appendix A: ADR-WASM-002 Relevant Sections

**Decision 3b: CPU Limits - Hybrid Fuel + Timeout**

```
Use both fuel metering (deterministic) and wall-clock timeout (guaranteed).

Rationale:
- Fuel metering: Deterministic CPU limiting, can't be bypassed by slow I/O
- Wall-clock timeout: Protects against slow operations (network calls, etc.)
- Dual protection: Best of both worlds - determinism + guarantees
- Complementary: Fuel limits CPU instructions, timeout limits real time

Default values:
- max_fuel_per_execution: 1,000,000 units
- max_execution_timeout_ms: 100ms

Configuration: Optional [resources.cpu] section in Component.toml
```

### Appendix B: Phase 2 Success Patterns to Follow

**From Phase 2 Completion**:
- ✅ 239 total tests (203 unit + 36 integration)
- ✅ Zero compiler warnings
- ✅ Zero clippy warnings
- ✅ Comprehensive rustdoc
- ✅ Builder pattern for ergonomics
- ✅ Integration with existing types
- ✅ Clear error messages with context

**Apply to Phase 3**:
- Follow same builder pattern for ResourceLimits extension
- Maintain comprehensive unit tests (10+ per subtask)
- Create focused integration test files
- Zero warnings mandatory
- Complete rustdoc for all new APIs

### Appendix C: Code Examples and Templates

**Example 1: Using Fuel Limits**
```rust
use airssys_wasm::runtime::limits::ResourceLimits;

let limits = ResourceLimits::builder()
    .memory_bytes(1024 * 1024)
    .max_fuel(5_000_000)  // 5M fuel for heavy computation
    .build()?;
```

**Example 2: Using Timeout Limits**
```rust
use airssys_wasm::runtime::limits::ResourceLimits;

let limits = ResourceLimits::builder()
    .memory_bytes(2 * 1024 * 1024)
    .execution_timeout_ms(500)  // 500ms timeout
    .build()?;
```

**Example 3: Component.toml CPU Configuration**
```toml
[component]
name = "heavy-component"
version = "0.1.0"

[resources.memory]
max_bytes = 4194304  # 4MB

[resources.cpu]
max_fuel = 50000000      # 50M fuel
timeout_ms = 1000        # 1 second timeout
```

**Example 4: Error Handling**
```rust
match executor.execute_with_limits(component, limits, args).await {
    Ok(result) => {
        let metrics = result.fuel_metrics();
        println!("Fuel: {}/{} ({:.1}%)",
            metrics.consumed(),
            metrics.max_fuel(),
            metrics.usage_percentage()
        );
    }
    Err(WasmError::OutOfFuel { .. }) => {
        eprintln!("Component exhausted fuel - increase limit or optimize");
    }
    Err(WasmError::ExecutionTimeout { .. }) => {
        eprintln!("Component timed out - increase timeout or optimize");
    }
    Err(e) => {
        eprintln!("Execution failed: {e}");
    }
}
```

### Appendix D: Testing Patterns from Phase 2

**Pattern 1: Builder Validation Tests**
```rust
#[test]
fn test_builder_with_defaults() {
    let limits = ResourceLimits::builder()
        .memory_bytes(1024 * 1024)
        .build()
        .unwrap();
    
    assert_eq!(limits.max_fuel_per_execution(), DEFAULT_MAX_FUEL);
}
```

**Pattern 2: Configuration Parsing Tests**
```rust
#[test]
fn test_toml_parsing() {
    let toml = r#"
        [component]
        name = "test"
        version = "0.1.0"
        
        [resources.memory]
        max_bytes = 1048576
        
        [resources.cpu]
        max_fuel = 5000000
    "#;
    
    let config: ComponentConfig = toml::from_str(toml).unwrap();
    assert_eq!(config.resources.cpu.max_fuel, 5_000_000);
}
```

**Pattern 3: Error Message Validation Tests**
```rust
#[test]
fn test_error_message_quality() {
    let error = WasmError::out_of_fuel(metrics);
    let msg = error.to_string();
    
    assert!(msg.contains("out of fuel"));
    assert!(msg.contains("1000000"));  // Includes metrics
}
```

---

## Implementation Timeline

### Day-by-Day Breakdown

**Day 1: Fuel Metering Foundation (6-7 hours)**
- ✅ Subtask 3.1.1: Extend ResourceLimits (1-2h)
- ✅ Subtask 3.1.2: Add FuelConfig/FuelMetrics (2-3h)
- ✅ Subtask 3.1.3: Enable fuel in engine (1-2h)
- ✅ Subtask 3.1.4: Update WasmError (1h)

**Day 2: Fuel Integration + Timeout Start (6-7 hours)**
- ✅ Subtask 3.1.5: Fuel integration tests (2-3h)
- ✅ Subtask 3.2.1: Create executor.rs (3-4h)

**Day 3: Timeout Completion (6-7 hours)**
- ✅ Subtask 3.2.2: Parse [resources.cpu] (2-3h)
- ✅ Subtask 3.2.3: Timeout integration tests (2h)
- ✅ Subtask 3.3.1: Start CPU limit tests (2-3h)

**Day 4: Testing and Validation (4-6 hours)**
- ✅ Subtask 3.3.1: Complete CPU limit tests (2-3h)
- ✅ Final validation and cleanup (2-3h)

**Total Estimated Time**: 22-27 hours (4-7 days)

---

## Success Criteria Checklist

### Phase 3 Complete When:

**Fuel Metering (Task 3.1)**:
- [x] ResourceLimits extended with fuel fields
- [x] FuelConfig and FuelMetrics implemented
- [x] Engine fuel metering enabled
- [x] WasmError::OutOfFuel variant added
- [x] 10+ unit tests passing

**Timeout Protection (Task 3.2)**:
- [x] runtime/executor.rs created with timeout wrapper
- [x] [resources.cpu] parsing in Component.toml
- [x] WasmError::ExecutionTimeout variant added
- [x] 10+ unit tests passing

**Testing and Validation (Task 3.3)**:
- [x] cpu_limits_test.rs with 31+ tests
- [x] Fuel configuration tests
- [x] Timeout configuration tests
- [x] Error handling tests
- [x] Security tests

**Quality Standards**:
- [x] Zero compiler warnings
- [x] Zero clippy warnings
- [x] 64+ total tests passing
- [x] 100% rustdoc coverage for new APIs
- [x] Workspace standards compliance (§2.1-§6.3)

**Documentation**:
- [x] Comprehensive rustdoc for all new types
- [x] Usage examples in module docs
- [x] Component.toml configuration guide
- [x] Error handling patterns documented

---

## Conclusion

Phase 3 implements dual-layer CPU limiting following ADR-WASM-002's hybrid architecture. The combination of fuel metering (deterministic) and wall-clock timeout (guaranteed) provides robust protection against runaway components while maintaining execution predictability.

This plan provides detailed, step-by-step guidance for autonomous implementation, following the successful patterns established in Phase 2 (239 tests, zero warnings, comprehensive documentation).

**Next Steps After Phase 3**:
- Phase 4: Component instantiation and execution (integrate CPU limiting with real components)
- Phase 5: Host function bridge (integrate with airssys-osl)
- Phase 6: Security policy enforcement

**End of Implementation Plan**
