// Callback Component - WASM Guest
//
// Implements the airssys:core/component-lifecycle interface.
// Uses host-messaging.request() to send requests to other components,
// exercising the request-response pattern and host function bindings.
//
// Message payload format: "namespace|name|instance|data"
// - Parses the target component ID from the incoming message
// - Calls host-messaging.request() with the data portion as payload
// - Returns the correlation-id as UTF-8 bytes on success
//
// References:
// - KNOWLEDGE-WASM-043: Guest-side uses wit_bindgen::generate!
// - ADR-WASM-032: callback.wasm fixture specification
// - ADR-WASM-009: Request-response pattern specification
// - airssys-wasm/wit/core/host-messaging.wit: request() function signature
// - airssys-wasm/wit/core/component-lifecycle.wit: Interface contract

wit_bindgen::generate!({
    world: "runtime-host",
    path: "wit/core",
});

use airssys::core::errors::{ComponentError, MessagingError, WasmError};
use airssys::core::host_messaging;
use airssys::core::types::{ComponentConfig, ComponentId, ComponentMessage, HealthStatus, MessagePayload};
use exports::airssys::core::component_lifecycle::{ComponentMetadata, Guest};

/// Default timeout for request-response calls (milliseconds).
/// Consistent with ADR-WASM-009 Section "Pattern 2" examples.
const REQUEST_TIMEOUT_MS: u64 = 5000;

struct CallbackComponent;

/// Parse a pipe-delimited payload into (ComponentId, forward_payload).
///
/// Expected format: "namespace|name|instance|data"
/// Returns None if the payload does not contain at least 4 pipe-separated segments.
fn parse_request_payload(payload: &[u8]) -> Option<(ComponentId, Vec<u8>)> {
    let text = core::str::from_utf8(payload).ok()?;
    let parts: Vec<&str> = text.splitn(4, '|').collect();
    if parts.len() < 4 {
        return None;
    }

    let target = ComponentId {
        namespace: parts[0].to_string(),
        name: parts[1].to_string(),
        instance: parts[2].to_string(),
    };

    let forward_payload = parts[3].as_bytes().to_vec();
    Some((target, forward_payload))
}

/// Convert a MessagingError to a human-readable description string.
fn messaging_error_description(err: &MessagingError) -> String {
    match err {
        MessagingError::DeliveryFailed(s) => format!("delivery failed: {s}"),
        MessagingError::CorrelationTimeout(id) => format!("correlation timeout: {id}"),
        MessagingError::InvalidMessage(s) => format!("invalid message: {s}"),
        MessagingError::QueueFull => "queue full".to_string(),
        MessagingError::TargetNotFound(id) => {
            format!("target not found: {}/{}/{}", id.namespace, id.name, id.instance)
        }
    }
}

impl Guest for CallbackComponent {
    fn initialize(_config: ComponentConfig) -> Result<(), ComponentError> {
        // No-op: callback component requires no initialization state
        Ok(())
    }

    fn handle_message(
        msg: ComponentMessage,
    ) -> Result<Option<MessagePayload>, WasmError> {
        // Parse the incoming payload to extract target and forward data
        let (target, forward_payload) = match parse_request_payload(&msg.payload) {
            Some(parsed) => parsed,
            None => {
                return Err(WasmError::RuntimeError(
                    "invalid payload format: expected namespace|name|instance|data".to_string(),
                ));
            }
        };

        // Call host-messaging.request() to send a request to the target component
        match host_messaging::request(&target, &forward_payload, REQUEST_TIMEOUT_MS) {
            Ok(correlation_id) => {
                // Return the correlation ID as UTF-8 bytes
                Ok(Some(correlation_id.into_bytes()))
            }
            Err(err) => Err(WasmError::RuntimeError(
                format!("request failed: {}", messaging_error_description(&err)),
            )),
        }
    }

    fn handle_callback(_msg: ComponentMessage) -> Result<(), WasmError> {
        // Callback handling would be used in the full request-response round-trip.
        // For this fixture, handle-callback is a no-op stub.
        // In integration tests (WASM-TASK-053), the host will invoke handle-callback
        // with the response from the target component.
        Ok(())
    }

    fn metadata() -> ComponentMetadata {
        ComponentMetadata {
            name: "callback".to_string(),
            version: "0.1.0".to_string(),
            description: "Callback component - uses host-messaging.request() for request-response pattern".to_string(),
            author: "AirsStack Team".to_string(),
            license: "MIT OR Apache-2.0".to_string(),
            supported_operations: vec!["request".to_string(), "callback".to_string()],
            stateful: false,
        }
    }

    fn health() -> HealthStatus {
        HealthStatus::Healthy
    }

    fn shutdown() -> Result<(), ComponentError> {
        // No-op: callback component has no external resources to clean up
        Ok(())
    }
}

export!(CallbackComponent);
