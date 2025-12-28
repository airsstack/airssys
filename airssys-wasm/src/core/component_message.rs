//! Component message types for actor communication.
//!
//! This module defines the fundamental message types used for inter-component
//! communication within the airssys-wasm framework. These types are placed in
//! `core/` to prevent circular dependencies between `actor/` and `runtime/`.
//!
//! # Architecture (ADR-WASM-022)
//!
//! ComponentMessage is a data type that needs to be accessible from multiple
//! layers (actor/, runtime/). Following the principle that data types belong
//! in core/, this module was moved from actor/ to core/.
//!
//! # Message Types
//!
//! - **Invoke**: Call a WASM function with arguments
//! - **InvokeResult**: Result of a function invocation (request-response pattern)
//! - **InterComponent**: Message from another component (direct addressing)
//! - **InterComponentWithCorrelation**: Message with correlation ID (request-response)
//! - **Shutdown**: Signal to stop the actor
//! - **HealthCheck**: Request health status
//! - **HealthStatus**: Health status response
//!
//! # Multicodec Encoding
//!
//! Invoke and InterComponent messages use multicodec-prefixed payloads
//! (ADR-WASM-001) supporting Borsh, CBOR, and JSON codecs.
//!
//! # References
//!
//! - **ADR-WASM-022**: Circular Dependency Remediation
//! - **ADR-WASM-001**: Multicodec Compatibility Strategy
//! - **KNOWLEDGE-WASM-028**: Circular Dependency Between actor/ and runtime/

// Layer 1: Standard library imports
use std::io::{Read, Write};

// Layer 2: External crate imports
use borsh::{BorshDeserialize, BorshSerialize};
use serde::{Deserialize, Serialize};

// Layer 3: Internal module imports
use crate::core::ComponentId;

/// Component message types for actor communication.
///
/// ComponentMessage defines all message types that ComponentActor can handle.
/// Messages are processed sequentially by the actor's message handler.
///
/// # Message Types
///
/// - **Invoke**: Call a WASM function with arguments
/// - **InvokeResult**: Result of a function invocation (request-response pattern)
/// - **InterComponent**: Message from another component
/// - **Shutdown**: Signal to stop the actor
/// - **HealthCheck**: Request health status
/// - **HealthStatus**: Health status response
///
/// # Multicodec Encoding
///
/// Invoke and InterComponent messages use multicodec-prefixed payloads
/// (ADR-WASM-001) supporting Borsh, CBOR, and JSON codecs.
///
/// # Example
///
/// ```rust
/// use airssys_wasm::core::{ComponentMessage, ComponentId};
///
/// // Invoke WASM function
/// let msg = ComponentMessage::Invoke {
///     function: "process_data".to_string(),
///     args: vec![1, 2, 3, 4], // Multicodec-encoded
/// };
///
/// // Inter-component message
/// let sender = ComponentId::new("sender-component");
/// let to = ComponentId::new("target-component");
/// let msg = ComponentMessage::InterComponent {
///     sender,
///     to,
///     payload: vec![0x70, 0x01, 0x00, 0x01], // Multicodec Borsh
/// };
///
/// // Health check
/// let msg = ComponentMessage::HealthCheck;
/// ```
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ComponentMessage {
    /// Invoke WASM function with arguments.
    ///
    /// Arguments are multicodec-encoded (ADR-WASM-001).
    Invoke {
        /// Function name to invoke
        function: String,
        /// Multicodec-encoded arguments
        args: Vec<u8>,
    },

    /// Result of invoke (for request-response pattern).
    InvokeResult {
        /// Multicodec-encoded result
        result: Vec<u8>,
        /// Error message if invocation failed
        error: Option<String>,
    },

    /// Message from another component.
    ///
    /// Phase 1 uses direct ComponentId addressing (KNOWLEDGE-WASM-024).
    /// The `to` field specifies the target component for routing.
    InterComponent {
        /// Sender component ID
        sender: ComponentId,
        /// Target component ID (direct addressing)
        to: ComponentId,
        /// Multicodec-encoded payload
        payload: Vec<u8>,
    },

    /// Message from another component with correlation ID (request-response).
    ///
    /// Used for request-response patterns where the sender expects a correlated
    /// response message. The correlation_id allows matching responses to requests.
    ///
    /// Phase 1 uses direct ComponentId addressing (KNOWLEDGE-WASM-024).
    /// The `to` field specifies the target component for routing.
    InterComponentWithCorrelation {
        /// Sender component ID
        sender: ComponentId,
        /// Target component ID (direct addressing)
        to: ComponentId,
        /// Multicodec-encoded payload
        payload: Vec<u8>,
        /// Correlation ID for request-response pattern
        correlation_id: uuid::Uuid,
    },

    /// Signal to shutdown the actor.
    Shutdown,

    /// Request health status.
    HealthCheck,

    /// Health status response.
    HealthStatus(ComponentHealthStatus),
}

