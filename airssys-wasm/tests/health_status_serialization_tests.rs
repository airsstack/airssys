#![allow(clippy::panic, clippy::expect_used, clippy::unwrap_used)]

//! Integration tests for HealthStatus serialization across formats.
//! Borsh, CBOR, and JSON formats, ensuring cross-format compatibility and
//! multicodec round-trip fidelity.
//!
//! # Test Coverage
//!
//! - Borsh serialization/deserialization round-trips
//! - JSON format validation and compatibility
//! - CBOR binary format correctness
//! - Multicodec encoding/decoding
//! - Cross-format interoperability
//! - Performance benchmarking
//!
//! # References
//!
//! - **WASM-TASK-004 Phase 1 Task 1.4**: Health Check Implementation
//! - **ADR-WASM-001**: Inter-Component Communication Design (multicodec)

// Layer 1: Standard library imports
use std::time::Instant;

// Layer 2: Third-party crate imports
use borsh::BorshDeserialize;

// Layer 3: Internal module imports
use airssys_wasm::actor::HealthStatus;
use airssys_wasm::core::multicodec::{decode_multicodec, encode_multicodec, Codec};

// ========================================================================
// Borsh Serialization Tests
// ========================================================================

#[test]
fn test_health_status_borsh_healthy_round_trip() {
    let original = HealthStatus::Healthy;

    // Serialize
    let bytes = borsh::to_vec(&original).expect("Borsh serialization should succeed");
    assert!(!bytes.is_empty());

    // Deserialize
    let decoded =
        HealthStatus::try_from_slice(&bytes).expect("Borsh deserialization should succeed");

    assert_eq!(decoded, original);
}

#[test]
fn test_health_status_borsh_degraded_round_trip() {
    let original = HealthStatus::Degraded {
        reason: "High memory pressure".to_string(),
    };

    let bytes = borsh::to_vec(&original).expect("Borsh serialization should succeed");
    let decoded =
        HealthStatus::try_from_slice(&bytes).expect("Borsh deserialization should succeed");

    assert_eq!(decoded, original);
}

#[test]
fn test_health_status_borsh_unhealthy_round_trip() {
    let original = HealthStatus::Unhealthy {
        reason: "Database connection lost".to_string(),
    };

    let bytes = borsh::to_vec(&original).expect("Borsh serialization should succeed");
    let decoded =
        HealthStatus::try_from_slice(&bytes).expect("Borsh deserialization should succeed");

    assert_eq!(decoded, original);
}

#[test]
fn test_health_status_borsh_empty_reason() {
    let original = HealthStatus::Degraded {
        reason: String::new(),
    };

    let bytes = borsh::to_vec(&original).expect("Borsh serialization should succeed");
    let decoded =
        HealthStatus::try_from_slice(&bytes).expect("Borsh deserialization should succeed");

    assert_eq!(decoded, original);
}

#[test]
fn test_health_status_borsh_long_reason() {
    let long_reason = "A".repeat(1000);
    let original = HealthStatus::Unhealthy {
        reason: long_reason.clone(),
    };

    let bytes = borsh::to_vec(&original).expect("Borsh serialization should succeed");
    let decoded =
        HealthStatus::try_from_slice(&bytes).expect("Borsh deserialization should succeed");

    assert_eq!(decoded, original);
    if let HealthStatus::Unhealthy { reason } = decoded {
        assert_eq!(reason.len(), 1000);
    }
}

#[test]
fn test_health_status_borsh_utf8_reason() {
    let original = HealthStatus::Degraded {
        reason: "日本語 🚀 Здравствуй 你好".to_string(),
    };

    let bytes = borsh::to_vec(&original).expect("Borsh serialization should succeed");
    let decoded =
        HealthStatus::try_from_slice(&bytes).expect("Borsh deserialization should succeed");

    assert_eq!(decoded, original);
}

// ========================================================================
// JSON Serialization Tests
// ========================================================================

#[test]
fn test_health_status_json_healthy() {
    let status = HealthStatus::Healthy;

    let json = serde_json::to_string(&status).expect("JSON serialization should succeed");

    // JSON should be human-readable
    assert!(json.contains("healthy"));

    // Round-trip
    let decoded: HealthStatus =
        serde_json::from_str(&json).expect("JSON deserialization should succeed");
    assert_eq!(decoded, status);
}

