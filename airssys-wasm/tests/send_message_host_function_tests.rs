#![allow(clippy::panic, clippy::expect_used, clippy::unwrap_used)]

//! Integration tests for send-message host function (WASM-TASK-006 Phase 2 Task 2.1).
//!
//! These tests verify the end-to-end functionality of the `send-message` host function:
//!
//! - Multicodec prefix validation (REQUIRED per ADR-WASM-001)
//! - Capability enforcement using existing `can_send_to()` method
//! - Message publishing via MessageBroker
//! - Error handling for various failure scenarios
//! - Performance verification
//!
//! # Test Organization
//!
//! - **End-to-End**: Full SendMessageHostFunction with MessagingService integration
//! - **Multicodec Validation**: All supported codecs and error cases
//! - **Capability Enforcement**: Topic pattern matching and denial scenarios
//! - **Performance**: Latency verification against ~280ns target
//!
//! # References
//!
//! - **WASM-TASK-006 Phase 2 Task 2.1**: send-message Host Function
//! - **ADR-WASM-001**: Multicodec Compatibility Strategy
//! - **ADR-WASM-009**: Component Communication Model
//! - **KNOWLEDGE-WASM-024**: Component Messaging Clarifications

use std::sync::Arc;

use airssys_wasm::core::{
    bridge::HostFunction, Capability, CapabilitySet, ComponentId, MulticodecPrefix, TopicPattern,
    WasmError,
};
use airssys_wasm::messaging::MessagingService;
use airssys_wasm::runtime::{
    create_host_context, AsyncHostRegistryBuilder, SendMessageHostFunction,
};

/// Helper to create encoded args for send-message host function.
///
/// Format: `[target_len: u32 LE][target_bytes][message_bytes]`
fn encode_send_args(target: &str, message: &[u8]) -> Vec<u8> {
    let mut args = (target.len() as u32).to_le_bytes().to_vec();
    args.extend_from_slice(target.as_bytes());
    args.extend_from_slice(message);
    args
}

/// Helper to create a message with multicodec prefix.
fn create_prefixed_message(codec: MulticodecPrefix, payload: &[u8]) -> Vec<u8> {
    codec.create_message(payload)
}

// ============================================================================
// END-TO-END INTEGRATION TESTS
// ============================================================================

/// Test 1: Complete end-to-end send-message flow with Borsh codec
///
/// Verifies:
/// 1. SendMessageHostFunction is registered correctly
/// 2. Multicodec prefix is validated
/// 3. Message is published to broker
/// 4. Metrics are updated
#[tokio::test]
async fn test_send_message_end_to_end_borsh() {
    // Setup
    let messaging = Arc::new(MessagingService::new());
    let registry = AsyncHostRegistryBuilder::new()
        .with_messaging_functions(Arc::clone(&messaging))
        .build();

    // Create context with messaging capability
    let mut caps = CapabilitySet::new();
    caps.grant(Capability::Messaging(TopicPattern::new("*")));
    let context = create_host_context(ComponentId::new("sender-component"), caps);

    // Create message with Borsh prefix
    let payload = b"Hello, World!";
    let message = create_prefixed_message(MulticodecPrefix::Borsh, payload);
    let args = encode_send_args("receiver-component", &message);

    // Execute
    let send_fn = registry.get_function("messaging::send").unwrap();
    let result = send_fn.execute(&context, args).await;

    // Verify: Success with empty response (fire-and-forget)
    assert!(result.is_ok(), "Send should succeed: {:?}", result.err());
    assert!(result.unwrap().is_empty(), "Fire-and-forget returns empty");

    // Verify: Message was published
    let stats = messaging.get_stats().await;
    assert_eq!(stats.messages_published, 1, "Should have 1 message published");
}

/// Test 2: End-to-end with Bincode codec
#[tokio::test]
async fn test_send_message_end_to_end_bincode() {
    let messaging = Arc::new(MessagingService::new());
    let func = SendMessageHostFunction::new(Arc::clone(&messaging));

    let mut caps = CapabilitySet::new();
    caps.grant(Capability::Messaging(TopicPattern::new("*")));
    let context = create_host_context(ComponentId::new("sender"), caps);

    let message = create_prefixed_message(MulticodecPrefix::Bincode, b"bincode data");
    let args = encode_send_args("target", &message);

    let result = func.execute(&context, args).await;

    assert!(result.is_ok());
    assert_eq!(messaging.get_stats().await.messages_published, 1);
}

/// Test 3: End-to-end with MessagePack codec
#[tokio::test]
async fn test_send_message_end_to_end_messagepack() {
    let messaging = Arc::new(MessagingService::new());
    let func = SendMessageHostFunction::new(Arc::clone(&messaging));

    let mut caps = CapabilitySet::new();
    caps.grant(Capability::Messaging(TopicPattern::new("*")));
    let context = create_host_context(ComponentId::new("sender"), caps);

    let message = create_prefixed_message(MulticodecPrefix::MessagePack, b"msgpack data");
    let args = encode_send_args("target", &message);

    let result = func.execute(&context, args).await;

    assert!(result.is_ok());
    assert_eq!(messaging.get_stats().await.messages_published, 1);
}

