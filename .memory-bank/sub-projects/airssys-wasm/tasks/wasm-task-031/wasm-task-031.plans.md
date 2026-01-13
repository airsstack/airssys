# WASM-TASK-031: Implementation Plans

## Plan References
- **PROJECTS_STANDARD.md:** AirsSys Workspace Shared Patterns (MANDATORY POLICY)
- **ADR-WASM-030:** Runtime Module Design (primary specification)
- **ADR-WASM-025:** Clean-Slate Rebuild Architecture
- **ADR-WASM-026:** Implementation Roadmap (Phase 5)
- **ADR-WASM-002:** WASM Runtime Engine Selection
- **ADR-WASM-023:** Module Boundary Enforcement (foundation patterns)
- **KNOWLEDGE-WASM-027:** Duplicate WASM Runtime Fatal Architecture Violation
- **KNOWLEDGE-WASM-037:** Rebuild Architecture Clean-Slate

## Mandatory Standards Compliance

### PROJECTS_STANDARD.md Requirements
This implementation MUST comply with the following mandatory standards:

#### §2.1 3-Layer Import Organization
- ✅ Layer 1: Standard library imports (`std::*`)
- ✅ Layer 2: Third-party crate imports (`wasmtime::*`)
- ✅ Layer 3: Internal module imports (`crate::core::*`)

#### §2.2 No Fully Qualified Names (FQN) in Type Annotations
- ✅ All types imported at top of file
- ❌ No `std::path::PathBuf` in type annotations
- ✅ Use simple names: `PathBuf`, `HashMap`, `ComponentHandle`

#### §4.3 Module Architecture Patterns
- ✅ `mod.rs` contains ONLY module declarations
- ✅ NO type re-exports in `mod.rs`
- ✅ NO glob re-exports (`pub use config::*`)

#### Error Handling Strategy
- ✅ Use `WasmError` from foundation (uses `thiserror::Error`)
- ✅ Implement `From<wasmtime::Error>` for error conversion
- ✅ Include contextual information in all errors

#### Testing Patterns (MANDATORY)
- ✅ Unit tests for all public functions
- ✅ **Integration tests for component interactions using REAL WASM binaries**
- Property-based testing for complex algorithms (future)

#### §6.1 YAGNI Principles
- ✅ Implement only required functionality
- ✅ Placeholder methods for future tasks (WASM-TASK-033, WASM-TASK-034)

#### §6.2 Avoid `dyn` Patterns
- ✅ Use generic constraints and static dispatch
- ❌ No trait objects (`Box<dyn Trait>`)

#### §6.4 Implementation Quality Gates
- ✅ Zero compiler warnings
- ✅ Zero clippy warnings
- ✅ **Comprehensive tests (>90% code coverage with REAL WASM components)**
- ✅ Proper resource management

---

## Target Structure Reference

Per ADR-WASM-030:
```
runtime/
├── mod.rs           # Module declarations
├── engine.rs        # WasmtimeEngine (RuntimeEngine impl) ← THIS TASK
├── loader.rs        # ComponentLoader (WASM-TASK-032)
├── store.rs         # StoreManager (WASM-TASK-033)
├── host_fn.rs       # Host function bindings (WASM-TASK-034)
└── limiter.rs       # ResourceLimiter (WASM-TASK-035)

tests/
├── engine-integration-tests.rs  # Integration tests ← THIS TASK

tests/fixtures/
├── minimal-component.wasm       # Valid minimal WASM component ← THIS TASK
└── minimal-component.wit        # WIT interface definition ← THIS TASK
```

---

## Implementation Actions

### Action 1: Create `runtime/mod.rs`

**Objective:** Module declarations following §4.3 pattern (PROJECTS_STANDARD.md)

**File:** `airssys-wasm/src/runtime/mod.rs`

**Standards Compliance:**
- PROJECTS_STANDARD.md §4.3: Only module declarations, NO implementation
- PROJECTS_STANDARD.md §2.1: 3-Layer import organization

```rust
//! # Runtime Module
//!
//! WASM component execution using wasmtime Component Model.
//!
//! ## Layer 2B - Runtime Layer
//!
//! Import Rules (ADR-WASM-023):
//! - ✅ Can import: `core/`, `security/`
//! - ❌ Cannot import: `component/`, `messaging/`, `system/`
//!
//! ## Submodules
//!
//! - [`engine`] - WasmtimeEngine (RuntimeEngine implementation)

pub mod engine;
// Future submodules:
// pub mod loader;
// pub mod store;
// pub mod host_fn;
// pub mod limiter;
```

