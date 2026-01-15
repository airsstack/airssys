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
        let _instance = self
            .instance
            .as_ref()
            .ok_or(WasmError::StoreNotInitialized)?;

        // Placeholder - actual implementation uses wit-bindgen generated bindings
        // Example with generated bindings:
        // let exports = ComponentLifecycle::new(&mut self.store, instance)?;
        // let result = exports.call_handle_message(&mut self.store, msg)?;

        Ok(None)
    }

    /// Call handle-callback on the component
    pub fn call_handle_callback(&mut self, _msg: &ComponentMessage) -> Result<(), WasmError> {
        let _instance = self
            .instance
            .as_ref()
            .ok_or(WasmError::StoreNotInitialized)?;

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
    use crate::core::component::id::ComponentId;
    use crate::core::component::message::MessageMetadata;
    use wasmtime::{Config, Engine};

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
        let wasm = wat::parse_str(
            r#"
            (component)
        "#,
        )
        .unwrap();
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

        let wasm = wat::parse_str(
            r#"
            (component)
        "#,
        )
        .unwrap();
        let component = Component::from_binary(&engine, &wasm).unwrap();

        let mut manager = StoreManager::new(store, component);
        let msg = create_test_message();

        // Should return StoreNotInitialized error
        let result = manager.call_handle_message(&msg);
        assert!(result.is_err());
        assert!(matches!(
            result.unwrap_err(),
            WasmError::StoreNotInitialized
        ));
    }

    #[test]
    fn test_call_callback_not_initialized() {
        let engine = create_test_engine();
        let component_id = ComponentId::new("test", "comp", "0");
        let host_state = HostState {
            component_id: component_id.clone(),
        };
        let store = Store::new(&engine, host_state);

        let wasm = wat::parse_str(
            r#"
            (component)
        "#,
        )
        .unwrap();
        let component = Component::from_binary(&engine, &wasm).unwrap();

        let mut manager = StoreManager::new(store, component);
        let msg = create_test_message();

        // Should return StoreNotInitialized error
        let result = manager.call_handle_callback(&msg);
        assert!(result.is_err());
        assert!(matches!(
            result.unwrap_err(),
            WasmError::StoreNotInitialized
        ));
    }

    #[test]
    fn test_store_manager_initialization_success() {
        let engine = create_test_engine();
        let component_id = ComponentId::new("test", "comp", "0");
        let host_state = HostState {
            component_id: component_id.clone(),
        };
        let store = Store::new(&engine, host_state);

        let wasm = wat::parse_str(
            r#"
            (component)
        "#,
        )
        .unwrap();
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

        let wasm = wat::parse_str(
            r#"
            (component)
        "#,
        )
        .unwrap();
        let component = Component::from_binary(&engine, &wasm).unwrap();

        let mut manager = StoreManager::new(store, component);

        // Test store accessors
        let _store_ref = manager.store();
        let _store_mut = manager.store_mut();
        let _component_ref = manager.component();

        // Should not panic
    }
}