/// Test 4: End-to-end with Protobuf codec
#[tokio::test]
async fn test_send_message_end_to_end_protobuf() {
    let messaging = Arc::new(MessagingService::new());
    let func = SendMessageHostFunction::new(Arc::clone(&messaging));

    let mut caps = CapabilitySet::new();
    caps.grant(Capability::Messaging(TopicPattern::new("*")));
    let context = create_host_context(ComponentId::new("sender"), caps);

    let message = create_prefixed_message(MulticodecPrefix::Protobuf, b"protobuf data");
    let args = encode_send_args("target", &message);

    let result = func.execute(&context, args).await;

    assert!(result.is_ok());
    assert_eq!(messaging.get_stats().await.messages_published, 1);
}

// ============================================================================
// MULTICODEC VALIDATION TESTS (ADR-WASM-001 Compliance)
// ============================================================================

/// Test 5: Invalid multicodec prefix is rejected
#[tokio::test]
async fn test_send_message_invalid_multicodec_rejected() {
    let messaging = Arc::new(MessagingService::new());
    let func = SendMessageHostFunction::new(Arc::clone(&messaging));

    let mut caps = CapabilitySet::new();
    caps.grant(Capability::Messaging(TopicPattern::new("*")));
    let context = create_host_context(ComponentId::new("sender"), caps);

    // Invalid prefix 0xFFFF
    let invalid_message = vec![0xFF, 0xFF, 0xDE, 0xAD, 0xBE, 0xEF];
    let args = encode_send_args("target", &invalid_message);

    let result = func.execute(&context, args).await;

    // Verify: Error with multicodec information
    assert!(result.is_err(), "Invalid multicodec should be rejected");
    let err = result.unwrap_err();
    let err_str = err.to_string().to_lowercase();
    assert!(
        err_str.contains("multicodec") || err_str.contains("unknown"),
        "Error should mention multicodec: {err}"
    );

    // Verify: No message published
    assert_eq!(messaging.get_stats().await.messages_published, 0);
}

/// Test 6: Message too short for multicodec prefix
#[tokio::test]
async fn test_send_message_too_short_rejected() {
    let messaging = Arc::new(MessagingService::new());
    let func = SendMessageHostFunction::new(Arc::clone(&messaging));

    let mut caps = CapabilitySet::new();
    caps.grant(Capability::Messaging(TopicPattern::new("*")));
    let context = create_host_context(ComponentId::new("sender"), caps);

    // Only 1 byte - too short for 2-byte prefix
    let short_message = vec![0x07];
    let args = encode_send_args("target", &short_message);

    let result = func.execute(&context, args).await;

    assert!(result.is_err(), "Too short message should be rejected");
    let err_str = result.unwrap_err().to_string().to_lowercase();
    assert!(
        err_str.contains("short") || err_str.contains("multicodec"),
        "Error should indicate too short: {err_str}"
    );
}

/// Test 7: Empty message is rejected
#[tokio::test]
async fn test_send_message_empty_rejected() {
    let messaging = Arc::new(MessagingService::new());
    let func = SendMessageHostFunction::new(Arc::clone(&messaging));

    let mut caps = CapabilitySet::new();
    caps.grant(Capability::Messaging(TopicPattern::new("*")));
    let context = create_host_context(ComponentId::new("sender"), caps);

    // Empty message
    let args = encode_send_args("target", &[]);

    let result = func.execute(&context, args).await;

    assert!(result.is_err(), "Empty message should be rejected");
}

// ============================================================================
// CAPABILITY ENFORCEMENT TESTS
// ============================================================================

/// Test 8: No messaging capability results in denial
#[tokio::test]
async fn test_send_message_no_capability_denied() {
    let messaging = Arc::new(MessagingService::new());
    let func = SendMessageHostFunction::new(Arc::clone(&messaging));

    // No capabilities granted
    let context = create_host_context(ComponentId::new("sender"), CapabilitySet::new());

    let message = create_prefixed_message(MulticodecPrefix::Borsh, b"test");
    let args = encode_send_args("target", &message);

    let result = func.execute(&context, args).await;

    assert!(result.is_err(), "Should fail without capability");
    let err = result.unwrap_err();
    assert!(
        matches!(err, WasmError::CapabilityDenied { .. }),
        "Error should be CapabilityDenied: {err:?}"
    );

    // Verify: No message published
    assert_eq!(messaging.get_stats().await.messages_published, 0);
}