---

### Action 2: Create `runtime/engine.rs`

**Objective:** Implement WasmtimeEngine with RuntimeEngine trait

**File:** `airssys-wasm/src/runtime/engine.rs`

**Standards Compliance:**
- PROJECTS_STANDARD.md §2.1: 3-Layer import organization
- PROJECTS_STANDARD.md §2.2: No FQN in type annotations
- PROJECTS_STANDARD.md Error Handling: Use `thiserror` patterns
- PROJECTS_STANDARD.md Testing Patterns: Unit tests for all public functions
- PROJECTS_STANDARD.md §6.4: Zero warnings, comprehensive tests

**Specification (ADR-WASM-030 lines 59-193):**

```rust
//! WasmtimeEngine implementation using wasmtime Component Model.

// Layer 1: Standard library imports
use std::collections::HashMap;
use std::sync::RwLock;

// Layer 2: Third-party crate imports
use wasmtime::component::{Component, Linker, ResourceTable};
use wasmtime::{Config, Engine, Store};

// Layer 3: Internal module imports
use crate::core::component::handle::ComponentHandle;
use crate::core::component::id::ComponentId;
use crate::core::component::message::ComponentMessage;
use crate::core::component::MessagePayload;
use crate::core::runtime::traits::RuntimeEngine;
use crate::core::runtime::errors::WasmError;

/// Convert wasmtime errors to WasmError
/// 
/// Follows PROJECTS_STANDARD.md Error Handling Strategy:
/// - Implement From traits for error conversion
impl From<wasmtime::Error> for WasmError {
    fn from(e: wasmtime::Error) -> Self {
        // Map wasmtime errors to appropriate WasmError variants
        WasmError::RuntimeError(e.to_string())
    }
}

/// Host state passed to WASM components
pub struct HostState {
    /// The component ID for this instance
    pub component_id: ComponentId,
    /// Resource table for component resources
    pub resource_table: ResourceTable,
}

/// Internal store wrapper (placeholder until StoreManager is implemented)
struct StoreEntry {
    store: Store<HostState>,
    component: Component,
}

/// WASM runtime engine using wasmtime Component Model
pub struct WasmtimeEngine {
    engine: Engine,
    linker: Linker<HostState>,
    stores: RwLock<HashMap<u64, StoreEntry>>,
    next_handle_id: RwLock<u64>,
}

impl WasmtimeEngine {
    /// Create a new WasmtimeEngine
    pub fn new() -> Result<Self, WasmError> {
        let mut config = Config::new();
        config.wasm_component_model(true);
        config.async_support(true);
        config.consume_fuel(true);

        let engine = Engine::new(&config)
            .map_err(|e| WasmError::InstantiationFailed(e.to_string()))?;

        let linker = Linker::new(&engine);

        Ok(Self {
            engine,
            linker,
            stores: RwLock::new(HashMap::new()),
            next_handle_id: RwLock::new(1),
        })
    }

    /// Get the wasmtime Engine
    pub fn engine(&self) -> &Engine {
        &self.engine
    }

    /// Get the linker
    pub fn linker(&self) -> &Linker<HostState> {
        &self.linker
    }

    /// Get mutable linker for host function registration
    pub fn linker_mut(&mut self) -> &mut Linker<HostState> {
        &mut self.linker
    }

    fn allocate_handle_id(&self) -> u64 {
        let mut id = self.next_handle_id.write().unwrap();
        let current = *id;
        *id += 1;
        current
    }
}

impl RuntimeEngine for WasmtimeEngine {
    fn load_component(
        &self,
        id: &ComponentId,
        bytes: &[u8],
    ) -> Result<ComponentHandle, WasmError> {
        // Parse component from bytes
        let component = Component::from_binary(&self.engine, bytes)
            .map_err(|e| WasmError::InstantiationFailed(e.to_string()))?;

        // Create store with host state
        let host_state = HostState {
            component_id: id.clone(),
            resource_table: ResourceTable::new(),
        };

        let store = Store::new(&self.engine, host_state);
        let handle_id = self.allocate_handle_id();

        // Store the entry
        let entry = StoreEntry { store, component };
        {
            let mut stores = self.stores.write().unwrap();
            stores.insert(handle_id, entry);
        }

        Ok(ComponentHandle::new(id.clone(), handle_id))
    }

    fn unload_component(&self, handle: &ComponentHandle) -> Result<(), WasmError> {
        let mut stores = self.stores.write().unwrap();
        stores.remove(&handle.handle_id());
        Ok(())
    }

    fn call_handle_message(
        &self,
        handle: &ComponentHandle,
        _msg: &ComponentMessage,
    ) -> Result<Option<MessagePayload>, WasmError> {
        let stores = self.stores.read().unwrap();
        
        let _entry = stores.get(&handle.handle_id()).ok_or_else(|| {
            WasmError::ComponentNotFound(handle.id().to_string())
        })?;

        // Placeholder - actual implementation requires wit-bindgen integration
        // Will be implemented in WASM-TASK-033 (StoreManager)
        Ok(None)
    }

    fn call_handle_callback(
        &self,
        handle: &ComponentHandle,
        _msg: &ComponentMessage,
    ) -> Result<(), WasmError> {
        let stores = self.stores.read().unwrap();
        
        let _entry = stores.get(&handle.handle_id()).ok_or_else(|| {
            WasmError::ComponentNotFound(handle.id().to_string())
        })?;

        // Placeholder - actual implementation requires wit-bindgen integration
        // Will be implemented in WASM-TASK-033 (StoreManager)
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_wasmtime_engine_creation() {
        let engine = WasmtimeEngine::new();
        assert!(engine.is_ok());
    }

    #[test]
    fn test_engine_config() {
        let engine = WasmtimeEngine::new().unwrap();
        // Verify engine is accessible
        let _ = engine.engine();
    }

    #[test]
    fn test_linker_accessible() {
        let engine = WasmtimeEngine::new().unwrap();
        let _ = engine.linker();
    }

    #[test]
    fn test_handle_id_allocation() {
        let engine = WasmtimeEngine::new().unwrap();
        let id1 = engine.allocate_handle_id();
        let id2 = engine.allocate_handle_id();
        assert_eq!(id1, 1);
        assert_eq!(id2, 2);
    }

    #[test]
    fn test_unload_nonexistent_component() {
        let engine = WasmtimeEngine::new().unwrap();
        let component_id = ComponentId::new("test", "comp", "0");
        let handle = ComponentHandle::new(component_id, 999);
        // Should succeed even if component doesn't exist
        let result = engine.unload_component(&handle);
        assert!(result.is_ok());
    }

    #[test]
    fn test_call_message_component_not_found() {
        let engine = WasmtimeEngine::new().unwrap();
        let component_id = ComponentId::new("test", "comp", "0");
        let handle = ComponentHandle::new(component_id.clone(), 999);
        
        let msg = ComponentMessage::new(
            component_id.clone(),
            component_id,
            "test".to_string(),
            MessagePayload::empty(),
        );
        
        let result = engine.call_handle_message(&handle, &msg);
        assert!(matches!(result, Err(WasmError::ComponentNotFound(_))));
    }

    #[test]
    fn test_wasm_error_display() {
        let err = WasmError::InstantiationFailed("test error".to_string());
        assert!(err.to_string().contains("Instantiation failed"));
        
        let err = WasmError::ComponentNotFound("comp-1".to_string());
        assert!(err.to_string().contains("Component not found"));
    }
}
```

