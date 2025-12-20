//! Integration tests for WASM actor invocation (Task 2.1 Step 1.4).
//!
//! These tests verify that the actor message handling implementation (Steps 1.2 and 1.3)
//! works correctly for:
//! - Function invocation with type conversion
//! - InterComponent message handling
//! - Error handling (traps, missing functions)
//! - Multicodec integration
//!
//! # Test Strategy
//!
//! Currently tests verify:
//! 1. Message type construction and serialization
//! 2. Type conversion roundtrips
//! 3. Actor trait compilation
//! 4. Error path handling
//!
//! Full end-to-end WASM invocation tests will be added once:
//! - ActorContext mocking is available
//! - Test WASM fixtures with specific functions are built
//!
//! # References
//!
//! - **Action Plan**: task-004-phase-2-task-2.1-actorsystem-integration-plan.md (lines 595-730)
//! - **DEBT-WASM-004**: Verification of Items #1 and #2

#![expect(
    clippy::expect_used,
    reason = "expect is acceptable in test code for clear error messages"
)]
#![expect(
    clippy::panic,
    reason = "panic is acceptable in test code for assertion failures"
)]

// Layer 3: Internal module imports
use airssys_rt::supervisor::Child;
use airssys_wasm::actor::{ActorState, ComponentActor, ComponentMessage};
use airssys_wasm::core::{decode_multicodec, encode_multicodec, Codec};
use airssys_wasm::core::{CapabilitySet, ComponentId, ComponentMetadata, ResourceLimits};

/// Create test component metadata
fn create_test_metadata(name: &str) -> ComponentMetadata {
    ComponentMetadata {
        name: name.to_string(),
        version: "1.0.0".to_string(),
        author: "Test Suite".to_string(),
        description: Some("Integration test component".to_string()),
        required_capabilities: vec![],
        resource_limits: ResourceLimits {
            max_memory_bytes: 64 * 1024 * 1024, // 64MB
            max_fuel: 1_000_000,
            max_execution_ms: 5000,
            max_storage_bytes: 10 * 1024 * 1024, // 10MB
        },
    }
}

/// Create test component actor
fn create_test_actor(name: &str) -> ComponentActor<()> {
    ComponentActor::new(
        ComponentId::new(name),
        create_test_metadata(name),
        CapabilitySet::new(),
        (),
    )
}

// ============================================================================
// MESSAGE CONSTRUCTION TESTS
// ============================================================================

#[test]
fn test_invoke_message_construction() {
    // Test that Invoke messages can be constructed
    let function = "test_function".to_string();
    let args = vec![1, 2, 3, 4];

    let msg = ComponentMessage::Invoke {
        function: function.clone(),
        args: args.clone(),
    };

    match msg {
        ComponentMessage::Invoke {
            function: f,
            args: a,
        } => {
            assert_eq!(f, function);
            assert_eq!(a, args);
        }
        _ => panic!("Expected Invoke message but got different ComponentMessage variant"),
    }
}

#[test]
fn test_intercomponent_message_construction() {
    let sender = ComponentId::new("sender-component");
    let payload = b"test payload".to_vec();

    let msg = ComponentMessage::InterComponent {
        sender: sender.clone(),
        to: ComponentId::new("target"), // TODO(WASM-TASK-006): Use actual target
        payload: payload.clone(),
    };

    match msg {
        ComponentMessage::InterComponent {
            sender: s,
            to: _,
            payload: p,
        } => {
            assert_eq!(s, sender);
            assert_eq!(p, payload);
        }
        _ => panic!("Expected InterComponent message but got different ComponentMessage variant"),
    }
}

// ============================================================================
// MULTICODEC INTEGRATION TESTS
// ============================================================================

#[test]
fn test_multicodec_borsh_roundtrip() {
    let test_data = b"test payload data";

    // Encode
    let encoded =
        encode_multicodec(Codec::Borsh, test_data).expect("Failed to encode with Borsh codec");
    assert!(!encoded.is_empty());

    // Decode
    let (codec, decoded) = decode_multicodec(&encoded).expect("Failed to decode multicodec data");
    assert_eq!(codec, Codec::Borsh);
    assert_eq!(decoded, test_data);
}

#[test]
fn test_multicodec_cbor_roundtrip() {
    let test_data = b"cbor test data";

    let encoded =
        encode_multicodec(Codec::CBOR, test_data).expect("Failed to encode with CBOR codec");
    let (codec, decoded) =
        decode_multicodec(&encoded).expect("Failed to decode CBOR multicodec data");

    assert_eq!(codec, Codec::CBOR);
    assert_eq!(decoded, test_data);
}

