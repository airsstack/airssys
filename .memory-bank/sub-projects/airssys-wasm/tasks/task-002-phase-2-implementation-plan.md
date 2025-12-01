# WASM-TASK-002 Phase 2: Implementation Plan
## Memory Management and Sandboxing

**Status:** Planning Complete - Ready for Implementation  
**Created:** 2025-10-23  
**Duration:** 5-10 days (Week 2-3)  
**Priority:** Critical Path - Security Foundation Layer

---

## Executive Summary

This document provides a comprehensive, day-by-day implementation plan for **WASM-TASK-002 Phase 2**, covering:

- **Task 2.1**: Linear Memory Limit Enforcement (Days 1-3)
- **Task 2.2**: Component.toml Memory Configuration (Days 4-6)
- **Task 2.3**: Memory Isolation Verification (Days 7-9)

Phase 2 establishes mandatory memory management and 100% memory isolation between components, implementing the critical security boundary of the 4-layer defense-in-depth architecture defined in ADR-WASM-006.

---

## Context and Prerequisites

### Current Project State

**Completion Status**:
- **Overall**: 25% complete (post-Phase 1: will be ~30%)
- **WASM-TASK-000**: 100% complete - Core abstractions implemented
  - 9,283 lines of production code
  - 363 comprehensive tests
  - Zero compiler warnings
  - All 15 core modules complete
- **WASM-TASK-002 Phase 1**: Expected complete before Phase 2 start
  - Wasmtime v24.0+ integrated with Component Model
  - Basic component loading and execution working
  - Error handling foundation established
  - <10ms component instantiation validated

**Codebase Structure** (post-Phase 1):
```
airssys-wasm/src/
├── core/ (15 modules - COMPLETE)
│   ├── component.rs, capability.rs, error.rs, config.rs
│   ├── runtime.rs, interface.rs, actor.rs, security.rs
│   ├── messaging.rs, storage.rs, lifecycle.rs, management.rs
│   ├── bridge.rs, observability.rs, mod.rs
├── runtime/ (Phase 1 - COMPLETE)
│   ├── mod.rs, engine.rs, loader.rs
│   └── (Phase 2 will add: limits.rs, instance.rs)
├── lib.rs, prelude.rs
```

**Current Dependencies** (post-Phase 1):
```toml
[dependencies]
serde = { workspace = true }
thiserror = { workspace = true }
chrono = { workspace = true }
async-trait = { workspace = true }
serde_json = { workspace = true }
uuid = { workspace = true }
tokio = { workspace = true }
wasmtime = { version = "24.0", features = ["component-model", "async", "cranelift"] }
wasmtime-wasi = { version = "24.0" }
```

### Key Reference Documents

**Architecture Decisions**:
- **ADR-WASM-002**: WASM Runtime Engine Selection (PRIMARY REFERENCE - 1,100 lines)
  - Memory limits: MANDATORY in Component.toml (NO defaults - intentional design)
  - Memory range: 512KB (minimum) to 4MB (maximum)
  - Enforcement: Wasmtime `ResourceLimiter` trait implementation
  - Rationale: Forces engineers to think about resource usage
  - Philosophy: Prevent "works on my machine" production surprises

- **ADR-WASM-006**: Component Isolation and Sandboxing (SECURITY REFERENCE - 494 lines)
  - 4-layer defense-in-depth architecture
  - Layer 2 focus: WASM linear memory isolation (this phase)
  - 100% memory isolation requirement (MANDATORY security boundary)
  - No shared memory between components
  - Integration with airssys-rt actor system

**Module Structure**:
- **KNOWLEDGE-WASM-012**: Module Structure Architecture
  - Hybrid block-aligned approach with core abstractions
  - `runtime/` module structure and dependency rules
  - Public API patterns and re-export conventions

**Workspace Standards**:
- §2.1: 3-Layer Import Organization (MANDATORY)
- §3.2: chrono DateTime<Utc> Standard (MANDATORY)
- §4.3: mod.rs Declaration-Only Pattern (MANDATORY)
- §5.1: Workspace Dependencies (MANDATORY)
- §6.1: YAGNI Principles (MANDATORY)
- §6.2: Avoid dyn Patterns (MANDATORY)
- §6.3: Microsoft Rust Guidelines (MANDATORY)
  - M-DESIGN-FOR-AI: Idiomatic APIs with thorough documentation
  - M-DI-HIERARCHY: Concrete types > generics > dyn traits
  - M-ERRORS-CANONICAL-STRUCTS: Structured errors with Backtrace
  - M-MOCKABLE-SYSCALLS: All I/O and system calls mockable

### Technical Requirements

**Memory Management Requirements** (from ADR-WASM-002):
- Memory limits MANDATORY (must be explicit in Component.toml)
- No default values (engineers must declare limits)
- Memory range: 512KB (524,288 bytes) to 4MB (4,194,304 bytes)
- Enforcement: Pre-instantiation validation + runtime enforcement
- Error handling: Graceful OOM with informative error messages

**Security Requirements** (from ADR-WASM-006):
- 100% memory isolation between components (MANDATORY)
- No shared memory access (verified through comprehensive tests)
- Memory boundaries strictly enforced by WASM linear memory
- Component failures contained (no host crash)

**Performance Targets** (from ADR-WASM-002):
- Memory tracking overhead: <5% (measured)
- Component instantiation: <10ms cold start (Phase 1 baseline)
- Execution speed: Near-native (95%+ of native code)

**Integration Requirements**:
- Component.toml parsing integration (resources.memory section)
- Wasmtime ResourceLimiter trait implementation
- Error propagation with existing RuntimeError types
- Foundation for airssys-rt ComponentActor integration (future)

---

## Phase 2 Objectives

### Primary Objective

Implement mandatory memory limit enforcement and verify 100% memory isolation between components, establishing the critical WASM-layer security boundary in the 4-layer defense-in-depth architecture.

### Specific Deliverables

**Task 2.1: Linear Memory Limit Enforcement**
- ✅ `runtime/limits.rs` module with ResourceLimits struct
- ✅ MemoryConfig struct with validation logic (512KB-4MB range)
- ✅ ComponentResourceLimiter implementing Wasmtime ResourceLimiter trait
- ✅ Real-time memory usage monitoring with atomic tracking
- ✅ Graceful OOM error handling and component shutdown
- ✅ Builder pattern for limit configuration

**Task 2.2: Component.toml Memory Configuration**
- ✅ Component.toml parsing with `[resources.memory]` section
- ✅ MANDATORY field validation (rejects missing memory limits)
- ✅ Range validation (512KB-4MB) with clear error messages
- ✅ Integration with ComponentResourceLimiter
- ✅ Memory limit enforcement pipeline end-to-end
- ✅ User-friendly configuration error messages

**Task 2.3: Memory Isolation Verification**
- ✅ Component memory boundary tests (single component limits)
- ✅ Cross-component isolation tests (multi-component independence)
- ✅ Memory leak detection tests (long-running components)
- ✅ Stress testing under high memory usage
- ✅ 100% isolation verification (security validation)
- ✅ OOM attack scenario testing

### Success Criteria

This phase is complete when:
1. ✅ Memory limits enforced at runtime via Wasmtime ResourceLimiter
2. ✅ Components cannot exceed configured limits (verified with tests)
3. ✅ OOM handling graceful with informative error messages
4. ✅ Component.toml validation rejects missing `[resources.memory]`
5. ✅ Memory range validation (512KB-4MB) working correctly
6. ✅ 100% memory isolation between components verified
7. ✅ All unit tests passing (~50 tests total)
8. ✅ All integration tests passing (~30 tests total)
9. ✅ All security tests passing (~15 tests total)
10. ✅ Performance overhead <5% (measured and validated)
11. ✅ Zero compiler warnings
12. ✅ Zero clippy warnings (--all-targets --all-features)
13. ✅ Documentation complete (rustdoc + architecture docs)

---

## Implementation Details

### Task 2.1: Linear Memory Limit Enforcement (Days 1-3)

#### Subtask 2.1.1: Create runtime/limits.rs Module Structure

**Duration**: Day 1, 2-3 hours

**File**: `airssys-wasm/src/runtime/limits.rs`

**Implementation**:
```rust
//! Resource limits and enforcement for WASM components.
//!
//! This module implements memory limit enforcement following ADR-WASM-002 design:
//! - MANDATORY memory limits (no defaults)
//! - 512KB-4MB configurable range
//! - Wasmtime ResourceLimiter trait implementation
//! - Real-time memory usage tracking
//! - Graceful OOM handling
//!
//! # Architecture
//!
//! Memory limits are enforced at two levels:
//! 1. **Pre-instantiation**: Component.toml validation rejects missing limits
//! 2. **Runtime**: Wasmtime ResourceLimiter trait enforcement
//!
//! # Design Philosophy (ADR-WASM-002)
//!
//! NO default memory limits - engineers must explicitly declare limits in
//! Component.toml. This intentional design forces thinking about resource
//! usage and prevents "works on my machine" production surprises.
//!
//! # Examples
//!
//! ```rust,ignore
//! use airssys_wasm::runtime::limits::{ResourceLimits, ComponentResourceLimiter};
//!
//! // Create resource limits (builder pattern)
//! let limits = ResourceLimits::builder()
//!     .memory_bytes(1024 * 1024) // 1MB
//!     .build()?;
//!
//! // Create resource limiter for Wasmtime
//! let limiter = ComponentResourceLimiter::new(limits);
//! ```
//!
//! # Security
//!
//! Memory limits are a critical security boundary (Layer 2 of 4-layer
//! defense-in-depth from ADR-WASM-006). 100% isolation between components
//! is MANDATORY and verified through comprehensive testing.

// §2.1: 3-Layer Import Organization (MANDATORY)

// Layer 1: Standard library imports
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::Arc;
use std::time::Duration;

// Layer 2: Third-party crate imports
use thiserror::Error;
use wasmtime::{ResourceLimiter, Store};

// Layer 3: Internal module imports
use crate::core::error::WasmResult;
```

**Module Declaration Update**:

**File**: `src/runtime/mod.rs`

Add to module declarations:
```rust
pub mod limits;

pub use limits::{ResourceLimits, ComponentResourceLimiter, MemoryConfig};
```

**Verification**:
```bash
cargo check --package airssys-wasm
```

**Tests**: None yet (structure only)

---

#### Subtask 2.1.2: Implement ResourceLimits Struct

**Duration**: Day 1, 2-3 hours

**File**: `src/runtime/limits.rs` (continued)

**Implementation**:
```rust
/// Errors related to resource limit configuration and enforcement.
#[derive(Debug, Clone, Error)]
pub enum ResourceLimitError {
    /// Memory limit is below minimum of 512KB.
    #[error("Memory limit {0} bytes is below minimum of 524288 bytes (512KB)")]
    MemoryTooLow(usize),
    
    /// Memory limit exceeds maximum of 4MB.
    #[error("Memory limit {0} bytes exceeds maximum of 4194304 bytes (4MB)")]
    MemoryTooHigh(usize),
    