---

### Action 3: Create Test Fixtures (REAL WASM Components)

**Objective:** Create valid WASM Component Model binaries for integration testing

**Files:**
- `airssys-wasm/tests/fixtures/minimal-component.wit` - WIT interface definition
- `airssys-wasm/tests/fixtures/minimal-component.wasm` - Valid WASM component binary

**Standards Compliance:**
- PROJECTS_STANDARD.md Testing Patterns: Integration tests with REAL data
- PROJECTS_STANDARD.md §6.1: Minimal viable components (YAGNI)

**Step 3.1: Create WIT Interface**

**File:** `airssys-wasm/tests/fixtures/minimal-component.wit`

```wit
/// Minimal WASM component for testing component loading
/// 
/// This component has no exports (empty interface) to test basic
/// component loading and instantiation without requiring host functions
/// or complex WIT features.
///
/// Used for integration testing of WasmtimeEngine load_component()
```

**Step 3.2: Generate WASM Component Binary**

**Instructions:** Use `wasm-tools` to compile minimal WIT to WASM component

```bash
# Install wasm-tools if not already installed
# cargo install wasm-tools

# Navigate to fixtures directory
cd tests/fixtures

# Create minimal component from WIT
# The component has no functions, just validates loading
wasm-tools component new minimal-component.wit --out minimal-component.wasm

# Verify the component is valid
wasm-tools validate minimal-component.wasm

# Verify it's a component (not a module)
wasm-tools print minimal-component.wasm | head -20
# Should show: (component <name>)
```

