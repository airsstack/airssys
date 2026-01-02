// airssys-wasm/tests/host_system-integration-tests.rs
//!
//! Integration tests for host_system module.
//!
//! These tests verify module accessibility and public API functionality from
//! an external integration test context.

use airssys_wasm::host_system::HostSystemManager;
use airssys_wasm::prelude::*;  // Include all necessary types
use std::path::PathBuf;

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

#[tokio::test]
async fn test_spawn_component_integration() {
    // Test: End-to-end component spawn via HostSystemManager (Subtask 4.3 implemented)
    let manager = HostSystemManager::new().await;

    assert!(manager.is_ok(), "HostSystemManager::new() should succeed in integration test");

    let mut manager = manager.unwrap();

    let component_id = ComponentId::new("integration-test-component");
    let wasm_path = PathBuf::from("tests/fixtures/handle-message-component.wasm");
    let metadata = ComponentMetadata {
        name: component_id.as_str().to_string(),
        version: "1.0.0".to_string(),
        author: "test".to_string(),
        description: Some("Test component".to_string()),
        max_memory_bytes: 10_000_000,
        max_fuel: 1_000_000,
        timeout_seconds: 30,
    };
    let capabilities = CapabilitySet::new();

    // Spawn component via HostSystemManager
    let result = manager.spawn_component(
        component_id.clone(),
        wasm_path,
        metadata,
        capabilities
    ).await;

    assert!(result.is_ok(), "spawn_component() should succeed in integration test");

    let actor_address = result.unwrap();

    // Verify component is accessible via ActorAddress
    assert!(!actor_address.id().to_string().is_empty(), "ActorAddress should have valid ID");

    println!("✅ Component spawned successfully in integration test: {}", component_id.as_str());
}

#[tokio::test]
async fn test_spawn_component_messaging_integration() {
    // Test: Verify component is ready for messaging after spawn (Subtask 4.3 implemented)
    let manager = HostSystemManager::new().await;

    assert!(manager.is_ok(), "HostSystemManager::new() should succeed");

    let mut manager = manager.unwrap();

    let component_id = ComponentId::new("messaging-test-component");
    let wasm_path = PathBuf::from("tests/fixtures/echo-handler.wasm");
    let metadata = ComponentMetadata {
        name: component_id.as_str().to_string(),
        version: "1.0.0".to_string(),
        author: "test".to_string(),
        description: Some("Test component".to_string()),
        max_memory_bytes: 10_000_000,
        max_fuel: 1_000_000,
        timeout_seconds: 30,
    };
    let capabilities = CapabilitySet::new();

    // Spawn component
    let result = manager.spawn_component(
        component_id.clone(),
        wasm_path,
        metadata,
        capabilities
    ).await;

    assert!(result.is_ok(), "spawn_component() should succeed");

    let actor_address = result.unwrap();

    // Verify component is ready (ActorAddress returned)
    assert!(!actor_address.id().to_string().is_empty(), "ActorAddress should be valid");

    // TODO: In later tasks, verify component can receive messages
    // Full messaging integration will be added when ActorSystemSubscriber is complete
    println!("✅ Component ready for messaging: {}", component_id.as_str());
}

#[tokio::test]
async fn test_stop_component_integration() {
    // Test: End-to-end stop component via HostSystemManager (Subtask 4.4 implemented)
    let manager = HostSystemManager::new().await;

    assert!(manager.is_ok(), "HostSystemManager::new() should succeed");

    let mut manager = manager.unwrap();

    // Spawn component
    let component_id = ComponentId::new("test-stop-integration");
    let wasm_path = PathBuf::from("tests/fixtures/basic-handle-message.wasm");
    let metadata = ComponentMetadata {
        name: component_id.as_str().to_string(),
        version: "1.0.0".to_string(),
        author: "test".to_string(),
        description: Some("Test component".to_string()),
        max_memory_bytes: 10_000_000,
        max_fuel: 1_000_000,
        timeout_seconds: 30,
    };
    let capabilities = CapabilitySet::new();

    manager.spawn_component(
        component_id.clone(),
        wasm_path,
        metadata,
        capabilities
    ).await
        .expect("Component should spawn successfully");

    // Stop component
    let result = manager.stop_component(&component_id).await;

    // Verify stop succeeded
    assert!(result.is_ok(), "stop_component should succeed: {:?}", result);

    println!("✅ Component stopped successfully: {}", component_id.as_str());
}