    /// Memory limit not specified in Component.toml.
    ///
    /// This is intentional - NO default memory limits per ADR-WASM-002.
    /// Engineers must explicitly declare memory requirements.
    #[error(
        "Memory limit not specified in Component.toml [resources.memory] section.\n\
         Memory limits are MANDATORY (no defaults).\n\
         Add the following to your Component.toml:\n\
         [resources.memory]\n\
         max_bytes = 1048576  # Range: 524288 (512KB) to 4194304 (4MB)"
    )]
    MemoryNotConfigured,
}

/// Resource limits for WASM component execution.
///
/// Encapsulates memory and CPU limits following ADR-WASM-002 decisions:
/// - Memory: MANDATORY, 512KB-4MB range
/// - CPU: Optional timeout (defaults to 30s)
///
/// # Examples
///
/// ```rust,ignore
/// use airssys_wasm::runtime::limits::ResourceLimits;
///
/// // Valid configuration (1MB memory)
/// let limits = ResourceLimits::builder()
///     .memory_bytes(1024 * 1024)
///     .build()?;
///
/// // Custom CPU timeout
/// let limits = ResourceLimits::builder()
///     .memory_bytes(2 * 1024 * 1024)  // 2MB
///     .cpu_timeout(Duration::from_secs(60))
///     .build()?;
/// ```
///
/// # Validation
///
/// The builder validates limits on build():
/// - Memory must be 512KB-4MB
/// - Memory must be explicitly configured (no defaults)
/// - CPU timeout optional (defaults to 30s)
#[derive(Debug, Clone)]
pub struct ResourceLimits {
    memory_bytes: usize,
    cpu_timeout: Duration,
}

impl ResourceLimits {
    /// Minimum memory limit: 512KB.
    pub const MIN_MEMORY: usize = 512 * 1024;
    
    /// Maximum memory limit: 4MB.
    pub const MAX_MEMORY: usize = 4 * 1024 * 1024;
    
    /// Default CPU timeout: 30 seconds.
    const DEFAULT_CPU_TIMEOUT: Duration = Duration::from_secs(30);

    /// Creates a new builder for ResourceLimits.
    pub fn builder() -> ResourceLimitsBuilder {
        ResourceLimitsBuilder::default()
    }

    /// Returns the configured memory limit in bytes.
    pub fn memory_bytes(&self) -> usize {
        self.memory_bytes
    }

    /// Returns the configured CPU timeout.
    pub fn cpu_timeout(&self) -> Duration {
        self.cpu_timeout
    }

    /// Validates resource limits against constraints.
    fn validate(&self) -> Result<(), ResourceLimitError> {
        if self.memory_bytes < Self::MIN_MEMORY {
            return Err(ResourceLimitError::MemoryTooLow(self.memory_bytes));
        }
        if self.memory_bytes > Self::MAX_MEMORY {
            return Err(ResourceLimitError::MemoryTooHigh(self.memory_bytes));
        }
        Ok(())
    }
}

/// Builder for ResourceLimits (§M-DESIGN-FOR-AI: Idiomatic API).
#[derive(Debug, Default)]
pub struct ResourceLimitsBuilder {
    memory_bytes: Option<usize>,
    cpu_timeout: Option<Duration>,
}

impl ResourceLimitsBuilder {
    /// Sets the memory limit in bytes.
    ///
    /// Must be between 512KB (524288) and 4MB (4194304).
    pub fn memory_bytes(mut self, bytes: usize) -> Self {
        self.memory_bytes = Some(bytes);
        self
    }

    /// Sets the CPU timeout duration.
    ///
    /// Defaults to 30 seconds if not specified.
    pub fn cpu_timeout(mut self, timeout: Duration) -> Self {
        self.cpu_timeout = Some(timeout);
        self
    }

    /// Builds ResourceLimits with validation.
    ///
    /// # Errors
    ///
    /// Returns error if:
    /// - Memory not configured (MANDATORY per ADR-WASM-002)
    /// - Memory below 512KB minimum
    /// - Memory above 4MB maximum
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
```

**Tests**:

**File**: `src/runtime/limits.rs` (test module)

```rust
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
        assert_eq!(limits.cpu_timeout(), Duration::from_secs(30));
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

    #[test]
    fn test_memory_at_minimum() {
        let limits = ResourceLimits::builder()
            .memory_bytes(ResourceLimits::MIN_MEMORY)
            .build()
            .unwrap();
        assert_eq!(limits.memory_bytes(), 512 * 1024);
    }

    #[test]
    fn test_memory_at_maximum() {
        let limits = ResourceLimits::builder()
            .memory_bytes(ResourceLimits::MAX_MEMORY)
            .build()
            .unwrap();
        assert_eq!(limits.memory_bytes(), 4 * 1024 * 1024);
    }

    #[test]
    fn test_custom_cpu_timeout() {
        let limits = ResourceLimits::builder()
            .memory_bytes(1024 * 1024)
            .cpu_timeout(Duration::from_secs(60))
            .build()
            .unwrap();
        assert_eq!(limits.cpu_timeout(), Duration::from_secs(60));
    }

    #[test]
    fn test_default_cpu_timeout() {
        let limits = ResourceLimits::builder()
            .memory_bytes(1024 * 1024)
            .build()
            .unwrap();
        assert_eq!(limits.cpu_timeout(), Duration::from_secs(30));
    }
}
```

**Verification**:
```bash
cargo test --package airssys-wasm limits::tests
cargo clippy --package airssys-wasm
```

**Expected**: 8 tests passing, zero warnings

---

#### Subtask 2.1.3: Implement MemoryConfig Struct

**Duration**: Day 1, 2-3 hours

**File**: `src/runtime/limits.rs` (continued)

**Implementation**:
```rust
use serde::{Deserialize, Serialize};

/// Memory configuration from Component.toml.
///
/// Represents the `[resources.memory]` section in Component.toml.
/// Memory limits are MANDATORY per ADR-WASM-002 design.
///
/// # Component.toml Example
///
/// ```toml
/// [resources.memory]
/// max_bytes = 1048576  # 1MB (range: 512KB-4MB)
/// ```
///
/// # Validation
///
/// MemoryConfig validates limits when converted to ResourceLimits:
/// - max_bytes must be present (no defaults)
/// - max_bytes must be 512KB-4MB range
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MemoryConfig {
    /// Maximum memory in bytes (MANDATORY).
    pub max_bytes: usize,
}

impl MemoryConfig {
    /// Validates and converts to ResourceLimits.
    ///
    /// # Errors
    ///
    /// Returns error if max_bytes is outside 512KB-4MB range.
    pub fn to_resource_limits(&self) -> Result<ResourceLimits, ResourceLimitError> {
        ResourceLimits::builder()
            .memory_bytes(self.max_bytes)
            .build()
    }
}
```

**Tests**:

Add to `src/runtime/limits.rs` test module:

```rust
#[cfg(test)]
mod tests {
    // ... existing tests ...

    #[test]
    fn test_memory_config_valid() {
        let config = MemoryConfig { max_bytes: 1024 * 1024 };
        let limits = config.to_resource_limits().unwrap();
        assert_eq!(limits.memory_bytes(), 1024 * 1024);
    }

    #[test]
    fn test_memory_config_too_low() {
        let config = MemoryConfig { max_bytes: 256 * 1024 };
        let result = config.to_resource_limits();
        assert!(matches!(result, Err(ResourceLimitError::MemoryTooLow(_))));
    }

    #[test]
    fn test_memory_config_too_high() {
        let config = MemoryConfig { max_bytes: 8 * 1024 * 1024 };
        let result = config.to_resource_limits();
        assert!(matches!(result, Err(ResourceLimitError::MemoryTooHigh(_))));
    }

    #[test]
    fn test_memory_config_serialization() {
        let config = MemoryConfig { max_bytes: 2 * 1024 * 1024 };
        let json = serde_json::to_string(&config).unwrap();
        let deserialized: MemoryConfig = serde_json::from_str(&json).unwrap();
        assert_eq!(deserialized.max_bytes, 2 * 1024 * 1024);
    }
}
```

**Verification**:
```bash
cargo test --package airssys-wasm limits::tests::test_memory_config
```

**Expected**: 4 additional tests passing (12 total)

---

#### Subtask 2.1.4: Implement ComponentResourceLimiter

**Duration**: Day 2, 4-6 hours

**File**: `src/runtime/limits.rs` (continued)

**Implementation**:
```rust
/// Wasmtime ResourceLimiter implementation for component memory enforcement.
///
/// Implements Wasmtime's `ResourceLimiter` trait to enforce memory limits
/// at runtime. Memory allocation requests are checked against configured
/// limits, and excessive allocations are denied.
///
/// # Architecture
///
/// - Uses atomic tracking for thread-safe current usage monitoring
/// - Enforces limits on memory growth (memory_growing callback)
/// - Allows table growth (separate from memory limits)
/// - Provides current usage metrics for monitoring
///
/// # Examples
///
/// ```rust,ignore
/// use airssys_wasm::runtime::limits::{ResourceLimits, ComponentResourceLimiter};
/// use wasmtime::Store;
///
/// let limits = ResourceLimits::builder()
///     .memory_bytes(1024 * 1024)
///     .build()?;
///
/// let limiter = ComponentResourceLimiter::new(limits);
/// let mut store = Store::new(&engine, limiter);
/// ```
///
/// # Thread Safety
///
/// Current memory usage is tracked with Arc<AtomicUsize> for safe
/// concurrent access across async boundaries.
pub struct ComponentResourceLimiter {
    limits: ResourceLimits,
    current_memory: Arc<AtomicUsize>,
}

impl ComponentResourceLimiter {
    /// Creates a new ComponentResourceLimiter with specified limits.
    pub fn new(limits: ResourceLimits) -> Self {
        Self {
            limits,
            current_memory: Arc::new(AtomicUsize::new(0)),
        }
    }

    /// Returns current memory usage in bytes.
    pub fn current_memory_usage(&self) -> usize {
        self.current_memory.load(Ordering::SeqCst)
    }

    /// Returns configured memory limit in bytes.
    pub fn memory_limit(&self) -> usize {
        self.limits.memory_bytes()
    }

    /// Returns memory usage as percentage of limit (0.0-100.0).
    pub fn memory_usage_percent(&self) -> f64 {
        let current = self.current_memory_usage() as f64;
        let limit = self.memory_limit() as f64;
        (current / limit) * 100.0
    }
}

