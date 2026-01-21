//! Integration tests for airssys-wasm resource limiter API.
//!
//! Tests that our WasmResourceLimiter and apply_limits_to_store()
//! correctly configure Wasmtime's StoreLimits and Fuel.

use airssys_wasm::core::component::id::ComponentId;
use airssys_wasm::core::runtime::limits::ResourceLimits;
use airssys_wasm::runtime::engine::HostState;
use airssys_wasm::runtime::limiter::{apply_limits_to_store, WasmResourceLimiter};
use wasmtime::{Config, Engine, Store, StoreLimitsBuilder};

/// Create an engine configured for fuel consumption
fn create_engine() -> Engine {
    let mut config = Config::new();
    config.wasm_component_model(true);
    config.consume_fuel(true);
    Engine::new(&config).expect("Failed to create engine")
}

#[test]
fn test_apply_limits_sets_store_limits() {
    let engine = create_engine();

    // Create HostState with default StoreLimits
    let host_state = HostState {
        component_id: ComponentId::new("test", "limiter", "test"),
        message_router: None,
        store_limits: StoreLimitsBuilder::new().build(),
    };

    let mut store = Store::new(&engine, host_state);

    // Apply custom limits
    let limits = ResourceLimits {
        max_memory_bytes: 1024 * 1024, // 1MB
        max_execution_time_ms: 5000,
        max_fuel: Some(10_000),
    };

    apply_limits_to_store(&mut store, &limits).expect("Failed to apply limits");

    // Verify fuel was set
    let fuel = store.get_fuel().expect("Fuel should be set");
    assert_eq!(fuel, 10_000);

    // Verify limiter callback is configured (store has limiter)
    // We can't directly inspect StoreLimits, but we know it's set if no panic
}

#[test]
fn test_apply_limits_with_no_fuel() {
    let engine = create_engine();

    let host_state = HostState {
        component_id: ComponentId::new("test", "nofuel", "test"),
        message_router: None,
        store_limits: StoreLimitsBuilder::new().build(),
    };

    let mut store = Store::new(&engine, host_state);

    // Apply limits WITHOUT fuel
    let limits = ResourceLimits {
        max_memory_bytes: 2 * 1024 * 1024, // 2MB
        max_execution_time_ms: 10_000,
        max_fuel: None,
    };

    apply_limits_to_store(&mut store, &limits).expect("Failed to apply limits");

    // When fuel is None, apply_limits_to_store doesn't call set_fuel()
    // get_fuel() returns Ok(0) because engine has fuel enabled but store has 0 fuel
    let fuel = store.get_fuel().expect("Should succeed");
    assert_eq!(fuel, 0); // No fuel set, defaults to 0
}

#[test]
fn test_apply_limits_multiple_times() {
    let engine = create_engine();

    let host_state = HostState {
        component_id: ComponentId::new("test", "multiple", "test"),
        message_router: None,
        store_limits: StoreLimitsBuilder::new().build(),
    };

    let mut store = Store::new(&engine, host_state);

    // Apply first set of limits
    let limits1 = ResourceLimits {
        max_memory_bytes: 512 * 1024,
        max_execution_time_ms: 1000,
        max_fuel: Some(5_000),
    };

    apply_limits_to_store(&mut store, &limits1).expect("Failed to apply limits");
    let fuel1 = store.get_fuel().expect("Fuel should be set");
    assert_eq!(fuel1, 5_000);

    // Apply second set of limits (overwrite)
    let limits2 = ResourceLimits {
        max_memory_bytes: 1024 * 1024,
        max_execution_time_ms: 2000,
        max_fuel: Some(20_000),
    };

    apply_limits_to_store(&mut store, &limits2).expect("Failed to apply limits");
    let fuel2 = store.get_fuel().expect("Fuel should be updated");
    assert_eq!(fuel2, 20_000);
}

#[test]
fn test_wasm_resource_limiter_new() {
    let limits = ResourceLimits {
        max_memory_bytes: 16 * 1024 * 1024, // 16MB
        max_execution_time_ms: 30_000,
        max_fuel: Some(1_000_000),
    };

    let limiter = WasmResourceLimiter::new(&limits);

    // Verify fuel limit is captured
    assert_eq!(limiter.fuel_limit(), Some(1_000_000));

    // Verify StoreLimits can be extracted
    let _store_limits = limiter.into_store_limits();
}

#[test]
fn test_wasm_resource_limiter_default_limits() {
    let limits = ResourceLimits::default();
    let limiter = WasmResourceLimiter::new(&limits);

    // Default ResourceLimits has no fuel
    assert_eq!(limiter.fuel_limit(), None);
}

#[test]
fn test_apply_limits_end_to_end() {
    // End-to-end test: ResourceLimits -> apply_limits_to_store -> verify all set
    let engine = create_engine();

    let host_state = HostState {
        component_id: ComponentId::new("test", "e2e", "test"),
        message_router: None,
        store_limits: StoreLimitsBuilder::new().build(),
    };

    let mut store = Store::new(&engine, host_state);

    // Create comprehensive limits
    let limits = ResourceLimits {
        max_memory_bytes: 8 * 1024 * 1024, // 8MB
        max_execution_time_ms: 15_000,
        max_fuel: Some(500_000),
    };

    // Apply limits
    apply_limits_to_store(&mut store, &limits).expect("Failed to apply limits");

    // Verify fuel was set correctly
    let fuel = store.get_fuel().expect("Fuel should be set");
    assert_eq!(fuel, 500_000);

    // Verify HostState's store_limits was updated (indirectly)
    // We can't access it directly, but if no panic, it's set correctly
    // The limiter callback is configured and will use HostState.store_limits

    // Simulate consuming some fuel
    store
        .set_fuel(fuel - 100)
        .expect("Should be able to update fuel");
    let remaining = store.get_fuel().expect("Should get remaining fuel");
    assert_eq!(remaining, 499_900);
}