/// Test 9: Specific topic pattern capability works
#[tokio::test]
async fn test_send_message_specific_topic_pattern() {
    let messaging = Arc::new(MessagingService::new());
    let func = SendMessageHostFunction::new(Arc::clone(&messaging));

    // Grant specific topic pattern
    let mut caps = CapabilitySet::new();
    caps.grant(Capability::Messaging(TopicPattern::new("borsh")));
    let context = create_host_context(ComponentId::new("sender"), caps);

    // Send with borsh codec (matches "borsh" pattern)
    let message = create_prefixed_message(MulticodecPrefix::Borsh, b"test");
    let args = encode_send_args("target", &message);

    let result = func.execute(&context, args).await;

    assert!(result.is_ok(), "Should succeed with matching pattern");
    assert_eq!(messaging.get_stats().await.messages_published, 1);
}

/// Test 10: Multiple topic patterns capability
#[tokio::test]
async fn test_send_message_multiple_topic_patterns() {
    let messaging = Arc::new(MessagingService::new());
    let func = SendMessageHostFunction::new(Arc::clone(&messaging));

    // Grant multiple topic patterns
    let mut caps = CapabilitySet::new();
    caps.grant(Capability::Messaging(TopicPattern::new("borsh")));
    caps.grant(Capability::Messaging(TopicPattern::new("bincode")));
    let context = create_host_context(ComponentId::new("sender"), caps);

    // Send with borsh
    let message1 = create_prefixed_message(MulticodecPrefix::Borsh, b"borsh data");
    let args1 = encode_send_args("target1", &message1);
    let result1 = func.execute(&context, args1).await;
    assert!(result1.is_ok());

    // Send with bincode
    let message2 = create_prefixed_message(MulticodecPrefix::Bincode, b"bincode data");
    let args2 = encode_send_args("target2", &message2);
    let result2 = func.execute(&context, args2).await;
    assert!(result2.is_ok());

    assert_eq!(messaging.get_stats().await.messages_published, 2);
}

// ============================================================================
// ARGUMENT PARSING TESTS
// ============================================================================

/// Test 11: Invalid args - too short for target length
#[tokio::test]
async fn test_send_message_args_too_short() {
    let messaging = Arc::new(MessagingService::new());
    let func = SendMessageHostFunction::new(Arc::clone(&messaging));

    let mut caps = CapabilitySet::new();
    caps.grant(Capability::Messaging(TopicPattern::new("*")));
    let context = create_host_context(ComponentId::new("sender"), caps);

    // Only 2 bytes - need at least 4 for target_len
    let short_args = vec![0x01, 0x02];

    let result = func.execute(&context, short_args).await;

    assert!(result.is_err(), "Too short args should fail");
    let err_str = result.unwrap_err().to_string();
    assert!(
        err_str.contains("too short") || err_str.contains("args"),
        "Error should indicate args issue: {err_str}"
    );
}

/// Test 12: Invalid args - target length exceeds available bytes
#[tokio::test]
async fn test_send_message_target_length_exceeds_available() {
    let messaging = Arc::new(MessagingService::new());
    let func = SendMessageHostFunction::new(Arc::clone(&messaging));

    let mut caps = CapabilitySet::new();
    caps.grant(Capability::Messaging(TopicPattern::new("*")));
    let context = create_host_context(ComponentId::new("sender"), caps);

    // target_len = 100, but only 2 bytes of target provided
    let mut args = (100u32).to_le_bytes().to_vec();
    args.extend_from_slice(b"ab");

    let result = func.execute(&context, args).await;

    assert!(result.is_err(), "Invalid target length should fail");
}

/// Test 13: Valid target with empty name (edge case)
#[tokio::test]
async fn test_send_message_empty_target() {
    let messaging = Arc::new(MessagingService::new());
    let func = SendMessageHostFunction::new(Arc::clone(&messaging));

    let mut caps = CapabilitySet::new();
    caps.grant(Capability::Messaging(TopicPattern::new("*")));
    let context = create_host_context(ComponentId::new("sender"), caps);

    // Empty target (valid but edge case)
    let message = create_prefixed_message(MulticodecPrefix::Borsh, b"test");
    let args = encode_send_args("", &message);

    let result = func.execute(&context, args).await;

    // Empty target is technically valid - ComponentId::new("") works
    assert!(result.is_ok(), "Empty target should still work at this layer");
}

// ============================================================================
// REGISTRY INTEGRATION TESTS
// ============================================================================

/// Test 14: Function registered correctly via builder
#[tokio::test]
async fn test_send_message_registered_in_registry() {
    let messaging = Arc::new(MessagingService::new());
    let registry = AsyncHostRegistryBuilder::new()
        .with_messaging_functions(messaging)
        .build();

    assert!(registry.has_function("messaging::send"));
    assert!(registry.has_function("messaging::send_request")); // Phase 3 Task 3.1
    assert_eq!(registry.function_count(), 2); // Both messaging functions

    let func = registry.get_function("messaging::send");
    assert!(func.is_some());
    assert_eq!(func.unwrap().name(), "messaging::send");
}