impl ResourceLimiter for ComponentResourceLimiter {
    /// Called when WASM linear memory attempts to grow.
    ///
    /// Returns `Ok(true)` if growth is allowed (within limits).
    /// Returns `Ok(false)` if growth would exceed configured limits.
    ///
    /// # Parameters
    ///
    /// - `current`: Current memory size in bytes
    /// - `desired`: Desired memory size in bytes
    /// - `_maximum`: Optional maximum from WASM module (ignored)
    fn memory_growing(
        &mut self,
        current: usize,
        desired: usize,
        _maximum: Option<usize>,
    ) -> anyhow::Result<bool> {
        // Check if new size would exceed our configured limit
        if desired > self.limits.memory_bytes() {
            // Deny allocation - exceeds limit
            return Ok(false);
        }

        // Update current usage tracking
        self.current_memory.store(desired, Ordering::SeqCst);
        
        // Allow allocation
        Ok(true)
    }

    /// Called when WASM table attempts to grow.
    ///
    /// Always returns `Ok(true)` - table growth is separate from memory limits.
    fn table_growing(
        &mut self,
        _current: u32,
        _desired: u32,
        _maximum: Option<u32>,
    ) -> anyhow::Result<bool> {
        // Allow table growth (not part of memory limits)
        Ok(true)
    }
}

impl Clone for ComponentResourceLimiter {
    fn clone(&self) -> Self {
        Self {
            limits: self.limits.clone(),
            current_memory: Arc::clone(&self.current_memory),
        }
    }
}
```

**Tests**:

Add to `src/runtime/limits.rs` test module:

```rust
#[cfg(test)]
mod tests {
    // ... existing tests ...

    #[test]
    fn test_limiter_memory_within_limits() {
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
    fn test_limiter_memory_exceeds_limits() {
        let limits = ResourceLimits::builder()
            .memory_bytes(1024 * 1024)
            .build()
            .unwrap();
        let mut limiter = ComponentResourceLimiter::new(limits);

        let result = limiter.memory_growing(0, 2 * 1024 * 1024, None);
        assert!(!result.unwrap()); // Should deny
    }

    #[test]
    fn test_limiter_memory_at_exact_limit() {
        let limits = ResourceLimits::builder()
            .memory_bytes(1024 * 1024)
            .build()
            .unwrap();
        let mut limiter = ComponentResourceLimiter::new(limits);

        let result = limiter.memory_growing(0, 1024 * 1024, None);
        assert!(result.unwrap()); // Exactly at limit should be allowed
    }

    #[test]
    fn test_limiter_memory_one_byte_over() {
        let limits = ResourceLimits::builder()
            .memory_bytes(1024 * 1024)
            .build()
            .unwrap();
        let mut limiter = ComponentResourceLimiter::new(limits);

        let result = limiter.memory_growing(0, (1024 * 1024) + 1, None);
        assert!(!result.unwrap()); // 1 byte over should be denied
    }

    #[test]
    fn test_limiter_multiple_allocations() {
        let limits = ResourceLimits::builder()
            .memory_bytes(1024 * 1024)
            .build()
            .unwrap();
        let mut limiter = ComponentResourceLimiter::new(limits);

        // First allocation
        assert!(limiter.memory_growing(0, 256 * 1024, None).unwrap());
        assert_eq!(limiter.current_memory_usage(), 256 * 1024);

        // Second allocation (growth)
        assert!(limiter.memory_growing(256 * 1024, 512 * 1024, None).unwrap());
        assert_eq!(limiter.current_memory_usage(), 512 * 1024);

        // Third allocation (exceeds limit)
        assert!(!limiter.memory_growing(512 * 1024, 2 * 1024 * 1024, None).unwrap());
    }

    #[test]
    fn test_limiter_usage_percent() {
        let limits = ResourceLimits::builder()
            .memory_bytes(1024 * 1024)
            .build()
            .unwrap();
        let mut limiter = ComponentResourceLimiter::new(limits);

        limiter.memory_growing(0, 512 * 1024, None).unwrap();
        assert_eq!(limiter.memory_usage_percent(), 50.0);
    }

    #[test]
    fn test_limiter_table_growth_allowed() {
        let limits = ResourceLimits::builder()
            .memory_bytes(1024 * 1024)
            .build()
            .unwrap();
        let mut limiter = ComponentResourceLimiter::new(limits);

        let result = limiter.table_growing(0, 100, None);
        assert!(result.unwrap()); // Table growth always allowed
    }

    #[test]
    fn test_limiter_clone_shares_usage_tracking() {
        let limits = ResourceLimits::builder()
            .memory_bytes(1024 * 1024)
            .build()
            .unwrap();
        let mut limiter1 = ComponentResourceLimiter::new(limits);
        let mut limiter2 = limiter1.clone();

        limiter1.memory_growing(0, 512 * 1024, None).unwrap();
        
        // Clone should see the same usage (shared Arc)
        assert_eq!(limiter2.current_memory_usage(), 512 * 1024);
    }
}
```

**Verification**:
```bash
cargo test --package airssys-wasm limits::tests::test_limiter
```

**Expected**: 9 additional tests passing (21 total)

---

#### Subtask 2.1.5: Memory Usage Monitoring

**Duration**: Day 2-3, 3-4 hours

**File**: `src/runtime/limits.rs` (additions)

**Implementation**:
```rust
use chrono::{DateTime, Utc};

/// Memory usage metrics snapshot.
///
/// Captures memory usage at a specific point in time for monitoring
/// and observability.
#[derive(Debug, Clone)]
pub struct MemoryMetrics {
    /// When metrics were recorded (§3.2: chrono DateTime<Utc>).
    pub recorded_at: DateTime<Utc>,
    
    /// Current memory usage in bytes.
    pub current_bytes: usize,
    
    /// Configured memory limit in bytes.
    pub limit_bytes: usize,
    
    /// Usage as percentage (0.0-100.0).
    pub usage_percent: f64,
}

impl ComponentResourceLimiter {
    /// Captures current memory metrics snapshot.
    pub fn memory_metrics(&self) -> MemoryMetrics {
        let current = self.current_memory_usage();
        let limit = self.memory_limit();
        
        MemoryMetrics {
            recorded_at: Utc::now(),
            current_bytes: current,
            limit_bytes: limit,
            usage_percent: (current as f64 / limit as f64) * 100.0,
        }
    }
}
```

**Tests**:

Add to test module:

```rust
#[cfg(test)]
mod tests {
    // ... existing tests ...

    #[test]
    fn test_memory_metrics_snapshot() {
        let limits = ResourceLimits::builder()
            .memory_bytes(1024 * 1024)
            .build()
            .unwrap();
        let mut limiter = ComponentResourceLimiter::new(limits);
        
        limiter.memory_growing(0, 512 * 1024, None).unwrap();
        
        let metrics = limiter.memory_metrics();
        assert_eq!(metrics.current_bytes, 512 * 1024);
        assert_eq!(metrics.limit_bytes, 1024 * 1024);
        assert_eq!(metrics.usage_percent, 50.0);
    }

    #[test]
    fn test_memory_metrics_timestamp() {
        let limits = ResourceLimits::builder()
            .memory_bytes(1024 * 1024)
            .build()
            .unwrap();
        let limiter = ComponentResourceLimiter::new(limits);
        
        let before = Utc::now();
        let metrics = limiter.memory_metrics();
        let after = Utc::now();
        
        assert!(metrics.recorded_at >= before);
        assert!(metrics.recorded_at <= after);
    }
}
```

**Verification**:
```bash
cargo test --package airssys-wasm limits::tests::test_memory_metrics
```

**Expected**: 2 additional tests passing (23 total)

---

#### Subtask 2.1.6: OOM Error Handling

**Duration**: Day 3, 2-3 hours

**File**: `src/core/error.rs` (additions)

**Implementation**:
```rust
use crate::runtime::limits::MemoryMetrics;

/// Errors that can occur during WASM runtime operations.
#[derive(Debug, Error)]
pub enum WasmError {
    // ... existing variants ...
    
    /// Component exceeded memory limit (Out Of Memory).
    ///
    /// This error indicates a component attempted to allocate more memory
    /// than its configured limit. The component should be shut down gracefully.
    #[error("Component '{component_id}' exceeded memory limit: {current_bytes} bytes used, {limit_bytes} bytes limit")]
    OutOfMemory {
        component_id: String,
        current_bytes: usize,
        limit_bytes: usize,
        metrics: MemoryMetrics,
    },
}
```

**File**: `src/runtime/limits.rs` (additions)

```rust
use crate::core::error::WasmError;

impl ComponentResourceLimiter {
    /// Creates an OutOfMemory error with current metrics.
    pub fn create_oom_error(&self, component_id: String) -> WasmError {
        let metrics = self.memory_metrics();
        
        WasmError::OutOfMemory {
            component_id,
            current_bytes: metrics.current_bytes,
            limit_bytes: metrics.limit_bytes,
            metrics,
        }
    }
}
```

**Tests**:

Add to test module:

```rust
#[cfg(test)]
mod tests {
    // ... existing tests ...

    #[test]
    fn test_oom_error_creation() {
        let limits = ResourceLimits::builder()
            .memory_bytes(1024 * 1024)
            .build()
            .unwrap();
        let mut limiter = ComponentResourceLimiter::new(limits);
        
        limiter.memory_growing(0, 512 * 1024, None).unwrap();
        
        let error = limiter.create_oom_error("test-component".to_string());
        
        match error {
            WasmError::OutOfMemory { component_id, current_bytes, limit_bytes, .. } => {
                assert_eq!(component_id, "test-component");
                assert_eq!(current_bytes, 512 * 1024);
                assert_eq!(limit_bytes, 1024 * 1024);
            }
            _ => panic!("Expected OutOfMemory error"),
        }
    }

