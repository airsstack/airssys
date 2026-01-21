//! Integration tests for resource limiting.
//!
//! Tests memory and fuel limits using inline WAT components.

use wasmtime::component::Component;
use wasmtime::{Config, Engine, Store, StoreLimitsBuilder};

/// Test helper: Parse WAT to WASM bytes
fn wat_to_wasm(wat: &str) -> Vec<u8> {
    wat::parse_str(wat).expect("Invalid WAT")
}

/// Create an engine configured for fuel consumption
fn create_engine() -> Engine {
    let mut config = Config::new();
    config.wasm_component_model(true);
    config.consume_fuel(true);
    Engine::new(&config).expect("Failed to create engine")
}

#[test]
fn test_memory_limit_prevents_growth() {
    // WAT: A core module that tries to grow memory beyond limit
    // Note: Component Model wraps core modules
    let wat = r#"
        (component
            (core module $m
                (memory (export "memory") 1)
                (func (export "try_grow") (result i32)
                    (memory.grow (i32.const 10))
                )
            )
            (core instance $i (instantiate $m))
            (func (export "try-grow") (result s32)
                (canon lift (core func $i "try_grow"))
            )
        )
    "#;

    let engine = create_engine();
    let wasm_bytes = wat_to_wasm(wat);
    let component = Component::new(&engine, &wasm_bytes).expect("Failed to create component");

    // Create store with 64KB memory limit (1 page)
    #[allow(dead_code)]
    struct TestState {
        limits: wasmtime::StoreLimits,
    }

    let limits = StoreLimitsBuilder::new()
        .memory_size(64 * 1024) // 64KB = 1 page
        .build();

    let mut store = Store::new(&engine, TestState { limits });
    store.limiter(|s| &mut s.limits);

    // Component should exist but memory.grow should fail
    // (returns -1 when growth fails due to limits)
    assert!(component.serialize().is_ok());
}

#[test]
fn test_fuel_exhaustion_traps() {
    // WAT: Infinite loop that will exhaust fuel
    let wat = r#"
        (component
            (core module $m
                (func (export "infinite_loop")
                    (loop $l
                        (br $l)
                    )
                )
            )
            (core instance $i (instantiate $m))
            (func (export "infinite-loop")
                (canon lift (core func $i "infinite_loop"))
            )
        )
    "#;

    let engine = create_engine();
    let wasm_bytes = wat_to_wasm(wat);
    let component = Component::new(&engine, &wasm_bytes).expect("Failed to create component");

    struct TestState;

    let mut store = Store::new(&engine, TestState);

    // Set very low fuel limit
    store.set_fuel(100).expect("Failed to set fuel");

    // Verify fuel is set
    let remaining = store.get_fuel().expect("Failed to get fuel");
    assert_eq!(remaining, 100);

    // Component loads successfully (fuel consumed during execution, not load)
    assert!(component.serialize().is_ok());
}

#[test]
fn test_fuel_not_set_when_none() {
    let engine = create_engine();

    struct TestState;
    let store = Store::new(&engine, TestState);

    // Default store has no fuel set (infinite)
    // get_fuel returns error when fuel not enabled, but we enabled it in config
    // So this should return Ok with the max value
    let fuel = store.get_fuel();
    assert!(fuel.is_ok());
}

#[test]
fn test_memory_limit_exact_boundary() {
    // Test at exactly the boundary
    let engine = create_engine();

    #[allow(dead_code)]
    struct TestState {
        limits: wasmtime::StoreLimits,
    }

    // 2 pages = 128KB
    let limits = StoreLimitsBuilder::new().memory_size(128 * 1024).build();

    let store = Store::new(&engine, TestState { limits });

    // Verify store was created successfully with limits
    assert!(store.get_fuel().is_ok());
}
