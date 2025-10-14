//! OSL Supervisor Hierarchy Integration Tests
//!
//! Integration tests for OSL (Operating System Layer) supervisor hierarchy with
//! broker-based communication patterns. These tests verify supervisor lifecycle,
//! broker integration, and actor management.
//!
//! # Current Test Coverage
//!
//! 1. **Supervisor Creation** (3 tests)
//!    - OSLSupervisor creation with shared broker
//!    - Actor registration and startup  
//!    - Actor address configuration
//!
//! 2. **Broker Integration** (3 tests)
//!    - Message envelope creation
//!    - Broker publish/subscribe patterns
//!    - Message correlation with request IDs
//!
//! 3. **Lifecycle Management** (2 tests)
//!    - Idempotent start operations
//!    - Supervisor concurrent operation handling
//!
//! # Note on Full Actor Message Flow
//!
//! Full end-to-end actor message processing (request → actor.handle_message() → response)
//! requires actor run loops that:
//! 1. Subscribe to broker messages  
//! 2. Filter messages for their address
//! 3. Call handle_message() with the message
//! 4. Publish responses back to broker
//!
//! This actor runtime enhancement is planned for future tasks. Current tests focus on
//! supervisor lifecycle, broker integration, and message structure validation without
//! requiring full actor message loop processing.

// Layer 1: Standard library imports
use std::collections::HashMap;
use std::path::PathBuf;
use std::str::FromStr;
use std::time::Duration;

// Layer 2: Third-party crate imports
use tokio;

// Layer 3: Internal module imports
use airssys_rt::broker::{InMemoryMessageBroker, MessageBroker};
use airssys_rt::message::MessageEnvelope;
use airssys_rt::osl::actors::messages::{
    FileSystemOperation, FileSystemRequest, NetworkOperation, NetworkRequest, ProcessOperation,
    ProcessRequest,
};
use airssys_rt::osl::supervisor::OSLMessage;
use airssys_rt::osl::OSLSupervisor;
use airssys_rt::util::{ActorAddress, MessageId};

// ============================================================================
// TEST GROUP 1: Supervisor Creation and Lifecycle
// ============================================================================

#[tokio::test]
async fn test_osl_supervisor_creation_with_shared_broker() {
    // Create shared broker
    let broker = InMemoryMessageBroker::<OSLMessage>::new();

    // Create OSLSupervisor with broker injection
    let _supervisor = OSLSupervisor::new(broker.clone());

    // Supervisor created successfully (no panics)
    // Demonstrates ADR-RT-009 broker dependency injection pattern
}

#[tokio::test]
async fn test_osl_supervisor_actor_startup() {
    // Create shared broker and supervisor
    let broker = InMemoryMessageBroker::<OSLMessage>::new();
    let supervisor = OSLSupervisor::new(broker.clone());

    // Start supervisor
    let result = supervisor.start().await;
    assert!(
        result.is_ok(),
        "Supervisor should start successfully: {:?}",
        result
    );

    // Allow actors to initialize
    tokio::time::sleep(Duration::from_millis(100)).await;

    // Verify all three actors started (check logs for "Actor starting" messages)
    // FileSystemActor, ProcessActor, and NetworkActor should all be running
}

#[tokio::test]
async fn test_osl_supervisor_actor_addresses() {
    // Create broker and supervisor
    let broker = InMemoryMessageBroker::<OSLMessage>::new();
    let supervisor = OSLSupervisor::new(broker.clone());

    // Get actor addresses
    let fs_addr = supervisor.filesystem_addr();
    let proc_addr = supervisor.process_addr();
    let net_addr = supervisor.network_addr();

    // Verify addresses are correctly configured (by name, not full address with UUID)
    assert_eq!(
        fs_addr.name(),
        Some("osl-filesystem"),
        "FileSystem actor should have correct name"
    );
    assert_eq!(
        proc_addr.name(),
        Some("osl-process"),
        "Process actor should have correct name"
    );
    assert_eq!(
        net_addr.name(),
        Some("osl-network"),
        "Network actor should have correct name"
    );
}

// ============================================================================
// TEST GROUP 2: Broker Integration
// ============================================================================

#[tokio::test]
async fn test_broker_message_envelope_creation() {
    // Create broker
    let broker = InMemoryMessageBroker::<OSLMessage>::new();
    let _supervisor = OSLSupervisor::new(broker.clone());

    // Create client address
    let client_addr = ActorAddress::named("test-client");

    // Create FileSystem request message
    let request_id = MessageId::new();
    let request = FileSystemRequest {
        operation: FileSystemOperation::ReadFile {
            path: "/etc/hosts".into(),
        },
        reply_to: client_addr.clone(),
        request_id: request_id.clone(),
    };

    // Wrap in OSLMessage enum
    let osl_message = OSLMessage::FileSystemReq(request);

    // Create message envelope
    let envelope = MessageEnvelope::new(osl_message)
        .with_sender(client_addr.clone())
        .with_reply_to(client_addr.clone());

    // Verify envelope structure
    assert_eq!(envelope.sender, Some(client_addr.clone()));
    assert_eq!(envelope.reply_to, Some(client_addr));
}

