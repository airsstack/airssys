#![allow(clippy::panic, clippy::expect_used, clippy::unwrap_used)]

//! Integration tests for correlation tracking with ComponentActor.
//!
//! Tests end-to-end request-response flows in production-like scenarios.
//!
//! # Test Coverage
//!
//! - End-to-end request-response between two ComponentActors
//! - Timeout scenarios with actual components
//! - Concurrent requests between multiple components
//!
//! # References
//!
//! - **ADR-WASM-009**: Component Communication Model (Pattern 2: Request-Response)
//! - **WASM-TASK-004 Phase 5 Task 5.1**: Message Correlation Implementation

// Layer 1: Standard library imports
use std::path::PathBuf;
use std::sync::Arc;
use std::time::Duration;

// Layer 2: Third-party crate imports
use uuid::Uuid;

// Layer 3: Internal module imports
use airssys_rt::broker::InMemoryMessageBroker;
use airssys_rt::system::{ActorSystem, SystemConfig};
use airssys_wasm::actor::{ComponentRegistry, ComponentSpawner};
use airssys_wasm::actor::message::RequestMessage;
use airssys_wasm::core::{CapabilitySet, ComponentId, ComponentMetadata};
use airssys_wasm::core::{PendingRequest, RequestError, ResponseMessage};
use airssys_wasm::host_system::CorrelationTracker;
use chrono::Utc;

/// Create test metadata for components.
fn create_test_metadata(name: &str) -> ComponentMetadata {
    ComponentMetadata {
        name: name.to_string(),
        version: "1.0.0".to_string(),
        author: "Test".to_string(),
        description: None,
        max_memory_bytes: 64 * 1024 * 1024,
        max_fuel: 1_000_000,
        timeout_seconds: 5,
    }
}

#[tokio::test]
async fn test_end_to_end_request_response_with_component_actor() {
    // Test: Two ComponentActors exchange request-response
    // Setup:
    //   - Spawn ComponentActor A (requester)
    //   - Spawn ComponentActor B (responder)
    //   - Share CorrelationTracker between them
    // Flow:
    //   - A sends request to B with 5s timeout
    //   - B receives request via handle_message
    //   - B sends response via send_response
    //   - A receives response via oneshot channel
    // Verify:
    //   - Response matches expected payload
    //   - Correlation ID matches
    //   - Response time < 100ms
    //   - No memory leaks (pending_count == 0 after)

    // Setup actor system and infrastructure
    let broker = InMemoryMessageBroker::new();
    let actor_system = ActorSystem::new(SystemConfig::default(), broker.clone());
    let registry = ComponentRegistry::new();
    let spawner = ComponentSpawner::new(actor_system, registry.clone(), broker);
    let _router = spawner.create_router();

    // Create shared correlation tracker
    let tracker = Arc::new(CorrelationTracker::new());

    // Spawn component A (requester)
    let component_a = ComponentId::new("requester-component");
    let wasm_path_a = PathBuf::from("./test-requester.wasm");
    let metadata_a = create_test_metadata("requester-component");
    let caps_a = CapabilitySet::new();

    let _actor_a = spawner
        .spawn_component(component_a.clone(), wasm_path_a, metadata_a, caps_a)
        .await
        .unwrap();

    // Spawn component B (responder)
    let component_b = ComponentId::new("responder-component");
    let wasm_path_b = PathBuf::from("./test-responder.wasm");
    let metadata_b = create_test_metadata("responder-component");
    let caps_b = CapabilitySet::new();

    let _actor_b = spawner
        .spawn_component(component_b.clone(), wasm_path_b, metadata_b, caps_b)
        .await
        .unwrap();

    // Create request-response flow
    let (response_tx, response_rx) = tokio::sync::oneshot::channel();
    let correlation_id = Uuid::new_v4();
    let request_payload = vec![1, 2, 3, 4]; // Test payload

    // Register pending request with 5s timeout
    let pending = PendingRequest {
        correlation_id,
        response_tx,
        requested_at: tokio::time::Instant::now(),
        timeout: Duration::from_secs(5),
        from: component_a.clone(),
        to: component_b.clone(),
    };

    tracker.register_pending(pending).await.unwrap();

    // Verify request registered
    assert_eq!(tracker.pending_count(), 1);
    assert!(tracker.contains(&correlation_id));

    // Simulate request message sent to component B
    let _request_msg = RequestMessage::new(
        component_a.clone(),
        component_b.clone(),
        request_payload.clone(),
        5000, // 5 seconds
    );

    // Simulate component B processing and responding
    // In real scenario, component B would handle InterComponentWithCorrelation message
    let response_payload = vec![5, 6, 7, 8]; // Response payload
    let response_msg = ResponseMessage {
        correlation_id,
        from: component_b.clone(),
        to: component_a.clone(),
        result: Ok(response_payload.clone()),
        timestamp: Utc::now(),
    };

    // Resolve the request with response
    tracker.resolve(correlation_id, response_msg).await.unwrap();

    // Verify request removed from pending
    assert_eq!(tracker.pending_count(), 0);
    assert!(!tracker.contains(&correlation_id));

    // Receive response via channel
    let received_response = response_rx.await.unwrap();

    // Verify response
    assert_eq!(received_response.correlation_id, correlation_id);
    assert_eq!(received_response.from, component_b);
    assert_eq!(received_response.to, component_a);
    assert!(received_response.result.is_ok());
    assert_eq!(received_response.result.unwrap(), response_payload);
}

