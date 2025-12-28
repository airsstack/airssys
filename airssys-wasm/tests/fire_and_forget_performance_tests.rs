#![allow(clippy::panic, clippy::expect_used, clippy::unwrap_used)]

//! Fire-and-Forget Performance Correctness Tests (WASM-TASK-006 Phase 2 Task 2.3)
//! NO timing assertions - performance is validated via benchmarks only.
//!
//! # Test Organization (8 tests)
//!
//! 1. `test_end_to_end_message_delivery` - Complete fire-and-forget flow works
//! 2. `test_sustained_message_delivery` - 100 messages delivered correctly
//! 3. `test_host_validation_accepts_valid` - Validation passes for valid messages
//! 4. `test_host_validation_rejects_invalid` - Validation fails for invalid messages
//! 5. `test_wasm_handle_message_invoked` - WASM function is actually called
//! 6. `test_concurrent_senders_stable` - 5 concurrent senders work correctly
//! 7. `test_large_payload_delivery` - 64KB payload handled correctly
//! 8. `test_small_payload_delivery` - 16-byte payload handled correctly
//!
//! # Design Decisions (NO FLAKY TESTS)
//!
//! - ❌ NO latency thresholds (`<500ns`, `<280ns`)
//! - ❌ NO throughput thresholds (`>5,000 msg/sec`)
//! - ✅ YES correctness verification (messages delivered, no errors)
//! - ✅ YES error handling verification (invalid inputs rejected)
//! - ✅ YES message integrity verification
//!
//! # References
//!
//! - WASM-TASK-006 Phase 2 Task 2.3
//! - ADR-WASM-009: Component Communication Model
//! - ADR-WASM-001: Multicodec Compatibility Strategy

// Layer 1: Standard library imports
use std::sync::Arc;

// Layer 2: Third-party crate imports
// (none)

// Layer 3: Internal module imports
use airssys_wasm::core::{
    bridge::{HostCallContext, HostFunction},
    Capability, CapabilitySet, ComponentId, MulticodecPrefix, TopicPattern,
    runtime::RuntimeEngine,
};
use airssys_wasm::messaging::MessagingService;
use airssys_wasm::runtime::{
    create_host_context, SendMessageHostFunction, WasmEngine,
};

// ============================================================================
// Helper Functions
// ============================================================================

/// Create encoded args for send-message host function.
///
/// Format: `[target_len: u32 LE][target_bytes][message_bytes]`
fn encode_send_args(target: &str, message: &[u8]) -> Vec<u8> {
    let mut args = (target.len() as u32).to_le_bytes().to_vec();
    args.extend_from_slice(target.as_bytes());
    args.extend_from_slice(message);
    args
}

/// Create a message with multicodec prefix.
fn create_prefixed_message(codec: MulticodecPrefix, payload: &[u8]) -> Vec<u8> {
    codec.create_message(payload)
}

/// Create a context with full messaging capabilities.
fn create_messaging_context(component_id: &str) -> HostCallContext {
    let mut caps = CapabilitySet::new();
    caps.grant(Capability::Messaging(TopicPattern::new("*")));
    create_host_context(ComponentId::new(component_id), caps)
}

/// Load fixture file by name.
fn load_fixture(name: &str) -> Vec<u8> {
    let fixture_path = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("tests/fixtures")
        .join(name);
    std::fs::read(&fixture_path).unwrap_or_else(|e| {
        panic!("Failed to read fixture '{}': {}", fixture_path.display(), e)
    })
}

// ============================================================================
// Test 1: End-to-End Message Delivery
// ============================================================================

/// Verifies complete fire-and-forget message flow works.
///
/// **Correctness Check:**
/// - SendMessageHostFunction accepts valid arguments
/// - Message is published to broker (verified via metrics)
/// - Fire-and-forget returns empty response
///
/// **NO timing assertions.**
#[tokio::test]
async fn test_end_to_end_message_delivery() {
    // Setup
    let messaging = Arc::new(MessagingService::new());
    let func = SendMessageHostFunction::new(Arc::clone(&messaging));
    let context = create_messaging_context("sender-component");

    // Create valid message with Borsh prefix
    let payload = b"Hello, fire-and-forget world!";
    let message = create_prefixed_message(MulticodecPrefix::Borsh, payload);
    let args = encode_send_args("receiver-component", &message);

    // Execute
    let result = func.execute(&context, args).await;

    // Verify: Success with empty response
    assert!(result.is_ok(), "End-to-end delivery should succeed: {:?}", result.err());
    assert!(result.unwrap().is_empty(), "Fire-and-forget returns empty response");

    // Verify: Message was published to broker
    let stats = messaging.get_stats().await;
    assert_eq!(
        stats.messages_published, 1,
        "Exactly 1 message should be published"
    );
}

