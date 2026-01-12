# WASM-TASK-033: Implementation Plans

## Plan References
- **ADR-WASM-030:** Runtime Module Design (primary specification)
- **ADR-WASM-026:** Implementation Roadmap (Phase 5)

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

### Action 1: Create `runtime/store.rs`

**Objective:** Implement StoreManager for WASM stores

**File:** `airssys-wasm/src/runtime/store.rs`

**Specification (ADR-WASM-030 lines 198-279):**

```rust
//! WASM store management.

use wasmtime::component::{Component, Instance, Linker};
use wasmtime::Store;

use crate::core::component::message::ComponentMessage;
use crate::core::component::MessagePayload;

use super::engine::{HostState, RuntimeError};

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
    pub fn initialize(&mut self, linker: &Linker<HostState>) -> Result<(), RuntimeError> {
        let instance = linker
            .instantiate(&mut self.store, &self.component)
            .map_err(|e| RuntimeError::InstantiationFailed(e.to_string()))?;

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
    ) -> Result<Option<MessagePayload>, RuntimeError> {
        let _instance = self.instance.as_ref().ok_or_else(|| {
            RuntimeError::ExecutionError("Component not initialized".to_string())
        })?;

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
    ) -> Result<(), RuntimeError> {
        let _instance = self.instance.as_ref().ok_or_else(|| {
            RuntimeError::ExecutionError("Component not initialized".to_string())
        })?;

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
    use wasmtime::component::ResourceTable;

    fn create_test_engine() -> Engine {
        let mut config = Config::new();
        config.wasm_component_model(true);
        Engine::new(&config).unwrap()
    }

    // Note: Full tests require valid WASM component fixtures
    // These tests verify the API structure

    #[test]
    fn test_store_manager_not_initialized() {
        // This test would require a valid Component
        // Placeholder for API verification
    }

    #[test]
    fn test_call_message_not_initialized() {
        // Tests that calling without initialization returns error
        // Requires valid Component fixture
    }
}
```

### Action 2: Update `runtime/mod.rs`

Add `pub mod store;` to module declarations.

---

## Verification Commands

```bash
cargo build -p airssys-wasm
cargo clippy -p airssys-wasm --all-targets -- -D warnings
cargo test -p airssys-wasm --lib runtime::store
```

---

## Success Criteria

- [ ] StoreManager struct implemented
- [ ] Initialize, call_handle_message, call_handle_callback methods
- [ ] Build/Clippy pass with zero warnings
- [ ] Unit tests pass

## Notes

> **wit-bindgen Integration**: Full message handling requires wit-bindgen generated bindings.
> Placeholder implementations return Ok(None) / Ok(()) until Phase 6 integration.