/// Component health status for monitoring and supervision.
///
/// HealthStatus represents the operational state of a component, used by both
/// internal health checks and external monitoring systems. This enum supports
/// serialization via Borsh (binary), CBOR (binary), and JSON (text).
///
/// # Serialization Formats
///
/// **Borsh (Recommended):**
/// ```text
/// Healthy:    [0x00]
/// Degraded:   [0x01, len_u32, reason_bytes...]
/// Unhealthy:  [0x02, len_u32, reason_bytes...]
/// ```
///
/// **JSON:**
/// ```json
/// { "status": "healthy" }
/// { "status": "degraded", "reason": "High latency" }
/// { "status": "unhealthy", "reason": "Database unreachable" }
/// ```
///
/// **CBOR:** Binary equivalent of JSON structure
///
/// # Example
///
/// ```rust
/// use airssys_wasm::core::ComponentHealthStatus;
/// use serde_json;
///
/// let status = ComponentHealthStatus::Degraded {
///     reason: "High memory usage".to_string(),
/// };
///
/// // JSON serialization
/// let json = serde_json::to_string(&status).unwrap();
/// assert!(json.contains("degraded"));
///
/// // Deserialization
/// let parsed: HealthStatus = serde_json::from_str(&json).unwrap();
/// assert!(matches!(parsed, ComponentHealthStatus::Degraded { .. }));
/// ```
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(tag = "status", content = "reason", rename_all = "lowercase")]
pub enum ComponentHealthStatus {
    /// Component operating normally
    #[serde(rename = "healthy")]
    Healthy,

    /// Component operational but experiencing issues
    #[serde(rename = "degraded")]
    Degraded {
        /// Reason for degraded status
        reason: String,
    },

    /// Component failed or non-functional
    #[serde(rename = "unhealthy")]
    Unhealthy {
        /// Reason for unhealthy status
        reason: String,
    },
}

// Borsh serialization for compact binary format
impl BorshSerialize for ComponentHealthStatus {
    fn serialize<W: Write>(&self, writer: &mut W) -> std::io::Result<()> {
        match self {
            ComponentHealthStatus::Healthy => BorshSerialize::serialize(&0u8, writer),
            ComponentHealthStatus::Degraded { reason } => {
                BorshSerialize::serialize(&1u8, writer)?;
                BorshSerialize::serialize(reason, writer)
            }
            ComponentHealthStatus::Unhealthy { reason } => {
                BorshSerialize::serialize(&2u8, writer)?;
                BorshSerialize::serialize(reason, writer)
            }
        }
    }
}