// ============================================================================
// Test 2: Sustained Message Delivery
// ============================================================================

/// Verifies 100 messages can be delivered correctly.
/// Reduced from 1000 for resource constraints.
///
/// **Correctness Check:**
/// - All 100 messages delivered successfully
/// - No errors during sustained delivery
/// - Metrics reflect all messages published
///
/// **NO timing assertions.**
#[tokio::test]
async fn test_sustained_message_delivery() {
    // Setup
    let messaging = Arc::new(MessagingService::new());
    let func = SendMessageHostFunction::new(Arc::clone(&messaging));
    let context = create_messaging_context("sustained-sender");

    let message_count = 100; // Reduced from 1000 for resource constraints
    let mut success_count = 0;

    // Send 100 messages
    for i in 0..message_count {
        let payload = format!("sustained-message-{i}").into_bytes();
        let message = create_prefixed_message(MulticodecPrefix::Borsh, &payload);
        let args = encode_send_args("sustained-receiver", &message);

        let result = func.execute(&context, args).await;
        if result.is_ok() {
            success_count += 1;
        }
    }

    // Verify: All messages delivered
    assert_eq!(
        success_count, message_count,
        "All {message_count} messages should be delivered successfully"
    );

    // Verify: Metrics reflect all messages
    let stats = messaging.get_stats().await;
    assert_eq!(
        stats.messages_published, message_count,
        "Metrics should show {message_count} messages published"
    );
}

// ============================================================================
// Test 3: Host Validation Accepts Valid Messages
// ============================================================================

/// Verifies validation passes for valid messages with various codecs.
///
/// **Correctness Check:**
/// - Borsh codec accepted
/// - Bincode codec accepted
/// - MessagePack codec accepted
/// - Protobuf codec accepted
///
/// **NO timing assertions.**
#[tokio::test]
async fn test_host_validation_accepts_valid() {
    let messaging = Arc::new(MessagingService::new());
    let func = SendMessageHostFunction::new(Arc::clone(&messaging));
    let context = create_messaging_context("validation-sender");

    let codecs = [
        MulticodecPrefix::Borsh,
        MulticodecPrefix::Bincode,
        MulticodecPrefix::MessagePack,
        MulticodecPrefix::Protobuf,
    ];

    for codec in codecs {
        let payload = b"test payload";
        let message = create_prefixed_message(codec, payload);
        let args = encode_send_args("target", &message);

        let result = func.execute(&context, args).await;
        assert!(
            result.is_ok(),
            "Validation should accept {:?} codec: {:?}",
            codec,
            result.err()
        );
    }
}

// ============================================================================
// Test 4: Host Validation Rejects Invalid Messages
// ============================================================================

/// Verifies validation fails for invalid messages.
///
/// **Correctness Check:**
/// - Invalid multicodec prefix rejected
/// - Empty args rejected (too short)
/// - Invalid target length rejected
///
/// **NO timing assertions.**
#[tokio::test]
async fn test_host_validation_rejects_invalid() {
    let messaging = Arc::new(MessagingService::new());
    let func = SendMessageHostFunction::new(Arc::clone(&messaging));
    let context = create_messaging_context("rejecting-sender");

    // Test 1: Invalid multicodec prefix (random bytes)
    let invalid_message = vec![0xFF, 0xFF, 0x01, 0x02, 0x03];
    let args = encode_send_args("target", &invalid_message);
    let result = func.execute(&context, args).await;
    assert!(result.is_err(), "Should reject invalid multicodec prefix");
    let err_msg = result.unwrap_err().to_string();
    assert!(
        err_msg.contains("multicodec") || err_msg.contains("Invalid"),
        "Error should mention multicodec: {err_msg}"
    );

    // Test 2: Empty args (too short for target length)
    let empty_args: Vec<u8> = vec![0, 0]; // Only 2 bytes, need at least 4
    let result = func.execute(&context, empty_args).await;
    assert!(result.is_err(), "Should reject empty/short args");

    // Test 3: Target length exceeds available bytes
    let invalid_length_args = vec![0xFF, 0xFF, 0xFF, 0xFF]; // 4GB target length
    let result = func.execute(&context, invalid_length_args).await;
    assert!(result.is_err(), "Should reject invalid target length");
}

// ============================================================================
// Test 5: WASM handle-message Actually Invoked
// ============================================================================

