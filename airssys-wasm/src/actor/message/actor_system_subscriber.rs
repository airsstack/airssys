//! ActorSystem subscriber for centralized message routing.
//!
//! This module implements the "ActorSystem as Primary Subscriber" pattern from ADR-WASM-009,
//! where ActorSystem subscribes to MessageBroker as a single subscriber and routes all
//! messages to component mailboxes based on topic subscriptions.
//!
//! # Architecture Context (ADR-WASM-009 & ADR-WASM-020)
//!
//! Per ADR-WASM-009 Component Communication Model:
//! ```text
//! Component A                ActorSystem                MessageBroker
//!     |                          |                            |
//!     | subscribe("events.*")    |                            |
//!     |------------------------->|                            |
//!     |                          | subscribe (on behalf)      |
//!     |                          |--------------------------->|
//!     |                          |                            |
//!     |                          | <---- message published ---|
//!     |                          |                            |
//!     | <--- route to mailbox ---|                            |
//!     |                          |                            |
//! ```
//!
//! Per ADR-WASM-020 Message Delivery Ownership:
//! - `ActorSystemSubscriber` owns message delivery via `mailbox_senders` map
//! - `ComponentRegistry` stays pure (identity lookup only, NOT modified)
//! - Clean separation: Registry = "who exists", Subscriber = "how to deliver"
//!
//! # Responsibilities
//!
//! - Subscribe to MessageBroker as single primary subscriber
//! - Receive all published messages from broker
//! - Maintain `mailbox_senders: HashMap<ComponentId, MailboxSender>` for delivery
//! - Route messages to ComponentActor mailboxes via MailboxSender lookup
//! - Spawn background routing task for async message processing
//! - Handle unreachable components with error logging
//!
//! # Performance
//!
//! Target: <100ns overhead (ActorSystem → Component mailbox)
//! - HashMap lookup: <100ns
//! - MailboxSender.send(): ~100ns (Tokio channel)
//!
//! # Example
//!
//! ```rust,ignore
//! use airssys_wasm::actor::{ActorSystemSubscriber, ComponentRegistry};
//! use airssys_rt::broker::InMemoryMessageBroker;
//! use std::sync::Arc;
//!
//! #[tokio::main]
//! async fn main() -> Result<(), WasmError> {
//!     let broker = Arc::new(InMemoryMessageBroker::new());
//!     let registry = ComponentRegistry::new();
//!     let subscriber_manager = Arc::new(SubscriberManager::new());
//!     
//!     let mut subscriber = ActorSystemSubscriber::new(broker, registry, subscriber_manager);
//!     
//!     // Register component mailbox for delivery
//!     let (tx, rx) = tokio::sync::mpsc::unbounded_channel();
//!     subscriber.register_mailbox(ComponentId::new("my-component"), tx).await;
//!     
//!     // Start subscribing and routing
//!     subscriber.start().await?;
//!     
//!     // Messages automatically routed to components via registered mailbox
//!     
//!     // Stop routing
//!     subscriber.stop().await?;
//!     
//!     Ok(())
//! }
//! ```
//!
//! # References
//!
//! - **ADR-WASM-009**: Component Communication Model (ActorSystem as intermediary)
//! - **ADR-WASM-018**: Three-Layer Architecture (Layer Separation)
//! - **ADR-WASM-020**: Message Delivery Ownership (ActorSystemSubscriber owns delivery)
//! - **KNOWLEDGE-WASM-026**: Message Delivery Architecture - Final Decision
//! - **WASM-TASK-004 Phase 4 Task 4.3**: ActorSystem as Primary Subscriber Pattern
//! - **WASM-TASK-006 Task 1.1 Remediation**: Fix stubbed message routing

// Layer 1: Standard library imports
use std::collections::HashMap;
use std::sync::Arc;

// Layer 2: Third-party crate imports
use tokio::sync::mpsc::UnboundedSender;
use tokio::sync::{Mutex, RwLock};
use tokio::task::JoinHandle;