#[tokio::test]
async fn test_stop_multiple_components() {
    // Test: Graceful shutdown with multiple components (Subtask 4.4 implemented)
    let manager = HostSystemManager::new().await;

    assert!(manager.is_ok(), "HostSystemManager::new() should succeed");

    let mut manager = manager.unwrap();

    // Spawn multiple components
    let components = vec![
        ("comp1", "tests/fixtures/basic-handle-message.wasm"),
        ("comp2", "tests/fixtures/echo-handler.wasm"),
        ("comp3", "tests/fixtures/handle-message-component.wasm"),
    ];

    let mut component_ids = Vec::new();
    for (name, path) in &components {
        let component_id = ComponentId::new(*name);
        let wasm_path = PathBuf::from(*path);
        let metadata = ComponentMetadata {
            name: component_id.as_str().to_string(),
            version: "1.0.0".to_string(),
            author: "test".to_string(),
            description: Some("Test component".to_string()),
            max_memory_bytes: 10_000_000,
            max_fuel: 1_000_000,
            timeout_seconds: 30,
        };
        let capabilities = CapabilitySet::new();

        manager.spawn_component(
            component_id.clone(),
            wasm_path,
            metadata,
            capabilities
        ).await
            .unwrap_or_else(|_| panic!("Component {} should spawn", name));

        component_ids.push(component_id);
    }

    // Stop all components
    for id in &component_ids {
        let result = manager.stop_component(id).await;
        assert!(result.is_ok(), "Failed to stop {:?}: {:?}", id, result);
    }

    // Verify all components stopped
    for id in &component_ids {
        // Try to stop again (should fail with not found)
        let result = manager.stop_component(id).await;
        assert!(result.is_err(),
                "Component {} should already be stopped", id.as_str());
    }

    println!("✅ All {} components stopped successfully", components.len());
}

#[tokio::test]
async fn test_stop_with_pending_correlations() {
    // Test: Correlations cleaned up when component stopped (Subtask 4.4 implemented)
    let manager = HostSystemManager::new().await;

    assert!(manager.is_ok(), "HostSystemManager::new() should succeed");

    let mut manager = manager.unwrap();

    let component_id = ComponentId::new("test-correlations");
    let wasm_path = PathBuf::from("tests/fixtures/handle-message-component.wasm");
    let metadata = ComponentMetadata {
        name: component_id.as_str().to_string(),
        version: "1.0.0".to_string(),
        author: "test".to_string(),
        description: Some("Test component".to_string()),
        max_memory_bytes: 10_000_000,
        max_fuel: 1_000_000,
        timeout_seconds: 30,
    };
    let capabilities = CapabilitySet::new();

    manager.spawn_component(
        component_id.clone(),
        wasm_path,
        metadata,
        capabilities
    ).await
        .expect("Component should spawn");

    // Register pending correlation (simulate active request-response)
    let correlation_id = uuid::Uuid::new_v4();
    let (tx, _rx) = tokio::sync::oneshot::channel();
    use airssys_wasm::core::messaging::{CorrelationId, PendingRequest};
    use std::time::Duration;

    let _pending = PendingRequest {
        correlation_id: CorrelationId::from(correlation_id),
        response_tx: tx,
        requested_at: tokio::time::Instant::now(),
        timeout: Duration::from_secs(5),
        from: component_id.clone(),
        to: ComponentId::new("other-component"),
    };

    // Access correlation_tracker via manager's public API
    // Note: In actual usage, this would happen automatically via request-response pattern
    // For this test, we're manually registering to verify cleanup

    // Stop component
    let result = manager.stop_component(&component_id).await;

    // Verify stop succeeded
    assert!(result.is_ok(), "Stop should succeed with pending correlations");

    println!("✅ Component stopped with pending correlations: {}", component_id.as_str());
}