impl BorshDeserialize for ComponentHealthStatus {
    fn deserialize(buf: &mut &[u8]) -> std::io::Result<Self> {
        let variant: u8 = BorshDeserialize::deserialize(buf)?;
        match variant {
            0u8 => Ok(ComponentHealthStatus::Healthy),
            1u8 => Ok(ComponentHealthStatus::Degraded {
                reason: BorshDeserialize::deserialize(buf)?,
            }),
            2u8 => Ok(ComponentHealthStatus::Unhealthy {
                reason: BorshDeserialize::deserialize(buf)?,
            }),
            _ => Err(std::io::Error::new(
                std::io::ErrorKind::InvalidData,
                format!("Invalid HealthStatus variant: {}", variant),
            )),
        }
    }

    fn deserialize_reader<R: Read>(reader: &mut R) -> std::io::Result<Self> {
        let mut buf = Vec::new();
        reader.read_to_end(&mut buf)?;
        let mut slice = buf.as_slice();
        <Self as BorshDeserialize>::deserialize(&mut slice)
    }
}

#[allow(clippy::expect_used, clippy::unwrap_used, clippy::unwrap_err_used, clippy::expect_err_used, clippy::panic, clippy::unwrap_on_result, clippy::indexing_slicing, clippy::too_many_arguments, clippy::type_complexity, reason = "test code")]
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_component_message_invoke() {
        let msg = ComponentMessage::Invoke {
            function: "test".to_string(),
            args: vec![1, 2, 3],
        };
        if let ComponentMessage::Invoke { function, args } = msg {
            assert_eq!(function, "test");
            assert_eq!(args, vec![1, 2, 3]);
        } else {
            panic!("Expected Invoke variant");
        }
    }

    #[test]
    fn test_component_message_inter_component() {
        let sender = ComponentId::new("sender");
        let to = ComponentId::new("target");
        let msg = ComponentMessage::InterComponent {
            sender: sender.clone(),
            to: to.clone(),
            payload: vec![0x70, 0x01],
        };
        if let ComponentMessage::InterComponent {
            sender: s,
            to: t,
            payload,
        } = msg
        {
            assert_eq!(s.as_str(), "sender");
            assert_eq!(t.as_str(), "target");
            assert_eq!(payload, vec![0x70, 0x01]);
        } else {
            panic!("Expected InterComponent variant");
        }
    }

    #[test]
    fn test_component_message_shutdown() {
        let msg = ComponentMessage::Shutdown;
        assert!(matches!(msg, ComponentMessage::Shutdown));
    }

    #[test]
    fn test_component_message_health_check() {
        let msg = ComponentMessage::HealthCheck;
        assert!(matches!(msg, ComponentMessage::HealthCheck));
    }

    #[test]
    fn test_health_status_healthy() {
        let status = ComponentHealthStatus::Healthy;
        let json = serde_json::to_string(&status).unwrap();
        assert!(json.contains("healthy"));
    }

    #[test]
    fn test_health_status_degraded() {
        let status = ComponentHealthStatus::Degraded {
            reason: "High latency".to_string(),
        };
        let json = serde_json::to_string(&status).unwrap();
        assert!(json.contains("degraded"));
        assert!(json.contains("High latency"));
    }

    #[test]
    fn test_health_status_unhealthy() {
        let status = ComponentHealthStatus::Unhealthy {
            reason: "Database down".to_string(),
        };
        let json = serde_json::to_string(&status).unwrap();
        assert!(json.contains("unhealthy"));
    }

    #[test]
    fn test_health_status_borsh_roundtrip() {
        use borsh::{BorshDeserialize, BorshSerialize};

        let statuses = vec![
            ComponentHealthStatus::Healthy,
            ComponentHealthStatus::Degraded {
                reason: "test".to_string(),
            },
            ComponentHealthStatus::Unhealthy {
                reason: "error".to_string(),
            },
        ];

        for status in statuses {
            let mut buf = Vec::new();
            BorshSerialize::serialize(&status, &mut buf).unwrap();
            let mut slice = buf.as_slice();
            let decoded =
                <ComponentHealthStatus as BorshDeserialize>::deserialize(&mut slice).unwrap();
            assert_eq!(status, decoded);
        }
    }
}