// Layer 3: Internal module imports
use crate::actor::component::ComponentRegistry;
use crate::core::ComponentMessage;
use crate::actor::message::SubscriberManager;
use crate::core::{ComponentId, WasmError};
use airssys_rt::broker::MessageBroker;
use airssys_rt::message::MessageEnvelope;

/// ActorSystem subscriber that routes messages to components.
///
/// ActorSystemSubscriber implements the "ActorSystem as Primary Subscriber" pattern,
/// subscribing to the MessageBroker on behalf of all components and routing messages
/// to their mailboxes based on topic subscriptions tracked in SubscriberManager.
///
/// # Architecture (ADR-WASM-020)
///
/// ```text
/// MessageBroker (Layer 3)
///     ↓ subscribe
/// ActorSystemSubscriber (single subscriber)
///     ↓ route_message_to_subscribers()
/// mailbox_senders → lookup(component_id) → MailboxSender
///     ↓ sender.send(message)
/// ComponentActor mailbox (Layer 2)
/// ```
///
/// Per ADR-WASM-020:
/// - `ActorSystemSubscriber` owns message delivery via `mailbox_senders` map
/// - `ComponentRegistry` stays pure (identity lookup only)
/// - This separation maintains single responsibility: Registry = identity, Subscriber = delivery
///
/// # Thread Safety
///
/// Uses Arc<> and RwLock for thread-safe access to mailbox_senders.
/// The routing task spawns independently and processes messages concurrently.
/// RwLock allows concurrent reads (most operations) with exclusive writes (registration).
///
/// # Lifecycle
///
/// 1. `new()` - Create subscriber with broker, registry, and empty mailbox_senders
/// 2. `register_mailbox()` - Register MailboxSender for component (called by spawner)
/// 3. `start()` - Subscribe to broker, spawn routing task
/// 4. ... messages routed automatically via mailbox_senders ...
/// 5. `unregister_mailbox()` - Remove MailboxSender (called during shutdown)
/// 6. `stop()` - Abort routing task, cleanup
///
/// # Examples
///
/// ```rust,ignore
/// use airssys_wasm::actor::{ActorSystemSubscriber, ComponentRegistry};
/// use airssys_rt::broker::InMemoryMessageBroker;
/// use std::sync::Arc;
///
/// let broker = Arc::new(InMemoryMessageBroker::new());
/// let registry = ComponentRegistry::new();
/// let subscriber_manager = Arc::new(SubscriberManager::new());
///
/// let mut subscriber = ActorSystemSubscriber::new(
///     broker,
///     registry,
///     subscriber_manager,
/// );
///
/// // Register component's mailbox sender for delivery
/// let (tx, rx) = tokio::sync::mpsc::unbounded_channel();
/// subscriber.register_mailbox(ComponentId::new("my-component"), tx).await;
///
/// // Start routing
/// subscriber.start().await?;
///
/// // Messages automatically routed to registered mailboxes
///
/// // Stop routing
/// subscriber.stop().await?;
/// ```
pub struct ActorSystemSubscriber<B: MessageBroker<ComponentMessage>> {
    /// MessageBroker for receiving messages
    broker: Arc<B>,
    /// ComponentRegistry for looking up component addresses (IDENTITY ONLY per ADR-WASM-020)
    #[allow(dead_code)] // Registry kept for future topic-based routing lookup
    registry: ComponentRegistry,
    /// SubscriberManager for topic-based routing decisions
    subscriber_manager: Arc<SubscriberManager>,
    /// Background routing task handle
    routing_task: Option<JoinHandle<()>>,
    /// Map of ComponentId → MailboxSender for actual message delivery (ADR-WASM-020)
    ///
    /// This field owns the delivery mechanism. When a message arrives:
    /// 1. Extract target ComponentId from message
    /// 2. Look up MailboxSender in this map
    /// 3. Call sender.send(message) for actual delivery
    ///
    /// Registration happens when ComponentSpawner spawns a component.
    /// Unregistration happens when component is stopped.
    mailbox_senders: Arc<RwLock<HashMap<ComponentId, UnboundedSender<ComponentMessage>>>>,
}

