//! HealthStatus serialization tests (Task 1.4).
//!
//! Comprehensive round-trip serialization tests for HealthStatus enum across
//! all supported formats: Borsh (binary), JSON (text), and CBOR (binary).
//!
//! # Test Coverage
//!
//! - Borsh serialization (all variants)
//! - JSON serialization (all variants)
//! - CBOR serialization (all variants)
//! - Round-trip equality verification
//! - Serialized size validation
//!
//! # References
//!
//! - WASM-TASK-004 Phase 1 Task 1.4: Health Check System
//! - KNOWLEDGE-WASM-017: Health Check System Architecture

#![allow(clippy::expect_used, reason = "expect is acceptable in test code")]
#![allow(clippy::panic, reason = "panic in test assertions is acceptable")]

use airssys_wasm::actor::HealthStatus;
use borsh::BorshDeserialize;

// ======================================================================
// BORSH SERIALIZATION TESTS
// ======================================================================

#[test]
fn test_health_status_healthy_borsh_round_trip() {
    let original = HealthStatus::Healthy;
    let serialized = borsh::to_vec(&original).expect("Failed to serialize");
    let deserialized = HealthStatus::try_from_slice(&serialized).expect("Failed to deserialize");
    assert_eq!(original, deserialized);

    // Healthy should be 1 byte (variant tag)
    assert_eq!(serialized.len(), 1, "Healthy should serialize to 1 byte");
    assert_eq!(serialized[0], 0u8, "Healthy variant tag should be 0");
}

#[test]
fn test_health_status_degraded_borsh_round_trip() {
    let original = HealthStatus::Degraded {
        reason: "High latency".to_string(),
    };
    let serialized = borsh::to_vec(&original).expect("Failed to serialize");
    let deserialized = HealthStatus::try_from_slice(&serialized).expect("Failed to deserialize");
    assert_eq!(original, deserialized);

    // Degraded: 1 byte (variant) + 4 bytes (string len) + string bytes
    let expected_len = 1 + 4 + "High latency".len();
    assert_eq!(
        serialized.len(),
        expected_len,
        "Degraded should serialize to {} bytes",
        expected_len
    );
    assert_eq!(serialized[0], 1u8, "Degraded variant tag should be 1");
}

#[test]
fn test_health_status_degraded_empty_reason_borsh() {
    let original = HealthStatus::Degraded {
        reason: String::new(),
    };
    let serialized = borsh::to_vec(&original).expect("Failed to serialize");
    let deserialized = HealthStatus::try_from_slice(&serialized).expect("Failed to deserialize");
    assert_eq!(original, deserialized);

    // Empty string: 1 byte (variant) + 4 bytes (string len = 0)
    assert_eq!(serialized.len(), 5);
}

#[test]
fn test_health_status_unhealthy_borsh_round_trip() {
    let original = HealthStatus::Unhealthy {
        reason: "Database unreachable".to_string(),
    };
    let serialized = borsh::to_vec(&original).expect("Failed to serialize");
    let deserialized = HealthStatus::try_from_slice(&serialized).expect("Failed to deserialize");
    assert_eq!(original, deserialized);

    // Unhealthy: 1 byte (variant) + 4 bytes (string len) + string bytes
    let expected_len = 1 + 4 + "Database unreachable".len();
    assert_eq!(
        serialized.len(),
        expected_len,
        "Unhealthy should serialize to {} bytes",
        expected_len
    );
    assert_eq!(serialized[0], 2u8, "Unhealthy variant tag should be 2");
}

#[test]
fn test_health_status_borsh_invalid_variant() {
    // Invalid variant tag (3) should fail deserialization
    let invalid_bytes = vec![3u8, 0, 0, 0, 0]; // variant=3, empty string
    let result = HealthStatus::try_from_slice(&invalid_bytes);

    assert!(
        result.is_err(),
        "Invalid variant should fail to deserialize"
    );

    if let Err(e) = result {
        assert!(
            e.to_string().contains("Invalid HealthStatus variant")
                || e.to_string().contains("invalid"),
            "Error should mention invalid variant: {}",
            e
        );
    }
}

// ======================================================================
// JSON SERIALIZATION TESTS
// ======================================================================

#[test]
fn test_health_status_healthy_json_round_trip() {
    let original = HealthStatus::Healthy;
    let serialized = serde_json::to_string(&original).expect("Failed to serialize");
    let deserialized: HealthStatus =
        serde_json::from_str(&serialized).expect("Failed to deserialize");
    assert_eq!(original, deserialized);

    // Verify JSON structure
    assert!(
        serialized.contains("\"status\":\"healthy\""),
        "JSON should contain status field: {}",
        serialized
    );
}

#[test]
fn test_health_status_degraded_json_round_trip() {
    let original = HealthStatus::Degraded {
        reason: "High latency".to_string(),
    };
    let serialized = serde_json::to_string(&original).expect("Failed to serialize");
    let deserialized: HealthStatus =
        serde_json::from_str(&serialized).expect("Failed to deserialize");
    assert_eq!(original, deserialized);

    // Verify JSON structure
    assert!(
        serialized.contains("\"status\":\"degraded\""),
        "JSON should contain degraded status: {}",
        serialized
    );
    assert!(
        serialized.contains("\"reason\":\"High latency\""),
        "JSON should contain reason: {}",
        serialized
    );
}

#[test]
fn test_health_status_unhealthy_json_round_trip() {
    let original = HealthStatus::Unhealthy {
        reason: "Connection lost".to_string(),
    };
    let serialized = serde_json::to_string(&original).expect("Failed to serialize");
    let deserialized: HealthStatus =
        serde_json::from_str(&serialized).expect("Failed to deserialize");
    assert_eq!(original, deserialized);

    // Verify JSON structure
    assert!(
        serialized.contains("\"status\":\"unhealthy\""),
        "JSON should contain unhealthy status: {}",
        serialized
    );
    assert!(
        serialized.contains("\"reason\":\"Connection lost\""),
        "JSON should contain reason: {}",
        serialized
    );
}

