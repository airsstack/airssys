// airssys-wasm/tests/host_system-integration-tests.rs
//!
//! Integration tests for host_system module.
//!
//! These tests verify module accessibility and public API functionality from
//! an external integration test context.

use airssys_wasm::host_system::HostSystemManager;

#[tokio::test]
async fn test_host_system_manager_integration() {
    // Test that HostSystemManager can be instantiated from external context
    let manager = HostSystemManager::new().await;
    assert!(manager.is_ok(), "HostSystemManager should instantiate");

    let manager = manager.unwrap();

    // Test Debug trait implementation (integration-level verification)
    let debug_str = format!("{:?}", manager);
    assert!(!debug_str.is_empty(), "Debug output should not be empty");
}

#[tokio::test]
async fn test_module_accessibility() {
    // Test that all public types are accessible from integration context
    // This verifies module structure and public API surface
    use airssys_wasm::host_system::HostSystemManager;

    // Verify we can construct types
    let manager = HostSystemManager::new().await;
    assert!(manager.is_ok(), "Module API should be accessible");
}

#[tokio::test]
async fn test_module_wiring() {
    // Test that host_system module is properly wired in lib.rs
    // This verifies the module is publicly exposed
    use airssys_wasm::host_system::HostSystemManager;

    // If this compiles, the module is properly wired
    let _manager = HostSystemManager::new().await;
}