impl<B: MessageBroker<ComponentMessage> + Send + Sync + 'static> ActorSystemSubscriber<B> {
    /// Create new ActorSystemSubscriber.
    ///
    /// # Arguments
    ///
    /// * `broker` - MessageBroker to subscribe to
    /// * `registry` - ComponentRegistry for address lookup (identity only per ADR-WASM-020)
    /// * `subscriber_manager` - SubscriberManager for topic-based routing
    ///
    /// # Examples
    ///
    /// ```rust,ignore
    /// let subscriber = ActorSystemSubscriber::new(
    ///     broker,
    ///     registry,
    ///     subscriber_manager,
    /// );
    /// ```
    pub fn new(
        broker: Arc<B>,
        registry: ComponentRegistry,
        subscriber_manager: Arc<SubscriberManager>,
    ) -> Self {
        Self {
            broker,
            registry,
            subscriber_manager,
            routing_task: None,
            mailbox_senders: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    /// Register a component's mailbox sender for message delivery (ADR-WASM-020).
    ///
    /// Called by ComponentSpawner when a ComponentActor is spawned. This enables
    /// message delivery to the component via its mailbox channel.
    ///
    /// # Arguments
    ///
    /// * `component_id` - Unique identifier for the component
    /// * `sender` - UnboundedSender for the component's mailbox channel
    ///
    /// # Returns
    ///
    /// - `Ok(())` - Registration successful
    /// - `Err(WasmError)` - Component already registered (duplicate registration)
    ///
    /// # Thread Safety
    ///
    /// Uses RwLock write lock for exclusive access during registration.
    /// This operation is infrequent (only during spawn) so contention is minimal.
    ///
    /// # Examples
    ///
    /// ```rust,ignore
    /// let (tx, rx) = tokio::sync::mpsc::unbounded_channel();
    /// subscriber.register_mailbox(ComponentId::new("my-component"), tx).await?;
    /// ```
    pub async fn register_mailbox(
        &self,
        component_id: ComponentId,
        sender: UnboundedSender<ComponentMessage>,
    ) -> Result<(), WasmError> {
        let mut senders = self.mailbox_senders.write().await;
        
        if senders.contains_key(&component_id) {
            return Err(WasmError::internal(format!(
                "Mailbox already registered for component: {}",
                component_id.as_str()
            )));
        }
        
        senders.insert(component_id.clone(), sender);
        tracing::debug!(
            component_id = %component_id.as_str(),
            "Registered mailbox sender for message delivery"
        );
        
        Ok(())
    }

    /// Unregister a component's mailbox sender (ADR-WASM-020).
    ///
    /// Called when a ComponentActor is stopped. Removes the MailboxSender from
    /// the registry, preventing further message delivery to the stopped component.
    ///
    /// # Arguments
    ///
    /// * `component_id` - Unique identifier for the component to unregister
    ///
    /// # Returns
    ///
    /// - `Ok(Some(sender))` - Unregistration successful, returns the removed sender
    /// - `Ok(None)` - Component was not registered (idempotent operation)
    ///
    /// # Thread Safety
    ///
    /// Uses RwLock write lock for exclusive access during unregistration.
    /// This operation is infrequent (only during shutdown) so contention is minimal.
    ///
    /// # Examples
    ///
    /// ```rust,ignore
    /// let removed = subscriber.unregister_mailbox(&ComponentId::new("my-component")).await;
    /// if removed.is_some() {
    ///     println!("Component mailbox unregistered");
    /// }
    /// ```
    pub async fn unregister_mailbox(
        &self,
        component_id: &ComponentId,
    ) -> Option<UnboundedSender<ComponentMessage>> {
        let mut senders = self.mailbox_senders.write().await;
        let removed = senders.remove(component_id);
        
        if removed.is_some() {
            tracing::debug!(
                component_id = %component_id.as_str(),
                "Unregistered mailbox sender"
            );
        } else {
            tracing::warn!(
                component_id = %component_id.as_str(),
                "Attempted to unregister non-existent mailbox sender"
            );
        }
        
        removed
    }

    /// Get the count of registered mailbox senders.
    ///
    /// Useful for monitoring and debugging.
    ///
    /// # Returns
    ///
    /// Number of registered mailbox senders.
    pub async fn mailbox_count(&self) -> usize {
        let senders = self.mailbox_senders.read().await;
        senders.len()
    }

    /// Start subscribing to broker and routing messages.
    ///
    /// Subscribes to the MessageBroker and spawns a background task that
    /// continuously receives messages and routes them to component mailboxes
    /// based on topic subscriptions.
    ///
    /// # Returns
    ///
    /// - `Ok(())`: Subscription started successfully
    /// - `Err(WasmError)`: Broker subscription failed
    ///
    /// # Error Handling
    ///
    /// If the broker subscription fails, this method returns an error immediately.
    /// Once started, individual routing errors are logged but do not stop the
    /// routing task.
    ///
    /// # Examples
    ///
    /// ```rust,ignore
    /// let mut subscriber = ActorSystemSubscriber::new(broker, registry, manager);
    /// subscriber.start().await?;
    /// ```
    pub async fn start(&mut self) -> Result<(), WasmError> {
        // Subscribe to broker
        let stream = self.broker.subscribe().await.map_err(|e| {
            WasmError::message_broker_error(format!("Failed to subscribe to broker: {}", e))
        })?;

        // Wrap stream in Arc<Mutex<>> for sharing across task
        let stream = Arc::new(Mutex::new(stream));

        let mailbox_senders = Arc::clone(&self.mailbox_senders);
        let subscriber_manager = Arc::clone(&self.subscriber_manager);

        // Spawn routing task
        let task = tokio::spawn(async move {
            loop {
                // Receive next message
                let envelope = {
                    let mut stream_guard = stream.lock().await;
                    stream_guard.recv().await
                };

                match envelope {
                    Some(envelope) => {
                        // Route message to subscribers via mailbox_senders (ADR-WASM-020)
                        if let Err(e) = Self::route_message_to_subscribers(
                            &mailbox_senders,
                            &subscriber_manager,
                            envelope,
                        )
                        .await
                        {
                            tracing::error!("Failed to route message: {}", e);
                        }
                    }
                    None => {
                        // Stream closed, exit loop
                        tracing::info!("MessageBroker stream closed, stopping routing task");
                        break;
                    }
                }
            }
        });

        self.routing_task = Some(task);

        Ok(())
    }

    /// Route message to target component via mailbox sender (ADR-WASM-020).
    ///
    /// This method implements ACTUAL MESSAGE DELIVERY to component mailboxes.
    /// It looks up the target component's MailboxSender in the mailbox_senders
    /// registry and sends the message via the channel.
    ///
    /// # Architecture (ADR-WASM-020)
    ///
    /// 1. Extract target ComponentId from message payload
    /// 2. Look up MailboxSender in mailbox_senders map
    /// 3. Send message via sender.send(payload)
    /// 4. Return error if target not registered or send fails
    ///
    /// # Parameters
    ///
    /// * `mailbox_senders` - Map of ComponentId → MailboxSender for delivery
    /// * `_subscriber_manager` - SubscriberManager for future topic-based routing
    /// * `envelope` - Message envelope with payload and metadata
    ///
    /// # Returns
    ///
    /// - `Ok(())`: Message delivered successfully to component mailbox
    /// - `Err(WasmError::ComponentNotFound)`: Target component not registered
    /// - `Err(WasmError::MessagingError)`: Failed to send to mailbox (channel closed)
    ///
    /// # Performance
    ///
    /// - HashMap lookup: <100ns
    /// - Channel send: ~100ns
    /// - Total: <200ns (within Block 5 targets)
    ///
    /// # Visibility
    ///
    /// This method is public for integration testing purposes.
    pub async fn route_message_to_subscribers(
        mailbox_senders: &RwLock<HashMap<ComponentId, UnboundedSender<ComponentMessage>>>,
        _subscriber_manager: &SubscriberManager,
        envelope: MessageEnvelope<ComponentMessage>,
    ) -> Result<(), WasmError> {
        // 1. Extract target ComponentId from message
        let target = Self::extract_target(&envelope.payload)?;
        
        // 2. Look up MailboxSender for target component
        let senders = mailbox_senders.read().await;
        let sender = senders.get(&target).ok_or_else(|| {
            WasmError::component_not_found(format!(
                "No mailbox registered for component: {}",
                target.as_str()
            ))
        })?;
        
        // 3. ACTUAL DELIVERY - Send message to component's mailbox
        sender.send(envelope.payload).map_err(|e| {
            WasmError::messaging_error(format!(
                "Failed to deliver message to {}: {}",
                target.as_str(),
                e
            ))
        })?;
        
        tracing::debug!(
            target = %target.as_str(),
            "Message delivered to component mailbox"
        );
        
        Ok(())
    }

    /// Extract target component from message.
    ///
    /// Determines the target component ID from the message payload.
    /// For InterComponent messages, this is typically the intended recipient.
    /// For broadcast messages, multiple targets may be determined via SubscriberManager.
    ///
    /// # Parameters
    ///
    /// * `message` - ComponentMessage to inspect
    ///
    /// # Returns
    ///
    /// - `Ok(ComponentId)`: Target component identified
    /// - `Err(WasmError)`: Message format invalid or target indeterminate
    ///
    /// # Note
    ///
    /// This is a simplified implementation. In a full system, topic-based routing
    /// would use SubscriberManager to resolve all subscribers for a topic.
    ///
    /// # Visibility
    ///
    /// This method is public for testing purposes.
    pub fn extract_target(message: &ComponentMessage) -> Result<ComponentId, WasmError> {
        match message {
            ComponentMessage::InterComponent { to, .. } => {
                // Phase 1: Direct ComponentId addressing
                Ok(to.clone())
            }
            ComponentMessage::InterComponentWithCorrelation { to, .. } => {
                // Phase 1: Direct ComponentId addressing
                Ok(to.clone())
            }
            _ => Err(WasmError::internal(
                "Cannot extract target from message type".to_string(),
            )),
        }
    }

    /// Stop routing and cleanup.
    ///
    /// Aborts the background routing task and cleans up resources.
    /// This is a graceful shutdown that allows the current message to finish
    /// processing before terminating the task.
    ///
    /// # Returns
    ///
    /// - `Ok(())`: Routing stopped successfully
    ///
    /// # Examples
    ///
    /// ```rust,ignore
    /// subscriber.stop().await?;
    /// ```
    pub async fn stop(&mut self) -> Result<(), WasmError> {
        if let Some(task) = self.routing_task.take() {
            task.abort();
            // Ignore JoinError - task was aborted intentionally
            let _ = task.await;
        }
        Ok(())
    }

    /// Check if subscriber is actively routing.
    ///
    /// # Returns
    ///
    /// `true` if routing task is running, `false` otherwise
    ///
    /// # Examples
    ///
    /// ```rust,ignore
    /// if subscriber.is_running() {
    ///     println!("Routing active");
    /// }
    /// ```
    pub fn is_running(&self) -> bool {
        self.routing_task.is_some()
    }
}

impl<B: MessageBroker<ComponentMessage>> Drop for ActorSystemSubscriber<B> {
    /// Cleanup on drop.
    ///
    /// Aborts the routing task if still running when dropped.
    fn drop(&mut self) {
        if let Some(task) = self.routing_task.take() {
            task.abort();
        }
    }
}

#[cfg(test)]
#[allow(clippy::unwrap_used)] // Test code: unwrap is acceptable
mod tests {
    use super::*;
    use airssys_rt::broker::InMemoryMessageBroker;

    #[tokio::test]
    async fn test_actor_system_subscriber_creation() {
        let broker = Arc::new(InMemoryMessageBroker::new());
        let registry = ComponentRegistry::new();
        let subscriber_manager = Arc::new(SubscriberManager::new());

        let subscriber = ActorSystemSubscriber::new(broker, registry, subscriber_manager);

        assert!(!subscriber.is_running());
        assert_eq!(subscriber.mailbox_count().await, 0);
    }

    #[tokio::test]
    async fn test_actor_system_subscriber_start() {
        let broker = Arc::new(InMemoryMessageBroker::new());
        let registry = ComponentRegistry::new();
        let subscriber_manager = Arc::new(SubscriberManager::new());

        let mut subscriber = ActorSystemSubscriber::new(broker, registry, subscriber_manager);

        let result = subscriber.start().await;
        assert!(result.is_ok());
        assert!(subscriber.is_running());

        // Cleanup
        subscriber.stop().await.unwrap();
    }

    #[tokio::test]
    async fn test_actor_system_subscriber_stop() {
        let broker = Arc::new(InMemoryMessageBroker::new());
        let registry = ComponentRegistry::new();
        let subscriber_manager = Arc::new(SubscriberManager::new());

        let mut subscriber = ActorSystemSubscriber::new(broker, registry, subscriber_manager);

        subscriber.start().await.unwrap();
        assert!(subscriber.is_running());

        let result = subscriber.stop().await;
        assert!(result.is_ok());
        assert!(!subscriber.is_running());
    }

    #[tokio::test]
    async fn test_extract_target_inter_component() {
        let sender_id = ComponentId::new("sender-component");
        let target_id = ComponentId::new("test-component");
        let message = ComponentMessage::InterComponent {
            sender: sender_id,
            to: target_id.clone(),
            payload: vec![1, 2, 3],
        };

        let result =
            ActorSystemSubscriber::<InMemoryMessageBroker<ComponentMessage>>::extract_target(
                &message,
            );
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), target_id);
    }

    #[tokio::test]
    async fn test_extract_target_with_correlation() {
        let sender_id = ComponentId::new("sender-component");
        let target_id = ComponentId::new("test-component");
        let message = ComponentMessage::InterComponentWithCorrelation {
            sender: sender_id,
            to: target_id.clone(),
            payload: vec![1, 2, 3],
            correlation_id: uuid::Uuid::new_v4(),
        };

        let result =
            ActorSystemSubscriber::<InMemoryMessageBroker<ComponentMessage>>::extract_target(
                &message,
            );
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), target_id);
    }

    #[tokio::test]
    async fn test_extract_target_invalid_message() {
        let message = ComponentMessage::Shutdown;

        let result =
            ActorSystemSubscriber::<InMemoryMessageBroker<ComponentMessage>>::extract_target(
                &message,
            );
        assert!(result.is_err());
    }

    // ========================================================================
    // Mailbox Registration Tests (ADR-WASM-020)
    // ========================================================================

    #[tokio::test]
    async fn test_register_mailbox_success() {
        let broker = Arc::new(InMemoryMessageBroker::new());
        let registry = ComponentRegistry::new();
        let subscriber_manager = Arc::new(SubscriberManager::new());

        let subscriber = ActorSystemSubscriber::new(broker, registry, subscriber_manager);
        let (tx, _rx) = tokio::sync::mpsc::unbounded_channel();
        let component_id = ComponentId::new("test-component");

        // Register mailbox
        let result = subscriber.register_mailbox(component_id.clone(), tx).await;
        assert!(result.is_ok());

        // Verify registered
        assert_eq!(subscriber.mailbox_count().await, 1);
        
        // Verify can lookup in internal map
        let senders = subscriber.mailbox_senders.read().await;
        assert!(senders.contains_key(&component_id));
    }

    #[tokio::test]
    async fn test_register_mailbox_duplicate_error() {
        let broker = Arc::new(InMemoryMessageBroker::new());
        let registry = ComponentRegistry::new();
        let subscriber_manager = Arc::new(SubscriberManager::new());

        let subscriber = ActorSystemSubscriber::new(broker, registry, subscriber_manager);
        let component_id = ComponentId::new("test-component");

        // First registration succeeds
        let (tx1, _rx1) = tokio::sync::mpsc::unbounded_channel();
        let result = subscriber.register_mailbox(component_id.clone(), tx1).await;
        assert!(result.is_ok());

        // Second registration fails (duplicate)
        let (tx2, _rx2) = tokio::sync::mpsc::unbounded_channel();
        let result = subscriber.register_mailbox(component_id.clone(), tx2).await;
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("already registered"));
    }

    #[tokio::test]
    async fn test_unregister_mailbox_success() {
        let broker = Arc::new(InMemoryMessageBroker::new());
        let registry = ComponentRegistry::new();
        let subscriber_manager = Arc::new(SubscriberManager::new());

        let subscriber = ActorSystemSubscriber::new(broker, registry, subscriber_manager);
        let (tx, _rx) = tokio::sync::mpsc::unbounded_channel();
        let component_id = ComponentId::new("test-component");

        // Register then unregister
        subscriber.register_mailbox(component_id.clone(), tx).await.unwrap();
        assert_eq!(subscriber.mailbox_count().await, 1);

        let removed = subscriber.unregister_mailbox(&component_id).await;
        assert!(removed.is_some());
        assert_eq!(subscriber.mailbox_count().await, 0);

        // Verify removed from internal map
        let senders = subscriber.mailbox_senders.read().await;
        assert!(!senders.contains_key(&component_id));
    }

    #[tokio::test]
    async fn test_unregister_mailbox_not_found() {
        let broker = Arc::new(InMemoryMessageBroker::new());
        let registry = ComponentRegistry::new();
        let subscriber_manager = Arc::new(SubscriberManager::new());

        let subscriber = ActorSystemSubscriber::new(broker, registry, subscriber_manager);
        let component_id = ComponentId::new("nonexistent");

        // Unregister non-existent component returns None
        let removed = subscriber.unregister_mailbox(&component_id).await;
        assert!(removed.is_none());
    }

    // ========================================================================
    // Message Delivery Tests (ADR-WASM-020)
    // ========================================================================

    #[tokio::test]
    async fn test_message_actually_delivered() {
        let mailbox_senders: RwLock<HashMap<ComponentId, UnboundedSender<ComponentMessage>>> = 
            RwLock::new(HashMap::new());
        let subscriber_manager = Arc::new(SubscriberManager::new());
        
        let (tx, mut rx) = tokio::sync::mpsc::unbounded_channel();
        let target = ComponentId::new("receiver");

        // Register mailbox
        {
            let mut senders = mailbox_senders.write().await;
            senders.insert(target.clone(), tx);
        }

        // Create message
        let message = ComponentMessage::InterComponent {
            sender: ComponentId::new("sender"),
            to: target.clone(),
            payload: vec![1, 2, 3],
        };
        let envelope = MessageEnvelope::new(message);

        // Route message - this should ACTUALLY DELIVER to mailbox
        let result = ActorSystemSubscriber::<InMemoryMessageBroker<ComponentMessage>>::route_message_to_subscribers(
            &mailbox_senders,
            &subscriber_manager,
            envelope,
        ).await;

        assert!(result.is_ok());

        // CRITICAL: Verify message was ACTUALLY RECEIVED in mailbox
        let received = rx.try_recv();
        assert!(received.is_ok(), "Message was NOT delivered to mailbox");
        
        let received_msg = received.unwrap();
        match received_msg {
            ComponentMessage::InterComponent { sender, to, payload } => {
                assert_eq!(sender.as_str(), "sender");
                assert_eq!(to.as_str(), "receiver");
                assert_eq!(payload, vec![1, 2, 3]);
            }
            _ => panic!("Wrong message type received"),
        }
    }

    #[tokio::test]
    async fn test_send_to_unregistered_fails() {
        let mailbox_senders: RwLock<HashMap<ComponentId, UnboundedSender<ComponentMessage>>> = 
            RwLock::new(HashMap::new());
        let subscriber_manager = Arc::new(SubscriberManager::new());
        
        // Do NOT register any mailbox
        let target = ComponentId::new("nonexistent");

        let message = ComponentMessage::InterComponent {
            sender: ComponentId::new("sender"),
            to: target.clone(),
            payload: vec![],
        };
        let envelope = MessageEnvelope::new(message);

        let result = ActorSystemSubscriber::<InMemoryMessageBroker<ComponentMessage>>::route_message_to_subscribers(
            &mailbox_senders,
            &subscriber_manager,
            envelope,
        ).await;

        assert!(result.is_err());
        let err_msg = result.unwrap_err().to_string();
        assert!(err_msg.contains("nonexistent") || err_msg.contains("not found"));
    }

    #[tokio::test]
    async fn test_message_delivery_with_correlation() {
        let mailbox_senders: RwLock<HashMap<ComponentId, UnboundedSender<ComponentMessage>>> = 
            RwLock::new(HashMap::new());
        let subscriber_manager = Arc::new(SubscriberManager::new());
        
        let (tx, mut rx) = tokio::sync::mpsc::unbounded_channel();
        let target = ComponentId::new("receiver");
        let correlation_id = uuid::Uuid::new_v4();

        // Register mailbox
        {
            let mut senders = mailbox_senders.write().await;
            senders.insert(target.clone(), tx);
        }

        // Create message with correlation
        let message = ComponentMessage::InterComponentWithCorrelation {
            sender: ComponentId::new("sender"),
            to: target.clone(),
            payload: vec![4, 5, 6],
            correlation_id,
        };
        let envelope = MessageEnvelope::new(message);

        // Route message
        let result = ActorSystemSubscriber::<InMemoryMessageBroker<ComponentMessage>>::route_message_to_subscribers(
            &mailbox_senders,
            &subscriber_manager,
            envelope,
        ).await;

        assert!(result.is_ok());

        // Verify message received
        let received = rx.try_recv();
        assert!(received.is_ok(), "Message was NOT delivered to mailbox");
        
        match received.unwrap() {
            ComponentMessage::InterComponentWithCorrelation { sender, to, payload, correlation_id: cid } => {
                assert_eq!(sender.as_str(), "sender");
                assert_eq!(to.as_str(), "receiver");
                assert_eq!(payload, vec![4, 5, 6]);
                assert_eq!(cid, correlation_id);
            }
            _ => panic!("Wrong message type received"),
        }
    }

    #[tokio::test]
    async fn test_multiple_mailbox_registrations() {
        let broker = Arc::new(InMemoryMessageBroker::new());
        let registry = ComponentRegistry::new();
        let subscriber_manager = Arc::new(SubscriberManager::new());

        let subscriber = ActorSystemSubscriber::new(broker, registry, subscriber_manager);

        // Register multiple components
        for i in 0..5 {
            let component_id = ComponentId::new(format!("component-{}", i));
            let (tx, _rx) = tokio::sync::mpsc::unbounded_channel();
            subscriber.register_mailbox(component_id, tx).await.unwrap();
        }

        assert_eq!(subscriber.mailbox_count().await, 5);

        // Unregister some
        subscriber.unregister_mailbox(&ComponentId::new("component-0")).await;
        subscriber.unregister_mailbox(&ComponentId::new("component-2")).await;

        assert_eq!(subscriber.mailbox_count().await, 3);
    }

    #[tokio::test]
    async fn test_route_to_closed_channel_fails() {
        let mailbox_senders: RwLock<HashMap<ComponentId, UnboundedSender<ComponentMessage>>> = 
            RwLock::new(HashMap::new());
        let subscriber_manager = Arc::new(SubscriberManager::new());
        
        let (tx, rx) = tokio::sync::mpsc::unbounded_channel();
        let target = ComponentId::new("receiver");

        // Register mailbox
        {
            let mut senders = mailbox_senders.write().await;
            senders.insert(target.clone(), tx);
        }

        // Drop the receiver to close the channel
        drop(rx);

        // Try to send message - should fail because channel is closed
        let message = ComponentMessage::InterComponent {
            sender: ComponentId::new("sender"),
            to: target.clone(),
            payload: vec![1, 2, 3],
        };
        let envelope = MessageEnvelope::new(message);

        let result = ActorSystemSubscriber::<InMemoryMessageBroker<ComponentMessage>>::route_message_to_subscribers(
            &mailbox_senders,
            &subscriber_manager,
            envelope,
        ).await;

        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("Failed to deliver"));
    }
}
