//! Integration tests for StoreManager.
//!
//! Tests end-to-end store lifecycle and REAL WASM message dispatch
//! using echo.wasm, counter.wasm, and callback.wasm test fixtures.

// Layer 1: Standard library imports
use std::path::Path;

// Layer 2: Third-party crate imports
// (none)

// Layer 3: Internal module imports
use airssys_wasm::core::component::id::ComponentId;
use airssys_wasm::core::component::message::{ComponentMessage, MessageMetadata, MessagePayload};
use airssys_wasm::core::runtime::errors::WasmError;
use airssys_wasm::runtime::engine::HostState;
use airssys_wasm::runtime::host_functions::marker_traits::register_host_functions;
use airssys_wasm::runtime::store::StoreManager;
use wasmtime::component::{Component, Linker};
use wasmtime::{Config, Engine, Store};

/// Load a real WASM component binary from fixtures.
fn load_fixture_wasm(name: &str) -> Vec<u8> {
    let fixture_path = Path::new("tests/fixtures")
        .join(name)
        .with_extension("wasm");

    std::fs::read(&fixture_path)
        .unwrap_or_else(|_| panic!("Failed to read fixture: {}", fixture_path.display()))
}

/// Create an engine with async_support and component model enabled,
/// matching the production WasmtimeEngine configuration.
fn create_async_engine() -> Engine {
    let mut config = Config::new();
    config.wasm_component_model(true);
    config.async_support(true);
    Engine::new(&config).unwrap()
}

/// Create a linker with all host functions registered.
fn create_linker_with_host_functions(engine: &Engine) -> Linker<HostState> {
    let mut linker = Linker::new(engine);
    register_host_functions(&mut linker).unwrap();
    linker
}

fn create_test_message_with_payload(data: Vec<u8>) -> ComponentMessage {
    let sender = ComponentId::new("test", "sender", "0");
    let payload = MessagePayload::new(data);
    let metadata = MessageMetadata::default();
    ComponentMessage::new(sender, payload, metadata)
}

/// Helper: Create and initialize a StoreManager with a fixture component.
fn create_initialized_store_manager(
    engine: &Engine,
    linker: &Linker<HostState>,
    fixture_name: &str,
    component_id: &ComponentId,
) -> StoreManager {
    let wasm_bytes = load_fixture_wasm(fixture_name);
    let component =
        Component::from_binary(engine, &wasm_bytes).expect("Valid WASM component should load");

    let host_state = HostState {
        component_id: component_id.clone(),
        message_router: None,
        store_limits: wasmtime::StoreLimitsBuilder::new().build(),
    };
    let store = Store::new(engine, host_state);
    let mut manager = StoreManager::new(store, component);

    futures::executor::block_on(manager.initialize(linker)).expect("Initialization should succeed");

    manager
}

#[test]
fn test_echo_dispatch_returns_payload() {
    // Integration test: echo.wasm returns the exact payload that was sent.
    let engine = create_async_engine();
    let linker = create_linker_with_host_functions(&engine);
    let component_id = ComponentId::new("test", "echo", "0");

    let mut manager = create_initialized_store_manager(&engine, &linker, "echo", &component_id);

    let msg = create_test_message_with_payload(vec![1, 2, 3]);
    let result = manager.call_handle_message(&msg);

    assert!(result.is_ok(), "Echo dispatch failed: {:?}", result);
    let payload = result.unwrap();
    assert!(
        payload.is_some(),
        "echo.wasm should return Some(payload), got None"
    );
    assert_eq!(
        payload.unwrap().as_bytes(),
        &[1, 2, 3],
        "echo.wasm should return the exact payload [1,2,3]"
    );
}

#[test]
fn test_echo_dispatch_empty_payload() {
    // Integration test: echo.wasm correctly handles empty payload.
    let engine = create_async_engine();
    let linker = create_linker_with_host_functions(&engine);
    let component_id = ComponentId::new("test", "echo", "0");

    let mut manager = create_initialized_store_manager(&engine, &linker, "echo", &component_id);

    let msg = create_test_message_with_payload(vec![]);
    let result = manager.call_handle_message(&msg);

    assert!(result.is_ok());
    let payload = result.unwrap();
    assert!(
        payload.is_some(),
        "echo.wasm should return Some even for empty payload"
    );
    assert!(
        payload.unwrap().is_empty(),
        "echo.wasm should return empty payload for empty input"
    );
}

#[test]
fn test_echo_callback_succeeds() {
    // Integration test: echo.wasm handles callback without error.
    let engine = create_async_engine();
    let linker = create_linker_with_host_functions(&engine);
    let component_id = ComponentId::new("test", "echo", "0");

    let mut manager = create_initialized_store_manager(&engine, &linker, "echo", &component_id);

    let msg = create_test_message_with_payload(vec![10, 20, 30]);
    let result = manager.call_handle_callback(&msg);

    assert!(
        result.is_ok(),
        "echo.wasm callback should succeed: {:?}",
        result
    );
}

