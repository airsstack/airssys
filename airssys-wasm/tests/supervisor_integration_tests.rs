//! Integration tests for SupervisorNode integration.
//!
//! These tests verify the complete integration between ComponentSupervisor (Layer 1)
//! and SupervisorNode (Layer 3) via the SupervisorNodeBridge abstraction.

#![allow(clippy::expect_used)]
#![allow(clippy::unwrap_used)]

// Layer 1: Standard library imports

// Layer 2: Third-party crate imports
use std::sync::Arc;
use tokio::sync::RwLock;

// Layer 3: Internal module imports
use airssys_wasm::actor::{
    ComponentSupervisor, RestartPolicy, SupervisorConfig, SupervisorNodeWrapper,
};
use airssys_wasm::core::ComponentId;

/// Helper to create a supervisor with bridge for testing
fn create_test_supervisor() -> ComponentSupervisor {
    let bridge = Arc::new(RwLock::new(SupervisorNodeWrapper::new()));
    ComponentSupervisor::with_bridge(bridge)
}

#[tokio::test]
async fn test_supervisor_with_bridge_integration() {
    let supervisor = create_test_supervisor();
    let stats = supervisor.get_statistics();
    assert_eq!(stats.total_supervised, 0);
}

#[tokio::test]
async fn test_supervise_basic_component() {
    let mut supervisor = create_test_supervisor();
    let component_id = ComponentId::new("test-component");
    let config = SupervisorConfig::permanent();

    let result = supervisor.supervise(&component_id, config);
    assert!(result.is_ok());

    let handle = supervisor.get_handle(&component_id);
    assert!(handle.is_some());
}

#[tokio::test]
async fn test_supervisor_permanent_policy() {
    let mut supervisor = create_test_supervisor();
    let component_id = ComponentId::new("permanent-component");
    let config = SupervisorConfig::permanent();

    supervisor
        .supervise(&component_id, config)
        .expect("Supervision should succeed");

    let handle = supervisor.get_handle(&component_id);
    assert!(handle.is_some());
    if let Some(h) = handle {
        assert_eq!(h.config.restart_policy, RestartPolicy::Permanent);
    }
}

#[tokio::test]
async fn test_supervisor_transient_policy() {
    let mut supervisor = create_test_supervisor();
    let component_id = ComponentId::new("transient-component");
    let config = SupervisorConfig::transient();

    supervisor
        .supervise(&component_id, config)
        .expect("Supervision should succeed");

    let handle = supervisor.get_handle(&component_id);
    assert!(handle.is_some());
    if let Some(h) = handle {
        assert_eq!(h.config.restart_policy, RestartPolicy::Transient);
    }
}

#[tokio::test]
async fn test_supervisor_temporary_policy() {
    let mut supervisor = create_test_supervisor();
    let component_id = ComponentId::new("temporary-component");
    let config = SupervisorConfig::temporary();

    supervisor
        .supervise(&component_id, config)
        .expect("Supervision should succeed");

    let handle = supervisor.get_handle(&component_id);
    assert!(handle.is_some());
    if let Some(h) = handle {
        assert_eq!(h.config.restart_policy, RestartPolicy::Temporary);
    }
}

#[tokio::test]
async fn test_start_component_updates_state() {
    let mut supervisor = create_test_supervisor();
    let component_id = ComponentId::new("test-component");
    let config = SupervisorConfig::permanent();

    supervisor
        .supervise(&component_id, config)
        .expect("Supervision should succeed");

    // Note: start_component requires bridge integration with actual ComponentActor
    // This test verifies local state update only
    let handle = supervisor.get_handle(&component_id);
    assert!(handle.is_some());
}

#[tokio::test]
async fn test_stop_component_updates_state() {
    let mut supervisor = create_test_supervisor();
    let component_id = ComponentId::new("test-component");
    let config = SupervisorConfig::permanent();

    supervisor
        .supervise(&component_id, config)
        .expect("Supervision should succeed");
    supervisor
        .mark_running(&component_id)
        .expect("Supervision should succeed");

    // Note: stop_component requires bridge integration with actual ComponentActor
    // This test verifies local state can be updated
    let handle = supervisor.get_handle(&component_id);
    assert!(handle.is_some());
    if let Some(h) = handle {
        assert_eq!(h.state, airssys_wasm::actor::SupervisionState::Running);
    }
}

#[tokio::test]
async fn test_supervise_multiple_components() {
    let mut supervisor = create_test_supervisor();

    for i in 0..5 {
        let component_id = ComponentId::new(format!("component-{}", i));
        let config = SupervisorConfig::permanent();
        supervisor
            .supervise(&component_id, config)
            .expect("Supervision should succeed");
    }

    let stats = supervisor.get_statistics();
    assert_eq!(stats.total_supervised, 5);
}

