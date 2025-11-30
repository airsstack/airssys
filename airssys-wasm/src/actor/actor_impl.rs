//! Actor trait implementation for ComponentActor (message handling).
//!
//! This module implements the `Actor` trait from airssys-rt, enabling ComponentActor
//! to handle inter-component messages. The Actor trait defines message processing logic:
//! - `handle_message()`: Process incoming ComponentMessage variants
//! - `pre_start()`: Initialize before message processing (optional)
//! - `post_stop()`: Cleanup after actor stops (optional)
//!
//! # Design Rationale (ADR-WASM-006)
//!
//! Actor trait handles message passing and business logic, while Child trait handles
//! lifecycle management. This separation enables clean integration with both ActorSystem
//! (for messaging) and SupervisorNode (for supervision).
//!
//! # Implementation Status
//!
//! **STUB IMPLEMENTATION - Task 1.3**
//!
//! This is a stub implementation to unblock Task 1.1 (structure and traits).
//! Full message handling logic will be implemented in Task 1.3 (Actor Trait Message Handling).
//!
//! Current behavior:
//! - `handle_message()`: Stub implementation (logs message, returns Ok)
//! - `pre_start()`: Verify WASM loaded, stub registry registration
//! - `post_stop()`: Stub registry deregistration
//!
//! # References
//!
//! - **WASM-TASK-004 Phase 1 Task 1.3**: Actor Trait Message Handling (16-20 hours)
//! - **KNOWLEDGE-WASM-016**: Actor System Integration Implementation Guide
//! - **ADR-RT-004**: Actor and Child Trait Separation

// Layer 1: Standard library imports
use std::error::Error;
use std::fmt;

// Layer 2: Third-party crate imports
use async_trait::async_trait;

// Layer 3: Internal module imports
use super::component_actor::{ActorState, ComponentActor, ComponentMessage};
use airssys_rt::actor::{Actor, ActorContext};
use airssys_rt::broker::MessageBroker;
use airssys_rt::message::Message;

/// Error type for ComponentActor operations.
///
/// This is a minimal error type for Task 1.1. Future implementation will use
/// WasmError from core module.
#[derive(Debug)]
pub struct ComponentActorError {
    message: String,
}

#[allow(dead_code)] // Will be used in Task 1.3 full implementation
impl ComponentActorError {
    fn new(msg: impl Into<String>) -> Self {
        Self {
            message: msg.into(),
        }
    }
}

impl fmt::Display for ComponentActorError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "ComponentActor error: {}", self.message)
    }
}

impl Error for ComponentActorError {}

/// Message trait implementation for ComponentMessage.
///
/// Required by Actor trait to enable message passing.
impl Message for ComponentMessage {
    const MESSAGE_TYPE: &'static str = "component_message";
}

/// Actor trait implementation for ComponentActor (STUB).
///
/// This is a stub implementation to unblock Task 1.1. Full message handling
/// logic will be implemented in Task 1.3.
///
/// # Stub Behavior
///
/// - **handle_message()**: Stub implementation (logs and returns Ok)
/// - **pre_start()**: Verifies WASM loaded (stub registry registration)
/// - **post_stop()**: Stub registry deregistration
///
/// # Future Implementation (Task 1.3)
///
/// - **handle_message()**: Full message dispatch to WASM (Invoke, InterComponent, etc.)
/// - **pre_start()**: Register with component registry
/// - **post_stop()**: Deregister from component registry
#[async_trait]
impl Actor for ComponentActor {
    type Message = ComponentMessage;
    type Error = ComponentActorError;

    /// Handle incoming component messages (STUB).
    ///
    /// **STUB IMPLEMENTATION**: Currently logs message and returns Ok.
    /// Full implementation in Task 1.3 will:
    /// 1. Match on ComponentMessage variants
    /// 2. Invoke: Deserialize multicodec args, call WASM function, encode result
    /// 3. InterComponent: Validate capabilities, route to WASM handle-message export
    /// 4. HealthCheck: Call _health export, return HealthStatus
    /// 5. Shutdown: Signal ActorSystem to stop this actor
    ///
    /// # Example
    ///
    /// ```rust,ignore
    /// use airssys_wasm::actor::{ComponentActor, ComponentMessage};
    /// use airssys_rt::actor::Actor;
    ///
    /// let msg = ComponentMessage::HealthCheck;
    /// let result = actor.handle_message(msg, &mut ctx).await;
    /// ```
    async fn handle_message<B: MessageBroker<Self::Message>>(
        &mut self,
        msg: Self::Message,
        _ctx: &mut ActorContext<Self::Message, B>,
    ) -> Result<(), Self::Error> {
        // TODO(Task 1.3): Implement full message handling
        //
        // Full implementation will match on message variants:
        // - Invoke: decode_multicodec, call_wasm_function, encode_result
        // - InterComponent: check_capabilities, route_to_wasm
        // - HealthCheck: call_health_export, reply with HealthStatus
        // - Shutdown: ctx.stop()

        // Stub: Log and return Ok
        match msg {
            ComponentMessage::Invoke { function, .. } => {
                // Stub: Would call WASM function here
                println!("STUB: Invoke {} on component {}", function, self.component_id().as_str());
            }
            ComponentMessage::InterComponent { sender, .. } => {
                // Stub: Would route to WASM handle-message export
                println!("STUB: InterComponent from {} to {}", sender.as_str(), self.component_id().as_str());
            }
            ComponentMessage::HealthCheck => {
                // Stub: Would call _health export
                println!("STUB: HealthCheck for component {}", self.component_id().as_str());
            }
            ComponentMessage::Shutdown => {
                // Stub: Would signal ActorSystem to stop
                println!("STUB: Shutdown component {}", self.component_id().as_str());
                self.set_state(ActorState::Stopping);
            }
            _ => {
                println!("STUB: Unhandled message for component {}", self.component_id().as_str());
            }
        }

        Ok(())
    }