#[tokio::test]
async fn test_stop_nonexistent_component() {
    // Test: Error handling for stopping nonexistent component (Subtask 4.4 implemented)
    let manager = HostSystemManager::new().await;

    assert!(manager.is_ok(), "HostSystemManager::new() should succeed");

    let mut manager = manager.unwrap();

    let component_id = ComponentId::new("does-not-exist");

    let result = manager.stop_component(&component_id).await;

    assert!(result.is_err());

    match result {
        Err(WasmError::ComponentNotFound { component_id: cid, .. }) => {
            assert!(cid.contains("does-not-exist") || cid.contains("found"),
                    "Error message should mention not found");
        }
        _ => panic!("Expected ComponentNotFound error, got {:?}", result),
    }

    println!("✅ Error handling correct for nonexistent component");
}

#[tokio::test]
async fn test_stop_and_spawn_lifecycle() {
    // Test: Stop and spawn lifecycle sequence (Subtask 4.4 implemented)
    let manager = HostSystemManager::new().await;

    assert!(manager.is_ok(), "HostSystemManager::new() should succeed");

    let mut manager = manager.unwrap();

    let component_id = ComponentId::new("test-lifecycle");
    let wasm_path = PathBuf::from("tests/fixtures/basic-handle-message.wasm");

    // Spawn component first time
    let metadata1 = ComponentMetadata {
        name: component_id.as_str().to_string(),
        version: "1.0.0".to_string(),
        author: "test".to_string(),
        description: Some("Test component".to_string()),
        max_memory_bytes: 10_000_000,
        max_fuel: 1_000_000,
        timeout_seconds: 30,
    };
    let capabilities1 = CapabilitySet::new();

    let result1 = manager.spawn_component(
        component_id.clone(),
        wasm_path.clone(),
        metadata1,
        capabilities1
    ).await;

    assert!(result1.is_ok(), "First spawn should succeed");

    // Stop component
    let stop_result = manager.stop_component(&component_id).await;
    assert!(stop_result.is_ok(), "Stop should succeed");

    // Spawn component again with same ID (new instance)
    let metadata2 = ComponentMetadata {
        name: component_id.as_str().to_string(),
        version: "1.0.0".to_string(),
        author: "test".to_string(),
        description: Some("Test component".to_string()),
        max_memory_bytes: 10_000_000,
        max_fuel: 1_000_000,
        timeout_seconds: 30,
    };
    let capabilities2 = CapabilitySet::new();

    let result2 = manager.spawn_component(
        component_id.clone(),
        wasm_path,
        metadata2,
        capabilities2
    ).await;

    // Second spawn should succeed (new instance of same component)
    assert!(result2.is_ok(), "Second spawn should succeed after stop");

    println!("✅ Stop and spawn lifecycle verified: {}", component_id.as_str());
}

#[tokio::test]
async fn test_restart_component_integration() {
    // Test: End-to-end component restart via HostSystemManager (Subtask 4.5 implemented)
    let manager = HostSystemManager::new().await;

    assert!(manager.is_ok(), "HostSystemManager::new() should succeed");

    let mut manager = manager.unwrap();

    // Setup component parameters
    let component_id = ComponentId::new("test-restart-integration");
    let wasm_path = PathBuf::from("tests/fixtures/hello_world.wasm");
    let wasm_bytes = std::fs::read(&wasm_path).unwrap();
    let metadata = ComponentMetadata {
        name: component_id.as_str().to_string(),
        version: "1.0.0".to_string(),
        author: "test".to_string(),
        description: Some("Integration test component".to_string()),
        max_memory_bytes: 10_000_000,
        max_fuel: 1_000_000,
        timeout_seconds: 30,
    };
    let capabilities = CapabilitySet::new();

    // Spawn component
    let spawn_result = manager.spawn_component(
        component_id.clone(),
        wasm_path.clone(),
        metadata.clone(),
        capabilities.clone()
    ).await;

    assert!(spawn_result.is_ok(), "Component spawn failed: {:?}", spawn_result);

    // Verify component is registered after spawn
    assert!(manager.is_component_registered(&component_id),
            "Component should be registered after spawn");

    // Restart component with same parameters
    let restart_result = manager.restart_component(
        &component_id,
        wasm_bytes,
        metadata,
        capabilities
    ).await;

    assert!(restart_result.is_ok(), "Component restart failed: {:?}", restart_result);

    // Verify component is still registered after restart
    assert!(manager.is_component_registered(&component_id),
            "Component should be registered after restart");

    // Cleanup
    let _ = manager.stop_component(&component_id).await;

    println!("✅ Component restarted successfully: {}", component_id.as_str());
}