**Expected minimal-component.wasm Properties:**
- ✅ Valid WASM Component Model binary
- ✅ No functions (empty interface)
- ✅ Can be loaded by wasmtime
- ✅ No host function imports
- ✅ Minimal size (< 1KB)
- ✅ Validates component model support

**Verification Commands:**

```bash
# Verify WASM binary exists and is not empty
ls -lh tests/fixtures/minimal-component.wasm
# Expected: > 100 bytes, valid binary

# Verify it's a valid component
wasm-tools validate tests/fixtures/minimal-component.wasm
# Expected: No errors

# Check it's a component (not module)
wasm-tools print tests/fixtures/minimal-component.wasm | grep "(component"
# Expected: Found (component ...) in output

# Display component structure
wasm-tools print tests/fixtures/minimal-component.wasm
# Should show component structure with empty interface
```

---

### Action 4: Create Integration Tests with REAL WASM Components

**Objective:** Integration tests using REAL WASM component binaries (PROJECTS_STANDARD.md Testing Patterns)

**File:** `airssys-wasm/tests/engine-integration-tests.rs`

**Standards Compliance:**
- PROJECTS_STANDARD.md Testing Patterns: Integration tests for component interactions
- PROJECTS_STANDARD.md Testing Patterns: **Must use REAL WASM components, not placeholders**
- PROJECTS_STANDARD.md §2.1: 3-Layer import organization
- PROJECTS_STANDARD.md Error Handling: Error propagation across layers
- PROJECTS_STANDARD.md §6.4: >90% code coverage