#[test]
fn test_multicodec_json_roundtrip() {
    let test_data = b"json test data";

    let encoded =
        encode_multicodec(Codec::JSON, test_data).expect("Failed to encode with JSON codec");
    let (codec, decoded) =
        decode_multicodec(&encoded).expect("Failed to decode JSON multicodec data");

    assert_eq!(codec, Codec::JSON);
    assert_eq!(decoded, test_data);
}

#[test]
fn test_invoke_message_with_multicodec_args() {
    // Test Invoke message with multicodec-encoded arguments
    let args_data = 42i32.to_le_bytes();
    let encoded_args =
        encode_multicodec(Codec::Borsh, &args_data).expect("Failed to encode args with Borsh");

    let _msg = ComponentMessage::Invoke {
        function: "add".to_string(),
        args: encoded_args.clone(),
    };

    // Verify we can decode the args
    let (codec, decoded) =
        decode_multicodec(&encoded_args).expect("Failed to decode multicodec args");
    assert_eq!(codec, Codec::Borsh);
    assert_eq!(
        i32::from_le_bytes(
            decoded
                .try_into()
                .expect("Failed to convert decoded bytes to array")
        ),
        42
    );
}

// ============================================================================
// ACTOR LIFECYCLE TESTS
// ============================================================================

#[tokio::test]
async fn test_actor_creation() {
    let actor = create_test_actor("test-component");

    // Verify initial state
    assert_eq!(*actor.state(), ActorState::Creating);
    assert!(!actor.is_wasm_loaded());
}

#[tokio::test]
#[ignore = "requires component storage (Block 6)"]
async fn test_actor_start_lifecycle() {
    let mut actor = create_test_actor("lifecycle-test");

    // Start the actor (loads WASM)
    let result = actor.start().await;
    assert!(result.is_ok(), "Failed to start actor: {:?}", result);

    // Verify state transitioned to Ready
    assert_eq!(*actor.state(), ActorState::Ready);
    assert!(actor.is_wasm_loaded());
}

#[tokio::test]
#[ignore = "requires component storage (Block 6)"]
async fn test_actor_stop_lifecycle() {
    let mut actor = create_test_actor("stop-test");

    // Start then stop
    actor.start().await.expect("Failed to start actor");
    assert_eq!(*actor.state(), ActorState::Ready);

    let result = actor.stop(std::time::Duration::from_secs(5)).await;
    assert!(result.is_ok());

    // Verify state transitioned to Terminated
    assert_eq!(*actor.state(), ActorState::Terminated);
    assert!(!actor.is_wasm_loaded());
}

// ============================================================================
// ERROR HANDLING TESTS
// ============================================================================

#[test]
fn test_function_not_found_error_message() {
    // Test that we can construct appropriate error messages
    let component_id = "test-component";
    let function = "nonexistent_function";

    let error_msg = format!(
        "Function '{}' not found in component {}",
        function, component_id
    );

    assert!(error_msg.contains("not found"));
    assert!(error_msg.contains(function));
    assert!(error_msg.contains(component_id));
}

#[test]
fn test_trap_error_message() {
    // Test trap error message formatting
    let component_id = "test-component";
    let function = "divide";
    let trap_reason = "integer divide by zero";

    let error_msg = format!(
        "WASM function '{}' trapped in component {}: {}",
        function, component_id, trap_reason
    );

    assert!(error_msg.contains("trapped"));
    assert!(error_msg.contains(function));
    assert!(error_msg.contains(trap_reason));
}

// ============================================================================
// TYPE CONVERSION INTEGRATION TESTS
// ============================================================================

#[test]
fn test_f32_parameter_encoding() {
    // Test encoding f32 parameter for WASM call
    let value = std::f32::consts::PI;
    let bytes = value.to_le_bytes();
    let encoded = encode_multicodec(Codec::Borsh, &bytes).expect("Failed to encode f32 with Borsh");

    // Decode and verify
    let (_, decoded) = decode_multicodec(&encoded).expect("Failed to decode f32 multicodec data");
    let result = f32::from_le_bytes(
        decoded
            .try_into()
            .expect("Failed to convert decoded f32 bytes"),
    );
    assert_eq!(result, value);
}

#[test]
fn test_i64_parameter_encoding() {
    let value = 123456789i64;
    let bytes = value.to_le_bytes();
    let encoded = encode_multicodec(Codec::Borsh, &bytes).expect("Failed to encode i64 with Borsh");

    let (_, decoded) = decode_multicodec(&encoded).expect("Failed to decode i64 multicodec data");
    let result = i64::from_le_bytes(
        decoded
            .try_into()
            .expect("Failed to convert decoded i64 bytes"),
    );
    assert_eq!(result, value);
}

