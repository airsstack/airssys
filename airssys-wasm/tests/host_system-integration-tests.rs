// airssys-wasm/tests/host_system-integration-tests.rs
//!
//! Integration tests for host_system module.
//!
//! These tests verify module accessibility and public API functionality from
//! an external integration test context.

use airssys_wasm::host_system::HostSystemManager;
use airssys_wasm::core::WasmError;

#[tokio::test]
async fn test_host_system_manager_integration() {
    // Test: HostSystemManager::new() returns placeholder error (Subtask 4.1)
    // Note: Subtask 4.2 will implement initialization, after which new() will succeed

    let manager = HostSystemManager::new().await;

    // Subtask 4.1: Expect error (fields added but not initialized yet)
    assert!(manager.is_err(),
        "Subtask 4.1: HostSystemManager::new() should return error (not yet implemented)");

    let err = manager.unwrap_err();

    // Verify error is Internal variant with explanation
    assert!(matches!(err, WasmError::Internal { .. }),
        "Error should be Internal variant explaining Subtask 4.2");

    // Verify error message mentions Subtask 4.2
    if let WasmError::Internal { reason, .. } = err {
        assert!(reason.contains("Subtask 4.2"),
            "Error message should mention Subtask 4.2: {}", reason);
    }
}

#[tokio::test]
async fn test_module_accessibility() {
    // Test: Module API is accessible (even if new() returns error)
    // Note: This verifies module structure is correct, not that new() succeeds

    use airssys_wasm::host_system::HostSystemManager;

    // Subtask 4.1: Expect error (new() is placeholder)
    let manager = HostSystemManager::new().await;
    assert!(manager.is_err(),
        "Subtask 4.1: HostSystemManager::new() should return error");
}

#[tokio::test]
async fn test_module_wiring() {
    // Test: Module is properly wired in lib.rs (compiles correctly)
    // Note: This is a compile-time check - if this compiles, wiring is correct

    use airssys_wasm::host_system::HostSystemManager;

    // Subtask 4.1: Expect error (new() is placeholder)
    let result = HostSystemManager::new().await;

    // Just verify we can call new() (module is accessible)
    let _ = result;  // Result may be error, that's OK for Subtask 4.1
}
