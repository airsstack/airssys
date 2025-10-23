# WASM-TASK-002 Phase 1: Implementation Plan
## Wasmtime Setup and Basic Execution

**Status:** Planning Complete - Ready for Implementation  
**Created:** 2025-10-23  
**Duration:** 6 days (Week 1-2)  
**Priority:** Critical Path - Foundation Layer

---

## Executive Summary

This document provides a comprehensive, day-by-day implementation plan for **WASM-TASK-002 Phase 1**, covering:

- **Task 1.1**: Wasmtime Dependency and Engine Setup (Days 1-2)
- **Task 1.2**: Component Loading and Instantiation (Days 3-4)
- **Task 1.3**: Error Handling Foundation (Days 5-6)

Phase 1 establishes the foundational WASM runtime layer using Wasmtime v24.0+ with Component Model support, enabling basic component execution with proper error handling.

---

## Context and Prerequisites

### Current Project State

**Completion Status**:
- **Overall**: 25% complete
- **WASM-TASK-000**: 100% complete - Core abstractions implemented
  - 9,283 lines of production code
  - 363 comprehensive tests
  - Zero compiler warnings
  - All 15 core modules complete

**Codebase Structure**:
```
airssys-wasm/src/
├── core/ (15 modules - COMPLETE)
│   ├── component.rs, capability.rs, error.rs, config.rs
│   ├── runtime.rs, interface.rs, actor.rs, security.rs
│   ├── messaging.rs, storage.rs, lifecycle.rs, management.rs
│   ├── bridge.rs, observability.rs, mod.rs
├── lib.rs, prelude.rs
```

**Current Dependencies** (Cargo.toml):
```toml
[dependencies]
serde = { workspace = true }
thiserror = { workspace = true }
chrono = { workspace = true }
async-trait = { workspace = true }
serde_json = { workspace = true }
uuid = { workspace = true }
tokio = { workspace = true, features = ["macros", "rt"] }
```

**Missing**: Wasmtime dependencies, `runtime/` implementation module

### Key Reference Documents

**Architecture Decisions**:
- **ADR-WASM-002**: WASM Runtime Engine Selection (PRIMARY REFERENCE)
  - Runtime: Wasmtime v24.0+ with Component Model + async
  - Memory limits: MANDATORY in Component.toml (no defaults)
  - CPU limiting: Hybrid fuel metering + wall-clock timeout
  - Async: Mandatory async-first with Tokio integration
  - Crash isolation: Component failures don't crash host

**Module Structure**:
- **KNOWLEDGE-WASM-012**: Module Structure Architecture (MANDATORY)
  - Hybrid block-aligned approach with core abstractions
  - `runtime/` module structure defined
  - Dependency rules and public API patterns

**Workspace Standards**:
- §2.1: 3-Layer Import Organization (MANDATORY)
- §3.2: chrono DateTime<Utc> Standard (MANDATORY)
- §4.3: mod.rs Declaration-Only Pattern (MANDATORY)
- §6.1: YAGNI Principles (MANDATORY)
- §6.2: Avoid dyn Patterns (MANDATORY)
- §6.3: Microsoft Rust Guidelines (MANDATORY)

### Technical Requirements

**Performance Targets** (from ADR-WASM-002):
- Component instantiation: <10ms cold start (baseline target)
- Function call overhead: <100ns (not critical but measured)
- Execution speed: Near-native (95%+ of native code)

**Security Requirements**:
- Memory limits MANDATORY (must be explicit in Component.toml)
- CPU limiting: Dual protection (fuel + timeout)
- Crash isolation: Component failures contained
- No defaults: Engineers must declare resource limits

**Integration Requirements**:
- Async-first architecture (Tokio integration)
- Compatible with airssys-rt actor system (future)
- Foundation for WIT interface system (Block 2)

---

## Phase 1 Objectives

### Primary Objective
Establish a working WASM runtime layer that can load and execute basic Component Model WASM modules with proper error handling, laying the foundation for all subsequent runtime features.

### Specific Deliverables