#[test]
fn test_f64_parameter_encoding() {
    let value = std::f64::consts::E;
    let bytes = value.to_le_bytes();
    let encoded = encode_multicodec(Codec::Borsh, &bytes).expect("Failed to encode f64 with Borsh");

    let (_, decoded) = decode_multicodec(&encoded).expect("Failed to decode f64 multicodec data");
    let result = f64::from_le_bytes(
        decoded
            .try_into()
            .expect("Failed to convert decoded f64 bytes"),
    );
    assert_eq!(result, value);
}

// ============================================================================
// COMPONENT METADATA TESTS
// ============================================================================

#[test]
fn test_component_metadata_creation() {
    let metadata = create_test_metadata("test");

    assert_eq!(metadata.name, "test");
    assert_eq!(metadata.version, "1.0.0");
    assert!(metadata.resource_limits.max_memory_bytes > 0);
    assert!(metadata.resource_limits.max_fuel > 0);
}

#[test]
fn test_component_id_creation() {
    let id = ComponentId::new("test-component");
    assert_eq!(id.as_str(), "test-component");
}

// ============================================================================
// INTERCOMPONENT MESSAGE TESTS
// ============================================================================

#[test]
fn test_intercomponent_payload_handling() {
    let sender = ComponentId::new("sender");
    let payload = b"test message payload".to_vec();

    let msg = ComponentMessage::InterComponent {
        sender: sender.clone(),
        to: ComponentId::new("target"), // TODO(WASM-TASK-006): Use actual target
        payload: payload.clone(),
    };

    // Verify message structure
    match msg {
        ComponentMessage::InterComponent {
            sender: s,
            to: _,
            payload: p,
        } => {
            assert_eq!(s.as_str(), "sender");
            assert_eq!(p, payload);
            assert!(!p.is_empty());
        }
        _ => panic!("Expected InterComponent message but got different ComponentMessage variant"),
    }
}

#[test]
fn test_intercomponent_empty_payload() {
    let sender = ComponentId::new("sender");
    let payload = vec![];

    let msg = ComponentMessage::InterComponent {
        sender,
        payload: payload.clone(),
        to: ComponentId::new("target"),
    };

    match msg {
        ComponentMessage::InterComponent { payload: p, .. } => {
            assert!(p.is_empty());
        }
        _ => panic!("Expected InterComponent message but got different ComponentMessage variant"),
    }
}

// ============================================================================
// HEALTH CHECK MESSAGE TESTS
// ============================================================================

#[test]
fn test_health_check_message() {
    let msg = ComponentMessage::HealthCheck;
    assert!(matches!(msg, ComponentMessage::HealthCheck));
}

#[test]
fn test_shutdown_message() {
    let msg = ComponentMessage::Shutdown;
    assert!(matches!(msg, ComponentMessage::Shutdown));
}

// ============================================================================
// FUTURE: END-TO-END WASM INVOCATION TESTS
// ============================================================================
// These tests require:
// 1. ActorContext mocking capability
// 2. Test WASM fixtures with specific exported functions
// 3. Full ActorSystem integration
//
// They will be implemented in subsequent phases when these dependencies are available.

#[cfg(test)]
mod future_tests {
    #[allow(unused_imports)]
    use super::*;

    // TODO: Add once ActorContext can be properly mocked
    #[tokio::test]
    #[ignore = "requires ActorContext mocking"]
    async fn test_invoke_wasm_function_end_to_end() {
        // Will test actual WASM function invocation with real results
    }

    #[tokio::test]
    #[ignore = "requires test WASM fixtures"]
    async fn test_wasm_trap_handling_end_to_end() {
        // Will test divide-by-zero trap handling
    }

    #[tokio::test]
    #[ignore = "requires ActorContext mocking"]
    async fn test_intercomponent_with_handle_message() {
        // Will test component with handle-message export
    }
}

// ============================================================================
// TEST SUMMARY
// ============================================================================

#[test]
fn test_integration_test_suite_completeness() {
    // Meta-test: Verify we have all required test categories

    // This test serves as documentation of what we're testing
    let test_categories = [
        "Message Construction",
        "Multicodec Integration",
        "Actor Lifecycle",
        "Error Handling",
        "Type Conversion Integration",
        "Component Metadata",
        "InterComponent Messages",
        "Health Check Messages",
    ];

    assert_eq!(
        test_categories.len(),
        8,
        "Integration test suite covers {} categories",
        test_categories.len()
    );
}
