//! Message routing for inter-component communication.
//!
//! This module provides `MessageRouter`, which handles routing messages to
//! ComponentActor instances via ActorAddress lookup in ComponentRegistry.
//!
//! # Architecture
//!
//! ```text
//! MessageRouter
//!     ↓
//! ComponentRegistry.lookup(component_id) → ActorAddress
//!     ↓
//! MessageBroker.publish_message() → ComponentActor mailbox
//! ```
//!
//! # Performance
//!
//! - **Lookup**: O(1) via ComponentRegistry
//! - **Routing**: <500ns via airssys-rt ActorAddress
//! - **Total**: <500ns end-to-end target
//!
//! # References
//!
//! - **WASM-TASK-004 Phase 2 Task 2.3**: Actor Address and Routing
//! - **ADR-WASM-009**: Component Communication Model
//! - **ADR-WASM-006**: Component Isolation and Sandboxing

// Layer 1: Standard library imports
use std::sync::Arc;

// Layer 2: Third-party crate imports
// (none)

// Layer 3: Internal module imports
use crate::actor::component::ComponentMessage;
use crate::actor::component::ComponentRegistry;
use crate::core::{ComponentId, WasmError};
use airssys_rt::broker::MessageBroker;
use airssys_rt::message::MessageEnvelope;

/// Message router for inter-component communication.
///
/// MessageRouter provides high-level routing API that:
/// - Looks up ActorAddress via ComponentRegistry
/// - Routes messages using airssys-rt MessageBroker
/// - Handles component-not-found errors gracefully
///
/// # Thread Safety
///
/// MessageRouter is Clone-able and can be shared across threads.
/// All operations are thread-safe via ComponentRegistry's Arc<RwLock<>>.
///
/// # Performance
///
/// Target: <500ns routing latency
/// - ComponentRegistry.lookup(): <100ns (HashMap + RwLock)
/// - MessageBroker.publish(): ~211ns (proven RT-TASK-008)
/// - ActorAddress routing: <500ns (airssys-rt baseline)
///
/// # Examples
///
/// ```rust,ignore
/// use airssys_wasm::actor::{MessageRouter, ComponentRegistry};
/// use airssys_wasm::core::ComponentId;
/// use airssys_rt::broker::InMemoryMessageBroker;
///
/// let registry = ComponentRegistry::new();
/// let broker = Arc::new(InMemoryMessageBroker::new());
/// let router = MessageRouter::new(registry, broker);
///
/// // Route message to component
/// let target = ComponentId::new("target-component");
/// let message = ComponentMessage::HealthCheck;
/// router.send_message(&target, message).await?;
/// ```
#[derive(Clone)]
pub struct MessageRouter<B: MessageBroker<ComponentMessage>> {
    registry: ComponentRegistry,
    broker: Arc<B>,
}

impl<B: MessageBroker<ComponentMessage>> MessageRouter<B> {
    /// Create a new MessageRouter.
    ///
    /// # Arguments
    ///
    /// * `registry` - ComponentRegistry for ActorAddress lookup
    /// * `broker` - MessageBroker for message publishing
    ///
    /// # Examples
    ///
    /// ```rust,ignore
    /// let registry = ComponentRegistry::new();
    /// let broker = Arc::new(InMemoryMessageBroker::new());
    /// let router = MessageRouter::new(registry, broker);
    /// ```
    pub fn new(registry: ComponentRegistry, broker: Arc<B>) -> Self {
        Self { registry, broker }
    }

    /// Send message to component by ComponentId.
    ///
    /// # Arguments
    ///
    /// * `target` - Target component ID
    /// * `message` - Message to send
    ///
    /// # Errors
    ///
    /// Returns error if:
    /// - Component not found in registry
    /// - Message routing fails
    ///
    /// # Performance
    ///
    /// Target: <500ns
    /// - Registry lookup: <100ns
    /// - Message publish: ~211ns
    /// - ActorAddress routing: <500ns
    ///
    /// # Examples
    ///
    /// ```rust,ignore
    /// let message = ComponentMessage::HealthCheck;
    /// router.send_message(&target_id, message).await?;
    /// ```
    pub async fn send_message(
        &self,
        target: &ComponentId,
        message: ComponentMessage,
    ) -> Result<(), WasmError> {
        // Lookup ActorAddress in registry (O(1))
        let actor_address = self.registry
            .lookup(target)?;

        // Create message envelope with reply_to address
        let envelope = MessageEnvelope::new(message)
            .with_reply_to(actor_address);

        // Publish message via broker
        self.broker
            .publish(envelope)
            .await
            .map_err(|e| WasmError::messaging_error(
                format!("Failed to route message to {}: {}", target.as_str(), e)
            ))?;

        Ok(())
    }