    #[test]
    fn test_oom_error_message() {
        let limits = ResourceLimits::builder()
            .memory_bytes(1024 * 1024)
            .build()
            .unwrap();
        let limiter = ComponentResourceLimiter::new(limits);
        
        let error = limiter.create_oom_error("my-component".to_string());
        let error_msg = error.to_string();
        
        assert!(error_msg.contains("my-component"));
        assert!(error_msg.contains("exceeded memory limit"));
    }
}
```

**Verification**:
```bash
cargo test --package airssys-wasm limits::tests::test_oom
cargo clippy --package airssys-wasm
```

**Expected**: 2 additional tests passing (25 total), zero warnings

**Day 1-3 Checkpoint**:
- ✅ runtime/limits.rs module complete (~350 lines)
- ✅ ResourceLimits struct with builder pattern
- ✅ MemoryConfig struct with validation
- ✅ ComponentResourceLimiter trait implementation
- ✅ Memory monitoring with MemoryMetrics
- ✅ OOM error handling
- ✅ 25 unit tests passing
- ✅ Zero warnings

---

### Task 2.2: Component.toml Memory Configuration (Days 4-6)

#### Subtask 2.2.1: Update Component.toml Parsing

**Duration**: Day 4, 3-4 hours

**File**: `src/core/config.rs` (or create if doesn't exist)

**Implementation**:
```rust
//! Component configuration from Component.toml.
//!
//! This module handles parsing and validation of Component.toml files,
//! including MANDATORY memory limit configuration per ADR-WASM-002.

// §2.1: 3-Layer Import Organization
use std::path::Path;

use serde::{Deserialize, Serialize};
use thiserror::Error;

use crate::runtime::limits::{MemoryConfig, ResourceLimits, ResourceLimitError};

/// Errors that can occur during configuration parsing and validation.
#[derive(Debug, Error)]
pub enum ConfigError {
    /// Missing [resources.memory] section in Component.toml.
    ///
    /// Memory limits are MANDATORY per ADR-WASM-002 design.
    #[error(
        "Missing [resources.memory] section in Component.toml\n\
         Memory limits are MANDATORY (no defaults - intentional design).\n\
         \n\
         Add the following to your Component.toml:\n\
         [resources.memory]\n\
         max_bytes = 1048576  # Range: 524288 (512KB) to 4194304 (4MB)\n\
         \n\
         See ADR-WASM-002 for rationale."
    )]
    MissingMemoryConfig,
    
    /// Invalid resource limits configuration.
    #[error("Invalid resource limits: {0}")]
    InvalidResourceLimits(#[from] ResourceLimitError),
    
    /// TOML parsing error.
    #[error("Failed to parse Component.toml: {0}")]
    TomlParseError(#[from] toml::de::Error),
    
    /// File I/O error.
    #[error("Failed to read Component.toml: {0}")]
    IoError(#[from] std::io::Error),
}

/// Component metadata from [component] section.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ComponentMetadata {
    /// Component name.
    pub name: String,
    
    /// Component version.
    pub version: String,
    
    /// Optional description.
    #[serde(default)]
    pub description: Option<String>,
}

/// CPU resource configuration (optional).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CpuConfig {
    /// CPU timeout in seconds (optional, defaults to 30).
    #[serde(default = "default_cpu_timeout")]
    pub timeout_seconds: u64,
}

fn default_cpu_timeout() -> u64 {
    30
}

/// Resources configuration from [resources] section.
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct ResourcesConfig {
    /// Memory configuration (MANDATORY).
    pub memory: Option<MemoryConfig>,
    
    /// CPU configuration (optional).
    #[serde(default)]
    pub cpu: Option<CpuConfig>,
}

/// Complete Component.toml configuration.
///
/// # Example Component.toml
///
/// ```toml
/// [component]
/// name = "my-component"
/// version = "0.1.0"
/// description = "Example component"
///
/// [resources.memory]
/// max_bytes = 1048576  # 1MB (MANDATORY)
///
/// [resources.cpu]
/// timeout_seconds = 60  # Optional
/// ```
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ComponentConfig {
    /// Component metadata.
    pub component: ComponentMetadata,
    
    /// Resource limits configuration.
    #[serde(default)]
    pub resources: ResourcesConfig,
}

impl ComponentConfig {
    /// Loads and parses Component.toml from file path.
    ///
    /// # Errors
    ///
    /// Returns error if:
    /// - File cannot be read
    /// - TOML syntax is invalid
    /// - Validation fails (missing memory config, invalid limits)
    pub fn from_file(path: impl AsRef<Path>) -> Result<Self, ConfigError> {
        let content = std::fs::read_to_string(path)?;
        Self::from_str(&content)
    }

    /// Parses Component.toml from string content.
    pub fn from_str(content: &str) -> Result<Self, ConfigError> {
        let config: ComponentConfig = toml::from_str(content)?;
        config.validate()?;
        Ok(config)
    }

    /// Validates configuration.
    ///
    /// # Errors
    ///
    /// Returns error if:
    /// - [resources.memory] section missing (MANDATORY)
    /// - Memory limits invalid (outside 512KB-4MB range)
    pub fn validate(&self) -> Result<(), ConfigError> {
        // MANDATORY: Memory configuration must be present
        let memory_config = self
            .resources
            .memory
            .as_ref()
            .ok_or(ConfigError::MissingMemoryConfig)?;

        // Validate memory limits
        memory_config
            .to_resource_limits()
            .map_err(ConfigError::InvalidResourceLimits)?;

        Ok(())
    }

    /// Converts to ResourceLimits for runtime enforcement.
    pub fn to_resource_limits(&self) -> Result<ResourceLimits, ConfigError> {
        let memory_config = self
            .resources
            .memory
            .as_ref()
            .ok_or(ConfigError::MissingMemoryConfig)?;

        let mut builder = memory_config.to_resource_limits()?.builder();

        // Add CPU timeout if configured
        if let Some(cpu_config) = &self.resources.cpu {
            builder = builder.cpu_timeout(std::time::Duration::from_secs(cpu_config.timeout_seconds));
        }

        Ok(builder.build()?)
    }
}
```

**Module Declaration**:

**File**: `src/core/mod.rs`

Add (if not already present):
```rust
pub mod config;

pub use config::{ComponentConfig, ComponentMetadata, ResourcesConfig, ConfigError};
```

**Tests**:

**File**: `src/core/config.rs` (test module)

```rust
#[cfg(test)]
mod tests {
    use super::*;

    const VALID_CONFIG: &str = r#"
        [component]
        name = "test-component"
        version = "0.1.0"

        [resources.memory]
        max_bytes = 1048576
    "#;

    const MISSING_MEMORY_CONFIG: &str = r#"
        [component]
        name = "test-component"
        version = "0.1.0"
    "#;

    const MEMORY_TOO_LOW: &str = r#"
        [component]
        name = "test-component"
        version = "0.1.0"

        [resources.memory]
        max_bytes = 262144
    "#;

    #[test]
    fn test_valid_config_parsing() {
        let config = ComponentConfig::from_str(VALID_CONFIG).unwrap();
        assert_eq!(config.component.name, "test-component");
        assert_eq!(config.component.version, "0.1.0");
        assert_eq!(config.resources.memory.as_ref().unwrap().max_bytes, 1048576);
    }

    #[test]
    fn test_missing_memory_config() {
        let result = ComponentConfig::from_str(MISSING_MEMORY_CONFIG);
        assert!(matches!(result, Err(ConfigError::MissingMemoryConfig)));
    }

    #[test]
    fn test_memory_too_low() {
        let result = ComponentConfig::from_str(MEMORY_TOO_LOW);
        assert!(matches!(result, Err(ConfigError::InvalidResourceLimits(_))));
    }

    #[test]
    fn test_to_resource_limits() {
        let config = ComponentConfig::from_str(VALID_CONFIG).unwrap();
        let limits = config.to_resource_limits().unwrap();
        assert_eq!(limits.memory_bytes(), 1048576);
    }

    #[test]
    fn test_config_with_cpu_timeout() {
        let config_str = r#"
            [component]
            name = "test"
            version = "0.1.0"

            [resources.memory]
            max_bytes = 1048576

            [resources.cpu]
            timeout_seconds = 60
        "#;
        
        let config = ComponentConfig::from_str(config_str).unwrap();
        let limits = config.to_resource_limits().unwrap();
        assert_eq!(limits.cpu_timeout(), std::time::Duration::from_secs(60));
    }

    #[test]
    fn test_missing_memory_error_message() {
        let result = ComponentConfig::from_str(MISSING_MEMORY_CONFIG);
        let error_msg = result.unwrap_err().to_string();
        assert!(error_msg.contains("MANDATORY"));
        assert!(error_msg.contains("[resources.memory]"));
        assert!(error_msg.contains("ADR-WASM-002"));
    }
}
```

**Verification**:
```bash
cargo test --package airssys-wasm config::tests
```

**Expected**: 6 tests passing

---

#### Subtask 2.2.2: Validation Logic and Error Messages

**Duration**: Day 4-5, 4-5 hours

**Already implemented in Subtask 2.2.1**

Additional tests for comprehensive error message validation:

```rust
#[cfg(test)]
mod tests {
    // ... existing tests ...

    #[test]
    fn test_memory_too_low_error_message() {
        let result = ComponentConfig::from_str(MEMORY_TOO_LOW);
        let error_msg = result.unwrap_err().to_string();
        assert!(error_msg.contains("below minimum"));
        assert!(error_msg.contains("512KB"));
    }

    #[test]
    fn test_memory_too_high_error_message() {
        let config_str = r#"
            [component]
            name = "test"
            version = "0.1.0"

            [resources.memory]
            max_bytes = 8388608
        "#;
        
        let result = ComponentConfig::from_str(config_str);
        let error_msg = result.unwrap_err().to_string();
        assert!(error_msg.contains("exceeds maximum"));
        assert!(error_msg.contains("4MB"));
    }
}
```

**Verification**:
```bash
cargo test --package airssys-wasm config::tests
```

**Expected**: 8 tests passing total

---

#### Subtask 2.2.3: Add toml Dependency

**Duration**: Day 5, 1 hour

**File**: Root `Cargo.toml`

Add to `[workspace.dependencies]`:
```toml
toml = "0.8"
```

**File**: `airssys-wasm/Cargo.toml`

Add to `[dependencies]`:
```toml
toml = { workspace = true }
```

**Verification**:
```bash
cargo check --package airssys-wasm
cargo test --package airssys-wasm config
```

---

#### Subtask 2.2.4: Integration with ComponentResourceLimiter

**Duration**: Day 5-6, 4-5 hours

**File**: `tests/component_config_integration_test.rs` (NEW)

```rust
//! Integration tests for Component.toml parsing and resource limit enforcement.

use airssys_wasm::core::config::ComponentConfig;
use airssys_wasm::runtime::limits::ComponentResourceLimiter;

#[test]
fn test_config_to_limiter_pipeline() {
    let config_str = r#"
        [component]
        name = "test-component"
        version = "0.1.0"

        [resources.memory]
        max_bytes = 2097152
    "#;

    let config = ComponentConfig::from_str(config_str).unwrap();
    let limits = config.to_resource_limits().unwrap();
    let mut limiter = ComponentResourceLimiter::new(limits);

    // Should allow allocation within limits
    assert!(limiter.memory_growing(0, 1024 * 1024, None).unwrap());

    // Should deny allocation exceeding limits
    assert!(!limiter.memory_growing(0, 3 * 1024 * 1024, None).unwrap());
}

#[test]
fn test_config_validation_rejects_missing_memory() {
    let config_str = r#"
        [component]
        name = "test"
        version = "0.1.0"
    "#;

    let result = ComponentConfig::from_str(config_str);
    assert!(result.is_err());
}

#[test]
fn test_config_validation_rejects_invalid_memory() {
    let config_str = r#"
        [component]
        name = "test"
        version = "0.1.0"

        [resources.memory]
        max_bytes = 256000
    "#;

    let result = ComponentConfig::from_str(config_str);
    assert!(result.is_err());
}
```

**Verification**:
```bash
cargo test --package airssys-wasm --test component_config_integration_test
```

**Expected**: 3 integration tests passing

**Day 4-6 Checkpoint**:
- ✅ Component.toml parsing with [resources.memory] section
- ✅ MANDATORY field validation
- ✅ Range validation (512KB-4MB)
- ✅ Clear error messages with ADR references
- ✅ Integration with ComponentResourceLimiter
- ✅ 8 unit tests + 3 integration tests passing
- ✅ Zero warnings

---

### Task 2.3: Memory Isolation Verification (Days 7-9)

#### Subtask 2.3.1: Component Memory Boundary Tests

**Duration**: Day 7, 3-4 hours

**File**: `tests/memory_limits_test.rs` (NEW)

```rust
//! Memory limit enforcement tests.