**Task 1.1: Wasmtime Dependency and Engine Setup**
- ✅ Wasmtime dependencies added to Cargo.toml with correct features
- ✅ `runtime/` module structure created following KNOWLEDGE-WASM-012
- ✅ `WasmEngine` struct with production-ready configuration
- ✅ Component Model and async support enabled
- ✅ Basic engine instantiation tests passing

**Task 1.2: Component Loading and Instantiation**
- ✅ `ComponentLoader` struct for .wasm file loading
- ✅ Component validation and verification logic
- ✅ Basic instantiation pipeline working
- ✅ "Hello World" component executes successfully
- ✅ Instantiation timing < 10ms measured

**Task 1.3: Error Handling Foundation**
- ✅ `RuntimeError` enum with comprehensive error variants
- ✅ Wasmtime error translation layer implemented
- ✅ Error context propagation working
- ✅ User-friendly error messages
- ✅ Error recovery strategies documented

### Success Criteria

This phase is complete when:
1. ✅ Engine initializes successfully with Component Model support
2. ✅ Valid Component Model .wasm files load correctly
3. ✅ Invalid components rejected with clear error messages
4. ✅ Basic "hello world" component executes
5. ✅ Cold start instantiation < 10ms (measured)
6. ✅ All Wasmtime errors properly translated
7. ✅ Zero compiler warnings
8. ✅ Unit tests pass for all new code
9. ✅ Integration test demonstrates end-to-end flow
10. ✅ Documentation complete for all public APIs

---

## Implementation Details

### Task 1.1: Wasmtime Dependency and Engine Setup

#### Subtask 1.1.1: Add Wasmtime Dependencies

**File**: `airssys-wasm/Cargo.toml`

**Changes**:
```toml
[dependencies]
# Existing dependencies...
serde = { workspace = true }
thiserror = { workspace = true }
chrono = { workspace = true }
async-trait = { workspace = true }
serde_json = { workspace = true }
uuid = { workspace = true }
tokio = { workspace = true, features = ["macros", "rt"] }

# NEW: Wasmtime dependencies (Block 1: WASM Runtime Layer)
wasmtime = { version = "24.0", features = ["component-model", "async", "cranelift"] }
wasmtime-wasi = { version = "24.0" }

[dev-dependencies]
# NEW: Test fixture for WASM components
wat = "1.0"  # WAT (WebAssembly Text) to WASM compiler for test fixtures
```

**Rationale**:
- `wasmtime v24.0`: Latest stable with mature Component Model support
- `component-model` feature: Enables Component Model (not just core WASM)
- `async` feature: Mandatory for Tokio integration (ADR-WASM-002)
- `cranelift` feature: JIT compiler (ADR-WASM-002 decision)
- `wasmtime-wasi v24.0`: WASI Preview 2 support (future phases)
- `wat` (dev): Allows writing test components in WAT format inline

**Verification**:
```bash
cargo check --package airssys-wasm
```

---

#### Subtask 1.1.2: Create runtime/ Module Structure

**Files to Create**:
```
airssys-wasm/src/runtime/
├── mod.rs          # Module declarations and re-exports ONLY (§4.3)
├── engine.rs       # WasmEngine implementation
├── instance.rs     # WasmInstance management (Phase 2)
├── limits.rs       # ResourceLimits types (Phase 2)
├── loader.rs       # ComponentLoader implementation
└── executor.rs     # Component execution logic (Phase 2)
```

**File 1**: `src/runtime/mod.rs`

```rust
//! WASM runtime execution engine.
//!
//! This module provides the core WASM runtime layer using Wasmtime with
//! Component Model support, memory sandboxing, CPU limiting, and crash isolation.
//!
//! # Overview
//!
//! The runtime module implements Block 1 (WASM Runtime Layer) from the
//! implementation roadmap. It provides:
//!
//! - Component Model execution via Wasmtime
//! - Memory limits (512KB-4MB configurable)
//! - CPU limiting (hybrid fuel + wall-clock timeout)
//! - Async execution with Tokio integration
//! - Crash isolation (component failures don't crash host)
//!
//! # Architecture
//!
//! Following ADR-WASM-002 (WASM Runtime Engine Selection), this module:
//! - Uses Wasmtime v24.0+ as the runtime engine
//! - Enables Component Model and async support
//! - Enforces mandatory memory limits from Component.toml
//! - Implements hybrid CPU limiting (fuel + timeout)
//!
//! # Examples
//!
//! ```rust,ignore
//! use airssys_wasm::runtime::{WasmEngine, ComponentLoader};
//!
//! // Create runtime engine
//! let engine = WasmEngine::new()?;
//!
//! // Load component
//! let loader = ComponentLoader::new(&engine);
//! let component = loader.load_from_file("component.wasm").await?;
//! ```
//!
//! # Module Organization (KNOWLEDGE-WASM-012)
//!
//! - [`engine`] - Wasmtime engine wrapper and configuration
//! - [`loader`] - Component loading and validation
//! - [`instance`] - Component instance management (Phase 2)
//! - [`limits`] - Resource limits enforcement (Phase 2)
//! - [`executor`] - Component execution logic (Phase 2)

