// airssys-wasm/tests/host_system-integration-tests.rs
//!
//! Integration tests for host_system module.
//!
//! These tests verify module accessibility and public API functionality from
//! an external integration test context.

use airssys_wasm::host_system::HostSystemManager;

#[tokio::test]
async fn test_host_system_manager_integration() {
    // Test: HostSystemManager::new() succeeds (Subtask 4.2 implemented)
    // Note: Initialization logic is now complete, new() should succeed

    let manager = HostSystemManager::new().await;

    // Subtask 4.2: Expect success (initialization implemented)
    assert!(manager.is_ok(),
        "Subtask 4.2: HostSystemManager::new() should succeed");

    let manager = manager.unwrap();
    assert!(manager.started(), "System should be started after initialization");
}

#[tokio::test]
async fn test_module_accessibility() {
    // Test: Module API is accessible and new() succeeds (Subtask 4.2 implemented)
    // Note: This verifies module structure is correct and initialization works

    use airssys_wasm::host_system::HostSystemManager;

    // Subtask 4.2: Expect success (initialization implemented)
    let manager = HostSystemManager::new().await;
    assert!(manager.is_ok(),
        "Subtask 4.2: HostSystemManager::new() should succeed");

    let manager = manager.unwrap();
    assert!(manager.started(), "System should be started");
}

#[tokio::test]
async fn test_module_wiring() {
    // Test: Module is properly wired in lib.rs and initialization works (Subtask 4.2 implemented)
    // Note: This is a compile-time check - if this compiles, wiring is correct

    use airssys_wasm::host_system::HostSystemManager;

    // Subtask 4.2: Expect success (initialization implemented)
    let result = HostSystemManager::new().await;

    // Verify new() succeeds (module is accessible and initialization works)
    assert!(result.is_ok(), "HostSystemManager::new() should succeed");

    let manager = result.unwrap();
    assert!(manager.started(), "System should be started");
}