use airssys_wasm::runtime::limits::{ResourceLimits, ComponentResourceLimiter};

#[test]
fn test_single_component_respects_limit() {
    let limits = ResourceLimits::builder()
        .memory_bytes(1024 * 1024)
        .build()
        .unwrap();
    let mut limiter = ComponentResourceLimiter::new(limits);

    // Within limit - should succeed
    assert!(limiter.memory_growing(0, 512 * 1024, None).unwrap());
    
    // At limit - should succeed
    assert!(limiter.memory_growing(512 * 1024, 1024 * 1024, None).unwrap());
    
    // Over limit - should fail
    assert!(!limiter.memory_growing(1024 * 1024, 2 * 1024 * 1024, None).unwrap());
}

#[test]
fn test_oom_at_maximum_allocation() {
    let limits = ResourceLimits::builder()
        .memory_bytes(ResourceLimits::MAX_MEMORY)
        .build()
        .unwrap();
    let mut limiter = ComponentResourceLimiter::new(limits);

    // Should allow allocation up to MAX_MEMORY
    assert!(limiter.memory_growing(0, 4 * 1024 * 1024, None).unwrap());
    
    // Should deny any allocation beyond MAX_MEMORY
    assert!(!limiter.memory_growing(0, (4 * 1024 * 1024) + 1, None).unwrap());
}

#[test]
fn test_oom_at_minimum_allocation() {
    let limits = ResourceLimits::builder()
        .memory_bytes(ResourceLimits::MIN_MEMORY)
        .build()
        .unwrap();
    let mut limiter = ComponentResourceLimiter::new(limits);

    // Should allow allocation up to MIN_MEMORY
    assert!(limiter.memory_growing(0, 512 * 1024, None).unwrap());
    
    // Should deny allocation beyond MIN_MEMORY
    assert!(!limiter.memory_growing(0, (512 * 1024) + 1, None).unwrap());
}

#[test]
fn test_gradual_memory_growth() {
    let limits = ResourceLimits::builder()
        .memory_bytes(1024 * 1024)
        .build()
        .unwrap();
    let mut limiter = ComponentResourceLimiter::new(limits);

    // Simulate gradual memory growth
    assert!(limiter.memory_growing(0, 256 * 1024, None).unwrap());
    assert_eq!(limiter.current_memory_usage(), 256 * 1024);
    
    assert!(limiter.memory_growing(256 * 1024, 512 * 1024, None).unwrap());
    assert_eq!(limiter.current_memory_usage(), 512 * 1024);
    
    assert!(limiter.memory_growing(512 * 1024, 768 * 1024, None).unwrap());
    assert_eq!(limiter.current_memory_usage(), 768 * 1024);
    
    // Final growth hits limit
    assert!(limiter.memory_growing(768 * 1024, 1024 * 1024, None).unwrap());
    
    // Cannot grow beyond
    assert!(!limiter.memory_growing(1024 * 1024, 1024 * 1024 + 1, None).unwrap());
}

#[test]
fn test_memory_usage_tracking() {
    let limits = ResourceLimits::builder()
        .memory_bytes(1024 * 1024)
        .build()
        .unwrap();
    let mut limiter = ComponentResourceLimiter::new(limits);

    assert_eq!(limiter.current_memory_usage(), 0);
    
    limiter.memory_growing(0, 512 * 1024, None).unwrap();
    assert_eq!(limiter.current_memory_usage(), 512 * 1024);
    assert_eq!(limiter.memory_usage_percent(), 50.0);
}
```

**Verification**:
```bash
cargo test --package airssys-wasm --test memory_limits_test
```

**Expected**: 5 tests passing

---

#### Subtask 2.3.2: Cross-Component Isolation Tests

**Duration**: Day 7-8, 4-5 hours

**File**: `tests/memory_isolation_test.rs` (NEW)

```rust
//! Memory isolation verification tests.
//!
//! These tests verify 100% memory isolation between components,
//! a MANDATORY security requirement per ADR-WASM-006.

use airssys_wasm::runtime::limits::{ResourceLimits, ComponentResourceLimiter};

#[test]
fn test_two_components_independent_limits() {
    // Component A: 1MB limit
    let limits_a = ResourceLimits::builder()
        .memory_bytes(1024 * 1024)
        .build()
        .unwrap();
    let mut limiter_a = ComponentResourceLimiter::new(limits_a);

    // Component B: 2MB limit
    let limits_b = ResourceLimits::builder()
        .memory_bytes(2 * 1024 * 1024)
        .build()
        .unwrap();
    let mut limiter_b = ComponentResourceLimiter::new(limits_b);

    // A allocates up to its limit
    assert!(limiter_a.memory_growing(0, 1024 * 1024, None).unwrap());
    assert_eq!(limiter_a.current_memory_usage(), 1024 * 1024);

    // B allocates independently
    assert!(limiter_b.memory_growing(0, 1024 * 1024, None).unwrap());
    assert_eq!(limiter_b.current_memory_usage(), 1024 * 1024);

    // A cannot exceed its limit
    assert!(!limiter_a.memory_growing(1024 * 1024, 2 * 1024 * 1024, None).unwrap());

    // B can still allocate up to its limit
    assert!(limiter_b.memory_growing(1024 * 1024, 2 * 1024 * 1024, None).unwrap());
}

#[test]
fn test_component_oom_does_not_affect_other() {
    let limits = ResourceLimits::builder()
        .memory_bytes(1024 * 1024)
        .build()
        .unwrap();

    let mut limiter_a = ComponentResourceLimiter::new(limits.clone());
    let mut limiter_b = ComponentResourceLimiter::new(limits);

    // A hits OOM
    limiter_a.memory_growing(0, 1024 * 1024, None).unwrap();
    assert!(!limiter_a.memory_growing(1024 * 1024, 2 * 1024 * 1024, None).unwrap());

    // B can still allocate normally
    assert!(limiter_b.memory_growing(0, 512 * 1024, None).unwrap());
    assert_eq!(limiter_b.current_memory_usage(), 512 * 1024);
}

#[test]
fn test_multiple_components_concurrent_allocation() {
    let limits = ResourceLimits::builder()
        .memory_bytes(1024 * 1024)
        .build()
        .unwrap();

    let mut limiters: Vec<_> = (0..10)
        .map(|_| ComponentResourceLimiter::new(limits.clone()))
        .collect();

    // All components allocate independently
    for limiter in &mut limiters {
        assert!(limiter.memory_growing(0, 512 * 1024, None).unwrap());
        assert_eq!(limiter.current_memory_usage(), 512 * 1024);
    }

    // Each component's limits are independent
    for limiter in &mut limiters {
        assert!(!limiter.memory_growing(512 * 1024, 2 * 1024 * 1024, None).unwrap());
    }
}

#[test]
fn test_component_usage_isolation() {
    let limits_a = ResourceLimits::builder()
        .memory_bytes(1024 * 1024)
        .build()
        .unwrap();
    let limits_b = ResourceLimits::builder()
        .memory_bytes(2 * 1024 * 1024)
        .build()
        .unwrap();

    let mut limiter_a = ComponentResourceLimiter::new(limits_a);
    let mut limiter_b = ComponentResourceLimiter::new(limits_b);

    // A allocates
    limiter_a.memory_growing(0, 512 * 1024, None).unwrap();

    // B's usage tracking is completely independent
    assert_eq!(limiter_b.current_memory_usage(), 0);

    // B allocates
    limiter_b.memory_growing(0, 1024 * 1024, None).unwrap();

    // A's usage unchanged
    assert_eq!(limiter_a.current_memory_usage(), 512 * 1024);
    assert_eq!(limiter_b.current_memory_usage(), 1024 * 1024);
}
```

**Verification**:
```bash
cargo test --package airssys-wasm --test memory_isolation_test
```

**Expected**: 4 tests passing

---

#### Subtask 2.3.3: Memory Leak Detection Tests

**Duration**: Day 8, 3-4 hours

**File**: `tests/memory_leak_test.rs` (NEW)

```rust
//! Memory leak detection tests.

use airssys_wasm::runtime::limits::{ResourceLimits, ComponentResourceLimiter};

#[test]
fn test_repeated_allocations_stable_usage() {
    let limits = ResourceLimits::builder()
        .memory_bytes(1024 * 1024)
        .build()
        .unwrap();
    let mut limiter = ComponentResourceLimiter::new(limits);

    // Simulate repeated allocations at same size (no growth)
    for _ in 0..1000 {
        limiter.memory_growing(0, 512 * 1024, None).unwrap();
        assert_eq!(limiter.current_memory_usage(), 512 * 1024);
    }
}

#[test]
fn test_allocation_deallocation_cycle() {
    let limits = ResourceLimits::builder()
        .memory_bytes(1024 * 1024)
        .build()
        .unwrap();
    let mut limiter = ComponentResourceLimiter::new(limits);

    // Grow
    limiter.memory_growing(0, 512 * 1024, None).unwrap();
    assert_eq!(limiter.current_memory_usage(), 512 * 1024);

    // "Shrink" (simulated by lower value)
    limiter.memory_growing(512 * 1024, 256 * 1024, None).unwrap();
    assert_eq!(limiter.current_memory_usage(), 256 * 1024);

    // Grow again
    limiter.memory_growing(256 * 1024, 768 * 1024, None).unwrap();
    assert_eq!(limiter.current_memory_usage(), 768 * 1024);
}

#[test]
fn test_long_running_stable_memory() {
    let limits = ResourceLimits::builder()
        .memory_bytes(1024 * 1024)
        .build()
        .unwrap();
    let mut limiter = ComponentResourceLimiter::new(limits);

    // Initial allocation
    limiter.memory_growing(0, 512 * 1024, None).unwrap();
    let initial_usage = limiter.current_memory_usage();

    // Simulate long-running component (no leaks)
    for _ in 0..10000 {
        // Memory stays stable
        assert_eq!(limiter.current_memory_usage(), initial_usage);
    }
}
```

**Verification**:
```bash
cargo test --package airssys-wasm --test memory_leak_test
```

**Expected**: 3 tests passing

---

#### Subtask 2.3.4: Stress Testing

**Duration**: Day 8-9, 4-5 hours

**File**: `tests/memory_stress_test.rs` (NEW)

```rust
//! Memory stress tests under high load.

use airssys_wasm::runtime::limits::{ResourceLimits, ComponentResourceLimiter};

#[test]
fn test_high_frequency_allocations() {
    let limits = ResourceLimits::builder()
        .memory_bytes(1024 * 1024)
        .build()
        .unwrap();
    let mut limiter = ComponentResourceLimiter::new(limits);

    // Rapid allocation attempts
    for i in 0..10000 {
        let size = 256 * 1024 + (i % 10) * 1024;
        limiter.memory_growing(0, size.min(1024 * 1024), None).unwrap();
    }
}