pub mod engine;
pub mod loader;

pub use engine::WasmEngine;
pub use loader::ComponentLoader;
```

**Verification**:
```bash
cargo check --package airssys-wasm
```

---

#### Subtask 1.1.3: Implement WasmEngine

**File**: `src/runtime/engine.rs`

**Implementation**:
```rust
//! Wasmtime engine wrapper and configuration.
//!
//! This module provides the `WasmEngine` type that wraps Wasmtime's `Engine`
//! with production-ready configuration following ADR-WASM-002 decisions.

use std::sync::Arc;
use wasmtime::{Config, Engine, OptLevel, Strategy};
use crate::core::error::{WasmError, WasmResult};

/// Wasmtime engine wrapper with production configuration.
///
/// `WasmEngine` encapsulates Wasmtime's `Engine` with configuration aligned
/// to ADR-WASM-002 decisions:
/// - Component Model support enabled
/// - Async support enabled (mandatory for Tokio integration)
/// - JIT compilation with Cranelift optimizer
/// - Fuel metering enabled (CPU limiting)
/// - Production-ready defaults
///
/// # Examples
///
/// ```rust,ignore
/// use airssys_wasm::runtime::WasmEngine;
///
/// let engine = WasmEngine::new()?;
/// ```
///
/// # Architecture
///
/// The engine is designed to be shared across multiple component instances.
/// It implements `Clone` cheaply via `Arc<Engine>` (§6.3 M-SERVICES-CLONE).
#[derive(Clone)]
pub struct WasmEngine {
    inner: Arc<Engine>,
}

impl WasmEngine {
    /// Create a new WasmEngine with production configuration.
    ///
    /// This configures Wasmtime according to ADR-WASM-002:
    /// - Component Model: Enabled (critical requirement)
    /// - Async support: Enabled (Tokio integration)
    /// - Compilation: JIT with Cranelift optimizer
    /// - Fuel metering: Enabled (CPU limiting)
    /// - Stack limit: 2MB default (configurable per-component)
    ///
    /// # Errors
    ///
    /// Returns `WasmError::EngineInitialization` if engine creation fails.
    pub fn new() -> WasmResult<Self> {
        let mut config = Config::new();
        
        config.wasm_component_model(true);
        config.async_support(true);
        config.strategy(Strategy::Cranelift);
        config.cranelift_opt_level(OptLevel::Speed);
        config.consume_fuel(true);
        config.max_wasm_stack(2 * 1024 * 1024);
        config.parallel_compilation(true);

        let engine = Engine::new(&config).map_err(|e| {
            WasmError::EngineInitialization {
                reason: e.to_string(),
            }
        })?;

        Ok(Self {
            inner: Arc::new(engine),
        })
    }

    pub(crate) fn inner(&self) -> &Engine {
        &self.inner
    }
}

impl Default for WasmEngine {
    fn default() -> Self {
        Self::new().expect("Failed to create default WasmEngine")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_engine_creation() {
        let engine = WasmEngine::new();
        assert!(engine.is_ok(), "Engine creation should succeed");
    }

    #[test]
    fn test_engine_clone() {
        let engine1 = WasmEngine::new().unwrap();
        let engine2 = engine1.clone();
        assert!(Arc::ptr_eq(&engine1.inner, &engine2.inner));
    }

    #[test]
    fn test_default_engine() {
        let _engine = WasmEngine::default();
    }
}
```

**Error Type Addition**:

**File**: `src/core/error.rs`

Add new error variant:
```rust
#[derive(Debug, thiserror::Error)]
pub enum WasmError {
    // Existing variants...
    
