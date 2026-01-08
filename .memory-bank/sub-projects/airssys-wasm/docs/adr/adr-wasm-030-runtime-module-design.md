# ADR-WASM-030: Runtime Module Design

**ADR ID:** ADR-WASM-030  
**Created:** 2026-01-05  
**Status:** Accepted  
**Deciders:** Architecture Team  
**Category:** Module Design / WASM Runtime  
**Parent:** [ADR-WASM-026](adr-wasm-026-implementation-roadmap-clean-slate-rebuild.md) (Phase 5)

---

## Title

Runtime Module Design for WASM Execution

---

## Context

The `runtime/` module is **Layer 2B** of the architecture. It:
- Implements `core/runtime/traits.rs` (RuntimeEngine, ComponentLoader)
- Uses **wasmtime Component Model** API (NOT core WASM)
- Manages WASM stores, memory, and resource limits

**Import Rules:**
- ✅ Can import: `core/`, `security/`
- ❌ Cannot import: `component/`, `messaging/`, `system/`

### Critical Constraint

> ⚠️ **MANDATORY**: Use `wasmtime::component::Component`, NOT `wasmtime::Module`.
> See [KNOWLEDGE-WASM-027](../knowledges/knowledge-wasm-027-duplicate-wasm-runtime-fatal-architecture-violation.md) for why this is critical.

### References

- [ADR-WASM-002](adr-wasm-002-wasm-runtime-engine-selection.md): Runtime Engine Selection
- [ADR-WASM-025](adr-wasm-025-clean-slate-rebuild-architecture.md): Clean-Slate Architecture

---

## Decision

### Runtime Module Structure

```
runtime/
├── mod.rs
├── engine.rs           # WasmtimeEngine (RuntimeEngine impl)
├── loader.rs           # ComponentLoader implementation
├── store.rs            # StoreManager for WASM stores
├── host_fn.rs          # Host function bindings
└── limiter.rs          # ResourceLimiter for fuel/memory
```

---

## Detailed Specifications

### runtime/engine.rs

```rust
use std::collections::HashMap;
use std::sync::{Arc, RwLock};

use wasmtime::component::{Component, Linker, ResourceTable};
use wasmtime::{Config, Engine, Store};

use crate::core::component::handle::ComponentHandle;
use crate::core::component::id::ComponentId;
use crate::core::component::message::ComponentMessage;
use crate::core::errors::wasm::WasmError;
use crate::core::component::MessagePayload;
use crate::core::runtime::limits::ResourceLimits;
use crate::core::runtime::traits::RuntimeEngine;

use super::store::StoreManager;

/// WASM runtime engine using wasmtime Component Model
pub struct WasmtimeEngine {
    engine: Engine,
    linker: Linker<HostState>,
    stores: RwLock<HashMap<u64, StoreManager>>,
    next_handle_id: RwLock<u64>,
}

/// Host state passed to WASM components
pub struct HostState {
    pub component_id: ComponentId,
    pub resource_table: ResourceTable,
    // Add other host state as needed
}

impl WasmtimeEngine {
    pub fn new() -> Result<Self, WasmError> {
        let mut config = Config::new();
        config.wasm_component_model(true);
        config.async_support(true);
        config.consume_fuel(true);

        let engine = Engine::new(&config)
            .map_err(|e| WasmError::InstantiationFailed(e.to_string()))?;

        let mut linker = Linker::new(&engine);
        
        // Link host functions here
        // linker.root().func_wrap(...)?;

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
            .map_err(|e| WasmError::InvalidComponent(e.to_string()))?;

        // Create store with host state
        let host_state = HostState {
            component_id: id.clone(),
            resource_table: ResourceTable::new(),
        };

        let store = Store::new(&self.engine, host_state);
        let handle_id = self.allocate_handle_id();

        // Store the store manager
        let store_manager = StoreManager::new(store, component);
        {
            let mut stores = self.stores.write().unwrap();
            stores.insert(handle_id, store_manager);
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
        msg: &ComponentMessage,
    ) -> Result<Option<MessagePayload>, WasmError> {
        let mut stores = self.stores.write().unwrap();
        
        let store_manager = stores.get_mut(&handle.handle_id()).ok_or_else(|| {
            WasmError::ComponentNotFound(handle.id().to_string())
        })?;

        // Call the component's handle-message export
        // This uses Component Model typed interface
        store_manager.call_handle_message(msg)
    }

    fn call_handle_callback(
        &self,
        handle: &ComponentHandle,
        msg: &ComponentMessage,
    ) -> Result<(), WasmError> {
        let mut stores = self.stores.write().unwrap();
        
        let store_manager = stores.get_mut(&handle.handle_id()).ok_or_else(|| {
            WasmError::ComponentNotFound(handle.id().to_string())
        })?;

        store_manager.call_handle_callback(msg)
    }
}
```

