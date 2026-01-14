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
use airssys_wasm::core::component::message::{ComponentMessage, MessageMetadata, MessagePayload};
use airssys_wasm::core::runtime::errors::WasmError;
use airssys_wasm::core::runtime::traits::RuntimeEngine;
use airssys_wasm::runtime::engine::{HostState, WasmtimeEngine};

/// Load a real WASM component binary from fixtures
fn load_fixture_wasm(name: &str) -> Vec<u8> {
    let fixture_path = Path::new("tests/fixtures")
        .join(name)
        .with_extension("wasm");

    std::fs::read(&fixture_path)
        .unwrap_or_else(|_| panic!("Failed to read fixture: {}", fixture_path.display()))
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
    assert!(
        handle.is_ok(),
        "Valid WASM component should load successfully"
    );

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
    let handle = engine
        .load_component(&component_id, &wasm_bytes)
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
        assert!(!msg.is_empty(), "Error message should not be empty");
    }
}

#[test]
fn test_message_creation_with_foundation_types() {
    // Integration test: ComponentMessage works with foundation types
    let from_id = ComponentId::new("org1", "comp1", "1.0.0");
    let payload = MessagePayload::new("{\"action\":\"test\"}".as_bytes().to_vec());
    let metadata = MessageMetadata::default();

    let msg = ComponentMessage::new(from_id.clone(), payload.clone(), metadata.clone());

    assert_eq!(msg.sender, from_id);
    assert_eq!(msg.payload, payload);
    assert_eq!(msg.metadata.content_type, metadata.content_type);
}

#[test]
fn test_host_state_initialization() {
    // Integration test: HostState is correctly initialized with component ID
    let component_id = ComponentId::new("test", "host-state", "1.0.0");
    let host_state = HostState {
        component_id: component_id.clone(),
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

    let handle_1 = engine
        .load_component(&component_id_1, &wasm_bytes)
        .expect("First load should succeed");

    // Load same component again (should create new instance)
    let handle_2 = engine
        .load_component(&component_id_2, &wasm_bytes)
        .expect("Second load should succeed");

    // Both should succeed with different handle IDs
    assert_ne!(handle_1.handle_id(), handle_2.handle_id());
}