```rust
//! Integration tests for WasmtimeEngine.
//!
//! Tests end-to-end component lifecycle and interaction with foundation types
//! using REAL WASM Component Model binaries (not placeholders).

// Layer 1: Standard library imports
use std::path::Path;

// Layer 2: Third-party crate imports
// None needed for integration tests

// Layer 3: Internal module imports
use airssys_wasm::core::component::handle::ComponentHandle;
use airssys_wasm::core::component::id::ComponentId;
use airssys_wasm::core::component::message::ComponentMessage;
use airssys_wasm::core::component::MessagePayload;
use airssys_wasm::core::runtime::errors::WasmError;
use airssys_wasm::runtime::engine::WasmtimeEngine;
use airssys_wasm::runtime::engine::HostState;

/// Load a real WASM component binary from fixtures
fn load_fixture_wasm(name: &str) -> Vec<u8> {
    let fixture_path = Path::new("tests/fixtures")
        .join(name)
        .with_extension("wasm");
    
    std::fs::read(&fixture_path)
        .expect(&format!("Failed to read fixture: {}", fixture_path.display()))
}

#[test]
fn test_load_real_wasm_component_success() {
    // Integration test: Load a REAL, valid WASM component
    let engine = WasmtimeEngine::new().expect("Engine creation should succeed");
    
    let component_id = ComponentId::new("test-org", "test-comp", "0.1.0");
    
    // Load REAL WASM component from fixtures
    let wasm_bytes = load_fixture_wasm("minimal-component");
    
    // Load component with REAL WASM binary
    let handle = engine.load_component(&component_id, &wasm_bytes);
    
    // Should succeed with valid WASM component
    assert!(handle.is_ok(), "Valid WASM component should load successfully");
    
    let handle = handle.unwrap();
    assert_eq!(handle.id(), &component_id);
    assert!(handle.handle_id() > 0);
}

#[test]
fn test_load_invalid_wasm_bytes_failure() {
    // Integration test: Invalid WASM bytes should fail appropriately
    let engine = WasmtimeEngine::new().expect("Engine creation should succeed");
    
    let component_id = ComponentId::new("test", "invalid", "0");
    
    // Invalid bytes (not a valid WASM component)
    let invalid_bytes = vec![0xFF, 0xFF, 0xFF, 0xFF, 0xFF];
    
    let result = engine.load_component(&component_id, &invalid_bytes);
    
    // Should fail with InstantiationFailed
    assert!(result.is_err());
    assert!(matches!(result, Err(WasmError::InstantiationFailed(_))));
}

#[test]
fn test_multiple_real_components_simultaneous() {
    // Integration test: Load multiple REAL WASM components simultaneously
    let engine = WasmtimeEngine::new().expect("Engine creation should succeed");
    
    let component_id_1 = ComponentId::new("org1", "comp1", "1.0.0");
    let component_id_2 = ComponentId::new("org2", "comp2", "1.0.0");
    
    // Load REAL WASM component twice
    let wasm_bytes = load_fixture_wasm("minimal-component");
    
    let handle_1 = engine.load_component(&component_id_1, &wasm_bytes);
    let handle_2 = engine.load_component(&component_id_2, &wasm_bytes);
    
    // Both should succeed with same component binary
    assert!(handle_1.is_ok());
    assert!(handle_2.is_ok());
    
    let handle_1 = handle_1.unwrap();
    let handle_2 = handle_2.unwrap();
    
    // Should have different handle IDs
    assert_ne!(handle_1.handle_id(), handle_2.handle_id());
}

#[test]
fn test_component_lifecycle_real_wasm() {
    // Integration test: Complete lifecycle with REAL WASM (load → verify → unload)
    let engine = WasmtimeEngine::new().expect("Engine creation should succeed");
    
    let component_id = ComponentId::new("test-org", "lifecycle-test", "1.0.0");
    
    // Load REAL WASM component
    let wasm_bytes = load_fixture_wasm("minimal-component");
    let handle = engine.load_component(&component_id, &wasm_bytes)
        .expect("Component should load");
    
    // Verify handle
    assert_eq!(handle.id(), &component_id);
    assert!(handle.handle_id() > 0);
    
    // Unload component
    let unload_result = engine.unload_component(&handle);
    assert!(unload_result.is_ok());
}

#[test]
fn test_handle_persistence_with_real_wasm() {
    // Integration test: ComponentHandle correctly stores IDs with REAL WASM
    let component_id = ComponentId::new("test-org", "test-comp", "1.0.0");
    let handle = ComponentHandle::new(component_id.clone(), 12345);
    
    assert_eq!(handle.id(), &component_id);
    assert_eq!(handle.handle_id(), 12345);
}

#[test]
fn test_error_propagation_from_real_wasmtime() {
    // Integration test: Error propagation with REAL WASM and invalid data
    let engine = WasmtimeEngine::new().expect("Engine creation should succeed");
    
    let component_id = ComponentId::new("test", "corrupted", "0");
    
    // Corrupted WASM bytes (partial header)
    let corrupted_bytes = vec![0x00, 0x61, 0x73]; // Incomplete magic number
    
    let result = engine.load_component(&component_id, &corrupted_bytes);
    
    // Should fail with WasmError::InstantiationFailed
    assert!(result.is_err());
    assert!(matches!(result, Err(WasmError::InstantiationFailed(_))));
    
    // Error message should indicate parsing failure
    if let Err(WasmError::InstantiationFailed(msg)) = result {
        assert!(msg.len() > 0, "Error message should not be empty");
    }
}

#[test]
fn test_message_creation_with_foundation_types() {
    // Integration test: ComponentMessage works with foundation types
    let from_id = ComponentId::new("org1", "comp1", "1.0.0");
    let to_id = ComponentId::new("org2", "comp2", "1.0.0");
    
    let payload = MessagePayload::from_json("{\"action\":\"test\"}").expect("Valid JSON");
    
    let msg = ComponentMessage::new(from_id.clone(), to_id.clone(), "test-action".to_string(), payload);
    
    assert_eq!(msg.from(), &from_id);
    assert_eq!(msg.to(), &to_id);
    assert_eq!(msg.message_type(), "test-action");
}

#[test]
fn test_host_state_initialization() {
    // Integration test: HostState is correctly initialized with component ID
    let component_id = ComponentId::new("test", "host-state", "1.0.0");
    let host_state = HostState {
        component_id: component_id.clone(),
        resource_table: wasmtime::component::ResourceTable::new(),
    };
    
    assert_eq!(host_state.component_id, component_id);
}

#[test]
fn test_engine_linker_mutability() {
    // Integration test: Linker can be mutated for host function registration
    let mut engine = WasmtimeEngine::new().expect("Engine creation should succeed");
    
    // Test that linker_mut() works (actual host function registration is WASM-TASK-034)
    let _linker = engine.linker_mut();
    
    // Should not panic or return null
}

#[test]
fn test_component_unload_idempotent() {
    // Integration test: Unloading component is idempotent
    let engine = WasmtimeEngine::new().expect("Engine creation should succeed");
    
    let component_id = ComponentId::new("test", "nonexistent", "0");
    let handle = ComponentHandle::new(component_id, 99999);
    
    // First unload (component doesn't exist)
    let result1 = engine.unload_component(&handle);
    assert!(result1.is_ok());
    
    // Second unload (still doesn't exist) - should also succeed
    let result2 = engine.unload_component(&handle);
    assert!(result2.is_ok());
}

#[test]
fn test_real_wasm_component_reusability() {
    // Integration test: Same WASM binary can be loaded multiple times
    let engine = WasmtimeEngine::new().expect("Engine creation should succeed");
    
    let component_id_1 = ComponentId::new("org", "comp", "1.0.0");
    let component_id_2 = ComponentId::new("org", "comp", "1.0.0");
    
    // Load REAL WASM component twice with same ID
    let wasm_bytes = load_fixture_wasm("minimal-component");
    
    let handle_1 = engine.load_component(&component_id_1, &wasm_bytes)
        .expect("First load should succeed");
    
    // Load same component again (should create new instance)
    let handle_2 = engine.load_component(&component_id_2, &wasm_bytes)
        .expect("Second load should succeed");
    
    // Both should succeed with different handle IDs
    assert_ne!(handle_1.handle_id(), handle_2.handle_id());
}
```