#[test]
fn test_health_status_json_degraded() {
    let status = HealthStatus::Degraded {
        reason: "High latency detected".to_string(),
    };

    let json = serde_json::to_string(&status).expect("JSON serialization should succeed");

    assert!(json.contains("degraded"));
    assert!(json.contains("High latency detected"));

    let decoded: HealthStatus =
        serde_json::from_str(&json).expect("JSON deserialization should succeed");
    assert_eq!(decoded, status);
}

#[test]
fn test_health_status_json_unhealthy() {
    let status = HealthStatus::Unhealthy {
        reason: "Service unavailable".to_string(),
    };

    let json = serde_json::to_string(&status).expect("JSON serialization should succeed");

    assert!(json.contains("unhealthy"));
    assert!(json.contains("Service unavailable"));

    let decoded: HealthStatus =
        serde_json::from_str(&json).expect("JSON deserialization should succeed");
    assert_eq!(decoded, status);
}

#[test]
fn test_health_status_json_format_readability() {
    // Verify JSON is human-readable
    let status = HealthStatus::Degraded {
        reason: "Test reason".to_string(),
    };

    let json = serde_json::to_string(&status).expect("JSON serialization should succeed");

    // JSON should contain both the status and reason in some form
    assert!(json.contains("degraded"));
    assert!(json.contains("Test reason"));
}

#[test]
fn test_health_status_json_healthy_format() {
    let status = HealthStatus::Healthy;

    let json_str = serde_json::to_string(&status).expect("JSON serialization should succeed");

    // Healthy should serialize to include the status
    assert!(json_str.contains("healthy"));
}

// ========================================================================
// CBOR Serialization Tests
// ========================================================================

#[test]
fn test_health_status_cbor_healthy_round_trip() {
    let original = HealthStatus::Healthy;

    let bytes = serde_cbor::to_vec(&original).expect("CBOR serialization should succeed");
    let decoded: HealthStatus =
        serde_cbor::from_slice(&bytes).expect("CBOR deserialization should succeed");

    assert_eq!(decoded, original);
}

#[test]
fn test_health_status_cbor_degraded_round_trip() {
    let original = HealthStatus::Degraded {
        reason: "Network congestion".to_string(),
    };

    let bytes = serde_cbor::to_vec(&original).expect("CBOR serialization should succeed");
    let decoded: HealthStatus =
        serde_cbor::from_slice(&bytes).expect("CBOR deserialization should succeed");

    assert_eq!(decoded, original);
}

#[test]
fn test_health_status_cbor_unhealthy_round_trip() {
    let original = HealthStatus::Unhealthy {
        reason: "Critical error".to_string(),
    };

    let bytes = serde_cbor::to_vec(&original).expect("CBOR serialization should succeed");
    let decoded: HealthStatus =
        serde_cbor::from_slice(&bytes).expect("CBOR deserialization should succeed");

    assert_eq!(decoded, original);
}

#[test]
fn test_health_status_cbor_compact() {
    // CBOR should be more compact than JSON for same data
    let status = HealthStatus::Degraded {
        reason: "Testing compactness".to_string(),
    };

    let cbor_bytes = serde_cbor::to_vec(&status).expect("CBOR serialization should succeed");
    let json_bytes = serde_json::to_vec(&status).expect("JSON serialization should succeed");

    // CBOR should generally be smaller than JSON
    assert!(
        cbor_bytes.len() <= json_bytes.len(),
        "CBOR ({} bytes) should be <= JSON ({} bytes)",
        cbor_bytes.len(),
        json_bytes.len()
    );
}

// ========================================================================
// Multicodec Integration Tests
// ========================================================================

#[test]
fn test_multicodec_borsh_round_trip() {
    let original = HealthStatus::Healthy;

    // Encode with Borsh multicodec
    let borsh_bytes = borsh::to_vec(&original).expect("Borsh serialization should succeed");
    let encoded =
        encode_multicodec(Codec::Borsh, &borsh_bytes).expect("Multicodec encoding should succeed");

    // Should have multicodec prefix
    assert!(encoded.len() > borsh_bytes.len());

    // Decode multicodec
    let (codec, payload) = decode_multicodec(&encoded).expect("Multicodec decoding should succeed");
    assert_eq!(codec, Codec::Borsh);
    assert_eq!(payload, borsh_bytes);

    // Deserialize HealthStatus
    let decoded =
        HealthStatus::try_from_slice(&payload).expect("Borsh deserialization should succeed");
    assert_eq!(decoded, original);
}