#[test]
fn test_concurrent_components_high_load() {
    let limits = ResourceLimits::builder()
        .memory_bytes(1024 * 1024)
        .build()
        .unwrap();

    // Create many components
    let mut limiters: Vec<_> = (0..100)
        .map(|_| ComponentResourceLimiter::new(limits.clone()))
        .collect();

    // All allocate simultaneously
    for limiter in &mut limiters {
        assert!(limiter.memory_growing(0, 512 * 1024, None).unwrap());
    }

    // Verify each component's isolation under load
    for (i, limiter) in limiters.iter().enumerate() {
        assert_eq!(
            limiter.current_memory_usage(),
            512 * 1024,
            "Component {} has incorrect usage",
            i
        );
    }
}

#[test]
fn test_oom_recovery_stress() {
    let limits = ResourceLimits::builder()
        .memory_bytes(1024 * 1024)
        .build()
        .unwrap();
    let mut limiter = ComponentResourceLimiter::new(limits);

    // Repeatedly hit OOM and recover
    for _ in 0..1000 {
        // Hit OOM
        assert!(!limiter.memory_growing(0, 2 * 1024 * 1024, None).unwrap());
        
        // Allocate within limits (recovery)
        assert!(limiter.memory_growing(0, 512 * 1024, None).unwrap());
    }
}

#[test]
fn test_edge_case_allocations() {
    let limits = ResourceLimits::builder()
        .memory_bytes(1024 * 1024)
        .build()
        .unwrap();
    let mut limiter = ComponentResourceLimiter::new(limits);

    // Zero allocation
    assert!(limiter.memory_growing(0, 0, None).unwrap());
    
    // Single byte
    assert!(limiter.memory_growing(0, 1, None).unwrap());
    
    // Exactly at limit
    assert!(limiter.memory_growing(0, 1024 * 1024, None).unwrap());
    
    // One byte over
    assert!(!limiter.memory_growing(0, 1024 * 1024 + 1, None).unwrap());
}
```

**Verification**:
```bash
cargo test --package airssys-wasm --test memory_stress_test
```

**Expected**: 4 tests passing

---

#### Subtask 2.3.5: 100% Isolation Verification

**Duration**: Day 9, 3-4 hours

**File**: `tests/isolation_security_test.rs` (NEW)

```rust
//! Security-focused isolation verification tests.
//!
//! Verifies 100% memory isolation (MANDATORY per ADR-WASM-006).

use airssys_wasm::runtime::limits::{ResourceLimits, ComponentResourceLimiter};

#[test]
fn test_component_cannot_see_other_memory() {
    let limits = ResourceLimits::builder()
        .memory_bytes(1024 * 1024)
        .build()
        .unwrap();

    let mut limiter_a = ComponentResourceLimiter::new(limits.clone());
    let mut limiter_b = ComponentResourceLimiter::new(limits);

    // A allocates
    limiter_a.memory_growing(0, 512 * 1024, None).unwrap();

    // B sees zero usage (not A's usage)
    assert_eq!(limiter_b.current_memory_usage(), 0);

    // B allocates
    limiter_b.memory_growing(0, 768 * 1024, None).unwrap();

    // A still sees only its own usage
    assert_eq!(limiter_a.current_memory_usage(), 512 * 1024);
    assert_eq!(limiter_b.current_memory_usage(), 768 * 1024);
}

#[test]
fn test_oom_isolation_security() {
    let limits_a = ResourceLimits::builder()
        .memory_bytes(512 * 1024)
        .build()
        .unwrap();
    let limits_b = ResourceLimits::builder()
        .memory_bytes(2 * 1024 * 1024)
        .build()
        .unwrap();

    let mut limiter_a = ComponentResourceLimiter::new(limits_a);
    let mut limiter_b = ComponentResourceLimiter::new(limits_b);

    // A hits OOM with small limit
    assert!(!limiter_a.memory_growing(0, 1024 * 1024, None).unwrap());

    // B can still allocate its full limit (not affected by A's OOM)
    assert!(limiter_b.memory_growing(0, 2 * 1024 * 1024, None).unwrap());
}

#[test]
fn test_limit_independence() {
    let limits_512kb = ResourceLimits::builder()
        .memory_bytes(512 * 1024)
        .build()
        .unwrap();
    let limits_4mb = ResourceLimits::builder()
        .memory_bytes(4 * 1024 * 1024)
        .build()
        .unwrap();

    let mut limiter_small = ComponentResourceLimiter::new(limits_512kb);
    let mut limiter_large = ComponentResourceLimiter::new(limits_4mb);

    // Small component hits limit quickly
    assert!(limiter_small.memory_growing(0, 512 * 1024, None).unwrap());
    assert!(!limiter_small.memory_growing(512 * 1024, 1024 * 1024, None).unwrap());

    // Large component unaffected by small component's limit
    assert!(limiter_large.memory_growing(0, 3 * 1024 * 1024, None).unwrap());
    assert_eq!(limiter_large.current_memory_usage(), 3 * 1024 * 1024);
}

#[test]
fn test_clone_isolation() {
    let limits = ResourceLimits::builder()
        .memory_bytes(1024 * 1024)
        .build()
        .unwrap();

    let mut limiter_original = ComponentResourceLimiter::new(limits);
    let limiter_clone = limiter_original.clone();

    // Original allocates
    limiter_original.memory_growing(0, 512 * 1024, None).unwrap();

    // Clone sees the same usage (shared Arc<AtomicUsize>)
    assert_eq!(limiter_clone.current_memory_usage(), 512 * 1024);
    
    // But separate limiters are independent
    let mut limiter_independent = ComponentResourceLimiter::new(limiter_original.limits.clone());
    assert_eq!(limiter_independent.current_memory_usage(), 0);
}

#[test]
fn test_security_boundary_enforcement() {
    // Verify that components with different limits cannot interfere
    let components: Vec<_> = vec![512, 1024, 2048, 4096]
        .into_iter()
        .map(|kb| {
            let limits = ResourceLimits::builder()
                .memory_bytes(kb * 1024)
                .build()
                .unwrap();
            ComponentResourceLimiter::new(limits)
        })
        .collect();

    // Each component can allocate up to its own limit
    for (i, mut limiter) in components.into_iter().enumerate() {
        let limit_bytes = limiter.memory_limit();
        assert!(
            limiter.memory_growing(0, limit_bytes, None).unwrap(),
            "Component {} failed to allocate within its limit",
            i
        );
        assert_eq!(limiter.current_memory_usage(), limit_bytes);
    }
}
```

**Verification**:
```bash
cargo test --package airssys-wasm --test isolation_security_test
```

**Expected**: 5 tests passing

**Day 7-9 Checkpoint**:
- ✅ Component memory boundary tests (5 tests)
- ✅ Cross-component isolation tests (4 tests)
- ✅ Memory leak detection tests (3 tests)
- ✅ Stress testing (4 tests)
- ✅ 100% isolation verification (5 tests)
- ✅ Total: 21 integration/security tests
- ✅ All tests passing
- ✅ Zero warnings

---

## Testing Strategy

### Test Organization

**Unit Tests** (~50 tests total):
- `src/runtime/limits.rs` test module: 25 tests
  - ResourceLimits configuration: 8 tests
  - MemoryConfig validation: 4 tests
  - ComponentResourceLimiter logic: 9 tests
  - Memory monitoring: 2 tests
  - OOM handling: 2 tests
- `src/core/config.rs` test module: 8 tests
  - Component.toml parsing: 6 tests
  - Error message validation: 2 tests

**Integration Tests** (~30 tests total):
- `tests/component_config_integration_test.rs`: 3 tests
- `tests/memory_limits_test.rs`: 5 tests
- `tests/memory_isolation_test.rs`: 4 tests
- `tests/memory_leak_test.rs`: 3 tests
- `tests/memory_stress_test.rs`: 4 tests
- `tests/isolation_security_test.rs`: 5 tests
- Additional end-to-end tests: 6 tests

**Security Tests** (~15 tests, subset of integration):
- Memory isolation breaches: 5 tests (isolation_security_test.rs)
- OOM attack scenarios: 5 tests (memory_stress_test.rs subset)
- Resource exhaustion: 5 tests (cross-component tests)

**Performance Tests** (~10 tests):
- Will be added in separate `benches/` directory
- Memory tracking overhead benchmarks
- Enforcement performance impact benchmarks
- Monitoring efficiency benchmarks

### Test Execution

```bash
# Run all tests
cargo test --package airssys-wasm

# Run specific test suites
cargo test --package airssys-wasm --lib                    # Unit tests only
cargo test --package airssys-wasm --tests                  # Integration tests only
cargo test --package airssys-wasm --test memory_limits_test # Specific test file

# Run with output
cargo test --package airssys-wasm -- --nocapture

# Run ignored tests (performance)
cargo test --package airssys-wasm -- --ignored
```

### Performance Validation

**File**: `benches/memory_overhead_bench.rs` (NEW - optional for Phase 2)

```rust
use criterion::{black_box, criterion_group, criterion_main, Criterion};
use airssys_wasm::runtime::limits::{ResourceLimits, ComponentResourceLimiter};

fn benchmark_memory_tracking_overhead(c: &mut Criterion) {
    let limits = ResourceLimits::builder()
        .memory_bytes(1024 * 1024)
        .build()
        .unwrap();
    let mut limiter = ComponentResourceLimiter::new(limits);

    c.bench_function("memory_growing_check", |b| {
        b.iter(|| {
            limiter.memory_growing(
                black_box(0),
                black_box(512 * 1024),
                None
            )
        })
    });
}