---

## Implementation Acceptance Criteria

These criteria must be met for implementation to be considered complete. Each item represents a deliverable that must be implemented and verified.

### Functional Requirements
- [ ] WasmtimeEngine struct implemented
- [ ] HostState struct implemented
- [ ] RuntimeEngine trait implemented
- [ ] Engine config uses component_model, async, fuel
- [ ] All unit tests pass (7+ unit tests)
- [ ] All integration tests pass (11+ integration tests)
- [ ] Integration tests use REAL WASM components (not placeholders)
- [ ] Combined test coverage >90%

### Standards Compliance
- [ ] Uses WasmError from core/runtime/errors (NOT RuntimeError)
- [ ] wasmtime → WasmError conversion strategy defined
- [ ] Build passes with zero warnings (PROJECTS_STANDARD.md §6.4)
- [ ] Clippy passes with zero warnings (PROJECTS_STANDARD.md §6.4)
- [ ] No FQN in type annotations (PROJECTS_STANDARD.md §2.2)
- [ ] 3-Layer import organization (PROJECTS_STANDARD.md §2.1)
- [ ] mod.rs contains only declarations (PROJECTS_STANDARD.md §4.3)
- [ ] No RuntimeError type defined (foundation pattern)
- [ ] Only imports from core/, security/ (ADR-WASM-023)
- [ ] Integration tests for component interactions (PROJECTS_STANDARD.md Testing Patterns)
- [ ] Integration tests use REAL WASM binaries (PROJECTS_STANDARD.md Testing Patterns)

---

## Implementation Dependencies

This section documents dependencies on other tasks to avoid misinterpretation of placeholder methods and future work.

### Dependencies on Other Tasks

**WASM-TASK-033 (StoreManager)**
- **Required for:** Full implementation of `call_handle_message()` and `call_handle_callback()` methods
- **Current Scope:** Placeholder methods that return `Ok(None)` and `Ok(())`
- **Future Work:** When WASM-TASK-033 is implemented, these methods will:
  - Load and instantiate WASM exports (handle-message, handle-callback)
  - Execute WASM functions with message payloads
  - Handle message serialization/deserialization via wit-bindgen
  - Process function results and return to caller

