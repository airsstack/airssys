# WASM-TASK-031: Implementation Plans

## Plan References
- **ADR-WASM-030:** Runtime Module Design (primary specification)
- **ADR-WASM-025:** Clean-Slate Rebuild Architecture
- **ADR-WASM-026:** Implementation Roadmap (Phase 5)
- **ADR-WASM-002:** WASM Runtime Engine Selection
- **KNOWLEDGE-WASM-027:** Duplicate WASM Runtime Fatal Architecture Violation
- **KNOWLEDGE-WASM-037:** Rebuild Architecture Clean-Slate

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
```

---

## Implementation Actions

### Action 1: Create `runtime/mod.rs`

**Objective:** Module declarations following §4.3 pattern

**File:** `airssys-wasm/src/runtime/mod.rs`

```rust
//! # Runtime Module
//!
//! WASM component execution using wasmtime Component Model.
//!
//! ## Layer 2B - Runtime Layer
//!
//! Import Rules:
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

**Specification (ADR-WASM-030 lines 59-193):**

```rust
//! WasmtimeEngine implementation using wasmtime Component Model.

use std::collections::HashMap;
use std::sync::RwLock;

use wasmtime::component::{Component, Linker, ResourceTable};
use wasmtime::{Config, Engine, Store};

use crate::core::component::handle::ComponentHandle;
use crate::core::component::id::ComponentId;
use crate::core::component::message::ComponentMessage;
use crate::core::component::MessagePayload;
use crate::core::runtime::traits::RuntimeEngine;

/// Error type for runtime operations
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum RuntimeError {
    /// Engine creation failed
    EngineCreation(String),
    /// Component instantiation failed
    InstantiationFailed(String),
    /// Component not found
    ComponentNotFound(String),
    /// Runtime execution error
    ExecutionError(String),
}

impl std::fmt::Display for RuntimeError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::EngineCreation(msg) => write!(f, "Engine creation failed: {}", msg),
            Self::InstantiationFailed(msg) => write!(f, "Instantiation failed: {}", msg),
            Self::ComponentNotFound(id) => write!(f, "Component not found: {}", id),
            Self::ExecutionError(msg) => write!(f, "Execution error: {}", msg),
        }
    }
}

impl std::error::Error for RuntimeError {}

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
    pub fn new() -> Result<Self, RuntimeError> {
        let mut config = Config::new();
        config.wasm_component_model(true);
        config.async_support(true);
        config.consume_fuel(true);

        let engine = Engine::new(&config)
            .map_err(|e| RuntimeError::EngineCreation(e.to_string()))?;

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
    type Error = RuntimeError;

    fn load_component(
        &self,
        id: &ComponentId,
        bytes: &[u8],
    ) -> Result<ComponentHandle, Self::Error> {
        // Parse component from bytes
        let component = Component::from_binary(&self.engine, bytes)
            .map_err(|e| RuntimeError::InstantiationFailed(e.to_string()))?;

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

    fn unload_component(&self, handle: &ComponentHandle) -> Result<(), Self::Error> {
        let mut stores = self.stores.write().unwrap();
        stores.remove(&handle.handle_id());
        Ok(())
    }

    fn call_handle_message(
        &self,
        handle: &ComponentHandle,
        _msg: &ComponentMessage,
    ) -> Result<Option<MessagePayload>, Self::Error> {
        let stores = self.stores.read().unwrap();
        
        let _entry = stores.get(&handle.handle_id()).ok_or_else(|| {
            RuntimeError::ComponentNotFound(handle.id().to_string())
        })?;

        // Placeholder - actual implementation requires wit-bindgen integration
        // Will be implemented in WASM-TASK-033 (StoreManager)
        Ok(None)
    }

    fn call_handle_callback(
        &self,
        handle: &ComponentHandle,
        _msg: &ComponentMessage,
    ) -> Result<(), Self::Error> {
        let stores = self.stores.read().unwrap();
        
        let _entry = stores.get(&handle.handle_id()).ok_or_else(|| {
            RuntimeError::ComponentNotFound(handle.id().to_string())
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
        assert!(matches!(result, Err(RuntimeError::ComponentNotFound(_))));
    }

    #[test]
    fn test_runtime_error_display() {
        let err = RuntimeError::EngineCreation("test error".to_string());
        assert!(err.to_string().contains("Engine creation failed"));
        
        let err = RuntimeError::ComponentNotFound("comp-1".to_string());
        assert!(err.to_string().contains("Component not found"));
    }
}
```

---

## Verification Commands

```bash
# 1. Build check
cargo build -p airssys-wasm

# 2. Lint check
cargo clippy -p airssys-wasm --all-targets -- -D warnings

# 3. Run runtime tests
cargo test -p airssys-wasm --lib runtime::engine

# 4. Module boundary check
grep -rn "use crate::component\|use crate::messaging\|use crate::system" src/runtime/
# Should return empty (no forbidden imports)
```

---

## Success Criteria

- [ ] WasmtimeEngine struct implemented
- [ ] HostState struct implemented
- [ ] RuntimeEngine trait implemented
- [ ] Engine config uses component_model, async, fuel
- [ ] Build passes with zero warnings
- [ ] Clippy passes with zero warnings
- [ ] All unit tests pass (7+ tests)
- [ ] Only imports from core/, security/ (Layer 1, Layer 2A)
- [ ] mod.rs contains only declarations

---

## Critical Notes

> ⚠️ **MANDATORY**: Use `wasmtime::component::Component`, NOT `wasmtime::Module`.
> Per KNOWLEDGE-WASM-027, using core WASM Module API instead of Component Model is a fatal architecture violation.

> **Placeholder Methods**: `call_handle_message` and `call_handle_callback` are placeholders.
> Full implementation requires wit-bindgen integration in WASM-TASK-033 (StoreManager).