    /// Initialize actor before message processing (STUB).
    ///
    /// **STUB IMPLEMENTATION**: Verifies WASM loaded, logs initialization.
    /// Full implementation in Task 1.3 will:
    /// 1. Verify WASM runtime is loaded (from Child::start())
    /// 2. Register with component registry
    /// 3. Start mailbox receiver loop
    /// 4. Transition state to Ready
    ///
    /// # Errors
    ///
    /// Returns error if WASM runtime not loaded.
    async fn pre_start<B: MessageBroker<Self::Message>>(
        &mut self,
        _ctx: &mut ActorContext<Self::Message, B>,
    ) -> Result<(), Self::Error> {
        // TODO(Task 1.3): Register with component registry
        //
        // Full implementation will:
        // 1. ctx.registry.register(self.component_id.clone(), self.clone())
        // 2. self.mailbox_rx = Some(ctx.mailbox.clone())
        // 3. Transition state to Ready

        // Stub: Verify WASM loaded (though stub Child::start doesn't load it)
        if !self.is_wasm_loaded() && *self.state() == ActorState::Ready {
            // Allow in stub mode where WASM isn't actually loaded
            println!("STUB: pre_start for component {} (WASM not loaded in stub)", self.component_id().as_str());
        }

        Ok(())
    }

    /// Cleanup when actor stops (STUB).
    ///
    /// **STUB IMPLEMENTATION**: Logs cleanup.
    /// Full implementation in Task 1.3 will:
    /// 1. Deregister from component registry
    /// 2. Verify WASM cleanup completed (should be done by Child::stop())
    /// 3. Log final state
    async fn post_stop<B: MessageBroker<Self::Message>>(
        &mut self,
        _ctx: &mut ActorContext<Self::Message, B>,
    ) -> Result<(), Self::Error> {
        // TODO(Task 1.3): Deregister from component registry
        //
        // Full implementation will:
        // 1. ctx.registry.unregister(&self.component_id)
        // 2. Verify WASM runtime cleanup

        // Stub: Log cleanup
        println!("STUB: post_stop for component {}", self.component_id().as_str());
        self.set_state(ActorState::Terminated);

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::{ComponentId, ComponentMetadata, CapabilitySet, ResourceLimits};
    use airssys_rt::supervisor::Child;

    fn create_test_metadata() -> ComponentMetadata {
        ComponentMetadata {
            name: "test-component".to_string(),
            version: "1.0.0".to_string(),
            author: "Test Author".to_string(),
            description: None,
            required_capabilities: vec![],
            resource_limits: ResourceLimits {
                max_memory_bytes: 64 * 1024 * 1024,
                max_fuel: 1_000_000,
                max_execution_ms: 5000,
                max_storage_bytes: 10 * 1024 * 1024,
            },
        }
    }

    fn create_test_actor() -> ComponentActor {
        ComponentActor::new(
            ComponentId::new("test-component"),
            create_test_metadata(),
            CapabilitySet::new(),
        )
    }

    #[test]
    fn test_actor_trait_compiles() {
        // Verify ComponentActor implements Actor trait
        fn assert_actor<T: Actor>() {}
        assert_actor::<ComponentActor>();
    }

    #[test]
    fn test_message_trait_implemented() {
        // Verify ComponentMessage implements Message trait
        assert_eq!(ComponentMessage::MESSAGE_TYPE, "component_message");
    }

    #[tokio::test]
    async fn test_actor_pre_start() {
        let mut actor = create_test_actor();

        // Start component first (Child trait)
        let result = actor.start().await;
        assert!(result.is_ok(), "Failed to start actor: {result:?}");

        // pre_start should succeed
        // Note: Can't easily test without full ActorContext, so this is minimal
        assert_eq!(*actor.state(), ActorState::Ready);
    }

    #[tokio::test]
    async fn test_actor_post_stop() {
        let mut actor = create_test_actor();
        let result = actor.start().await;
        assert!(result.is_ok(), "Failed to start actor: {result:?}");

        // post_stop should transition to Terminated
        // Note: Can't easily test without full ActorContext, so this tests state only
        assert_eq!(*actor.state(), ActorState::Ready);
    }

    #[test]
    fn test_component_actor_error_display() {
        let error = ComponentActorError::new("test error");
        let display = format!("{error}");
        assert!(display.contains("test error"));
    }

    #[test]
    fn test_component_actor_error_debug() {
        let error = ComponentActorError::new("debug test");
        let debug = format!("{error:?}");
        assert!(debug.contains("debug test"));
    }
}
