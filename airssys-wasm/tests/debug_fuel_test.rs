//! Debug test to isolate fuel metering issue
#![allow(clippy::unwrap_used)]
#![allow(clippy::expect_used)]
#![allow(clippy::panic)]
#![allow(clippy::uninlined_format_args)]

use std::collections::HashMap;
use std::path::PathBuf;
use wasmtime::{Config, Engine, Store};
use wasmtime::component::{Component, Linker};
use airssys_wasm::core::{
    capability::CapabilitySet,
    component::{ComponentId, ComponentInput, ResourceLimits},
    runtime::{ExecutionContext, RuntimeEngine},
};
use airssys_wasm::runtime::WasmEngine;

fn fixtures_dir() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures")
}

fn build_wasm_from_wat(wat_filename: &str) -> Vec<u8> {
    let wat_path = fixtures_dir().join(wat_filename);
    let wat_source = std::fs::read_to_string(&wat_path)
        .unwrap_or_else(|e| panic!("Failed to read WAT fixture at {wat_path:?}: {e}"));
    
    wat::parse_str(&wat_source)
        .unwrap_or_else(|e| panic!("Failed to compile WAT {wat_filename} to WASM: {e}"))
}

#[tokio::test]
async fn debug_fuel_metering_inline_wat() {
    println!("\n=== DEBUG TEST 1: Inline WAT (working baseline) ===\n");
    
    let mut config = Config::new();
    config.wasm_component_model(true);
    config.async_support(true);
    config.consume_fuel(true);
    
    let engine = Engine::new(&config).expect("Engine creation failed");
    
    let wat = r#"
(component
  (core module $M
    (func (export "hello") (result i32)
      i32.const 42
    )
  )
  (core instance $m (instantiate $M))
  (func (export "hello") (result s32)
    (canon lift (core func $m "hello"))
  )
)
"#;
    let wasm = wat::parse_str(wat).expect("WAT parsing failed");
    println!("WAT parsed: {} bytes", wasm.len());
    
    let component = Component::new(&engine, wasm).expect("Component creation failed");
    let mut store = Store::new(&engine, ());
    store.set_fuel(10_000).expect("Failed to set fuel");
    
    let linker = Linker::new(&engine);
    let instance = linker.instantiate_async(&mut store, &component)
        .await
        .expect("Instantiation failed");
    
    let func = instance.get_typed_func::<(), (i32,)>(&mut store, "hello")
        .expect("Failed to get function");
    
    let (result,) = func.call_async(&mut store, ()).await
        .expect("Function call failed");
    
    println!("✓ Result: {}", result);
    assert_eq!(result, 42);
}

#[tokio::test]
async fn debug_fuel_metering_file_wat() {
    println!("\n=== DEBUG TEST 2: File WAT (testing fixture loading) ===\n");
    
    let mut config = Config::new();
    config.wasm_component_model(true);
    config.async_support(true);
    config.consume_fuel(true);
    
    let engine = Engine::new(&config).expect("Engine creation failed");
    
    let wasm = build_wasm_from_wat("hello_world.wat");
    println!("WAT file parsed: {} bytes", wasm.len());
    
    let component = Component::new(&engine, &wasm).expect("Component creation failed");
    let mut store = Store::new(&engine, ());
    store.set_fuel(10_000).expect("Failed to set fuel");
    
    let linker = Linker::new(&engine);
    let instance = linker.instantiate_async(&mut store, &component)
        .await
        .expect("Instantiation failed");
    
    let func = instance.get_typed_func::<(), (i32,)>(&mut store, "hello")
        .expect("Failed to get function");
    
    let (result,) = func.call_async(&mut store, ()).await
        .expect("Function call failed");
    
    println!("✓ Result: {}", result);
    assert_eq!(result, 42);
}

#[tokio::test]
async fn debug_fuel_metering_via_engine() {
    println!("\n=== DEBUG TEST 3: Via WasmEngine (full integration path) ===\n");
    
    let engine = WasmEngine::new().expect("Failed to create engine");
    
    let component_id = ComponentId::new("debug-test");
    let wasm_bytes = build_wasm_from_wat("hello_world.wat");
    println!("Loading component: {} bytes", wasm_bytes.len());
    
    let handle = engine
        .load_component(&component_id, &wasm_bytes)
        .await
        .expect("Failed to load component");
    println!("✓ Component loaded");
    
    let context = ExecutionContext {
        component_id: component_id.clone(),
        limits: ResourceLimits {
            max_memory_bytes: 1024 * 1024,
            max_fuel: 10_000,
            max_execution_ms: 30_000,
            max_storage_bytes: 0,
        },
        capabilities: CapabilitySet::new(),
        timeout_ms: 30_000,
    };
    
    let input = ComponentInput {
        data: Vec::new(),
        codec: 0,
        metadata: HashMap::new(),
    };
    
    println!("Executing function 'hello'...");
    let result = engine.execute(&handle, "hello", input, context).await;
    
    match result {
        Ok(output) => {
            println!("✓ Execution succeeded");
            let value = output.to_i32();
            println!("Result: {:?}", value);
            assert_eq!(value, Some(42));
        }
        Err(e) => {
            panic!("Execution failed: {:?}", e);
        }
    }
}