    #[error("Failed to initialize WASM engine: {reason}")]
    EngineInitialization { reason: String },
}
```

**Verification**:
```bash
cargo test --package airssys-wasm runtime::engine
```

---

### Task 1.2: Component Loading and Instantiation

#### Subtask 1.2.1: Implement ComponentLoader

**File**: `src/runtime/loader.rs`

**Implementation**:
```rust
//! Component loading and validation.

use std::path::Path;
use wasmtime::component::Component;
use crate::core::error::{WasmError, WasmResult};
use crate::runtime::engine::WasmEngine;

/// Component loader for WebAssembly Component Model modules.
pub struct ComponentLoader {
    engine: WasmEngine,
}

impl ComponentLoader {
    pub fn new(engine: &WasmEngine) -> Self {
        Self {
            engine: engine.clone(),
        }
    }

    pub async fn load_from_file<P: AsRef<Path>>(
        &self,
        path: P,
    ) -> WasmResult<Component> {
        let path = path.as_ref();

        let component = Component::from_file(self.engine.inner(), path)
            .map_err(|e| WasmError::ComponentLoadFailed {
                path: path.display().to_string(),
                reason: e.to_string(),
            })?;

        Ok(component)
    }

    pub async fn load_from_bytes(&self, bytes: &[u8]) -> WasmResult<Component> {
        let component = Component::from_binary(self.engine.inner(), bytes)
            .map_err(|e| WasmError::ComponentParseFailed {
                reason: e.to_string(),
            })?;

        Ok(component)
    }

    pub async fn validate(&self, bytes: &[u8]) -> WasmResult<()> {
        Component::from_binary(self.engine.inner(), bytes)
            .map_err(|e| WasmError::ComponentValidationFailed {
                reason: e.to_string(),
            })?;

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn test_engine() -> WasmEngine {
        WasmEngine::new().unwrap()
    }

    #[tokio::test]
    async fn test_loader_creation() {
        let engine = test_engine();
        let _loader = ComponentLoader::new(&engine);
    }

    #[tokio::test]
    async fn test_load_invalid_bytes() {
        let engine = test_engine();
        let loader = ComponentLoader::new(&engine);

        let invalid_bytes = vec![0x00, 0x61, 0x73, 0x6d];
        let result = loader.load_from_bytes(&invalid_bytes).await;

        assert!(result.is_err());
        assert!(matches!(result.unwrap_err(), WasmError::ComponentParseFailed { .. }));
    }

    #[tokio::test]
    async fn test_validate_invalid_bytes() {
        let engine = test_engine();
        let loader = ComponentLoader::new(&engine);

        let invalid_bytes = vec![0x00, 0x61, 0x73, 0x6d];
        let result = loader.validate(&invalid_bytes).await;

        assert!(result.is_err());
        assert!(matches!(result.unwrap_err(), WasmError::ComponentValidationFailed { .. }));
    }
}
```

**Error Type Additions**:

**File**: `src/core/error.rs`

Add new error variants:
```rust
#[derive(Debug, thiserror::Error)]
pub enum WasmError {
    // Existing variants...
    
    #[error("Failed to load component from {path}: {reason}")]
    ComponentLoadFailed { path: String, reason: String },

    #[error("Failed to parse component from bytes: {reason}")]
    ComponentParseFailed { reason: String },