---

### runtime/store.rs

```rust
use wasmtime::component::{Component, Instance, Linker};
use wasmtime::Store;

use crate::core::component::message::ComponentMessage;
use crate::core::errors::wasm::WasmError;
use crate::core::component::MessagePayload;

use super::engine::HostState;

/// Manages a WASM store and its associated component instance
pub struct StoreManager {
    store: Store<HostState>,
    component: Component,
    instance: Option<Instance>,
}

impl StoreManager {
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

    /// Call handle-message on the component
    pub fn call_handle_message(
        &mut self,
        msg: &ComponentMessage,
    ) -> Result<Option<MessagePayload>, WasmError> {
        let instance = self.instance.as_ref().ok_or_else(|| {
            WasmError::RuntimeError("Component not initialized".to_string())
        })?;

        // Get the component-lifecycle export
        // Use wit-bindgen generated bindings for type-safe calls
        // This is a placeholder - actual implementation uses generated bindings
        
        // Example with generated bindings:
        // let exports = ComponentLifecycle::new(&mut self.store, instance)?;
        // let result = exports.call_handle_message(&mut self.store, msg)?;
        
        todo!("Implement with wit-bindgen generated bindings")
    }

    /// Call handle-callback on the component
    pub fn call_handle_callback(
        &mut self,
        msg: &ComponentMessage,
    ) -> Result<(), WasmError> {
        let instance = self.instance.as_ref().ok_or_else(|| {
            WasmError::RuntimeError("Component not initialized".to_string())
        })?;

        // Similar to handle_message, use generated bindings
        todo!("Implement with wit-bindgen generated bindings")
    }

    /// Get the store
    pub fn store(&self) -> &Store<HostState> {
        &self.store
    }

    /// Get mutable store
    pub fn store_mut(&mut self) -> &mut Store<HostState> {
        &mut self.store
    }
}
```

---

### runtime/loader.rs

```rust
use std::path::Path;

use crate::core::component::id::ComponentId;
use crate::core::errors::wasm::WasmError;
use crate::core::runtime::traits::ComponentLoader;

/// File-based component loader
pub struct FileComponentLoader {
    base_path: String,
}

impl FileComponentLoader {
    pub fn new(base_path: impl Into<String>) -> Self {
        Self {
            base_path: base_path.into(),
        }
    }

    fn component_path(&self, id: &ComponentId) -> String {
        format!(
            "{}/{}/{}/{}.wasm",
            self.base_path, id.namespace, id.name, id.instance
        )
    }
}

impl ComponentLoader for FileComponentLoader {
    fn load_bytes(&self, id: &ComponentId) -> Result<Vec<u8>, WasmError> {
        let path = self.component_path(id);
        
        std::fs::read(&path).map_err(|e| {
            WasmError::ComponentNotFound(format!("Failed to load {}: {}", path, e))
        })
    }

    fn validate(&self, bytes: &[u8]) -> Result<(), WasmError> {
        // Basic validation: check WASM magic number
        if bytes.len() < 8 {
            return Err(WasmError::InvalidComponent("File too small".to_string()));
        }

        // WASM magic number: 0x00 0x61 0x73 0x6D (\0asm)
        if &bytes[0..4] != b"\0asm" {
            return Err(WasmError::InvalidComponent(
                "Invalid WASM magic number".to_string(),
            ));
        }

        // For component model, could add additional validation
        // using wasmtime's Component::validate()

        Ok(())
    }
}

/// In-memory component loader for testing
#[cfg(test)]
pub struct InMemoryComponentLoader {
    components: std::collections::HashMap<String, Vec<u8>>,
}

#[cfg(test)]
impl InMemoryComponentLoader {
    pub fn new() -> Self {
        Self {
            components: std::collections::HashMap::new(),
        }
    }

    pub fn add_component(&mut self, id: &ComponentId, bytes: Vec<u8>) {
        self.components.insert(id.to_string_id(), bytes);
    }
}

#[cfg(test)]
impl ComponentLoader for InMemoryComponentLoader {
    fn load_bytes(&self, id: &ComponentId) -> Result<Vec<u8>, WasmError> {
        self.components
            .get(&id.to_string_id())
            .cloned()
            .ok_or_else(|| WasmError::ComponentNotFound(id.to_string_id()))
    }

    fn validate(&self, bytes: &[u8]) -> Result<(), WasmError> {
        if bytes.len() < 4 || &bytes[0..4] != b"\0asm" {
            return Err(WasmError::InvalidComponent("Invalid WASM".to_string()));
        }
        Ok(())
    }
}
```

