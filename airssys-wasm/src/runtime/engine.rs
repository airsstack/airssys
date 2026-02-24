//! WasmtimeEngine implementation using wasmtime Component Model.

// Layer 1: Standard library imports
use std::collections::HashMap;
use std::sync::{Arc, RwLock};

// Layer 2: Third-party crate imports
use wasmtime::component::{Component, Linker};
use wasmtime::{Config, Engine, Store, StoreLimits, StoreLimitsBuilder};

// Layer 3: Internal module imports
use crate::core::component::handle::ComponentHandle;
use crate::core::component::id::ComponentId;
use crate::core::component::message::{ComponentMessage, MessagePayload};
use crate::core::messaging::traits::MessageRouter;
use crate::core::runtime::errors::WasmError;
use crate::core::runtime::traits::RuntimeEngine;
use crate::runtime::host_functions::marker_traits::register_host_functions;

use super::store::StoreManager;

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
///
/// NOTE: ResourceTable is intentionally omitted here because it is not Sync.
/// It will be added in WASM-TASK-033 (StoreManager) when needed,
/// using a different approach to satisfy Sync requirements.
pub struct HostState {
    /// The component ID for this instance
    pub component_id: ComponentId,
    /// Message router for inter-component communication
    /// Using dyn Trait for dependency injection across module boundaries
    pub message_router: Option<Arc<dyn MessageRouter>>,
    /// Store limits for memory/table enforcement
    pub store_limits: StoreLimits,
}

/// WASM runtime engine using wasmtime Component Model
pub struct WasmtimeEngine {
    engine: Engine,
    linker: Linker<HostState>,
    stores: RwLock<HashMap<u64, StoreManager>>,
    next_handle_id: RwLock<u64>,
}

impl WasmtimeEngine {
    /// Create a new WasmtimeEngine
    pub fn new() -> Result<Self, WasmError> {
        let mut config = Config::new();
        config.wasm_component_model(true);
        config.async_support(true);
        config.consume_fuel(true);

        let engine =
            Engine::new(&config).map_err(|e| WasmError::InstantiationFailed(e.to_string()))?;

        let mut linker = Linker::new(&engine);

        // Register host functions
        register_host_functions(&mut linker)
            .map_err(|e| WasmError::InstantiationFailed(e.to_string()))?;

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
    fn load_component(&self, id: &ComponentId, bytes: &[u8]) -> Result<ComponentHandle, WasmError> {
        let component = Component::from_binary(&self.engine, bytes)
            .map_err(|e| WasmError::InstantiationFailed(e.to_string()))?;

        let host_state = HostState {
            component_id: id.clone(),
            message_router: None,
            store_limits: StoreLimitsBuilder::new().build(),
        };

        let mut store = Store::new(&self.engine, host_state);
        store.limiter(|state| &mut state.store_limits);

        let handle_id = self.allocate_handle_id();

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
            MessagePayload::new(vec![]),
            Default::default(),
        );

        let result = engine.call_handle_message(&handle, &msg);
        assert!(matches!(result, Err(WasmError::ComponentNotFound(_))));
    }

    #[test]
    fn test_wasm_error_display() {
        let err = WasmError::InstantiationFailed("test error".to_string());
        assert!(err.to_string().contains("Component instantiation failed"));

        let err = WasmError::ComponentNotFound("comp-1".to_string());
        assert!(err.to_string().contains("Component not found"));
    }
}