#[test]
fn test_multicodec_json_round_trip() {
    let original = HealthStatus::Degraded {
        reason: "Testing multicodec JSON".to_string(),
    };

    let json_bytes = serde_json::to_vec(&original).expect("JSON serialization should succeed");
    let encoded =
        encode_multicodec(Codec::JSON, &json_bytes).expect("Multicodec encoding should succeed");

    let (codec, payload) = decode_multicodec(&encoded).expect("Multicodec decoding should succeed");
    assert_eq!(codec, Codec::JSON);

    let decoded: HealthStatus =
        serde_json::from_slice(&payload).expect("JSON deserialization should succeed");
    assert_eq!(decoded, original);
}

#[test]
fn test_multicodec_cbor_round_trip() {
    let original = HealthStatus::Unhealthy {
        reason: "Testing multicodec CBOR".to_string(),
    };

    let cbor_bytes = serde_cbor::to_vec(&original).expect("CBOR serialization should succeed");
    let encoded =
        encode_multicodec(Codec::CBOR, &cbor_bytes).expect("Multicodec encoding should succeed");

    let (codec, payload) = decode_multicodec(&encoded).expect("Multicodec decoding should succeed");
    assert_eq!(codec, Codec::CBOR);

    let decoded: HealthStatus =
        serde_cbor::from_slice(&payload).expect("CBOR deserialization should succeed");
    assert_eq!(decoded, original);
}

#[test]
fn test_multicodec_all_formats_compatible() {
    // Same HealthStatus should deserialize from all formats
    let original = HealthStatus::Degraded {
        reason: "Cross-format test".to_string(),
    };

    // Encode with all formats
    let borsh_bytes = borsh::to_vec(&original).expect("Borsh serialization should succeed");
    let borsh_encoded =
        encode_multicodec(Codec::Borsh, &borsh_bytes).expect("Multicodec encoding should succeed");

    let json_bytes = serde_json::to_vec(&original).expect("JSON serialization should succeed");
    let json_encoded =
        encode_multicodec(Codec::JSON, &json_bytes).expect("Multicodec encoding should succeed");

    let cbor_bytes = serde_cbor::to_vec(&original).expect("CBOR serialization should succeed");
    let cbor_encoded =
        encode_multicodec(Codec::CBOR, &cbor_bytes).expect("Multicodec encoding should succeed");

    // Decode all formats
    let (_, borsh_payload) =
        decode_multicodec(&borsh_encoded).expect("Multicodec decoding should succeed");
    let borsh_decoded =
        HealthStatus::try_from_slice(&borsh_payload).expect("Borsh deserialization should succeed");

    let (_, json_payload) =
        decode_multicodec(&json_encoded).expect("Multicodec decoding should succeed");
    let json_decoded: HealthStatus =
        serde_json::from_slice(&json_payload).expect("JSON deserialization should succeed");

    let (_, cbor_payload) =
        decode_multicodec(&cbor_encoded).expect("Multicodec decoding should succeed");
    let cbor_decoded: HealthStatus =
        serde_cbor::from_slice(&cbor_payload).expect("CBOR deserialization should succeed");

    // All should equal original
    assert_eq!(borsh_decoded, original);
    assert_eq!(json_decoded, original);
    assert_eq!(cbor_decoded, original);
}

// ========================================================================
// Performance Benchmarking Tests
// ========================================================================

#[test]
fn test_serialization_performance_borsh_fastest() {
    let status = HealthStatus::Degraded {
        reason: "Performance test".to_string(),
    };

    const ITERATIONS: usize = 10000;

    // Measure Borsh serialization
    let start = Instant::now();
    for _ in 0..ITERATIONS {
        let _ = borsh::to_vec(&status).expect("Borsh serialization should succeed");
    }
    let borsh_time = start.elapsed();

    // Measure JSON serialization
    let start = Instant::now();
    for _ in 0..ITERATIONS {
        let _ = serde_json::to_vec(&status).expect("JSON serialization should succeed");
    }
    let json_time = start.elapsed();

    // Borsh should be faster than JSON
    assert!(
        borsh_time < json_time,
        "Borsh ({:?}) should be faster than JSON ({:?})",
        borsh_time,
        json_time
    );

    // All should complete quickly (<100ms for 10k ops)
    assert!(borsh_time.as_millis() < 100);
    assert!(json_time.as_millis() < 100);
}