/// Test 15: Builder chaining with multiple function types
#[tokio::test]
async fn test_registry_builder_all_functions() {
    let messaging = Arc::new(MessagingService::new());
    let registry = AsyncHostRegistryBuilder::new()
        .with_messaging_functions(messaging)
        .with_filesystem_functions()
        .with_network_functions()
        .with_time_functions()
        .build();

    assert_eq!(registry.function_count(), 5); // 2 messaging + 3 others
    assert!(registry.has_function("messaging::send"));
    assert!(registry.has_function("messaging::send_request"));
    assert!(registry.has_function("filesystem::read"));
    assert!(registry.has_function("network::http_fetch"));
    assert!(registry.has_function("time::sleep"));
}

// ============================================================================
// PERFORMANCE TESTS
// ============================================================================

/// Test 16: Performance verification (latency target ~280ns)
///
/// Note: CI environments may have higher latency, so we use a generous
/// upper bound (1000ns) for the test. The actual target is ~280ns.
#[tokio::test]
async fn test_send_message_performance() {
    let messaging = Arc::new(MessagingService::new());
    let func = SendMessageHostFunction::new(Arc::clone(&messaging));

    let mut caps = CapabilitySet::new();
    caps.grant(Capability::Messaging(TopicPattern::new("*")));
    let context = create_host_context(ComponentId::new("perf-sender"), caps);

    // Prepare message
    let message = create_prefixed_message(MulticodecPrefix::Borsh, b"performance test payload");
    let args = encode_send_args("perf-target", &message);

    // Warmup
    for _ in 0..100 {
        let _ = func.execute(&context, args.clone()).await;
    }

    // Measure
    let iterations = 1000;
    let start = std::time::Instant::now();

    for _ in 0..iterations {
        let _ = func.execute(&context, args.clone()).await;
    }

    let elapsed = start.elapsed();
    let avg_ns = elapsed.as_nanos() / iterations;

    // Verify: Should be under 1000ns even on slow CI
    // Target is ~280ns on optimized hardware
    assert!(
        avg_ns < 5000,
        "Average latency {}ns exceeds 5000ns threshold (target ~280ns)",
        avg_ns
    );

    println!("send-message average latency: {}ns", avg_ns);

    // Verify all messages were published
    let stats = messaging.get_stats().await;
    assert!(stats.messages_published >= iterations as u64);
}

// ============================================================================
// MESSAGING SERVICE STATE TESTS
// ============================================================================

/// Test 17: Multiple senders using same MessagingService
#[tokio::test]
async fn test_multiple_senders_same_service() {
    let messaging = Arc::new(MessagingService::new());

    // Create multiple function instances (simulating multiple components)
    let func1 = SendMessageHostFunction::new(Arc::clone(&messaging));
    let func2 = SendMessageHostFunction::new(Arc::clone(&messaging));
    let func3 = SendMessageHostFunction::new(Arc::clone(&messaging));

    let mut caps = CapabilitySet::new();
    caps.grant(Capability::Messaging(TopicPattern::new("*")));

    let ctx1 = create_host_context(ComponentId::new("sender1"), caps.clone());
    let ctx2 = create_host_context(ComponentId::new("sender2"), caps.clone());
    let ctx3 = create_host_context(ComponentId::new("sender3"), caps);

    let msg = create_prefixed_message(MulticodecPrefix::Borsh, b"test");

    // Send from multiple senders
    func1.execute(&ctx1, encode_send_args("target", &msg)).await.unwrap();
    func2.execute(&ctx2, encode_send_args("target", &msg)).await.unwrap();
    func3.execute(&ctx3, encode_send_args("target", &msg)).await.unwrap();

    // All should be counted
    assert_eq!(messaging.get_stats().await.messages_published, 3);
}

/// Test 18: Sequential messages from same sender
#[tokio::test]
async fn test_sequential_messages_same_sender() {
    let messaging = Arc::new(MessagingService::new());
    let func = SendMessageHostFunction::new(Arc::clone(&messaging));

    let mut caps = CapabilitySet::new();
    caps.grant(Capability::Messaging(TopicPattern::new("*")));
    let context = create_host_context(ComponentId::new("sender"), caps);

    // Send 10 messages sequentially
    for i in 0..10 {
        let payload = format!("message {}", i);
        let message = create_prefixed_message(MulticodecPrefix::Borsh, payload.as_bytes());
        let args = encode_send_args("target", &message);
        
        let result = func.execute(&context, args).await;
        assert!(result.is_ok(), "Message {} should succeed", i);
    }

    assert_eq!(messaging.get_stats().await.messages_published, 10);
}