    #[error("Component validation failed: {reason}")]
    ComponentValidationFailed { reason: String },
}
```

**Verification**:
```bash
cargo test --package airssys-wasm runtime::loader
```

---

#### Subtask 1.2.2: Create Integration Test with WAT Fixture

**File**: `tests/runtime_basic_execution_test.rs`

**Implementation**:
```rust
//! Integration tests for basic WASM component execution.

use std::path::PathBuf;
use airssys_wasm::runtime::{ComponentLoader, WasmEngine};

fn fixtures_dir() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures")
}

#[tokio::test]
async fn test_load_hello_world_component() {
    let engine = WasmEngine::new().expect("Failed to create engine");
    let loader = ComponentLoader::new(&engine);

    let component_path = fixtures_dir().join("hello_world.wasm");
    let result = loader.load_from_file(&component_path).await;

    assert!(
        result.is_ok(),
        "Should successfully load hello world component: {:?}",
        result.err()
    );
}

#[tokio::test]
async fn test_load_nonexistent_component() {
    let engine = WasmEngine::new().expect("Failed to create engine");
    let loader = ComponentLoader::new(&engine);

    let result = loader.load_from_file("nonexistent.wasm").await;

    assert!(result.is_err(), "Should fail to load non-existent component");
}

#[tokio::test]
async fn test_validate_valid_component_bytes() {
    let engine = WasmEngine::new().expect("Failed to create engine");
    let loader = ComponentLoader::new(&engine);

    let component_path = fixtures_dir().join("hello_world.wasm");
    let bytes = std::fs::read(&component_path).expect("Failed to read fixture");

    let result = loader.validate(&bytes).await;

    assert!(result.is_ok(), "Should validate valid component bytes");
}

#[tokio::test]
async fn test_cold_start_instantiation_timing() {
    use std::time::Instant;

    let engine = WasmEngine::new().expect("Failed to create engine");
    let loader = ComponentLoader::new(&engine);

    let component_path = fixtures_dir().join("hello_world.wasm");
    
    let start = Instant::now();
    let result = loader.load_from_file(&component_path).await;
    let duration = start.elapsed();

    assert!(result.is_ok(), "Component should load successfully");
    
    println!("Component load time: {:?}", duration);
    
    assert!(
        duration.as_millis() < 50,
        "Component load should be reasonably fast (< 50ms), got {:?}",
        duration
    );
}
```

**Test Fixture Creation**:

**File**: `tests/fixtures/hello_world.wat`

```wat
;; Minimal WebAssembly Component Model component for testing

(component
  (core module $m
    (func (export "hello") (result i32)
      i32.const 42
    )
  )
  
  (core instance $i (instantiate $m))
  
  (func (export "hello") (result s32)
    (canon lift (core func $i "hello"))
  )
)
```

**Fixture Compilation Script**:

**File**: `tests/fixtures/build.sh`

```bash
#!/bin/bash
# Compile WAT fixtures to WASM
# Requires: wasm-tools (install via: cargo install wasm-tools)

set -e

FIXTURES_DIR="$(dirname "$0")"

echo "Compiling WAT fixtures to WASM..."

for wat_file in "$FIXTURES_DIR"/*.wat; do
    if [ -f "$wat_file" ]; then
        wasm_file="${wat_file%.wat}.wasm"
        echo "  $wat_file -> $wasm_file"
        wasm-tools component new "$wat_file" -o "$wasm_file"
    fi
done

echo "Done."
```

**Verification**:
```bash
cd airssys-wasm/tests/fixtures
chmod +x build.sh
./build.sh

cargo test --package airssys-wasm --test runtime_basic_execution_test
```

---

### Task 1.3: Error Handling Foundation

#### Subtask 1.3.1: Expand Core Error Types

**File**: `src/core/error.rs`

Add runtime-specific errors:

```rust
#[derive(Debug, thiserror::Error)]
pub enum WasmError {
    // ... existing variants from WASM-TASK-000 ...

    // === Runtime Layer Errors (Phase 1) ===
    
    #[error("Failed to initialize WASM engine: {reason}")]
    EngineInitialization {
        reason: String,
    },

    #[error("Failed to load component from {path}: {reason}")]
    ComponentLoadFailed {
        path: String,
        reason: String,
    },

    #[error("Failed to parse component from bytes: {reason}")]
    ComponentParseFailed {
        reason: String,
    },

    #[error("Component validation failed: {reason}")]
    ComponentValidationFailed {
        reason: String,
    },

    // === Placeholder for Future Phases ===
    
    #[error("Failed to instantiate component {component_id}: {reason}")]
    ComponentInstantiationFailed {
        component_id: String,
        reason: String,
    },

    #[error("Component trapped during execution: {reason}")]
    ComponentTrapped {
        reason: String,
        fuel_consumed: Option<u64>,
    },

    #[error("Component execution exceeded timeout ({max_execution_ms}ms)")]
    ComponentTimeout {
        max_execution_ms: u64,
        fuel_consumed: Option<u64>,
    },
}

impl WasmError {
    pub fn is_load_error(&self) -> bool {
        matches!(
            self,
            WasmError::ComponentLoadFailed { .. }
                | WasmError::ComponentParseFailed { .. }
                | WasmError::ComponentValidationFailed { .. }
        )
    }

    pub fn is_engine_error(&self) -> bool {
        matches!(self, WasmError::EngineInitialization { .. })
    }

    pub fn is_runtime_error(&self) -> bool {
        matches!(
            self,
            WasmError::ComponentTrapped { .. } | WasmError::ComponentTimeout { .. }
        )
    }
}
```

**Verification**:
```bash
cargo test --package airssys-wasm core::error
```

---

#### Subtask 1.3.2: Update Public API

**File**: `src/lib.rs`

Update module declaration:
```rust
//! # airssys-wasm - WASM Component Framework
//!
//! Runtime deployment infrastructure for WebAssembly Component Model-based
//! pluggable systems with capability-based security.

pub mod core;
pub mod runtime;
pub mod prelude;

pub use core::error::{WasmError, WasmResult};
```

**File**: `src/prelude.rs`

Add runtime types:
```rust
//! Prelude module for common airssys-wasm types and traits.

pub use crate::core::{
    capability::{Capability, CapabilitySet},
    component::{Component, ComponentConfig, ComponentId, ComponentMetadata},
    config::RuntimeConfig,
    error::{WasmError, WasmResult},
};

pub use crate::runtime::{ComponentLoader, WasmEngine};
```

**Verification**:
```bash
cargo check --package airssys-wasm
```

---

## Testing Strategy

### Unit Testing Requirements

**Coverage Target**: >90% for Phase 1 code

**Test Files**:
- `src/runtime/engine.rs` - Tests inline
- `src/runtime/loader.rs` - Tests inline

**Test Categories**:
1. Success paths - Normal operation with valid inputs
2. Error paths - Invalid inputs and error handling
3. Edge cases - Boundary conditions and unusual inputs

### Integration Testing Requirements

**Test File**: `tests/runtime_basic_execution_test.rs`

**Test Scenarios**:
1. ✅ Load valid Component Model WASM from file
2. ✅ Load non-existent file (error handling)
3. ✅ Validate valid component bytes
4. ✅ Validate invalid component bytes (error handling)
5. ✅ Measure cold start instantiation timing

**Test Fixtures**:
- `tests/fixtures/hello_world.wat` - Minimal valid component
- `tests/fixtures/build.sh` - Fixture compilation script

### Test Execution

```bash
# Unit tests only
cargo test --package airssys-wasm --lib

# Integration tests only
cargo test --package airssys-wasm --test runtime_basic_execution_test

# All tests
cargo test --package airssys-wasm

# With output
cargo test --package airssys-wasm -- --nocapture
```

---

## Standards Compliance Checklist

### §2.1: 3-Layer Import Organization ✅

All `.rs` files follow pattern:
```rust
// Layer 1: Standard library
// Layer 2: Third-party crates
// Layer 3: Internal modules
```

### §4.3: mod.rs Declaration-Only Pattern ✅

`src/runtime/mod.rs` contains ONLY:
- Module declarations (`pub mod engine;`)
- Re-exports (`pub use engine::WasmEngine;`)
- Documentation
- NO implementation code ✅

### §6.1: YAGNI Principles ✅

Applied:
- ✅ No module caching yet (defer to Phase 2)
- ✅ No AOT compilation support (Phase 2 optimization)
- ✅ Simple loader implementation
- ✅ Basic error types (expand as needed)

### §6.2: Avoid dyn Patterns ✅

No `Box<dyn Trait>` or `&dyn Trait` in public APIs

### §6.3: Microsoft Rust Guidelines ✅

- **M-SERVICES-CLONE**: `WasmEngine` implements cheap `Clone` via `Arc<Engine>` ✅
- **M-ERRORS-CANONICAL-STRUCTS**: `WasmError` with helper methods ✅
- **M-DESIGN-FOR-AI**: Comprehensive rustdoc with examples ✅
- **M-ESSENTIAL-FN-INHERENT**: Core methods in inherent impl blocks ✅

---

## Day-by-Day Implementation Timeline

### Day 1: Dependencies and Module Structure (4 hours)

**Tasks**:
1. Add Wasmtime dependencies to `Cargo.toml` (30 min)
2. Create `runtime/` module structure (30 min)
3. Implement `runtime/mod.rs` with documentation (1 hour)
4. Verify compilation with `cargo check` (30 min)
5. Update `lib.rs` and `prelude.rs` (1 hour)
6. Run initial tests (30 min)

**Deliverable**: Compilable crate with `runtime/` module structure

**Verification**:
```bash
cargo check --package airssys-wasm
cargo test --package airssys-wasm
```

---

### Day 2: WasmEngine Implementation (6 hours)

**Tasks**:
1. Implement `runtime/engine.rs` (2 hours)
2. Add error types to `core/error.rs` (1 hour)
3. Write unit tests for engine (1.5 hours)
4. Run tests and fix issues (1 hour)
5. Documentation review and improvements (30 min)

**Deliverable**: Working `WasmEngine` with tests

**Verification**:
```bash
cargo test --package airssys-wasm runtime::engine
cargo clippy --package airssys-wasm
```

---

### Day 3: ComponentLoader Implementation (6 hours)

**Tasks**:
1. Implement `runtime/loader.rs` (2 hours)
2. Add loader error types (30 min)
3. Write unit tests for loader (1.5 hours)
4. Test error handling paths (1 hour)
5. Documentation review (30 min)
6. Integration with engine (30 min)

**Deliverable**: Working `ComponentLoader` with tests

**Verification**:
```bash
cargo test --package airssys-wasm runtime::loader
```

---

### Day 4: Test Fixtures and Integration Tests (6 hours)

**Tasks**:
1. Create WAT fixtures (`hello_world.wat`) (1 hour)
2. Write fixture build script (30 min)
3. Install `wasm-tools` and compile fixtures (30 min)
4. Implement integration tests (2 hours)
5. Run integration tests (1 hour)
6. Fix any issues found (1 hour)

**Deliverable**: Passing integration tests with fixtures

**Verification**:
```bash
cd tests/fixtures && ./build.sh
cargo test --package airssys-wasm --test runtime_basic_execution_test
```

---

### Day 5: Error Handling and Edge Cases (5 hours)

**Tasks**:
1. Review all error paths (1 hour)
2. Add helper methods to `WasmError` (30 min)
3. Write additional edge case tests (2 hours)
4. Test invalid inputs thoroughly (1 hour)
5. Documentation improvements (30 min)

**Deliverable**: Comprehensive error handling with tests

**Verification**:
```bash
cargo test --package airssys-wasm -- --nocapture
```

---

### Day 6: Documentation, Review, and Polish (6 hours)

**Tasks**:
1. Review all rustdoc comments (1 hour)
2. Add missing examples (1 hour)
3. Run `cargo clippy` and fix warnings (1 hour)
4. Run full test suite (30 min)
5. Measure cold start timing (30 min)
6. Code review and cleanup (1 hour)
7. Update memory bank `progress.md` (1 hour)

**Deliverable**: Phase 1 complete, ready for Phase 2

**Verification**:
```bash
cargo clippy --package airssys-wasm --all-targets
cargo test --package airssys-wasm
cargo doc --package airssys-wasm --no-deps --open
```

---

## Acceptance Criteria

### Task 1.1: Wasmtime Dependency and Engine Setup ✅

- [ ] Wasmtime v24.0 dependencies added with correct features
- [ ] `runtime/` module structure created following KNOWLEDGE-WASM-012
- [ ] `WasmEngine` struct implemented with production config
- [ ] Component Model support enabled and verified
- [ ] Async support enabled and verified
- [ ] Unit tests pass for engine setup
- [ ] Zero compiler warnings
- [ ] Documentation complete

### Task 1.2: Component Loading and Instantiation ✅

- [ ] `ComponentLoader` struct implemented
- [ ] Load from file working
- [ ] Load from bytes working
- [ ] Component validation working
- [ ] Invalid components rejected with clear errors
- [ ] "Hello world" component fixture compiles
- [ ] Integration test passes end-to-end
- [ ] Cold start timing < 10ms measured (lenient check)
- [ ] Zero compiler warnings
- [ ] Documentation complete

### Task 1.3: Error Handling Foundation ✅

- [ ] `RuntimeError` enum expanded with Phase 1 variants
- [ ] Wasmtime errors translated to `WasmError`
- [ ] Error context propagated correctly
- [ ] User-friendly error messages
- [ ] Helper methods for error categorization
- [ ] Error paths tested thoroughly
- [ ] Documentation complete

### Overall Phase 1 Completion Criteria ✅

- [ ] Engine initializes successfully
- [ ] Valid Component Model .wasm files load
- [ ] Invalid components rejected clearly
- [ ] Basic component executes successfully
- [ ] Instantiation timing measured (baseline)
- [ ] All tests pass (unit + integration)
- [ ] Zero compiler warnings
- [ ] >90% code coverage
- [ ] Documentation complete
- [ ] Standards compliance verified (§2.1-§6.3)

---

## Risk Assessment

### Risk 1: Wasmtime API Complexity

**Probability**: Medium  
**Impact**: Medium  
**Mitigation**:
- Follow Wasmtime documentation examples closely
- Start with minimal API surface (YAGNI)
- Add complexity incrementally in future phases
- Comprehensive error handling for Wasmtime errors

### Risk 2: Component Model Maturity

**Probability**: Low  
**Impact**: High  
**Mitigation**:
- Component Model v1.0 is stable
- Wasmtime v24.0 has mature Component Model support
- Pin version initially, upgrade carefully
- Test with real-world component fixtures

### Risk 3: Performance Target (10ms)

**Probability**: Medium  
**Impact**: Low  
**Mitigation**:
- 10ms is "ballpark acceptable" not hard requirement
- JIT compilation overhead is expected
- Module caching can be added in Phase 2 if needed
- Measure actual performance, optimize if proven necessary

### Risk 4: Test Fixture Complexity

**Probability**: Low  
**Impact**: Low  
**Mitigation**:
- Start with minimal WAT fixtures
- Use `wasm-tools` for compilation (standard tooling)
- Document fixture build process clearly

### Risk 5: Integration with Future Phases

**Probability**: Low  
**Impact**: Medium  
**Mitigation**:
- Module structure follows KNOWLEDGE-WASM-012
- Clear extension points for Phase 2
- Error types include placeholder variants
- Public API designed for future expansion

---

## Next Steps After Phase 1

### Phase 2: Memory Management and Sandboxing (Week 2-3)

**Prerequisites**: Phase 1 complete ✅

**Tasks**:
- Task 2.1: Linear Memory Limit Enforcement
- Task 2.2: Component.toml Memory Configuration
- Task 2.3: Memory Isolation Verification

**New Modules**:
- `runtime/limits.rs` - ResourceLimits struct
- `runtime/instance.rs` - WasmInstance management

---

## References

### Primary Documents

- **ADR-WASM-002**: WASM Runtime Engine Selection (PRIMARY)
- **KNOWLEDGE-WASM-012**: Module Structure Architecture (MANDATORY)
- **Workspace Standards**: `.copilot/memory_bank/workspace/shared_patterns.md` (§2.1-§6.3)
- **Microsoft Rust Guidelines**: `.copilot/memory_bank/workspace/microsoft_rust_guidelines.md`

### External References

- [Wasmtime Documentation](https://docs.wasmtime.dev/)
- [WebAssembly Component Model](https://github.com/WebAssembly/component-model)
- [WIT Language Guide](https://component-model.bytecodealliance.org/design/wit.html)
- [wasm-tools Documentation](https://github.com/bytecodealliance/wasm-tools)

---

## Document Status

**Status**: ✅ **PLANNING COMPLETE - APPROVED FOR IMPLEMENTATION**

**Revision**: Removed CI/CD integration per user request (2025-10-23)

**Next Action**: Ready to begin Day 1 implementation

**Estimated Implementation Time**: 6 days (as outlined in day-by-day timeline)

---

**End of Implementation Plan Document**
