// Counter Component - WASM Guest
//
// Implements the airssys:core/component-lifecycle interface.
// Maintains an internal counter that increments on each handle-message call.
// Returns the current count as a UTF-8 string payload.
//
// References:
// - KNOWLEDGE-WASM-043: Guest-side uses wit_bindgen::generate!
// - ADR-WASM-032: counter.wasm fixture specification
// - WASM-TASK-050: Echo component pattern reference
// - airssys-wasm/wit/core/component-lifecycle.wit: Interface contract

use std::cell::Cell;

wit_bindgen::generate!({
    world: "runtime-host",
    path: "wit/core",
});

use airssys::core::errors::{ComponentError, WasmError};
use airssys::core::types::{ComponentConfig, ComponentMessage, HealthStatus, MessagePayload};
use exports::airssys::core::component_lifecycle::{ComponentMetadata, Guest};

// WASM is single-threaded; thread_local! + Cell is the idiomatic pattern for mutable state.
thread_local! {
    static COUNTER: Cell<u32> = const { Cell::new(0) };
}

struct CounterComponent;

impl Guest for CounterComponent {
    fn initialize(_config: ComponentConfig) -> Result<(), ComponentError> {
        // Reset counter to 0 on initialization
        COUNTER.with(|c| c.set(0));
        Ok(())
    }

    fn handle_message(
        _msg: ComponentMessage,
    ) -> Result<Option<MessagePayload>, WasmError> {
        // Increment counter and return current count as UTF-8 string
        let new_count = COUNTER.with(|c| {
            let next = c.get() + 1;
            c.set(next);
            next
        });
        Ok(Some(new_count.to_string().into_bytes()))
    }

    fn handle_callback(_msg: ComponentMessage) -> Result<(), WasmError> {
        // No-op: counter component does not use request-response callbacks
        Ok(())
    }

    fn metadata() -> ComponentMetadata {
        ComponentMetadata {
            name: "counter".to_string(),
            version: "0.1.0".to_string(),
            description: "Counter component - increments on each message, returns count".to_string(),
            author: "AirsStack Team".to_string(),
            license: "MIT OR Apache-2.0".to_string(),
            supported_operations: vec!["increment".to_string(), "count".to_string()],
            stateful: true,
        }
    }

    fn health() -> HealthStatus {
        HealthStatus::Healthy
    }

    fn shutdown() -> Result<(), ComponentError> {
        // No-op: counter component has no external resources to clean up
        // Counter state lives in WASM linear memory and is freed with the instance
        Ok(())
    }
}

export!(CounterComponent);