**WASM-TASK-034 (Host Functions)**
- **Required for:** Actual host function registration via `linker_mut()` method
- **Current Scope:** Framework provided (linker can be mutated)
- **Future Work:** When WASM-TASK-034 is implemented, `linker_mut()` will be used to:
  - Register host functions for WASM components to call
  - Define security policies for host function access
  - Implement capability validation for host function invocation

### Current Task Scope (WASM-TASK-031)

This task implements the foundational WasmtimeEngine infrastructure:

**Foundation Components:**
- ✅ WasmtimeEngine struct with wasmtime Engine, Linker, Store management
- ✅ HostState struct for component-specific state
- ✅ RuntimeEngine trait implementation (foundation methods)
- ✅ wasmtime → WasmError error conversion
- ✅ Handle ID allocation and management

**Placeholder Methods (Intentional YAGNI):**
- ✅ `call_handle_message()` returns `Ok(None)` (placeholder)
- ✅ `call_handle_callback()` returns `Ok(())` (placeholder)
- ✅ Placeholders documented with references to future tasks

**Rationale for Placeholders:**
- Per PROJECTS_STANDARD.md §6.1 YAGNI Principles, we implement only what's needed now
- Full message execution requires wit-bindgen integration (WASM-TASK-033)
- Host function registration requires security policy integration (WASM-TASK-034)
- Foundation infrastructure (loading, unloading, handle management) is sufficient for Phase 5

### Task Execution Order

**Recommended Order:**
1. **WASM-TASK-031** (This task) - Foundation engine and component loading
2. **WASM-TASK-033** - StoreManager for message execution
3. **WASM-TASK-034** - Host function registration and security

**Dependencies:**
- WASM-TASK-033 depends on WASM-TASK-031 completion
- WASM-TASK-034 depends on WASM-TASK-031 completion
- WASM-TASK-033 and WASM-TASK-034 can be implemented in parallel (after WASM-TASK-031)

---

## Verification Commands

```bash
# 1. Build check (PROJECTS_STANDARD.md §6.4: Zero warnings)
cargo build -p airssys-wasm

# 2. Lint check (PROJECTS_STANDARD.md §6.4: Zero clippy warnings)
cargo clippy -p airssys-wasm --all-targets -- -D warnings

# 3. Run unit tests (PROJECTS_STANDARD.md Testing Patterns)
cargo test -p airssys-wasm --lib runtime::engine

# 4. Run integration tests (PROJECTS_STANDARD.md Testing Patterns)
cargo test -p airssys-wasm --test engine-integration-tests

# 5. Run ALL tests (PROJECTS_STANDARD.md §6.4: Comprehensive tests)
cargo test -p airssys-wasm

# 6. Module boundary check (ADR-WASM-023)
grep -rn "use crate::component\|use crate::messaging\|use crate::system" src/runtime/
# Should return empty (no forbidden imports)

# 7. Verify WasmError is used (not RuntimeError)
grep -rn "RuntimeError" src/runtime/engine.rs
# Should return empty (RuntimeError should NOT exist)

# 8. Verify WasmError import exists
grep -rn "use crate::core::runtime::errors::WasmError" src/runtime/engine.rs
# Should return found

# 9. Verify thiserror pattern
grep -rn "#\[error(" src/core/runtime/errors.rs
# Should return found (foundation uses thiserror)

# 10. Check for FQN usage in type annotations (PROJECTS_STANDARD.md §2.2)
grep -rnE "struct\s+\w+\s*\{[^}]*std::::" src/runtime/engine.rs
grep -rnE "fn\s+\w+\([^)]*:std::::" src/runtime/engine.rs
grep -rnE "->\s*Result<std::::|->\s*std::::" src/runtime/engine.rs
# Should return empty (no FQN in type annotations)

# 11. Verify 3-Layer import organization (PROJECTS_STANDARD.md §2.1)
grep -A5 "^// Layer" src/runtime/engine.rs | grep -E "^(//|use)" | head -10
# Should show 3-layer pattern

# 12. Verify mod.rs has only declarations (PROJECTS_STANDARD.md §4.3)
grep -v "^\s*//" src/runtime/mod.rs | grep -v "^$" | grep -v "^pub mod" | grep -v "^// Future"
# Should return empty (no implementation in mod.rs)

# 13. Check test fixtures exist (PROJECTS_STANDARD.md Testing Patterns)
ls -lh tests/fixtures/minimal-component.wasm tests/fixtures/minimal-component.wit
# Both files should exist

# 14. Verify WASM component is valid (PROJECTS_STANDARD.md Testing Patterns)
wasm-tools validate tests/fixtures/minimal-component.wasm
# Should succeed with no errors

# 15. Verify WASM component format (PROJECTS_STANDARD.md Testing Patterns)
wasm-tools print tests/fixtures/minimal-component.wasm | grep "(component"
# Should show component structure

# 16. Check test coverage (PROJECTS_STANDARD.md §6.4: >90% coverage)
# Requires cargo-tarpaulin or similar tool
cargo tarpaulin --out Html --output-dir coverage/
# Should show >90% coverage for runtime/engine.rs
```