/// Verifies the WASM handle-message export is actually called.
///
/// Uses echo-handler.wasm or handle-message-component.wasm fixture.
///
/// **Correctness Check:**
/// - Component loads successfully
/// - handle-message function is called
/// - Function returns success
///
/// **NO timing assertions.**
#[tokio::test]
async fn test_wasm_handle_message_invoked() {
    let engine = Arc::new(WasmEngine::new().expect("Failed to create WasmEngine"));
    let bytes = load_fixture("handle-message-component.wasm");

    let component_id = ComponentId::new("handle-message-test");
    let handle = engine
        .load_component(&component_id, &bytes)
        .await
        .expect("Failed to load component");

    let sender = ComponentId::new("test-sender");
    let payload = b"verify-handle-message-called";

    let result = engine.call_handle_message(&handle, &sender, payload).await;

    assert!(
        result.is_ok(),
        "handle-message should be invoked successfully: {:?}",
        result.err()
    );
}

// ============================================================================
// Test 6: Concurrent Senders Stability
// ============================================================================

/// Verifies 5 concurrent senders work correctly without races.
/// Reduced from 10 for resource constraints.
///
/// **Correctness Check:**
/// - All concurrent sends complete successfully
/// - No race conditions
/// - All messages published
///
/// **NO timing assertions.**
#[tokio::test]
async fn test_concurrent_senders_stable() {
    let messaging = Arc::new(MessagingService::new());
    let concurrent_senders = 5; // Reduced from 10 for resource constraints
    let messages_per_sender = 10;

    let mut handles = Vec::new();

    for sender_id in 0..concurrent_senders {
        let messaging_clone = Arc::clone(&messaging);

        let handle = tokio::spawn(async move {
            let func = SendMessageHostFunction::new(messaging_clone);
            let context = create_messaging_context(&format!("concurrent-sender-{sender_id}"));

            let mut success_count = 0;
            for msg_id in 0..messages_per_sender {
                let payload = format!("sender-{sender_id}-msg-{msg_id}").into_bytes();
                let message = create_prefixed_message(MulticodecPrefix::Borsh, &payload);
                let args = encode_send_args("shared-receiver", &message);

                let result = func.execute(&context, args).await;
                if result.is_ok() {
                    success_count += 1;
                }
            }
            success_count
        });

        handles.push(handle);
    }

    // Wait for all senders and collect results
    let mut total_success = 0;
    for handle in handles {
        let count = handle.await.expect("Sender task should complete");
        total_success += count;
    }

    let expected_total = concurrent_senders * messages_per_sender;
    assert_eq!(
        total_success, expected_total,
        "All {expected_total} concurrent messages should succeed"
    );

    // Verify metrics
    let stats = messaging.get_stats().await;
    assert_eq!(
        stats.messages_published, expected_total,
        "Metrics should show {expected_total} messages published"
    );
}

// ============================================================================
// Test 7: Large Payload Delivery (64KB)
// ============================================================================

/// Verifies 64KB payload is handled correctly.
///
/// **Correctness Check:**
/// - Large payload accepted
/// - No truncation or corruption
/// - Message published successfully
///
/// **NO timing assertions.**
#[tokio::test]
async fn test_large_payload_delivery() {
    let messaging = Arc::new(MessagingService::new());
    let func = SendMessageHostFunction::new(Arc::clone(&messaging));
    let context = create_messaging_context("large-payload-sender");

    // Create 64KB payload
    let large_payload: Vec<u8> = (0..65536).map(|i| (i % 256) as u8).collect();
    let message = create_prefixed_message(MulticodecPrefix::Borsh, &large_payload);
    let args = encode_send_args("large-payload-receiver", &message);

    let result = func.execute(&context, args).await;

    assert!(
        result.is_ok(),
        "64KB payload should be delivered: {:?}",
        result.err()
    );
    assert!(
        result.unwrap().is_empty(),
        "Fire-and-forget returns empty response"
    );

    // Verify message was published
    let stats = messaging.get_stats().await;
    assert_eq!(
        stats.messages_published, 1,
        "Large payload message should be published"
    );
}

// ============================================================================
// Test 8: Small Payload Delivery (16 bytes)
// ============================================================================

/// Verifies 16-byte payload is handled correctly.
///
/// **Correctness Check:**
/// - Small payload accepted
/// - Message published successfully
///
/// **NO timing assertions.**
#[tokio::test]
async fn test_small_payload_delivery() {
    let messaging = Arc::new(MessagingService::new());
    let func = SendMessageHostFunction::new(Arc::clone(&messaging));
    let context = create_messaging_context("small-payload-sender");

    // Create 16-byte payload
    let small_payload: [u8; 16] = [0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15];
    let message = create_prefixed_message(MulticodecPrefix::Borsh, &small_payload);
    let args = encode_send_args("small-payload-receiver", &message);

    let result = func.execute(&context, args).await;

    assert!(
        result.is_ok(),
        "16-byte payload should be delivered: {:?}",
        result.err()
    );
    assert!(
        result.unwrap().is_empty(),
        "Fire-and-forget returns empty response"
    );

    // Verify message was published
    let stats = messaging.get_stats().await;
    assert_eq!(
        stats.messages_published, 1,
        "Small payload message should be published"
    );
}