#[test]
fn test_deserialization_performance() {
    let status = HealthStatus::Unhealthy {
        reason: "Deserialization benchmark".to_string(),
    };

    const ITERATIONS: usize = 10000;

    // Prepare encoded data
    let borsh_bytes = borsh::to_vec(&status).expect("Borsh serialization should succeed");
    let json_bytes = serde_json::to_vec(&status).expect("JSON serialization should succeed");
    let cbor_bytes = serde_cbor::to_vec(&status).expect("CBOR serialization should succeed");

    // Measure Borsh deserialization
    let start = Instant::now();
    for _ in 0..ITERATIONS {
        let _ = HealthStatus::try_from_slice(&borsh_bytes)
            .expect("Borsh deserialization should succeed");
    }
    let borsh_time = start.elapsed();

    // Measure JSON deserialization
    let start = Instant::now();
    for _ in 0..ITERATIONS {
        let _: HealthStatus =
            serde_json::from_slice(&json_bytes).expect("JSON deserialization should succeed");
    }
    let json_time = start.elapsed();

    // Measure CBOR deserialization
    let start = Instant::now();
    for _ in 0..ITERATIONS {
        let _: HealthStatus =
            serde_cbor::from_slice(&cbor_bytes).expect("CBOR deserialization should succeed");
    }
    let cbor_time = start.elapsed();

    // Borsh should be fastest
    assert!(
        borsh_time <= json_time,
        "Borsh ({:?}) should be <= JSON ({:?})",
        borsh_time,
        json_time
    );

    // All should complete quickly (<100ms for 10k ops)
    assert!(borsh_time.as_millis() < 100);
    assert!(json_time.as_millis() < 100);
    assert!(cbor_time.as_millis() < 100);
}

#[test]
fn test_multicodec_overhead_minimal() {
    let status = HealthStatus::Healthy;

    let borsh_bytes = borsh::to_vec(&status).expect("Borsh serialization should succeed");
    let encoded =
        encode_multicodec(Codec::Borsh, &borsh_bytes).expect("Multicodec encoding should succeed");

    // Multicodec overhead should be <10 bytes (typically 2-3 bytes for varint)
    let overhead = encoded.len() - borsh_bytes.len();
    assert!(
        overhead < 10,
        "Multicodec overhead is {} bytes (expected <10)",
        overhead
    );
}

// ========================================================================
// Edge Cases and Error Handling
// ========================================================================

#[test]
fn test_health_status_reason_special_characters() {
    let reasons = vec![
        "Newline\ncharacter",
        "Tab\tcharacter",
        "Quote\"character",
        "Backslash\\character",
        "Null\0character",
        "Unicode 🚀 emojis",
        "Right-to-left: مرحبا",
        "Mixed: Hello世界Привет",
    ];

    for reason in reasons {
        let original = HealthStatus::Degraded {
            reason: reason.to_string(),
        };

        // Test Borsh round-trip
        let borsh_bytes = borsh::to_vec(&original).expect("Borsh serialization should succeed");
        let borsh_decoded = HealthStatus::try_from_slice(&borsh_bytes)
            .expect("Borsh deserialization should succeed");
        assert_eq!(borsh_decoded, original);

        // Test JSON round-trip
        let json = serde_json::to_string(&original).expect("JSON serialization should succeed");
        let json_decoded: HealthStatus =
            serde_json::from_str(&json).expect("JSON deserialization should succeed");
        assert_eq!(json_decoded, original);

        // Test CBOR round-trip
        let cbor_bytes = serde_cbor::to_vec(&original).expect("CBOR serialization should succeed");
        let cbor_decoded: HealthStatus =
            serde_cbor::from_slice(&cbor_bytes).expect("CBOR deserialization should succeed");
        assert_eq!(cbor_decoded, original);
    }
}

#[test]
fn test_health_status_all_variants() {
    let variants = vec![
        HealthStatus::Healthy,
        HealthStatus::Degraded {
            reason: "Test degraded".to_string(),
        },
        HealthStatus::Unhealthy {
            reason: "Test unhealthy".to_string(),
        },
    ];

    for variant in variants {
        // Test all serialization formats
        let borsh_bytes = borsh::to_vec(&variant).expect("Borsh serialization should succeed");
        let borsh_decoded = HealthStatus::try_from_slice(&borsh_bytes)
            .expect("Borsh deserialization should succeed");
        assert_eq!(borsh_decoded, variant);

        let json_bytes = serde_json::to_vec(&variant).expect("JSON serialization should succeed");
        let json_decoded: HealthStatus =
            serde_json::from_slice(&json_bytes).expect("JSON deserialization should succeed");
        assert_eq!(json_decoded, variant);

        let cbor_bytes = serde_cbor::to_vec(&variant).expect("CBOR serialization should succeed");
        let cbor_decoded: HealthStatus =
            serde_cbor::from_slice(&cbor_bytes).expect("CBOR deserialization should succeed");
        assert_eq!(cbor_decoded, variant);
    }
}