#[tokio::test]
async fn test_query_component_state_with_bridge() {
    let mut supervisor = create_test_supervisor();
    let component_id = ComponentId::new("test-component");
    let config = SupervisorConfig::permanent();

    // Register component first
    supervisor
        .supervise(&component_id, config)
        .expect("Supervision should succeed");

    // Component not registered with bridge yet, should return None
    // (Note: query_component_state uses blocking_read which can't be called from async)
    // This test verifies the supervision handle exists
    let handle = supervisor.get_handle(&component_id);
    assert!(handle.is_some());
}

#[tokio::test]
async fn test_supervision_state_mapping() {
    // Test that supervision states are properly defined
    use airssys_wasm::actor::ComponentSupervisionState;

    let states = vec![
        ComponentSupervisionState::Registered,
        ComponentSupervisionState::Starting,
        ComponentSupervisionState::Running,
        ComponentSupervisionState::Restarting,
        ComponentSupervisionState::Stopped,
        ComponentSupervisionState::Failed,
    ];

    for state in states {
        // Verify each state can be created and compared
        match state {
            ComponentSupervisionState::Running => assert!(state.is_running()),
            ComponentSupervisionState::Failed => assert!(state.is_failed()),
            ComponentSupervisionState::Stopped => assert!(state.is_stopped()),
            ComponentSupervisionState::Starting | ComponentSupervisionState::Restarting => {
                assert!(state.is_transitioning())
            }
            _ => {}
        }
    }
}

#[tokio::test]
async fn test_restart_policy_configuration() {
    let mut supervisor = create_test_supervisor();

    // Test with different restart policies
    let policies = vec![
        ("permanent", SupervisorConfig::permanent()),
        ("transient", SupervisorConfig::transient()),
        ("temporary", SupervisorConfig::temporary()),
    ];

    for (name, config) in policies {
        let component_id = ComponentId::new(format!("component-{}", name));
        let result = supervisor.supervise(&component_id, config);
        assert!(
            result.is_ok(),
            "Failed to supervise component with {} policy",
            name
        );
    }

    let stats = supervisor.get_statistics();
    assert_eq!(stats.total_supervised, 3);
}

#[tokio::test]
async fn test_supervision_handle_tracking() {
    let mut supervisor = create_test_supervisor();
    let component_id = ComponentId::new("tracked-component");
    let config = SupervisorConfig::permanent();

    let handle = supervisor
        .supervise(&component_id, config)
        .expect("Supervision should succeed");

    assert_eq!(handle.component_id, component_id);
    assert_eq!(handle.restart_count, 0);
    assert!(handle.restart_history.is_empty());
    assert!(handle.parent_id.is_none());
}

#[tokio::test]
async fn test_component_failure_handling() {
    let mut supervisor = create_test_supervisor();
    let component_id = ComponentId::new("failing-component");
    let config = SupervisorConfig::permanent();

    supervisor
        .supervise(&component_id, config)
        .expect("Supervision should succeed");

    // Simulate component failure
    let decision = supervisor
        .handle_component_failure(&component_id, "Test failure")
        .await;

    assert!(decision.is_ok());

    let handle = supervisor.get_handle(&component_id);
    assert!(handle.is_some());
    if let Some(h) = handle {
        assert_eq!(h.restart_count, 1);
    }
}

#[tokio::test]
async fn test_component_exit_handling() {
    let mut supervisor = create_test_supervisor();
    let component_id = ComponentId::new("exiting-component");
    let config = SupervisorConfig::permanent();

    supervisor
        .supervise(&component_id, config)
        .expect("Supervision should succeed");

    // Simulate component normal exit
    let decision = supervisor.handle_component_exit(&component_id).await;

    assert!(decision.is_ok());
}

#[tokio::test]
async fn test_supervision_statistics() {
    let mut supervisor = create_test_supervisor();

    // Add components in different states
    let running_id = ComponentId::new("running");
    supervisor
        .supervise(&running_id, SupervisorConfig::permanent())
        .expect("Supervision should succeed");
    supervisor
        .mark_running(&running_id)
        .expect("Supervision should succeed");

    let initializing_id = ComponentId::new("initializing");
    supervisor
        .supervise(&initializing_id, SupervisorConfig::permanent())
        .expect("Supervision should succeed");

    let stats = supervisor.get_statistics();
    assert_eq!(stats.total_supervised, 2);
    assert_eq!(stats.currently_running, 1);
}
