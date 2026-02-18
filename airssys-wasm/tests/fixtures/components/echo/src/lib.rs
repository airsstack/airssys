// Echo Component - WASM Guest
//
// Implements the airssys:core/component-lifecycle interface.
// handle-message returns the input payload unchanged (echo behavior).
//
// References:
// - KNOWLEDGE-WASM-043: Guest-side uses wit_bindgen::generate!
// - ADR-WASM-032: echo.wasm fixture specification
// - airssys-wasm/wit/core/component-lifecycle.wit: Interface contract

wit_bindgen::generate!({
    world: "runtime-host",
    path: "wit/core",
});

use airssys::core::errors::{ComponentError, WasmError};
use airssys::core::types::{ComponentConfig, ComponentMessage, HealthStatus, MessagePayload};
use exports::airssys::core::component_lifecycle::{ComponentMetadata, Guest};

struct EchoComponent;

impl Guest for EchoComponent {
    fn initialize(_config: ComponentConfig) -> Result<(), ComponentError> {
        // No-op: echo component requires no initialization
        Ok(())
    }

    fn handle_message(
        msg: ComponentMessage,
    ) -> Result<Option<MessagePayload>, WasmError> {
        // Echo behavior: return the message payload unchanged
        Ok(Some(msg.payload))
    }

    fn handle_callback(_msg: ComponentMessage) -> Result<(), WasmError> {
        // No-op: echo component does not use request-response
        Ok(())
    }

    fn metadata() -> ComponentMetadata {
        ComponentMetadata {
            name: "echo".to_string(),
            version: "0.1.0".to_string(),
            description: "Echo component - returns message payload unchanged".to_string(),
            author: "AirsStack Team".to_string(),
            license: "MIT OR Apache-2.0".to_string(),
            supported_operations: vec!["echo".to_string()],
            stateful: false,
        }
    }

    fn health() -> HealthStatus {
        HealthStatus::Healthy
    }

    fn shutdown() -> Result<(), ComponentError> {
        // No-op: echo component has no resources to clean up
        Ok(())
    }
}

export!(EchoComponent);