criterion_group!(benches, benchmark_memory_tracking_overhead);
criterion_main!(benches);
```

**Performance Target**: Memory tracking overhead <5%

---

## Standards Compliance Checklist

### Workspace Standards (MANDATORY)

- [ ] **§2.1 - 3-Layer Import Organization**
  - [ ] Standard library imports first
  - [ ] Third-party crate imports second
  - [ ] Internal module imports third
  - [ ] Verified in: limits.rs, config.rs, all test files

- [ ] **§3.2 - chrono DateTime<Utc> Standard**
  - [ ] MemoryMetrics uses DateTime<Utc>
  - [ ] No std::time::SystemTime for timestamps
  - [ ] Verified in: limits.rs (MemoryMetrics struct)

- [ ] **§4.3 - mod.rs Declaration-Only Pattern**
  - [ ] runtime/mod.rs contains only declarations and re-exports
  - [ ] No implementation code in mod.rs files
  - [ ] Verified in: runtime/mod.rs, core/mod.rs

- [ ] **§5.1 - Workspace Dependencies**
  - [ ] toml dependency added to workspace
  - [ ] All dependencies use workspace = true
  - [ ] Verified in: Cargo.toml files

- [ ] **§6.1 - YAGNI Principles**
  - [ ] Only Task 2.1, 2.2, 2.3 implemented
  - [ ] No speculative features (e.g., dynamic limit updates)
  - [ ] Simple solutions first (builder pattern, not complex DSL)
  - [ ] Verified in: All implementation files

- [ ] **§6.2 - Avoid dyn Patterns**
  - [ ] No `Box<dyn Trait>` in public APIs
  - [ ] Generic constraints where needed
  - [ ] Concrete types preferred
  - [ ] Verified in: All public APIs

### Microsoft Rust Guidelines (MANDATORY)

- [ ] **M-DESIGN-FOR-AI: Idiomatic APIs**
  - [ ] ResourceLimits::builder() pattern
  - [ ] Clear, self-documenting method names
  - [ ] Comprehensive rustdoc comments
  - [ ] Code examples in documentation

- [ ] **M-DI-HIERARCHY: Concrete > generics > dyn**
  - [ ] ResourceLimits: concrete type
  - [ ] ComponentResourceLimiter: concrete type implementing trait
  - [ ] No unnecessary dyn trait objects

- [ ] **M-ERRORS-CANONICAL-STRUCTS: Structured Errors**
  - [ ] ResourceLimitError enum with thiserror
  - [ ] ConfigError enum with thiserror
  - [ ] OutOfMemory error with context (WasmError)
  - [ ] Error messages include guidance and ADR references

- [ ] **M-MOCKABLE-SYSCALLS: Testable I/O**
  - [ ] ComponentConfig::from_str() (no file I/O in core logic)
  - [ ] ComponentConfig::from_file() separate for file operations
  - [ ] All Wasmtime interactions in ComponentResourceLimiter (trait)

### Code Quality Standards

- [ ] **Zero Compiler Warnings**
  - [ ] `cargo check --package airssys-wasm` passes
  - [ ] All files compile without warnings

- [ ] **Zero Clippy Warnings**
  - [ ] `cargo clippy --package airssys-wasm --all-targets --all-features` passes
  - [ ] No clippy warnings in production or test code

- [ ] **Comprehensive Testing**
  - [ ] Unit tests: ~50 tests (25 + 8 + additional)
  - [ ] Integration tests: ~30 tests
  - [ ] Security tests: ~15 tests (subset of integration)
  - [ ] All tests passing

- [ ] **Documentation Complete**
  - [ ] All public APIs have rustdoc comments
  - [ ] Module-level documentation with examples
  - [ ] Architecture references (ADR-WASM-002, ADR-WASM-006)
  - [ ] Error messages include guidance

---

## Success Validation Criteria

### Functional Requirements

- [ ] **Memory Limits Enforced**
  - [ ] Wasmtime ResourceLimiter trait implemented
  - [ ] Memory allocations checked at runtime
  - [ ] Allocations exceeding limits denied (Ok(false))
  - [ ] Verified with: `memory_limits_test.rs`

- [ ] **Components Cannot Exceed Limits**
  - [ ] Single component OOM behavior correct
  - [ ] Memory usage tracking accurate
  - [ ] Verified with: unit tests + integration tests

- [ ] **OOM Handling Graceful**
  - [ ] OutOfMemory error with context
  - [ ] Error messages informative (component ID, usage, limit)
  - [ ] MemoryMetrics included in error
  - [ ] Verified with: error handling tests

- [ ] **Component.toml Validation**
  - [ ] Missing [resources.memory] rejected (MissingMemoryConfig error)
  - [ ] Invalid memory values rejected (MemoryTooLow/High errors)
  - [ ] Error messages include configuration guidance
  - [ ] Verified with: `config_test.rs`

- [ ] **Memory Range Validation**
  - [ ] 512KB minimum enforced
  - [ ] 4MB maximum enforced
  - [ ] Edge cases (exactly 512KB, exactly 4MB) working
  - [ ] Verified with: unit tests

- [ ] **100% Memory Isolation**
  - [ ] Components cannot see other component memory
  - [ ] OOM in one component doesn't affect others
  - [ ] Independent usage tracking verified
  - [ ] Verified with: `isolation_security_test.rs`

### Testing Requirements

- [ ] **All Unit Tests Passing** (~50 tests)
  - [ ] ResourceLimits tests (8 tests)
  - [ ] MemoryConfig tests (4 tests)
  - [ ] ComponentResourceLimiter tests (9 tests)
  - [ ] Memory monitoring tests (2 tests)
  - [ ] OOM handling tests (2 tests)
  - [ ] Config parsing tests (8 tests)
  - [ ] Additional validation tests

- [ ] **All Integration Tests Passing** (~30 tests)
  - [ ] End-to-end enforcement (10 tests)
  - [ ] Component.toml pipeline (3 tests)
  - [ ] Cross-component isolation (4 tests)
  - [ ] Memory leak detection (3 tests)
  - [ ] Stress testing (4 tests)
  - [ ] Security isolation (5 tests)

- [ ] **All Security Tests Passing** (~15 tests)
  - [ ] Memory isolation breaches (5 tests)
  - [ ] OOM attack scenarios (5 tests)
  - [ ] Resource exhaustion (5 tests)

- [ ] **Performance Tests Passing** (~10 tests)
  - [ ] Memory tracking overhead <5%
  - [ ] Enforcement performance acceptable
  - [ ] Monitoring efficiency validated

### Quality Requirements

- [ ] **Zero Compiler Warnings**
  - [ ] All production code compiles cleanly
  - [ ] All test code compiles cleanly

- [ ] **Zero Clippy Warnings**
  - [ ] `--all-targets --all-features` passes
  - [ ] Production and test code lint-clean

- [ ] **Documentation Complete**
  - [ ] All public APIs documented (rustdoc)
  - [ ] Module-level docs with examples
  - [ ] Code examples tested (doc tests)
  - [ ] Architecture docs updated (if applicable)

- [ ] **Standards Compliance Verified**
  - [ ] §2.1-§6.3 workspace standards followed
  - [ ] Microsoft Rust Guidelines compliance verified
  - [ ] No violations in code review

### Integration Requirements

- [ ] **Component.toml Parsing Updated**
  - [ ] `[resources.memory]` section parsed
  - [ ] MANDATORY validation implemented
  - [ ] Integration with ComponentConfig

- [ ] **WasmEngine Integration**
  - [ ] ComponentResourceLimiter usable with Wasmtime Store
  - [ ] ResourceLimiter trait correctly implemented
  - [ ] Memory limits enforced at runtime

- [ ] **Error Handling Integration**
  - [ ] OutOfMemory error in WasmError enum
  - [ ] Error propagation working end-to-end
  - [ ] User-friendly error messages

- [ ] **Monitoring API Accessible**
  - [ ] MemoryMetrics available from public interface
  - [ ] Current usage queryable
  - [ ] Usage percentage calculated correctly

---

## Phase 2 Completion Criteria

Phase 2 is **100% COMPLETE** when:

1. ✅ All implementation subtasks finished (Tasks 2.1, 2.2, 2.3)
2. ✅ All tests passing (~105 tests total: 50 unit + 30 integration + 15 security + 10 performance)
3. ✅ Zero compiler warnings
4. ✅ Zero clippy warnings
5. ✅ 100% memory isolation verified through comprehensive tests
6. ✅ Performance overhead <5% measured and validated
7. ✅ Documentation complete (rustdoc + architecture docs)
8. ✅ Standards compliance verified (§2.1-§6.3 + Microsoft Guidelines)
9. ✅ All success criteria checkboxes marked complete
10. ✅ Code review passed (if applicable)

---

## Risk Assessment and Mitigation

### Technical Risks

**Risk 1: Wasmtime ResourceLimiter API changes**
- **Likelihood**: Low (stable API in v24.0)
- **Impact**: High (core functionality)
- **Mitigation**: 
  - Pin Wasmtime version in Cargo.toml
  - Monitor Wasmtime changelog before upgrades
  - Comprehensive test suite catches API changes

**Risk 2: Performance overhead exceeds 5% target**
- **Likelihood**: Medium (atomic tracking has cost)
- **Impact**: Medium (acceptable if <10%)
- **Mitigation**:
  - Benchmark early in implementation
  - Use SeqCst ordering only where necessary (consider Relaxed)
  - Profile hotspots with `cargo flamegraph`
  - Consider batching metrics if needed

**Risk 3: 100% isolation difficult to verify**
- **Likelihood**: Medium (requires exhaustive testing)
- **Impact**: High (MANDATORY security requirement)
- **Mitigation**:
  - Comprehensive test suite with 15+ isolation tests
  - Property-based testing with `proptest` (future enhancement)
  - Security review by security team
  - Fuzzing with `cargo-fuzz` (future enhancement)

**Risk 4: OOM handling edge cases**
- **Likelihood**: Medium (complex failure scenarios)
- **Impact**: Medium (graceful degradation critical)
- **Mitigation**:
  - Extensive edge case testing (1000+ iterations)
  - Error injection tests
  - Stress testing under high load
  - Integration with airssys-rt supervisor (future)

### Schedule Risks

**Risk 1: Implementation takes longer than 9 days**
- **Likelihood**: Medium (ambitious timeline)
- **Impact**: Low (no hard deadline, foundational work)
- **Mitigation**:
  - Break into smaller increments (subtasks)
  - Parallel test development (start tests early)
  - Focus on core functionality first (defer optimization)
  - Time-box subtasks (move to next if stuck)

**Risk 2: Testing reveals architectural issues**
- **Likelihood**: Low (well-researched from ADRs)
- **Impact**: High (requires redesign)
- **Mitigation**:
  - Early integration testing (Day 4-5)
  - Continuous validation against ADR requirements
  - Prototype critical paths first
  - Consult ADRs when uncertain

**Risk 3: Integration with Phase 1 problems**
- **Likelihood**: Low (Phase 1 complete before Phase 2)
- **Impact**: Medium (delays start of Phase 2)
- **Mitigation**:
  - Verify Phase 1 completion before starting Phase 2
  - Review Phase 1 implementation for integration points
  - Keep Phase 2 loosely coupled where possible

---

## Post-Phase 2 Integration Plan

### Integration with Phase 3 (CPU Limiting)

Phase 3 will build on Phase 2 foundation:

**Reusable Components**:
- `ResourceLimits` struct already includes `cpu_timeout` field
- `ComponentResourceLimiter` can be extended for fuel metering
- `MemoryMetrics` pattern reusable for CPU metrics
- Error handling infrastructure applies to CPU limit errors

**Phase 3 Additions** (future):
```rust
// cpu_limits.rs will follow similar pattern
pub struct CpuMetrics {
    pub recorded_at: DateTime<Utc>,
    pub fuel_consumed: u64,
    pub fuel_limit: u64,
    pub execution_time: Duration,
}

impl ComponentResourceLimiter {
    pub fn cpu_metrics(&self) -> CpuMetrics { /* ... */ }
}
```

### Integration with airssys-rt

ComponentActor will use Phase 2 infrastructure:

```rust
// Future integration (Phase 5+)
use airssys_rt::{Actor, ActorContext, Child};
use airssys_wasm::runtime::limits::ComponentResourceLimiter;