#[tokio::test]
async fn test_broker_publish_subscribe_pattern() {
    // Create broker
    let broker = InMemoryMessageBroker::<OSLMessage>::new();

    // Subscribe to messages
    let mut subscriber1 = broker.subscribe().await.expect("Failed to subscribe");
    let mut subscriber2 = broker.subscribe().await.expect("Failed to subscribe");

    // Publish a test message
    let client_addr = ActorAddress::named("test-publisher");
    let request_id = MessageId::new();

    let envelope = MessageEnvelope::new(OSLMessage::FileSystemReq(FileSystemRequest {
        operation: FileSystemOperation::ReadFile {
            path: "/test".into(),
        },
        reply_to: client_addr.clone(),
        request_id,
    }))
    .with_sender(client_addr);

    broker
        .publish(envelope.clone())
        .await
        .expect("Failed to publish");

    // Both subscribers should receive the message (pub-sub pattern)
    let msg1 = tokio::time::timeout(Duration::from_millis(100), subscriber1.recv())
        .await
        .expect("Timeout")
        .expect("No message");

    let msg2 = tokio::time::timeout(Duration::from_millis(100), subscriber2.recv())
        .await
        .expect("Timeout")
        .expect("No message");

    // Verify both received the same message
    assert!(matches!(msg1.payload, OSLMessage::FileSystemReq(_)));
    assert!(matches!(msg2.payload, OSLMessage::FileSystemReq(_)));
}

#[tokio::test]
async fn test_message_request_id_correlation() {
    // Create multiple messages with different request IDs
    let client_addr = ActorAddress::named("test-correlation");

    let request_id_1 = MessageId::new();
    let request_id_2 = MessageId::new();

    let req1 = FileSystemRequest {
        operation: FileSystemOperation::ReadFile {
            path: "/file1".into(),
        },
        reply_to: client_addr.clone(),
        request_id: request_id_1.clone(),
    };

    let req2 = FileSystemRequest {
        operation: FileSystemOperation::ReadFile {
            path: "/file2".into(),
        },
        reply_to: client_addr.clone(),
        request_id: request_id_2.clone(),
    };

    // Verify request IDs are unique
    assert_ne!(request_id_1, request_id_2, "Request IDs should be unique");

    // Verify request IDs are preserved in message structures
    assert_eq!(req1.request_id, request_id_1);
    assert_eq!(req2.request_id, request_id_2);
}

// ============================================================================
// TEST GROUP 3: Lifecycle Management
// ============================================================================

#[tokio::test]
async fn test_supervisor_idempotent_start() {
    let broker = InMemoryMessageBroker::<OSLMessage>::new();
    let supervisor = OSLSupervisor::new(broker.clone());

    // First start
    supervisor
        .start()
        .await
        .expect("First start should succeed");

    // Second start (idempotent - should not fail)
    supervisor
        .start()
        .await
        .expect("Second start should succeed (idempotent)");

    // Both starts completed without error
    tokio::time::sleep(Duration::from_millis(100)).await;
}

#[tokio::test]
async fn test_supervisor_concurrent_operations() {
    // Create broker and supervisor
    let broker = InMemoryMessageBroker::<OSLMessage>::new();
    let supervisor = OSLSupervisor::new(broker.clone());
    supervisor
        .start()
        .await
        .expect("Failed to start supervisor");

    tokio::time::sleep(Duration::from_millis(100)).await;

    // Create multiple message envelopes of different types
    let client_addr = ActorAddress::named("test-concurrent");

    let fs_envelope = MessageEnvelope::new(OSLMessage::FileSystemReq(FileSystemRequest {
        operation: FileSystemOperation::ReadFile {
            path: "/etc/hosts".into(),
        },
        reply_to: client_addr.clone(),
        request_id: MessageId::new(),
    }))
    .with_sender(client_addr.clone());

    let proc_envelope = MessageEnvelope::new(OSLMessage::ProcessReq(ProcessRequest {
        operation: ProcessOperation::Spawn {
            program: PathBuf::from("echo"),
            args: vec!["test".to_string()],
            env: HashMap::new(),
            working_dir: None,
        },
        reply_to: client_addr.clone(),
        request_id: MessageId::new(),
    }))
    .with_sender(client_addr.clone());

    let net_envelope = MessageEnvelope::new(OSLMessage::NetworkReq(NetworkRequest {
        operation: NetworkOperation::TcpConnect {
            addr: std::net::SocketAddr::from_str("127.0.0.1:8080").unwrap(),
            timeout: Duration::from_secs(1),
        },
        reply_to: client_addr.clone(),
        request_id: MessageId::new(),
    }))
    .with_sender(client_addr);

    // Publish all three concurrently
    let publish_fs = broker.publish(fs_envelope);
    let publish_proc = broker.publish(proc_envelope);
    let publish_net = broker.publish(net_envelope);

    // All publishes should succeed
    tokio::try_join!(publish_fs, publish_proc, publish_net)
        .expect("All concurrent publishes should succeed");

    // Supervisor and broker handle concurrent operations without errors
}

#[tokio::test]
async fn test_multiple_supervisor_instances() {
    // Create two independent supervisor instances
    let broker1 = InMemoryMessageBroker::<OSLMessage>::new();
    let supervisor1 = OSLSupervisor::new(broker1.clone());

    let broker2 = InMemoryMessageBroker::<OSLMessage>::new();
    let supervisor2 = OSLSupervisor::new(broker2.clone());

    // Start both supervisors
    supervisor1
        .start()
        .await
        .expect("Supervisor 1 should start");
    supervisor2
        .start()
        .await
        .expect("Supervisor 2 should start");

    tokio::time::sleep(Duration::from_millis(100)).await;

    // Verify they have independent addresses (different IDs, same names)
    assert_eq!(
        supervisor1.filesystem_addr().name(),
        supervisor2.filesystem_addr().name(),
        "Both should have same actor name"
    );
    assert_ne!(
        supervisor1.filesystem_addr().id(),
        supervisor2.filesystem_addr().id(),
        "But different actor IDs"
    );
}