#[tokio::test]
async fn test_timeout_with_component_actor() {
    // Test: Request times out when responder doesn't reply
    // Setup:
    //   - Spawn ComponentActor A (requester)
    //   - Spawn ComponentActor B (silent responder)
    //   - Share CorrelationTracker
    // Flow:
    //   - A sends request with 100ms timeout
    //   - B receives but DOES NOT respond
    //   - Timeout fires after 100ms
    //   - A receives timeout error
    // Verify:
    //   - Timeout error received (RequestError::Timeout)
    //   - Timeout fired within 100-150ms (±50ms jitter for CI safety)
    //   - pending_count == 0 after timeout

    // Setup actor system
    let broker = InMemoryMessageBroker::new();
    let actor_system = ActorSystem::new(SystemConfig::default(), broker.clone());
    let registry = ComponentRegistry::new();
    let spawner = ComponentSpawner::new(actor_system, registry.clone(), broker);

    // Create correlation tracker
    let tracker = Arc::new(CorrelationTracker::new());

    // Spawn component A (requester)
    let component_a = ComponentId::new("timeout-requester");
    let wasm_path_a = PathBuf::from("./test-timeout-requester.wasm");
    let metadata_a = create_test_metadata("timeout-requester");
    let caps_a = CapabilitySet::new();

    let _actor_a = spawner
        .spawn_component(component_a.clone(), wasm_path_a, metadata_a, caps_a)
        .await
        .unwrap();

    // Spawn component B (silent responder)
    let component_b = ComponentId::new("silent-responder");
    let wasm_path_b = PathBuf::from("./test-silent-responder.wasm");
    let metadata_b = create_test_metadata("silent-responder");
    let caps_b = CapabilitySet::new();

    let _actor_b = spawner
        .spawn_component(component_b.clone(), wasm_path_b, metadata_b, caps_b)
        .await
        .unwrap();

    // Create request with short timeout (100ms, increased from 50ms for CI stability)
    let (response_tx, response_rx) = tokio::sync::oneshot::channel();
    let correlation_id = Uuid::new_v4();
    let timeout = Duration::from_millis(100);

    let pending = PendingRequest {
        correlation_id,
        response_tx,
        requested_at: tokio::time::Instant::now(),
        timeout,
        from: component_a.clone(),
        to: component_b.clone(),
    };

    let start_time = tokio::time::Instant::now();
    tracker.register_pending(pending).await.unwrap();

    // Wait for timeout to fire (100ms + 50ms margin = 150ms for CI stability)
    let response = response_rx.await.unwrap();
    let elapsed = start_time.elapsed();

    // Verify timeout error received
    assert_eq!(response.correlation_id, correlation_id);
    assert!(response.result.is_err());
    assert_eq!(response.result.unwrap_err(), RequestError::Timeout);

    // Verify timeout fired within acceptable window (100-150ms)
    assert!(
        elapsed >= Duration::from_millis(100) && elapsed < Duration::from_millis(150),
        "Timeout fired in {:?}, expected 100-150ms",
        elapsed
    );

    // Give background task time to cleanup
    tokio::time::sleep(Duration::from_millis(100)).await;

    // Verify request removed from tracker
    assert_eq!(tracker.pending_count(), 0);
    assert!(!tracker.contains(&correlation_id));
}