#[test]
fn test_counter_stateful_dispatch() {
    // Integration test: counter.wasm maintains state across calls
    // and returns incrementing values.
    let engine = create_async_engine();
    let linker = create_linker_with_host_functions(&engine);
    let component_id = ComponentId::new("test", "counter", "0");

    let mut manager = create_initialized_store_manager(&engine, &linker, "counter", &component_id);

    let msg = create_test_message_with_payload(vec![1, 2, 3]);

    // First call
    let result1 = manager.call_handle_message(&msg);
    assert!(result1.is_ok());
    let payload1 = result1.unwrap();
    assert!(payload1.is_some(), "counter.wasm should return a payload");

    // Second call
    let result2 = manager.call_handle_message(&msg);
    assert!(result2.is_ok());
    let payload2 = result2.unwrap();
    assert!(payload2.is_some(), "counter.wasm should return a payload");

    // Third call
    let result3 = manager.call_handle_message(&msg);
    assert!(result3.is_ok());
    let payload3 = result3.unwrap();
    assert!(payload3.is_some(), "counter.wasm should return a payload");

    // Verify counter increments: each call produces different output
    let bytes1 = payload1.unwrap().into_bytes();
    let bytes2 = payload2.unwrap().into_bytes();
    let bytes3 = payload3.unwrap().into_bytes();

    assert_ne!(bytes1, bytes2, "First and second calls should differ");
    assert_ne!(bytes2, bytes3, "Second and third calls should differ");
}

#[test]
fn test_uninitialized_store_returns_error() {
    // Integration test: Uninitialized store returns StoreNotInitialized error.
    let engine = create_async_engine();
    let component_id = ComponentId::new("test", "echo", "0");

    let wasm_bytes = load_fixture_wasm("echo");
    let component = Component::from_binary(&engine, &wasm_bytes).unwrap();

    let host_state = HostState {
        component_id: component_id.clone(),
        message_router: None,
        store_limits: wasmtime::StoreLimitsBuilder::new().build(),
    };
    let store = Store::new(&engine, host_state);

    let mut manager = StoreManager::new(store, component);
    let msg = create_test_message_with_payload(vec![1, 2, 3]);

    // Should return StoreNotInitialized
    let result = manager.call_handle_message(&msg);
    assert!(result.is_err());
    assert!(matches!(
        result.unwrap_err(),
        WasmError::StoreNotInitialized
    ));

    // Same for callback
    let result = manager.call_handle_callback(&msg);
    assert!(result.is_err());
    assert!(matches!(
        result.unwrap_err(),
        WasmError::StoreNotInitialized
    ));
}

#[test]
fn test_multiple_stores_independent_state() {
    // Integration test: Two separate echo.wasm instances operate independently.
    let engine = create_async_engine();
    let linker = create_linker_with_host_functions(&engine);

    let id1 = ComponentId::new("test", "echo", "1");
    let id2 = ComponentId::new("test", "echo", "2");

    let mut manager1 = create_initialized_store_manager(&engine, &linker, "echo", &id1);
    let mut manager2 = create_initialized_store_manager(&engine, &linker, "echo", &id2);

    // Send different payloads to each
    let msg1 = create_test_message_with_payload(vec![10, 20]);
    let msg2 = create_test_message_with_payload(vec![30, 40, 50]);

    let result1 = manager1.call_handle_message(&msg1);
    let result2 = manager2.call_handle_message(&msg2);

    assert!(result1.is_ok());
    assert!(result2.is_ok());

    // Each should echo its own payload
    assert_eq!(result1.unwrap().unwrap().as_bytes(), &[10, 20]);
    assert_eq!(result2.unwrap().unwrap().as_bytes(), &[30, 40, 50]);
}

#[test]
fn test_store_lifecycle_create_init_dispatch() {
    // Integration test: Complete lifecycle: create -> init -> dispatch -> verify.
    let engine = create_async_engine();
    let linker = create_linker_with_host_functions(&engine);
    let component_id = ComponentId::new("test", "lifecycle", "0");

    let wasm_bytes = load_fixture_wasm("echo");
    let component = Component::from_binary(&engine, &wasm_bytes).unwrap();

    let host_state = HostState {
        component_id: component_id.clone(),
        message_router: None,
        store_limits: wasmtime::StoreLimitsBuilder::new().build(),
    };
    let store = Store::new(&engine, host_state);

    // Step 1: Create
    let mut manager = StoreManager::new(store, component);
    assert!(!manager.is_initialized());

    // Step 2: Initialize
    let init_result = futures::executor::block_on(manager.initialize(&linker));
    assert!(init_result.is_ok(), "Init failed: {:?}", init_result);
    assert!(manager.is_initialized());

    // Step 3: Dispatch
    let msg = create_test_message_with_payload(vec![42]);
    let dispatch_result = manager.call_handle_message(&msg);

    // Step 4: Verify
    assert!(dispatch_result.is_ok());
    let payload = dispatch_result.unwrap();
    assert!(payload.is_some());
    assert_eq!(payload.unwrap().as_bytes(), &[42]);
}

#[test]
fn test_store_manager_accessors_with_real_component() {
    // Integration test: Store accessors work correctly with real component.
    let engine = create_async_engine();
    let component_id = ComponentId::new("test", "accessors", "0");

    let wasm_bytes = load_fixture_wasm("echo");
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