---

### runtime/limiter.rs

```rust
use wasmtime::{ResourceLimiter, StoreLimits, StoreLimitsBuilder};

use crate::core::runtime::limits::ResourceLimits;

/// Resource limiter for WASM execution
pub struct WasmResourceLimiter {
    limits: ResourceLimits,
    store_limits: StoreLimits,
}

impl WasmResourceLimiter {
    pub fn new(limits: ResourceLimits) -> Self {
        let store_limits = StoreLimitsBuilder::new()
            .memory_size(limits.max_memory_bytes as usize)
            .build();

        Self {
            limits,
            store_limits,
        }
    }

    /// Get the store limits for wasmtime
    pub fn store_limits(&self) -> &StoreLimits {
        &self.store_limits
    }

    /// Get fuel limit if configured
    pub fn fuel_limit(&self) -> Option<u64> {
        self.limits.max_fuel
    }

    /// Get execution timeout
    pub fn timeout_ms(&self) -> u64 {
        self.limits.max_execution_time_ms
    }
}

/// Apply resource limits to a store
pub fn apply_limits<T>(
    store: &mut wasmtime::Store<T>,
    limiter: &WasmResourceLimiter,
) -> Result<(), crate::core::errors::wasm::WasmError> {
    // Set fuel if configured
    if let Some(fuel) = limiter.fuel_limit() {
        store.set_fuel(fuel).map_err(|e| {
            crate::core::errors::wasm::WasmError::RuntimeError(format!(
                "Failed to set fuel: {}",
                e
            ))
        })?;
    }

    Ok(())
}
```

---

### runtime/host_fn.rs

```rust
use wasmtime::component::Linker;

use crate::core::component::id::ComponentId;
use crate::core::component::MessagePayload;

use super::engine::HostState;

/// Register host functions with the linker
pub fn register_host_functions(
    linker: &mut Linker<HostState>,
) -> Result<(), wasmtime::Error> {
    // host-messaging interface
    register_messaging_functions(linker)?;
    
    // host-services interface
    register_services_functions(linker)?;
    
    // storage interface
    register_storage_functions(linker)?;

    Ok(())
}

fn register_messaging_functions(
    linker: &mut Linker<HostState>,
) -> Result<(), wasmtime::Error> {
    // These will be implemented using wit-bindgen generated bindings
    // Example structure:
    //
    // linker.root().func_wrap(
    //     "host-messaging",
    //     "send",
    //     |caller: Caller<HostState>, target: ComponentId, payload: Vec<u8>| {
    //         // Implementation
    //     },
    // )?;

    Ok(())
}

fn register_services_functions(
    linker: &mut Linker<HostState>,
) -> Result<(), wasmtime::Error> {
    // log, current-time, etc.
    Ok(())
}

fn register_storage_functions(
    linker: &mut Linker<HostState>,
) -> Result<(), wasmtime::Error> {
    // get, set, delete, etc.
    Ok(())
}
```

---

## wasmtime Dependency Configuration

```toml
# Cargo.toml
[dependencies]
wasmtime = { version = "24.0", features = ["component-model", "async"] }
```

---

## History

| Date | Version | Change |
|------|---------|--------|
| 2026-01-05 | 1.0 | Initial runtime module design |

---

**This ADR defines the runtime module structure for Phase 5 of the rebuild.**