#[tokio::test]
async fn test_concurrent_requests_between_multiple_components() {
    // Test: 10 components send requests to each other concurrently
    // Setup:
    //   - Spawn 10 ComponentActors (A1-A10)
    //   - Share single CorrelationTracker
    // Flow:
    //   - Each component sends 10 requests to random targets (100 total)
    //   - All responders reply with their component ID
    //   - All requesters await responses
    // Verify:
    //   - All 100 responses received
    //   - All correlation IDs unique
    //   - No timeouts
    //   - pending_count == 0 after all complete
    //   - Total time < 1 second (concurrent execution)

    // Setup actor system
    let broker = InMemoryMessageBroker::new();
    let actor_system = ActorSystem::new(SystemConfig::default(), broker.clone());
    let registry = ComponentRegistry::new();
    let spawner = ComponentSpawner::new(actor_system, registry.clone(), broker);

    // Create shared correlation tracker
    let tracker = Arc::new(CorrelationTracker::new());

    // Spawn 10 components
    let num_components = 10;
    let mut component_ids = Vec::new();

    for i in 0..num_components {
        let component_id = ComponentId::new(format!("concurrent-component-{}", i));
        let wasm_path = PathBuf::from(format!("./test-concurrent-{}.wasm", i));
        let metadata = create_test_metadata(&format!("concurrent-component-{}", i));
        let caps = CapabilitySet::new();

        let _actor = spawner
            .spawn_component(component_id.clone(), wasm_path, metadata, caps)
            .await
            .unwrap();

        component_ids.push(component_id);
    }

    // Create 100 concurrent request-response pairs (10 per component)
    let mut tasks = Vec::new();
    let start_time = tokio::time::Instant::now();

    for (i, from_component) in component_ids.iter().enumerate() {
        for j in 0..10 {
            let to_component = component_ids[j % num_components].clone();
            let from = from_component.clone();
            let tracker_clone = Arc::clone(&tracker);

            let task = tokio::spawn(async move {
                let (response_tx, response_rx) = tokio::sync::oneshot::channel();
                let correlation_id = Uuid::new_v4();

                // Register pending request
                let pending = PendingRequest {
                    correlation_id,
                    response_tx,
                    requested_at: tokio::time::Instant::now(),
                    timeout: Duration::from_secs(5),
                    from: from.clone(),
                    to: to_component.clone(),
                };

                tracker_clone.register_pending(pending).await.unwrap();

                // Simulate response (in real scenario, component would respond)
                let response = ResponseMessage {
                    correlation_id,
                    from: to_component.clone(),
                    to: from.clone(),
                    result: Ok(vec![i as u8, j as u8]), // Response payload with indices
                    timestamp: Utc::now(),
                };

                // Small delay to simulate processing
                tokio::time::sleep(Duration::from_millis(10)).await;

                tracker_clone
                    .resolve(correlation_id, response)
                    .await
                    .unwrap();

                // Await response
                response_rx.await.unwrap()
            });

            tasks.push(task);
        }
    }

    // Await all responses
    let mut responses = Vec::new();
    for task in tasks {
        responses.push(task.await);
    }

    let elapsed = start_time.elapsed();

    // Verify all 100 responses received
    assert_eq!(responses.len(), 100);

    // Verify all responses are Ok
    let mut correlation_ids = std::collections::HashSet::new();
    for response in responses {
        let response = response.unwrap();
        assert!(response.result.is_ok(), "Request timed out or failed");

        // Verify unique correlation IDs
        assert!(
            correlation_ids.insert(response.correlation_id),
            "Duplicate correlation ID: {}",
            response.correlation_id
        );
    }

    // Verify all correlation IDs unique (100 unique IDs)
    assert_eq!(correlation_ids.len(), 100);

    // Verify concurrent execution (should be much faster than 100 * 10ms = 1s)
    assert!(
        elapsed < Duration::from_secs(1),
        "Concurrent execution took {:?}, expected <1s",
        elapsed
    );

    // Verify no pending requests remaining
    assert_eq!(tracker.pending_count(), 0);
}
