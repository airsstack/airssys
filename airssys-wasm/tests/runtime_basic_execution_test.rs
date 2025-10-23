//! Integration tests for basic WASM component execution.
//!
//! Tests end-to-end component loading workflow with real WASM fixtures.
//!
//! ## Test Fixture Strategy
//!
//! This test suite compiles WASM binaries from WAT (WebAssembly Text) sources
//! at test runtime using the `wat` crate. This approach:
//! - Keeps binary artifacts out of git (cleaner history)
//! - Makes fixtures human-readable and reviewable (WAT is text)
//! - Ensures reproducible builds (pinned `wat` crate version)
//! - Works seamlessly in CI/CD (no external build tools required)

// Allow panic-style testing in test code (workspace lint exceptions)
#![allow(clippy::unwrap_used)]
#![allow(clippy::expect_used)]
#![allow(clippy::panic)]

// Layer 1: Standard library imports (§2.1 - 3-layer import organization)
use std::path::PathBuf;

// Layer 2: External crate imports
// (none required)

// Layer 3: Internal module imports
use airssys_wasm::runtime::{ComponentLoader, WasmEngine};

/// Get path to test fixtures directory.
fn fixtures_dir() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures")
}

/// Build WASM binary from WAT source at test runtime.
///
/// Compiles WebAssembly Text format to binary using the `wat` crate.
/// This ensures test fixtures remain human-readable in source control
/// while providing real WASM binaries for integration testing.
///
/// # Arguments
/// * `wat_filename` - Name of WAT file in fixtures directory (e.g., "hello_world.wat")
///
/// # Returns
/// Compiled WASM binary as bytes
///
/// # Panics
/// Panics if WAT file cannot be read or compilation fails (acceptable in tests)
fn build_wasm_from_wat(wat_filename: &str) -> Vec<u8> {
    let wat_path = fixtures_dir().join(wat_filename);
    let wat_source = std::fs::read_to_string(&wat_path)
        .unwrap_or_else(|e| panic!("Failed to read WAT fixture at {wat_path:?}: {e}"));
    
    wat::parse_str(&wat_source)
        .unwrap_or_else(|e| panic!("Failed to compile WAT {wat_filename} to WASM: {e}"))
}

#[tokio::test]
async fn test_engine_initialization() {
    let result = WasmEngine::new();
    
    assert!(
        result.is_ok(),
        "Engine initialization should succeed: {:?}",
        result.err()
    );
}

#[tokio::test]
async fn test_load_valid_component() {
    let engine = WasmEngine::new().expect("Failed to create engine");
    let loader = ComponentLoader::new(&engine);
    
    let wasm_bytes = build_wasm_from_wat("hello_world.wat");
    let result = loader.load_from_bytes(&wasm_bytes).await;
    
    assert!(
        result.is_ok(),
        "Should successfully load hello world component: {:?}",
        result.err()
    );
    
    let bytes = result.unwrap();
    assert!(!bytes.is_empty(), "Component bytes should not be empty");
    assert!(
        bytes.len() < 1024,
        "Hello world component should be small (< 1KB)"
    );
}

#[tokio::test]
async fn test_load_invalid_component() {
    let engine = WasmEngine::new().expect("Failed to create engine");
    let loader = ComponentLoader::new(&engine);
    
    let result = loader.load_from_file("nonexistent.wasm").await;
    
    assert!(
        result.is_err(),
        "Should fail to load non-existent component"
    );
    
    // Verify error message contains path
    if let Err(e) = result {
        let error_msg = e.to_string();
        assert!(
            error_msg.contains("nonexistent.wasm"),
            "Error should mention the file path"
        );
    }
}

#[tokio::test]
async fn test_component_validation() {
    let engine = WasmEngine::new().expect("Failed to create engine");
    let loader = ComponentLoader::new(&engine);
    
    let bytes = build_wasm_from_wat("hello_world.wat");
    
    let result = loader.validate(&bytes);
    
    assert!(
        result.is_ok(),
        "Should validate valid component bytes: {:?}",
        result.err()
    );
}

#[tokio::test]
async fn test_load_component_from_bytes() {
    let engine = WasmEngine::new().expect("Failed to create engine");
    let loader = ComponentLoader::new(&engine);
    
    let wasm_bytes = build_wasm_from_wat("hello_world.wat");
    
    let result = loader.load_from_bytes(&wasm_bytes).await;
    
    assert!(
        result.is_ok(),
        "Should successfully load component from bytes: {:?}",
        result.err()
    );
    
    let bytes = result.unwrap();
    assert!(!bytes.is_empty(), "Component bytes should not be empty");
    assert!(
        bytes.len() < 1024,
        "Hello world component should be small (< 1KB)"
    );
}

#[tokio::test]
async fn test_validate_invalid_bytes() {
    let engine = WasmEngine::new().expect("Failed to create engine");
    let loader = ComponentLoader::new(&engine);
    
    let invalid_bytes = b"INVALID_WASM_DATA";
    let result = loader.validate(invalid_bytes);
    
    assert!(
        result.is_err(),
        "Should fail to validate invalid bytes"
    );
    
    // Verify error message is descriptive
    if let Err(e) = result {
        let error_msg = e.to_string();
        assert!(
            error_msg.contains("magic number") || error_msg.contains("parse"),
            "Error should describe validation failure: {error_msg}"
        );
    }
}

#[tokio::test]
async fn test_load_from_bytes_with_validation() {
    let engine = WasmEngine::new().expect("Failed to create engine");
    let loader = ComponentLoader::new(&engine);
    
    let bytes = build_wasm_from_wat("hello_world.wat");
    
    let result = loader.load_from_bytes(&bytes).await;
    
    assert!(
        result.is_ok(),
        "Should load valid component from bytes: {:?}",
        result.err()
    );
    
    let loaded_bytes = result.unwrap();
    assert_eq!(
        bytes.len(),
        loaded_bytes.len(),
        "Loaded bytes should match input bytes"
    );
}