---

## Critical Notes

> ⚠️ **MANDATORY**: Use `wasmtime::component::Component`, NOT `wasmtime::Module`.
> Per KNOWLEDGE-WASM-027, using core WASM Module API instead of Component Model is a fatal architecture violation.

> ⚠️ **MANDATORY**: Use `WasmError` from `core/runtime/errors`, NOT define custom `RuntimeError`.
> Per ADR-WASM-023 and ADR-WASM-028, foundation error types must be reused. Custom error types violate "single source of truth" principle.

> ⚠️ **MANDATORY**: Follow PROJECTS_STANDARD.md for ALL implementation patterns.
> This is workspace-wide mandatory policy covering:
> - §2.1: 3-Layer import organization
> - §2.2: No FQN in type annotations
> - §4.3: Module architecture patterns
> - Error Handling: Use `thiserror` patterns
> - Testing Patterns: Unit tests AND integration tests (MANDATORY)
> - **Testing Patterns: Integration tests MUST use REAL WASM binaries, NOT placeholders**
> - §6.1: YAGNI principles
> - §6.2: Avoid `dyn` patterns
> - §6.4: Implementation quality gates (>90% test coverage)

> **REAL WASM Components**: Per PROJECTS_STANDARD.md Testing Patterns, integration tests MUST use REAL WASM Component Model binaries.
> 
> **FIXTURES REQUIRED:**
> - `tests/fixtures/minimal-component.wit` - WIT interface definition
> - `tests/fixtures/minimal-component.wasm` - Valid WASM component binary
> 
> **GENERATION:**
> ```bash
> wasm-tools component new tests/fixtures/minimal-component.wit --out tests/fixtures/minimal-component.wasm
> wasm-tools validate tests/fixtures/minimal-component.wasm
> wasm-tools print tests/fixtures/minimal-component.wasm | grep "(component"
> ```
> 
> **TESTS USING REAL WASM:**
> - `test_load_real_wasm_component_success` - Load valid component
> - `test_multiple_real_components_simultaneous` - Multiple instances
> - `test_component_lifecycle_real_wasm` - Complete lifecycle
> - `test_real_wasm_component_reusability` - Same binary, multiple loads
> - `test_load_invalid_wasm_bytes_failure` - Error handling (invalid bytes)
> - `test_error_propagation_from_real_wasmtime` - Corrupted data handling

> **Placeholder Methods**: `call_handle_message` and `call_handle_callback` are placeholders.
> Full implementation requires wit-bindgen integration in WASM-TASK-033 (StoreManager).
> See "Implementation Dependencies" section below for detailed explanation.
> Follows PROJECTS_STANDARD.md §6.1 YAGNI principles.

> **Implementation Dependencies**: See the "Implementation Dependencies" section for complete documentation of:
> - Dependencies on WASM-TASK-033 (StoreManager) for message execution
> - Dependencies on WASM-TASK-034 (Host Functions) for host function registration
> - Rationale for placeholder methods in this task
> - Recommended task execution order

> **Error Handling**: All error returns use `WasmError` from foundation, which provides comprehensive error variants:
> - `ComponentNotFound`
> - `InstantiationFailed`
> - `ExportNotFound` (for future use when calling WASM exports)
> - `Timeout` (for future execution timeout enforcement)
> - `ResourceLimitExceeded` (for future memory/fuel limit violations)
> - `InvalidComponent` (for future WASM binary validation)
> - `RuntimeError` (generic execution errors)
> Follows PROJECTS_STANDARD.md Error Handling Strategy.