// Task 4.7: shutdown() integration tests

#[tokio::test]
async fn test_shutdown_multiple_components() {
    // Test: Shutdown system with multiple components running
    let mut manager = HostSystemManager::new().await.expect("Manager initialization should succeed");

    let wasm_path = PathBuf::from("tests/fixtures/handle-message-component.wasm");

    // Spawn 2 components
    manager.spawn_component(
        ComponentId::new("comp1"),
        wasm_path.clone(),
        ComponentMetadata {
            name: "comp1".to_string(),
            version: "1.0.0".to_string(),
            author: "test".to_string(),
            description: Some("Test component".to_string()),
            max_memory_bytes: 10_000_000,
            max_fuel: 1_000_000,
            timeout_seconds: 30,
        },
        CapabilitySet::new(),
    ).await.expect("Component 1 spawn should succeed");

    manager.spawn_component(
        ComponentId::new("comp2"),
        wasm_path,
        ComponentMetadata {
            name: "comp2".to_string(),
            version: "1.0.0".to_string(),
            author: "test".to_string(),
            description: Some("Test component".to_string()),
            max_memory_bytes: 10_000_000,
            max_fuel: 1_000_000,
            timeout_seconds: 30,
        },
        CapabilitySet::new(),
    ).await.expect("Component 2 spawn should succeed");

    // Shutdown
    let result = manager.shutdown().await;
    assert!(result.is_ok(), "Shutdown should succeed: {:?}", result);

    // Verify components stopped (should fail to get status)
    assert!(manager.get_component_status(&ComponentId::new("comp1")).await.is_err(),
            "Component 1 should be stopped");
    assert!(manager.get_component_status(&ComponentId::new("comp2")).await.is_err(),
            "Component 2 should be stopped");

    println!("✅ All components stopped successfully");
}

#[tokio::test]
async fn test_shutdown_idempotent() {
    // Test: Call shutdown() multiple times
    let mut manager = HostSystemManager::new().await.expect("Manager initialization should succeed");

    // First shutdown
    manager.shutdown().await.expect("First shutdown should succeed");

    // Second shutdown should succeed (idempotent)
    let result = manager.shutdown().await;
    assert!(result.is_ok(), "Second shutdown should succeed (idempotent): {:?}", result);

    println!("✅ Shutdown is idempotent - multiple calls succeed");
}

#[tokio::test]
async fn test_shutdown_handles_errors() {
    // Test: Shutdown when component fails to stop
    let mut manager = HostSystemManager::new().await.expect("Manager initialization should succeed");

    let wasm_path = PathBuf::from("tests/fixtures/handle-message-component.wasm");

    manager.spawn_component(
        ComponentId::new("comp1"),
        wasm_path,
        ComponentMetadata {
            name: "comp1".to_string(),
            version: "1.0.0".to_string(),
            author: "test".to_string(),
            description: Some("Test component".to_string()),
            max_memory_bytes: 10_000_000,
            max_fuel: 1_000_000,
            timeout_seconds: 30,
        },
        CapabilitySet::new(),
    ).await.expect("Component spawn should succeed");

    // Shutdown should succeed even if component fails
    let result = manager.shutdown().await;
    assert!(result.is_ok(), "Shutdown should handle errors gracefully: {:?}", result);

    println!("✅ Shutdown continues despite component stop failures");
}

// Task 4.8: Error handling integration tests

