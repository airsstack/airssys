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
use airssys_wasm::runtime::engine::HostState;
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
    let component =
        Component::from_binary(&engine, &wasm_bytes).expect("Valid WASM component should load");

    let host_state = HostState {
        component_id: component_id.clone(),
        message_router: None,
        store_limits: wasmtime::StoreLimitsBuilder::new().build(),
    };
    let store = Store::new(&engine, host_state);
    let linker = Linker::new(&engine);

    let mut manager = StoreManager::new(store, component);

    // Initialize with linker
    let init_result = manager.initialize(&linker);
    assert!(init_result.is_ok(), "Initialization should succeed");
    assert!(
        manager.is_initialized(),
        "Should be initialized after initialization"
    );

    // Call handle_message (placeholder returns Ok(None))
    let msg = create_test_message();
    let result = manager.call_handle_message(&msg);
    assert!(
        result.is_ok(),
        "Message handling should succeed after initialization"
    );
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
        message_router: None,
        store_limits: wasmtime::StoreLimitsBuilder::new().build(),
    };
    let store = Store::new(&engine, host_state);

    let mut manager = StoreManager::new(store, component);
    let msg = create_test_message();

    // Should return StoreNotInitialized
    let result = manager.call_handle_message(&msg);
    assert!(result.is_err());
    assert!(matches!(
        result.unwrap_err(),
        WasmError::StoreNotInitialized
    ));

    // Same for call_handle_callback
    let result = manager.call_handle_callback(&msg);
    assert!(result.is_err());
    assert!(matches!(
        result.unwrap_err(),
        WasmError::StoreNotInitialized
    ));
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
        message_router: None,
        store_limits: wasmtime::StoreLimitsBuilder::new().build(),
    };
    let store = Store::new(&engine, host_state);
    let linker = Linker::new(&engine);

    let mut manager = StoreManager::new(store, component);
    manager
        .initialize(&linker)
        .expect("Initialization should succeed");

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
        message_router: None,
        store_limits: wasmtime::StoreLimitsBuilder::new().build(),
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
fn test_store_manager_initialization_with_minimal_component() {
    // Integration test: Minimal component should initialize successfully
    let engine = create_test_engine();
    let component_id = ComponentId::new("test", "minimal", "0");

    // Create minimal WASM component inline
    let wasm = wat::parse_str(
        r#"
        (component
            (core module $empty)
        )
    "#,
    )
    .unwrap();
    let component = Component::from_binary(&engine, &wasm).unwrap();

    let host_state = HostState {
        component_id: component_id.clone(),
        message_router: None,
        store_limits: wasmtime::StoreLimitsBuilder::new().build(),
    };
    let store = Store::new(&engine, host_state);
    let linker = Linker::new(&engine);

    let mut manager = StoreManager::new(store, component);

    // Initialization should succeed (component is valid, just minimal)
    let result = manager.initialize(&linker);
    assert!(result.is_ok());
}