    /// Broadcast message to multiple components.
    ///
    /// # Arguments
    ///
    /// * `targets` - List of target component IDs
    /// * `message` - Message to broadcast (cloned for each target)
    ///
    /// # Errors
    ///
    /// Returns error on first routing failure. Use `try_broadcast_message` for
    /// best-effort delivery.
    ///
    /// # Examples
    ///
    /// ```rust,ignore
    /// let targets = vec![
    ///     ComponentId::new("component-a"),
    ///     ComponentId::new("component-b"),
    /// ];
    /// router.broadcast_message(&targets, message).await?;
    /// ```
    pub async fn broadcast_message(
        &self,
        targets: &[ComponentId],
        message: ComponentMessage,
    ) -> Result<(), WasmError> {
        for target in targets {
            self.send_message(target, message.clone()).await?;
        }
        Ok(())
    }

    /// Best-effort broadcast (continues on individual failures).
    ///
    /// # Returns
    ///
    /// Vec of (ComponentId, Result) showing which deliveries succeeded/failed.
    ///
    /// # Examples
    ///
    /// ```rust,ignore
    /// let results = router.try_broadcast_message(&targets, message).await;
    /// for (component_id, result) in results {
    ///     if let Err(e) = result {
    ///         log::warn!("Failed to deliver to {}: {}", component_id, e);
    ///     }
    /// }
    /// ```
    pub async fn try_broadcast_message(
        &self,
        targets: &[ComponentId],
        message: ComponentMessage,
    ) -> Vec<(ComponentId, Result<(), WasmError>)> {
        let mut results = Vec::with_capacity(targets.len());
        
        for target in targets {
            let result = self.send_message(target, message.clone()).await;
            results.push((target.clone(), result));
        }
        
        results
    }

    /// Check if component exists in registry.
    ///
    /// # Examples
    ///
    /// ```rust,ignore
    /// if router.component_exists(&target_id) {
    ///     router.send_message(&target_id, message).await?;
    /// }
    /// ```
    pub fn component_exists(&self, component_id: &ComponentId) -> bool {
        self.registry.lookup(component_id).is_ok()
    }

    /// Get current component count in registry.
    ///
    /// # Examples
    ///
    /// ```rust,ignore
    /// println!("Registry has {} components", router.component_count()?);
    /// ```
    pub fn component_count(&self) -> Result<usize, WasmError> {
        self.registry.count()
    }
}

// Manual Debug implementation since MessageBroker doesn't implement Debug
impl<B: MessageBroker<ComponentMessage>> std::fmt::Debug for MessageRouter<B> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("MessageRouter")
            .field("registry", &self.registry)
            .field("broker", &"<MessageBroker>")
            .finish()
    }
}

#[cfg(test)]
#[expect(clippy::unwrap_used, reason = "unwrap is acceptable in test code")]
mod tests {
    use super::*;
    use airssys_rt::broker::InMemoryMessageBroker;
    use airssys_rt::util::ActorAddress;

    #[tokio::test]
    async fn test_send_message_component_not_found() {
        let registry = ComponentRegistry::new();
        let broker = Arc::new(InMemoryMessageBroker::new());
        let router = MessageRouter::new(registry, broker);

        let target = ComponentId::new("nonexistent");
        let message = ComponentMessage::HealthCheck;

        let result = router.send_message(&target, message).await;
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("not found"));
    }

    #[tokio::test]
    async fn test_component_exists() {
        let registry = ComponentRegistry::new();
        let component_id = ComponentId::new("test");
        let actor_addr = ActorAddress::named("test");
        registry.register(component_id.clone(), actor_addr).unwrap();

        let broker = Arc::new(InMemoryMessageBroker::new());
        let router = MessageRouter::new(registry, broker);

        assert!(router.component_exists(&component_id));
        
        let nonexistent = ComponentId::new("nope");
        assert!(!router.component_exists(&nonexistent));
    }

    #[tokio::test]
    async fn test_component_count() {
        let registry = ComponentRegistry::new();
        let broker = Arc::new(InMemoryMessageBroker::new());
        let router = MessageRouter::new(registry.clone(), broker);

        assert_eq!(router.component_count().unwrap(), 0);

        registry.register(
            ComponentId::new("comp1"),
            ActorAddress::named("comp1")
        ).unwrap();
        assert_eq!(router.component_count().unwrap(), 1);
    }

    #[tokio::test]
    async fn test_try_broadcast_partial_failure() {
        let registry = ComponentRegistry::new();
        
        // Register only one component
        let comp1 = ComponentId::new("exists");
        registry.register(comp1.clone(), ActorAddress::named("exists")).unwrap();

        let broker = Arc::new(InMemoryMessageBroker::new());
        let router = MessageRouter::new(registry, broker);

        let targets = vec![
            comp1.clone(),
            ComponentId::new("nonexistent"),
        ];

        let results = router.try_broadcast_message(
            &targets,
            ComponentMessage::HealthCheck
        ).await;

        assert_eq!(results.len(), 2);
        assert!(results[0].1.is_ok()); // First component exists
        assert!(results[1].1.is_err()); // Second doesn't exist
    }
}