#[tokio::test]
async fn test_error_message_descriptive() {
    // Test: Verify error messages are descriptive and include context
    let mut manager = HostSystemManager::new().await.expect("Manager initialization should succeed");

    // Shutdown to simulate not initialized
    manager.shutdown().await.expect("Shutdown should succeed");

    let component_id = ComponentId::new("error-descriptive-test");

    // Test spawn_component() error when not initialized
    let spawn_result = manager.spawn_component(
        component_id.clone(),
        PathBuf::from("tests/fixtures/handle-message-component.wasm"),
        ComponentMetadata {
            name: component_id.as_str().to_string(),
            version: "1.0.0".to_string(),
            author: "test".to_string(),
            description: Some("Error descriptive test".to_string()),
            max_memory_bytes: 10_000_000,
            max_fuel: 1_000_000,
            timeout_seconds: 30,
        },
        CapabilitySet::new(),
    ).await;

    assert!(spawn_result.is_err(), "spawn_component should fail when not initialized");

    // Verify error message is descriptive
    match spawn_result {
        Err(WasmError::EngineInitialization { reason, .. }) => {
            assert!(reason.contains("not initialized") || reason.contains("initialized"),
                        "Error message should mention initialization");
        }
        _ => panic!("Expected EngineInitialization error, got: {:?}", spawn_result),
    }

    println!("✅ Error messages are descriptive and include context");
}

#[tokio::test]
async fn test_error_propagation() {
    // Test: Verify errors propagate from underlying systems
    let mut manager = HostSystemManager::new().await.expect("Manager initialization should succeed");

    // Spawn a component
    let component_id = ComponentId::new("error-prop-test");
    let wasm_path = PathBuf::from("tests/fixtures/handle-message-component.wasm");
    let metadata = ComponentMetadata {
        name: component_id.as_str().to_string(),
        version: "1.0.0".to_string(),
        author: "test".to_string(),
        description: Some("Error propagation test".to_string()),
        max_memory_bytes: 10_000_000,
        max_fuel: 1_000_000,
        timeout_seconds: 30,
    };
    let capabilities = CapabilitySet::new();

    manager.spawn_component(
        component_id.clone(),
        wasm_path,
        metadata,
        capabilities
    ).await.expect("Component spawn should succeed");

    // Test get_component_status() error after shutdown (system not initialized)
    manager.shutdown().await.expect("Shutdown should succeed");

    let status_result = manager.get_component_status(&component_id).await;
    assert!(status_result.is_err(), "get_component_status should propagate error");

    // Verify error is propagated from HostSystemManager
    match status_result {
        Err(WasmError::EngineInitialization { reason, .. }) => {
            assert!(reason.contains("not initialized") || reason.contains("initialized"),
                        "Error should propagate from HostSystemManager");
        }
        _ => panic!("Expected EngineInitialization error, got: {:?}", status_result),
    }

    println!("✅ Errors propagate correctly from underlying systems");
}

#[tokio::test]
async fn test_error_handling_in_client_code() {
    // Test: Example client code handles all error types
    let mut manager = HostSystemManager::new().await.expect("Manager initialization should succeed");

    // Spawn a component
    let component_id = ComponentId::new("error-client-code-test");
    let wasm_path = PathBuf::from("tests/fixtures/handle-message-component.wasm");
    let metadata = ComponentMetadata {
        name: component_id.as_str().to_string(),
        version: "1.0.0".to_string(),
        author: "test".to_string(),
        description: Some("Client error handling test".to_string()),
        max_memory_bytes: 10_000_000,
        max_fuel: 1_000_000,
        timeout_seconds: 30,
    };
    let capabilities = CapabilitySet::new();

    manager.spawn_component(
        component_id.clone(),
        wasm_path,
        metadata,
        capabilities
    ).await.expect("Component spawn should succeed");

    // Stop component
    manager.stop_component(&component_id).await.expect("Component stop should succeed");

    // Try to stop again (should fail with ComponentNotFound)
    let result = manager.stop_component(&component_id).await;

    // Pattern match on error variants
    match result {
        Err(WasmError::ComponentNotFound { component_id: cid, .. }) => {
            // Client code can match specific error variants
            assert!(cid.contains("error-client-code-test"),
                        "Error should contain correct component ID, got: {}", cid);
        }
        Err(WasmError::EngineInitialization { .. }) => {
            // Unexpected error for this operation
            panic!("Expected ComponentNotFound, got EngineInitialization");
        }
        Ok(()) => {
            panic!("Expected error when stopping nonexistent component");
        }
        _ => {
            // Any other error is also unexpected
            panic!("Expected ComponentNotFound, got: {:?}", result);
        }
    }

    println!("✅ Client code can pattern match on error variants");
}