pub struct ComponentActor {
    limiter: ComponentResourceLimiter,
    // ... other fields
}

impl Actor for ComponentActor {
    // Memory limits enforced via limiter
    // OOM triggers supervisor restart
}
```

**OOM Handling Strategy**:
- ComponentActor monitors `MemoryMetrics`
- OOM error triggers graceful shutdown
- Supervisor receives failure notification
- Automatic restart with exponential backoff
- Persistent OOM → circuit breaker pattern

### Integration with Security Middleware

Memory limits as security policy:

```rust
// Future integration
use airssys_osl::middleware::security::{SecurityPolicy, SecurityContext};

impl SecurityPolicy for MemoryLimitPolicy {
    fn enforce(&self, context: &SecurityContext) -> Result<(), PolicyViolation> {
        // Check component memory limits before execution
        // Integrate with RBAC for per-role memory limits
    }
}
```

**Audit Logging**:
- Log all memory limit violations
- Include component ID, usage, limit, timestamp
- Feed into security monitoring dashboard
- Trigger alerts for suspicious patterns

---

## Documentation Updates Required

### API Documentation (rustdoc)

- [ ] **Module-level docs**: `src/runtime/limits.rs`
  - Overview of memory limit enforcement
  - Architecture explanation (2-level enforcement)
  - Design philosophy (MANDATORY limits, no defaults)
  - Examples of ResourceLimits usage
  - Security considerations (Layer 2 of defense-in-depth)

- [ ] **Struct documentation**:
  - `ResourceLimits`: Purpose, validation, builder pattern
  - `MemoryConfig`: Component.toml mapping, validation
  - `ComponentResourceLimiter`: Wasmtime integration, usage tracking
  - `MemoryMetrics`: Snapshot pattern, monitoring use cases

- [ ] **Error documentation**:
  - `ResourceLimitError`: All variants with examples
  - `ConfigError`: User-friendly guidance in error messages
  - `WasmError::OutOfMemory`: Context and recovery strategies

- [ ] **Code examples**:
  - Basic ResourceLimits usage
  - Component.toml configuration
  - Integration with Wasmtime Store
  - Monitoring memory metrics
  - Error handling patterns

### Architecture Documentation (mdBook)

**Update**: `airssys-wasm/docs/src/architecture/memory-management.md` (NEW)

```markdown
# Memory Management and Isolation

## Overview

AirsSys WASM implements mandatory memory limit enforcement following
ADR-WASM-002 design philosophy: NO default memory limits - engineers
must explicitly declare resource requirements.

## Design Philosophy

### Why No Defaults?

Explicit memory limits prevent "works on my machine" production surprises:
- Forces engineers to think about resource usage
- Makes resource requirements visible in Component.toml
- Prevents unbounded memory growth in production
- Enables capacity planning and resource allocation

## Architecture

### Two-Level Enforcement

1. **Pre-instantiation**: Component.toml validation
   - Rejects components without `[resources.memory]` section
   - Validates limits are within 512KB-4MB range
   - Clear error messages with configuration guidance

2. **Runtime**: Wasmtime ResourceLimiter trait
   - Real-time memory allocation tracking
   - Deny allocations exceeding configured limits
   - Graceful OOM error handling

## Configuration

See [Configuration Guide](../guides/component-toml.md) for examples.

## Security

Memory limits are Layer 2 of the 4-layer defense-in-depth architecture
(ADR-WASM-006). See [Security](../architecture/security.md) for details.
```

**Update**: `airssys-wasm/docs/src/guides/component-toml.md` (NEW or UPDATE)

Add section:
```markdown
## Memory Configuration

Memory limits are MANDATORY in Component.toml.

### Example

```toml
[resources.memory]
max_bytes = 1048576  # 1MB
```

### Constraints

- Minimum: 512KB (524,288 bytes)
- Maximum: 4MB (4,194,304 bytes)

### Error Handling

Missing `[resources.memory]` section will cause validation error:
```
Missing [resources.memory] section in Component.toml
Memory limits are MANDATORY (no defaults - intentional design).
...
```
```

### Memory Bank Updates

- [ ] **Update progress.md**:
  - Mark Phase 2 complete
  - Update overall completion: 25% → 35%
  - Document key deliverables
  - Note any deviations from plan

- [ ] **Create knowledge doc** (optional):
  - `knowledge_wasm_0XX_resourcelimiter_pattern.md`
  - Document Wasmtime ResourceLimiter trait implementation
  - Reusable pattern for future resource limiting (CPU, I/O)
  - Lessons learned and gotchas

- [ ] **Update tech_context.md**:
  - Memory performance targets achieved (<5% overhead)
  - 100% isolation verification complete
  - MANDATORY memory limits enforced

---

## Appendix: Key Reference Documents

### Architecture Decision Records (ADRs)

**ADR-WASM-002: WASM Runtime Engine Selection** (1,100 lines)
- **Location**: `.memory-bank/sub_projects/airssys-wasm/docs/adr/adr_wasm_002_runtime_engine_selection.md`
- **Key Sections**:
  - Memory limits MANDATORY (lines 450-520)
  - Memory range: 512KB-4MB (line 487)
  - ResourceLimiter trait implementation (lines 521-580)
  - Design rationale: No defaults philosophy (lines 490-510)
  - Performance targets: <5% overhead (line 823)

**ADR-WASM-006: Component Isolation and Sandboxing** (494 lines)
- **Location**: `.memory-bank/sub_projects/airssys-wasm/docs/adr/adr_wasm_006_component_isolation_sandboxing.md`
- **Key Sections**:
  - 4-layer defense-in-depth (lines 45-110)
  - Layer 2: WASM linear memory isolation (lines 75-85)
  - 100% isolation requirement (line 78)
  - ComponentActor dual-trait design (lines 280-350)
  - Integration with airssys-rt supervisors (lines 351-420)

### Knowledge Documentation

**KNOWLEDGE-WASM-012: Module Structure Architecture**
- **Location**: `.memory-bank/sub_projects/airssys-wasm/docs/knowledges/knowledge_wasm_012_module_structure_architecture.md`
- **Key Sections**:
  - Hybrid block-aligned approach
  - `runtime/` module structure (this phase)
  - Dependency rules and public API patterns
  - Re-export conventions

### Workspace Standards

**shared_patterns.md**
- **Location**: `.memory-bank/workspace/shared_patterns.md`
- **Mandatory Sections**:
  - §2.1: 3-Layer Import Organization
  - §3.2: chrono DateTime<Utc> Standard
  - §4.3: mod.rs Declaration-Only Pattern
  - §5.1: Workspace Dependencies
  - §6.1: YAGNI Principles
  - §6.2: Avoid dyn Patterns
  - §6.3: Microsoft Rust Guidelines Integration

**microsoft_rust_guidelines.md**
- **Location**: `.memory-bank/workspace/microsoft_rust_guidelines.md`
- **Key Guidelines**:
  - M-DESIGN-FOR-AI: Idiomatic API design
  - M-DI-HIERARCHY: Concrete > generics > dyn
  - M-ERRORS-CANONICAL-STRUCTS: Structured errors with Backtrace
  - M-MOCKABLE-SYSCALLS: All I/O mockable for testing

**documentation_terminology_standards.md**
- **Location**: `.memory-bank/workspace/documentation_terminology_standards.md`
- **Standards**:
  - Professional, objective language
  - No hyperbole or self-promotion
  - Factual, sourced claims only
  - Implementation status clarity

### Templates

**Phase 1 Implementation Plan** (template reference)
- **Location**: `.memory-bank/sub_projects/airssys-wasm/tasks/task_002_phase_1_implementation_plan.md`
- **Size**: 1,155 lines
- **Used for**:
  - Structure template for this plan
  - Code example formatting patterns
  - Testing strategy organization
  - Standards checklist approach

---

## Implementation Timeline Summary

**Total Duration**: 5-10 days (flexible based on complexity)

**Day 1**: Task 2.1 Subtasks 1-3 (limits.rs foundation)
- Module structure
- ResourceLimits struct with builder
- MemoryConfig struct with validation

**Day 2**: Task 2.1 Subtasks 4-5 (Wasmtime integration)
- ComponentResourceLimiter trait implementation
- Memory usage monitoring with MemoryMetrics

**Day 3**: Task 2.1 Subtask 6 (error handling)
- OOM error types and messages
- Error propagation integration

**Day 4**: Task 2.2 Subtasks 1-2 (Component.toml parsing)
- ComponentConfig struct
- [resources.memory] section parsing
- Validation logic

**Day 5**: Task 2.2 Subtasks 3-4 (validation and integration)
- Error messages refinement
- toml dependency
- End-to-end enforcement pipeline

**Day 6**: Buffer day / catchup
- Address any issues from Days 1-5
- Additional testing
- Documentation refinement

**Day 7**: Task 2.3 Subtasks 1-2 (isolation tests)
- Component memory boundary tests
- Cross-component isolation tests

**Day 8**: Task 2.3 Subtasks 3-4 (advanced testing)
- Memory leak detection tests
- Stress testing

**Day 9**: Task 2.3 Subtask 5 + final validation
- 100% isolation verification
- Final quality checks
- Documentation completion

---

## Next Steps After Phase 2

After Phase 2 completion, proceed to **Phase 3: CPU Limiting** or continue with other Block 1 phases based on priority.

**Recommended Sequence**:
1. Phase 2: Memory Management ✅ (this document)
2. Phase 3: CPU Limiting (fuel metering + wall-clock timeout)
3. Phase 4: Crash Isolation (component failure containment)
4. Integration with airssys-rt ComponentActor
5. Block 2: WIT Interface System

**Preparation for Phase 3**:
- Review CPU limiting requirements from ADR-WASM-002
- Research Wasmtime fuel metering API
- Plan hybrid fuel + timeout approach
- Design CpuMetrics similar to MemoryMetrics pattern

---

## Conclusion

This implementation plan provides a comprehensive, day-by-day roadmap for Phase 2 of WASM-TASK-002. It follows established patterns from Phase 1, adheres to all mandatory workspace standards and Microsoft Rust Guidelines, and delivers the critical memory management and isolation foundation required for the AirsSys WASM Component Framework.

**Key Achievements After Phase 2**:
- ✅ MANDATORY memory limits enforced (512KB-4MB range)
- ✅ 100% memory isolation between components verified
- ✅ Graceful OOM handling with informative errors
- ✅ Component.toml validation with user-friendly guidance
- ✅ Comprehensive test suite (~105 tests)
- ✅ Performance overhead <5% validated
- ✅ Foundation for Layer 2 security (4-layer defense-in-depth)

**Ready to begin Phase 2 implementation.**

---

**Document Version**: 1.0  
**Last Updated**: 2025-10-23  
**Status**: Planning Complete - Ready for Implementation