#[test]
fn test_health_status_json_pretty_print() {
    let status = HealthStatus::Degraded {
        reason: "High memory usage".to_string(),
    };
    let pretty = serde_json::to_string_pretty(&status).expect("Failed to pretty print");

    // Verify pretty printing works
    assert!(pretty.contains('\n'), "Pretty print should have newlines");
    assert!(
        pretty.contains("\"status\""),
        "Pretty print should contain status"
    );
}

// ======================================================================
// CBOR SERIALIZATION TESTS
// ======================================================================

#[test]
fn test_health_status_healthy_cbor_round_trip() {
    let original = HealthStatus::Healthy;
    let serialized = serde_cbor::to_vec(&original).expect("Failed to serialize");
    let deserialized: HealthStatus =
        serde_cbor::from_slice(&serialized).expect("Failed to deserialize");
    assert_eq!(original, deserialized);

    // CBOR should be compact (typically smaller than JSON)
    assert!(
        serialized.len() < 20,
        "CBOR Healthy should be compact (<20 bytes), got {}",
        serialized.len()
    );
}

#[test]
fn test_health_status_degraded_cbor_round_trip() {
    let original = HealthStatus::Degraded {
        reason: "High latency".to_string(),
    };
    let serialized = serde_cbor::to_vec(&original).expect("Failed to serialize");
    let deserialized: HealthStatus =
        serde_cbor::from_slice(&serialized).expect("Failed to deserialize");
    assert_eq!(original, deserialized);
}

#[test]
fn test_health_status_unhealthy_cbor_round_trip() {
    let original = HealthStatus::Unhealthy {
        reason: "Database connection failed".to_string(),
    };
    let serialized = serde_cbor::to_vec(&original).expect("Failed to serialize");
    let deserialized: HealthStatus =
        serde_cbor::from_slice(&serialized).expect("Failed to deserialize");
    assert_eq!(original, deserialized);
}

#[test]
fn test_health_status_cbor_size_comparison() {
    let status = HealthStatus::Degraded {
        reason: "Test reason".to_string(),
    };

    let cbor_size = serde_cbor::to_vec(&status)
        .expect("CBOR serialization failed")
        .len();
    let json_size = serde_json::to_string(&status)
        .expect("JSON serialization failed")
        .len();

    // CBOR should typically be smaller than JSON for structured data
    assert!(
        cbor_size <= json_size,
        "CBOR ({} bytes) should be <= JSON ({} bytes)",
        cbor_size,
        json_size
    );
}

// ======================================================================
// CROSS-FORMAT COMPATIBILITY TESTS
// ======================================================================

#[test]
fn test_all_variants_all_formats() {
    let test_cases = vec![
        HealthStatus::Healthy,
        HealthStatus::Degraded {
            reason: "Test degradation".to_string(),
        },
        HealthStatus::Unhealthy {
            reason: "Test failure".to_string(),
        },
    ];

    for original in test_cases {
        // Borsh round-trip
        let borsh_bytes = borsh::to_vec(&original).expect("Borsh serialization failed");
        let borsh_deserialized =
            HealthStatus::try_from_slice(&borsh_bytes).expect("Borsh deserialization failed");
        assert_eq!(
            original, borsh_deserialized,
            "Borsh round-trip failed for {:?}",
            original
        );

        // JSON round-trip
        let json_str = serde_json::to_string(&original).expect("JSON serialization failed");
        let json_deserialized: HealthStatus =
            serde_json::from_str(&json_str).expect("JSON deserialization failed");
        assert_eq!(
            original, json_deserialized,
            "JSON round-trip failed for {:?}",
            original
        );

        // CBOR round-trip
        let cbor_bytes = serde_cbor::to_vec(&original).expect("CBOR serialization failed");
        let cbor_deserialized: HealthStatus =
            serde_cbor::from_slice(&cbor_bytes).expect("CBOR deserialization failed");
        assert_eq!(
            original, cbor_deserialized,
            "CBOR round-trip failed for {:?}",
            original
        );
    }
}

#[test]
fn test_health_status_with_unicode_reason() {
    let status = HealthStatus::Degraded {
        reason: "数据库延迟 (Database latency) 🔥".to_string(),
    };

    // Test all formats with Unicode
    let borsh_result = borsh::to_vec(&status);
    assert!(
        borsh_result.is_ok(),
        "Borsh should handle Unicode: {:?}",
        borsh_result
    );

    let json_result = serde_json::to_string(&status);
    assert!(
        json_result.is_ok(),
        "JSON should handle Unicode: {:?}",
        json_result
    );

    let cbor_result = serde_cbor::to_vec(&status);
    assert!(
        cbor_result.is_ok(),
        "CBOR should handle Unicode: {:?}",
        cbor_result
    );
}

#[test]
fn test_health_status_with_long_reason() {
    // Test with a very long reason string (1KB)
    let long_reason = "A".repeat(1024);
    let status = HealthStatus::Unhealthy {
        reason: long_reason.clone(),
    };

    // Borsh round-trip
    let borsh_bytes = borsh::to_vec(&status).expect("Borsh serialization failed");
    let borsh_deserialized =
        HealthStatus::try_from_slice(&borsh_bytes).expect("Borsh deserialization failed");

    if let HealthStatus::Unhealthy { reason } = borsh_deserialized {
        assert_eq!(reason.len(), 1024, "Long reason should be preserved");
        assert_eq!(reason, long_reason);
    } else {
        panic!("Expected Unhealthy variant");
    }
}
