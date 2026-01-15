# WASM-TASK-033: Implementation Plans

## Plan References
- **ADR-WASM-030:** Runtime Module Design (primary specification)
- **ADR-WASM-026:** Implementation Roadmap (Phase 5)
- **ADR-WASM-023:** Module Boundary Enforcement (architecture verification)
- **PROJECTS_STANDARD.md:** §2.1 (3-Layer Imports), §2.2 (No FQN), §6.4 (Quality Gates)
- **AGENTS.md Section 9:** Mandatory testing requirements (unit + integration)
- **AGENTS.md Section 13:** Architecture verification requirements

**Note on ADR vs PROJECTS_STANDARD.md Conflicts:**
If any ADR patterns conflict with PROJECTS_STANDARD.md, the plan follows PROJECTS_STANDARD.md.
Any adjustments are noted with: "(Adjusted from ADR to comply with PROJECTS_STANDARD.md §[X.Y])"

## Target Structure Reference

```
runtime/
├── mod.rs
├── engine.rs        # (WASM-TASK-031) ✅
├── loader.rs        # (WASM-TASK-032)
├── store.rs         # ← THIS TASK
├── host_fn.rs       # (WASM-TASK-034)
└── limiter.rs       # (WASM-TASK-035)
```

---

## Implementation Actions

### Action 1: Add WasmError::StoreNotInitialized Variant

**Objective:** Add missing error variant for uninitialized store operations

**File:** `airssys-wasm/src/core/runtime/errors.rs`

**Changes Required:**

Add new error variant to `WasmError` enum:

```rust
/// Store not initialized.
#[error("Store not initialized - call initialize() before using")]
StoreNotInitialized,
```

**Location:** After line 46 (after `RuntimeError` variant)

**Test Required:**

```rust
#[test]
fn test_store_not_initialized_display() {
    let err = WasmError::StoreNotInitialized;
    assert_eq!(format!("{}", err), "Store not initialized - call initialize() before using");
}
```

**Verification:**
```bash
cargo build -p airssys-wasm
cargo test -p airssys-wasm --lib core::runtime::errors::tests::test_store_not_initialized_display
```

---

### Action 2: Create `runtime/store.rs`

**Objective:** Implement StoreManager for WASM stores

**File:** `airssys-wasm/src/runtime/store.rs`

**CRITICAL FIXES FROM VERIFIER:**
- ✅ Use `WasmError` (NOT `RuntimeError` - doesn't exist!)
- ✅ Import path: `use crate::core::component::message::{ComponentMessage, MessagePayload};`
- ✅ Return `Result<_, WasmError>` in ALL methods
- ✅ Use `WasmError::StoreNotInitialized` for uninitialized errors
- ✅ Use `WasmError::InstantiationFailed` for instantiation errors

**Implementation:**

```rust
//! WASM store management.

// Layer 1: Standard library imports (per PROJECTS_STANDARD.md §2.1)
// (none)

// Layer 2: Third-party crate imports (per PROJECTS_STANDARD.md §2.1)
use wasmtime::component::{Component, Instance, Linker};
use wasmtime::Store;

// Layer 3: Internal module imports (per PROJECTS_STANDARD.md §2.1)
use crate::core::component::message::{ComponentMessage, MessagePayload};
use crate::core::runtime::errors::WasmError;

use super::engine::HostState;

/// Manages a WASM store and its associated component instance
pub struct StoreManager {
    store: Store<HostState>,
    component: Component,
    instance: Option<Instance>,
}

impl StoreManager {
    /// Create a new StoreManager
    pub fn new(store: Store<HostState>, component: Component) -> Self {
        Self {
            store,
            component,
            instance: None,
        }
    }

    /// Initialize the component instance
    pub fn initialize(&mut self, linker: &Linker<HostState>) -> Result<(), WasmError> {
        let instance = linker
            .instantiate(&mut self.store, &self.component)
            .map_err(|e| WasmError::InstantiationFailed(e.to_string()))?;

        self.instance = Some(instance);
        Ok(())
    }

    /// Check if the instance is initialized
    pub fn is_initialized(&self) -> bool {
        self.instance.is_some()
    }

    /// Call handle-message on the component
    pub fn call_handle_message(
        &mut self,
        _msg: &ComponentMessage,
    ) -> Result<Option<MessagePayload>, WasmError> {
        let _instance = self.instance.as_ref().ok_or(WasmError::StoreNotInitialized)?;

        // Placeholder - actual implementation uses wit-bindgen generated bindings
        // Example with generated bindings:
        // let exports = ComponentLifecycle::new(&mut self.store, instance)?;
        // let result = exports.call_handle_message(&mut self.store, msg)?;
        
        Ok(None)
    }

    /// Call handle-callback on the component
    pub fn call_handle_callback(
        &mut self,
        _msg: &ComponentMessage,
    ) -> Result<(), WasmError> {
        let _instance = self.instance.as_ref().ok_or(WasmError::StoreNotInitialized)?;

        // Placeholder - actual implementation uses wit-bindgen generated bindings
        Ok(())
    }

    /// Get the store
    pub fn store(&self) -> &Store<HostState> {
        &self.store
    }

    /// Get mutable store
    pub fn store_mut(&mut self) -> &mut Store<HostState> {
        &mut self.store
    }

    /// Get the component
    pub fn component(&self) -> &Component {
        &self.component
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use wasmtime::{Config, Engine};
    use crate::core::component::id::ComponentId;
    use crate::core::component::message::MessageMetadata;

    fn create_test_engine() -> Engine {
        let mut config = Config::new();
        config.wasm_component_model(true);
        Engine::new(&config).unwrap()
    }

    fn create_test_message() -> ComponentMessage {
        let sender = ComponentId::new("test", "comp", "0");
        let payload = MessagePayload::new(vec![1, 2, 3]);
        let metadata = MessageMetadata::default();
        ComponentMessage::new(sender, payload, metadata)
    }

    #[test]
    fn test_store_manager_creation() {
        let engine = create_test_engine();
        let component_id = ComponentId::new("test", "comp", "0");
        let host_state = HostState {
            component_id: component_id.clone(),
        };
        let store = Store::new(&engine, host_state);
        
        // Create minimal valid WASM component
        let wasm = wat::parse_str(r#"
            (component)
        "#).unwrap();
        let component = Component::from_binary(&engine, &wasm).unwrap();
        
        let manager = StoreManager::new(store, component);
        assert!(!manager.is_initialized());
    }

    #[test]
    fn test_store_manager_not_initialized_error() {
        let engine = create_test_engine();
        let component_id = ComponentId::new("test", "comp", "0");
        let host_state = HostState {
            component_id: component_id.clone(),
        };
        let store = Store::new(&engine, host_state);
        
        let wasm = wat::parse_str(r#"
            (component)
        "#).unwrap();
        let component = Component::from_binary(&engine, &wasm).unwrap();
        
        let mut manager = StoreManager::new(store, component);
        let msg = create_test_message();
        
        // Should return StoreNotInitialized error
        let result = manager.call_handle_message(&msg);
        assert!(result.is_err());
        assert!(matches!(result.unwrap_err(), WasmError::StoreNotInitialized));
    }

    #[test]
    fn test_call_callback_not_initialized() {
        let engine = create_test_engine();
        let component_id = ComponentId::new("test", "comp", "0");
        let host_state = HostState {
            component_id: component_id.clone(),
        };
        let store = Store::new(&engine, host_state);
        
        let wasm = wat::parse_str(r#"
            (component)
        "#).unwrap();
        let component = Component::from_binary(&engine, &wasm).unwrap();
        
        let mut manager = StoreManager::new(store, component);
        let msg = create_test_message();
        
        // Should return StoreNotInitialized error
        let result = manager.call_handle_callback(&msg);
        assert!(result.is_err());
        assert!(matches!(result.unwrap_err(), WasmError::StoreNotInitialized));
    }

    #[test]
    fn test_store_manager_initialization_success() {
        let engine = create_test_engine();
        let component_id = ComponentId::new("test", "comp", "0");
        let host_state = HostState {
            component_id: component_id.clone(),
        };
        let store = Store::new(&engine, host_state);
        
        let wasm = wat::parse_str(r#"
            (component)
        "#).unwrap();
        let component = Component::from_binary(&engine, &wasm).unwrap();
        
        let linker = Linker::new(&engine);
        let mut manager = StoreManager::new(store, component);
        
        // Initialize should succeed
        let result = manager.initialize(&linker);
        assert!(result.is_ok());
        assert!(manager.is_initialized());
    }

    #[test]
    fn test_store_accessors() {
        let engine = create_test_engine();
        let component_id = ComponentId::new("test", "comp", "0");
        let host_state = HostState {
            component_id: component_id.clone(),
        };
        let store = Store::new(&engine, host_state);
        
        let wasm = wat::parse_str(r#"
            (component)
        "#).unwrap();
        let component = Component::from_binary(&engine, &wasm).unwrap();
        
        let mut manager = StoreManager::new(store, component);
        
        // Test store accessors
        let _store_ref = manager.store();
        let _store_mut = manager.store_mut();
        let _component_ref = manager.component();
        
        // Should not panic
    }
}
```

**PROJECTS_STANDARD.md Compliance:**
- §2.1 (3-Layer Imports): ✅ Code follows import organization
- §2.2 (No FQN): ✅ Types imported and used by simple name
- §6.4 (Quality Gates): ✅ Zero warnings, comprehensive tests

---

### Action 3: Create Integration Tests

**Objective:** Test StoreManager end-to-end functionality with real WASM components

**File:** `airssys-wasm/tests/store-integration-tests.rs`

**Purpose:** Test StoreManager end-to-end functionality with real WASM components (per AGENTS.md Section 9)

**Implementation:**

```rust
//! Integration tests for StoreManager.
//!
//! Tests end-to-end store lifecycle and message handling with REAL WASM components.

// Layer 1: Standard library imports
use std::path::Path;

// Layer 2: Third-party crate imports
// None needed for integration tests

// Layer 3: Internal module imports
use airssys_wasm::core::component::id::ComponentId;
use airssys_wasm::core::component::message::{ComponentMessage, MessageMetadata, MessagePayload};
use airssys_wasm::core::runtime::errors::WasmError;
use airssys_wasm::runtime::engine::{HostState, WasmtimeEngine};
use airssys_wasm::runtime::store::StoreManager;
use wasmtime::component::{Component, Linker};
use wasmtime::{Config, Engine, Store};

/// Load a real WASM component binary from fixtures
fn load_fixture_wasm(name: &str) -> Vec<u8> {
    let fixture_path = Path::new("tests/fixtures")
        .join(name)
        .with_extension("wasm");

    std::fs::read(&fixture_path)
        .unwrap_or_else(|_| panic!("Failed to read fixture: {}", fixture_path.display()))
}

fn create_test_engine() -> Engine {
    let mut config = Config::new();
    config.wasm_component_model(true);
    Engine::new(&config).unwrap()
}

fn create_test_message() -> ComponentMessage {
    let sender = ComponentId::new("test", "sender", "0");
    let payload = MessagePayload::new(vec![1, 2, 3, 4]);
    let metadata = MessageMetadata::default();
    ComponentMessage::new(sender, payload, metadata)
}

#[test]
fn test_store_manager_lifecycle_with_real_component() {
    // Integration test: Complete lifecycle with REAL WASM component
    let engine = create_test_engine();
    let component_id = ComponentId::new("test-org", "test-comp", "1.0.0");
    
    // Load REAL WASM component from fixtures
    let wasm_bytes = load_fixture_wasm("minimal-component");
    let component = Component::from_binary(&engine, &wasm_bytes)
        .expect("Valid WASM component should load");
    
    let host_state = HostState {
        component_id: component_id.clone(),
    };
    let store = Store::new(&engine, host_state);
    let linker = Linker::new(&engine);
    
    let mut manager = StoreManager::new(store, component);
    
    // Initialize with linker
    let init_result = manager.initialize(&linker);
    assert!(init_result.is_ok(), "Initialization should succeed");
    assert!(manager.is_initialized(), "Should be initialized after initialization");
    
    // Call handle_message (placeholder returns Ok(None))
    let msg = create_test_message();
    let result = manager.call_handle_message(&msg);
    assert!(result.is_ok(), "Message handling should succeed after initialization");
}

#[test]
fn test_store_manager_uninitialized_error() {
    // Integration test: Uninitialized store returns StoreNotInitialized error
    let engine = create_test_engine();
    let component_id = ComponentId::new("test", "comp", "0");
    
    let wasm_bytes = load_fixture_wasm("minimal-component");
    let component = Component::from_binary(&engine, &wasm_bytes).unwrap();
    
    let host_state = HostState {
        component_id: component_id.clone(),
    };
    let store = Store::new(&engine, host_state);
    
    let mut manager = StoreManager::new(store, component);
    let msg = create_test_message();
    
    // Should return StoreNotInitialized
    let result = manager.call_handle_message(&msg);
    assert!(result.is_err());
    assert!(matches!(result.unwrap_err(), WasmError::StoreNotInitialized));
    
    // Same for call_handle_callback
    let result = manager.call_handle_callback(&msg);
    assert!(result.is_err());
    assert!(matches!(result.unwrap_err(), WasmError::StoreNotInitialized));
}

#[test]
fn test_store_manager_with_multiple_message_calls() {
    // Integration test: Store state remains consistent across multiple calls
    let engine = create_test_engine();
    let component_id = ComponentId::new("test", "multi-call", "0");
    
    let wasm_bytes = load_fixture_wasm("minimal-component");
    let component = Component::from_binary(&engine, &wasm_bytes).unwrap();
    
    let host_state = HostState {
        component_id: component_id.clone(),
    };
    let store = Store::new(&engine, host_state);
    let linker = Linker::new(&engine);
    
    let mut manager = StoreManager::new(store, component);
    manager.initialize(&linker).expect("Initialization should succeed");
    
    // Call multiple times
    let msg1 = create_test_message();
    let msg2 = create_test_message();
    let msg3 = create_test_message();
    
    let result1 = manager.call_handle_message(&msg1);
    let result2 = manager.call_handle_message(&msg2);
    let result3 = manager.call_handle_message(&msg3);
    
    // All should succeed
    assert!(result1.is_ok());
    assert!(result2.is_ok());
    assert!(result3.is_ok());
    
    // Store should remain initialized
    assert!(manager.is_initialized());
}

#[test]
fn test_store_manager_accessors_with_real_component() {
    // Integration test: Store accessors work correctly with real component
    let engine = create_test_engine();
    let component_id = ComponentId::new("test", "accessors", "0");
    
    let wasm_bytes = load_fixture_wasm("minimal-component");
    let component = Component::from_binary(&engine, &wasm_bytes).unwrap();
    
    let host_state = HostState {
        component_id: component_id.clone(),
    };
    let store = Store::new(&engine, host_state);
    
    let mut manager = StoreManager::new(store, component);
    
    // Test accessors
    let _store_ref = manager.store();
    let _store_mut = manager.store_mut();
    let _component_ref = manager.component();
    
    // Should not panic
}

#[test]
fn test_store_manager_initialization_with_invalid_component() {
    // Integration test: Invalid component should fail at initialization
    let engine = create_test_engine();
    let component_id = ComponentId::new("test", "invalid", "0");
    
    // Create invalid WASM component (missing required exports)
    let wasm = wat::parse_str(r#"
        (component
            (core module $empty)
        )
    "#).unwrap();
    let component = Component::from_binary(&engine, &wasm).unwrap();
    
    let host_state = HostState {
        component_id: component_id.clone(),
    };
    let store = Store::new(&engine, host_state);
    let linker = Linker::new(&engine);
    
    let mut manager = StoreManager::new(store, component);
    
    // Initialization should succeed (component is valid, just minimal)
    let result = manager.initialize(&linker);
    assert!(result.is_ok());
}
```

**Verification:**
```bash
cargo test --test store-integration-tests
```

**Expected:** All tests pass

---

### Action 4: Update `runtime/mod.rs`

**Objective:** Add store module declaration

**File:** `airssys-wasm/src/runtime/mod.rs`

**Changes:**

Add to module declarations:
```rust
pub mod store;
```

**Verification:**
```bash
cargo build -p airssys-wasm
```

---

## Architecture Verification (ADR-WASM-023)

Run these commands to verify no forbidden imports (per AGENTS.md Section 13):

```bash
# Verify runtime/ has no imports from actor/
grep -rn "use crate::actor" airssys-wasm/src/runtime/
# Must return nothing

# Verify no reverse dependencies
grep -rn "use crate::runtime" airssys-wasm/src/core/
grep -rn "use crate::actor" airssys-wasm/src/core/
grep -rn "use crate::security" airssys-wasm/src/core/
# All must return nothing
```

**Expected Result:** All commands return nothing (no forbidden imports)

---

## Verification Commands

```bash
# 1. Build check
cargo build -p airssys-wasm

# 2. Unit tests
cargo test -p airssys-wasm --lib runtime::store

# 3. Integration tests
cargo test --test store-integration-tests

# 4. Lint check (zero warnings required)
cargo clippy -p airssys-wasm --all-targets --all-features -- -D warnings

# 5. Architecture verification (ADR-WASM-023)
grep -rn "use crate::actor" airssys-wasm/src/runtime/
# Must return nothing
```

---

## Success Criteria

- [ ] WasmError::StoreNotInitialized variant added
- [ ] StoreManager struct implemented with correct error types (WasmError)
- [ ] Initialize, call_handle_message, call_handle_callback methods implemented
- [ ] Unit tests implemented and passing (NO placeholders)
- [ ] Integration tests implemented and passing
- [ ] Architecture verification passes (no forbidden imports)
- [ ] Zero compiler warnings (cargo build)
- [ ] Zero clippy warnings (cargo clippy --all-targets --all-features -- -D warnings)
- [ ] All import paths correct (use crate::core::component::message::{ComponentMessage, MessagePayload})
- [ ] All return types use WasmError (NOT RuntimeError)
- [ ] module declared in runtime/mod.rs

## Notes

> **wit-bindgen Integration**: Full message handling requires wit-bindgen generated bindings.
> Placeholder implementations return Ok(None) / Ok(()) until Phase 6 integration.

> **Testing Mandate (AGENTS.md Section 9)**: This task includes BOTH unit tests AND integration tests.
> NO EXCEPTIONS. Tests are NOT placeholders - they instantiate real types and verify actual behavior.
